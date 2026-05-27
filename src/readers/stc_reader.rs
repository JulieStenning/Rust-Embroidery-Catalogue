use std::io::Read;
use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Seek, SeekFrom};

pub struct StcReader;

impl EmbroideryReader for StcReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_stc(data)
    }
}

fn read_stc(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    cursor.seek(SeekFrom::Current(0x28))?;
    loop {
        let mut byte = [0u8; 3];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }
        let x = byte[0] as i8 as f32;
        let y = -(byte[1] as i8 as f32);
        let ctrl = byte[2];
        match ctrl {
            0x01 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
            0x00 => pattern.add_stitch_relative(StitchType::Jump, x, y),
            25 => break,
            c if c > 2 => pattern.add_stitch_relative(StitchType::ColorChange, x, y),
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
    fn test_read_cake3_stc() {
        let path = "tests/testdata/Cake 3.stc";
        let data = fs::read(path).expect("Failed to read test STC file");
        let pattern = StcReader.read(&data).expect("Failed to parse STC file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        let num_colour_changes = pattern.count_color_changes();
        println!("Number of colour changes: {}", num_colour_changes);
        for (i, stitch) in pattern.stitches.iter().take(5).enumerate() {
            println!("Stitch {}: x = {}, y = {}", i, stitch.x, stitch.y);
        }
        assert_eq!(pattern.stitches.len(), 15141, "Unexpected stitch count");
        assert_eq!(pattern.threadlist.len(), 19, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 18, "Unexpected number of colour changes");
    }
}
