use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct FxyReader;

impl EmbroideryReader for FxyReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_fxy(data)
    }
}

fn read_fxy(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    cursor.seek(SeekFrom::Start(0x100))?;

    let mut threads: Vec<u8> = Vec::new();
    loop {
        let mut byte = [0u8; 3];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }
        let mut y = -(byte[0] as i8 as f32);
        let mut x = byte[1] as i8 as f32;
        let ctrl = byte[2];
        if ctrl & 0x40 != 0 {
            x = -x;
        }
        if ctrl & 0x20 != 0 {
            y = -y;
        }

        match ctrl & 0b11111 {
            0 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
            1 => pattern.add_stitch_relative(StitchType::Jump, x, y),
            _ => match ctrl {
                0x82 => pattern.add_stitch_relative(StitchType::Stop, x, y),
                0x9B => pattern.add_stitch_relative(StitchType::Trim, x, y),
                c if (0x83..=0x9A).contains(&c) => {
                    let needle = (c - 0x83) >> 1;
                    pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
                    threads.push(needle);
                }
                _ => break,
            },
        }
    }

    if pattern.threadlist.is_empty() && !threads.is_empty() {
        for needle in threads {
            let color = match needle {
                0 => 0x000000,
                1 => 0xFF0000,
                2 => 0x00FF00,
                3 => 0x0000FF,
                4 => 0xFFFF00,
                5 => 0xFF00FF,
                6 => 0x00FFFF,
                7 => 0x888888,
                8 => 0xFFFFFF,
                9 => 0x800000,
                10 => 0x008000,
                11 => 0x000080,
                _ => 0xCCCCCC,
            };
            pattern.add_thread(crate::models::EmbThread::new(color));
        }
    }

    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_fxy() {
        let path = "tests/testdata/Not Mandatory/Bean.fxy";
        let data = fs::read(path).expect("Failed to read test FXY file");
        let pattern = FxyReader.read(&data).expect("Failed to parse FXY file");

        assert!(pattern.stitches.len() > 10, "expected parsed FXY stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero FXY coordinates"
        );
    }
}
