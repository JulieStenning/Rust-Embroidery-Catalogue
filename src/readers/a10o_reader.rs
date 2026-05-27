use std::io::Read;
use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::Cursor;

pub struct A10oReader;

impl EmbroideryReader for A10oReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_a10o(data)
    }
}

fn read_a10o(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    loop {
        let mut byte = [0u8; 3];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }
        let ctrl = byte[0];
        let mut y = -(byte[1] as i8 as f32);
        let mut x = byte[2] as i8 as f32;
        if ctrl & 0x20 != 0 { x = -x; }
        if ctrl & 0x40 != 0 { y = -y; }
        match ctrl & 0b11111 {
            0 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
            0x10 => pattern.add_stitch_relative(StitchType::Jump, x, y),
            _ => match ctrl {
                0x8A => continue,
                0x85 => pattern.add_stitch_relative(StitchType::ColorChange, x, y),
                0x82 => pattern.add_stitch_relative(StitchType::Stop, x, y),
                0x81 => pattern.add_stitch_relative(StitchType::Trim, x, y),
                0x87 => break,
                _ => break,
            },
        }
    }
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_10o() {
        let path = "tests/testdata/Cake 3.10o";
        let data = fs::read(path).expect("Failed to read test 10O file");
        let pattern = A10oReader.read(&data).expect("Failed to parse 10O file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        let num_colour_changes = pattern.count_color_changes();
        println!("Number of colour changes: {}", num_colour_changes);
        for (i, stitch) in pattern.stitches.iter().take(5).enumerate() {
        }
        assert_eq!(pattern.stitches.len(), 15141, "Unexpected stitch count");
        assert_eq!(pattern.threadlist.len(), 19, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 18, "Unexpected number of colour changes");
    }
}
