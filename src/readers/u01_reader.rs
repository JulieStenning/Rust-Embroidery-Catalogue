use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct U01Reader;

impl EmbroideryReader for U01Reader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_u01(data)
    }
}

fn current_position(pattern: &EmbPattern) -> (f32, f32) {
    pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0))
}

fn add_command_here(pattern: &mut EmbPattern, stitch_type: StitchType) {
    let (x, y) = current_position(pattern);
    pattern.add_stitch_absolute(stitch_type, x, y);
}

fn thread_color_from_index(idx: usize) -> u32 {
    const PALETTE: [u32; 16] = [
        0x000000, 0x000080, 0x0000FF, 0x008080, 0x00FFFF, 0x800080, 0xFF00FF, 0x800000,
        0xFF0000, 0x008000, 0x00FF00, 0x808000, 0xFFFF00, 0x808080, 0xC0C0C0, 0xFFFFFF,
    ];
    PALETTE[idx % PALETTE.len()]
}

fn read_u01(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);

    // U01 contains two 0x80-byte header blocks before stitch commands.
    cursor.seek(SeekFrom::Start(0x100))?;

    // U01 does not carry explicit thread metadata in this path.
    pattern.add_thread(EmbThread::new(thread_color_from_index(0)));

    loop {
        let mut byte = [0u8; 3];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }

        let ctrl = byte[0];
        let mut dy = -(byte[1] as i32);
        let mut dx = byte[2] as i32;

        if (ctrl & 0x20) != 0 {
            dx = -dx;
        }
        if (ctrl & 0x40) != 0 {
            dy = -dy;
        }

        let command = ctrl & 0x1F;

        match command {
            0x00 => {
                pattern.add_stitch_relative(StitchType::Stitch, dx as f32, dy as f32);
            }
            0x01 => {
                pattern.add_stitch_relative(StitchType::Jump, dx as f32, dy as f32);
            }
            0x02 => {
                add_command_here(&mut pattern, StitchType::Fast);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(StitchType::Stitch, dx as f32, dy as f32);
                }
            }
            0x03 => {
                add_command_here(&mut pattern, StitchType::Fast);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx as f32, dy as f32);
                }
            }
            0x04 => {
                add_command_here(&mut pattern, StitchType::Slow);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(StitchType::Stitch, dx as f32, dy as f32);
                }
            }
            0x05 => {
                add_command_here(&mut pattern, StitchType::Slow);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx as f32, dy as f32);
                }
            }
            0x06 | 0x07 => {
                add_command_here(&mut pattern, StitchType::Trim);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx as f32, dy as f32);
                }
            }
            0x08 => {
                add_command_here(&mut pattern, StitchType::Stop);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx as f32, dy as f32);
                }
            }
            0x09..=0x17 => {
                let needle = (command - 0x08) as usize;
                pattern.add_thread(EmbThread::new(thread_color_from_index(needle)));
                add_command_here(&mut pattern, StitchType::ColorChange);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx as f32, dy as f32);
                }
            }
            0x18 => {
                break;
            }
            _ if ctrl == 0x2B => {
                break;
            }
            _ => {
                break;
            }
        }
    }

    let (end_x, end_y) = current_position(&pattern);
    pattern.add_stitch_absolute(StitchType::End, end_x, end_y);
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_bean_u01() {
        let path = "tests/testdata/Not Mandatory/Bean.u01";
        let data = fs::read(path).expect("Failed to read test U01 file");
        let pattern = U01Reader.read(&data).expect("Failed to parse U01 file");

        assert!(pattern.stitches.len() > 10, "expected parsed U01 stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero U01 coordinates"
        );
        assert!(
            !pattern.threadlist.is_empty(),
            "expected at least one U01 thread"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
