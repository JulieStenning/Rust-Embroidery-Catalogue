use crate::models::{EmbPattern, StitchType};
use crate::models::EmbThread;
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct TbfReader;

impl EmbroideryReader for TbfReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_tbf(data)
    }
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, std::io::Error> {
    let mut b = [0u8; 1];
    cursor.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_u24_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
    let mut b = [0u8; 3];
    cursor.read_exact(&mut b)?;
    Ok(((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32))
}

fn signed8(v: u8) -> i8 {
    v as i8
}

fn read_tbf(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();

    let mut cursor = Cursor::new(data);
    cursor.seek(SeekFrom::Start(0x83))?;
    let mut name_bytes = [0u8; 0x10];
    cursor.read_exact(&mut name_bytes)?;
    let name = String::from_utf8_lossy(&name_bytes)
        .trim_matches(char::from(0))
        .trim()
        .to_string();
    if !name.is_empty() {
        pattern.extras.insert("name".to_string(), name);
    }

    cursor.seek(SeekFrom::Start(0x10A))?;
    let mut thread_order = vec![0u8; 0x100];
    cursor.read_exact(&mut thread_order)?;

    cursor.seek(SeekFrom::Start(0x20E))?;
    loop {
        let marker = match read_u8(&mut cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        if marker != 0x45 {
            break;
        }
        let color = read_u24_be(&mut cursor)?;
        let _ = read_u8(&mut cursor)?; // expected 0x20
        pattern.add_thread(EmbThread::new(color));
    }

    cursor.seek(SeekFrom::Start(0x600))?;
    let mut needle_idx = 0usize;
    loop {
        let mut byte = [0u8; 3];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }

        let x = signed8(byte[0]) as f32;
        let y = -(signed8(byte[1]) as f32);
        let ctrl = byte[2];

        match ctrl {
            0x80 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
            0x81 => {
                let needle_value = *thread_order.get(needle_idx).unwrap_or(&0);
                needle_idx += 1;
                if needle_value == 0 {
                    pattern.add_stitch_relative(StitchType::Stop, 0.0, 0.0);
                } else {
                    pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
                }
            }
            0x90 => {
                if x == 0.0 && y == 0.0 {
                    pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
                } else {
                    pattern.add_stitch_relative(StitchType::Jump, x, y);
                }
            }
            0x40 => pattern.add_stitch_relative(StitchType::Stop, 0.0, 0.0),
            0x86 => pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0),
            0x8F => break,
            _ => break,
        }
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
    fn test_read_cake3_tbf() {
        let path = "tests/testdata/Not Mandatory/Bean.tbf";
        let data = fs::read(path).expect("Failed to read test TBF file");
        let pattern = TbfReader.read(&data).expect("Failed to parse TBF file");

        assert!(pattern.stitches.len() > 10, "expected parsed TBF stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero TBF coordinates"
        );
        assert!(
            pattern.stitches.last().map(|s| s.stitch_type) == Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
