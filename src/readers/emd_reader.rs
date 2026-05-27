use std::io::Read;
use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::Cursor;

pub struct EmdReader;

impl EmbroideryReader for EmdReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_emd(data)
    }
}

fn signed8(b: u8) -> i8 {
    b as i8
}

fn read_emd(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    cursor.set_position(0x30);
    let mut threads: Vec<u8> = Vec::new();
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
        let control = b[1];
        match control {
            0x80 => {
                let mut b2 = [0u8; 2];
                if cursor.read_exact(&mut b2).is_err() {
                    break;
                }
                let x = signed8(b2[0]) as f32;
                let y = -(signed8(b2[1]) as f32);
                pattern.add_stitch_relative(StitchType::Jump, x, y);
            }
            0x2A => {
                pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
                threads.push(1); // Just a placeholder, not used for color value
            },
            0x7D => continue,
            0xAD | 0x90 => pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0),
            0xFD => break,
            _ => break,
        }
    }
    if pattern.threadlist.is_empty() && !threads.is_empty() {
        for _ in threads {
            // Assign a dummy color for each color change
            pattern.add_thread(crate::models::EmbThread::new(0x000000));
        }
    }
    pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_emd() {
        let path = "tests/testdata/Cake 3.emd";
        let data = fs::read(path).expect("Failed to read test EMD file");
        let pattern = EmdReader.read(&data).expect("Failed to parse EMD file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        let num_colour_changes = pattern.count_color_changes();
        println!("Number of colour changes: {}", num_colour_changes);
        for (i, stitch) in pattern.stitches.iter().take(5).enumerate() {
        }
        assert_eq!(pattern.stitches.len(), 15087, "Unexpected stitch count");
        assert_eq!(pattern.threadlist.len(), 11, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 19, "Unexpected number of colour changes");
    }
}
