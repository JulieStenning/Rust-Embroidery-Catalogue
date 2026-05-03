use crate::readers::embroidery_reader::EmbroideryReader;

pub struct JefReader;

impl EmbroideryReader for JefReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        Ok(read_jef(data)?)
    }
}
use binrw::{BinRead, BinReaderExt};
use std::io::{Cursor, Seek, SeekFrom};

use crate::models::{EmbPattern, EmbThread, StitchType};

// ---------------------------------------------------------------------------
// JEF header – parsed with binrw.  Total size: 116 bytes before colour data.
// ---------------------------------------------------------------------------
#[derive(BinRead, Debug)]
#[br(little)]
struct JefHeader {
    /// Absolute file offset where stitch data begins.
    stitch_offset: u32,
    /// Unknown / reserved padding (20 bytes).
    _pad1: [u8; 20],
    /// Number of colour entries in the palette table that follows.
    count_colors: u32,
    /// Unknown / reserved padding (88 bytes).
    _pad2: [u8; 88],
}

// ---------------------------------------------------------------------------
// JEF built-in thread palette (index 0 = placeholder, 1..84 = real colours)
// ---------------------------------------------------------------------------

/// Helper macro to create a JEF `EmbThread` with less boilerplate.
macro_rules! jef_thread {
    ($color:expr, $desc:expr, $cat:expr) => {
        EmbThread {
            color: $color,
            description: Some($desc.into()),
            catalog_number: Some($cat.into()),
            details: None,
            brand: Some("Jef".into()),
            chart: Some("Jef".into()),
            weight: None,
        }
    };
}

/// Returns the built-in JEF thread palette (85 entries).
/// Index 0 is a placeholder – the Python reader treats colour index 0 as
/// "no thread", emitting a STOP instead of a COLOR_CHANGE.
fn get_jef_thread_set() -> Vec<EmbThread> {
    vec![
        // Index 0 – placeholder (never exposed as a real thread)
        EmbThread::new(0x000000),
        // 1
        jef_thread!(0x000000, "Black", "002"),
        jef_thread!(0xFFFFFF, "White", "001"),
        jef_thread!(0xFFFF17, "Yellow", "204"),
        jef_thread!(0xFF6600, "Orange", "203"),
        jef_thread!(0x2F5933, "Olive Green", "219"),
        jef_thread!(0x237336, "Green", "226"),
        jef_thread!(0x65C2C8, "Sky", "217"),
        jef_thread!(0xAB5A96, "Purple", "208"),
        jef_thread!(0xF669A0, "Pink", "201"),
        // 10
        jef_thread!(0xFF0000, "Red", "225"),
        jef_thread!(0xB1704E, "Brown", "214"),
        jef_thread!(0x0B2F84, "Blue", "207"),
        jef_thread!(0xE4C35D, "Gold", "003"),
        jef_thread!(0x481A05, "Dark Brown", "205"),
        jef_thread!(0xAC9CC7, "Pale Violet", "209"),
        jef_thread!(0xFCF294, "Pale Yellow", "210"),
        jef_thread!(0xF999B7, "Pale Pink", "211"),
        jef_thread!(0xFAB381, "Peach", "212"),
        jef_thread!(0xC9A480, "Beige", "213"),
        // 20
        jef_thread!(0x970533, "Wine Red", "215"),
        jef_thread!(0xA0B8CC, "Pale Sky", "216"),
        jef_thread!(0x7FC21C, "Yellow Green", "218"),
        jef_thread!(0xE5E5E5, "Silver Gray", "220"),
        jef_thread!(0x889B9B, "Gray", "221"),
        jef_thread!(0x98D6BD, "Pale Aqua", "227"),
        jef_thread!(0xB2E1E3, "Baby Blue", "228"),
        jef_thread!(0x368BA0, "Powder Blue", "229"),
        jef_thread!(0x4F83AB, "Bright Blue", "230"),
        jef_thread!(0x386A91, "Slate Blue", "231"),
        // 30
        jef_thread!(0x071650, "Navy Blue", "232"),
        jef_thread!(0xF999A2, "Salmon Pink", "233"),
        jef_thread!(0xF9676B, "Coral", "234"),
        jef_thread!(0xE3311F, "Burnt Orange", "235"),
        jef_thread!(0xE2A188, "Cinnamon", "236"),
        jef_thread!(0xB59474, "Umber", "237"),
        jef_thread!(0xE4CF99, "Blond", "238"),
        jef_thread!(0xFFCB00, "Sunflower", "239"),
        jef_thread!(0xE1ADD4, "Orchid Pink", "240"),
        jef_thread!(0xC3007E, "Peony Purple", "241"),
        // 40
        jef_thread!(0x80004B, "Burgundy", "242"),
        jef_thread!(0x540571, "Royal Purple", "243"),
        jef_thread!(0xB10525, "Cardinal Red", "244"),
        jef_thread!(0xCAE0C0, "Opal Green", "245"),
        jef_thread!(0x899856, "Moss Green", "246"),
        jef_thread!(0x5C941A, "Meadow Green", "247"),
        jef_thread!(0x003114, "Dark Green", "248"),
        jef_thread!(0x5DAE94, "Aquamarine", "249"),
        jef_thread!(0x4CBF8F, "Emerald Green", "250"),
        jef_thread!(0x007772, "Peacock Green", "251"),
        // 50
        jef_thread!(0x595B61, "Dark Gray", "252"),
        jef_thread!(0xFFFFF2, "Ivory White", "253"),
        jef_thread!(0xB15818, "Hazel", "254"),
        jef_thread!(0xCB8A07, "Toast", "255"),
        jef_thread!(0x986C80, "Salmon", "256"),
        jef_thread!(0x98692D, "Cocoa Brown", "257"),
        jef_thread!(0x4D3419, "Sienna", "258"),
        jef_thread!(0x4C330B, "Sepia", "259"),
        jef_thread!(0x33200A, "Dark Sepia", "260"),
        jef_thread!(0x523A97, "Violet Blue", "261"),
        // 60
        jef_thread!(0x0D217E, "Blue Ink", "262"),
        jef_thread!(0x1E77AC, "Sola Blue", "263"),
        jef_thread!(0xB2DD53, "Green Dust", "264"),
        jef_thread!(0xF33689, "Crimson", "265"),
        jef_thread!(0xDE649E, "Floral Pink", "266"),
        jef_thread!(0x984161, "Wine", "267"),
        jef_thread!(0x4C5612, "Olive Drab", "268"),
        jef_thread!(0x4C881F, "Meadow", "269"),
        jef_thread!(0xE4DE79, "Mustard", "270"),
        jef_thread!(0xCB8A1A, "Yellow Ocher", "271"),
        // 70
        jef_thread!(0xCBA21C, "Old Gold", "272"),
        jef_thread!(0xFF9805, "Honey Dew", "273"),
        jef_thread!(0xFCB257, "Tangerine", "274"),
        jef_thread!(0xFFE505, "Canary Yellow", "275"),
        jef_thread!(0xF0331F, "Vermilion", "202"),
        jef_thread!(0x1A842D, "Bright Green", "206"),
        jef_thread!(0x386CAE, "Ocean Blue", "222"),
        jef_thread!(0xE3C4B4, "Beige Gray", "223"),
        jef_thread!(0xE3AC81, "Bamboo", "224"),
        // 80
        jef_thread!(0x80ffff, "Unknown 080", ""),
        jef_thread!(0x80ff80, "Unknown 081", ""),
        jef_thread!(0xff80ff, "Unknown 082", ""),
        jef_thread!(0x80ffff, "Unknown 083", ""),
        jef_thread!(0x8080ff, "Unknown 084", ""),
    ]
}

// ---------------------------------------------------------------------------
// Low-level helpers
// ---------------------------------------------------------------------------

/// Interpret a byte as a signed 8-bit integer (mirrors Python `signed8`).
#[inline]
fn signed8(b: u8) -> i16 {
    if b > 127 {
        -256 + b as i16
    } else {
        b as i16
    }
}

/// Read exactly `n` bytes from the cursor into a Vec, or return an error on
/// EOF.
fn read_exact(cursor: &mut Cursor<&[u8]>, n: usize) -> Result<Vec<u8>, binrw::Error> {
    let pos = cursor.position();
    let data = cursor.get_ref();
    let end = pos as usize + n;
    if end > data.len() {
        return Err(binrw::Error::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "unexpected end of JEF stitch data",
        )));
    }
    let bytes = data[pos as usize..end].to_vec();
    cursor.set_position(end as u64);
    Ok(bytes)
}

// ---------------------------------------------------------------------------
// Public entry-point
// ---------------------------------------------------------------------------

/// Parse a JEF-format embroidery file from a byte buffer.
///
/// Returns an [`EmbPattern`] containing the stitches, threads, and metadata
/// read from the file.
///
/// # Errors
///
/// Returns a [`binrw::Error`] if the data is truncated or malformed.
pub fn read_jef(data: &[u8]) -> Result<EmbPattern, binrw::Error> {
    let mut cursor = Cursor::new(data);

    // --- 1. Parse the fixed-size header with binrw ---------------------------
    let header: JefHeader = cursor.read_le()?;

    let count_colors = header.count_colors as usize;
    if count_colors > 1024 {
        // Sanity check – no real JEF file has thousands of colours.
        return Err(binrw::Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "JEF count_colors unreasonably large",
        )));
    }

    // --- 2. Read the colour-index table (count_colors × i32 LE) -------------
    let mut raw_indices: Vec<i32> = Vec::with_capacity(count_colors);
    for _ in 0..count_colors {
        raw_indices.push(cursor.read_le::<i32>()?);
    }

    // --- 3. Build the thread list (matching Python behaviour) ----------------
    let jef_threads = get_jef_thread_set();
    let mut pattern = EmbPattern::new();

    for &raw_idx in &raw_indices {
        let idx = raw_idx.unsigned_abs() as usize;
        if idx == 0 {
            // Colour index 0 → push a dummy placeholder thread.  The stitch
            // parser will emit STOP instead of COLOR_CHANGE for this entry
            // (mirrors Python's `out.threadlist.append(None)` approach).
            pattern.threadlist.push(EmbThread::new(0x000000));
        } else {
            let thread = &jef_threads[idx % jef_threads.len()];
            pattern.threadlist.push(thread.clone());
        }
    }

    // --- 4. Seek to stitch data and parse stitches ---------------------------
    cursor.seek(SeekFrom::Start(header.stitch_offset as u64))?;

    let mut color_index: usize = 1;
    let data_len = data.len() as u64;

    loop {
        if cursor.position() >= data_len {
            break;
        }

        let bytes = match read_exact(&mut cursor, 2) {
            Ok(b) => b,
            Err(_) => break,
        };

        if bytes[0] != 0x80 {
            // ---- Regular stitch (relative delta) ----------------------------
            let dx = signed8(bytes[0]) as f32;
            let dy = -(signed8(bytes[1]) as f32);
            pattern.add_stitch_relative(StitchType::Stitch, dx, dy);
            continue;
        }

        // ---- Control command (first byte == 0x80) ---------------------------
        let ctrl = bytes[1];

        let extra = match read_exact(&mut cursor, 2) {
            Ok(b) => b,
            Err(_) => break,
        };
        let _dx = signed8(extra[0]) as f32;
        let _dy = -(signed8(extra[1]) as f32);

        match ctrl {
            0x02 => {
                // Jump – use the absolute position encoded in extra bytes
                pattern.add_stitch_relative(StitchType::Jump, _dx, _dy);
            }
            0x01 => {
                // Colour change (or STOP if the thread entry is a
                // placeholder for index 0).
                if raw_indices.get(color_index).is_none_or(|&r| r == 0) {
                    // Placeholder → emit STOP
                    pattern.add_stitch_absolute(StitchType::Stop, 0.0, 0.0);
                    // Remove the dummy thread entry (mirrors Python's
                    // `del out.threadlist[color_index]`).
                    if color_index < pattern.threadlist.len() {
                        pattern.threadlist.remove(color_index);
                    }
                } else {
                    pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
                    color_index += 1;
                }
            }
            0x10 => {
                // End of design
                break;
            }
            _ => {
                // Unknown control – stop parsing (matching Python behaviour)
                break;
            }
        }
    }

    // --- 5. Append END command -----------------------------------------------
    pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);

    // TODO: The Python reader applies `interpolate_trims` here based on
    //       trim_distance / trim_at settings.  This can be added later as
    //       a method on EmbPattern.

    Ok(pattern)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StitchType;
    
    #[test]
    fn test_read_real_jef_file() {
        // Path to the real JEF file for testing
        let path = r"D:\My Software Development\Rust-Embroidery-Catalogue\tests\testdata\Cake 3.jef";
        let data = std::fs::read(path).expect("Failed to read test JEF file");
        let pattern = read_jef(&data).expect("Failed to parse JEF file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        assert_eq!(pattern.threadlist.len(), 19, "Unexpected number of colours");
        let num_colour_changes = pattern.stitches.iter().filter(|s| s.stitch_type == StitchType::ColorChange).count();
        println!("Number of colour changes: {}", num_colour_changes);
        assert_eq!(num_colour_changes, 18, "Unexpected number of colour changes");
        for (i, stitch) in pattern.stitches.iter().take(5).enumerate() {
        }
        assert!(pattern.stitches.len() > 0, "No stitches found");
        assert_eq!(pattern.stitches.len(), 15141, "Unexpected stitch count");
    }
    #[test]
    fn test_read_jef_two_stitches() {
        // Build a minimal JEF file with no colour table and two regular stitches.
        //
        // Header layout (116 bytes):
        //   0..4   stitch_offset  u32 LE  →  116 (data starts right after header)
        //   4..24  _pad1          [u8;20]
        //   24..28 count_colors   u32 LE  →  0
        //   28..116 _pad2         [u8;88]
        //
        // Stitch 1: dx=5, dy=10  → bytes [0x05, 0xF6]
        //   signed8(0x05)=5,  signed8(0xF6)=246-256=-10, dy = -(-10) = 10
        // Stitch 2: dx=-3, dy=7 → bytes [0xFD, 0xF9]
        //   signed8(0xFD)=253-256=-3, signed8(0xF9)=249-256=-7, dy = -(-7) = 7
        //
        // Expected cumulative coordinates from origin (0,0):
        //   stitch_1 → (5, 10)
        //   stitch_2 → (2, 17)
        //   + automatic END appended by read_jef

        let stitch_offset: u32 = 116;
        let mut data = Vec::with_capacity(120);

        // --- header ---
        data.extend_from_slice(&stitch_offset.to_le_bytes()); // stitch_offset
        data.extend_from_slice(&[0u8; 20]); // _pad1
        data.extend_from_slice(&0u32.to_le_bytes()); // count_colors = 0
        data.extend_from_slice(&[0u8; 88]); // _pad2

        // --- stitch data ---
        data.push(0x05); // stitch 1 dx
        data.push(0xF6); // stitch 1 dy
        data.push(0xFD); // stitch 2 dx
        data.push(0xF9); // stitch 2 dy

        let pattern = read_jef(&data).expect("should parse a valid minimal JEF");

        // Two regular Stitch entries + the mandatory End
        assert_eq!(
            pattern.count_stitch_commands(StitchType::Stitch),
            2,
            "expected exactly 2 regular stitches"
        );

        let stitches: Vec<_> = pattern
            .stitches
            .iter()
            .filter(|s| s.stitch_type == StitchType::Stitch)
            .collect();

        assert_eq!(stitches.len(), 2);
        assert_eq!(stitches[0].x, 5.0);
        assert_eq!(stitches[0].y, 10.0);
        assert_eq!(stitches[1].x, 2.0);
        assert_eq!(stitches[1].y, 17.0);

        // The parser always appends End
        assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);

        // No thread entries (count_colors was 0)
        assert!(pattern.threadlist.is_empty());
    }
}
