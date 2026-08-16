use crate::models::EmbPattern;
use crate::png_writer::{render_pattern_to_png, RenderSettings, ThreeDStyle};
use crate::readers::{
    DstReader, EmbroideryReader, ExpReader, HusReader, JefReader, PesReader, Vp3Reader,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const NATIVE_PREVIEW_EXTENSIONS: &[&str] = &["pes", "dst", "exp", "jef", "vp3", "hus"];

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

fn request_extension(file_path: &str) -> Option<String> {
    Path::new(file_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
}

fn extension_supported(file_path: &str) -> bool {
    let Some(extension) = request_extension(file_path) else {
        return false;
    };

    NATIVE_PREVIEW_EXTENSIONS
        .iter()
        .any(|candidate| *candidate == extension)
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

pub fn generate_preview(request: &ImageGenerationRequest) -> ImageGenerationResult {
    if !extension_supported(&request.file_path) {
        return unsupported_extension_result(&request.file_path, "native");
    }

    generate_preview_via_native(request)
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
    let image_data = render_pattern_to_png(pattern, &settings).unwrap_or_default();
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
#[cfg(test)]
#[path = "image_generation_tests.rs"]
mod tests;
