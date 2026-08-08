// Tests for the PNG writer.
//
// This module was split out of png_writer.rs so the writer file can stay
// focused on production rendering logic. It is included via a #[path]
// declaration in a #[cfg(test)] mod tests; module, so it retains full
// access to the private items in the parent module through use super::*;.

use super::*;
use crate::models::{EmbPattern, EmbThread, Stitch, StitchType};

fn count_non_bg_pixels(png_bytes: &[u8], bg: Rgba<u8>) -> usize {
    let img = image::load_from_memory(png_bytes)
        .expect("decode png")
        .to_rgba8();
    img.pixels().filter(|p| **p != bg).count()
}

fn image_dimensions(png_bytes: &[u8]) -> (u32, u32) {
    let img = image::load_from_memory(png_bytes)
        .expect("decode png")
        .to_rgba8();
    (img.width(), img.height())
}

#[test]
fn renders_simple_pattern() {
    let mut pattern = EmbPattern::new();
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 10.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.threadlist.push(EmbThread::new(0xFF0000)); // Red
    let settings = RenderSettings::default();
    let png = render_pattern_to_png(&pattern, &settings).unwrap();
    assert!(
        count_non_bg_pixels(&png, settings.background) > 0,
        "Should render visible line"
    );
}

#[test]
fn renders_blank_for_no_stitches() {
    let pattern = EmbPattern::new();
    let settings = RenderSettings::default();
    let png = render_pattern_to_png(&pattern, &settings).unwrap();
    assert_eq!(
        count_non_bg_pixels(&png, settings.background),
        0,
        "No stitches should be blank"
    );
}

#[test]
fn uses_default_color_if_no_threads() {
    let mut pattern = EmbPattern::new();
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 10.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    let settings = RenderSettings::default();
    let png = render_pattern_to_png(&pattern, &settings).unwrap();
    // Should not be blank
    assert!(
        count_non_bg_pixels(&png, settings.background) > 0,
        "Should render with default color"
    );
}

#[test]
fn renders_color_change_segments() {
    let mut pattern = EmbPattern::new();
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 10.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 10.0,
        y: 10.0,
        stitch_type: StitchType::ColorChange,
    });
    pattern.stitches.push(Stitch {
        x: 20.0,
        y: 10.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.threadlist.push(EmbThread::new(0xFF0000)); // Red
    pattern.threadlist.push(EmbThread::new(0x0000FF)); // Blue
    let settings = RenderSettings::default();
    let png = render_pattern_to_png(&pattern, &settings).unwrap();
    // Should not be blank
    assert!(
        count_non_bg_pixels(&png, settings.background) > 0,
        "Should render with color changes"
    );
}

#[test]
fn color_change_switches_to_next_thread_color() {
    let mut pattern = EmbPattern::new();
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 8.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 8.0,
        y: 6.0,
        stitch_type: StitchType::ColorChange,
    });
    pattern.stitches.push(Stitch {
        x: 12.0,
        y: 6.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 20.0,
        y: 6.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.threadlist.push(EmbThread::new(0xFF0000));
    pattern.threadlist.push(EmbThread::new(0x0000FF));

    let settings = RenderSettings::default().with_preview_3d(false);
    let png = render_pattern_to_png(&pattern, &settings).unwrap();
    let img = image::load_from_memory(&png)
        .expect("decode png")
        .to_rgba8();

    let red = Rgba([255, 0, 0, 255]);
    let blue = Rgba([0, 0, 255, 255]);

    assert!(
        img.pixels().any(|pixel| *pixel == red),
        "first thread color should be rendered"
    );
    assert!(
        img.pixels().any(|pixel| *pixel == blue),
        "second thread color should be rendered after color change"
    );
}

#[test]
fn does_not_panic_on_empty_pattern() {
    let pattern = EmbPattern::new();
    let settings = RenderSettings::default();
    let _ = render_pattern_to_png(&pattern, &settings).unwrap();
}

#[test]
fn ignores_jump_only_outliers_when_framing_preview() {
    let mut pattern = EmbPattern::new();
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 10.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 10000.0,
        y: 10000.0,
        stitch_type: StitchType::Jump,
    });
    pattern.stitches.push(Stitch {
        x: 12.0,
        y: 1.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.threadlist.push(EmbThread::new(0x00AA00));

    let settings = RenderSettings::default();
    let png = render_pattern_to_png(&pattern, &settings).unwrap();
    let (width, height) = image_dimensions(&png);

    assert!(
        width < 200 && height < 200,
        "jump-only outlier should not inflate preview size"
    );
}

#[test]
fn preview_3d_mode_produces_distinct_image_from_2d() {
    let mut pattern = EmbPattern::new();
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 24.0,
        y: 8.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 36.0,
        y: 14.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.threadlist.push(EmbThread::new(0x1E88E5));

    let settings_2d = RenderSettings::default().with_preview_3d(false);
    let settings_3d = RenderSettings::default().with_preview_3d(true);

    let png_2d = render_pattern_to_png(&pattern, &settings_2d).unwrap();
    let png_3d = render_pattern_to_png(&pattern, &settings_3d).unwrap();

    assert_ne!(png_2d, png_3d, "3D mode should generate a distinct image");
    assert!(
        count_non_bg_pixels(&png_3d, settings_3d.background)
            >= count_non_bg_pixels(&png_2d, settings_2d.background),
        "3D mode should render at least as much stitched coverage as 2D"
    );
}

#[test]
fn three_d_style_profile_changes_render_output() {
    let mut pattern = EmbPattern::new();
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 18.0,
        y: 8.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.stitches.push(Stitch {
        x: 30.0,
        y: 12.0,
        stitch_type: StitchType::Stitch,
    });
    pattern.threadlist.push(EmbThread::new(0xE53935));

    let soft_profile = ThreeDStyle {
        shadow_strength: 28,
        highlight_strength: 22,
        core_half_width: 1,
        shadow_offset: 1,
        highlight_offset: 1,
    };
    let punchy_profile = ThreeDStyle {
        shadow_strength: 64,
        highlight_strength: 52,
        core_half_width: 2,
        shadow_offset: 2,
        highlight_offset: 2,
    };

    let soft_png = render_pattern_to_png(
        &pattern,
        &RenderSettings::default()
            .with_preview_3d(true)
            .with_three_d_style(soft_profile),
    )
    .unwrap();
    let punchy_png = render_pattern_to_png(
        &pattern,
        &RenderSettings::default()
            .with_preview_3d(true)
            .with_three_d_style(punchy_profile),
    )
    .unwrap();

    assert_ne!(
        soft_png, punchy_png,
        "3D style tuning should affect output image"
    );
}
