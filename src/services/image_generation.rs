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
mod tests {
    use super::*;
    use crate::models::{EmbPattern, EmbThread, Stitch, StitchType};
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
    // 2. extension_supported
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn extension_supported_marks_pes_as_supported() {
        assert!(extension_supported("file.pes"));
    }

    #[test]
    fn extension_supported_marks_dst_as_supported() {
        assert!(extension_supported("file.dst"));
    }

    #[test]
    fn extension_supported_marks_exp_as_supported() {
        assert!(extension_supported("file.exp"));
    }

    #[test]
    fn extension_supported_marks_jef_as_supported() {
        assert!(extension_supported("file.jef"));
    }

    #[test]
    fn extension_supported_marks_vp3_as_supported() {
        assert!(extension_supported("file.vp3"));
    }

    #[test]
    fn extension_supported_marks_hus_as_supported() {
        assert!(extension_supported("C:/imports/sample.hus"));
    }

    #[test]
    fn extension_supported_marks_unknown_as_unsupported() {
        assert!(!extension_supported("C:/imports/sample.txt"));
    }

    #[test]
    fn extension_supported_marks_no_extension_as_unsupported() {
        assert!(!extension_supported("C:/imports/sample"));
    }

    #[test]
    fn extension_supported_marks_empty_path_as_unsupported() {
        assert!(!extension_supported(""));
    }

    #[test]
    fn extension_supported_is_case_insensitive() {
        assert!(extension_supported("file.PES"));
        assert!(extension_supported("file.HUS"));
        assert!(!extension_supported("file.TXT"));
    }

    // ════════════════════════════════════════════════════════════════════
    // 3. unsupported_extension_result
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn unsupported_extension_result_contains_extension_in_error() {
        let result = unsupported_extension_result("file.txt", "native");
        assert!(result.image_data.is_none());
        assert!(result.image_type.is_none());
        assert!(result.width_mm.is_none());
        assert!(result.height_mm.is_none());
        assert!(result.stitch_count.is_none());
        assert!(result.color_count.is_none());
        assert!(result.color_change_count.is_none());
        assert_eq!(result.backend, "native");
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
        let result = unsupported_extension_result("file.xyz", "native");
        assert_eq!(result.backend, "native");
    }

    // ════════════════════════════════════════════════════════════════════
    // 4. round_two
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
    // 5. three_d_style_from_profile_name
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
    // 6. drawable_bounds_mm
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
        assert!((w - 4.0).abs() < 0.001);
        assert!((h - 2.5).abs() < 0.001);
    }

    // ════════════════════════════════════════════════════════════════════
    // 7. analyze_pattern_with_native_renderer
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn native_analysis_returns_expected_contract_shape() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 20.0, y: 10.0, stitch_type: StitchType::Stitch });

        let result = analyze_pattern_with_native_renderer(&pattern, false, None);

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
    fn native_analysis_marks_3d_when_requested() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0x00FF00));
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 10.0, stitch_type: StitchType::Stitch });

        let result = analyze_pattern_with_native_renderer(&pattern, true, Some("balanced"));

        assert_eq!(result.backend, "native");
        assert!(result.error.is_none());
        assert_eq!(result.image_type.as_deref(), Some("3d"));
        assert!(result.image_data.as_ref().map(|bytes| !bytes.is_empty()).unwrap_or(false));
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

        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 1.0, y: 1.0, stitch_type: StitchType::ColorChange });
        pattern.stitches.push(Stitch { x: 2.0, y: 2.0, stitch_type: StitchType::ColorChange });

        let result = analyze_pattern_with_native_renderer(&pattern, false, None);

        assert_eq!(result.color_count, Some(3));
        assert_eq!(result.color_change_count, Some(2));
    }

    #[test]
    fn native_analysis_pattern_with_only_non_stitch_types_still_renders() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Jump });
        pattern.stitches.push(Stitch { x: 10.0, y: 10.0, stitch_type: StitchType::ColorChange });

        let result = analyze_pattern_with_native_renderer(&pattern, false, None);
        assert_eq!(result.backend, "native");
        assert!(result.error.is_none());
        assert!(result.image_data.is_some());
        assert_eq!(result.image_type.as_deref(), Some("2d"));
        assert!(result.width_mm.is_none());
        assert!(result.height_mm.is_none());
        assert_eq!(result.stitch_count, Some(2));
    }

    // ════════════════════════════════════════════════════════════════════
    // 8. read_pattern_from_file
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
        let dir = std::env::temp_dir();
        let file_path = dir.join("no_extension_file");
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
    // 9. generate_preview_via_native error path
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
    // 10. generate_preview (dispatcher)
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn generate_preview_unsupported_extension_returns_skip_error() {
        let result = generate_preview(&ImageGenerationRequest {
            file_path: "sample.txt".to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        });
        assert_eq!(result.backend, "native");
        assert!(result.error.as_deref().unwrap().contains("skipped"));
    }

    #[test]
    fn generate_preview_with_supported_extension_succeeds() {
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

    // ════════════════════════════════════════════════════════════════════
    // 11. Integration: Native backend with fixture files
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
        assert!(native.error.is_none(), "native backend should succeed for VP3 fixture");
        assert_eq!(native.image_type.as_deref(), Some("2d"));
        assert!(native.image_data.as_ref().map(|bytes| !bytes.is_empty()).unwrap_or(false));
        assert!(native.stitch_count.unwrap_or_default() > 0);
    }

    #[test]
    fn native_backend_parses_user_vp3_regression_fixture_when_present() {
        let file_path = PathBuf::from("tests").join("Test Designs").join("220306.vp3");
        if !file_path.exists() {
            return;
        }

        let request = ImageGenerationRequest {
            file_path: file_path.to_string_lossy().to_string(),
            preview_3d: false,
            preview_3d_profile: None,
        };

        let native = generate_preview_via_native(&request);
        assert!(native.error.is_none(), "native backend should succeed for user VP3 fixture");
        assert_eq!(native.image_type.as_deref(), Some("2d"));
        assert!(native.image_data.as_ref().map(|bytes| !bytes.is_empty()).unwrap_or(false));
        assert!(native.stitch_count.unwrap_or_default() > 0);
    }

    // ════════════════════════════════════════════════════════════════════
    // 12. Struct contract tests
    // ════════════════════════════════════════════════════════════════════

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
}