use crate::models::EmbPattern;
use crate::readers::embroidery_reader::EmbroideryReader;
use crate::readers::pes_reader::read_pes;

pub struct PecReader;

impl EmbroideryReader for PecReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_pec(data)
    }
}

/// PEC format is read by the PES reader (PES files can contain PEC data).
/// PEC files start with "#PEC0001", which the PES reader handles.
fn read_pec(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    Ok(read_pes(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_pec() {
        let path = "tests/testdata/Cake 3.pec";
        let data = fs::read(path).expect("Failed to read test PEC file");
        let pattern = PecReader.read(&data).expect("Failed to parse PEC file");
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
