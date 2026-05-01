use std::io::{Cursor, Seek, SeekFrom};

use crate::models::{EmbPattern, EmbThread, StitchType};

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
            pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
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
            let _val3 = read_u8(cursor)?; // consume the 3rd byte for X
        } else {
            x = signed7(val1);
        }

        // Decode Y — need to read the next byte (or val2 if X wasn't long)
        let y_byte1 = if val1 & FLAG_LONG != 0 {
            // We already consumed an extra byte above, read the next
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
            pattern.add_stitch_absolute(StitchType::Trim, 0.0, 0.0);
            pattern.add_stitch_relative(StitchType::Jump, x as f32, y as f32);
        } else {
            pattern.add_stitch_relative(StitchType::Stitch, x as f32, y as f32);
        }
    }

    pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);
    Ok(())
}

// ===========================================================================
// PEC colour & graphics reading
// ===========================================================================

/// Process PEC colour bytes using the built-in thread palette.
fn process_pec_colors(
    color_bytes: &[u8],
    pattern: &mut EmbPattern,
    values: &mut Vec<EmbThread>,
) {
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
    let mut thread_map: std::collections::HashMap<usize, EmbThread> = std::collections::HashMap::new();

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
    let stitch_block_end = read_u24_be(cursor)? as i64 - 5 + cursor.position() as i64;

    // Skip 0x0B bytes (3 bytes + 4×2-byte shorts)
    cursor.seek(SeekFrom::Current(0x0B))?;

    // Read PEC stitches
    read_pec_stitches(cursor, pattern)?;

    // Seek to stitch block end
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
            format!("{};{};{}", graphic_hex, pec_graphic_byte_stride, thread_color),
        );
    }

    Ok(())
}

// ===========================================================================
// PES header reading (version-specific)
// ===========================================================================

fn read_pes_metadata(cursor: &mut Cursor<&[u8]>, pattern: &mut EmbPattern) -> Result<(), binrw::Error> {
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

fn read_pes_thread(cursor: &mut Cursor<&[u8]>, threadlist: &mut Vec<EmbThread>) -> Result<(), binrw::Error> {
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

fn skip_complex_items(cursor: &mut Cursor<&[u8]>, threadlist: &mut Vec<EmbThread>) -> Result<bool, binrw::Error> {
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
        // Direct PEC format
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
            StitchType::ColorChange
            | StitchType::ColorBreak
            | StitchType::NeedleSet => {
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

    #[test]
    fn test_signed7() {
        assert_eq!(signed7(0), 0);
        assert_eq!(signed7(10), 10);
        assert_eq!(signed7(63), 63);
        assert_eq!(signed7(200), 200 - 128);
    }

    #[test]
    fn test_signed12() {
        assert_eq!(signed12(0), 0);
        assert_eq!(signed12(100), 100);
        assert_eq!(signed12(0x7FF), 0x7FF);
        assert_eq!(signed12(0xFFF), -1);
    }

    #[test]
    fn test_interpolate_duplicate_color_as_stop() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_thread(EmbThread::new(0xFF0000)); // duplicate
        pattern.add_thread(EmbThread::new(0x00FF00));

        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 1.0, 1.0);
        pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
        pattern.add_stitch_relative(StitchType::Stitch, 2.0, 2.0);

        interpolate_duplicate_color_as_stop(&mut pattern);

        // Second ColorChange (position 2, which precedes the duplicate
        // Red→Red transition) becomes Stop; first ColorChange stays.
        assert_eq!(pattern.stitches[0].stitch_type, StitchType::ColorChange);
        assert_eq!(pattern.stitches[2].stitch_type, StitchType::Stop);
        // Duplicate thread (index 1) removed, leaving Red + Green
        assert_eq!(pattern.threadlist.len(), 2);
        assert_eq!(pattern.threadlist[0].color, 0xFF0000);
        assert_eq!(pattern.threadlist[1].color, 0x00FF00);
    }
}