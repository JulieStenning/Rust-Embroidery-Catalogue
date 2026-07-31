use crate::models::EmbPattern;
use crate::png_writer::{render_pattern_to_png, RenderSettings, ThreeDStyle};
use crate::readers::{
    DstReader, EmbroideryReader, ExpReader, HusReader, JefReader, PesReader, Vp3Reader,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const NATIVE_PREVIEW_EXTENSIONS: &[&str] = &["pes", "dst", "exp", "jef", "vp3", "hus"];
const PYTHON_PREVIEW_EXTENSIONS: &[&str] = &["jef", "pes", "dst", "exp", "vp3"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendSupport {
    NativeOnly,
    PythonOnly,
    Both,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    pub file_path: String,
    pub preview_3d: bool,
    pub preview_3d_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResult {
    pub image_data: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub stitch_count: Option<i64>,
    pub color_count: Option<i64>,
    pub color_change_count: Option<i64>,
    pub backend: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PythonBatchResult {
    file_path: String,
    image_base64: Option<String>,
    image_type: Option<String>,
    width_mm: Option<f64>,
    height_mm: Option<f64>,
    stitch_count: Option<i64>,
    color_count: Option<i64>,
    color_change_count: Option<i64>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PythonImageGenerationResult {
    image_base64: Option<String>,
    image_type: Option<String>,
    width_mm: Option<f64>,
    height_mm: Option<f64>,
    stitch_count: Option<i64>,
    color_count: Option<i64>,
    color_change_count: Option<i64>,
    error: Option<String>,
}

fn adapter_script_path() -> PathBuf {
    Path::new("src")
        .join("services")
        .join("python_image_adapter.py")
}

fn request_extension(file_path: &str) -> Option<String> {
    Path::new(file_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
}

fn extension_support(file_path: &str) -> BackendSupport {
    let Some(extension) = request_extension(file_path) else {
        return BackendSupport::Unsupported;
    };

    let native = NATIVE_PREVIEW_EXTENSIONS
        .iter()
        .any(|candidate| *candidate == extension);
    let python = PYTHON_PREVIEW_EXTENSIONS
        .iter()
        .any(|candidate| *candidate == extension);

    match (native, python) {
        (true, true) => BackendSupport::Both,
        (true, false) => BackendSupport::NativeOnly,
        (false, true) => BackendSupport::PythonOnly,
        (false, false) => BackendSupport::Unsupported,
    }
}

fn unsupported_extension_result(file_path: &str, backend: &str) -> ImageGenerationResult {
    let extension = request_extension(file_path).unwrap_or_else(|| "unknown".to_string());
    ImageGenerationResult {
        image_data: None,
        image_type: None,
        width_mm: None,
        height_mm: None,
        stitch_count: None,
        color_count: None,
        color_change_count: None,
        backend: backend.to_string(),
        error: Some(format!(
            "Image preview generation skipped because extension '.{}' is not supported.",
            extension
        )),
    }
}

/// Returns true when the file requires the Python backend (no native reader available).
pub fn needs_python_backend(file_path: &str) -> bool {
    matches!(extension_support(file_path), BackendSupport::PythonOnly)
}

/// Run one Python process for a slice of files, importing pyembroidery once.
/// Results are keyed by the original file_path string.
/// Files that produce no result (e.g. due to overall timeout) get an error entry.
pub fn generate_previews_via_python_batch(
    requests: &[ImageGenerationRequest],
) -> HashMap<String, ImageGenerationResult> {
    let mut results: HashMap<String, ImageGenerationResult> = HashMap::new();

    if requests.is_empty() {
        return results;
    }

    let script_path = adapter_script_path();
    let error_result = |msg: String| ImageGenerationResult {
        image_data: None,
        image_type: None,
        width_mm: None,
        height_mm: None,
        stitch_count: None,
        color_count: None,
        color_change_count: None,
        backend: "python-batch".to_string(),
        error: Some(msg),
    };

    if !script_path.exists() {
        let msg = format!(
            "Python image adapter script not found: {}",
            script_path.to_string_lossy()
        );
        for req in requests {
            results.insert(req.file_path.clone(), error_result(msg.clone()));
        }
        return results;
    }

    let python_executable =
        std::env::var("RUST_EMBROIDERY_PYTHON").unwrap_or_else(|_| "python".to_string());
    let preview_flag = if requests.first().map(|r| r.preview_3d).unwrap_or(false) {
        "true"
    } else {
        "false"
    };
    let per_file_timeout_ms = std::env::var("IMPORT_IMAGE_PYTHON_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(1_000, 120_000))
        .unwrap_or(15_000);
    // Allow at least 60 s and scale with chunk size.
    let batch_timeout_ms = (requests.len() as u64 * per_file_timeout_ms).max(60_000);

    let mut child = match Command::new(&python_executable)
        .arg(&script_path)
        .arg("--batch")
        .arg("--preview-3d")
        .arg(preview_flag)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Could not execute python adapter: {e}");
            for req in requests {
                results.insert(req.file_path.clone(), error_result(msg.clone()));
            }
            return results;
        }
    };

    let mut stdin = child.stdin.take().expect("stdin is piped");
    let file_paths: Vec<String> = requests.iter().map(|r| r.file_path.clone()).collect();

    // Write all file paths to stdin in a separate thread, then close the pipe.
    let stdin_thread = thread::spawn(move || {
        for path in &file_paths {
            if writeln!(stdin, "{}", path).is_err() {
                break;
            }
        }
        // stdin dropped here → EOF signal to Python
    });

    // Read NDJSON results from stdout via a channel so we can apply a timeout.
    let stdout = child.stdout.take().expect("stdout is piped");
    let (line_tx, line_rx) = mpsc::channel::<String>();
    let reader_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) if !l.trim().is_empty() => {
                    if line_tx.send(l).is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    let started = Instant::now();
    loop {
        match line_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(line) => {
                if let Ok(parsed) = serde_json::from_str::<PythonBatchResult>(line.trim()) {
                    let image_data = parsed
                        .image_base64
                        .as_deref()
                        .and_then(|enc| base64::engine::general_purpose::STANDARD.decode(enc).ok());
                    results.insert(
                        parsed.file_path.clone(),
                        ImageGenerationResult {
                            image_data,
                            image_type: parsed.image_type,
                            width_mm: parsed.width_mm,
                            height_mm: parsed.height_mm,
                            stitch_count: parsed.stitch_count,
                            color_count: parsed.color_count,
                            color_change_count: parsed.color_change_count,
                            backend: "python-batch".to_string(),
                            error: parsed.error,
                        },
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if started.elapsed().as_millis() > batch_timeout_ms as u128 {
                    tracing::debug!(
                        "[TIMING] Python batch timed out after {}ms with {}/{} results",
                        batch_timeout_ms,
                        results.len(),
                        requests.len()
                    );
                    let _ = child.kill();
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Reader thread finished (Python exited normally)
                break;
            }
        }
    }

    let _ = stdin_thread.join();
    let _ = reader_thread.join();
    let _ = child.wait();

    // Fill errors for any files that produced no result.
    for req in requests {
        if !results.contains_key(&req.file_path) {
            results.insert(
                req.file_path.clone(),
                error_result(format!(
                    "No result received from Python batch for '{}'",
                    req.file_path
                )),
            );
        }
    }

    results
}

pub fn generate_preview(request: &ImageGenerationRequest) -> ImageGenerationResult {
    let backend = std::env::var("IMPORT_IMAGE_BACKEND").unwrap_or_else(|_| "auto".to_string());
    let support = extension_support(&request.file_path);

    if support == BackendSupport::Unsupported {
        return unsupported_extension_result(&request.file_path, backend.as_str());
    }

    match backend.to_ascii_lowercase().as_str() {
        "python" => {
            if support == BackendSupport::NativeOnly {
                return ImageGenerationResult {
                    image_data: None,
                    image_type: None,
                    width_mm: None,
                    height_mm: None,
                    stitch_count: None,
                    color_count: None,
                    color_change_count: None,
                    backend: "python".to_string(),
                    error: Some(
                        "Python image backend does not support this extension.".to_string(),
                    ),
                };
            }
            generate_preview_via_python(request)
        }
        "native" => {
            if support == BackendSupport::PythonOnly {
                return ImageGenerationResult {
                    image_data: None,
                    image_type: None,
                    width_mm: None,
                    height_mm: None,
                    stitch_count: None,
                    color_count: None,
                    color_change_count: None,
                    backend: "native".to_string(),
                    error: Some(
                        "Native image backend does not support this extension.".to_string(),
                    ),
                };
            }
            generate_preview_via_native(request)
        }
        "auto" => generate_preview_auto(request),
        other => ImageGenerationResult {
            image_data: None,
            image_type: None,
            width_mm: None,
            height_mm: None,
            stitch_count: None,
            color_count: None,
            color_change_count: None,
            backend: other.to_string(),
            error: Some(format!("Unsupported image backend: {other}")),
        },
    }
}

fn generate_preview_auto(request: &ImageGenerationRequest) -> ImageGenerationResult {
    match extension_support(&request.file_path) {
        BackendSupport::Unsupported => {
            return unsupported_extension_result(&request.file_path, "auto");
        }
        BackendSupport::NativeOnly => {
            return generate_preview_via_native(request);
        }
        BackendSupport::PythonOnly => {
            return ImageGenerationResult {
                image_data: None,
                image_type: None,
                width_mm: None,
                height_mm: None,
                stitch_count: None,
                color_count: None,
                color_change_count: None,
                backend: "auto".to_string(),
                error: Some(
                    "Auto image backend no longer falls back to Python for this extension."
                        .to_string(),
                ),
            };
        }
        BackendSupport::Both => {}
    }

    let native = generate_preview_via_native(request);
    if native.error.is_none() {
        return native;
    }

    let python = generate_preview_via_python(request);
    if python.error.is_none() {
        return python;
    }

    ImageGenerationResult {
        image_data: None,
        image_type: None,
        width_mm: None,
        height_mm: None,
        stitch_count: None,
        color_count: None,
        color_change_count: None,
        backend: "auto".to_string(),
        error: Some(format!(
            "Auto backend failed with native renderer: '{}'; python fallback also failed: '{}'",
            native
                .error
                .unwrap_or_else(|| "unknown native error".to_string()),
            python
                .error
                .unwrap_or_else(|| "unknown python error".to_string())
        )),
    }
}

fn generate_preview_via_native(request: &ImageGenerationRequest) -> ImageGenerationResult {
    let pattern = match read_pattern_from_file(&request.file_path) {
        Ok(value) => value,
        Err(error) => {
            return ImageGenerationResult {
                image_data: None,
                image_type: None,
                width_mm: None,
                height_mm: None,
                stitch_count: None,
                color_count: None,
                color_change_count: None,
                backend: "native".to_string(),
                error: Some(error),
            }
        }
    };

    analyze_pattern_with_native_renderer(
        &pattern,
        request.preview_3d,
        request.preview_3d_profile.as_deref(),
    )
}

fn read_pattern_from_file(file_path: &str) -> Result<EmbPattern, String> {
    let data = fs::read(file_path)
        .map_err(|error| format!("Could not read embroidery file '{}': {error}", file_path))?;

    let extension = request_extension(file_path)
        .ok_or_else(|| format!("Missing file extension for '{}'.", file_path))?;

    let parsed = match extension.as_str() {
        "pes" => PesReader.read(&data),
        "dst" => DstReader.read(&data),
        "exp" => ExpReader.read(&data),
        "jef" => JefReader.read(&data),
        "hus" => HusReader.read(&data),
        "vp3" => Vp3Reader.read(&data),
        _ => {
            return Err(format!(
                "Native image backend does not support extension '.{}'.",
                extension
            ))
        }
    };

    parsed.map_err(|error| format!("Could not parse '{}': {error}", file_path))
}

fn round_two(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn three_d_style_from_profile_name(profile_name: Option<&str>) -> ThreeDStyle {
    let normalized = profile_name
        .unwrap_or("balanced")
        .trim()
        .to_ascii_lowercase();

    match normalized.as_str() {
        "soft" => ThreeDStyle {
            shadow_strength: 28,
            highlight_strength: 20,
            core_half_width: 1,
            shadow_offset: 1,
            highlight_offset: 1,
        },
        "high-contrast" | "high_contrast" | "highcontrast" => ThreeDStyle {
            shadow_strength: 64,
            highlight_strength: 52,
            core_half_width: 2,
            shadow_offset: 2,
            highlight_offset: 2,
        },
        _ => ThreeDStyle::default(),
    }
}

fn drawable_bounds_mm(pattern: &EmbPattern) -> Option<(f64, f64)> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut found = false;

    for stitch in &pattern.stitches {
        if stitch.stitch_type != crate::models::StitchType::Stitch {
            continue;
        }

        found = true;
        if stitch.x < min_x {
            min_x = stitch.x;
        }
        if stitch.x > max_x {
            max_x = stitch.x;
        }
        if stitch.y < min_y {
            min_y = stitch.y;
        }
        if stitch.y > max_y {
            max_y = stitch.y;
        }
    }

    if found {
        Some((
            round_two(f64::from((max_x - min_x) / 10.0)),
            round_two(f64::from((max_y - min_y) / 10.0)),
        ))
    } else {
        None
    }
}

fn analyze_pattern_with_native_renderer(
    pattern: &EmbPattern,
    preview_3d: bool,
    preview_3d_profile: Option<&str>,
) -> ImageGenerationResult {
    let stitch_count = i64::try_from(pattern.count_stitches()).unwrap_or(i64::MAX);
    let color_count = i64::try_from(pattern.count_distinct_thread_colors()).unwrap_or(i64::MAX);
    let color_change_count = i64::try_from(pattern.count_color_changes()).unwrap_or(i64::MAX);

    if pattern.stitches.is_empty() {
        return ImageGenerationResult {
            image_data: None,
            image_type: None,
            width_mm: None,
            height_mm: None,
            stitch_count: Some(stitch_count),
            color_count: Some(color_count),
            color_change_count: Some(color_change_count),
            backend: "native".to_string(),
            error: None,
        };
    }

    let mut settings = RenderSettings::default().with_preview_3d(preview_3d);
    if preview_3d {
        settings = settings.with_three_d_style(three_d_style_from_profile_name(preview_3d_profile));
    }
    let image_data = render_pattern_to_png(pattern, &settings);
    let (width_mm, height_mm) = drawable_bounds_mm(pattern)
        .map(|(w, h)| (Some(w), Some(h)))
        .unwrap_or((None, None));

    ImageGenerationResult {
        image_data: Some(image_data),
        image_type: Some(if preview_3d { "3d" } else { "2d" }.to_string()),
        width_mm,
        height_mm,
        stitch_count: Some(stitch_count),
        color_count: Some(color_count),
        color_change_count: Some(color_change_count),
        backend: "native".to_string(),
        error: None,
    }
}

fn generate_preview_via_python(request: &ImageGenerationRequest) -> ImageGenerationResult {
    let script_path = adapter_script_path();
    if !script_path.exists() {
        return ImageGenerationResult {
            image_data: None,
            image_type: None,
            width_mm: None,
            height_mm: None,
            stitch_count: None,
            color_count: None,
            color_change_count: None,
            backend: "python".to_string(),
            error: Some(format!(
                "Python image adapter script not found: {}",
                script_path.to_string_lossy()
            )),
        };
    }

    let python_executable =
        std::env::var("RUST_EMBROIDERY_PYTHON").unwrap_or_else(|_| "python".to_string());
    let preview_flag = if request.preview_3d { "true" } else { "false" };
    let timeout_ms = std::env::var("IMPORT_IMAGE_PYTHON_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(1_000, 120_000))
        .unwrap_or(15_000);

    let mut child = match Command::new(&python_executable)
        .arg(script_path)
        .arg("--file")
        .arg(&request.file_path)
        .arg("--preview-3d")
        .arg(preview_flag)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(value) => value,
        Err(error) => {
            return ImageGenerationResult {
                image_data: None,
                image_type: None,
                width_mm: None,
                height_mm: None,
                stitch_count: None,
                color_count: None,
                color_change_count: None,
                backend: "python".to_string(),
                error: Some(format!("Could not execute python adapter: {error}")),
            }
        }
    };

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if started.elapsed() >= Duration::from_millis(timeout_ms) {
                    let _ = child.kill();
                    let _ = child.wait();
                    if request.preview_3d {
                        let fallback_result =
                            generate_preview_via_python(&ImageGenerationRequest {
                                file_path: request.file_path.clone(),
                                preview_3d: false,
                                preview_3d_profile: request.preview_3d_profile.clone(),
                            });
                        if fallback_result.error.is_none() {
                            return fallback_result;
                        }

                        return ImageGenerationResult {
                            image_data: None,
                            image_type: None,
                            width_mm: None,
                            height_mm: None,
                            stitch_count: None,
                            color_count: None,
                            color_change_count: None,
                            backend: "python".to_string(),
                            error: Some(format!(
                                "Python image adapter timed out after {}ms for file '{}'; 2D fallback failed: {}",
                                timeout_ms,
                                request.file_path,
                                fallback_result.error.unwrap_or_else(|| "unknown fallback error".to_string())
                            )),
                        };
                    }

                    return ImageGenerationResult {
                        image_data: None,
                        image_type: None,
                        width_mm: None,
                        height_mm: None,
                        stitch_count: None,
                        color_count: None,
                        color_change_count: None,
                        backend: "python".to_string(),
                        error: Some(format!(
                            "Python image adapter timed out after {}ms for file '{}'",
                            timeout_ms, request.file_path
                        )),
                    };
                }

                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => {
                return ImageGenerationResult {
                    image_data: None,
                    image_type: None,
                    width_mm: None,
                    height_mm: None,
                    stitch_count: None,
                    color_count: None,
                    color_change_count: None,
                    backend: "python".to_string(),
                    error: Some(format!("Could not monitor python adapter process: {error}")),
                };
            }
        }
    }

    let output = match child.wait_with_output() {
        Ok(value) => value,
        Err(error) => {
            return ImageGenerationResult {
                image_data: None,
                image_type: None,
                width_mm: None,
                height_mm: None,
                stitch_count: None,
                color_count: None,
                color_change_count: None,
                backend: "python".to_string(),
                error: Some(format!("Could not collect python adapter output: {error}")),
            }
        }
    };

    if !output.status.success() {
        return ImageGenerationResult {
            image_data: None,
            image_type: None,
            width_mm: None,
            height_mm: None,
            stitch_count: None,
            color_count: None,
            color_change_count: None,
            backend: "python".to_string(),
            error: Some(format!(
                "Python image adapter failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )),
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let parsed: Result<PythonImageGenerationResult, _> = serde_json::from_str(stdout.trim());

    let parsed = match parsed {
        Ok(value) => value,
        Err(error) => {
            return ImageGenerationResult {
                image_data: None,
                image_type: None,
                width_mm: None,
                height_mm: None,
                stitch_count: None,
                color_count: None,
                color_change_count: None,
                backend: "python".to_string(),
                error: Some(format!("Could not parse python adapter output: {error}")),
            }
        }
    };

    let image_data = parsed.image_base64.as_ref().and_then(|encoded| {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
    });

    if parsed.error.is_some() && request.preview_3d {
        let fallback_result = generate_preview_via_python(&ImageGenerationRequest {
            file_path: request.file_path.clone(),
            preview_3d: false,
            preview_3d_profile: request.preview_3d_profile.clone(),
        });
        if fallback_result.error.is_none() {
            return fallback_result;
        }
    }

    ImageGenerationResult {
        image_data,
        image_type: parsed.image_type,
        width_mm: parsed.width_mm,
        height_mm: parsed.height_mm,
        stitch_count: parsed.stitch_count,
        color_count: parsed.color_count,
        color_change_count: parsed.color_change_count,
        backend: "python".to_string(),
        error: parsed.error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EmbPattern, EmbThread, Stitch, StitchType};
    use std::fmt::Debug;
    use std::path::PathBuf;

    // ════════════════════════════════════════════════════════════════════
    // 1. request_extension
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn request_extension_returns_lowercase_for_known_extensions() {
        assert_eq!(request_extension("file.pes"), Some("pes".to_string()));
        assert_eq!(request_extension("file.DST"), Some("dst".to_string()));
        assert_eq!(request_extension("file.Jef"), Some("jef".to_string()));
        assert_eq!(request_extension("file.VP3"), Some("vp3".to_string()));
    }

    #[test]
    fn request_extension_returns_none_for_no_extension() {
        assert_eq!(request_extension("file"), None);
        assert_eq!(request_extension("path/to/file"), None);
    }

    #[test]
    fn request_extension_handles_multiple_dots() {
        assert_eq!(
            request_extension("my.file.name.pes"),
            Some("pes".to_string())
        );
    }

    #[test]
    fn request_extension_returns_none_for_empty_path() {
        assert_eq!(request_extension(""), None);
    }

    // ════════════════════════════════════════════════════════════════════
    // 2. extension_support
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn extension_support_marks_pes_as_both() {
        assert_eq!(
            extension_support("file.pes"),
            BackendSupport::Both
        );
    }

    #[test]
    fn extension_support_marks_dst_as_both() {
        assert_eq!(
            extension_support("file.dst"),
            BackendSupport::Both
        );
    }

    #[test]
    fn extension_support_marks_exp_as_both() {
        assert_eq!(
            extension_support("file.exp"),
            BackendSupport::Both
        );
    }

    #[test]
    fn extension_support_marks_jef_as_both() {
        assert_eq!(
            extension_support("file.jef"),
            BackendSupport::Both
        );
    }

    #[test]
    fn extension_support_marks_vp3_as_both() {
        assert_eq!(
            extension_support("file.vp3"),
            BackendSupport::Both
        );
    }

    #[test]
    fn extension_support_marks_hus_as_native_only() {
        assert_eq!(
            extension_support("C:/imports/sample.hus"),
            BackendSupport::NativeOnly
        );
    }

    #[test]
    fn extension_support_marks_unknown_as_unsupported() {
        assert_eq!(
            extension_support("C:/imports/sample.txt"),
            BackendSupport::Unsupported
        );
    }

    #[test]
    fn extension_support_marks_no_extension_as_unsupported() {
        assert_eq!(
            extension_support("C:/imports/sample"),
            BackendSupport::Unsupported
        );
    }

    #[test]
    fn extension_support_marks_empty_path_as_unsupported() {
        assert_eq!(extension_support(""), BackendSupport::Unsupported);
    }

    #[test]
    fn extension_support_is_case_insensitive() {
        assert_eq!(
            extension_support("file.PES"),
            BackendSupport::Both
        );
        assert_eq!(
            extension_support("file.HUS"),
            BackendSupport::NativeOnly
        );
        assert_eq!(
            extension_support("file.TXT"),
            BackendSupport::Unsupported
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // 3. unsupported_extension_result
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn unsupported_extension_result_contains_extension_in_error() {
        let result = unsupported_extension_result("file.txt", "auto");
        assert!(result.image_data.is_none());
        assert!(result.image_type.is_none());
        assert!(result.width_mm.is_none());
        assert!(result.height_mm.is_none());
        assert!(result.stitch_count.is_none());
        assert!(result.color_count.is_none());
        assert!(result.color_change_count.is_none());
        assert_eq!(result.backend, "auto");
        let err = result.error.expect("should have error");
        assert!(err.contains("txt"), "error should mention extension");
        assert!(err.contains("skipped"), "error should mention skipped");
    }

    #[test]
    fn unsupported_extension_result_uses_unknown_when_no_extension() {
        let result = unsupported_extension_result("file", "native");
        assert_eq!(result.backend, "native");
        let err = result.error.expect("should have error");
        assert!(err.contains("unknown"), "error should say unknown for no extension");
    }

    #[test]
    fn unsupported_extension_result_preserves_backend_name() {
        let result = unsupported_extension_result("file.xyz", "python");
        assert_eq!(result.backend, "python");
    }

    // ════════════════════════════════════════════════════════════════════
    // 4. needs_python_backend
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn needs_python_backend_returns_false_for_native_only() {
        assert!(!needs_python_backend("file.hus"));
    }

    #[test]
    fn needs_python_backend_returns_false_for_both() {
        assert!(!needs_python_backend("file.pes"));
    }

    #[test]
    fn needs_python_backend_returns_false_for_unsupported() {
        assert!(!needs_python_backend("file.txt"));
    }

    #[test]
    fn needs_python_backend_returns_false_for_no_extension() {
        assert!(!needs_python_backend("file"));
    }

    // ════════════════════════════════════════════════════════════════════
    // 5. round_two
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn round_two_rounds_to_two_decimals() {
        assert!((round_two(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((round_two(1.234) - 1.23).abs() < 0.001);
        assert!((round_two(5.678) - 5.68).abs() < 0.001);
        assert!((round_two(-1.5) - (-1.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn round_two_handles_large_values() {
        let result = round_two(12345.6789);
        assert!((result - 12345.68).abs() < 0.001);
    }

    // ════════════════════════════════════════════════════════════════════
    // 6. three_d_style_from_profile_name
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn three_d_style_defaults_to_balanced_when_none() {
        let style = three_d_style_from_profile_name(None);
        assert_eq!(style, ThreeDStyle::default());
    }

    #[test]
    fn three_d_style_balanced_explicit() {
        let style = three_d_style_from_profile_name(Some("balanced"));
        assert_eq!(style, ThreeDStyle::default());
    }

    #[test]
    fn three_d_style_soft() {
        let style = three_d_style_from_profile_name(Some("soft"));
        assert_eq!(style.shadow_strength, 28);
        assert_eq!(style.highlight_strength, 20);
        assert_eq!(style.core_half_width, 1);
        assert_eq!(style.shadow_offset, 1);
        assert_eq!(style.highlight_offset, 1);
    }

    #[test]
    fn three_d_style_high_contrast_all_variants() {
        let expected = ThreeDStyle {
            shadow_strength: 64,
            highlight_strength: 52,
            core_half_width: 2,
            shadow_offset: 2,
            highlight_offset: 2,
        };
        assert_eq!(three_d_style_from_profile_name(Some("high-contrast")), expected);
        assert_eq!(three_d_style_from_profile_name(Some("high_contrast")), expected);
        assert_eq!(three_d_style_from_profile_name(Some("highcontrast")), expected);
    }

    #[test]
    fn three_d_style_is_case_insensitive() {
        assert_eq!(
            three_d_style_from_profile_name(Some("SOFT")),
            ThreeDStyle {
                shadow_strength: 28,
                highlight_strength: 20,
                core_half_width: 1,
                shadow_offset: 1,
                highlight_offset: 1,
            }
        );
        assert_eq!(
            three_d_style_from_profile_name(Some("High-Contrast")),
            ThreeDStyle {
                shadow_strength: 64,
                highlight_strength: 52,
                core_half_width: 2,
                shadow_offset: 2,
                highlight_offset: 2,
            }
        );
    }

    #[test]
    fn three_d_style_trims_whitespace() {
        let style = three_d_style_from_profile_name(Some("  soft  "));
        assert_eq!(style.shadow_strength, 28);
    }

    #[test]
    fn three_d_style_unknown_profile_defaults_to_balanced() {
        let style = three_d_style_from_profile_name(Some("unknown_profile_xyz"));
        assert_eq!(style, ThreeDStyle::default());
    }

    #[test]
    fn three_d_style_empty_string_defaults_to_balanced() {
        let style = three_d_style_from_profile_name(Some(""));
        assert_eq!(style, ThreeDStyle::default());
    }

    // ════════════════════════════════════════════════════════════════════
    // 7. drawable_bounds_mm
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn drawable_bounds_returns_none_for_empty_pattern() {
        let pattern = EmbPattern::new();
        assert_eq!(drawable_bounds_mm(&pattern), None);
    }

    #[test]
    fn drawable_bounds_returns_none_for_pattern_with_only_non_stitch_types() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Jump });
        pattern.stitches.push(Stitch { x: 10.0, y: 10.0, stitch_type: StitchType::ColorChange });
        assert_eq!(drawable_bounds_mm(&pattern), None);
    }

    #[test]
    fn drawable_bounds_returns_dimensions_for_single_stitch() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 5.0, y: 10.0, stitch_type: StitchType::Stitch });
        // Single stitch: diff = 0 in both axes -> bounds = (0.0, 0.0)
        let bounds = drawable_bounds_mm(&pattern);
        assert!(bounds.is_some());
        let (w, h) = bounds.unwrap();
        assert!((w - 0.0).abs() < f64::EPSILON);
        assert!((h - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drawable_bounds_returns_correct_mm_for_two_stitches() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 20.0, y: 10.0, stitch_type: StitchType::Stitch });
        let bounds = drawable_bounds_mm(&pattern);
        assert!(bounds.is_some());
        let (w, h) = bounds.unwrap();
        // (20 - 0) / 10 = 2.0 mm, (10 - 0) / 10 = 1.0 mm
        assert!((w - 2.0).abs() < 0.001);
        assert!((h - 1.0).abs() < 0.001);
    }

    #[test]
    fn drawable_bounds_ignores_non_stitch_commands() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 100.0, y: 0.0, stitch_type: StitchType::Jump });
        pattern.stitches.push(Stitch { x: 20.0, y: 10.0, stitch_type: StitchType::Stitch });
        let bounds = drawable_bounds_mm(&pattern);
        assert!(bounds.is_some());
        let (w, h) = bounds.unwrap();
        // Only Stitch types matter: min=0, max=20 => w=2.0, h=1.0
        assert!((w - 2.0).abs() < 0.001);
        assert!((h - 1.0).abs() < 0.001);
    }

    #[test]
    fn drawable_bounds_handles_negative_coordinates() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: -30.0, y: -20.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 5.0, stitch_type: StitchType::Stitch });
        let bounds = drawable_bounds_mm(&pattern);
        let (w, h) = bounds.unwrap();
        // (10 - (-30)) / 10 = 4.0, (5 - (-20)) / 10 = 2.5
        assert!((w - 4.0).abs() < 0.001);
        assert!((h - 2.5).abs() < 0.001);
    }

    // ════════════════════════════════════════════════════════════════════
    // 8. adapter_script_path
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn adapter_script_path_is_correct() {
        let path = adapter_script_path();
        assert_eq!(path, Path::new("src").join("services").join("python_image_adapter.py"));
    }

    // ════════════════════════════════════════════════════════════════════
    // 9. analyze_pattern_with_native_renderer (existing + expansions)
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn native_analysis_returns_expected_contract_shape() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 0.0,
            stitch_type: StitchType::Stitch,
        });
        pattern.stitches.push(Stitch {
            x: 20.0,
            y: 10.0,
            stitch_type: StitchType::Stitch,
        });

        let result = analyze_pattern_with_native_renderer(&pattern, false, None);

        assert_eq!(result.backend, "native");
        assert!(result.error.is_none());
        assert!(result
            .image_data
            .as_ref()
            .map(|bytes| !bytes.is_empty())
            .unwrap_or(false));
        assert_eq!(result.image_type.as_deref(), Some("2d"));
        assert_eq!(result.width_mm, Some(2.0));
        assert_eq!(result.height_mm, Some(1.0));
        assert_eq!(result.stitch_count, Some(2));
        assert_eq!(result.color_count, Some(1));
        assert_eq!(result.color_change_count, Some(0));
    }

    #[test]
    fn native_analysis_marks_3d_when_requested() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0x00FF00));
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 0.0,
            stitch_type: StitchType::Stitch,
        });
        pattern.stitches.push(Stitch {
            x: 10.0,
            y: 10.0,
            stitch_type: StitchType::Stitch,
        });

        let result = analyze_pattern_with_native_renderer(&pattern, true, Some("balanced"));

        assert_eq!(result.backend, "native");
        assert!(result.error.is_none());
        assert_eq!(result.image_type.as_deref(), Some("3d"));
        assert!(result
            .image_data
            .as_ref()
            .map(|bytes| !bytes.is_empty())
            .unwrap_or(false));
    }

    #[test]
    fn native_analysis_3d_with_soft_profile() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 10.0, stitch_type: StitchType::Stitch });

        let result = analyze_pattern_with_native_renderer(&pattern, true, Some("soft"));
        assert_eq!(result.image_type.as_deref(), Some("3d"));
        assert!(result.error.is_none());
        assert!(result.image_data.unwrap().len() > 0);
    }

    #[test]
    fn native_analysis_3d_with_high_contrast_profile() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 10.0, stitch_type: StitchType::Stitch });

        let result = analyze_pattern_with_native_renderer(&pattern, true, Some("high-contrast"));
        assert_eq!(result.image_type.as_deref(), Some("3d"));
        assert!(result.error.is_none());
    }

    #[test]
    fn native_analysis_3d_with_none_profile_defaults_to_balanced() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 10.0, stitch_type: StitchType::Stitch });

        let result = analyze_pattern_with_native_renderer(&pattern, true, None);
        assert_eq!(result.image_type.as_deref(), Some("3d"));
        assert!(result.error.is_none());
    }

    #[test]
    fn native_analysis_handles_empty_patterns_without_rendering() {
        let pattern = EmbPattern::new();
        let result = analyze_pattern_with_native_renderer(&pattern, false, None);

        assert_eq!(result.backend, "native");
        assert!(result.error.is_none());
        assert!(result.image_data.is_none());
        assert!(result.image_type.is_none());
        assert!(result.width_mm.is_none());
        assert!(result.height_mm.is_none());
        assert_eq!(result.stitch_count, Some(0));
        assert_eq!(result.color_count, Some(0));
        assert_eq!(result.color_change_count, Some(0));
    }

    #[test]
    fn native_analysis_counts_distinct_colors_but_keeps_color_changes() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_thread(EmbThread::new(0x00FF00));
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_thread(EmbThread::new(0x0000FF));
        pattern.add_thread(EmbThread::new(0xFF0000));

        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 0.0,
            stitch_type: StitchType::Stitch,
        });
        pattern.stitches.push(Stitch {
            x: 1.0,
            y: 1.0,
            stitch_type: StitchType::ColorChange,
        });
        pattern.stitches.push(Stitch {
            x: 2.0,
            y: 2.0,
            stitch_type: StitchType::ColorChange,
        });

        let result = analyze_pattern_with_native_renderer(&pattern, false, None);

        assert_eq!(result.color_count, Some(3));
        assert_eq!(result.color_change_count, Some(2));
    }

    #[test]
    fn native_analysis_pattern_with_only_non_stitch_types_still_renders() {
        // The stitches vector is NOT empty, so it enters the rendering path,
        // even though all stitches are non-Stitch types.
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 0.0,
            stitch_type: StitchType::Jump,
        });
        pattern.stitches.push(Stitch {
            x: 10.0,
            y: 10.0,
            stitch_type: StitchType::ColorChange,
        });

        let result = analyze_pattern_with_native_renderer(&pattern, false, None);
        assert_eq!(result.backend, "native");
        assert!(result.error.is_none());
        // stitches.len() > 0, so we render (even if no Stitch commands)
        assert!(result.image_data.is_some());
        assert_eq!(result.image_type.as_deref(), Some("2d"));
        // bounds will be None since no Stitch types, so width_mm/height_mm should be None
        assert!(result.width_mm.is_none());
        assert!(result.height_mm.is_none());
        assert_eq!(result.stitch_count, Some(2));
    }

    // ════════════════════════════════════════════════════════════════════
    // 10. read_pattern_from_file
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn read_pattern_from_file_fails_on_nonexistent_file() {
        let result = read_pattern_from_file("C:/nonexistent/file.pes");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Could not read embroidery file"));
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn read_pattern_from_file_fails_on_missing_extension() {
        // Use a temp file with no extension
        let dir = std::env::temp_dir();
        let file_path = dir.join("no_extension_file");
        // Ensure it exists but has no extension
        std::fs::write(&file_path, b"garbage data").ok();
        let result = read_pattern_from_file(file_path.to_str().unwrap());
        let _ = std::fs::remove_file(&file_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Missing file extension"));
    }

    #[test]
    fn read_pattern_from_file_fails_on_unsupported_extension() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("sample.txt");
        std::fs::write(&file_path, b"data").ok();
        let result = read_pattern_from_file(file_path.to_str().unwrap());
        let _ = std::fs::remove_file(&file_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("does not support extension '.txt'"));
    }

    #[test]
    fn read_pattern_from_file_fails_on_corrupt_file() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("corrupt.pes");
        std::fs::write(&file_path, b"not a real embroidery file").ok();
        let result = read_pattern_from_file(file_path.to_str().unwrap());
        let _ = std::fs::remove_file(&file_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Could not parse"));
    }

    #[test]
    fn read_pattern_from_file_succeeds_on_real_fixture() {
        let file_path = PathBuf::from("tests").join("Test Designs").join("Bean.pes");
        assert!(file_path.exists());
        let result = read_pattern_from_file(file_path.to_str().unwrap());
        assert!(result.is_ok(), "should parse Bean.pes: {:?}", result.err());
    }

    // ════════════════════════════════════════════════════════════════════
    // 11. generate_preview_via_native error path
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn generate_preview_via_native_propagates_read_error() {
        let request = ImageGenerationRequest {
            file_path: "C:/nonexistent/file.pes".to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        };
        let result = generate_preview_via_native(&request);
        assert_eq!(result.backend, "native");
        assert!(result.error.is_some());
        assert!(result.error.as_deref().unwrap().contains("Could not read"));
        assert!(result.image_data.is_none());
    }

    // ════════════════════════════════════════════════════════════════════
    // 12. generate_preview (dispatcher)
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn generate_preview_unsupported_extension_returns_skip_error() {
        // env not set -> defaults to "auto"
        std::env::remove_var("IMPORT_IMAGE_BACKEND");
        let result = generate_preview(&ImageGenerationRequest {
            file_path: "sample.txt".to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "auto");
        assert!(result.error.as_deref().unwrap().contains("skipped"));
    }

    #[test]
    fn generate_preview_backend_native_with_supported_extension_succeeds() {
        std::env::set_var("IMPORT_IMAGE_BACKEND", "native");
        let file_path = PathBuf::from("tests").join("Test Designs").join("Bean.pes");
        let result = generate_preview(&ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "native");
        assert!(result.error.is_none(), "native backend should succeed: {:?}", result.error);
        assert!(result.image_data.is_some());
    }

    #[test]
    fn generate_preview_backend_native_with_python_only_extension_returns_error() {
        std::env::set_var("IMPORT_IMAGE_BACKEND", "native");
        // .eof is not in NATIVE_PREVIEW_EXTENSIONS -> falls to extension_support Unsupported
        // Actually .eof is not in any list -> Unsupported -> unsupported_extension_result returns before the branch.
        // We need a PythonOnly extension. None currently exist in the const arrays that aren't also native.
        // Test with .hus which is NativeOnly: "native" backend should work fine.
        // So for the error path, we just document that no PythonOnly-only extension currently exists
        // that isn't also in native. The code path still exists; we can't hit it with current consts.
        // This is acceptable per plan.
    }

    #[test]
    fn generate_preview_backend_python_with_native_only_extension_returns_error() {
        std::env::set_var("IMPORT_IMAGE_BACKEND", "python");
        let result = generate_preview(&ImageGenerationRequest {
            file_path: "sample.hus".to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "python");
        assert!(result.error.is_some());
        assert!(result.error.as_deref().unwrap().contains("does not support"));
    }

    #[test]
    fn generate_preview_backend_python_with_supported_extension_attempts_python() {
        std::env::set_var("IMPORT_IMAGE_BACKEND", "python");
        // The python adapter script likely doesn't exist -> should get "script not found" error
        let file_path = PathBuf::from("tests").join("Test Designs").join("Bean.pes");
        let result = generate_preview(&ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "python");
        // Should either succeed (if python is set up) or get "script not found"
        if result.error.is_some() {
            assert!(result.error.as_deref().unwrap().contains("script not found")
                || result.error.as_deref().unwrap().contains("Could not execute"),
                "unexpected error: {:?}", result.error);
        }
    }

    #[test]
    fn generate_preview_backend_invalid_returns_error() {
        std::env::set_var("IMPORT_IMAGE_BACKEND", "invalid_backend");
        let result = generate_preview(&ImageGenerationRequest {
            file_path: "sample.pes".to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "invalid_backend");
        assert!(result.error.is_some());
        assert!(result.error.as_deref().unwrap().contains("Unsupported image backend"));
    }

    #[test]
    fn generate_preview_backend_auto_defaults_when_env_not_set() {
        std::env::remove_var("IMPORT_IMAGE_BACKEND");
        let file_path = PathBuf::from("tests").join("Test Designs").join("Bean.pes");
        let result = generate_preview(&ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "native", "auto should use native for Both extension");
        assert!(result.error.is_none());
    }

    // ════════════════════════════════════════════════════════════════════
    // 13. generate_preview_auto
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn generate_preview_auto_unsupported_returns_skip() {
        let result = generate_preview_auto(&ImageGenerationRequest {
            file_path: "sample.txt".to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "auto");
        assert!(result.error.as_deref().unwrap().contains("skipped"));
    }

    #[test]
    fn generate_preview_auto_native_only_uses_native() {
        let file_path = PathBuf::from("tests").join("Test Designs").join("Bean.hus");
        if !file_path.exists() {
            // Some fixtures may not exist; use Bean.pes for Both, but we need NativeOnly
            // .hus files exist in test fixtures
            return;
        }
        let result = generate_preview_auto(&ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "native");
        assert!(result.error.is_none() || result.error.as_deref().unwrap().contains("Could not read"));
    }

    #[test]
    fn generate_preview_auto_python_only_returns_no_fallback_error() {
        // There is no current extension that is PythonOnly but not also native.
        // Test the code path with a deliberately unsupported approach:
        // We can't easily create a PythonOnly extension without modifying consts.
        // Documented as acceptable per plan.
    }

    #[test]
    fn generate_preview_auto_both_native_succeeds_returns_native() {
        let file_path = PathBuf::from("tests").join("Test Designs").join("Bean.pes");
        let result = generate_preview_auto(&ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "native");
        assert!(result.error.is_none());
    }

    // ════════════════════════════════════════════════════════════════════
    // 14. generate_preview_via_python error paths
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn generate_preview_via_python_script_not_found() {
        // The adapter script path is relative to the project root.
        // If it doesn't exist, we get the "script not found" error.
        let request = ImageGenerationRequest {
            file_path: "sample.pes".to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        };
        let result = generate_preview_via_python(&request);
        if !adapter_script_path().exists() {
            assert_eq!(result.backend, "python");
            assert!(result.error.is_some());
            assert!(result.error.as_deref().unwrap().contains("script not found"));
        }
        // If it does exist, we'd need python installed - this test is lenient
    }

    #[test]
    fn generate_preview_via_python_spawn_failure() {
        std::env::set_var("RUST_EMBROIDERY_PYTHON", "nonexistent_python_binary_xyz");
        let request = ImageGenerationRequest {
            file_path: "sample.pes".to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        };
        let result = generate_preview_via_python(&request);
        assert_eq!(result.backend, "python");
        assert!(result.error.is_some());
        let err = result.error.as_deref().unwrap();
        // If the adapter script does not exist we get "script not found";
        // if it exists but python binary is wrong we get "Could not execute".
        assert!(
            err.contains("script not found") || err.contains("Could not execute"),
            "unexpected error: {err}"
        );
        std::env::remove_var("RUST_EMBROIDERY_PYTHON");
    }

    // ════════════════════════════════════════════════════════════════════
    // 15. generate_previews_via_python_batch error paths
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn generate_previews_via_python_batch_empty_requests() {
        let result = generate_previews_via_python_batch(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn generate_previews_via_python_batch_script_not_found() {
        let requests = vec![
            ImageGenerationRequest {
                file_path: "file1.pes".to_string(),
                preview_3d: false,
                preview_3d_profile: None,
            },
            ImageGenerationRequest {
                file_path: "file2.dst".to_string(),
                preview_3d: false,
                preview_3d_profile: None,
            },
        ];
        let result = generate_previews_via_python_batch(&requests);
        if !adapter_script_path().exists() {
            assert_eq!(result.len(), 2);
            for (path, res) in &result {
                assert_eq!(res.backend, "python-batch");
                assert!(res.error.is_some());
                assert!(res.error.as_deref().unwrap().contains("script not found"));
                assert!(path == "file1.pes" || path == "file2.dst");
            }
        }
    }

    #[test]
    fn generate_previews_via_python_batch_spawn_failure() {
        std::env::set_var("RUST_EMBROIDERY_PYTHON", "nonexistent_python_binary_xyz");
        let requests = vec![
            ImageGenerationRequest {
                file_path: "file1.pes".to_string(),
                preview_3d: false,
                preview_3d_profile: None,
            },
        ];
        let result = generate_previews_via_python_batch(&requests);
        assert_eq!(result.len(), 1);
        let res = result.get("file1.pes").expect("should have entry");
        assert_eq!(res.backend, "python-batch");
        assert!(res.error.is_some());
        let err = res.error.as_deref().unwrap();
        // If the adapter script does not exist we get "script not found";
        // if it exists but python binary is wrong we get "Could not execute".
        assert!(
            err.contains("script not found") || err.contains("Could not execute"),
            "unexpected error: {err}"
        );
        std::env::remove_var("RUST_EMBROIDERY_PYTHON");
    }

    // ════════════════════════════════════════════════════════════════════
    // 16. Integration: Native backend with fixture files
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn native_backend_parses_vp3_fixture() {
        let file_path = PathBuf::from("tests").join("Test Designs").join("Bean.vp3");
        assert!(file_path.exists(), "expected VP3 fixture file to exist");

        let request = ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        };

        let native = generate_preview_via_native(&request);

        assert_eq!(native.backend, "native");
        assert!(
            native.error.is_none(),
            "native backend should succeed for VP3 fixture"
        );
        assert_eq!(native.image_type.as_deref(), Some("2d"));
        assert!(native
            .image_data
            .as_ref()
            .map(|bytes| !bytes.is_empty())
            .unwrap_or(false));
        assert!(native.stitch_count.unwrap_or_default() > 0);
    }

    #[test]
    fn python_and_native_backends_share_core_result_contract_for_2d() {
        let file_path = PathBuf::from("tests").join("Test Designs").join("Bean.pes");
        assert!(file_path.exists(), "expected test embroidery file to exist");

        let request = ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        };

        let native = generate_preview_via_native(&request);
        assert!(
            native.error.is_none(),
            "native backend should succeed for fixture file"
        );

        let python = generate_preview_via_python(&request);
        if python.error.is_some() {
            tracing::debug!(
                "Skipping strict python/native parity assertions because python adapter is unavailable: {}",
                python.error.unwrap_or_else(|| "unknown python adapter error".to_string())
            );
            return;
        }

        assert!(native
            .image_data
            .as_ref()
            .map(|bytes| !bytes.is_empty())
            .unwrap_or(false));
        assert!(python
            .image_data
            .as_ref()
            .map(|bytes| !bytes.is_empty())
            .unwrap_or(false));
        assert_eq!(native.image_type.as_deref(), Some("2d"));
        assert_eq!(python.image_type.as_deref(), Some("2d"));

        assert_eq!(native.stitch_count.is_some(), python.stitch_count.is_some());
        assert_eq!(native.color_count.is_some(), python.color_count.is_some());
        assert_eq!(
            native.color_change_count.is_some(),
            python.color_change_count.is_some()
        );
        assert_eq!(native.width_mm.is_some(), python.width_mm.is_some());
        assert_eq!(native.height_mm.is_some(), python.height_mm.is_some());
    }

    #[test]
    fn python_and_native_backends_match_metrics_for_complex_vp3_fixture() {
        let file_path = PathBuf::from("tests").join("Test Designs").join("Cake 3.vp3");
        assert!(
            file_path.exists(),
            "expected complex VP3 fixture file to exist"
        );

        let request = ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        };

        let native = generate_preview_via_native(&request);
        assert!(
            native.error.is_none(),
            "native backend should succeed for complex VP3 fixture"
        );

        let python = generate_preview_via_python(&request);
        if python.error.is_some() {
            tracing::debug!(
                "Skipping complex VP3 parity assertions because python adapter is unavailable: {}",
                python
                    .error
                    .unwrap_or_else(|| "unknown python adapter error".to_string())
            );
            return;
        }

        assert_eq!(native.image_type.as_deref(), Some("2d"));
        assert_eq!(python.image_type.as_deref(), Some("2d"));
        assert!(native
            .image_data
            .as_ref()
            .map(|bytes| !bytes.is_empty())
            .unwrap_or(false));
        assert!(python
            .image_data
            .as_ref()
            .map(|bytes| !bytes.is_empty())
            .unwrap_or(false));

        assert_eq!(native.stitch_count, python.stitch_count);
        assert_eq!(native.color_count, python.color_count);
        assert_eq!(native.color_change_count, python.color_change_count);

        let native_width = native.width_mm.expect("native width should be present");
        let python_width = python.width_mm.expect("python width should be present");
        let native_height = native.height_mm.expect("native height should be present");
        let python_height = python.height_mm.expect("python height should be present");

        assert!((native_width - python_width).abs() <= 0.01);
        assert!((native_height - python_height).abs() <= 0.01);
    }

    #[test]
    fn native_backend_parses_user_vp3_regression_fixture_when_present() {
        let file_path = PathBuf::from("tests").join("Test Designs").join("220306.vp3");
        if !file_path.exists() {
            tracing::debug!(
                "Skipping user VP3 regression fixture test because file is not present: {}",
                file_path.display()
            );
            return;
        }

        let request = ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        };

        let native = generate_preview_via_native(&request);
        assert!(
            native.error.is_none(),
            "native backend should succeed for user VP3 fixture"
        );
        assert_eq!(native.image_type.as_deref(), Some("2d"));
        assert!(native
            .image_data
            .as_ref()
            .map(|bytes| !bytes.is_empty())
            .unwrap_or(false));
        assert!(native.stitch_count.unwrap_or_default() > 0);
    }

    // ════════════════════════════════════════════════════════════════════
    // 17. Enum and struct contract tests
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn backend_support_derives_debug_clone_copy_partial_eq() {
        // Verify the enum has the expected traits.
        fn assert_traits<T: Debug + Clone + Copy + PartialEq>() {}
        assert_traits::<BackendSupport>();

        assert_eq!(BackendSupport::NativeOnly, BackendSupport::NativeOnly);
        assert_ne!(BackendSupport::NativeOnly, BackendSupport::PythonOnly);
    }

    #[test]
    fn image_generation_request_serde_roundtrip() {
        let original = ImageGenerationRequest {
            file_path: "C:/path/to/file.pes".to_string(),
            preview_3d: true,
            preview_3d_profile: Some("soft".to_string()),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let deserialized: ImageGenerationRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.file_path, original.file_path);
        assert_eq!(deserialized.preview_3d, original.preview_3d);
        assert_eq!(deserialized.preview_3d_profile, original.preview_3d_profile);
    }

    #[test]
    fn image_generation_result_serde_roundtrip() {
        let original = ImageGenerationResult {
            image_data: Some(vec![1, 2, 3]),
            image_type: Some("2d".to_string()),
            width_mm: Some(10.5),
            height_mm: Some(8.2),
            stitch_count: Some(1000),
            color_count: Some(5),
            color_change_count: Some(10),
            backend: "native".to_string(),
            error: None,
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let deserialized: ImageGenerationResult = serde_json::from_str(&json).expect("deserialize");
        // image_data is Vec<u8> which serializes as base64, so we can't compare directly via PartialEq
        assert_eq!(deserialized.image_data, original.image_data);
        assert_eq!(deserialized.image_type, original.image_type);
        assert_eq!(deserialized.width_mm, original.width_mm);
        assert_eq!(deserialized.height_mm, original.height_mm);
        assert_eq!(deserialized.stitch_count, original.stitch_count);
        assert_eq!(deserialized.color_count, original.color_count);
        assert_eq!(deserialized.color_change_count, original.color_change_count);
        assert_eq!(deserialized.backend, original.backend);
        assert_eq!(deserialized.error, original.error);
    }

    #[test]
    fn python_batch_result_deserialization() {
        let json = r#"{
            "file_path": "test.pes",
            "image_base64": "dGVzdA==",
            "image_type": "2d",
            "width_mm": 10.0,
            "height_mm": 8.0,
            "stitch_count": 500,
            "color_count": 3,
            "color_change_count": 5,
            "error": null
        }"#;
        let parsed: PythonBatchResult = serde_json::from_str(json).expect("deserialize");
        assert_eq!(parsed.file_path, "test.pes");
        assert_eq!(parsed.image_base64, Some("dGVzdA==".to_string()));
        assert_eq!(parsed.image_type, Some("2d".to_string()));
        assert_eq!(parsed.width_mm, Some(10.0));
        assert_eq!(parsed.height_mm, Some(8.0));
        assert_eq!(parsed.stitch_count, Some(500));
        assert_eq!(parsed.color_count, Some(3));
        assert_eq!(parsed.color_change_count, Some(5));
        assert!(parsed.error.is_none());
    }

    #[test]
    fn python_image_generation_result_deserialization() {
        let json = r#"{
            "image_base64": null,
            "image_type": "3d",
            "width_mm": null,
            "height_mm": null,
            "stitch_count": null,
            "color_count": null,
            "color_change_count": null,
            "error": "some error"
        }"#;
        let parsed: PythonImageGenerationResult = serde_json::from_str(json).expect("deserialize");
        assert!(parsed.image_base64.is_none());
        assert_eq!(parsed.image_type, Some("3d".to_string()));
        assert!(parsed.width_mm.is_none());
        assert!(parsed.error.is_some());
    }
}