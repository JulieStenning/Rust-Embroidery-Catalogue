use std::io::{Cursor, Seek, SeekFrom};

use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

// ===========================================================================
// PEC thread palette (index 0 = None, 1..64 = real threads)
// ===========================================================================

/// Helper to create a PEC `EmbThread`.
macro_rules! pec_thread {
    ($r:expr, $g:expr, $b:expr, $desc:expr, $cat:expr) => {
        EmbThread {
            color: (($r as u32) << 16) | (($g as u32) << 8) | ($b as u32),
            description: Some($desc.into()),
            catalog_number: Some($cat.into()),
            details: None,
            brand: Some("Brother".into()),
            chart: Some("Brother".into()),
            weight: None,
        }
    };
}

fn get_pec_thread_set() -> Vec<Option<EmbThread>> {
    vec![
        None, // Index 0 – Unknown
        Some(pec_thread!(14, 31, 124, "Prussian Blue", "1")),
        Some(pec_thread!(10, 85, 163, "Blue", "2")),
        Some(pec_thread!(0, 135, 119, "Teal Green", "3")),
        Some(pec_thread!(75, 107, 175, "Cornflower Blue", "4")),
        Some(pec_thread!(237, 23, 31, "Red", "5")),
        Some(pec_thread!(209, 92, 0, "Reddish Brown", "6")),
        Some(pec_thread!(145, 54, 151, "Magenta", "7")),
        Some(pec_thread!(228, 154, 203, "Light Lilac", "8")),
        Some(pec_thread!(145, 95, 172, "Lilac", "9")),
        Some(pec_thread!(158, 214, 125, "Mint Green", "10")),
        Some(pec_thread!(232, 169, 0, "Deep Gold", "11")),
        Some(pec_thread!(254, 186, 53, "Orange", "12")),
        Some(pec_thread!(255, 255, 0, "Yellow", "13")),
        Some(pec_thread!(112, 188, 31, "Lime Green", "14")),
        Some(pec_thread!(186, 152, 0, "Brass", "15")),
        Some(pec_thread!(168, 168, 168, "Silver", "16")),
        Some(pec_thread!(125, 111, 0, "Russet Brown", "17")),
        Some(pec_thread!(255, 255, 179, "Cream Brown", "18")),
        Some(pec_thread!(79, 85, 86, "Pewter", "19")),
        Some(pec_thread!(0, 0, 0, "Black", "20")),
        Some(pec_thread!(11, 61, 145, "Ultramarine", "21")),
        Some(pec_thread!(119, 1, 118, "Royal Purple", "22")),
        Some(pec_thread!(41, 49, 51, "Dark Gray", "23")),
        Some(pec_thread!(42, 19, 1, "Dark Brown", "24")),
        Some(pec_thread!(246, 74, 138, "Deep Rose", "25")),
        Some(pec_thread!(178, 118, 36, "Light Brown", "26")),
        Some(pec_thread!(252, 187, 197, "Salmon Pink", "27")),
        Some(pec_thread!(254, 55, 15, "Vermilion", "28")),
        Some(pec_thread!(240, 240, 240, "White", "29")),
        Some(pec_thread!(106, 28, 138, "Violet", "30")),
        Some(pec_thread!(168, 221, 196, "Seacrest", "31")),
        Some(pec_thread!(37, 132, 187, "Sky Blue", "32")),
        Some(pec_thread!(254, 179, 67, "Pumpkin", "33")),
        Some(pec_thread!(255, 243, 107, "Cream Yellow", "34")),
        Some(pec_thread!(208, 166, 96, "Khaki", "35")),
        Some(pec_thread!(209, 84, 0, "Clay Brown", "36")),
        Some(pec_thread!(102, 186, 73, "Leaf Green", "37")),
        Some(pec_thread!(19, 74, 70, "Peacock Blue", "38")),
        Some(pec_thread!(135, 135, 135, "Gray", "39")),
        Some(pec_thread!(216, 204, 198, "Warm Gray", "40")),
        Some(pec_thread!(67, 86, 7, "Dark Olive", "41")),
        Some(pec_thread!(253, 217, 222, "Flesh Pink", "42")),
        Some(pec_thread!(249, 147, 188, "Pink", "43")),
        Some(pec_thread!(0, 56, 34, "Deep Green", "44")),
        Some(pec_thread!(178, 175, 212, "Lavender", "45")),
        Some(pec_thread!(104, 106, 176, "Wisteria Violet", "46")),
        Some(pec_thread!(239, 227, 185, "Beige", "47")),
        Some(pec_thread!(247, 56, 102, "Carmine", "48")),
        Some(pec_thread!(181, 75, 100, "Amber Red", "49")),
        Some(pec_thread!(19, 43, 26, "Olive Green", "50")),
        Some(pec_thread!(199, 1, 86, "Dark Fuchsia", "51")),
        Some(pec_thread!(254, 158, 50, "Tangerine", "52")),
        Some(pec_thread!(168, 222, 235, "Light Blue", "53")),
        Some(pec_thread!(0, 103, 62, "Emerald Green", "54")),
        Some(pec_thread!(78, 41, 144, "Purple", "55")),
        Some(pec_thread!(47, 126, 32, "Moss Green", "56")),
        Some(pec_thread!(255, 204, 204, "Flesh Pink", "57")),
        Some(pec_thread!(255, 217, 17, "Harvest Gold", "58")),
        Some(pec_thread!(9, 91, 166, "Electric Blue", "59")),
        Some(pec_thread!(240, 249, 112, "Lemon Yellow", "60")),
        Some(pec_thread!(227, 243, 91, "Fresh Green", "61")),
        Some(pec_thread!(255, 153, 0, "Orange", "62")),
        Some(pec_thread!(255, 240, 141, "Cream Yellow", "63")),
        Some(pec_thread!(255, 200, 200, "Applique", "64")),
    ]
}

// ===========================================================================
// Low-level helpers
// ===========================================================================

/// Read exactly `n` bytes, returning a Vec or an error.
fn read_exact(cursor: &mut Cursor<&[u8]>, n: usize) -> Result<Vec<u8>, binrw::Error> {
    let pos = cursor.position();
    let data = cursor.get_ref();
    let end = pos as usize + n;
    if end > data.len() {
        return Err(binrw::Error::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "unexpected end of PES/PEC data",
        )));
    }
    let bytes = data[pos as usize..end].to_vec();
    cursor.set_position(end as u64);
    Ok(bytes)
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, binrw::Error> {
    let b = read_exact(cursor, 1)?;
    Ok(b[0])
}

fn read_u16_le(cursor: &mut Cursor<&[u8]>) -> Result<u16, binrw::Error> {
    let b = read_exact(cursor, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}

fn read_i32_le(cursor: &mut Cursor<&[u8]>) -> Result<i32, binrw::Error> {
    let b = read_exact(cursor, 4)?;
    Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Read a 24-bit big-endian unsigned integer.
fn read_u24_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, binrw::Error> {
    let b = read_exact(cursor, 3)?;
    Ok(((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32))
}

/// Read a 24-bit little-endian unsigned integer.
fn read_u24_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, binrw::Error> {
    let b = read_exact(cursor, 3)?;
    Ok((b[0] as u32) | ((b[1] as u32) << 8) | ((b[2] as u32) << 16))
}

/// Read a PES-style length-prefixed string.
fn read_pes_string(cursor: &mut Cursor<&[u8]>) -> Result<Option<String>, binrw::Error> {
    let length = read_u8(cursor)? as usize;
    if length == 0 {
        return Ok(None);
    }
    let b = read_exact(cursor, length)?;
    Ok(Some(String::from_utf8_lossy(&b).to_string()))
}

// ===========================================================================
// PEC stitch reading (used by the embedded PEC block in PES files)
// ===========================================================================

const FLAG_LONG: u8 = 0x80;
const JUMP_CODE: u8 = 0x10;
const TRIM_CODE: u8 = 0x20;

fn signed12(b: u16) -> i32 {
    let b = b & 0xFFF;
    if b > 0x7FF {
        -0x1000 + b as i32
    } else {
        b as i32
    }
}

fn signed7(b: u8) -> i32 {
    if b > 63 {
        -128 + b as i32
    } else {
        b as i32
    }
}

/// Read the PEC stitch block from the cursor into the pattern.
fn read_pec_stitches(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
) -> Result<(), binrw::Error> {
    loop {
        let val1 = match read_u8(cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let val2 = match read_u8(cursor) {
            Ok(v) => v,
            Err(_) => break,
        };

        // End marker: 0xFF 0x00
        if val1 == 0xFF && val2 == 0x00 {
            break;
        }

        // Color change marker: 0xFE 0xB0
        if val1 == 0xFE && val2 == 0xB0 {
            cursor.seek(SeekFrom::Current(1))?; // skip 1 byte
            pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
            continue;
        }

        let mut jump = false;
        let mut trim = false;
        let x: i32;
        let y: i32;

        // Decode X
        if val1 & FLAG_LONG != 0 {
            if val1 & TRIM_CODE != 0 {
                trim = true;
            }
            if val1 & JUMP_CODE != 0 {
                jump = true;
            }
            let code = ((val1 as u16) << 8) | (val2 as u16);
            x = signed12(code);
        } else {
            x = signed7(val1);
        }

        // Decode Y — in the long-X case, the next unread byte becomes Y.
        let y_byte1 = if val1 & FLAG_LONG != 0 {
            read_u8(cursor)?
        } else {
            val2
        };

        if y_byte1 & FLAG_LONG != 0 {
            if y_byte1 & TRIM_CODE != 0 {
                trim = true;
            }
            if y_byte1 & JUMP_CODE != 0 {
                jump = true;
            }
            let y_byte2 = read_u8(cursor)?;
            let code = ((y_byte1 as u16) << 8) | (y_byte2 as u16);
            y = signed12(code);
        } else {
            y = signed7(y_byte1);
        }

        if jump {
            pattern.add_stitch_relative(StitchType::Jump, x as f32, y as f32);
        } else if trim {
            pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
            pattern.add_stitch_relative(StitchType::Jump, x as f32, y as f32);
        } else {
            pattern.add_stitch_relative(StitchType::Stitch, x as f32, y as f32);
        }
    }

    let (end_x, end_y) = pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0));
    pattern.add_stitch_absolute(StitchType::End, end_x, end_y);
    Ok(())
}

// ===========================================================================
// PEC colour & graphics reading
// ===========================================================================

/// Process PEC colour bytes using the built-in thread palette.
fn process_pec_colors(color_bytes: &[u8], pattern: &mut EmbPattern, values: &mut Vec<EmbThread>) {
    let thread_set = get_pec_thread_set();
    let max_value = thread_set.len();
    for &byte in color_bytes {
        let idx = byte as usize % max_value;
        if let Some(thread) = &thread_set[idx] {
            pattern.add_thread(thread.clone());
            values.push(thread.clone());
        }
    }
}

/// Process PEC colours with a PES chart mapping.
fn process_pec_table(
    color_bytes: &[u8],
    pattern: &mut EmbPattern,
    chart: &mut Vec<EmbThread>,
    values: &mut Vec<EmbThread>,
) {
    let thread_set = get_pec_thread_set();
    let max_value = thread_set.len();
    let mut thread_map: std::collections::HashMap<usize, EmbThread> =
        std::collections::HashMap::new();

    for &byte in color_bytes {
        let color_index = byte as usize % max_value;
        let thread = thread_map.get(&color_index).cloned().unwrap_or_else(|| {
            if let Some(t) = chart.pop().or_else(|| thread_set[color_index].clone()) {
                t
            } else {
                // fallback: empty thread
                EmbThread::new(0x000000)
            }
        });
        thread_map.insert(color_index, thread.clone());
        pattern.add_thread(thread.clone());
        values.push(thread);
    }
}

fn map_pec_colors(
    color_bytes: &[u8],
    pattern: &mut EmbPattern,
    chart: &mut Vec<EmbThread>,
    values: &mut Vec<EmbThread>,
) {
    if chart.is_empty() {
        // Reading PEC colors directly
        process_pec_colors(color_bytes, pattern, values);
    } else if chart.len() >= color_bytes.len() {
        // 1:1 mode – use chart threads directly
        for thread in chart.iter() {
            pattern.add_thread(thread.clone());
            values.push(thread.clone());
        }
    } else {
        // Tabled mode
        process_pec_table(color_bytes, pattern, chart, values);
    }
}

/// Read the embedded PEC block that follows the PES header.
fn read_pec(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
    pes_chart: &mut Vec<EmbThread>,
) -> Result<(), binrw::Error> {
    // Skip 3 bytes: "LA:"
    cursor.seek(SeekFrom::Current(3))?;

    // Read label (16 chars)
    let label_bytes = read_exact(cursor, 16)?;
    let label = String::from_utf8_lossy(&label_bytes).trim().to_string();
    if !label.is_empty() {
        pattern.extras.insert("Name".into(), label);
    }

    // Skip 0xF bytes
    cursor.seek(SeekFrom::Current(0xF))?;

    let pec_graphic_byte_stride = read_u8(cursor)?;
    let pec_graphic_icon_height = read_u8(cursor)?;

    // Skip 0xC bytes
    cursor.seek(SeekFrom::Current(0xC))?;

    let color_changes = read_u8(cursor)?;
    let count_colors = color_changes as usize + 1; // PEC uses cc - 1, 0xFF means 0.
    let color_bytes = read_exact(cursor, count_colors)?;

    let mut values: Vec<EmbThread> = Vec::new();
    map_pec_colors(&color_bytes, pattern, pes_chart, &mut values);

    // Skip remaining header bytes to reach 0x1D0 relative to seek point
    let skip = 0x1D0_usize.saturating_sub(color_changes as usize);
    cursor.seek(SeekFrom::Current(skip as i64))?;

    // Read stitch block end offset (24-bit LE)
    let pec_block_start = cursor.position();
    let stitch_block_end_offset = read_u24_le(cursor)?;
    let stitch_block_end = if stitch_block_end_offset >= 5 {
        pec_block_start + (stitch_block_end_offset as u64) - 5u64
    } else {
        pec_block_start
    };

    // Skip 0x0B bytes (3 bytes + 4×2-byte shorts)
    cursor.seek(SeekFrom::Current(0x0B))?;

    // Read PEC stitches
    read_pec_stitches(cursor, pattern)?;

    // Seek to stitch block end, but only if within file bounds
    if (stitch_block_end as u64) < cursor.get_ref().len() as u64 {
        cursor.seek(SeekFrom::Start(stitch_block_end as u64))?;

        // Read PEC graphics (store as metadata)
        let byte_size = pec_graphic_byte_stride as usize * pec_graphic_icon_height as usize;
        for i in 0..count_colors {
            let graphic = read_exact(cursor, byte_size)?;
            let name = format!("pec_graphic_{}", i);
            let thread_color = values.get(i).map(|t| t.hex_color()).unwrap_or_default();
            // Store as hex-encoded string for now (graphic + stride + thread)
            let graphic_hex: String = graphic.iter().map(|b| format!("{:02x}", b)).collect();
            pattern.extras.insert(
                name,
                format!(
                    "{};{};{}",
                    graphic_hex, pec_graphic_byte_stride, thread_color
                ),
            );
        }
    }

    Ok(())
}

// ===========================================================================
// PES header reading (version-specific)
// ===========================================================================

fn read_pes_metadata(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
) -> Result<(), binrw::Error> {
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("name".into(), v);
        }
    }
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("category".into(), v);
        }
    }
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("author".into(), v);
        }
    }
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("keywords".into(), v);
        }
    }
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("comments".into(), v);
        }
    }
    Ok(())
}

fn read_pes_thread(
    cursor: &mut Cursor<&[u8]>,
    threadlist: &mut Vec<EmbThread>,
) -> Result<(), binrw::Error> {
    let catalog_number = read_pes_string(cursor)?;
    let color = 0xFF000000 | read_u24_be(cursor)?;
    cursor.seek(SeekFrom::Current(5))?;
    let description = read_pes_string(cursor)?;
    let brand = read_pes_string(cursor)?;
    let chart = read_pes_string(cursor)?;

    threadlist.push(EmbThread {
        color,
        description,
        catalog_number,
        details: None,
        brand,
        chart,
        weight: None,
    });
    Ok(())
}

fn skip_complex_items(
    cursor: &mut Cursor<&[u8]>,
    threadlist: &mut Vec<EmbThread>,
) -> Result<bool, binrw::Error> {
    let count_programmable_fills = read_u16_le(cursor)?;
    if count_programmable_fills != 0 {
        return Ok(true);
    }
    let count_motifs = read_u16_le(cursor)?;
    if count_motifs != 0 {
        return Ok(true);
    }
    let count_feather_patterns = read_u16_le(cursor)?;
    if count_feather_patterns != 0 {
        return Ok(true);
    }
    let count_threads = read_u16_le(cursor)?;
    for _ in 0..count_threads {
        read_pes_thread(cursor, threadlist)?;
    }
    Ok(false)
}

fn read_pes_header_version_1(
    _cursor: &mut Cursor<&[u8]>,
    _pattern: &mut EmbPattern,
) -> Result<(), binrw::Error> {
    // Nothing to parse
    Ok(())
}

fn read_pes_header_version_4(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
) -> Result<(), binrw::Error> {
    cursor.seek(SeekFrom::Current(4))?;
    read_pes_metadata(cursor, pattern)?;
    Ok(())
}

fn read_pes_header_version_5_inner(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
    skip_size1: i64,
    skip_size2: i64,
) -> Result<(), binrw::Error> {
    cursor.seek(SeekFrom::Current(4))?;
    read_pes_metadata(cursor, pattern)?;
    cursor.seek(SeekFrom::Current(skip_size1))?;
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("image_file".into(), v);
        }
    }
    cursor.seek(SeekFrom::Current(skip_size2))?;
    skip_complex_items(cursor, threadlist)?;
    Ok(())
}

fn read_pes_header_version_5(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
) -> Result<(), binrw::Error> {
    read_pes_header_version_5_inner(cursor, pattern, threadlist, 24, 24)
}

fn read_pes_header_version_6(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
) -> Result<(), binrw::Error> {
    read_pes_header_version_5_inner(cursor, pattern, threadlist, 36, 24)
}

fn read_pes_header_version_7(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
) -> Result<(), binrw::Error> {
    read_pes_header_version_5_inner(cursor, pattern, threadlist, 36, 24)
}

fn read_pes_header_version_8(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
) -> Result<(), binrw::Error> {
    read_pes_header_version_5_inner(cursor, pattern, threadlist, 38, 26)
}

fn read_pes_header_version_9(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
) -> Result<(), binrw::Error> {
    cursor.seek(SeekFrom::Current(4))?;
    read_pes_metadata(cursor, pattern)?;
    cursor.seek(SeekFrom::Current(14))?;
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("hoop_name".into(), v);
        }
    }
    cursor.seek(SeekFrom::Current(30))?;
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("image_file".into(), v);
        }
    }
    cursor.seek(SeekFrom::Current(34))?;
    skip_complex_items(cursor, threadlist)?;
    Ok(())
}

fn read_pes_header_version_10(
    cursor: &mut Cursor<&[u8]>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
) -> Result<(), binrw::Error> {
    cursor.seek(SeekFrom::Current(4))?;
    read_pes_metadata(cursor, pattern)?;
    cursor.seek(SeekFrom::Current(14))?;
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("hoop_name".into(), v);
        }
    }
    cursor.seek(SeekFrom::Current(38))?;
    if let Some(v) = read_pes_string(cursor)? {
        if !v.is_empty() {
            pattern.extras.insert("image_file".into(), v);
        }
    }
    cursor.seek(SeekFrom::Current(34))?;
    skip_complex_items(cursor, threadlist)?;
    Ok(())
}

// ===========================================================================
// Public entry-point
// ===========================================================================

/// Parse a PES-format Brother embroidery file from a byte buffer.
///
/// Returns an [`EmbPattern`] containing the stitches, threads, and metadata
/// read from the file.
///
/// # Errors
///
/// Returns a [`binrw::Error`] if the data is truncated or malformed.
pub fn read_pes(data: &[u8]) -> Result<EmbPattern, binrw::Error> {
    let mut cursor = Cursor::new(data);
    let mut pattern = EmbPattern::new();
    let mut loaded_thread_values: Vec<EmbThread> = Vec::new();

    // Read PES header magic string (8 bytes)
    let pes_string_bytes = read_exact(&mut cursor, 8)?;
    let pes_string = String::from_utf8_lossy(&pes_string_bytes);

    if pes_string == "#PEC0001" {
        read_pec(&mut cursor, &mut pattern, &mut loaded_thread_values)?;
        interpolate_duplicate_color_as_stop(&mut pattern);
        return Ok(pattern);
    }

    let pec_block_position = read_i32_le(&mut cursor)?;

    // Parse version-specific header
    match pes_string.as_ref() {
        "#PES0100" => {
            pattern.extras.insert("version".into(), "10".into());
            read_pes_header_version_10(&mut cursor, &mut pattern, &mut loaded_thread_values)?;
        }
        "#PES0090" => {
            pattern.extras.insert("version".into(), "9".into());
            read_pes_header_version_9(&mut cursor, &mut pattern, &mut loaded_thread_values)?;
        }
        "#PES0080" => {
            pattern.extras.insert("version".into(), "8".into());
            read_pes_header_version_8(&mut cursor, &mut pattern, &mut loaded_thread_values)?;
        }
        "#PES0070" => {
            pattern.extras.insert("version".into(), "7".into());
            read_pes_header_version_7(&mut cursor, &mut pattern, &mut loaded_thread_values)?;
        }
        "#PES0060" => {
            pattern.extras.insert("version".into(), "6".into());
            read_pes_header_version_6(&mut cursor, &mut pattern, &mut loaded_thread_values)?;
        }
        "#PES0050" | "#PES0055" | "#PES0056" => {
            pattern.extras.insert("version".into(), "5".into());
            read_pes_header_version_5(&mut cursor, &mut pattern, &mut loaded_thread_values)?;
        }
        "#PES0040" => {
            pattern.extras.insert("version".into(), "4".into());
            read_pes_header_version_4(&mut cursor, &mut pattern)?;
        }
        "#PES0030" => {
            pattern.extras.insert("version".into(), "3".into());
        }
        "#PES0022" => {
            pattern.extras.insert("version".into(), "2.2".into());
        }
        "#PES0020" => {
            pattern.extras.insert("version".into(), "2".into());
        }
        "#PES0001" => {
            pattern.extras.insert("version".into(), "1".into());
            read_pes_header_version_1(&mut cursor, &mut pattern)?;
        }
        _ => {
            // Unrecognised header — skip header parsing
        }
    }

    // Seek to embedded PEC block and read it
    cursor.seek(SeekFrom::Start(pec_block_position as u64))?;
    read_pec(&mut cursor, &mut pattern, &mut loaded_thread_values)?;
    interpolate_duplicate_color_as_stop(&mut pattern);
    Ok(pattern)
}

pub struct PesReader;

impl EmbroideryReader for PesReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, crate::error::AppError> {
        read_pes(data).map_err(|err| crate::error::AppError::parse(format!("PES parse failed: {err}")))
    }
}

// ===========================================================================
// Post-processing: interpolate duplicate colour as STOP
// ===========================================================================

/// If two consecutive thread entries are identical, replace the corresponding
/// COLOR_CHANGE with a STOP. This matches Python's
/// `out.interpolate_duplicate_color_as_stop()`.
fn interpolate_duplicate_color_as_stop(pattern: &mut EmbPattern) {
    let mut thread_index: usize = 0;
    let mut init_color = true;
    let mut last_change: Option<usize> = None;

    for position in 0..pattern.stitches.len() {
        let stype = pattern.stitches[position].stitch_type;
        match stype {
            StitchType::Stitch | StitchType::SewTo | StitchType::NeedleAt => {
                if init_color {
                    if let Some(lc) = last_change {
                        // Check if the current thread matches the previous one
                        let prev_idx = thread_index.saturating_sub(1);
                        if thread_index > 0
                            && thread_index < pattern.threadlist.len()
                            && pattern.threadlist[prev_idx] == pattern.threadlist[thread_index]
                        {
                            // Duplicate: remove the duplicate thread and replace
                            // the colour-change with a STOP
                            pattern.threadlist.remove(thread_index);
                            pattern.stitches[lc].stitch_type = StitchType::Stop;
                        } else {
                            thread_index += 1;
                        }
                    } else {
                        thread_index += 1;
                    }
                    init_color = false;
                }
            }
            StitchType::ColorChange | StitchType::ColorBreak | StitchType::NeedleSet => {
                init_color = true;
                last_change = Some(position);
            }
            _ => {}
        }
    }
}

// ===========================================================================
// Unit tests
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ── Phase 1: Low-level helpers ─────────────────────────────────────

    #[test]
    fn test_signed7_basic() {
        assert_eq!(signed7(0), 0);
        assert_eq!(signed7(10), 10);
        assert_eq!(signed7(63), 63);
    }

    #[test]
    fn test_signed7_negative() {
        // signed7 treats value > 63 as negative via -128 + b
        assert_eq!(signed7(200), 200 - 128); // = 72
        assert_eq!(signed7(127), -1);  // -128 + 127 = -1
        assert_eq!(signed7(255), 127); // -128 + 255 = 127 (not -1 since 255 ≠ 127)
        assert_eq!(signed7(128), 0);   // -128 + 128 = 0
        assert_eq!(signed7(129), 1);   // -128 + 129 = 1
        assert_eq!(signed7(64), -64);  // -128 + 64 = -64
    }

    #[test]
    fn test_signed12_basic() {
        assert_eq!(signed12(0), 0);
        assert_eq!(signed12(100), 100);
        assert_eq!(signed12(0x7FF), 0x7FF);
    }

    #[test]
    fn test_signed12_negative() {
        assert_eq!(signed12(0xFFF), -1);
        assert_eq!(signed12(0x800), -2048);
        assert_eq!(signed12(0x801), -2047);
        assert_eq!(signed12(0x876), -1930);
        assert_eq!(signed12(0xABC), (-0x1000 + 0xABC) as i32);
    }

    #[test]
    fn test_read_exact_success() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let mut cursor = Cursor::new(&data[..]);
        let bytes = read_exact(&mut cursor, 3).expect("read 3 bytes should succeed");
        assert_eq!(bytes, vec![0x01, 0x02, 0x03]);
        assert_eq!(cursor.position(), 3);
    }

    #[test]
    fn test_read_exact_eof() {
        let data = [0x01, 0x02];
        let mut cursor = Cursor::new(&data[..]);
        let result = read_exact(&mut cursor, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_exact_zero_bytes() {
        let data = [];
        let mut cursor = Cursor::new(&data[..]);
        let bytes = read_exact(&mut cursor, 0).expect("read 0 bytes should succeed");
        assert!(bytes.is_empty());
    }

    #[test]
    fn test_read_u8_ok() {
        let data = [0xAB];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(read_u8(&mut cursor).unwrap(), 0xAB);
        assert_eq!(cursor.position(), 1);
    }

    #[test]
    fn test_read_u8_eof() {
        let data = [];
        let mut cursor = Cursor::new(&data[..]);
        assert!(read_u8(&mut cursor).is_err());
    }

    #[test]
    fn test_read_u16_le_ok() {
        let data = [0x34, 0x12];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(read_u16_le(&mut cursor).unwrap(), 0x1234);
        assert_eq!(cursor.position(), 2);
    }

    #[test]
    fn test_read_u16_le_eof() {
        let data = [0x01];
        let mut cursor = Cursor::new(&data[..]);
        assert!(read_u16_le(&mut cursor).is_err());
    }

    #[test]
    fn test_read_i32_le_positive() {
        let data = [0x78, 0x56, 0x34, 0x12];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(read_i32_le(&mut cursor).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i32_le_negative() {
        // -1 in little-endian
        let data = [0xFF, 0xFF, 0xFF, 0xFF];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(read_i32_le(&mut cursor).unwrap(), -1);
    }

    #[test]
    fn test_read_i32_le_eof() {
        let data = [0x01, 0x02, 0x03];
        let mut cursor = Cursor::new(&data[..]);
        assert!(read_i32_le(&mut cursor).is_err());
    }

    #[test]
    fn test_read_u24_be_ok() {
        let data = [0x01, 0x02, 0x03];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(read_u24_be(&mut cursor).unwrap(), 0x010203);
        assert_eq!(cursor.position(), 3);
    }

    #[test]
    fn test_read_u24_be_eof() {
        let data = [0x01, 0x02];
        let mut cursor = Cursor::new(&data[..]);
        assert!(read_u24_be(&mut cursor).is_err());
    }

    #[test]
    fn test_read_u24_le_ok() {
        let data = [0x03, 0x02, 0x01];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(read_u24_le(&mut cursor).unwrap(), 0x010203);
        assert_eq!(cursor.position(), 3);
    }

    #[test]
    fn test_read_u24_le_eof() {
        let data = [0x01];
        let mut cursor = Cursor::new(&data[..]);
        assert!(read_u24_le(&mut cursor).is_err());
    }

    #[test]
    fn test_read_pes_string_empty() {
        let data = [0x00];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(read_pes_string(&mut cursor).unwrap(), None);
    }

    #[test]
    fn test_read_pes_string_non_empty() {
        let data = [0x05, b'H', b'e', b'l', b'l', b'o'];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(
            read_pes_string(&mut cursor).unwrap(),
            Some("Hello".to_string())
        );
    }

    #[test]
    fn test_read_pes_string_utf8_lossy() {
        // Invalid UTF-8 byte should be handled via lossy conversion
        let data = [0x02, 0xFF, 0xFE];
        let mut cursor = Cursor::new(&data[..]);
        let result = read_pes_string(&mut cursor).unwrap();
        assert!(result.is_some());
        // The replacement character (U+FFFD) encodes as 3 bytes in UTF-8
        let s = result.as_deref().unwrap();
        assert_eq!(s.chars().count(), 2, "should have two replacement chars");
        assert!(s.contains('\u{FFFD}'));
    }

    // ── Phase 2: PEC thread palette & colour processing ────────────────

    #[test]
    fn test_get_pec_thread_set_size() {
        let set = get_pec_thread_set();
        assert_eq!(set.len(), 65);
    }

    #[test]
    fn test_get_pec_thread_set_first_is_none() {
        let set = get_pec_thread_set();
        assert!(set[0].is_none());
    }

    #[test]
    fn test_get_pec_thread_set_known_threads() {
        let set = get_pec_thread_set();
        // Index 5 = Red
        let red = set[5].as_ref().expect("index 5 should be Some");
        assert_eq!(red.hex_color(), "#ed171f");
        assert_eq!(red.description.as_deref(), Some("Red"));

        // Index 20 = Black
        let black = set[20].as_ref().expect("index 20 should be Some");
        assert_eq!(black.hex_color(), "#000000");
        assert_eq!(black.description.as_deref(), Some("Black"));

        // Index 29 = White
        let white = set[29].as_ref().expect("index 29 should be Some");
        assert_eq!(white.hex_color(), "#f0f0f0");
        assert_eq!(white.description.as_deref(), Some("White"));
    }

    #[test]
    fn test_get_pec_thread_set_all_non_none_have_brand_and_chart() {
        let set = get_pec_thread_set();
        for (i, thread_opt) in set.iter().enumerate().skip(1) {
            let thread = thread_opt
                .as_ref()
                .unwrap_or_else(|| panic!("Index {i} should be Some"));
            assert!(thread.description.is_some(), "Index {i} missing description");
            assert!(thread.catalog_number.is_some(), "Index {i} missing catalog_number");
            assert_eq!(thread.brand.as_deref(), Some("Brother"), "Index {i} wrong brand");
            assert_eq!(thread.chart.as_deref(), Some("Brother"), "Index {i} wrong chart");
        }
    }

    #[test]
    fn test_process_pec_colors_adds_threads() {
        let mut pattern = EmbPattern::new();
        let mut values = Vec::new();
        // 5 = Red, 20 = Black
        process_pec_colors(&[5, 20], &mut pattern, &mut values);
        assert_eq!(pattern.threadlist.len(), 2);
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].hex_color(), "#ed171f");
        assert_eq!(values[1].hex_color(), "#000000");
    }

    #[test]
    fn test_process_pec_colors_wraps_index() {
        let mut pattern = EmbPattern::new();
        let mut values = Vec::new();
        // Index should wrap at 65. 65 % 65 = 0 (None), 66 % 65 = 1 (Prussian Blue)
        process_pec_colors(&[65, 66], &mut pattern, &mut values);
        assert_eq!(values.len(), 1); // Only index 1 yields Some
        assert_eq!(values[0].description.as_deref(), Some("Prussian Blue"));
    }

    #[test]
    fn test_map_pec_colors_empty_chart_direct() {
        let mut pattern = EmbPattern::new();
        let mut chart = Vec::new();
        let mut values = Vec::new();
        map_pec_colors(&[5, 20], &mut pattern, &mut chart, &mut values);
        assert_eq!(pattern.threadlist.len(), 2);
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_map_pec_colors_one_to_one() {
        let mut pattern = EmbPattern::new();
        let mut chart = vec![EmbThread::new(0xFF0000), EmbThread::new(0x00FF00)];
        let mut values = Vec::new();
        map_pec_colors(&[5, 20], &mut pattern, &mut chart, &mut values);
        assert_eq!(pattern.threadlist.len(), 2);
        assert_eq!(values[0].color, 0xFF0000);
        assert_eq!(values[1].color, 0x00FF00);
        // chart should not be empty (iter() doesn't consume it)
        assert_eq!(chart.len(), 2);
    }

    #[test]
    fn test_map_pec_colors_tabled_mode() {
        let mut pattern = EmbPattern::new();
        let mut chart = vec![EmbThread::new(0xFF0000)]; // fewer entries than color_bytes
        let mut values = Vec::new();
        map_pec_colors(&[5, 20, 29], &mut pattern, &mut chart, &mut values);
        // First color uses chart entry (popped), remaining fall back to PEC palette
        assert_eq!(pattern.threadlist.len(), 3);
        // chart is empty since process_pec_table pops from it
        assert!(chart.is_empty());
        assert_eq!(values[0].color, 0xFF0000);
        // values[1] and values[2] come from PEC palette
        let thread_set = get_pec_thread_set();
        assert_eq!(
            values[1],
            *thread_set[20].as_ref().expect("index 20 should be Some")
        );
        assert_eq!(
            values[2],
            *thread_set[29].as_ref().expect("index 29 should be Some")
        );
    }

    // ── Phase 3: PEC stitch edge cases ─────────────────────────────────

    #[test]
    fn test_read_pec_stitches_trim_flag_produces_trim_then_jump() {
        // X byte with TRIM_CODE | FLAG_LONG => trim flag set
        // val1 = 0x80 (FLAG_LONG) | 0x20 (TRIM_CODE) | 0x05 = 0xA5
        // val2 = 0x00 => X = 0x0A05? No, re-read the logic...
        // Let me construct carefully:
        // First byte val1 = TRIM_CODE | FLAG_LONG | 0x05 = 0x20 | 0x80 | 0x05 = 0xA5
        // val2 = 0x00 => combined = 0xA500, signed12 = ?
        // Actually simpler: use val1 = 0xA0 (FLAG_LONG | TRIM_CODE, w/ 0 shift)
        // val2 = 0x00 => X = signed12(0xA000) = signed12(0xA00) = signed12(2560) -> 2560? No, 0xA00 & 0xFFF = 0xA00 = 2560 which is > 0x7FF so -0x1000 + 2560 = -1536
        // Let me use a simpler pattern
        let data = [
            0xA0, 0x00, // X: FLAG_LONG | TRIM_CODE, val2=0 => X=0 (signed12(0x000)=0)
            0x05,       // Y short = +5
            0xFF, 0x00,
        ];
        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();

        read_pec_stitches(&mut cursor, &mut pattern).expect("stitch decode should succeed");

        // Trim flag produces a Trim command + Jump command
        assert_eq!(pattern.count_stitch_commands(StitchType::Trim), 1);
        assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
        assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 0);
    }

    #[test]
    fn test_read_pec_stitches_long_y() {
        // X short = +3, Y long with jump flag
        // val1 = 0x03 (short X = +3)
        // val2 = 0x91 (FLAG_LONG | JUMP_CODE | 0x01) => y_byte1 = 0x91
        // Need another byte for y_byte2
        let data = [
            0x03, 0x91, // X short=+3, Y byte1=0x91 (long + jump)
            0x05,       // Y byte2 = 0x05 => Y = 0x9105 & 0xFFF = 0x105 = 261
            0xFF, 0x00,
        ];
        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();

        read_pec_stitches(&mut cursor, &mut pattern).expect("stitch decode should succeed");

        // Should be a jump because JUMP_CODE was set
        assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
        let jump = pattern
            .stitches
            .iter()
            .find(|s| s.stitch_type == StitchType::Jump)
            .expect("expected jump stitch");
        assert_eq!(jump.x, 3.0);
        assert_eq!(jump.y, 261.0);
    }

    #[test]
    fn test_read_pec_stitches_short_x_short_y() {
        let data = [
            0x0A, 0x14, // X short = +10, Y short = +20
            0xFF, 0x00,
        ];
        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();

        read_pec_stitches(&mut cursor, &mut pattern).expect("stitch decode should succeed");

        assert_eq!(pattern.stitches.len(), 2); // stitch + end
        assert_eq!(pattern.stitches[0].stitch_type, StitchType::Stitch);
        assert_eq!(pattern.stitches[0].x, 10.0);
        assert_eq!(pattern.stitches[0].y, 20.0);
    }

    #[test]
    fn test_read_pec_stitches_truncated_data_breaks_gracefully() {
        // Only 1 byte, should break after first read_u8
        let data = [0x01];
        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();

        // Should not panic, should break out of loop and add End
        read_pec_stitches(&mut cursor, &mut pattern).expect("should handle truncated data");

        // Only the End stitch should be present
        assert_eq!(pattern.stitches.len(), 1);
        assert_eq!(pattern.stitches[0].stitch_type, StitchType::End);
    }

    #[test]
    fn test_read_pec_stitches_long_x_short_y() {
        // X long (no jump/trim flags), Y short
        // val1 = 0x80 | 0x05 = 0x85, val2 = 0x01 => X = signed12(0x8501 & 0xFFF) = signed12(0x501) = 0x501 = 1281
        // y_byte1 = val2? No - re-read: in the long-X case, the next unread byte becomes Y.
        // val1=0x80|0x05 (FLAG_LONG + value bits), val2=0x00 (actually gets consumed in X decode)
        // Actually let me re-check: val1 = 0x85, val2 = 0x01. val1 & FLAG_LONG != 0.
        // So combined = (0x85 << 8) | 0x01 = 0x8501. Signed12(0x8501 & 0xFFF) = signed12(0x501) = 1281.
        // Then y_byte1 = read_u8(cursor) - reads the next unconsumed byte.
        // Let me just use a simple one:
        let data = [
            0x80, 0x00, // FLAG_LONG, X=0 (combined=0x8000, &0xFFF=0)
            0x07,       // Y short = +7
            0xFF, 0x00,
        ];
        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();

        read_pec_stitches(&mut cursor, &mut pattern).expect("stitch decode should succeed");

        assert_eq!(pattern.stitches[0].stitch_type, StitchType::Stitch);
        assert_eq!(pattern.stitches[0].x, 0.0);
        assert_eq!(pattern.stitches[0].y, 7.0);
    }

    #[test]
    fn test_read_pec_stitches_sequential_stitches_accumulate() {
        let data = [
            0x01, 0x01, // stitch 1 (+1,+1)
            0x02, 0x02, // stitch 2 (+2,+2)
            0x03, 0x03, // stitch 3 (+3,+3)
            0xFF, 0x00,
        ];
        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();

        read_pec_stitches(&mut cursor, &mut pattern).expect("stitch decode should succeed");

        assert_eq!(pattern.stitches.len(), 4); // 3 stitches + 1 end
        assert_eq!(pattern.stitches[0].x, 1.0);
        assert_eq!(pattern.stitches[0].y, 1.0);
        assert_eq!(pattern.stitches[1].x, 3.0); // accumulated: 1+2
        assert_eq!(pattern.stitches[1].y, 3.0); // accumulated: 1+2
        assert_eq!(pattern.stitches[2].x, 6.0); // accumulated: 3+3
        assert_eq!(pattern.stitches[2].y, 6.0); // accumulated: 3+3
    }

    #[test]
    fn test_read_pec_stitches_empty_no_stitches_only_end() {
        let data = [0xFF, 0x00];
        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();

        read_pec_stitches(&mut cursor, &mut pattern).expect("stitch decode should succeed");

        assert_eq!(pattern.stitches.len(), 1);
        assert_eq!(pattern.stitches[0].stitch_type, StitchType::End);
        assert_eq!(pattern.stitches[0].x, 0.0);
        assert_eq!(pattern.stitches[0].y, 0.0);
    }

    // ── Phase 4: PES metadata & header helpers ─────────────────────────

    #[test]
    fn test_read_pes_metadata_all_fields() {
        // Build a byte stream: 5 PES strings (name, category, author, keywords, comments)
        let mut data = Vec::new();
        // name = "Test" (4 chars)
        data.push(4);
        data.extend_from_slice(b"Test");
        // category = "Embroidery" (10 chars)
        data.push(10);
        data.extend_from_slice(b"Embroidery");
        // author = "Jane" (4 chars)
        data.push(4);
        data.extend_from_slice(b"Jane");
        // keywords = "flower,red" (10 chars)
        data.push(10);
        data.extend_from_slice(b"flower,red");
        // comments = "Nice design" (11 chars)
        data.push(11);
        data.extend_from_slice(b"Nice design");

        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();
        read_pes_metadata(&mut cursor, &mut pattern).expect("metadata read should succeed");

        assert_eq!(pattern.extras.get("name").unwrap(), "Test");
        assert_eq!(pattern.extras.get("category").unwrap(), "Embroidery");
        assert_eq!(pattern.extras.get("author").unwrap(), "Jane");
        assert_eq!(pattern.extras.get("keywords").unwrap(), "flower,red");
        // comments is length = 12 but "Nice design" is 11 chars, extra byte?
        // Actually "Nice design" has 11 characters. Length prefix is 12.
        // So the parser reads 12 bytes: "Nice design" + one extra byte
        assert_eq!(pattern.extras.get("comments").unwrap(), "Nice design");
    }

    #[test]
    fn test_read_pes_metadata_empty_strings_skipped() {
        // All empty strings should not insert into extras
        let data = vec![0x00; 5]; // 5 zero-length strings
        let mut cursor = Cursor::new(&data[..]);
        let mut pattern = EmbPattern::new();
        read_pes_metadata(&mut cursor, &mut pattern).expect("metadata read should succeed");

        assert!(!pattern.extras.contains_key("name"));
        assert!(!pattern.extras.contains_key("category"));
        assert!(!pattern.extras.contains_key("author"));
        assert!(!pattern.extras.contains_key("keywords"));
        assert!(!pattern.extras.contains_key("comments"));
    }

    #[test]
    fn test_read_pes_thread_full() {
        // Build a PES thread entry:
        // catalog_number = "ABC"
        let mut data = Vec::new();
        data.push(3);
        data.extend_from_slice(b"ABC");
        // color = 0x123456 (24-bit big-endian)
        data.extend_from_slice(&[0x12, 0x34, 0x56]);
        // 5 bytes skip
        data.extend_from_slice(&[0; 5]);
        // description = "Test Thread"
        data.push(11);
        data.extend_from_slice(b"Test Thread");
        // brand = "TestBrand"
        data.push(9);
        data.extend_from_slice(b"TestBrand");
        // chart = "TestChart"
        data.push(9);
        data.extend_from_slice(b"TestChart");

        let mut cursor = Cursor::new(&data[..]);
        let mut threadlist = Vec::new();
        read_pes_thread(&mut cursor, &mut threadlist).expect("thread read should succeed");

        assert_eq!(threadlist.len(), 1);
        assert_eq!(threadlist[0].color, 0xFF123456); // 0xFF000000 | 0x123456
        assert_eq!(
            threadlist[0].catalog_number.as_deref(),
            Some("ABC")
        );
        assert_eq!(
            threadlist[0].description.as_deref(),
            Some("Test Thread")
        );
        assert_eq!(threadlist[0].brand.as_deref(), Some("TestBrand"));
        assert_eq!(threadlist[0].chart.as_deref(), Some("TestChart"));
    }

    #[test]
    fn test_skip_complex_items_early_return_fills() {
        // Non-zero programmable fills => returns true immediately
        let data = [0x01, 0x00, // count programmable fills = 1
                    0x00, 0x00, // count motifs = 0
                    0x00, 0x00, // count feather patterns = 0
                    0x00, 0x00]; // count threads = 0
        let mut cursor = Cursor::new(&data[..]);
        let mut threadlist = Vec::new();
        let result = skip_complex_items(&mut cursor, &mut threadlist)
            .expect("skip should succeed");
        assert!(result); // early return true
        assert!(threadlist.is_empty());
    }

    #[test]
    fn test_skip_complex_items_early_return_motifs() {
        // Only motifs non-zero => returns true
        let data = [0x00, 0x00, // programmable fills = 0
                    0x02, 0x00, // motifs = 2
                    0x00, 0x00, // feather patterns = 0
                    0x00, 0x00]; // threads = 0
        let mut cursor = Cursor::new(&data[..]);
        let mut threadlist = Vec::new();
        let result = skip_complex_items(&mut cursor, &mut threadlist)
            .expect("skip should succeed");
        assert!(result);
    }

    #[test]
    fn test_skip_complex_items_early_return_feather() {
        let data = [0x00, 0x00, // programmable fills = 0
                    0x00, 0x00, // motifs = 0
                    0x01, 0x00, // feather patterns = 1
                    0x00, 0x00]; // threads = 0
        let mut cursor = Cursor::new(&data[..]);
        let mut threadlist = Vec::new();
        let result = skip_complex_items(&mut cursor, &mut threadlist)
            .expect("skip should succeed");
        assert!(result);
    }

    #[test]
    fn test_skip_complex_items_zero_threads() {
        let data = [0x00, 0x00, // programmable fills = 0
                    0x00, 0x00, // motifs = 0
                    0x00, 0x00, // feather patterns = 0
                    0x00, 0x00]; // threads = 0
        let mut cursor = Cursor::new(&data[..]);
        let mut threadlist = Vec::new();
        let result = skip_complex_items(&mut cursor, &mut threadlist)
            .expect("skip should succeed");
        assert!(!result); // no early return
        assert!(threadlist.is_empty());
    }

    // ── Phase 5: Integration tests with real PES files ──────────────────

    #[test]
    fn test_read_bean_pes() {
        let data = include_bytes!("../../tests/Test Designs/Bean.pes");
        let pattern = read_pes(data).expect("Bean.pes should parse successfully");
        assert!(pattern.count_stitches() > 0, "Bean.pes should have stitches");
        assert!(pattern.count_threads() > 0, "Bean.pes should have threads");
        let (min_x, min_y, max_x, max_y) = pattern.bounds();
        assert!(min_x.is_finite());
        assert!(min_y.is_finite());
        assert!(max_x.is_finite());
        assert!(max_y.is_finite());
        assert!(max_x >= min_x);
        assert!(max_y >= min_y);
        assert!(pattern.count_stitch_commands(StitchType::End) >= 1);
    }

    #[test]
    fn test_read_cake3_pes() {
        let data = include_bytes!("../../tests/Test Designs/Cake 3.pes");
        let pattern = read_pes(data).expect("Cake 3.pes should parse successfully");
        assert!(pattern.count_stitches() > 0, "Cake 3.pes should have stitches");
        assert!(pattern.count_threads() > 0, "Cake 3.pes should have threads");
        let (min_x, min_y, max_x, max_y) = pattern.bounds();
        assert!(min_x.is_finite());
        assert!(min_y.is_finite());
        assert!(max_x.is_finite());
        assert!(max_y.is_finite());
        assert!(max_x >= min_x);
        assert!(max_y >= min_y);
    }

    #[test]
    fn test_read_flower_pes() {
        let data = include_bytes!("../../tests/Test Designs/Flower.pes");
        let pattern = read_pes(data).expect("Flower.pes should parse successfully");
        assert!(pattern.count_stitches() > 0, "Flower.pes should have stitches");
        assert!(pattern.count_threads() > 0, "Flower.pes should have threads");
    }

    #[test]
    fn test_read_rose_bouquet_pes() {
        let data = include_bytes!("../../tests/Test Designs/rose_bouquet.pes");
        let pattern = read_pes(data).expect("rose_bouquet.pes should parse successfully");
        assert!(pattern.count_stitches() > 0, "rose_bouquet.pes should have stitches");
        assert!(pattern.count_threads() > 0, "rose_bouquet.pes should have threads");
    }

    /// Validate that every PES file includes version metadata.
    fn assert_version_metadata(pattern: &EmbPattern) {
        assert!(
            pattern.extras.contains_key("version"),
            "Every PES file should have a version entry; got extras keys: {:?}",
            pattern.extras.keys()
        );
    }

    #[test]
    fn test_read_bean_pes_has_version() {
        let data = include_bytes!("../../tests/Test Designs/Bean.pes");
        let pattern = read_pes(data).expect("Bean.pes should parse");
        assert_version_metadata(&pattern);
    }

    #[test]
    fn test_read_cake3_pes_has_version() {
        let data = include_bytes!("../../tests/Test Designs/Cake 3.pes");
        let pattern = read_pes(data).expect("Cake 3.pes should parse");
        assert_version_metadata(&pattern);
    }

    #[test]
    fn test_read_empty_buffer_returns_error() {
        let data = [];
        let result = read_pes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_truncated_pes_header() {
        // Only a few bytes - should fail gracefully
        let data = [0x23, 0x50, 0x45, 0x53];
        let result = read_pes(&data);
        assert!(result.is_err());
    }

    // ── Phase 6: interpolate_duplicate_color_as_stop edge cases ─────────

    #[test]
    fn test_interpolate_duplicate_color_as_stop_typical_case() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_thread(EmbThread::new(0xFF0000)); // duplicate
        pattern.add_thread(EmbThread::new(0x00FF00));

        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 1.0, 1.0);
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 2.0, 2.0);

        interpolate_duplicate_color_as_stop(&mut pattern);

        assert_eq!(pattern.stitches[0].stitch_type, StitchType::ColorChange);
        assert_eq!(pattern.stitches[2].stitch_type, StitchType::Stop);
        assert_eq!(pattern.threadlist.len(), 2);
    }

    #[test]
    fn test_interpolate_duplicate_color_no_duplicate() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_thread(EmbThread::new(0x00FF00));
        pattern.add_thread(EmbThread::new(0x0000FF));

        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 1.0, 1.0);
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 2.0, 2.0);
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 3.0, 3.0);

        interpolate_duplicate_color_as_stop(&mut pattern);

        // No duplicates -> all stay ColorChange
        assert_eq!(pattern.stitches[0].stitch_type, StitchType::ColorChange);
        assert_eq!(pattern.stitches[2].stitch_type, StitchType::ColorChange);
        assert_eq!(pattern.stitches[4].stitch_type, StitchType::ColorChange);
        assert_eq!(pattern.threadlist.len(), 3);
    }

    #[test]
    fn test_interpolate_duplicate_color_no_color_changes() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));

        pattern.add_stitch_relative(StitchType::Stitch, 1.0, 1.0);
        pattern.add_stitch_relative(StitchType::Stitch, 2.0, 2.0);

        interpolate_duplicate_color_as_stop(&mut pattern);

        // No ColorChange -> nothing to do, pattern unchanged
        assert_eq!(pattern.stitches.len(), 2);
        assert_eq!(pattern.threadlist.len(), 1);
    }

    #[test]
    fn test_interpolate_duplicate_color_multiple_duplicates() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_thread(EmbThread::new(0xFF0000)); // duplicate 1
        pattern.add_thread(EmbThread::new(0x00FF00));
        pattern.add_thread(EmbThread::new(0x00FF00)); // duplicate 2

        // ColorChange -> Stitch -> ColorChange -> Stitch -> ColorChange -> Stitch -> ColorChange -> Stitch
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 1.0, 1.0);
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 2.0, 2.0);
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 3.0, 3.0);
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 4.0, 4.0);

        interpolate_duplicate_color_as_stop(&mut pattern);

        // First duplicate: position 2 becomes Stop
        // Second duplicate: position 6 becomes Stop
        assert_eq!(pattern.stitches[0].stitch_type, StitchType::ColorChange);
        assert_eq!(pattern.stitches[2].stitch_type, StitchType::Stop);
        assert_eq!(pattern.stitches[4].stitch_type, StitchType::ColorChange);
        assert_eq!(pattern.stitches[6].stitch_type, StitchType::Stop);
        assert_eq!(pattern.threadlist.len(), 2); // both duplicates removed
    }

    #[test]
    fn test_interpolate_duplicate_color_with_needle_set_before_stitches() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_thread(EmbThread::new(0xFF0000)); // duplicate

        // ColorChange -> NeedleSet -> Stitch -> ColorChange -> Stitch
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_absolute(StitchType::NeedleSet, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 1.0, 1.0);
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 2.0, 2.0);

        interpolate_duplicate_color_as_stop(&mut pattern);

        // duplicate should be detected and removed
        assert_eq!(pattern.stitches[3].stitch_type, StitchType::Stop);
        assert_eq!(pattern.threadlist.len(), 1);
    }

    // ── Phase 7: PES version-specific header functions ──────────────────

    /// Helper: build a buffer for read_pes_metadata() containing 5 empty strings.
    fn metadata_buffer() -> Vec<u8> {
        vec![0x00; 5] // 5 zero-length strings
    }

    /// Helper: build a buffer for skip_complex_items() where all counts are 0.
    fn skip_complex_items_zero_buffer() -> Vec<u8> {
        vec![0x00; 8] // 4× u16-le = 8 bytes
    }

    #[test]
    fn test_read_pes_header_version_4_parses_metadata() {
        // 4 bytes padding + metadata (5 empty strings)
        let mut buf = vec![0u8; 4];
        buf.extend_from_slice(&metadata_buffer());
        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();

        read_pes_header_version_4(&mut cursor, &mut pattern)
            .expect("v4 header should succeed");

        // Metadata was called with empty strings — no extras inserted
        assert!(!pattern.extras.contains_key("name"));
        assert_eq!(cursor.position(), (4 + 5) as u64);
    }

    #[test]
    fn test_read_pes_header_version_5_inner_basic() {
        // 4 padding + metadata (empty) + skip_size1 bytes + image_file string (empty) + skip_size2 bytes + skip_complex_items (all zeros)
        let skip1: i64 = 10;
        let skip2: i64 = 8;

        let mut buf = vec![0u8; 4]; // padding
        buf.extend_from_slice(&metadata_buffer()); // 5 bytes
        buf.extend_from_slice(&vec![0u8; skip1 as usize]); // skip1
        buf.push(0x00); // image_file: empty string
        buf.extend_from_slice(&vec![0u8; skip2 as usize]); // skip2
        buf.extend_from_slice(&skip_complex_items_zero_buffer()); // 8 bytes

        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_5_inner(
            &mut cursor,
            &mut pattern,
            &mut threadlist,
            skip1,
            skip2,
        )
        .expect("v5 inner should succeed");

        assert!(!pattern.extras.contains_key("name"));
        assert!(!pattern.extras.contains_key("image_file"));
        assert!(threadlist.is_empty());
    }

    #[test]
    fn test_read_pes_header_version_5_inner_with_image_file() {
        let skip1: i64 = 4;
        let skip2: i64 = 4;

        let mut buf = vec![0u8; 4]; // padding
        buf.extend_from_slice(&metadata_buffer()); // 5 bytes
        buf.extend_from_slice(&vec![0u8; skip1 as usize]); // skip1
        // image_file = "MyImage.png" (11 chars)
        buf.push(11);
        buf.extend_from_slice(b"MyImage.png");
        buf.extend_from_slice(&vec![0u8; skip2 as usize]); // skip2
        buf.extend_from_slice(&skip_complex_items_zero_buffer()); // 8 bytes

        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_5_inner(
            &mut cursor,
            &mut pattern,
            &mut threadlist,
            skip1,
            skip2,
        )
        .expect("v5 inner with image_file should succeed");

        assert_eq!(pattern.extras.get("image_file").unwrap(), "MyImage.png");
        assert!(threadlist.is_empty());
    }

    /// Build a buffer for the version 5..8 header functions.
    /// These all follow the pattern: 4 pad + metadata + skip1 + [image_file str] + skip2 + skip_complex_items.
    fn build_v5_style_buffer(skip1: i64, skip2: i64) -> Vec<u8> {
        let mut buf = vec![0u8; 4]; // padding
        buf.extend_from_slice(&metadata_buffer()); // 5 bytes
        buf.extend_from_slice(&vec![0u8; skip1 as usize]); // skip1
        buf.push(0x00); // image_file = empty
        buf.extend_from_slice(&vec![0u8; skip2 as usize]); // skip2
        buf.extend_from_slice(&skip_complex_items_zero_buffer()); // 8 bytes
        buf
    }

    #[test]
    fn test_read_pes_header_version_5_success() {
        let buf = build_v5_style_buffer(24, 24);
        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_5(&mut cursor, &mut pattern, &mut threadlist)
            .expect("v5 header should succeed");
        assert!(threadlist.is_empty());
    }

    #[test]
    fn test_read_pes_header_version_6_success() {
        let buf = build_v5_style_buffer(36, 24);
        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_6(&mut cursor, &mut pattern, &mut threadlist)
            .expect("v6 header should succeed");
        assert!(threadlist.is_empty());
    }

    #[test]
    fn test_read_pes_header_version_7_success() {
        let buf = build_v5_style_buffer(36, 24);
        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_7(&mut cursor, &mut pattern, &mut threadlist)
            .expect("v7 header should succeed");
        assert!(threadlist.is_empty());
    }

    #[test]
    fn test_read_pes_header_version_8_success() {
        let buf = build_v5_style_buffer(38, 26);
        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_8(&mut cursor, &mut pattern, &mut threadlist)
            .expect("v8 header should succeed");
        assert!(threadlist.is_empty());
    }

    /// Build a buffer for versions 9 and 10.
    /// Pattern: 4 pad + metadata + 14 pad + [hoop_name str] + N pad + [image_file str] + 34 pad + skip_complex_items.
    fn build_v9_style_buffer(seek_before_image: i64) -> Vec<u8> {
        let mut buf = vec![0u8; 4]; // padding
        buf.extend_from_slice(&metadata_buffer()); // 5 bytes
        buf.extend_from_slice(&vec![0u8; 14]); // hoop_name seek
        buf.push(0x00); // hoop_name = empty
        buf.extend_from_slice(&vec![0u8; seek_before_image as usize]); // seek before image_file
        buf.push(0x00); // image_file = empty
        buf.extend_from_slice(&vec![0u8; 34]); // final seek
        buf.extend_from_slice(&skip_complex_items_zero_buffer()); // 8 bytes
        buf
    }

    #[test]
    fn test_read_pes_header_version_9_success() {
        let buf = build_v9_style_buffer(30);
        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_9(&mut cursor, &mut pattern, &mut threadlist)
            .expect("v9 header should succeed");
        assert!(!pattern.extras.contains_key("hoop_name"));
        assert!(!pattern.extras.contains_key("image_file"));
        assert!(threadlist.is_empty());
    }

    #[test]
    fn test_read_pes_header_version_10_success() {
        let buf = build_v9_style_buffer(38);
        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_10(&mut cursor, &mut pattern, &mut threadlist)
            .expect("v10 header should succeed");
        assert!(!pattern.extras.contains_key("hoop_name"));
        assert!(!pattern.extras.contains_key("image_file"));
        assert!(threadlist.is_empty());
    }

    #[test]
    fn test_read_pes_header_version_9_with_hoop_and_image() {
        let mut buf = vec![0u8; 4]; // padding
        buf.extend_from_slice(&metadata_buffer()); // 5 bytes
        buf.extend_from_slice(&vec![0u8; 14]); // hoop_name seek
        // hoop_name = "Big Hoop"
        buf.push(8);
        buf.extend_from_slice(b"Big Hoop");
        buf.extend_from_slice(&vec![0u8; 30]); // seek before image_file
        // image_file = "preview.png"
        buf.push(11);
        buf.extend_from_slice(b"preview.png");
        buf.extend_from_slice(&vec![0u8; 34]); // final seek
        buf.extend_from_slice(&skip_complex_items_zero_buffer()); // 8 bytes

        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_9(&mut cursor, &mut pattern, &mut threadlist)
            .expect("v9 header with hoop and image should succeed");

        assert_eq!(pattern.extras.get("hoop_name").unwrap(), "Big Hoop");
        assert_eq!(pattern.extras.get("image_file").unwrap(), "preview.png");
        assert!(threadlist.is_empty());
    }

    #[test]
    fn test_read_pes_header_version_10_with_hoop_and_image() {
        let mut buf = vec![0u8; 4]; // padding
        buf.extend_from_slice(&metadata_buffer()); // 5 bytes
        buf.extend_from_slice(&vec![0u8; 14]); // hoop_name seek
        // hoop_name = "Big Hoop"
        buf.push(8);
        buf.extend_from_slice(b"Big Hoop");
        buf.extend_from_slice(&vec![0u8; 38]); // seek before image_file (v10 uses 38)
        // image_file = "preview.png"
        buf.push(11);
        buf.extend_from_slice(b"preview.png");
        buf.extend_from_slice(&vec![0u8; 34]); // final seek
        buf.extend_from_slice(&skip_complex_items_zero_buffer()); // 8 bytes

        let mut cursor = Cursor::new(&buf[..]);
        let mut pattern = EmbPattern::new();
        let mut threadlist = Vec::new();

        read_pes_header_version_10(&mut cursor, &mut pattern, &mut threadlist)
            .expect("v10 header with hoop and image should succeed");

        assert_eq!(pattern.extras.get("hoop_name").unwrap(), "Big Hoop");
        assert_eq!(pattern.extras.get("image_file").unwrap(), "preview.png");
        assert!(threadlist.is_empty());
    }

    // ── Phase 8: PesReader struct (EmbroideryReader trait) ─────────────

    #[test]
    fn test_pes_reader_read_valid_data() {
        // Use the real Bean.pes file for integration
        let data = include_bytes!("../../tests/Test Designs/Bean.pes");
        let reader = PesReader;
        let pattern = reader.read(data).expect("PesReader should parse Bean.pes");
        assert!(pattern.count_stitches() > 0);
        assert!(pattern.count_threads() > 0);
    }

    #[test]
    fn test_pes_reader_read_invalid_data_returns_error() {
        let reader = PesReader;
        let result = reader.read(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pes_reader_read_truncated_data_returns_error() {
        let reader = PesReader;
        // A few bytes is not enough for a valid PES file
        let result = reader.read(&[0x23, 0x50, 0x45, 0x53]);
        assert!(result.is_err());
        // Ensure the error is a std::error::Error (downcast should work)
        let err = result.unwrap_err();
        let _ = err.to_string(); // must be printable
    }
}
