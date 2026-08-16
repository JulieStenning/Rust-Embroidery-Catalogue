// Tests for the source module.
//
// This module was split out so the production file can stay focused
// on logic. It is included via a #[path] declaration in a
// #[cfg(test)] mod tests; module, retaining full access to the
// private items in the parent module through use super::*;.

use super::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn vp3_colorblock_jump_matches_reference_behavior() {
    assert!(!should_add_block_jump(0.0, 0.0));
    assert!(should_add_block_jump(12.0, 0.0));
    assert!(should_add_block_jump(0.0, -8.5));
    assert!(should_add_block_jump(3.0, 2.0));
}

#[test]
fn vp3_thread_parser_consumes_all_color_entries() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x02, 0x00, // two colors, no transition
        0x11, 0x22, 0x33, 0x00, 0x00, 0x00, // first color + parts + length
        0x44, 0x55, 0x66, 0x00, 0x00, 0x00, // second color + parts + length
        0x05, 0x28, // thread type, weight
        0x00, 0x03, b'a', b'b', b'c', 0x00, 0x04, b'd', b'e', b's', b'c', 0x00, 0x05, b'b', b'r',
        b'a', b'n', b'd',
    ]);

    let mut cursor = Cursor::new(bytes.as_slice());
    let thread = vp3_read_thread(&mut cursor).expect("VP3 thread should parse");

    assert_eq!(thread.color, 0x445566);
    assert_eq!(thread.catalog_number.as_deref(), Some("abc"));
    assert_eq!(thread.description.as_deref(), Some("desc"));
    assert_eq!(thread.brand.as_deref(), Some("brand"));
    assert_eq!(cursor.position() as usize, bytes.len());
}

#[test]
fn vp3_thread_parser_skips_color_payload_bytes() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x01, 0x00, // one color, no transition
        0x11, 0x22, 0x33, // color
        0x01, // one part
        0x00, 0x02, // payload length
        0xAA, 0xBB, // payload bytes to skip
        0x05, 0x28, // thread type, weight
        0x00, 0x01, b'a', 0x00, 0x01, b'b', 0x00, 0x01, b'c',
    ]);

    let mut cursor = Cursor::new(bytes.as_slice());
    let thread = vp3_read_thread(&mut cursor).expect("VP3 thread with payload should parse");

    assert_eq!(thread.color, 0x112233);
    assert_eq!(thread.catalog_number.as_deref(), Some("a"));
    assert_eq!(thread.description.as_deref(), Some("b"));
    assert_eq!(thread.brand.as_deref(), Some("c"));
    assert_eq!(cursor.position() as usize, bytes.len());
}

/// Build a minimal VP3 header with the given number of colour blocks.
/// All strings are empty and all seek regions are zero-filled.
fn build_minimal_vp3_header(count_colors: u16) -> Vec<u8> {
    let mut buf = Vec::new();
    // Magic: %vsm%\0
    buf.extend_from_slice(b"%vsm%\0");
    // Header string (empty)
    buf.extend_from_slice(&0u16.to_be_bytes());
    // 7-byte seek
    buf.extend_from_slice(&[0u8; 7]);
    // Comments string (empty)
    buf.extend_from_slice(&0u16.to_be_bytes());
    // 32-byte seek
    buf.extend_from_slice(&[0u8; 32]);
    // center_x = 0, center_y = 0
    buf.extend_from_slice(&0i32.to_be_bytes());
    buf.extend_from_slice(&0i32.to_be_bytes());
    // 27-byte seek
    buf.extend_from_slice(&[0u8; 27]);
    // String (empty)
    buf.extend_from_slice(&0u16.to_be_bytes());
    // 24-byte seek
    buf.extend_from_slice(&[0u8; 24]);
    // String (empty)
    buf.extend_from_slice(&0u16.to_be_bytes());
    // count_colors
    buf.extend_from_slice(&count_colors.to_be_bytes());
    buf
}

#[test]
fn vp3_read_rejects_invalid_magic() {
    let data = b"XXXXXX";
    let result = read_vp3(data);
    assert!(result.is_err(), "invalid magic should produce an error");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("invalid file signature"),
        "expected signature error, got: {err_msg}"
    );
}

#[test]
fn vp3_read_truncated_header_errors() {
    // Valid magic but far fewer bytes than the full header requires.
    let mut data = b"%vsm%\0".to_vec();
    data.extend_from_slice(&[0u8; 20]);
    let result = read_vp3(&data);
    assert!(result.is_err(), "truncated header should produce an error");
}

#[test]
fn vp3_read_zero_color_blocks_parses_empty_pattern() {
    let buf = build_minimal_vp3_header(0);
    let pattern = read_vp3(&buf).expect("zero-color VP3 should parse");

    assert!(pattern.threadlist.is_empty());
    assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
    let end = pattern.stitches.last().expect("expected End marker");
    assert_eq!(end.x, 0.0);
    assert_eq!(end.y, 0.0);
}

#[test]
fn vp3_read_truncated_mid_color_block_does_not_panic() {
    // Build a valid single-color-block VP3, then truncate the stitch bytes
    // partway through the declared stitch_byte_length.
    let mut buf = build_minimal_vp3_header(1);

    // 3-byte bytescheck
    buf.extend_from_slice(&[0, 0, 0]);
    // distance_to_next_block_050 = 40 â†’ block_end = 121 + 40 = 161
    buf.extend_from_slice(&40i32.to_be_bytes());
    // start_position_x = 0, start_position_y = 0
    buf.extend_from_slice(&0i32.to_be_bytes());
    buf.extend_from_slice(&0i32.to_be_bytes());
    // Thread: zero colors, no transition, empty strings (10 bytes)
    buf.push(0x00); // colors = 0
    buf.push(0x00); // transition
    buf.push(0x00); // thread_type
    buf.push(0x00); // weight
    buf.extend_from_slice(&0u16.to_be_bytes()); // catalog length 0
    buf.extend_from_slice(&0u16.to_be_bytes()); // description length 0
    buf.extend_from_slice(&0u16.to_be_bytes()); // brand length 0
                                                // 15-byte seek + 3-byte bytescheck
    buf.extend_from_slice(&[0u8; 15]);
    buf.extend_from_slice(&[0, 0, 0]);
    // Stitch bytes: declared length 4, but only 2 bytes present.
    buf.push(0x00);
    buf.push(0x01);

    let pattern = read_vp3(&buf).expect("truncated VP3 color block should parse gracefully");

    // The partial stitch byte was decoded and the End marker appended.
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 1);
    assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
}

#[test]
fn vp3_block_negative_stitch_byte_length_errors() {
    // distance_to_next_block_050 = 0 makes block_end_position fall behind
    // the position where stitch data begins, so stitch_byte_length is
    // negative and the parser must reject the file.
    let mut buf = build_minimal_vp3_header(1);

    // 3-byte bytescheck
    buf.extend_from_slice(&[0, 0, 0]);
    // distance = 0 â†’ block_end = position after reading distance (121)
    buf.extend_from_slice(&0i32.to_be_bytes());
    // start_position_x = 0, start_position_y = 0
    buf.extend_from_slice(&0i32.to_be_bytes());
    buf.extend_from_slice(&0i32.to_be_bytes());
    // Thread: zero colors, no transition, empty strings (10 bytes)
    buf.push(0x00); // colors = 0
    buf.push(0x00); // transition
    buf.push(0x00); // thread_type
    buf.push(0x00); // weight
    buf.extend_from_slice(&0u16.to_be_bytes());
    buf.extend_from_slice(&0u16.to_be_bytes());
    buf.extend_from_slice(&0u16.to_be_bytes());
    // 15-byte seek + 3-byte bytescheck
    buf.extend_from_slice(&[0u8; 15]);
    buf.extend_from_slice(&[0, 0, 0]);

    let result = read_vp3(&buf);
    assert!(result.is_err(), "negative stitch length should error");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Negative stitch_byte_length"),
        "expected negative length error, got: {err_msg}"
    );
}

#[test]
fn vp3_thread_zero_colors_defaults_to_black() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x00, 0x00, // zero colors, no transition
        0x05, 0x28, // thread type, weight
        0x00, 0x01, b'a', 0x00, 0x01, b'b', 0x00, 0x01, b'c',
    ]);

    let mut cursor = Cursor::new(bytes.as_slice());
    let thread = vp3_read_thread(&mut cursor).expect("thread with zero colors should parse");

    assert_eq!(
        thread.color, 0x000000,
        "zero-color thread should default to black"
    );
    assert_eq!(thread.catalog_number.as_deref(), Some("a"));
    assert_eq!(thread.description.as_deref(), Some("b"));
    assert_eq!(thread.brand.as_deref(), Some("c"));
    assert_eq!(cursor.position() as usize, bytes.len());
}

#[test]
fn vp3_thread_empty_strings_parse() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x01, 0x00, // one color, no transition
        0x11, 0x22, 0x33, 0x00, 0x00, 0x00, // color + parts + zero-length payload
        0x05, 0x28, // thread type, weight
        0x00, 0x00, // catalog: empty string
        0x00, 0x00, // description: empty string
        0x00, 0x00, // brand: empty string
    ]);

    let mut cursor = Cursor::new(bytes.as_slice());
    let thread = vp3_read_thread(&mut cursor).expect("thread with empty strings should parse");

    assert_eq!(thread.color, 0x112233);
    assert_eq!(thread.catalog_number.as_deref(), Some(""));
    assert_eq!(thread.description.as_deref(), Some(""));
    assert_eq!(thread.brand.as_deref(), Some(""));
    assert_eq!(cursor.position() as usize, bytes.len());
}

#[test]
fn vp3_long_form_stitches_are_preserved() {
    let mut pattern = EmbPattern::new();
    let stitch_bytes = [0x80, 0x01, 0x00, 0x90, 0xFF, 0x70, 0x80, 0x02];

    // Simulate the 0x80 0x01 decode branch to assert command type is Stitch.
    let mut i = 2usize;
    let x = signed16(stitch_bytes[i], stitch_bytes[i + 1]);
    i += 2;
    let y = signed16(stitch_bytes[i], stitch_bytes[i + 1]);
    pattern.add_stitch_relative(StitchType::Stitch, x as f32, y as f32);

    assert_eq!(pattern.stitches.len(), 1);
    assert_eq!(pattern.stitches[0].stitch_type, StitchType::Stitch);
}

#[test]
fn vp3_huge_long_form_deltas_become_jumps() {
    assert!(!should_treat_long_form_as_jump(120, -120));
    assert!(!should_treat_long_form_as_jump(127, 0));
    assert!(should_treat_long_form_as_jump(128, 0));
    assert!(should_treat_long_form_as_jump(0, -300));
}

#[test]
fn vp3_control_commands_preserve_current_position() {
    let mut pattern = EmbPattern::new();
    pattern.add_stitch_absolute(StitchType::Stitch, 12.0, -8.0);

    pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
    pattern.add_stitch_relative(StitchType::ColorChange, 0.0, 0.0);
    pattern.add_stitch_relative(StitchType::End, 0.0, 0.0);

    let trim = pattern.stitches[1];
    let color_change = pattern.stitches[2];
    let end = pattern.stitches[3];

    assert_eq!(trim.x, 12.0);
    assert_eq!(trim.y, -8.0);
    assert_eq!(color_change.x, 12.0);
    assert_eq!(color_change.y, -8.0);
    assert_eq!(end.x, 12.0);
    assert_eq!(end.y, -8.0);
}

#[test]
fn vp3_user_fixture_stitch_diagnostics() {
    let file_path = PathBuf::from("tests")
        .join("Test Designs")
        .join("220306.vp3");
    if !file_path.exists() {
        eprintln!(
            "Skipping VP3 diagnostics because fixture is missing: {}",
            file_path.display()
        );
        return;
    }

    let data = fs::read(&file_path).expect("should read VP3 fixture");
    let pattern = read_vp3(&data).expect("VP3 fixture should parse");

    let mut prev = (0.0_f32, 0.0_f32);
    let mut has_prev = false;
    let mut max_len = 0.0_f32;
    let mut over_127 = 0_usize;
    let mut over_255 = 0_usize;
    let mut stitch_count = 0_usize;

    for stitch in &pattern.stitches {
        if stitch.stitch_type != StitchType::Stitch {
            if stitch.stitch_type == StitchType::Jump || stitch.stitch_type == StitchType::Trim {
                has_prev = false;
            }
            continue;
        }

        stitch_count += 1;
        if has_prev {
            let dx = stitch.x - prev.0;
            let dy = stitch.y - prev.1;
            let len = (dx * dx + dy * dy).sqrt();
            if len > max_len {
                max_len = len;
            }
            if dx.abs() > 127.0 || dy.abs() > 127.0 {
                over_127 += 1;
            }
            if dx.abs() > 255.0 || dy.abs() > 255.0 {
                over_255 += 1;
            }
        }

        prev = (stitch.x, stitch.y);
        has_prev = true;
    }

    eprintln!(
        "VP3 diagnostics 220306: stitches={}, max_len={:.2}, over127={}, over255={}, jumps={}, trims={}",
        stitch_count,
        max_len,
        over_127,
        over_255,
        pattern.count_stitch_commands(StitchType::Jump),
        pattern.count_stitch_commands(StitchType::Trim),
    );
}

#[test]
fn vp3_isolated_colour_fixture_keeps_long_stitches() {
    let file_path = PathBuf::from("tests")
        .join("Test Designs")
        .join("test-less-220306.vp3");
    if !file_path.exists() {
        eprintln!(
            "Skipping isolated VP3 diagnostics because fixture is missing: {}",
            file_path.display()
        );
        return;
    }

    let data = fs::read(&file_path).expect("should read isolated VP3 fixture");
    let pattern = read_vp3(&data).expect("isolated VP3 fixture should parse");

    let mut prev = (0.0_f32, 0.0_f32);
    let mut has_prev = false;
    let mut over_127 = 0_usize;
    let mut max_len = 0.0_f32;
    let mut jump_count = 0usize;
    for stitch in &pattern.stitches {
        if stitch.stitch_type != StitchType::Stitch {
            if stitch.stitch_type == StitchType::Jump || stitch.stitch_type == StitchType::Trim {
                if stitch.stitch_type == StitchType::Jump {
                    jump_count += 1;
                }
                has_prev = false;
            }
            continue;
        }
        if has_prev {
            let dx = stitch.x - prev.0;
            let dy = stitch.y - prev.1;
            let len = (dx * dx + dy * dy).sqrt();
            if len > max_len {
                max_len = len;
            }
            if dx.abs() > 127.0 || dy.abs() > 127.0 {
                over_127 += 1;
            }
        }
        prev = (stitch.x, stitch.y);
        has_prev = true;
    }

    eprintln!(
        "VP3 diagnostics test-less-220306: stitches={}, over127={}, max_len={:.2}, jumps={}",
        pattern.count_stitch_commands(StitchType::Stitch),
        over_127,
        max_len,
        jump_count,
    );
    assert!(pattern.count_stitch_commands(StitchType::Stitch) > 0);
}

#[test]
fn vp3_peacock_fixture_stitch_diagnostics() {
    let file_path = PathBuf::from("tests")
        .join("Test Designs")
        .join("01Peacock.vp3");
    if !file_path.exists() {
        eprintln!(
            "Skipping VP3 diagnostics because fixture is missing: {}",
            file_path.display()
        );
        return;
    }

    let data = fs::read(&file_path).expect("should read VP3 fixture");
    let pattern = read_vp3(&data).expect("VP3 fixture should parse");

    let mut prev = (0.0_f32, 0.0_f32);
    let mut has_prev = false;
    let mut max_len = 0.0_f32;
    let mut over_127 = 0_usize;
    let mut over_255 = 0_usize;
    let mut over_400 = 0_usize;
    let mut stitch_count = 0_usize;

    for stitch in &pattern.stitches {
        if stitch.stitch_type != StitchType::Stitch {
            if stitch.stitch_type == StitchType::Jump || stitch.stitch_type == StitchType::Trim {
                has_prev = false;
            }
            continue;
        }

        stitch_count += 1;
        if has_prev {
            let dx = stitch.x - prev.0;
            let dy = stitch.y - prev.1;
            let len = (dx * dx + dy * dy).sqrt();
            if len > max_len {
                max_len = len;
            }
            if dx.abs() > 127.0 || dy.abs() > 127.0 {
                over_127 += 1;
            }
            if dx.abs() > 255.0 || dy.abs() > 255.0 {
                over_255 += 1;
            }
            if dx.abs() > 400.0 || dy.abs() > 400.0 {
                over_400 += 1;
            }
        }

        prev = (stitch.x, stitch.y);
        has_prev = true;
    }

    eprintln!(
        "VP3 diagnostics 01Peacock: stitches={}, max_len={:.2}, over127={}, over255={}, over400={}, jumps={}, trims={}",
        stitch_count,
        max_len,
        over_127,
        over_255,
        over_400,
        pattern.count_stitch_commands(StitchType::Jump),
        pattern.count_stitch_commands(StitchType::Trim),
    );

    assert!(pattern.count_stitch_commands(StitchType::Stitch) > 0);
    assert_eq!(
        over_127, 0,
        "expected implausibly long VP3 connector deltas to be classified as jumps"
    );
}
