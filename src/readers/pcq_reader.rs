use std::io::{Cursor, Read};
use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

pub struct PcqReader;

impl EmbroideryReader for PcqReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_pcq(data)
    }
}

fn read_pcq(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
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

    fn read_u24_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
        let mut b = [0u8; 3];
        cursor.read_exact(&mut b)?;
        Ok(((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32))
    }

    fn read_u24_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
        let mut b = [0u8; 3];
        cursor.read_exact(&mut b)?;
        Ok((b[0] as u32) | ((b[1] as u32) << 8) | ((b[2] as u32) << 16))
    }

    fn signed24(v: u32) -> i32 {
        let v = v & 0x00FF_FFFF;
        if v > 0x7F_FFFF {
            -((0x01_00_00_00 - v) as i32)
        } else {
            v as i32
        }
    }

    const PC_SIZE_CONVERSION_RATIO: f32 = 5.0 / 3.0;

    let mut pattern = EmbPattern::new();

    let mut cursor = Cursor::new(data);
    let _version = match read_u8(&mut cursor) {
        Ok(v) => v,
        Err(_) => return Ok(pattern),
    };
    let _hoop_size = read_u8(&mut cursor).ok();
    let color_count = match read_u16_le(&mut cursor) {
        Ok(v) => v as usize,
        Err(_) => return Ok(pattern),
    };

    for _ in 0..color_count {
        let color = match read_u24_be(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        pattern.add_thread(EmbThread::new(color));
        let _ = read_u8(&mut cursor).ok();
    }

    let _stitch_count = read_u16_le(&mut cursor).ok();

    loop {
        let _c0 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let x = match read_u24_le(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let _c1 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let y = match read_u24_le(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let ctrl = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };

        let x = signed24(x) as f32 * PC_SIZE_CONVERSION_RATIO;
        let y = -(signed24(y) as f32) * PC_SIZE_CONVERSION_RATIO;

        if ctrl == 0x00 {
            pattern.add_stitch_absolute(StitchType::Stitch, x, y);
            continue;
        }
        if ctrl & 0x01 != 0 {
            pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
            continue;
        }
        if ctrl & 0x04 != 0 {
            pattern.add_stitch_absolute(StitchType::Jump, x, y);
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

    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_pcq() {
        let path = "tests/testdata/Not Mandatory/Bean.pcq";
        let data = fs::read(path).expect("Failed to read test PCQ file");
        let pattern = PcqReader.read(&data).expect("Failed to parse PCQ file");

        assert!(pattern.stitches.len() > 10, "expected parsed PCQ stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero PCQ coordinates"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
