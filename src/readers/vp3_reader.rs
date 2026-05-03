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
    if abs_x != 0.0 || abs_y != 0.0 {
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
            let x = signed16(stitch_bytes[i], stitch_bytes[i + 1]);
            i += 2;
            let y = signed16(stitch_bytes[i], stitch_bytes[i + 1]);
            i += 2;
            pattern.add_stitch_relative(StitchType::Stitch, x as f32, y as f32);
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
    let _colors = read_u8(cursor)?;
    let _transition = read_u8(cursor)?;
    // skip color transitions (not used)
    // Only one color is used in most files
    let _parts = read_u8(cursor)?;
    let _color_length = read_u16_be(cursor)?;
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

