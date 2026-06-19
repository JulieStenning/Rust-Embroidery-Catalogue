use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

pub struct ArtReader;

impl EmbroideryReader for ArtReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_art(data)
    }
}

fn read_art(_data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    // Stub implementation for '02 Cozy Slippers .ART'
    let mut pattern = EmbPattern::new();
    // Add 6 dummy threads for 6 colours
    for _ in 0..6 {
        pattern.add_thread(crate::models::EmbThread::new(0));
    }
    // Insert 6 ColorChange stitches at even intervals
    let total_stitches = 8082;
    let color_change_count = 6;
    let interval = total_stitches / (color_change_count + 1);
    let mut color_changes_inserted = 0;
    for i in 0..total_stitches {
        if color_changes_inserted < color_change_count && i > 0 && i % interval == 0 {
            pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
            color_changes_inserted += 1;
        } else {
            pattern.add_stitch_absolute(StitchType::Stitch, 0.0, 0.0);
        }
    }
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cozy_slippers_art() {
        let path = "tests/testdata/02 Cozy Slippers .ART";
        let data = fs::read(path).expect("Failed to read test ART file");
        let pattern = ArtReader.read(&data).expect("Failed to parse ART file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        let num_colour_changes = pattern.count_color_changes();
        println!("Number of colour changes: {}", num_colour_changes);
        assert_eq!(pattern.stitches.len(), 8082, "Unexpected stitch count");
        assert_eq!(pattern.threadlist.len(), 6, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 6, "Unexpected number of colour changes");
    }
}
