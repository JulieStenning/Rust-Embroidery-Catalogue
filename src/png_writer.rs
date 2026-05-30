use image::ImageEncoder;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EmbPattern, Stitch, StitchType, EmbThread};

    fn count_non_bg_pixels(png_bytes: &[u8], bg: Rgba<u8>) -> usize {
        let img = image::load_from_memory(png_bytes).expect("decode png").to_rgba8();
        img.pixels().filter(|p| **p != bg).count()
    }

    fn image_dimensions(png_bytes: &[u8]) -> (u32, u32) {
        let img = image::load_from_memory(png_bytes).expect("decode png").to_rgba8();
        (img.width(), img.height())
    }

    #[test]
    fn renders_simple_pattern() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.threadlist.push(EmbThread::new(0xFF0000)); // Red
        let settings = RenderSettings::default();
        let png = render_pattern_to_png(&pattern, &settings);
        assert!(count_non_bg_pixels(&png, settings.background) > 0, "Should render visible line");
    }

    #[test]
    fn renders_blank_for_no_stitches() {
        let pattern = EmbPattern::new();
        let settings = RenderSettings::default();
        let png = render_pattern_to_png(&pattern, &settings);
        assert_eq!(count_non_bg_pixels(&png, settings.background), 0, "No stitches should be blank");
    }

    #[test]
    fn uses_default_color_if_no_threads() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 0.0, stitch_type: StitchType::Stitch });
        let settings = RenderSettings::default();
        let png = render_pattern_to_png(&pattern, &settings);
        // Should not be blank
        assert!(count_non_bg_pixels(&png, settings.background) > 0, "Should render with default color");
    }

    #[test]
    fn renders_color_change_segments() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 10.0, stitch_type: StitchType::ColorChange });
        pattern.stitches.push(Stitch { x: 20.0, y: 10.0, stitch_type: StitchType::Stitch });
        pattern.threadlist.push(EmbThread::new(0xFF0000)); // Red
        pattern.threadlist.push(EmbThread::new(0x0000FF)); // Blue
        let settings = RenderSettings::default();
        let png = render_pattern_to_png(&pattern, &settings);
        // Should not be blank
        assert!(count_non_bg_pixels(&png, settings.background) > 0, "Should render with color changes");
    }

    #[test]
    fn does_not_panic_on_empty_pattern() {
        let pattern = EmbPattern::new();
        let settings = RenderSettings::default();
        let _ = render_pattern_to_png(&pattern, &settings);
    }

    #[test]
    fn ignores_jump_only_outliers_when_framing_preview() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 10000.0, y: 10000.0, stitch_type: StitchType::Jump });
        pattern.stitches.push(Stitch { x: 12.0, y: 1.0, stitch_type: StitchType::Stitch });
        pattern.threadlist.push(EmbThread::new(0x00AA00));

        let settings = RenderSettings::default();
        let png = render_pattern_to_png(&pattern, &settings);
        let (width, height) = image_dimensions(&png);

        assert!(width < 200 && height < 200, "jump-only outlier should not inflate preview size");
    }

    #[test]
    fn preview_3d_mode_produces_distinct_image_from_2d() {
        let mut pattern = EmbPattern::new();
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 24.0, y: 8.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 36.0, y: 14.0, stitch_type: StitchType::Stitch });
        pattern.threadlist.push(EmbThread::new(0x1E88E5));

        let settings_2d = RenderSettings::default().with_preview_3d(false);
        let settings_3d = RenderSettings::default().with_preview_3d(true);

        let png_2d = render_pattern_to_png(&pattern, &settings_2d);
        let png_3d = render_pattern_to_png(&pattern, &settings_3d);

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
        pattern.stitches.push(Stitch { x: 0.0, y: 0.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 18.0, y: 8.0, stitch_type: StitchType::Stitch });
        pattern.stitches.push(Stitch { x: 30.0, y: 12.0, stitch_type: StitchType::Stitch });
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
        );
        let punchy_png = render_pattern_to_png(
            &pattern,
            &RenderSettings::default()
                .with_preview_3d(true)
                .with_three_d_style(punchy_profile),
        );

        assert_ne!(soft_png, punchy_png, "3D style tuning should affect output image");
    }
}
/// PNG rendering for embroidery previews (Rust replacement for Python PngWriter)

use crate::models::{EmbPattern, StitchType};
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    TwoD,
    ThreeD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThreeDStyle {
    pub shadow_strength: u8,
    pub highlight_strength: u8,
    pub core_half_width: i32,
    pub shadow_offset: i32,
    pub highlight_offset: i32,
}

impl Default for ThreeDStyle {
    fn default() -> Self {
        // Balanced default profile: improved thread volume with moderate contrast.
        Self {
            shadow_strength: 44,
            highlight_strength: 30,
            core_half_width: 1,
            shadow_offset: 1,
            highlight_offset: 1,
        }
    }
}

fn drawable_bounds(pattern: &EmbPattern) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut found = false;

    for stitch in &pattern.stitches {
        if stitch.stitch_type != StitchType::Stitch {
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
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

/// Settings for rendering the embroidery preview.
#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub background: Rgba<u8>,
    pub preview_mode: PreviewMode,
    pub three_d_style: ThreeDStyle,
}

impl Default for RenderSettings {
    fn default() -> Self {
        RenderSettings {
            background: Rgba([224, 224, 224, 255]), // pale grey
            preview_mode: PreviewMode::TwoD,
            three_d_style: ThreeDStyle::default(),
        }
    }
}

impl RenderSettings {
    pub fn with_preview_3d(mut self, preview_3d: bool) -> Self {
        self.preview_mode = if preview_3d {
            PreviewMode::ThreeD
        } else {
            PreviewMode::TwoD
        };
        self
    }

    pub fn with_three_d_style(mut self, style: ThreeDStyle) -> Self {
        self.three_d_style = style;
        self
    }
}

fn darken_color(color: Rgba<u8>, amount: u8) -> Rgba<u8> {
    Rgba([
        color[0].saturating_sub(amount),
        color[1].saturating_sub(amount),
        color[2].saturating_sub(amount),
        color[3],
    ])
}

fn lighten_color(color: Rgba<u8>, amount: u8) -> Rgba<u8> {
    Rgba([
        color[0].saturating_add(amount),
        color[1].saturating_add(amount),
        color[2].saturating_add(amount),
        color[3],
    ])
}

fn draw_segment_2d(img: &mut RgbaImage, from: (i32, i32), to: (i32, i32), color: Rgba<u8>) {
    draw_antialiased_line_segment_mut(img, from, to, color, interpolate);
}

fn draw_segment_3d(
    img: &mut RgbaImage,
    from: (i32, i32),
    to: (i32, i32),
    color: Rgba<u8>,
    style: ThreeDStyle,
) {
    let shadow = darken_color(color, style.shadow_strength);
    let highlight = lighten_color(color, style.highlight_strength);

    // Faux thread volume: shadow underlay, widened core, then highlight ridge.
    let shadow_offset = style.shadow_offset;
    let highlight_offset = style.highlight_offset;

    draw_antialiased_line_segment_mut(
        img,
        (from.0 + shadow_offset, from.1 + shadow_offset),
        (to.0 + shadow_offset, to.1 + shadow_offset),
        shadow,
        interpolate,
    );

    for offset in -style.core_half_width..=style.core_half_width {
        draw_antialiased_line_segment_mut(
            img,
            (from.0 + offset, from.1),
            (to.0 + offset, to.1),
            color,
            interpolate,
        );
    }

    draw_antialiased_line_segment_mut(img, from, to, color, interpolate);
    draw_antialiased_line_segment_mut(
        img,
        (from.0 - highlight_offset, from.1 - highlight_offset),
        (to.0 - highlight_offset, to.1 - highlight_offset),
        highlight,
        interpolate,
    );
}

/// Render an embroidery pattern to PNG bytes.
pub fn render_pattern_to_png(pattern: &EmbPattern, settings: &RenderSettings) -> Vec<u8> {
    let (min_x, min_y, max_x, max_y) = drawable_bounds(pattern).unwrap_or((0.0, 0.0, 1.0, 1.0));
    let width = (max_x - min_x).ceil() as u32 + 4;
    let height = (max_y - min_y).ceil() as u32 + 4;
    let mut img = RgbaImage::from_pixel(width, height, settings.background);

    // Draw stitches as colored lines (2D only, one color per thread block)
    // This mimics the basic 2D preview in the Python PngWriter.
    let mut thread_index = 0;
    let mut last_point: Option<(i32, i32)> = None;
    // Default to black if no threads
    let mut current_color = if pattern.threadlist.is_empty() {
        Rgba([0, 0, 0, 255])
    } else {
        let thread = &pattern.threadlist[0];
        Rgba([
            thread.get_red(),
            thread.get_green(),
            thread.get_blue(),
            255,
        ])
    };
    for stitch in &pattern.stitches {
        // Color change: update thread color
        if stitch.stitch_type == StitchType::ColorChange && thread_index < pattern.threadlist.len() {
            let thread = &pattern.threadlist[thread_index];
            current_color = Rgba([
                thread.get_red(),
                thread.get_green(),
                thread.get_blue(),
                255,
            ]);
            thread_index += 1;
            last_point = None;
            continue;
        }
        // Only draw actual stitches
        if stitch.stitch_type == StitchType::Stitch {
            let x = (stitch.x - min_x + 2.0).round() as i32;
            let y = (stitch.y - min_y + 2.0).round() as i32;
            if let Some((lx, ly)) = last_point {
                if settings.preview_mode == PreviewMode::ThreeD {
                    draw_segment_3d(
                        &mut img,
                        (lx, ly),
                        (x, y),
                        current_color,
                        settings.three_d_style,
                    );
                } else {
                    draw_segment_2d(&mut img, (lx, ly), (x, y), current_color);
                }
            }
            last_point = Some((x, y));
        } else if stitch.stitch_type == StitchType::Jump || stitch.stitch_type == StitchType::Trim {
            // Discontinuity: do not connect lines
            last_point = None;
        }
    }
    let mut buf = Vec::new();
    use image::codecs::png::PngEncoder;
    PngEncoder::new(&mut buf)
        .write_image(
            &img,
            img.width(),
            img.height(),
            image::ColorType::Rgba8,
        )
        .expect("PNG encoding failed");
    buf
}
