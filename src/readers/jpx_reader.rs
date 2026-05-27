use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct JpxReader;

impl EmbroideryReader for JpxReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_jpx(data)
    }
}

fn read_jpx(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
        let mut b = [0u8; 4];
        cursor.read_exact(&mut b)?;
        Ok(u32::from_le_bytes(b))
    }

    fn signed8(v: u8) -> i8 {
        v as i8
    }

    fn read_jpx_stitches(
        cursor: &mut Cursor<&[u8]>,
        pattern: &mut EmbPattern,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut b = [0u8; 2];
            if cursor.read_exact(&mut b).is_err() {
                break;
            }

            if b[0] != 0x80 {
                let x = signed8(b[0]) as f32;
                let y = -(signed8(b[1]) as f32);
                pattern.add_stitch_relative(StitchType::Stitch, x, y);
                continue;
            }

            let ctrl = b[1];
            if cursor.read_exact(&mut b).is_err() {
                break;
            }
            let x = signed8(b[0]) as f32;
            let y = -(signed8(b[1]) as f32);

            if ctrl == 0x02 {
                pattern.add_stitch_relative(StitchType::Jump, x, y);
                continue;
            }
            if ctrl == 0x01 {
                pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
                if x != 0.0 && y != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, x, y);
                }
                continue;
            }
            if ctrl == 0x10 {
                break;
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

    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);

    let stitch_start_position = read_u32_le(&mut cursor)? as u64;
    cursor.seek(SeekFrom::Current(0x1C))?;
    let colors = read_u32_le(&mut cursor)? as usize;
    cursor.seek(SeekFrom::Current(0x18))?;

    for _ in 0..colors {
        let color_index = match read_u32_le(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let color = (color_index.wrapping_mul(0x45D9F3B) & 0x00FF_FFFF) | 0x20_20_20;
        pattern.add_thread(EmbThread::new(color));
    }

    cursor.seek(SeekFrom::Start(stitch_start_position))?;
    read_jpx_stitches(&mut cursor, &mut pattern)?;

    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_jpx() {
        let path = "tests/testdata/Not Mandatory/Bean.jpx";
        let data = fs::read(path).expect("Failed to read test JPX file");
        let pattern = JpxReader.read(&data).expect("Failed to parse JPX file");

        assert!(pattern.stitches.len() > 10, "expected parsed JPX stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero JPX coordinates"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
