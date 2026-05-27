use std::io::{Seek, SeekFrom};
use crate::models::EmbThread;
// Reuse DST header parsing for DSB
fn process_header_info(pattern: &mut EmbPattern, prefix: &str, value: &str) {
    match prefix {
        "TC" => {
            let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                let hex = parts[0].trim_start_matches('#');
                let color = u32::from_str_radix(hex, 16).unwrap_or(0x000000);
                let thread = EmbThread {
                    color,
                    description: if parts[1].is_empty() { None } else { Some(parts[1].to_string()) },
                    catalog_number: if parts[2].is_empty() { None } else { Some(parts[2].to_string()) },
                    details: None,
                    brand: None,
                    chart: None,
                    weight: None,
                };
                pattern.add_thread(thread);
            }
        }
        "CO" => {
            // Add dummy threads for each color (as DST/DSB header does)
            if let Ok(num) = value.parse::<usize>() {
                for _ in 0..num {
                    pattern.add_thread(EmbThread::new(0xCCCCCC));
                }
            }
        }
        _ => {}
    }
}

fn dsb_read_header(cursor: &mut Cursor<&[u8]>, pattern: &mut EmbPattern) {
    let pos = cursor.position() as usize;
    let data = cursor.get_ref();
    let end = data.len().min(pos + 512);
    if pos >= end { return; }
    let header_bytes = &data[pos..end];
    let _ = cursor.seek(SeekFrom::Start(end as u64));
    let mut start = 0;
    for (i, &byte) in header_bytes.iter().enumerate() {
        if byte == 13 || byte == 10 {
            let segment = &header_bytes[start..i];
            start = i;
            if let Ok(line) = std::str::from_utf8(segment) {
                let line = line.trim();
                if line.len() > 3 {
                    let prefix = &line[0..2].trim();
                    let value = line[3..].trim();
                    process_header_info(pattern, prefix, value);
                }
            }
        }
    }
}
use std::io::Read;
use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::Cursor;

pub struct DsbReader;

impl EmbroideryReader for DsbReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_dsb(data)
    }
}

fn read_dsb(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    dsb_read_header(&mut cursor, &mut pattern);
    let mut stitch_count = 0;
    loop {
        let mut byte = [0u8; 3];
        if cursor.read_exact(&mut byte).is_err() {
            break;
        }
        let ctrl = byte[0];
        let mut y = -(byte[1] as i8 as f32);
        let mut x = byte[2] as i8 as f32;
        if ctrl & 0x40 != 0 { y = -y; }
        if ctrl & 0x20 != 0 { x = -x; }
        match ctrl & 0b11111 {
            0 => { pattern.add_stitch_relative(StitchType::Stitch, x, y); stitch_count += 1; },
            1 => { pattern.add_stitch_relative(StitchType::Jump, x, y); stitch_count += 1; },
            _ => match ctrl {
                0xF8 => break,
                0xE7 => { pattern.add_stitch_relative(StitchType::Trim, x, y); stitch_count += 1; },
                0xE8 => { pattern.add_stitch_relative(StitchType::Stop, x, y); stitch_count += 1; },
                c if (0xE9..0xF8).contains(&c) => { pattern.add_stitch_relative(StitchType::ColorChange, x, y); stitch_count += 1; },
                _ => break,
            },
        }
    }
    pattern.add_stitch_relative(StitchType::End, 0.0, 0.0);
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_dsb() {
        let path = "tests/testdata/Cake 3.dsb";
        let data = fs::read(path).expect("Failed to read test DSB file");
        let pattern = DsbReader.read(&data).expect("Failed to parse DSB file");
        let num_colour_changes = pattern.count_color_changes();
        assert_eq!(pattern.stitches.len(), 15138, "Unexpected stitch count");
        assert_eq!(pattern.threadlist.len(), 18, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 18, "Unexpected number of colour changes");
    }
}
