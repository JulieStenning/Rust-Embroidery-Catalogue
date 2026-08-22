use crate::error::AppError;
/// PNG rendering for embroidery previews (Rust replacement for Python PngWriter)
use crate::models::{EmbPattern, StitchType};
use image::ImageEncoder;
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
    // Match pyembroidery's default 2D thread thickness more closely so
    // satin columns render as filled thread paths rather than hairline combs.
    const THREAD_RADIUS: i32 = 2;

    for ox in -THREAD_RADIUS..=THREAD_RADIUS {
        for oy in -THREAD_RADIUS..=THREAD_RADIUS {
            if (ox * ox) + (oy * oy) > THREAD_RADIUS * THREAD_RADIUS {
                continue;
            }

            draw_antialiased_line_segment_mut(
                img,
                (from.0 + ox, from.1 + oy),
                (to.0 + ox, to.1 + oy),
                color,
                interpolate,
            );
        }
    }
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

    // Faux thread volume: shadow underlay, core, then highlight ridge.
    // Use the same disk-fill approach as draw_segment_2d so that 3D
    // has at least as much pixel coverage as 2D (3 overlapping thick
    // lines vs a single thick line).
    const THREAD_RADIUS: i32 = 2;

    // Shadow layer (offset down-right)
    for ox in -THREAD_RADIUS..=THREAD_RADIUS {
        for oy in -THREAD_RADIUS..=THREAD_RADIUS {
            if (ox * ox) + (oy * oy) > THREAD_RADIUS * THREAD_RADIUS {
                continue;
            }
            draw_antialiased_line_segment_mut(
                img,
                (
                    from.0 + ox + style.shadow_offset,
                    from.1 + oy + style.shadow_offset,
                ),
                (
                    to.0 + ox + style.shadow_offset,
                    to.1 + oy + style.shadow_offset,
                ),
                shadow,
                interpolate,
            );
        }
    }

    // Core layer (centred)
    for ox in -THREAD_RADIUS..=THREAD_RADIUS {
        for oy in -THREAD_RADIUS..=THREAD_RADIUS {
            if (ox * ox) + (oy * oy) > THREAD_RADIUS * THREAD_RADIUS {
                continue;
            }
            draw_antialiased_line_segment_mut(
                img,
                (from.0 + ox, from.1 + oy),
                (to.0 + ox, to.1 + oy),
                color,
                interpolate,
            );
        }
    }

    // Highlight layer (offset up-left)
    for ox in -THREAD_RADIUS..=THREAD_RADIUS {
        for oy in -THREAD_RADIUS..=THREAD_RADIUS {
            if (ox * ox) + (oy * oy) > THREAD_RADIUS * THREAD_RADIUS {
                continue;
            }
            draw_antialiased_line_segment_mut(
                img,
                (
                    from.0 + ox - style.highlight_offset,
                    from.1 + oy - style.highlight_offset,
                ),
                (
                    to.0 + ox - style.highlight_offset,
                    to.1 + oy - style.highlight_offset,
                ),
                highlight,
                interpolate,
            );
        }
    }
}

/// Render an embroidery pattern to PNG bytes.
pub fn render_pattern_to_png(
    pattern: &EmbPattern,
    settings: &RenderSettings,
) -> Result<Vec<u8>, AppError> {
    let (min_x, min_y, max_x, max_y) = drawable_bounds(pattern).unwrap_or((0.0, 0.0, 1.0, 1.0));
    let width = (max_x - min_x).ceil() as u32 + 4;
    let height = (max_y - min_y).ceil() as u32 + 4;
    let mut img = RgbaImage::from_pixel(width, height, settings.background);

    // Draw stitches as colored lines (2D only, one color per thread block)
    // This mimics the basic 2D preview in the Python PngWriter.
    let mut thread_index = usize::from(!pattern.threadlist.is_empty());
    let mut last_point: Option<(i32, i32)> = None;
    // Default to black if no threads
    let mut current_color = if pattern.threadlist.is_empty() {
        Rgba([0, 0, 0, 255])
    } else {
        let thread = &pattern.threadlist[0];
        Rgba([thread.get_red(), thread.get_green(), thread.get_blue(), 255])
    };
    for stitch in &pattern.stitches {
        // Color change: update thread color
        if stitch.stitch_type == StitchType::ColorChange && thread_index < pattern.threadlist.len()
        {
            let thread = &pattern.threadlist[thread_index];
            current_color = Rgba([thread.get_red(), thread.get_green(), thread.get_blue(), 255]);
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
            image::ColorType::Rgba8.into(),
        )
        .map_err(|err| AppError::parse(format!("failed to encode PNG: {err}")))?;
    Ok(buf)
}
#[cfg(test)]
#[path = "png_writer_tests.rs"]
mod tests;
