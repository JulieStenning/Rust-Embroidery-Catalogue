use crate::readers::embroidery_reader::EmbroideryReader;

pub struct ExpReader;

impl EmbroideryReader for ExpReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        Ok(read_exp(data)?)
    }
}
use std::io::Cursor;

use crate::models::{EmbPattern, StitchType};

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
/// - If byte[0] != 0x80: regular stitch with signed 8-bit deltas.
/// - If byte[0] == 0x80: control command; byte[1] is the control code,
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
                pattern.add_stitch_absolute(StitchType::Trim, 0.0, 0.0);
            }
            0x02 => {
                // This shouldn't exist, but treat as stitch
                pattern.add_stitch_relative(StitchType::Stitch, x, y);
            }
            0x04 => {
                // Jump
                pattern.add_stitch_relative(StitchType::Jump, x, y);
            }
            0x01 => {
                // Color change
                pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
                if x != 0.0 || y != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, x, y);
                }
            }
            _ => {
                // Uncaught control — stop parsing (matching Python behaviour)
                break;
            }
        }
    }

    pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);

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

    // If no threads, but color changes exist, add placeholder threads
    let num_colour_changes = pattern.stitches.iter().filter(|s| s.stitch_type == StitchType::ColorChange).count();
    if pattern.threadlist.is_empty() && num_colour_changes > 0 {
        for _ in 0..=num_colour_changes {
            pattern.threadlist.push(crate::models::EmbThread::new(0x000000));
        }
    }

    Ok(pattern)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_exp_two_stitches() {
        // Two regular stitches:
        // Stitch 1: dx=5, dy=10 → bytes [0x05, 0xF6]
        //   signed8(0x05)=5, signed8(0xF6)=246-256=-10, dy = -(-10) = 10
        // Stitch 2: dx=-3, dy=7 → bytes [0xFD, 0xF9]
        //   signed8(0xFD)=253-256=-3, signed8(0xF9)=249-256=-7, dy = -(-7) = 7
        let data = vec![0x05, 0xF6, 0xFD, 0xF9];

        let pattern = read_exp(&data).expect("should parse valid EXP");

        assert_eq!(
            pattern.count_stitch_commands(StitchType::Stitch),
            2,
            "expected exactly 2 regular stitches"
        );

        let stitches: Vec<_> = pattern
            .stitches
            .iter()
            .filter(|s| s.stitch_type == StitchType::Stitch)
            .collect();

        assert_eq!(stitches.len(), 2);
        assert_eq!(stitches[0].x, 5.0);
        assert_eq!(stitches[0].y, 10.0);
        assert_eq!(stitches[1].x, 2.0);
        assert_eq!(stitches[1].y, 17.0);

        // Always appends End
        assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
    }

    #[test]
    fn test_read_exp_jump() {
        // Jump command: 0x80 0x04 + dx dy
        let data = vec![0x80, 0x04, 0x0A, 0xF6]; // dx=10, dy=10

        let pattern = read_exp(&data).expect("should parse EXP with jump");

        assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);

        let jumps: Vec<_> = pattern
            .stitches
            .iter()
            .filter(|s| s.stitch_type == StitchType::Jump)
            .collect();
        assert_eq!(jumps.len(), 1);
        assert_eq!(jumps[0].x, 10.0);
        assert_eq!(jumps[0].y, 10.0);
    }

    #[test]
    fn test_read_exp_trim() {
        // Trim command: 0x80 0x80 + 2 extra bytes (ignored)
        let data = vec![0x80, 0x80, 0x00, 0x00];

        let pattern = read_exp(&data).expect("should parse EXP with trim");

        assert_eq!(pattern.count_stitch_commands(StitchType::Trim), 1);
    }

    #[test]
    fn test_read_exp_color_change() {
        // Color change with non-zero coords: 0x80 0x01 + dx dy
        // Expect ColorChange + Jump
        let data = vec![0x80, 0x01, 0x05, 0xFB]; // dx=5, dy=5

        let pattern = read_exp(&data).expect("should parse EXP with color change");

        assert_eq!(pattern.count_stitch_commands(StitchType::ColorChange), 1);
        assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
    }
}