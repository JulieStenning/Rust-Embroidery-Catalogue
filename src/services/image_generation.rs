use base64::Engine;
use crate::models::EmbPattern;
use crate::png_writer::{render_pattern_to_png, RenderSettings};
use crate::readers::{
    A10oReader, DatReader, DsbReader, DstReader, DszReader, EmdReader, EmbroideryReader,
    ExyReader, ExpReader, FxyReader, GtReader, HusReader, InbReader, JefReader, JpxReader,
    MaxReader, MitReader, NewReader, PcmReader, PcqReader, PcsReader, PecReader, PesReader,
    PhbReader, PhcReader, SewReader, ShvReader, StcReader, StxReader, TapReader, TbfReader,
    Vp3Reader, XxxReader,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const NATIVE_PREVIEW_EXTENSIONS: &[&str] = &[
    "pes", "dst", "exp", "jef", "vp3", "hus", "10o", "pec", "dat", "dsb", "dsz", "emd",
    "exy", "fxy", "gt", "inb", "jpx", "max", "mit", "new", "pcm", "pcq", "pcs", "phb",
    "phc", "sew", "shv", "stc", "stx", "tap", "tbf", "xxx",
];
const PYTHON_PREVIEW_EXTENSIONS: &[&str] = &[
    "jef", "pes", "dst", "exp", "vp3", "u01", "100", "zhs", "zxy", "gcode", "art", "pmv",
];

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
        let msg = format!("Python image adapter script not found: {}", script_path.to_string_lossy());
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
                    println!(
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
                    error: Some("Python image backend does not support this extension.".to_string()),
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
                    error: Some("Native image backend does not support this extension.".to_string()),
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
            return generate_preview_via_python(request);
        }
        BackendSupport::Both => {}
    }

    // Prefer native for fast 2D generation; prefer Python for 3D parity.
    if !request.preview_3d {
        let native = generate_preview_via_native(request);
        if native.error.is_none() {
            return native;
        }

        let python = generate_preview_via_python(request);
        if python.error.is_none() {
            return python;
        }

        return ImageGenerationResult {
            image_data: None,
            image_type: None,
            width_mm: None,
            height_mm: None,
            stitch_count: None,
            color_count: None,
            color_change_count: None,
            backend: "auto".to_string(),
            error: Some(format!(
                "Auto backend failed: native='{}'; python='{}'",
                native.error.unwrap_or_else(|| "unknown native error".to_string()),
                python.error.unwrap_or_else(|| "unknown python error".to_string())
            )),
        };
    }

    let python = generate_preview_via_python(request);
    if python.error.is_none() {
        return python;
    }

    let native = generate_preview_via_native(request);
    if native.error.is_none() {
        return native;
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
            "Auto backend failed: python='{}'; native='{}'",
            python.error.unwrap_or_else(|| "unknown python error".to_string()),
            native.error.unwrap_or_else(|| "unknown native error".to_string())
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

    analyze_pattern_with_native_renderer(&pattern, request.preview_3d)
}

fn read_pattern_from_file(file_path: &str) -> Result<EmbPattern, String> {
    let data = fs::read(file_path)
        .map_err(|error| format!("Could not read embroidery file '{}': {error}", file_path))?;

    let extension = request_extension(file_path)
        .ok_or_else(|| format!("Missing file extension for '{}'.", file_path))?;

    let parsed = match extension.as_str() {
        "10o" => A10oReader.read(&data),
        "pec" => PecReader.read(&data),
        "pes" => PesReader.read(&data),
        "dst" => DstReader.read(&data),
        "exp" => ExpReader.read(&data),
        "jef" => JefReader.read(&data),
        "hus" => HusReader.read(&data),
        "dat" => DatReader.read(&data),
        "dsb" => DsbReader.read(&data),
        "dsz" => DszReader.read(&data),
        "emd" => EmdReader.read(&data),
        "exy" => ExyReader.read(&data),
        "fxy" => FxyReader.read(&data),
        "gt" => GtReader.read(&data),
        "inb" => InbReader.read(&data),
        "jpx" => JpxReader.read(&data),
        "max" => MaxReader.read(&data),
        "mit" => MitReader.read(&data),
        "new" => NewReader.read(&data),
        "pcm" => PcmReader.read(&data),
        "pcq" => PcqReader.read(&data),
        "pcs" => PcsReader.read(&data),
        "phb" => PhbReader.read(&data),
        "phc" => PhcReader.read(&data),
        "sew" => SewReader.read(&data),
        "shv" => ShvReader.read(&data),
        "stc" => StcReader.read(&data),
        "stx" => StxReader.read(&data),
        "tap" => TapReader.read(&data),
        "tbf" => TbfReader.read(&data),
        "vp3" => Vp3Reader.read(&data),
        "xxx" => XxxReader.read(&data),
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

fn analyze_pattern_with_native_renderer(pattern: &EmbPattern, preview_3d: bool) -> ImageGenerationResult {
    let stitch_count = i64::try_from(pattern.count_stitches()).unwrap_or(i64::MAX);
    let color_count = i64::try_from(pattern.count_threads()).unwrap_or(i64::MAX);
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

    let settings = RenderSettings::default();
    let image_data = render_pattern_to_png(pattern, &settings);
    let (width_mm, height_mm) = drawable_bounds_mm(pattern)
        .map(|(w, h)| (Some(w), Some(h)))
        .unwrap_or((None, None));

    ImageGenerationResult {
        image_data: Some(image_data),
        image_type: Some(if preview_3d { "2d" } else { "2d" }.to_string()),
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

    let python_executable = std::env::var("RUST_EMBROIDERY_PYTHON").unwrap_or_else(|_| "python".to_string());
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
                        let fallback_result = generate_preview_via_python(&ImageGenerationRequest {
                            file_path: request.file_path.clone(),
                            preview_3d: false,
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
            };
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

    let image_data = parsed
        .image_base64
        .as_ref()
        .and_then(|encoded| base64::engine::general_purpose::STANDARD.decode(encoded).ok());

    if parsed.error.is_some() && request.preview_3d {
        let fallback_result = generate_preview_via_python(&ImageGenerationRequest {
            file_path: request.file_path.clone(),
            preview_3d: false,
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
    use std::path::PathBuf;

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

        let result = analyze_pattern_with_native_renderer(&pattern, false);

        assert_eq!(result.backend, "native");
        assert!(result.error.is_none());
        assert!(result.image_data.as_ref().map(|bytes| !bytes.is_empty()).unwrap_or(false));
        assert_eq!(result.image_type.as_deref(), Some("2d"));
        assert_eq!(result.width_mm, Some(2.0));
        assert_eq!(result.height_mm, Some(1.0));
        assert_eq!(result.stitch_count, Some(2));
        assert_eq!(result.color_count, Some(1));
        assert_eq!(result.color_change_count, Some(0));
    }

    #[test]
    fn native_analysis_handles_empty_patterns_without_rendering() {
        let pattern = EmbPattern::new();
        let result = analyze_pattern_with_native_renderer(&pattern, false);

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
    fn python_and_native_backends_share_core_result_contract_for_2d() {
        let file_path = PathBuf::from("tests").join("testdata").join("Bean.pes");
        assert!(file_path.exists(), "expected test embroidery file to exist");

        let request = ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
        };

        let native = generate_preview_via_native(&request);
        assert!(native.error.is_none(), "native backend should succeed for fixture file");

        let python = generate_preview_via_python(&request);
        if python.error.is_some() {
            eprintln!(
                "Skipping strict python/native parity assertions because python adapter is unavailable: {}",
                python.error.unwrap_or_else(|| "unknown python adapter error".to_string())
            );
            return;
        }

        assert!(native.image_data.as_ref().map(|bytes| !bytes.is_empty()).unwrap_or(false));
        assert!(python.image_data.as_ref().map(|bytes| !bytes.is_empty()).unwrap_or(false));
        assert_eq!(native.image_type.as_deref(), Some("2d"));
        assert_eq!(python.image_type.as_deref(), Some("2d"));

        assert_eq!(native.stitch_count.is_some(), python.stitch_count.is_some());
        assert_eq!(native.color_count.is_some(), python.color_count.is_some());
        assert_eq!(native.color_change_count.is_some(), python.color_change_count.is_some());
        assert_eq!(native.width_mm.is_some(), python.width_mm.is_some());
        assert_eq!(native.height_mm.is_some(), python.height_mm.is_some());
    }

    #[test]
    fn extension_support_marks_hus_as_native_only() {
        assert_eq!(extension_support("C:/imports/sample.hus"), BackendSupport::NativeOnly);
    }

    #[test]
    fn extension_support_marks_promoted_optional_formats_as_native_only() {
        assert_eq!(extension_support("C:/imports/sample.10o"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.dat"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.dsb"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.dsz"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.emd"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.exy"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.fxy"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.gt"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.inb"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.jpx"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.max"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.mit"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.new"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.pcm"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.pcq"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.pcs"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.phb"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.phc"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.pec"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.sew"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.shv"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.stc"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.stx"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.tap"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.tbf"), BackendSupport::NativeOnly);
        assert_eq!(extension_support("C:/imports/sample.xxx"), BackendSupport::NativeOnly);
    }

    #[test]
    fn extension_support_marks_unknown_as_unsupported() {
        assert_eq!(extension_support("C:/imports/sample.txt"), BackendSupport::Unsupported);
    }

    #[test]
    fn generate_preview_skips_unsupported_extension_without_invoking_backends() {
        let result = generate_preview(&ImageGenerationRequest {
            file_path: "C:/imports/sample.txt".to_string(),
            preview_3d: false,
        });

        assert_eq!(result.backend, "auto");
        assert!(result.image_data.is_none());
        assert!(result.error.is_some());
        assert!(result
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("skipped"));
    }

}