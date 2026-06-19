use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct NewReader;

impl EmbroideryReader for NewReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_new(data)
    }
}

fn read_new(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    cursor.seek(SeekFrom::Current(2))?;
    loop {
        let mut byte = [0u8; 3];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }
        let mut x = byte[0] as i8 as f32;
        let mut y = -(byte[1] as i8 as f32);
        let mut ctrl = byte[2];
        if ctrl & 0b01000000 != 0 { x = -x; }
        if ctrl & 0b00100000 != 0 { y = -y; }
        ctrl &= !0b11100000;
        match ctrl {
            0 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
            0b00010001 => break,
            c if c & 0b00000010 != 0 => pattern.add_stitch_relative(StitchType::ColorChange, x, y),
            c if c & 0b00000001 != 0 => pattern.add_stitch_relative(StitchType::Jump, x, y),
            _ => break,
        }
    }
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_new() {
        let path = "tests/testdata/Cake 3.new";
        let data = fs::read(path).expect("Failed to read test NEW file");
        let pattern = NewReader.read(&data).expect("Failed to parse NEW file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        let num_colour_changes = pattern.count_color_changes();
        println!("Number of colour changes: {}", num_colour_changes);
        assert_eq!(pattern.stitches.len(), 15141, "Unexpected stitch count");
        assert_eq!(pattern.threadlist.len(), 19, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 18, "Unexpected number of colour changes");
    }
}
