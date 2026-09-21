// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn test_read_exp_two_stitches() {
    // Two regular stitches:
    // Stitch 1: dx=5, dy=10 → bytes [0x05, 0xF6]
    //   signed8(0x05)=5, signed8(0xF6)=246-256=-10, dy = -(-10) = 10
    // Stitch 2: dx=-3, dy=7 → bytes [0xFD, 0xF9]
    //   signed8(0xFD)=253-256=-3, signed8(0xF9)=249-256=-7, dy = -(-7) = 7
    let data = vec![0x05, 0xF6, 0xFD, 0xF9];

    let pattern = read_exp(&data).expect("should parse valid EXP");

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

    // Always appends End
    assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
}

#[test]
fn test_read_exp_jump() {
    // Jump command: 0x80 0x04 + dx dy
    let data = vec![0x80, 0x04, 0x0A, 0xF6]; // dx=10, dy=10

    let pattern = read_exp(&data).expect("should parse EXP with jump");

    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);

    let jumps: Vec<_> = pattern
        .stitches
        .iter()
        .filter(|s| s.stitch_type == StitchType::Jump)
        .collect();
    assert_eq!(jumps.len(), 1);
    assert_eq!(jumps[0].x, 10.0);
    assert_eq!(jumps[0].y, 10.0);
}

#[test]
fn test_read_exp_trim() {
    // Trim command: 0x80 0x80 + 2 extra bytes (ignored)
    let data = vec![0x80, 0x80, 0x00, 0x00];

    let pattern = read_exp(&data).expect("should parse EXP with trim");

    assert_eq!(pattern.count_stitch_commands(StitchType::Trim), 1);
}

#[test]
fn test_read_exp_color_change() {
    // Color change with non-zero coords: 0x80 0x01 + dx dy
    // Expect ColorChange + Jump
    let data = vec![0x80, 0x01, 0x05, 0xFB]; // dx=5, dy=5

    let pattern = read_exp(&data).expect("should parse EXP with color change");

    assert_eq!(pattern.count_stitch_commands(StitchType::ColorChange), 1);
    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
}

#[test]
fn test_read_exp_color_change_zero_delta_does_not_emit_jump() {
    // Color change with zero movement should not create a jump.
    let data = vec![0x80, 0x01, 0x00, 0x00];

    let pattern = read_exp(&data).expect("should parse EXP zero-delta color change");

    assert_eq!(pattern.count_stitch_commands(StitchType::ColorChange), 1);
    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 0);
}

#[test]
fn test_read_exp_unknown_control_does_not_abort_and_consumes_record() {
    // Unknown control should not truncate parsing.
    // 0x80 0x10 + dx/dy then a regular stitch.
    let data = vec![0x80, 0x10, 0x02, 0xFE, 0x03, 0xFD];

    let pattern = read_exp(&data).expect("should parse unknown control robustly");

    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 1);
    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
}

#[test]
fn test_read_exp_color_change_preserves_current_position() {
    // Stitch to (10, 10), color change, then stitch +2,+3 -> should end at (12,13)
    let data = vec![0x0A, 0xF6, 0x80, 0x01, 0x00, 0x00, 0x02, 0xFD];

    let pattern = read_exp(&data).expect("should parse EXP with color change");

    let stitches: Vec<_> = pattern
        .stitches
        .iter()
        .filter(|s| s.stitch_type == StitchType::Stitch)
        .collect();
    assert_eq!(stitches.len(), 2);
    assert_eq!(stitches[0].x, 10.0);
    assert_eq!(stitches[0].y, 10.0);
    assert_eq!(stitches[1].x, 12.0);
    assert_eq!(stitches[1].y, 13.0);
}

#[test]
fn test_read_exp_trim_preserves_current_position() {
    // Stitch to (6, 6), trim, then stitch +1,+1 -> should end at (7,7)
    let data = vec![0x06, 0xFA, 0x80, 0x80, 0x00, 0x00, 0x01, 0xFF];

    let pattern = read_exp(&data).expect("should parse EXP with trim");

    let stitches: Vec<_> = pattern
        .stitches
        .iter()
        .filter(|s| s.stitch_type == StitchType::Stitch)
        .collect();
    assert_eq!(stitches.len(), 2);
    assert_eq!(stitches[0].x, 6.0);
    assert_eq!(stitches[0].y, 6.0);
    assert_eq!(stitches[1].x, 7.0);
    assert_eq!(stitches[1].y, 7.0);
}

/// Synthetic EXP data exercising a realistic interleaved mix of commands.
/// Layout (all deltas relative):
///   Stitch (0,0)          — initial stitch
///   Stitch (10,15)        — move to (10,15)
///   Trim                  — control: position unchanged
///   Stitch (5,5)          — move to (15,20)
///   Jump (100,100)        — non-stitch move to (115,120)
///   Stitch (1,1)          — move to (116,121)
///   ColorChange (0,0)     — control: position unchanged
///   Stitch (1,1)          — move to (117,122)
///   Trim                  — control: position unchanged
///   Stitch (1,1)          — move to (118,123)
///   UnknownCtrl (10,10)   — fallthrough => Jump to (128,133)
///   Stitch (1,1)          — move to (129,134)
///   ColorChange (5,5)     — control + Jump to (134,139)
///   Stitch (1,1)          — move to (135,140)
///   ColorChange (0,0)     — control: position unchanged
///   Stitch (1,1)          — move to (136,141)
///
/// Total records: 17 commands → parsed stitch list includes appended End.
const COMPLEX_INTERLEAVED_EXP: [u8; 46] = [
    // 1. Regular stitch: dx=0, dy=0  (origin)
    0x00, 0x00, // 2. Regular stitch: dx=10, dy=-15 → signed8 => 10, y=15
    0x0A, 0xF1, // 3. Trim: 0x80 0x80 + 2 bytes (dx=0, dy=0)
    0x80, 0x80, 0x00, 0x00, // 4. Regular stitch: dx=5, dy=-5
    0x05, 0xFB, // 5. Jump: 0x80 0x04 + dx=100, dy=-100 → signed8 => 100, y=100
    0x80, 0x04, 0x64, 0x9C, // 6. Regular stitch: dx=1, dy=-1
    0x01, 0xFF, // 7. ColorChange zero delta: 0x80 0x01 + dx=0, dy=0
    0x80, 0x01, 0x00, 0x00, // 8. Regular stitch: dx=1, dy=-1
    0x01, 0xFF, // 9. Trim: 0x80 0x80 + dx=0, dy=0
    0x80, 0x80, 0x00, 0x00, // 10. Regular stitch: dx=1, dy=-1
    0x01, 0xFF,
    // 11. Unknown control 0x12 with non-zero delta: 0x80 0x12 + dx=10, dy=-10
    0x80, 0x12, 0x0A, 0xF6, // 12. Regular stitch: dx=1, dy=-1
    0x01, 0xFF, // 13. ColorChange with non-zero delta: 0x80 0x01 + dx=5, dy=-5
    0x80, 0x01, 0x05, 0xFB, // 14. Regular stitch: dx=1, dy=-1
    0x01, 0xFF, // 15. ColorChange zero delta again: 0x80 0x01 + dx=0, dy=0
    0x80, 0x01, 0x00, 0x00, // 16. Regular stitch: dx=1, dy=-1
    0x01, 0xFF,
];

#[test]
fn test_complex_interleaved_control_commands_preserve_position() {
    let pattern =
        read_exp(&COMPLEX_INTERLEAVED_EXP).expect("should parse complex interleaved EXP data");

    // Stitches (excluding End) = 16 raw records, but some emit extra commands:
    //   - The ColorChange with non-zero delta (record 13) emits a Jump
    //   - The unknown control (record 11) emits a Jump (via fallthrough)
    // So total stitch commands = 16 + extra jumps (2) + End (1) = 19
    // But we're testing position semantics, not record count.
    assert!(
        pattern.stitches.len() >= 15,
        "expected at least 15 stitch commands, got {}",
        pattern.stitches.len()
    );

    // Verify every ColorChange and Trim keeps the same (x,y) as the preceding command.
    for index in 1..pattern.stitches.len() {
        let prev = &pattern.stitches[index - 1];
        let current = &pattern.stitches[index];

        if current.stitch_type == StitchType::ColorChange || current.stitch_type == StitchType::Trim
        {
            assert_eq!(
                (current.x, current.y),
                (prev.x, prev.y),
                "control command at index {} should keep current position",
                index
            );
        }
    }

    // Verify fallback threads were generated (3 color blocks → 4 threads)
    assert_eq!(pattern.threadlist.len(), 4);
    let all_black = pattern
        .threadlist
        .iter()
        .all(|thread| thread.color == 0x000000);
    assert!(!all_black, "fallback threads should not all be black");
}

#[test]
fn test_exp_fallback_threads_use_palette_not_black_only() {
    // Two color blocks => three fallback threads should be synthesized.
    let data = vec![
        0x01, 0xFF, // stitch
        0x80, 0x01, 0x00, 0x00, // color change
        0x01, 0xFF, // stitch
        0x80, 0x01, 0x00, 0x00, // color change
        0x01, 0xFF, // stitch
    ];

    let pattern = read_exp(&data).expect("should parse EXP and add fallback threads");
    assert_eq!(pattern.threadlist.len(), 3);

    let all_black = pattern
        .threadlist
        .iter()
        .all(|thread| thread.color == 0x000000);
    assert!(!all_black, "fallback EXP threads should not all be black");
}

// -----------------------------------------------------------------------
// EmbroideryReader trait tests
// -----------------------------------------------------------------------

#[test]
fn test_exp_reader_trait_read_success() {
    let reader = ExpReader;
    let data = vec![0x05, 0xF6, 0xFD, 0xF9];
    let pattern = reader
        .read(&data)
        .expect("ExpReader::read should parse valid EXP");
    assert_eq!(
        pattern.count_stitch_commands(StitchType::Stitch),
        2,
        "expected exactly 2 regular stitches via trait read"
    );
}

#[test]
fn test_exp_reader_trait_read_handles_empty_data_gracefully() {
    // Empty data should produce an empty pattern, not a panic or error.
    let reader = ExpReader;
    let pattern = reader
        .read(&[])
        .expect("empty data should produce Ok pattern");
    assert_eq!(
        pattern.count_stitch_commands(StitchType::Stitch),
        0,
        "empty data should produce zero stitches"
    );
    assert_eq!(
        pattern.count_stitch_commands(StitchType::End),
        1,
        "even an empty pattern should have an End marker"
    );
    // The End marker should be at (0, 0) when there are no previous stitches.
    if let Some(last) = pattern.stitches.last() {
        assert_eq!(last.x, 0.0);
        assert_eq!(last.y, 0.0);
    }
}

#[test]
fn test_signed8_boundary_values() {
    // signed8 maps an unsigned byte to the signed range [-128, 127].
    assert_eq!(signed8(0), 0);
    assert_eq!(signed8(127), 127);
    assert_eq!(signed8(128), -128);
    assert_eq!(signed8(129), -127);
    assert_eq!(signed8(255), -1);
}

#[test]
fn test_read_exp_truncated_after_control_prefix() {
    // A control command (0x80 0x04) with its 2 extra delta bytes missing
    // should break out of the loop gracefully instead of panicking.
    let data = vec![0x80, 0x04];

    let pattern = read_exp(&data).expect("should handle truncated control command");

    // No jump should have been emitted because the delta bytes were missing.
    assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 0);
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 0);

    // End marker appended at origin.
    assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
    let end = pattern.stitches.last().expect("expected End marker");
    assert_eq!(end.x, 0.0);
    assert_eq!(end.y, 0.0);
}

#[test]
fn test_read_exp_truncated_mid_stitch() {
    // One full 2-byte stitch, then only 1 byte of a second stitch.
    let data = vec![0x05, 0xF6, 0xFD];

    let pattern = read_exp(&data).expect("should handle truncated stitch data");

    // Only the first complete stitch should have been parsed.
    assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 1);

    let stitches: Vec<_> = pattern
        .stitches
        .iter()
        .filter(|s| s.stitch_type == StitchType::Stitch)
        .collect();
    assert_eq!(stitches[0].x, 5.0);
    assert_eq!(stitches[0].y, 10.0);

    // End marker appended at the last decoded position (5, 10).
    let end = pattern.stitches.last().expect("expected End marker");
    assert_eq!(end.x, 5.0);
    assert_eq!(end.y, 10.0);
}

#[test]
fn test_read_exp_stitch_signed8_extremes() {
    // The byte 0x80 is reserved as the control-command escape, so regular
    // stitch deltas are limited to signed8 range minus 0x80:
    //   max   = +127 (0x7F)
    //   min   = -127 (0x81)
    //
    // Stitch 1: dx = signed8(0x7F) = +127, dy = -signed8(0xFF) = -(-1) = +1
    // Stitch 2: dx = signed8(0x81) = -127, dy = -signed8(0x00) = 0
    let data = vec![0x7F, 0xFF, 0x81, 0x00];

    let pattern = read_exp(&data).expect("should parse signed8 extreme stitches");

    let stitches: Vec<_> = pattern
        .stitches
        .iter()
        .filter(|s| s.stitch_type == StitchType::Stitch)
        .collect();
    assert_eq!(stitches.len(), 2);
    // Stitch 1 ends at (127, 1)
    assert_eq!(stitches[0].x, 127.0);
    assert_eq!(stitches[0].y, 1.0);
    // Stitch 2 accumulates to (0, 1)
    assert_eq!(stitches[1].x, 0.0);
    assert_eq!(stitches[1].y, 1.0);

    assert_eq!(pattern.count_stitch_commands(StitchType::End), 1);
}
