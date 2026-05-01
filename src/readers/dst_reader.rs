use std::io::{Cursor, Seek, SeekFrom};

use crate::models::{EmbPattern, EmbThread, StitchType};

// ---------------------------------------------------------------------------
// DST header parsing — reads the 512-byte text header
// ---------------------------------------------------------------------------

/// Parse a single line from the DST header and apply it to the pattern.
fn process_header_info(pattern: &mut EmbPattern, prefix: &str, value: &str) {
    match prefix {
        "LA" => {
            pattern.extras.insert("name".into(), value.to_string());
        }
        "AU" => {
            pattern.extras.insert("author".into(), value.to_string());
        }
        "CP" => {
            pattern.extras.insert("copyright".into(), value.to_string());
        }
        "TC" => {
            // Thread colour entry: "hex,description,catalog"
            let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                let hex = parts[0].trim_start_matches('#');
                let color = u32::from_str_radix(hex, 16).unwrap_or(0x000000);
                let thread = EmbThread {
                    color,
                    description: if parts[1].is_empty() {
                        None
                    } else {
                        Some(parts[1].to_string())
                    },
                    catalog_number: if parts[2].is_empty() {
                        None
                    } else {
                        Some(parts[2].to_string())
                    },
                    details: None,
                    brand: None,
                    chart: None,
                    weight: None,
                };
                pattern.add_thread(thread);
            }
        }
        _ => {
            pattern
                .extras
                .insert(prefix.to_string(), value.to_string());
        }
    }
}

/// Parse the 512-byte DST text header for metadata.
///
/// The Python reader byte-scans for `\r` (13) and `\n` (10) delimiters,
/// splitting the header into segments and decoding each as UTF-8 with
/// graceful fallback on decode errors.
fn dst_read_header(cursor: &mut Cursor<&[u8]>, pattern: &mut EmbPattern) -> Result<(), binrw::Error> {
    let pos = cursor.position() as usize;
    let data = cursor.get_ref();
    let end = data.len().min(pos + 512);
    if pos >= end {
        return Ok(());
    }
    let header_bytes = &data[pos..end];
    cursor.seek(SeekFrom::Start(end as u64))?;

    // Byte-scan for \r (13) and \n (10) delimiters, matching Python's
    // per-byte iteration over the 512-byte header.
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
            // Non-UTF8 segments are silently skipped (matching Python's except).
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// DST stitch decoding — bit-level dx/dy extraction from 3-byte records
// ---------------------------------------------------------------------------

/// Extract a single bit at position `pos` from byte `b` (0 = LSB, 7 = MSB).
#[inline]
fn getbit(b: u8, pos: u8) -> i32 {
    ((b >> pos) & 1) as i32
}

fn decode_dx(b0: u8, b1: u8, b2: u8) -> f32 {
    let mut x: i32 = 0;
    x += getbit(b2, 2) * 81;
    x += getbit(b2, 3) * -81;
    x += getbit(b1, 2) * 27;
    x += getbit(b1, 3) * -27;
    x += getbit(b0, 2) * 9;
    x += getbit(b0, 3) * -9;
    x += getbit(b1, 0) * 3;
    x += getbit(b1, 1) * -3;
    x += getbit(b0, 0) * 1;
    x += getbit(b0, 1) * -1;
    x as f32
}

fn decode_dy(b0: u8, b1: u8, b2: u8) -> f32 {
    let mut y: i32 = 0;
    y += getbit(b2, 5) * 81;
    y += getbit(b2, 4) * -81;
    y += getbit(b1, 5) * 27;
    y += getbit(b1, 4) * -27;
    y += getbit(b0, 5) * 9;
    y += getbit(b0, 4) * -9;
    y += getbit(b1, 7) * 3;
    y += getbit(b1, 6) * -3;
    y += getbit(b0, 7) * 1;
    y += getbit(b0, 6) * -1;
    -(y as f32) // DST inverts Y
}

// ---------------------------------------------------------------------------
// DST stitch reader
// ---------------------------------------------------------------------------

/// Read exactly `n` bytes from the cursor into a Vec, or return an error.
fn read_exact(cursor: &mut Cursor<&[u8]>, n: usize) -> Result<Vec<u8>, binrw::Error> {
    let pos = cursor.position();
    let data = cursor.get_ref();
    let end = pos as usize + n;
    if end > data.len() {
        return Err(binrw::Error::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "unexpected end of DST stitch data",
        )));
    }
    let bytes = data[pos as usize..end].to_vec();
    cursor.set_position(end as u64);
    Ok(bytes)
}

fn dst_read_stitches(cursor: &mut Cursor<&[u8]>, pattern: &mut EmbPattern) -> Result<(), binrw::Error> {
    let mut sequin_mode = false;

    loop {
        if cursor.position() as usize >= cursor.get_ref().len() {
            break;
        }

        let bytes = match read_exact(cursor, 3) {
            Ok(b) => b,
            Err(_) => break,
        };

        let dx = decode_dx(bytes[0], bytes[1], bytes[2]);
        let dy = decode_dy(bytes[0], bytes[1], bytes[2]);

        // End of design: bits 0-1, 4-7 all set in byte[2]
        if bytes[2] & 0b11110011 == 0b11110011 {
            break;
        }
        // Color change: bits 0-1, 6-7 set
        else if bytes[2] & 0b11000011 == 0b11000011 {
            pattern.add_stitch_relative(StitchType::ColorChange, dx, dy);
        }
        // Sequin mode toggle: bits 0-1, 6 set
        else if bytes[2] & 0b01000011 == 0b01000011 {
            pattern.add_stitch_relative(StitchType::SequinMode, dx, dy);
            sequin_mode = !sequin_mode;
        }
        // Jump or sequin eject: bits 0-1, 7 set
        else if bytes[2] & 0b10000011 == 0b10000011 {
            if sequin_mode {
                pattern.add_stitch_relative(StitchType::SequinEject, dx, dy);
            } else {
                pattern.add_stitch_relative(StitchType::Jump, dx, dy);
            }
        }
        // Regular stitch
        else {
            pattern.add_stitch_relative(StitchType::Stitch, dx, dy);
        }
    }

    pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);

    // TODO: The Python reader applies `interpolate_trims` here based on
    //       trim_distance / trim_at / clipping settings. This can be added
    //       later as a method on EmbPattern.

    Ok(())
}

// ---------------------------------------------------------------------------
// Public entry-point
// ---------------------------------------------------------------------------

/// Parse a DST-format Tajima embroidery file from a byte buffer.
///
/// Returns an [`EmbPattern`] containing the stitches, threads, and metadata
/// read from the file.
///
/// # Errors
///
/// Returns a [`binrw::Error`] if the data is truncated or malformed.
pub fn read_dst(data: &[u8]) -> Result<EmbPattern, binrw::Error> {
    let mut cursor = Cursor::new(data);
    let mut pattern = EmbPattern::new();

    // --- 1. Parse the 512-byte text header ------------------------------------
    dst_read_header(&mut cursor, &mut pattern)?;

    // --- 2. Parse the stitch data ---------------------------------------------
    dst_read_stitches(&mut cursor, &mut pattern)?;

    Ok(pattern)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_dx() {
        // b0=0x01 → bit0=1 → +1
        assert_eq!(decode_dx(0x01, 0x00, 0x00), 1.0);
        // b0=0x02 → bit1=1 → -1
        assert_eq!(decode_dx(0x02, 0x00, 0x00), -1.0);
    }

    #[test]
    fn test_decode_dy() {
        // dy: b0 bit6=1 → -1 in sum → negated → +1
        assert_eq!(decode_dy(0x40, 0x00, 0x00), 1.0);
        // dy: b0 bit7=1 → +1 in sum → negated → -1
        assert_eq!(decode_dy(0x80, 0x00, 0x00), -1.0);
    }

    #[test]
    fn test_read_dst_header_metadata() {
        // The Python DST reader byte-scans for \r and \n as delimiters.
        // Build a minimal 512-byte header with \r-separated lines.
        let mut data: Vec<u8> = vec![0u8; 512];
        let header_str = "LA:Test Design\rAU:Test Author\r";
        let hdr_bytes = header_str.as_bytes();
        for (i, &b) in hdr_bytes.iter().enumerate() {
            data[i] = b;
        }
        // END marker: byte[2]=0xF3 = 0b11110011
        data.push(0x00);
        data.push(0x00);
        data.push(0xF3);

        let pattern = read_dst(&data).expect("should parse minimal DST");

        assert_eq!(
            pattern.extras.get("name").map(|s| s.as_str()),
            Some("Test Design")
        );
        assert_eq!(
            pattern.extras.get("author").map(|s| s.as_str()),
            Some("Test Author")
        );
    }

    #[test]
    fn test_read_dst_stitch_and_end() {
        let mut data = vec![0u8; 512];

        // Stitch 1: dx=+1, dy=+1 → b0=0x81
        data.push(0x81);
        data.push(0x00);
        data.push(0x00);

        // Stitch 2: dx=+1, dy=+1 → b0=0x81
        data.push(0x81);
        data.push(0x00);
        data.push(0x00);

        // END: byte[2]=0xF3
        data.push(0x00);
        data.push(0x00);
        data.push(0xF3);

        let pattern = read_dst(&data).expect("should parse valid DST");

        assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 2);
        assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
    }
}