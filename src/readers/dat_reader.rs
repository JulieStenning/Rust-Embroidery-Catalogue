use std::io::Read;
use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::Cursor;

pub struct DatReader;

impl EmbroideryReader for DatReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_dat(data)
    }
}

fn read_dat(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    // Try Barudan first
    let mut barudan_ok = true;
    let mut threads: Vec<u8> = Vec::new();
    loop {
        let mut byte = [0u8; 3];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }
        let ctrl = byte[0];
        let mut dy = -(byte[1] as i8 as f32);
        let mut dx = byte[2] as i8 as f32;
        if ctrl & 0x80 == 0 {
            // Not Barudan, try Sunstar
            barudan_ok = false;
            break;
        }
        if ctrl & 0x20 != 0 { dx = -dx; }
        if ctrl & 0x40 != 0 { dy = -dy; }
        let command = ctrl & 0b11111;
        match command {
            0x00 => { pattern.add_stitch_relative(StitchType::Stitch, dx, dy); },
            0x01 => { pattern.add_stitch_relative(StitchType::Jump, dx, dy); },
            0x02 => {
                pattern.add_stitch_relative(StitchType::Fast, 0.0, 0.0);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(StitchType::Stitch, dx, dy);
                }
            },
            0x03 => {
                pattern.add_stitch_relative(StitchType::Fast, 0.0, 0.0);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx, dy);
                }
            },
            0x04 => {
                pattern.add_stitch_relative(StitchType::Slow, 0.0, 0.0);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(StitchType::Stitch, dx, dy);
                }
            },
            0x05 => {
                pattern.add_stitch_relative(StitchType::Slow, 0.0, 0.0);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx, dy);
                }
            },
            0x06 => {
                pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx, dy);
                }
            },
            0x07 => {
                pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx, dy);
                }
            },
            0x08 => {
                pattern.add_stitch_relative(StitchType::Stop, 0.0, 0.0);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx, dy);
                }
            },
            0x09..=0x17 => {
                // Needle change (color change)
                let needle = command - 0x08;
                pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
                threads.push(needle);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(StitchType::Jump, dx, dy);
                }
            },
            0x18 => { break; },
            _ => { break; },
        }
    }
    if !barudan_ok {
        // Sunstar: seek to 0x100 and decode
        let mut cursor = Cursor::new(data);
        cursor.set_position(0x100);
        loop {
            let mut byte = [0u8; 3];
            if cursor.read_exact(&mut byte).is_err() {
                break;
            }
            let mut x = (byte[0] & 0x7F) as i8 as f32;
            let mut y = (byte[1] & 0x7F) as i8 as f32;
            if byte[0] & 0x80 != 0 { x = -x; }
            if byte[1] & 0x80 != 0 { y = -y; }
            y = -y;
            let ctrl = byte[2];
            match ctrl {
                0x07 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
                0x04 => pattern.add_stitch_relative(StitchType::Jump, x, y),
                0x80 => {
                    pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
                    if x != 0.0 || y != 0.0 {
                        pattern.add_stitch_relative(StitchType::Jump, x, y);
                    }
                },
                0x87 => {
                    pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
                    if x != 0.0 || y != 0.0 {
                        pattern.add_stitch_relative(StitchType::Jump, x, y);
                    }
                },
                0x84 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
                0x00 => break,
                _ => break,
            }
        }
    }
    // Add dummy threads for color changes if none present
    if pattern.threadlist.is_empty() && !threads.is_empty() {
        for needle in threads {
            // Assign a dummy color for each needle (could be improved)
            let color = match needle {
                0 => 0x000000, 1 => 0xFF0000, 2 => 0x00FF00, 3 => 0x0000FF,
                4 => 0xFFFF00, 5 => 0xFF00FF, 6 => 0x00FFFF, 7 => 0x888888,
                8 => 0xFFFFFF, 9 => 0x800000, 10 => 0x008000, 11 => 0x000080,
                _ => 0xCCCCCC,
            };
            pattern.add_thread(crate::models::EmbThread::new(color));
        }
    }
    // Add final End command to match Python logic
    pattern.add_stitch_relative(StitchType::End, 0.0, 0.0);
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_dat() {
        let path = "tests/testdata/Cake 3.dat";
        let data = fs::read(path).expect("Failed to read test DAT file");
        let pattern = DatReader.read(&data).expect("Failed to parse DAT file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        let num_colour_changes = pattern.count_color_changes();
        println!("Number of colour changes: {}", num_colour_changes);
        assert_eq!(pattern.stitches.len(), 15138, "Unexpected stitch count");
        assert_eq!(pattern.threadlist.len(), 11, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 18, "Unexpected number of colour changes");
    }
}
