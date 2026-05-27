use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct ShvReader;

impl EmbroideryReader for ShvReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_shv(data)
    }
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, std::io::Error> {
    let mut b = [0u8; 1];
    cursor.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_u32_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
    let mut b = [0u8; 4];
    cursor.read_exact(&mut b)?;
    Ok(u32::from_be_bytes(b))
}

fn read_i16_be(cursor: &mut Cursor<&[u8]>) -> Result<i16, std::io::Error> {
    let mut b = [0u8; 2];
    cursor.read_exact(&mut b)?;
    Ok(i16::from_be_bytes(b))
}

fn signed8(b: u8) -> i32 {
    if b > 127 {
        b as i32 - 256
    } else {
        b as i32
    }
}

fn thread_color_from_index(idx: u8) -> u32 {
    const PALETTE: [u32; 24] = [
        0x000000, 0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0xFF00FF, 0x00FFFF,
        0x800000, 0x008000, 0x000080, 0x808000, 0x800080, 0x008080, 0x808080, 0xC0C0C0,
        0xFFA500, 0xA52A2A, 0xFFC0CB, 0x7F00FF, 0x40E0D0, 0x556B2F, 0x8B4513, 0x191970,
    ];
    PALETTE[idx as usize % PALETTE.len()]
}

fn read_shv(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);

    cursor.seek(SeekFrom::Current(0x56))?;
    let design_name_len = read_u8(&mut cursor)? as i64;
    cursor.seek(SeekFrom::Current(design_name_len))?;

    let design_width = read_u8(&mut cursor)? as u64;
    let design_height = read_u8(&mut cursor)? as u64;
    let skip_image = design_height.div_ceil(2) * design_width;
    cursor.seek(SeekFrom::Current((4 + skip_image) as i64))?;

    let color_count = read_u8(&mut cursor)? as usize;
    cursor.seek(SeekFrom::Current(18))?;

    let mut stitches_per_color = Vec::with_capacity(color_count);
    for _ in 0..color_count {
        let stitch_count = read_u32_be(&mut cursor)?;
        let color_code = read_u8(&mut cursor)?;
        pattern.add_thread(EmbThread::new(thread_color_from_index(color_code)));
        stitches_per_color.push(stitch_count);
        cursor.seek(SeekFrom::Current(9))?;
    }

    cursor.seek(SeekFrom::Current(-2))?;
    let mut in_jump = false;
    let mut stitches_since_stop: u32 = 0;
    let mut current_color = 0usize;
    let mut max_stitches = stitches_per_color.get(current_color).copied().unwrap_or(u32::MAX);

    loop {
        if stitches_since_stop >= max_stitches {
            pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
            stitches_since_stop = 0;
            current_color += 1;
            max_stitches = stitches_per_color.get(current_color).copied().unwrap_or(u32::MAX);
        }

        let b0 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let b1 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };

        if b0 == 0x80 {
            stitches_since_stop = stitches_since_stop.saturating_add(1);
            if b1 == 3 {
                continue;
            }
            if b1 == 2 {
                in_jump = false;
                continue;
            }
            if b1 == 1 {
                stitches_since_stop = stitches_since_stop.saturating_add(2);
                let sx = read_i16_be(&mut cursor)? as f32;
                let sy = read_i16_be(&mut cursor)? as f32;
                in_jump = true;
                pattern.add_stitch_relative(StitchType::Jump, sx, sy);
                continue;
            }
        }

        stitches_since_stop = stitches_since_stop.saturating_add(1);
        let dx = signed8(b0) as f32;
        let dy = signed8(b1) as f32;
        if in_jump {
            pattern.add_stitch_relative(StitchType::Jump, dx, dy);
        } else {
            pattern.add_stitch_relative(StitchType::Stitch, dx, dy);
        }
    }

    let (end_x, end_y) = pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0));
    pattern.add_stitch_absolute(StitchType::End, end_x, end_y);

    if pattern.threadlist.is_empty() {
        pattern.add_thread(EmbThread::new(0x000000));
    }

    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_shv() {
        let path = "tests/testdata/Not Mandatory/Bean.shv";
        let data = fs::read(path).expect("Failed to read test SHV file");
        let pattern = ShvReader.read(&data).expect("Failed to parse SHV file");

        assert!(pattern.stitches.len() > 10, "expected parsed SHV stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero SHV coordinates"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
