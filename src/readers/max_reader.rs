use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Seek, SeekFrom};

pub struct MaxReader;

impl EmbroideryReader for MaxReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_max(data)
    }
}

const MAX_SIZE_CONVERSION_RATIO: f32 = 1.235;

/// Convert a 24-bit unsigned value to signed (2's complement).
fn signed24(val: u32) -> i32 {
    let val = val & 0xFFFFFF;
    if val > 0x7FFFFF {
        -((0x1000000 - val) as i32)
    } else {
        val as i32
    }
}

/// Read a 24-bit little-endian integer.
fn read_u24_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
    use std::io::Read;
    let mut buf = [0u8; 3];
    cursor.read_exact(&mut buf)?;
    Ok((buf[0] as u32) | ((buf[1] as u32) << 8) | ((buf[2] as u32) << 16))
}

/// Read a 32-bit little-endian integer.
fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
    use std::io::Read;
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_max(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    
    // Seek to stitch data offset
    cursor.seek(SeekFrom::Start(0xD5))?;
    
    // Read stitch count
    let stitch_count = read_u32_le(&mut cursor)?;
    
    for _ in 0..stitch_count {
        // Read x (24-bit LE)
        let x_raw = read_u24_le(&mut cursor)?;
        let _ = cursor.read(&mut [0u8; 1])?; // Skip byte
        
        // Read y (24-bit LE)
        let y_raw = read_u24_le(&mut cursor)?;
        let _ = cursor.read(&mut [0u8; 1])?; // Skip byte
        
        // Convert to signed and apply scaling
        let x = signed24(x_raw) as f32 * MAX_SIZE_CONVERSION_RATIO;
        let y = signed24(y_raw) as f32 * MAX_SIZE_CONVERSION_RATIO;
        
        pattern.add_stitch_absolute(StitchType::Stitch, x, y);
    }
    
    pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_max() {
        let path = "tests/testdata/Cake 3.max";
        let data = fs::read(path).expect("Failed to read test MAX file");
        let pattern = MaxReader.read(&data).expect("Failed to parse MAX file");
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
