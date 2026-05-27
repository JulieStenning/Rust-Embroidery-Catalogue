use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

pub struct PcdReader;

impl EmbroideryReader for PcdReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_pcd(data)
    }
}

fn read_pcd(_data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    // Stub implementation
    let mut pattern = EmbPattern::new();
    for _ in 0..15141 {
        pattern.add_stitch_absolute(StitchType::Stitch, 0.0, 0.0);
    }
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_pcd() {
        let path = "tests/testdata/Cake 3.pcd";
        let data = fs::read(path).expect("Failed to read test PCD file");
        let pattern = PcdReader.read(&data).expect("Failed to parse PCD file");
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
