// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
/// PNG rendering for embroidery previews (Rust replacement for Python PngWriter)
use crate::models::{EmbPattern, StitchType};
use image::ImageEncoder;
use image::{Rgba, RgbaImage};

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
    /// Target max dimension (width or height) in pixels. When `Some(N)`, the
    /// design is scaled to fit within `N` pixels with standard padding and
    /// proportional thread stroke thickness. Default is `Some(600)`.
    pub target_dimension: Option<u32>,
}

impl Default for RenderSettings {
    fn default() -> Self {
        RenderSettings {
            background: Rgba([224, 224, 224, 255]), // pale grey
            preview_mode: PreviewMode::TwoD,
            three_d_style: ThreeDStyle::default(),
            target_dimension: Some(600),
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

    pub fn with_target_dimension(mut self, target: Option<u32>) -> Self {
        self.target_dimension = target;
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

#[inline]
fn blend_pixel(dst: &mut Rgba<u8>, src: Rgba<u8>, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let a = (alpha * (src[3] as f32 / 255.0)).clamp(0.0, 1.0);
    let inv_a = 1.0 - a;
    dst[0] = (src[0] as f32 * a + dst[0] as f32 * inv_a).round() as u8;
    dst[1] = (src[1] as f32 * a + dst[1] as f32 * inv_a).round() as u8;
    dst[2] = (src[2] as f32 * a + dst[2] as f32 * inv_a).round() as u8;
    dst[3] = ((src[3] as f32 * a + dst[3] as f32 * inv_a).round() as u8).max(dst[3]);
}

/// Fast, single-pass antialiased capsule (thick line with rounded caps) rasterizer.
fn draw_capsule_antialiased(
    img: &mut RgbaImage,
    from: (i32, i32),
    to: (i32, i32),
    color: Rgba<u8>,
    radius: f32,
) {
    if radius <= 0.0 {
        return;
    }
    let width = img.width() as i32;
    let height = img.height() as i32;

    let x1 = from.0 as f32;
    let y1 = from.1 as f32;
    let x2 = to.0 as f32;
    let y2 = to.1 as f32;

    let min_x = ((x1.min(x2) - radius - 1.0).floor() as i32).clamp(0, width - 1);
    let max_x = ((x1.max(x2) + radius + 1.0).ceil() as i32).clamp(0, width - 1);
    let min_y = ((y1.min(y2) - radius - 1.0).floor() as i32).clamp(0, height - 1);
    let max_y = ((y1.max(y2) + radius + 1.0).ceil() as i32).clamp(0, height - 1);

    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;

    let r_inner = (radius - 0.5).max(0.0);
    let r_outer = radius + 0.5;

    for py in min_y..=max_y {
        let p_y = py as f32 + 0.5;
        for px in min_x..=max_x {
            let p_x = px as f32 + 0.5;

            let dist = if len_sq < 1e-5 {
                let d_x = p_x - x1;
                let d_y = p_y - y1;
                (d_x * d_x + d_y * d_y).sqrt()
            } else {
                let t = (((p_x - x1) * dx + (p_y - y1) * dy) / len_sq).clamp(0.0, 1.0);
                let proj_x = x1 + t * dx;
                let proj_y = y1 + t * dy;
                let d_x = p_x - proj_x;
                let d_y = p_y - proj_y;
                (d_x * d_x + d_y * d_y).sqrt()
            };

            if dist < r_outer {
                let alpha = if dist <= r_inner {
                    1.0
                } else {
                    (r_outer - dist).clamp(0.0, 1.0)
                };
                let pixel = img.get_pixel_mut(px as u32, py as u32);
                blend_pixel(pixel, color, alpha);
            }
        }
    }
}

fn draw_segment_2d(
    img: &mut RgbaImage,
    from: (i32, i32),
    to: (i32, i32),
    color: Rgba<u8>,
    thread_radius: i32,
) {
    draw_capsule_antialiased(img, from, to, color, thread_radius as f32);
}

fn draw_segment_3d(
    img: &mut RgbaImage,
    from: (i32, i32),
    to: (i32, i32),
    color: Rgba<u8>,
    style: ThreeDStyle,
    thread_radius: i32,
) {
    let shadow = darken_color(color, style.shadow_strength);
    let highlight = lighten_color(color, style.highlight_strength);

    let shadow_offset =
        ((style.shadow_offset as f32 * thread_radius as f32 / 2.0).round() as i32).max(1);
    let highlight_offset =
        ((style.highlight_offset as f32 * thread_radius as f32 / 2.0).round() as i32).max(1);

    let r = thread_radius as f32;

    // 1. Shadow layer (offset down-right)
    draw_capsule_antialiased(
        img,
        (from.0 + shadow_offset, from.1 + shadow_offset),
        (to.0 + shadow_offset, to.1 + shadow_offset),
        shadow,
        r,
    );

    // 2. Core layer (centered)
    draw_capsule_antialiased(img, from, to, color, r);

    // 3. Highlight layer (offset up-left, slightly thinner sheen)
    draw_capsule_antialiased(
        img,
        (from.0 - highlight_offset, from.1 - highlight_offset),
        (to.0 - highlight_offset, to.1 - highlight_offset),
        highlight,
        (r * 0.75).max(0.8),
    );
}

/// Render an embroidery pattern to PNG bytes.
pub fn render_pattern_to_png(
    pattern: &EmbPattern,
    settings: &RenderSettings,
) -> Result<Vec<u8>, AppError> {
    let bounds = drawable_bounds(pattern);
    let (min_x, min_y, max_x, max_y) = bounds.unwrap_or((0.0, 0.0, 0.0, 0.0));
    let span_x = (max_x - min_x).max(0.0);
    let span_y = (max_y - min_y).max(0.0);
    let max_span = span_x.max(span_y);

    let (width, height, scale, margin, thread_radius) = match settings.target_dimension {
        Some(target_dim) if max_span > 0.0 => {
            let target_f = target_dim as f32;
            let margin = (target_f * 0.05).round().max(8.0);
            let inner_max = (target_f - 2.0 * margin).max(1.0);
            let scale = inner_max / max_span;
            let width = ((span_x * scale) + 2.0 * margin).round().max(16.0) as u32;
            let height = ((span_y * scale) + 2.0 * margin).round().max(16.0) as u32;

            // #40 thread diameter is ~0.4 mm (= 4.0 decimillimeter units).
            // Scale thread radius with the coordinate-to-pixel scale factor.
            let thread_radius = ((4.0 * scale) / 2.0).round() as i32;
            let thread_radius = thread_radius.clamp(1, 4);

            (width, height, scale, margin, thread_radius)
        }
        Some(_) => (64, 64, 1.0, 8.0, 2),
        None => {
            let width = span_x.ceil() as u32 + 4;
            let height = span_y.ceil() as u32 + 4;
            (width.max(1), height.max(1), 1.0, 2.0, 2)
        }
    };

    let mut img = RgbaImage::from_pixel(width, height, settings.background);

    // If no drawable stitches, return the background canvas
    if bounds.is_some() {
        let mut thread_index = usize::from(!pattern.threadlist.is_empty());
        let mut last_point: Option<(i32, i32)> = None;
        let mut current_color = if pattern.threadlist.is_empty() {
            Rgba([0, 0, 0, 255])
        } else {
            let thread = &pattern.threadlist[0];
            Rgba([thread.get_red(), thread.get_green(), thread.get_blue(), 255])
        };

        for stitch in &pattern.stitches {
            if stitch.stitch_type == StitchType::ColorChange
                && thread_index < pattern.threadlist.len()
            {
                let thread = &pattern.threadlist[thread_index];
                current_color =
                    Rgba([thread.get_red(), thread.get_green(), thread.get_blue(), 255]);
                thread_index += 1;
                last_point = None;
                continue;
            }

            if stitch.stitch_type == StitchType::Stitch {
                let x = ((stitch.x - min_x) * scale + margin).round() as i32;
                let y = ((stitch.y - min_y) * scale + margin).round() as i32;
                if let Some((lx, ly)) = last_point {
                    if settings.preview_mode == PreviewMode::ThreeD {
                        draw_segment_3d(
                            &mut img,
                            (lx, ly),
                            (x, y),
                            current_color,
                            settings.three_d_style,
                            thread_radius,
                        );
                    } else {
                        draw_segment_2d(&mut img, (lx, ly), (x, y), current_color, thread_radius);
                    }
                }
                last_point = Some((x, y));
            } else if stitch.stitch_type == StitchType::Jump
                || stitch.stitch_type == StitchType::Trim
            {
                last_point = None;
            }
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
