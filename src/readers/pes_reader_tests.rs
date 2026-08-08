// Tests for the PES reader.
//
// This module was split out of pes_reader.rs so the reader file can stay
// focused on production parsing logic. It is included via a #[path]
// declaration in a #[cfg(test)] mod tests; module, so it retains full
// access to the private items in the parent module through use super::*;.

use super::*;

// â”€â”€ Phase 1: Low-level helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
    assert_eq!(signed7(127), -1); // -128 + 127 = -1
    assert_eq!(signed7(255), 127); // -128 + 255 = 127 (not -1 since 255 â‰  127)
    assert_eq!(signed7(128), 0); // -128 + 128 = 0
    assert_eq!(signed7(129), 1); // -128 + 129 = 1
    assert_eq!(signed7(64), -64); // -128 + 64 = -64
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

// â”€â”€ Phase 2: PEC thread palette & colour processing â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
        assert!(
            thread.description.is_some(),
            "Index {i} missing description"
        );
        assert!(
            thread.catalog_number.is_some(),
            "Index {i} missing catalog_number"
        );
        assert_eq!(
            thread.brand.as_deref(),
            Some("Brother"),
            "Index {i} wrong brand"
        );
        assert_eq!(
            thread.chart.as_deref(),
            Some("Brother"),
            "Index {i} wrong chart"
        );
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

// â”€â”€ Phase 3: PEC stitch edge cases â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
        0x05, // Y short = +5
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
        0x05, // Y byte2 = 0x05 => Y = 0x9105 & 0xFFF = 0x105 = 261
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
        0x07, // Y short = +7
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

// â”€â”€ Phase 4: PES metadata & header helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
    assert_eq!(threadlist[0].catalog_number.as_deref(), Some("ABC"));
    assert_eq!(threadlist[0].description.as_deref(), Some("Test Thread"));
    assert_eq!(threadlist[0].brand.as_deref(), Some("TestBrand"));
    assert_eq!(threadlist[0].chart.as_deref(), Some("TestChart"));
}

#[test]
fn test_skip_complex_items_early_return_fills() {
    // Non-zero programmable fills => returns true immediately
    let data = [
        0x01, 0x00, // count programmable fills = 1
        0x00, 0x00, // count motifs = 0
        0x00, 0x00, // count feather patterns = 0
        0x00, 0x00,
    ]; // count threads = 0
    let mut cursor = Cursor::new(&data[..]);
    let mut threadlist = Vec::new();
    let result = skip_complex_items(&mut cursor, &mut threadlist).expect("skip should succeed");
    assert!(result); // early return true
    assert!(threadlist.is_empty());
}

#[test]
fn test_skip_complex_items_early_return_motifs() {
    // Only motifs non-zero => returns true
    let data = [
        0x00, 0x00, // programmable fills = 0
        0x02, 0x00, // motifs = 2
        0x00, 0x00, // feather patterns = 0
        0x00, 0x00,
    ]; // threads = 0
    let mut cursor = Cursor::new(&data[..]);
    let mut threadlist = Vec::new();
    let result = skip_complex_items(&mut cursor, &mut threadlist).expect("skip should succeed");
    assert!(result);
}

#[test]
fn test_skip_complex_items_early_return_feather() {
    let data = [
        0x00, 0x00, // programmable fills = 0
        0x00, 0x00, // motifs = 0
        0x01, 0x00, // feather patterns = 1
        0x00, 0x00,
    ]; // threads = 0
    let mut cursor = Cursor::new(&data[..]);
    let mut threadlist = Vec::new();
    let result = skip_complex_items(&mut cursor, &mut threadlist).expect("skip should succeed");
    assert!(result);
}

#[test]
fn test_skip_complex_items_zero_threads() {
    let data = [
        0x00, 0x00, // programmable fills = 0
        0x00, 0x00, // motifs = 0
        0x00, 0x00, // feather patterns = 0
        0x00, 0x00,
    ]; // threads = 0
    let mut cursor = Cursor::new(&data[..]);
    let mut threadlist = Vec::new();
    let result = skip_complex_items(&mut cursor, &mut threadlist).expect("skip should succeed");
    assert!(!result); // no early return
    assert!(threadlist.is_empty());
}

// â”€â”€ Phase 5: Integration tests with real PES files â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_read_bean_pes() {
    let data = include_bytes!("../../tests/Test Designs/Bean.pes");
    let pattern = read_pes(data).expect("Bean.pes should parse successfully");
    assert!(
        pattern.count_stitches() > 0,
        "Bean.pes should have stitches"
    );
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
    assert!(
        pattern.count_stitches() > 0,
        "Cake 3.pes should have stitches"
    );
    assert!(
        pattern.count_threads() > 0,
        "Cake 3.pes should have threads"
    );
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
    assert!(
        pattern.count_stitches() > 0,
        "Flower.pes should have stitches"
    );
    assert!(
        pattern.count_threads() > 0,
        "Flower.pes should have threads"
    );
}

#[test]
fn test_read_rose_bouquet_pes() {
    let data = include_bytes!("../../tests/Test Designs/rose_bouquet.pes");
    let pattern = read_pes(data).expect("rose_bouquet.pes should parse successfully");
    assert!(
        pattern.count_stitches() > 0,
        "rose_bouquet.pes should have stitches"
    );
    assert!(
        pattern.count_threads() > 0,
        "rose_bouquet.pes should have threads"
    );
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

// â”€â”€ Phase 6: interpolate_duplicate_color_as_stop edge cases â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€ Phase 7: PES version-specific header functions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Helper: build a buffer for read_pes_metadata() containing 5 empty strings.
fn metadata_buffer() -> Vec<u8> {
    vec![0x00; 5] // 5 zero-length strings
}

/// Helper: build a buffer for skip_complex_items() where all counts are 0.
fn skip_complex_items_zero_buffer() -> Vec<u8> {
    vec![0x00; 8] // 4Ã— u16-le = 8 bytes
}

#[test]
fn test_read_pes_header_version_4_parses_metadata() {
    // 4 bytes padding + metadata (5 empty strings)
    let mut buf = vec![0u8; 4];
    buf.extend_from_slice(&metadata_buffer());
    let mut cursor = Cursor::new(&buf[..]);
    let mut pattern = EmbPattern::new();

    read_pes_header_version_4(&mut cursor, &mut pattern).expect("v4 header should succeed");

    // Metadata was called with empty strings â€” no extras inserted
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

    read_pes_header_version_5_inner(&mut cursor, &mut pattern, &mut threadlist, skip1, skip2)
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

    read_pes_header_version_5_inner(&mut cursor, &mut pattern, &mut threadlist, skip1, skip2)
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

// â”€â”€ Phase 8: PesReader struct (EmbroideryReader trait) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
