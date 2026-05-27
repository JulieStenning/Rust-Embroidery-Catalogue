use std::io::{Cursor, Read, Seek, SeekFrom};
use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

pub struct PcmReader;

impl EmbroideryReader for PcmReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_pcm(data)
    }
}

fn read_pcm(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, std::io::Error> {
        let mut b = [0u8; 1];
        cursor.read_exact(&mut b)?;
        Ok(b[0])
    }

    fn read_u16_be(cursor: &mut Cursor<&[u8]>) -> Result<u16, std::io::Error> {
        let mut b = [0u8; 2];
        cursor.read_exact(&mut b)?;
        Ok(u16::from_be_bytes(b))
    }

    fn read_u24_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
        let mut b = [0u8; 3];
        cursor.read_exact(&mut b)?;
        Ok(((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32))
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
    const PCM_THREAD_COLORS: [u32; 16] = [
        0x000000, 0x000080, 0x0000FF, 0x008080, 0x00FFFF, 0x800080, 0xFF00FF, 0x800000,
        0xFF0000, 0x008000, 0x00FF00, 0x808000, 0xFFFF00, 0x808080, 0xC0C0C0, 0xFFFFFF,
    ];

    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);

    cursor.seek(SeekFrom::Start(2))?;
    let colors = match read_u16_be(&mut cursor) {
        Ok(v) => v as usize,
        Err(_) => return Ok(pattern),
    };

    for _ in 0..colors {
        let idx = match read_u16_be(&mut cursor) {
            Ok(v) => v as usize,
            Err(_) => break,
        };
        let color = PCM_THREAD_COLORS[idx % PCM_THREAD_COLORS.len()];
        pattern.add_thread(EmbThread::new(color));
    }

    let _stitch_count = read_u16_be(&mut cursor).ok();

    loop {
        let x = match read_u24_be(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let _c0 = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let y = match read_u24_be(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let _c1 = match read_u8(&mut cursor) {
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
    fn test_read_cake3_pcm() {
        let path = "tests/testdata/Not Mandatory/Bean.pcm";
        let data = fs::read(path).expect("Failed to read test PCM file");
        let pattern = PcmReader.read(&data).expect("Failed to parse PCM file");

        assert!(pattern.stitches.len() > 10, "expected parsed PCM stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero PCM coordinates"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
