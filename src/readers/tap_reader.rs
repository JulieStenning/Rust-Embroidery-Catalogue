use crate::models::EmbPattern;
use crate::readers::embroidery_reader::EmbroideryReader;
use crate::readers::dst_reader::read_dst;

pub struct TapReader;

impl EmbroideryReader for TapReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_tap(data)
    }
}

/// TAP format is identical to DST but without the 512-byte header.
/// It goes straight to stitch data.
fn read_tap(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    // TAP format has no header - just call the DST stitch reader directly
    // by creating a fake 512-byte header and appending the data
    let mut padded_data = vec![0u8; 512];
    padded_data.extend_from_slice(data);
    Ok(read_dst(&padded_data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_tap() {
        let path = "tests/testdata/Cake 3.tap";
        let data = fs::read(path).expect("Failed to read test TAP file");
        let pattern = TapReader.read(&data).expect("Failed to parse TAP file");
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
