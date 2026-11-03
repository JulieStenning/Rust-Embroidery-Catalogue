// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

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

#[test]
fn test_read_dst_jump_command_maps_to_jump() {
    let mut data = vec![0u8; 512];

    // Jump record: bits 0-1 and 7 set on byte[2] => 0x83.
    data.push(0x00);
    data.push(0x00);
    data.push(0x83);

    // End marker.
    data.push(0x00);
    data.push(0x00);
    data.push(0xF3);

    let pattern = read_dst(&data).expect("should parse DST jump command");

    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 0);
}

#[test]
fn test_read_dst_synthetic_co_generates_threads() {
    // Build a minimal DST file with CO:18 in the header.
    // CO (colour-change count) = 18, so 19 thread blocks expected.
    let mut data = vec![0u8; 512];
    let header_text = b"CO:18\r";
    data[..header_text.len()].copy_from_slice(header_text);

    // Append END marker
    data.push(0x00);
    data.push(0x00);
    data.push(0xF3);

    let pattern = read_dst(&data).expect("should parse synthetic DST");

    assert_eq!(pattern.extras.get("CO").map(|s| s.as_str()), Some("18"));
    assert_eq!(pattern.threadlist.len(), 19);
}

#[test]
fn test_dst_reader_trait_read() {
    // Test that the EmbroideryReader trait impl (DstReader::read) works.
    let mut data = vec![0u8; 512];

    // Two stitches: dx=+1, dy=+1 -> b0=0x81
    data.push(0x81);
    data.push(0x00);
    data.push(0x00);
    data.push(0x81);
    data.push(0x00);
    data.push(0x00);

    // END marker
    data.push(0x00);
    data.push(0x00);
    data.push(0xF3);

    let reader = DstReader;
    let pattern = reader.read(&data).expect("DstReader::read should succeed");

    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 2);
    assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
}

#[test]
fn test_decode_dx_maximum_values() {
    // Maximum dx = +81 +27 +9 +3 +1 = +121
    // b2 bit2=1 (0x04), b1 bit2=1 + bit0=1 (0x05), b0 bit2=1 + bit0=1 (0x05)
    assert_eq!(decode_dx(0x05, 0x05, 0x04), 121.0);

    // Minimum dx = -81 -27 -9 -3 -1 = -121
    // b2 bit3=1 (0x08), b1 bit3=1 + bit1=1 (0x0A), b0 bit3=1 + bit1=1 (0x0A)
    assert_eq!(decode_dx(0x0A, 0x0A, 0x08), -121.0);
}

#[test]
fn test_decode_dy_maximum_values() {
    // Maximum dy = +121 (internal sum -121 negated)
    // b2 bit4=1 (0x10), b1 bit4=1 + bit6=1 (0x50), b0 bit4=1 + bit6=1 (0x50)
    assert_eq!(decode_dy(0x50, 0x50, 0x10), 121.0);

    // Minimum dy = -121 (internal sum +121 negated)
    // b2 bit5=1 (0x20), b1 bit5=1 + bit7=1 (0xA0), b0 bit5=1 + bit7=1 (0xA0)
    assert_eq!(decode_dy(0xA0, 0xA0, 0x20), -121.0);
}

#[test]
fn test_read_dst_truncated_mid_stitch_record() {
    // Build a valid header + two full stitches, then truncate mid-record
    // on the third stitch (only 2 of the required 3 bytes present).
    let mut data = vec![0u8; 512];

    // Stitch 1: dx=+1, dy=+1 -> b0=0x81
    data.push(0x81);
    data.push(0x00);
    data.push(0x00);

    // Stitch 2: dx=+1, dy=+1 -> b0=0x81
    data.push(0x81);
    data.push(0x00);
    data.push(0x00);

    // Truncated third stitch — only 2 of 3 bytes.
    data.push(0x81);
    data.push(0x00);
    // NOTE: third byte omitted → read_exact fails, loop breaks gracefully.

    let pattern = read_dst(&data).expect("should handle truncated DST gracefully");

    // The two complete stitches should have been retained.
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 2);

    // End marker appended at the last decoded position.
    assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
    let end = pattern
        .stitches
        .iter()
        .find(|s| s.stitch_type == StitchType::End)
        .expect("expected End marker");
    assert_eq!(end.x, 2.0);
    assert_eq!(end.y, -2.0);
}

#[test]
fn test_read_dst_oversized_exceeds_standard_hoop_succeeds() {
    // Safety test: verify that when a DST file has dimensions extending far
    // beyond standard embroidery hoops (e.g. 240x240 mm), DstReader parses
    // every stitch completely without error or truncation.
    let mut data = vec![0u8; 512];

    // Each stitch with b0=0x05, b1=0x05, b2=0x04 advances dx by +121 (12.1 mm).
    // Adding 20 stitches advances X by 2420 units (242.0 mm), far exceeding a 100x100mm hoop.
    for _ in 0..20 {
        data.push(0x05);
        data.push(0x05);
        data.push(0x04);
    }

    // END marker
    data.push(0x00);
    data.push(0x00);
    data.push(0xF3);

    let pattern = DstReader
        .read(&data)
        .expect("DST reader must parse oversized designs");
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 20);

    let (min_x, _min_y, max_x, _max_y) = pattern.bounds();
    let width_mm = ((max_x - min_x).abs()) / 10.0;
    assert!(width_mm > 200.0, "Expected width > 200mm, got {width_mm}");
}
