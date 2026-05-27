use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct XxxReader;

impl EmbroideryReader for XxxReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_xxx(data)
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

fn read_u32_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
    let mut b = [0u8; 4];
    cursor.read_exact(&mut b)?;
    Ok(u32::from_be_bytes(b))
}

fn signed8(v: u8) -> i8 {
    v as i8
}

fn signed16(v: u16) -> i16 {
    v as i16
}

fn read_xxx(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);

    cursor.seek(SeekFrom::Start(0x27))?;
    let num_of_colors = read_u16_le(&mut cursor)? as usize;
    cursor.seek(SeekFrom::Start(0x100))?;

    loop {
        let b1 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };

        if b1 == 0x7D || b1 == 0x7E {
            let x = signed16(read_u16_le(&mut cursor)?) as f32;
            let y = -(signed16(read_u16_le(&mut cursor)?) as f32);
            pattern.add_stitch_relative(StitchType::Jump, x, y);
            continue;
        }

        let b2 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };

        if b1 != 0x7F {
            pattern.add_stitch_relative(
                StitchType::Stitch,
                signed8(b1) as f32,
                -(signed8(b2) as f32),
            );
            continue;
        }

        let b3 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let b4 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };

        if b2 == 0x01 {
            pattern.add_stitch_relative(StitchType::Jump, signed8(b3) as f32, -(signed8(b4) as f32));
            continue;
        } else if b2 == 0x03 {
            pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
            let x = signed8(b3) as f32;
            let y = -(signed8(b4) as f32);
            if x != 0.0 || y != 0.0 {
                pattern.add_stitch_relative(StitchType::Jump, x, y);
            }
            continue;
        } else if b2 == 0x08 || (0x0A..=0x17).contains(&b2) {
            pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
            continue;
        } else if b2 == 0x7F || b2 == 0x18 {
            break;
        }
    }

    let (end_x, end_y) = pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0));
    pattern.add_stitch_absolute(StitchType::End, end_x, end_y);

    cursor.seek(SeekFrom::Current(2))?;
    for _ in 0..num_of_colors {
        let color = match read_u32_be(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        pattern.add_thread(EmbThread::new(color));
    }

    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_xxx() {
        let path = "tests/testdata/Not Mandatory/Bean.xxx";
        let data = fs::read(path).expect("Failed to read test XXX file");
        let pattern = XxxReader.read(&data).expect("Failed to parse XXX file");

        assert!(pattern.stitches.len() > 10, "expected parsed XXX stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero XXX coordinates"
        );
        assert!(
            pattern.stitches.last().map(|s| s.stitch_type) == Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
