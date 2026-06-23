// use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
// ...existing code...

pub struct Vp3Reader;

impl EmbroideryReader for Vp3Reader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_vp3(data)
    }
}

use std::io::{Cursor, Read, Seek, SeekFrom};
use crate::models::{EmbPattern, EmbThread, StitchType};

fn read_vp3(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);

    // Read magic code: %vsm%\0 (6 bytes)
    let mut magic = [0u8; 6];
    cursor.read_exact(&mut magic)?;
    if magic != *b"%vsm%\0" {
        return Err("VP3: invalid file signature".into());
    }
    // skip_vp3_string: header string (Produced by...)
    skip_vp3_string(&mut cursor)?;
    cursor.seek(SeekFrom::Current(7))?;
    // skip_vp3_string: comments/note string
    skip_vp3_string(&mut cursor)?;
    cursor.seek(SeekFrom::Current(32))?;
    let center_x = signed32(read_i32_be(&mut cursor)?) as f32 / 100.0;
    let center_y = -signed32(read_i32_be(&mut cursor)?) as f32 / 100.0;
    cursor.seek(SeekFrom::Current(27))?;
    skip_vp3_string(&mut cursor)?;
    cursor.seek(SeekFrom::Current(24))?;
    skip_vp3_string(&mut cursor)?;
    let count_colors = read_u16_be(&mut cursor)?;
    for i in 0..count_colors {
        vp3_read_colorblock(&mut cursor, &mut pattern, center_x, center_y)?;
        if i + 1 < count_colors {
            pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        }
    }
    pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);
    Ok(pattern)
}

fn signed16(b0: u8, b1: u8) -> i16 {
    let b = ((b0 as u16) << 8) | (b1 as u16);
    if b > 0x7FFF {
        ((b as i32) - 0x10000) as i16
    } else {
        b as i16
    }
}

fn should_add_block_jump(abs_x: f32, abs_y: f32) -> bool {
    abs_x != 0.0 || abs_y != 0.0
}

fn vp3_long_delta_is_jump(dx: i16, dy: i16) -> bool {
    // VP3 encodes long deltas with 0x80 0x01. Deltas outside short-stitch range
    // are treated as travel moves to avoid rendering bridge artifacts.
    dx.unsigned_abs() > 127 || dy.unsigned_abs() > 127
}

fn vp3_read_colorblock(cursor: &mut Cursor<&[u8]>, pattern: &mut EmbPattern, center_x: f32, center_y: f32) -> Result<(), Box<dyn std::error::Error>> {
    let mut _bytescheck = [0u8; 3];
    let _block_start = cursor.position();
    cursor.read_exact(&mut _bytescheck)?;
    let distance_to_next_block_050 = read_i32_be(cursor)?;
    let block_end_position = distance_to_next_block_050 as u64 + cursor.position();

    let start_position_x = signed32(read_i32_be(cursor)?) as f32 / 100.0;
    let start_position_y = -signed32(read_i32_be(cursor)?) as f32 / 100.0;
    let abs_x = start_position_x + center_x;
    let abs_y = start_position_y + center_y;
    if should_add_block_jump(abs_x, abs_y) {
        pattern.add_stitch_absolute(StitchType::Jump, abs_x, abs_y);
    }
    let thread = vp3_read_thread(cursor)?;
    pattern.add_thread(thread);
    cursor.seek(SeekFrom::Current(15))?;
    cursor.read_exact(&mut _bytescheck)?;
    let stitch_byte_length = block_end_position as i64 - cursor.position() as i64;
    if stitch_byte_length < 0 {
        return Err(format!("VP3: Negative stitch_byte_length: {}", stitch_byte_length).into());
    }
    let max_bytes = (cursor.get_ref().len() as i64 - cursor.position() as i64).max(0) as usize;
    let safe_length = stitch_byte_length.min(max_bytes as i64) as usize;
    let mut stitch_bytes = vec![0u8; safe_length];
    cursor.read_exact(&mut stitch_bytes)?;
    let mut i = 0;
    while i + 1 < stitch_bytes.len() {
        let x = stitch_bytes[i];
        let y = stitch_bytes[i + 1];
        i += 2;
        if x != 0x80 {
            pattern.add_stitch_relative(StitchType::Stitch, x as i8 as f32, y as i8 as f32);
            continue;
        }
        if y == 0x01 {
            if i + 5 >= stitch_bytes.len() {
                break;
            }
            let x = signed16(stitch_bytes[i], stitch_bytes[i + 1]);
            i += 2;
            let y = signed16(stitch_bytes[i], stitch_bytes[i + 1]);
            i += 2;
            let stitch_type = if vp3_long_delta_is_jump(x, y) {
                StitchType::Jump
            } else {
                StitchType::Stitch
            };
            pattern.add_stitch_relative(stitch_type, x as f32, y as f32);
            i += 2; // skip 2 bytes (usually 0x80 0x02)
        } else if y == 0x02 {
            // Only seen after 80 01, should have been skipped. No effect.
        } else if y == 0x03 {
            pattern.add_stitch_absolute(StitchType::Trim, 0.0, 0.0);
        }
    }
    Ok(())
}

fn skip_vp3_string(cursor: &mut Cursor<&[u8]>) -> Result<(), Box<dyn std::error::Error>> {
    let len = read_u16_be(cursor)? as u64;
    cursor.seek(SeekFrom::Current(len as i64))?;
    Ok(())
}


fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1];
    cursor.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16_be(cursor: &mut Cursor<&[u8]>) -> Result<u16, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u24_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 3];
    cursor.read_exact(&mut buf)?;
    Ok(((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | buf[2] as u32)
}

fn read_i32_be(cursor: &mut Cursor<&[u8]>) -> Result<i32, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn signed32(val: i32) -> i32 {
    val
}



fn vp3_read_thread(cursor: &mut Cursor<&[u8]>) -> Result<EmbThread, Box<dyn std::error::Error>> {
    let mut thread = EmbThread::new(0);
    let colors = read_u8(cursor)?;
    let _transition = read_u8(cursor)?;
    for _ in 0..colors {
        thread.color = read_u24_be(cursor)?;
        let _parts = read_u8(cursor)?;
        let color_length = read_u16_be(cursor)? as i64;
        if color_length > 0 {
            cursor.seek(SeekFrom::Current(color_length))?;
        }
    }
    let _thread_type = read_u8(cursor)?;
    let _weight = read_u8(cursor)?;
    // Catalog number, description, brand (all as vp3 strings)
    thread.catalog_number = Some(read_vp3_string_8(cursor)?);
    thread.description = Some(read_vp3_string_8(cursor)?);
    thread.brand = Some(read_vp3_string_8(cursor)?);
    Ok(thread)
}

fn read_vp3_string_8(cursor: &mut Cursor<&[u8]>) -> Result<String, Box<dyn std::error::Error>> {
    let len = read_u16_be(cursor)? as usize;
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn vp3_colorblock_jump_is_added_when_only_one_axis_is_non_zero() {
        assert!(!should_add_block_jump(0.0, 0.0));
        assert!(should_add_block_jump(12.0, 0.0));
        assert!(should_add_block_jump(0.0, -8.5));
        assert!(should_add_block_jump(3.0, 2.0));
    }

    #[test]
    fn vp3_thread_parser_consumes_all_color_entries() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[
            0x02, 0x00, // two colors, no transition
            0x11, 0x22, 0x33, 0x00, 0x00, 0x00, // first color + parts + length
            0x44, 0x55, 0x66, 0x00, 0x00, 0x00, // second color + parts + length
            0x05, 0x28, // thread type, weight
            0x00, 0x03, b'a', b'b', b'c',
            0x00, 0x04, b'd', b'e', b's', b'c',
            0x00, 0x05, b'b', b'r', b'a', b'n', b'd',
        ]);

        let mut cursor = Cursor::new(bytes.as_slice());
        let thread = vp3_read_thread(&mut cursor).expect("VP3 thread should parse");

        assert_eq!(thread.color, 0x445566);
        assert_eq!(thread.catalog_number.as_deref(), Some("abc"));
        assert_eq!(thread.description.as_deref(), Some("desc"));
        assert_eq!(thread.brand.as_deref(), Some("brand"));
        assert_eq!(cursor.position() as usize, bytes.len());
    }

    #[test]
    fn vp3_thread_parser_skips_color_payload_bytes() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[
            0x01, 0x00, // one color, no transition
            0x11, 0x22, 0x33, // color
            0x01, // one part
            0x00, 0x02, // payload length
            0xAA, 0xBB, // payload bytes to skip
            0x05, 0x28, // thread type, weight
            0x00, 0x01, b'a',
            0x00, 0x01, b'b',
            0x00, 0x01, b'c',
        ]);

        let mut cursor = Cursor::new(bytes.as_slice());
        let thread = vp3_read_thread(&mut cursor).expect("VP3 thread with payload should parse");

        assert_eq!(thread.color, 0x112233);
        assert_eq!(thread.catalog_number.as_deref(), Some("a"));
        assert_eq!(thread.description.as_deref(), Some("b"));
        assert_eq!(thread.brand.as_deref(), Some("c"));
        assert_eq!(cursor.position() as usize, bytes.len());
    }

    #[test]
    fn vp3_long_delta_jump_threshold_matches_vp3_stitch_limit() {
        assert!(!vp3_long_delta_is_jump(127, 0));
        assert!(!vp3_long_delta_is_jump(0, -127));
        assert!(vp3_long_delta_is_jump(128, 0));
        assert!(vp3_long_delta_is_jump(0, -3200));
    }

    #[test]
    fn vp3_user_fixture_stitch_diagnostics() {
        let file_path = PathBuf::from("tests").join("testdata").join("220306.vp3");
        if !file_path.exists() {
            eprintln!("Skipping VP3 diagnostics because fixture is missing: {}", file_path.display());
            return;
        }

        let data = fs::read(&file_path).expect("should read VP3 fixture");
        let pattern = read_vp3(&data).expect("VP3 fixture should parse");

        let mut prev = (0.0_f32, 0.0_f32);
        let mut has_prev = false;
        let mut max_len = 0.0_f32;
        let mut over_127 = 0_usize;
        let mut over_255 = 0_usize;
        let mut stitch_count = 0_usize;

        for stitch in &pattern.stitches {
            if stitch.stitch_type != StitchType::Stitch {
                if stitch.stitch_type == StitchType::Jump || stitch.stitch_type == StitchType::Trim {
                    has_prev = false;
                }
                continue;
            }

            stitch_count += 1;
            if has_prev {
                let dx = stitch.x - prev.0;
                let dy = stitch.y - prev.1;
                let len = (dx * dx + dy * dy).sqrt();
                if len > max_len {
                    max_len = len;
                }
                if dx.abs() > 127.0 || dy.abs() > 127.0 {
                    over_127 += 1;
                }
                if dx.abs() > 255.0 || dy.abs() > 255.0 {
                    over_255 += 1;
                }
            }

            prev = (stitch.x, stitch.y);
            has_prev = true;
        }

        eprintln!(
            "VP3 diagnostics 220306: stitches={}, max_len={:.2}, over127={}, over255={}, jumps={}, trims={}",
            stitch_count,
            max_len,
            over_127,
            over_255,
            pattern.count_stitch_commands(StitchType::Jump),
            pattern.count_stitch_commands(StitchType::Trim),
        );

        assert_eq!(over_127, 0, "oversized stitch segments should be classified as jumps");
    }
}

