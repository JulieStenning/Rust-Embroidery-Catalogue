use crate::models::EmbPattern;
use crate::readers::embroidery_reader::EmbroideryReader;
use crate::readers::dst_reader::read_dst;

pub struct ExyReader;

impl EmbroideryReader for ExyReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_exy(data)
    }
}

fn read_exy(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    // EXY stores 0x100 bytes of non-stitch header, then DST-style stitch data.
    let stitch_data = data.get(0x100..).unwrap_or(&[]);
    let mut padded = vec![0u8; 512];
    padded.extend_from_slice(stitch_data);
    Ok(read_dst(&padded)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_exy() {
        let path = "tests/testdata/Not Mandatory/Bean.exy";
        let data = fs::read(path).expect("Failed to read test EXY file");
        let pattern = ExyReader.read(&data).expect("Failed to parse EXY file");

        assert!(pattern.stitches.len() > 10, "expected parsed EXY stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero EXY coordinates"
        );
    }
}
