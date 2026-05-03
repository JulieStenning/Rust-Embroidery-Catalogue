#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EmbPattern, Stitch, StitchType, EmbThread};

    fn count_non_bg_pixels(png_bytes: &[u8], bg: Rgba<u8>) -> usize {
        let img = image::load_from_memory(png_bytes).expect("decode png").to_rgba8();
        img.pixels().filter(|p| **p != bg).count()
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
}
/// PNG rendering for embroidery previews (Rust replacement for Python PngWriter)

use crate::models::{EmbPattern, EmbThread, StitchType};
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;

/// Settings for rendering the embroidery preview.
#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub render_3d: bool,
    pub show_fabric: bool,
    pub thread_width: u32,
    pub background: Rgba<u8>,
    // Add more settings as needed
}

impl Default for RenderSettings {
    fn default() -> Self {
        RenderSettings {
            render_3d: true,
            show_fabric: true,
            thread_width: 5,
            background: Rgba([224, 224, 224, 255]), // pale grey
        }
    }
}

/// Render an embroidery pattern to PNG bytes.
pub fn render_pattern_to_png(pattern: &EmbPattern, settings: &RenderSettings) -> Vec<u8> {
    let (min_x, min_y, max_x, max_y) = pattern.bounds();
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
                draw_antialiased_line_segment_mut(
                    &mut img,
                    (lx, ly),
                    (x, y),
                    current_color,
                    interpolate,
                );
            }
            last_point = Some((x, y));
        } else if stitch.stitch_type == StitchType::Jump || stitch.stitch_type == StitchType::Trim {
            // Discontinuity: do not connect lines
            last_point = None;
        }
    }

    let mut buf = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buf)
        .encode(
            &img,
            img.width(),
            img.height(),
            image::ColorType::Rgba8,
        )
        .expect("PNG encoding failed");
    buf
}
