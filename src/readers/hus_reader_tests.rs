// Tests for the HUS reader.
//
// This module was split out of hus_reader.rs so the reader file can stay
// focused on production parsing logic. It is included via a #[path]
// declaration in a #[cfg(test)] mod tests; module, so it retains full
// access to the private items in the parent module through use super::*;.

use std::fs;

use super::*;

// â”€â”€ Integration tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn read_hus_fixture_produces_stitches_threads_and_end() {
    let path = "tests/Test Designs/Bean.hus";
    let data = fs::read(path).expect("expected HUS fixture file");
    let pattern = HusReader
        .read(&data)
        .expect("expected HUS parsing to succeed");

    assert!(pattern.stitches.len() > 10, "expected parsed stitches");
    assert!(
        !pattern.threadlist.is_empty(),
        "expected parsed thread entries"
    );
    assert_eq!(
        pattern.stitches.last().map(|s| s.stitch_type),
        Some(StitchType::End),
        "expected terminal End command"
    );
}

#[test]
fn read_hus_old_cake_fixture_is_not_stubbed_zeroes() {
    let path = "tests/Test Designs/Cake 3.hus";
    let data = fs::read(path).expect("expected old HUS fixture file");
    let pattern = read_hus(&data).expect("expected HUS parser to decode fixture");

    assert!(pattern.stitches.len() > 1000);
    let any_non_zero = pattern
        .stitches
        .iter()
        .any(|stitch| stitch.x != 0.0 || stitch.y != 0.0);
    assert!(any_non_zero, "expected decoded coordinates, not all zeros");
}

#[test]
fn read_hus_rectangle_fixture_succeeds() {
    let path = "tests/Test Designs/rectangle.hus";
    let data = fs::read(path).expect("expected HUS fixture file");
    let pattern = read_hus(&data).expect("expected HUS parser to decode fixture");

    assert!(pattern.stitches.len() > 5, "expected parsed stitches");
    assert!(
        !pattern.threadlist.is_empty(),
        "expected parsed thread entries"
    );
    assert_eq!(
        pattern.stitches.last().map(|s| s.stitch_type),
        Some(StitchType::End),
        "expected terminal End command"
    );
}

#[test]
fn read_hus_rejects_truncated_data() {
    let result = read_hus(&[0u8; 10]);
    assert!(result.is_err(), "expected error for truncated data");
}

#[test]
fn read_hus_rejects_invalid_offsets() {
    // Build a minimal header with command_offset > x_offset
    let mut buf = vec![0u8; 512];
    // Magic code (4 bytes)
    buf[0..4].copy_from_slice(&0x0000_0080u32.to_le_bytes());
    // number_of_stitches (4 bytes)
    buf[4..8].copy_from_slice(&1u32.to_le_bytes());
    // number_of_colors (4 bytes)
    buf[8..12].copy_from_slice(&0u32.to_le_bytes());
    // extend_pos_x (2) + extend_pos_y (2) + extend_neg_x (2) + extend_neg_y (2) = 8 bytes
    // command_offset (4 bytes) at offset 20
    buf[20..24].copy_from_slice(&100u32.to_le_bytes());
    // x_offset (4 bytes) at offset 24 â€” set lower than command_offset to trigger error
    buf[24..28].copy_from_slice(&50u32.to_le_bytes());
    // y_offset (4 bytes) at offset 28
    buf[28..32].copy_from_slice(&60u32.to_le_bytes());

    let result = read_hus(&buf);
    assert!(result.is_err(), "expected error for invalid offsets");
    assert!(
        result
            .unwrap_err()
            .contains("invalid HUS compressed section offsets"),
        "expected offset-related error message"
    );
}

#[test]
fn read_hus_zero_colors_produces_empty_threadlist() {
    // Minimal 42-byte header, zero stitches, zero colors.
    // All compressed-section offsets point at the end of the header,
    // so all three compressed chunks are empty.
    let mut buf = vec![0u8; 42];
    // number_of_stitches (4 bytes at offset 4) = 0 (already zero)
    // number_of_colors (4 bytes at offset 8) = 0 (already zero)
    // command_offset (4 bytes at offset 20)
    buf[20..24].copy_from_slice(&42u32.to_le_bytes());
    // x_offset (4 bytes at offset 24)
    buf[24..28].copy_from_slice(&42u32.to_le_bytes());
    // y_offset (4 bytes at offset 28)
    buf[28..32].copy_from_slice(&42u32.to_le_bytes());

    let pattern = read_hus(&buf).expect("zero-stitch HUS should parse");

    assert!(
        pattern.threadlist.is_empty(),
        "zero colors should produce an empty threadlist"
    );
    assert_eq!(
        pattern.count_stitch_commands(StitchType::End),
        1,
        "should still append the terminal End marker"
    );
}

#[test]
fn read_hus_color_index_out_of_range_defaults_to_black() {
    // 42-byte header + 1 colour index (2 bytes) = 44 bytes total.
    let mut buf = vec![0u8; 44];
    // number_of_stitches = 0
    buf[4..8].copy_from_slice(&0u32.to_le_bytes());
    // number_of_colors = 1
    buf[8..12].copy_from_slice(&1u32.to_le_bytes());
    // command_offset = x_offset = y_offset = 44 (no compressed data)
    buf[20..24].copy_from_slice(&44u32.to_le_bytes());
    buf[24..28].copy_from_slice(&44u32.to_le_bytes());
    buf[28..32].copy_from_slice(&44u32.to_le_bytes());
    // Colour index 50 is beyond the 29-entry built-in palette.
    buf[42..44].copy_from_slice(&50u16.to_le_bytes());

    let pattern = read_hus(&buf).expect("HUS with out-of-range colour index should parse");

    assert_eq!(pattern.threadlist.len(), 1);
    assert_eq!(
        pattern.threadlist[0].color, 0x000000,
        "out-of-range palette index should fall back to black"
    );
}

#[test]
fn read_hus_declared_stitches_exceed_available_data() {
    // number_of_stitches claims 100, but there is no compressed data at all.
    // The min() guard must clamp the parsed count to what is actually
    // available instead of indexing out of bounds.
    let mut buf = vec![0u8; 42];
    // number_of_stitches = 100
    buf[4..8].copy_from_slice(&100u32.to_le_bytes());
    // number_of_colors = 0
    buf[8..12].copy_from_slice(&0u32.to_le_bytes());
    // All offsets point to the end of the header â†’ empty compressed chunks.
    buf[20..24].copy_from_slice(&42u32.to_le_bytes());
    buf[24..28].copy_from_slice(&42u32.to_le_bytes());
    buf[28..32].copy_from_slice(&42u32.to_le_bytes());

    let pattern = read_hus(&buf).expect("HUS with over-declared stitch count should parse");

    // No stitch data available â†’ zero stitches parsed, no out-of-bounds panic.
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 0);
    assert_eq!(
        pattern.count_stitch_commands(StitchType::End),
        1,
        "should still append the terminal End marker"
    );
}

// â”€â”€ apply_hus_command tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn hus_jump_command_maps_to_jump() {
    let mut pattern = EmbPattern::new();

    let keep_parsing = apply_hus_command(&mut pattern, 0x81, 5.0, -7.0);

    assert!(keep_parsing);
    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 0);
    assert_eq!(pattern.stitches[0].x, 5.0);
    assert_eq!(pattern.stitches[0].y, -7.0);
}

#[test]
fn hus_stitch_command_maps_to_stitch() {
    let mut pattern = EmbPattern::new();
    let keep_parsing = apply_hus_command(&mut pattern, 0x80, 10.0, -3.5);
    assert!(keep_parsing);
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 1);
    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 0);
    assert_eq!(pattern.stitches[0].x, 10.0);
    assert_eq!(pattern.stitches[0].y, -3.5);
}

#[test]
fn hus_color_change_command_maps_to_color_change() {
    let mut pattern = EmbPattern::new();
    let keep_parsing = apply_hus_command(&mut pattern, 0x84, 0.0, 0.0);
    assert!(keep_parsing);
    assert_eq!(pattern.count_stitch_commands(StitchType::ColorChange), 1);
}

#[test]
fn hus_trim_command_with_nonzero_delta_adds_jump_then_trim() {
    let mut pattern = EmbPattern::new();
    // 0x88: Trim â€” adds a Jump if dx/dy != 0, then always adds Trim
    let keep_parsing = apply_hus_command(&mut pattern, 0x88, 5.0, 3.0);
    assert!(keep_parsing);
    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
    assert_eq!(pattern.count_stitch_commands(StitchType::Trim), 1);
    // Last stitch should be Trim at (0,0) relative to the last absolute position
    assert_eq!(
        pattern.stitches.last().unwrap().stitch_type,
        StitchType::Trim
    );
}

#[test]
fn hus_trim_command_with_zero_delta_adds_only_trim() {
    let mut pattern = EmbPattern::new();
    let keep_parsing = apply_hus_command(&mut pattern, 0x88, 0.0, 0.0);
    assert!(keep_parsing);
    assert_eq!(
        pattern.count_stitch_commands(StitchType::Jump),
        0,
        "no Jump for zero delta"
    );
    assert_eq!(pattern.count_stitch_commands(StitchType::Trim), 1);
}

#[test]
fn hus_end_command_returns_false() {
    let mut pattern = EmbPattern::new();
    let keep_parsing = apply_hus_command(&mut pattern, 0x90, 0.0, 0.0);
    assert!(!keep_parsing, "0x90 should signal end of commands");
}

#[test]
fn hus_unknown_command_returns_true() {
    let mut pattern = EmbPattern::new();
    let keep_parsing = apply_hus_command(&mut pattern, 0xFF, 1.0, 2.0);
    assert!(keep_parsing, "unknown command should not halt parsing");
    // No stitch should have been added
    assert!(pattern.stitches.is_empty(), "no stitch for unknown command");
}

#[test]
fn hus_stitch_i8_boundaries() {
    // Verify coordinate accumulation at the i8 extremes.
    // In HUS decoding, x is used directly and y is negated.
    let mut pattern = EmbPattern::new();

    // Stitch 1: (+127, +127)
    assert!(apply_hus_command(&mut pattern, 0x80, 127.0, 127.0));
    // Stitch 2: (-128, -128)
    assert!(apply_hus_command(&mut pattern, 0x80, -128.0, -128.0));

    let stitches: Vec<_> = pattern
        .stitches
        .iter()
        .filter(|s| s.stitch_type == StitchType::Stitch)
        .collect();
    assert_eq!(stitches.len(), 2);
    assert_eq!(stitches[0].x, 127.0);
    assert_eq!(stitches[0].y, 127.0);
    // Accumulated position after both stitches: 127 + (-128) = -1
    assert_eq!(stitches[1].x, -1.0);
    assert_eq!(stitches[1].y, -1.0);
}

// â”€â”€ parse_color tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn parse_color_with_hash() {
    assert_eq!(parse_color("#ff0000"), 0x00ff_0000);
    assert_eq!(parse_color("#000000"), 0);
    assert_eq!(parse_color("#ffffff"), 0x00ff_ffff);
}

#[test]
fn parse_color_without_hash() {
    assert_eq!(parse_color("00ff00"), 0x0000_ff00);
}

#[test]
fn parse_color_empty_string_falls_back_to_zero() {
    assert_eq!(parse_color(""), 0);
    assert_eq!(parse_color("#"), 0);
}

// â”€â”€ hus_thread_set tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn hus_thread_set_has_expected_entries() {
    let set = hus_thread_set();
    assert_eq!(set.len(), 29, "expected 29 Hus thread palette entries");

    // Spot-check known entries
    let black = &set[0];
    assert_eq!(black.color & 0x00FF_FFFF, 0x000000);
    assert_eq!(black.description.as_deref(), Some("Black"));
    assert_eq!(black.catalog_number.as_deref(), Some("026"));

    let white = &set[12];
    assert_eq!(white.color & 0x00FF_FFFF, 0xFFFFFF);
    assert_eq!(white.description.as_deref(), Some("White"));
    assert_eq!(white.catalog_number.as_deref(), Some("022"));

    let red = &set[3];
    assert_eq!(red.color & 0x00FF_FFFF, 0xFF0000);
    assert_eq!(red.description.as_deref(), Some("Red"));

    let thread = &set[12];
    assert_eq!(thread.brand.as_deref(), Some("Hus"));
    assert_eq!(thread.chart.as_deref(), Some("Hus"));
    assert!(thread.weight.is_none());
}

// â”€â”€ Huffman tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn huffman_with_default_lookup_returns_default() {
    let h = Huffman::with_default(42);
    let (value, len) = h.lookup(0xFFFF).expect("default lookup should succeed");
    assert_eq!(value, 42);
    assert_eq!(len, 0);
}

#[test]
fn huffman_with_lengths_rejects_empty() {
    let result = Huffman::with_lengths(vec![]);
    assert!(result.is_err(), "empty lengths should be rejected");
}

#[test]
fn huffman_with_lengths_rejects_oversized_width() {
    let result = Huffman::with_lengths(vec![17]);
    assert!(result.is_err(), "width > 16 should be rejected");
}

#[test]
fn huffman_with_lengths_rejects_zero_width() {
    let result = Huffman::with_lengths(vec![0]);
    assert!(
        result.is_err(),
        "zero width (max length = 0) should be rejected"
    );
}

#[test]
fn huffman_with_two_lengths_builds_table_and_lookup_works() {
    // Two symbols both at bit-length 1 fills 2^1 = 2 table entries.
    let h = Huffman::with_lengths(vec![1, 1]).expect("valid lengths");
    // table_width = 1, so idx = (lookup >> 15) â€” top bit of 16-bit value.
    // 0x8000 has bit 15 set -> idx = 1 -> symbol 1
    let (value, len) = h.lookup(0x8000).expect("lookup for high bit");
    assert_eq!(value, 1, "second symbol index");
    assert_eq!(len, 1, "bit length");
    // 0x0000 has bit 15 clear -> idx = 0 -> symbol 0
    let (value, len) = h.lookup(0x0000).expect("lookup for low bit");
    assert_eq!(value, 0, "first symbol index");
    assert_eq!(len, 1, "bit length");
}

#[test]
fn huffman_max_table_width() {
    // 16-bit table width is the maximum supported by the builder.
    let h = Huffman::with_lengths(vec![16]).expect("16-width should be accepted");
    assert_eq!(h.table_width, 16);

    // A single symbol at bit length 16 produces a 1-entry table because
    // the lookup shift is 0 (16 - table_width), so index 0 is the only
    // valid slot.
    let table = h.table.as_ref().expect("table should be built");
    assert_eq!(table.len(), 1);

    // Lookup at index 0 returns symbol 0 with length 16.
    let (value, len) = h.lookup(0x0000).expect("lookup at index 0 should succeed");
    assert_eq!(value, 0);
    assert_eq!(len, 16);

    // An incomplete Huffman table (Kraft sum < 1) only fills the slots it
    // has; higher indices are out of bounds and must fail gracefully with
    // a checked-index error rather than panicking.
    assert!(
        h.lookup(0xFFFF).is_err(),
        "lookup beyond table bounds should error safely"
    );
}

#[test]
fn huffman_lookup_on_missing_table_returns_default() {
    let h = Huffman {
        default_value: 99,
        lengths: None,
        table: None,
        table_width: 0,
    };
    let (value, len) = h.lookup(0).expect("no-table lookup should return default");
    assert_eq!(value, 99);
    assert_eq!(len, 0);
}

#[test]
fn huffman_with_all_zero_lengths() {
    // All entries at bit-length 0 â€” no symbols get placed into the table.
    let result = Huffman::with_lengths(vec![0, 0, 0]);
    // The table loop runs from bit_length=1..table_width, but table_width=0
    // so the loop doesn't execute â†’ table is empty â†’ Err("failed to build...")
    assert!(
        result.is_err(),
        "all-zero lengths should fail to build a table"
    );
}

// â”€â”€ read_u16_le / read_u32_le EOF tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn read_u16_le_eof_on_empty_cursor() {
    let mut cursor = Cursor::new(&[][..]);
    let result = read_u16_le(&mut cursor);
    assert!(result.is_err(), "expected EOF for empty cursor");
}

#[test]
fn read_u16_le_eof_on_short_cursor() {
    let mut cursor = Cursor::new(&[0x01u8][..]);
    let result = read_u16_le(&mut cursor);
    assert!(result.is_err(), "expected EOF for single-byte cursor");
}

#[test]
fn read_u16_le_single_valid_read() {
    let mut cursor = Cursor::new(&[0x34, 0x12][..]);
    let value = read_u16_le(&mut cursor).expect("should read valid u16");
    assert_eq!(value, 0x1234);
}

#[test]
fn read_u32_le_eof_on_empty_cursor() {
    let mut cursor = Cursor::new(&[][..]);
    let result = read_u32_le(&mut cursor);
    assert!(result.is_err(), "expected EOF for empty cursor");
}

#[test]
fn read_u32_le_eof_on_short_cursor() {
    let mut cursor = Cursor::new(&[0x01, 0x02, 0x03][..]);
    let result = read_u32_le(&mut cursor);
    assert!(result.is_err(), "expected EOF for 3-byte cursor");
}

#[test]
fn read_u32_le_single_valid_read() {
    let mut cursor = Cursor::new(&[0xEF, 0xBE, 0xAD, 0xDE][..]);
    let value = read_u32_le(&mut cursor).expect("should read valid u32");
    assert_eq!(value, 0xDEAD_BEEF);
}

// â”€â”€ EmbCompress bit-stream primitive tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn embcompress_get_bits_single_byte() {
    // Byte: 0b1010_0101
    // get_bits reads from bit position 0, MSB-first.
    // get_bits(0, 4): reads byte, unused_bits = (8-4)%8 = 4, value = 0xA5>>4 = 0b1010
    let comp = EmbCompress::new(vec![0b1010_0101]);
    assert_eq!(comp.get_bits(0, 4), 0b1010);
    // get_bits(4, 4): reads byte, unused_bits = (8-8)%8 = 0, value = 0xA5 = 0b0101
    assert_eq!(comp.get_bits(4, 4), 0b0101);
}

#[test]
fn embcompress_get_bits_crosses_byte_boundary() {
    // Bytes: 0xF0, 0x0F -> combined as u32 (big-endian accumulation): 0xF0_0F
    // get_bits(4, 8): reads 2 bytes, unused_bits = (8-12)%8 = 4, value = 0xF0_0F>>4 = 0x0F00
    // mask = 0xFF -> 0x00
    let comp = EmbCompress::new(vec![0b1111_0000, 0b0000_1111]);
    assert_eq!(comp.get_bits(4, 8), 0b0000_0000);
    // get_bits(0, 16): accumulated = 0xF00F, unused_bits = 0, mask = 0xFFFF -> 0xF00F
    assert_eq!(comp.get_bits(0, 16), 0xF00F);
}

#[test]
fn embcompress_get_bits_beyond_buffer_returns_zero_padded() {
    // Byte: 0xFF, missing second byte -> 0 in the shift accumulation.
    // get_bits(0, 16): byte0=0xFF, byte1 missing => 0,
    // accumulated: (0xFF << 8) | 0 = 0xFF00, unused_bits = 0, mask = 0xFFFF
    let comp = EmbCompress::new(vec![0xFF]);
    assert_eq!(comp.get_bits(0, 16), 0xFF00);
}

#[test]
fn embcompress_peek_slide_pop_sequence() {
    // Byte: 0b1100_1010
    // Bit positions from MSB: 0:1, 1:1, 2:0, 3:0, 4:1, 5:0, 6:1, 7:0
    let mut comp = EmbCompress::new(vec![0b1100_1010]);
    assert_eq!(comp.bit_position, 0);

    // peek(4) from pos 0 -> bits 0..=3 = 0b1100
    assert_eq!(comp.peek(4), 0b1100);
    assert_eq!(comp.bit_position, 0, "peek should not advance");

    // slide(2) -> position 2
    comp.slide(2);
    assert_eq!(comp.bit_position, 2);

    // peek(2) from pos 2 -> bits 2..=3 = 0b00
    assert_eq!(comp.peek(2), 0b00);
    assert_eq!(comp.bit_position, 2);

    // pop(2) -> 0b00, advances to position 4
    assert_eq!(comp.pop(2), 0b00);
    assert_eq!(comp.bit_position, 4);

    // pop(4) from pos 4 -> bits 4..=7 = 0b1010
    assert_eq!(comp.pop(4), 0b1010);
    assert_eq!(comp.bit_position, 8);
}

#[test]
fn embcompress_read_variable_length_short() {
    // Values 0..6 are encoded in 3 bits (MSB); ensure they come back directly.
    for expected in 0..7u32 {
        let byte = (expected as u8) << 5; // top 3 bits of first byte
        let mut comp = EmbCompress::new(vec![byte]);
        let result = comp.read_variable_length();
        assert_eq!(result, expected as usize, "direct 3-bit value {expected}");
    }
}

#[test]
fn embcompress_read_variable_length_longer() {
    // m = 7 triggers the extension loop: each pop(1)=1 extends m, pop(1)=0 stops.
    // Bits (MSB first): 111 (pop 3 -> m=7), 1 (extend), 1 (extend), 0 (stop)
    // Pattern in byte: 1111_1000 = 0xF8
    // After reads: m = 7 + 1 + 1 = 9
    let data = vec![0b1111_1000];
    let mut comp = EmbCompress::new(data);
    let result = comp.read_variable_length();
    assert_eq!(result, 9);
}

// â”€â”€ EmbCompress get_position tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn embcompress_get_position_returns_zero_for_distance_zero() {
    // Set up a distance_huffman that gives (0, 1) â€” lookup returns 0.
    let dist_huff = Huffman::with_lengths(vec![1]).expect("valid distance huffman");
    let mut comp = EmbCompress::new(vec![0x00]);
    comp.block_elements = 1; // prevent load_block from being called
    comp.distance_huffman = Some(dist_huff);
    // The character_huffman must also be set for get_token to work,
    // but we call get_position directly so it's fine to leave None.
    let pos = comp.get_position().expect("get_position should succeed");
    assert_eq!(pos, 0, "distance 0 maps to position 0");
}

#[test]
fn embcompress_get_position_missing_distance_huffman_errors() {
    let mut comp = EmbCompress::new(vec![]);
    comp.block_elements = 1;
    let result = comp.get_position();
    assert!(
        result.is_err(),
        "expected error when distance_huffman is missing"
    );
}

#[test]
fn embcompress_get_position_nonzero_distance() {
    // Build a distance_huffman with 2 symbols at bit-length 1
    // so the table has 2 entries (idx 0 â†’ symbol 0, idx 1 â†’ symbol 1).
    // Then v = 0, so (1 << 0) = 1, pop(0) = 0 â†’ position = 1.
    let dist_huff = Huffman::with_lengths(vec![1, 1]).expect("valid distance huffman");
    let mut comp = EmbCompress::new(vec![0x80]); // peek(1) = 1, then pop(0)
    comp.block_elements = 1;
    comp.distance_huffman = Some(dist_huff);
    let pos = comp.get_position().expect("get_position should succeed");
    // distance_huffman lookup with 0x8000 has idx=1 (table_width=1, top bit of 16)
    // value = 1, v = 0, pop(0) = 0, pos = (1 << 0) + 0 = 1
    assert_eq!(pos, 1, "single-bit distance lookup should return 1");
}

// â”€â”€ EmbCompress get_token tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn embcompress_get_token_triggers_block_load_when_empty() {
    // block_elements starts at 0 â†’ load_block will be called.
    // With only 1 byte of data (need 2+ for pop(16)), load_block should fail gracefully.
    let mut comp = EmbCompress::new(vec![0xFF]);
    comp.block_elements = 0;
    let _result = comp.get_token();
    // May return Ok or Err depending on what the single byte encodes,
    // but must not panic regardless.
}

#[test]
fn embcompress_get_token_missing_character_huffman_errors() {
    // block_elements > 0 so load_block isn't called, but character_huffman is None.
    let mut comp = EmbCompress::new(vec![]);
    comp.block_elements = 1;
    let result = comp.get_token();
    assert!(
        result.is_err(),
        "expected error when character_huffman is missing"
    );
}

#[test]
fn embcompress_get_token_with_valid_huffman() {
    // Use character_huffman with [1] (1 symbol, bit-length 1) returning 0.
    // feed peek(16) which returns 0 for leading-zero bits of 0x00.
    let char_huff = Huffman::with_lengths(vec![1]).expect("valid huffman");
    let mut comp = EmbCompress::new(vec![0x00]);
    comp.block_elements = 1;
    comp.character_huffman = Some(char_huff);
    let token = comp.get_token().expect("get_token should succeed");
    // lookup(0x0000): idx = 0 >> 15 = 0 â†’ value=0, len=1
    assert_eq!(token, 0, "single-symbol huffman should return 0");
}

// â”€â”€ EmbCompress decompress edge-case tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn embcompress_decompress_empty_data() {
    let mut comp = EmbCompress::new(vec![]);
    let result = comp.decompress(None);
    assert!(result.is_ok(), "empty data should decompress successfully");
    assert!(result.unwrap().is_empty(), "output should be empty");
}

#[test]
fn embcompress_decompress_garbage_does_not_panic() {
    // Garbage data that may or may not form a valid compressed stream.
    // The method must not panic regardless.
    let mut comp = EmbCompress::new(vec![0xFF, 0xFF, 0xFF, 0xFF]);
    comp.block_elements = 0;
    let _result = comp.decompress(None);
}

// â”€â”€ Reader trait conformance â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn hus_reader_no_panic_on_empty_data() {
    let _result = HusReader.read(&[]);
}
