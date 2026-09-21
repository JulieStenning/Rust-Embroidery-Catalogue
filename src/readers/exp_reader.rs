// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::readers::embroidery_reader::EmbroideryReader;

pub struct ExpReader;

impl EmbroideryReader for ExpReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, crate::error::AppError> {
        read_exp(data)
            .map_err(|err| crate::error::AppError::parse(format!("EXP parse failed: {err}")))
    }
}
use std::io::Cursor;

use crate::models::{EmbPattern, StitchType};

/// Bright fallback palette used when EXP files do not embed explicit thread colours.
const EXP_FALLBACK_PALETTE: [u32; 24] = [
    0x1F77B4, 0xD62728, 0x2CA02C, 0xFF7F0E, 0x9467BD, 0x8C564B, 0xE377C2, 0x17BECF, 0xBCBD22,
    0x7F7F7F, 0x00A651, 0xED1C24, 0x1C75BC, 0xFBB03B, 0x662D91, 0x39B54A, 0xF15A24, 0xA349A4,
    0x00AEEF, 0xC69C6D, 0xEF4136, 0x22B573, 0x2E3192, 0xFFF200,
];

// ---------------------------------------------------------------------------
// Low-level helpers
// ---------------------------------------------------------------------------

/// Interpret a byte as a signed 8-bit integer (mirrors Python `signed8`).
#[inline]
fn signed8(b: u8) -> i16 {
    if b > 127 {
        -256 + b as i16
    } else {
        b as i16
    }
}

/// Read exactly `n` bytes from the cursor into a Vec, or return an error on EOF.
fn read_exact(cursor: &mut Cursor<&[u8]>, n: usize) -> Result<Vec<u8>, binrw::Error> {
    let pos = cursor.position();
    let data = cursor.get_ref();
    let end = pos as usize + n;
    if end > data.len() {
        return Err(binrw::Error::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "unexpected end of EXP stitch data",
        )));
    }
    let bytes = data[pos as usize..end].to_vec();
    cursor.set_position(end as u64);
    Ok(bytes)
}

// ---------------------------------------------------------------------------
// EXP stitch reader
// ---------------------------------------------------------------------------

/// Read EXP stitch data from the cursor into the pattern.
///
/// EXP (Melco Expanded) uses 2-byte records:
/// - If `byte[0]` != 0x80: regular stitch with signed 8-bit deltas.
/// - If `byte[0]` == 0x80: control command; `byte[1]` is the control code,
///   followed by 2 extra bytes encoding a position/delta.
pub fn read_exp_stitches(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
) -> Result<(), binrw::Error> {
    loop {
        if cursor.position() as usize >= cursor.get_ref().len() {
            break;
        }

        let bytes = match read_exact(cursor, 2) {
            Ok(b) => b,
            Err(_) => break,
        };

        if bytes[0] != 0x80 {
            // Regular stitch (relative delta)
            let x = signed8(bytes[0]) as f32;
            let y = -(signed8(bytes[1]) as f32);
            pattern.add_stitch_relative(StitchType::Stitch, x, y);
            continue;
        }

        // Control command
        let control = bytes[1];

        let extra = match read_exact(cursor, 2) {
            Ok(b) => b,
            Err(_) => break,
        };
        let x = signed8(extra[0]) as f32;
        let y = -(signed8(extra[1]) as f32);

        match control {
            0x80 => {
                // Trim
                pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
            }
            0x02 => {
                // This shouldn't exist, but treat as stitch.
                pattern.add_stitch_relative(StitchType::Stitch, x, y);
            }
            0x04 => {
                // Jump
                pattern.add_stitch_relative(StitchType::Jump, x, y);
            }
            0x01 => {
                // Color change
                pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
                if x != 0.0 || y != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, x, y);
                }
            }
            _ => {
                // Some EXP variants contain vendor-specific control bytes.
                // Consume the record and continue rather than truncating parse.
                if x != 0.0 || y != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, x, y);
                }
            }
        }
    }

    if let Some(last) = pattern.stitches.last() {
        pattern.add_stitch_absolute(StitchType::End, last.x, last.y);
    } else {
        pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Public entry-point
// ---------------------------------------------------------------------------

/// Parse an EXP-format Melco embroidery file from a byte buffer.
///
/// Returns an [`EmbPattern`] containing the stitches read from the file.
///
/// # Errors
///
/// Returns a [`binrw::Error`] if the data is truncated or malformed.
pub fn read_exp(data: &[u8]) -> Result<EmbPattern, binrw::Error> {
    let mut cursor = Cursor::new(data);
    let mut pattern = EmbPattern::new();

    read_exp_stitches(&mut cursor, &mut pattern)?;

    // If no threads are declared in-file, synthesize preview colors from color blocks.
    let num_colour_changes = pattern
        .stitches
        .iter()
        .filter(|s| s.stitch_type == StitchType::ColorChange)
        .count();
    if pattern.threadlist.is_empty() && num_colour_changes > 0 {
        for i in 0..=num_colour_changes {
            let color = EXP_FALLBACK_PALETTE[i % EXP_FALLBACK_PALETTE.len()];
            pattern
                .threadlist
                .push(crate::models::EmbThread::new(color));
        }
    }

    Ok(pattern)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
#[path = "exp_reader_tests.rs"]
mod tests;
