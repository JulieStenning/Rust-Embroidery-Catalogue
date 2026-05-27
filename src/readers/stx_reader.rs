use crate::models::EmbPattern;
use crate::readers::embroidery_reader::EmbroideryReader;
use crate::readers::exp_reader::read_exp_stitches;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct StxReader;

impl EmbroideryReader for StxReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_stx(data)
    }
}

/// Read a 32-bit little-endian integer from the cursor.
fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// STX format starts with "STX", then has a header with offsets,
/// then uses EXP-format stitch data.
fn read_stx(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut cursor = Cursor::new(data);
    let mut pattern = EmbPattern::new();
    
    // Skip "STX" header (3 bytes) + 9 bytes
    cursor.seek(SeekFrom::Start(0x0C))?;
    
    // Read offsets (Python: color_start, dunno_block_start, stitch_start)
    let _color_start = read_u32_le(&mut cursor)?;
    let _dunno_block_start = read_u32_le(&mut cursor)?;
    let stitch_start = read_u32_le(&mut cursor)?;
    
    // Seek to stitch data and read using EXP stitch parser
    cursor.seek(SeekFrom::Start(stitch_start as u64))?;
    read_exp_stitches(&mut cursor, &mut pattern)?;
    
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_stx() {
        let path = "tests/testdata/Cake 3.stx";
        let data = fs::read(path).expect("Failed to read test STX file");
        let pattern = StxReader.read(&data).expect("Failed to parse STX file");
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
