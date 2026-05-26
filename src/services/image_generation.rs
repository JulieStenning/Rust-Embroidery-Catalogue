use base64::Engine;
use crate::models::EmbPattern;
use crate::png_writer::{render_pattern_to_png, RenderSettings};
use crate::readers::{DstReader, EmbroideryReader, ExpReader, JefReader, PesReader, Vp3Reader};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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

pub fn generate_preview(request: &ImageGenerationRequest) -> ImageGenerationResult {
    let backend = std::env::var("IMPORT_IMAGE_BACKEND").unwrap_or_else(|_| "auto".to_string());

    match backend.to_ascii_lowercase().as_str() {
        "python" => generate_preview_via_python(request),
        "native" => generate_preview_via_native(request),
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

    let extension = Path::new(file_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| format!("Missing file extension for '{}'.", file_path))?;

    let parsed = match extension.as_str() {
        "pes" => PesReader.read(&data),
        "dst" => DstReader.read(&data),
        "exp" => ExpReader.read(&data),
        "jef" => JefReader.read(&data),
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
}