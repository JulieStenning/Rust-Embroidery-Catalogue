use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct SewReader;

impl EmbroideryReader for SewReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_sew(data)
    }
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, std::io::Error> {
    let mut b = [0u8; 1];
    cursor.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_u16_le(cursor: &mut Cursor<&[u8]>) -> Result<u16, std::io::Error> {
    let mut b = [0u8; 2];
    cursor.read_exact(&mut b)?;
    Ok(u16::from_le_bytes(b))
}

fn signed8(b: u8) -> i32 {
    if b > 127 {
        b as i32 - 256
    } else {
        b as i32
    }
}

fn thread_color_from_index(idx: u16) -> u32 {
    const PALETTE: [u32; 24] = [
        0x000000, 0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0xFF00FF, 0x00FFFF,
        0x800000, 0x008000, 0x000080, 0x808000, 0x800080, 0x008080, 0x808080, 0xC0C0C0,
        0xFFA500, 0xA52A2A, 0xFFC0CB, 0x7F00FF, 0x40E0D0, 0x556B2F, 0x8B4513, 0x191970,
    ];
    PALETTE[idx as usize % PALETTE.len()]
}

fn read_sew_stitches(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
) -> Result<(), std::io::Error> {
    loop {
        let mut b = [0u8; 2];
        if cursor.read_exact(&mut b).is_err() {
            break;
        }

        if b[0] != 0x80 {
            let dx = signed8(b[0]);
            let dy = -signed8(b[1]);
            pattern.add_stitch_relative(StitchType::Stitch, dx as f32, dy as f32);
            continue;
        }

        let control = b[1];
        let x = signed8(read_u8(cursor)?);
        let y = -signed8(read_u8(cursor)?);

        if control & 0x01 != 0 {
            pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
            continue;
        }

        if control == 0x02 || control == 0x04 {
            pattern.add_stitch_relative(StitchType::Jump, x as f32, y as f32);
            continue;
        }

        if control == 0x10 {
            pattern.add_stitch_relative(StitchType::Stitch, x as f32, y as f32);
            continue;
        }

        break;
    }

    let (end_x, end_y) = pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0));
    pattern.add_stitch_absolute(StitchType::End, end_x, end_y);
    Ok(())
}

fn read_sew(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);

    let number_of_colors = read_u16_le(&mut cursor)? as usize;
    for _ in 0..number_of_colors {
        let idx = read_u16_le(&mut cursor)?;
        pattern.add_thread(EmbThread::new(thread_color_from_index(idx)));
    }

    cursor.seek(SeekFrom::Start(0x1D78))?;
    read_sew_stitches(&mut cursor, &mut pattern)?;

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
    fn test_read_cake3_sew() {
        let path = "tests/testdata/Not Mandatory/Bean.sew";
        let data = fs::read(path).expect("Failed to read test SEW file");
        let pattern = SewReader.read(&data).expect("Failed to parse SEW file");

        assert!(pattern.stitches.len() > 10, "expected parsed SEW stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero SEW coordinates"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
