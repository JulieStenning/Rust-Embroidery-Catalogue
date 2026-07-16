use crate::readers::embroidery_reader::EmbroideryReader;

pub struct ExpReader;

impl EmbroideryReader for ExpReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        Ok(read_exp(data)?)
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
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

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

    #[test]
    fn test_read_exp_color_change_zero_delta_does_not_emit_jump() {
        // Color change with zero movement should not create a jump.
        let data = vec![0x80, 0x01, 0x00, 0x00];

        let pattern = read_exp(&data).expect("should parse EXP zero-delta color change");

        assert_eq!(pattern.count_stitch_commands(StitchType::ColorChange), 1);
        assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 0);
    }

    #[test]
    fn test_read_exp_unknown_control_does_not_abort_and_consumes_record() {
        // Unknown control should not truncate parsing.
        // 0x80 0x10 + dx/dy then a regular stitch.
        let data = vec![0x80, 0x10, 0x02, 0xFE, 0x03, 0xFD];

        let pattern = read_exp(&data).expect("should parse unknown control robustly");

        assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 1);
        assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
    }

    #[test]
    fn test_read_exp_color_change_preserves_current_position() {
        // Stitch to (10, 10), color change, then stitch +2,+3 -> should end at (12,13)
        let data = vec![0x0A, 0xF6, 0x80, 0x01, 0x00, 0x00, 0x02, 0xFD];

        let pattern = read_exp(&data).expect("should parse EXP with color change");

        let stitches: Vec<_> = pattern
            .stitches
            .iter()
            .filter(|s| s.stitch_type == StitchType::Stitch)
            .collect();
        assert_eq!(stitches.len(), 2);
        assert_eq!(stitches[0].x, 10.0);
        assert_eq!(stitches[0].y, 10.0);
        assert_eq!(stitches[1].x, 12.0);
        assert_eq!(stitches[1].y, 13.0);
    }

    #[test]
    fn test_read_exp_trim_preserves_current_position() {
        // Stitch to (6, 6), trim, then stitch +1,+1 -> should end at (7,7)
        let data = vec![0x06, 0xFA, 0x80, 0x80, 0x00, 0x00, 0x01, 0xFF];

        let pattern = read_exp(&data).expect("should parse EXP with trim");

        let stitches: Vec<_> = pattern
            .stitches
            .iter()
            .filter(|s| s.stitch_type == StitchType::Stitch)
            .collect();
        assert_eq!(stitches.len(), 2);
        assert_eq!(stitches[0].x, 6.0);
        assert_eq!(stitches[0].y, 6.0);
        assert_eq!(stitches[1].x, 7.0);
        assert_eq!(stitches[1].y, 7.0);
    }

    #[test]
    fn test_real_peacock_fixture_control_commands_preserve_position() {
        let file_path = PathBuf::from("tests")
            .join("testdata")
            .join("01expPeacock.exp");
        assert!(
            file_path.exists(),
            "expected 01expPeacock.exp fixture to exist"
        );

        let data = fs::read(&file_path).expect("should read EXP fixture");
        let pattern = read_exp(&data).expect("should parse real EXP fixture");

        for index in 1..pattern.stitches.len() {
            let prev = &pattern.stitches[index - 1];
            let current = &pattern.stitches[index];

            if current.stitch_type == StitchType::ColorChange
                || current.stitch_type == StitchType::Trim
            {
                assert_eq!(
                    (current.x, current.y),
                    (prev.x, prev.y),
                    "control command at index {} should keep current position",
                    index
                );
            }
        }
    }

    #[test]
    fn test_exp_fallback_threads_use_palette_not_black_only() {
        // Two color blocks => three fallback threads should be synthesized.
        let data = vec![
            0x01, 0xFF, // stitch
            0x80, 0x01, 0x00, 0x00, // color change
            0x01, 0xFF, // stitch
            0x80, 0x01, 0x00, 0x00, // color change
            0x01, 0xFF, // stitch
        ];

        let pattern = read_exp(&data).expect("should parse EXP and add fallback threads");
        assert_eq!(pattern.threadlist.len(), 3);

        let all_black = pattern
            .threadlist
            .iter()
            .all(|thread| thread.color == 0x000000);
        assert!(!all_black, "fallback EXP threads should not all be black");
    }
}
