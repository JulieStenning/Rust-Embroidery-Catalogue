use std::io::Read;
use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::Cursor;

pub struct MitReader;

impl EmbroideryReader for MitReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_mit(data)
    }
}

fn read_mit(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    let mit_size_conversion_ratio = 2.0 / 1.0;
    let mut previous_ctrl = -1i32;
    loop {
        let mut byte = [0u8; 2];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }
        let mut x = (byte[0] & 0x1F) as f32;
        let mut y = -((byte[1] & 0x1F) as f32);
        x *= mit_size_conversion_ratio;
        y *= mit_size_conversion_ratio;
        if byte[0] & 0b10000000 != 0 { x = -x; }
        if byte[1] & 0b10000000 != 0 { y = -y; }
        let ctrl = (((byte[0] & 0x60) >> 3) | ((byte[1] & 0x60) >> 5)) as i32;
        match ctrl {
            0b0111 => { pattern.add_stitch_relative(StitchType::Stitch, x, y); previous_ctrl = ctrl; continue; },
            0b1100 => pattern.add_stitch_relative(StitchType::Jump, x, y),
            0b0100 | 0b0101 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
            0b1000 => if previous_ctrl == 0b111 { pattern.add_stitch_relative(StitchType::ColorChange, x, y); },
            0b0000 => break,
            _ => pattern.add_stitch_relative(StitchType::Stitch, x, y),
        }
        previous_ctrl = ctrl;
    }
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_mit() {
        let path = "tests/testdata/Cake 3.mit";
        let data = fs::read(path).expect("Failed to read test MIT file");
        let pattern = MitReader.read(&data).expect("Failed to parse MIT file");
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
