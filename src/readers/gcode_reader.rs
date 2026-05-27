use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

pub struct GcodeReader;

impl EmbroideryReader for GcodeReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_gcode(data)
    }
}

fn current_position(pattern: &EmbPattern) -> (f32, f32) {
    pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0))
}

fn add_command_here(pattern: &mut EmbPattern, stitch_type: StitchType) {
    let (x, y) = current_position(pattern);
    pattern.add_stitch_absolute(stitch_type, x, y);
}

fn parse_words(line: &str) -> Vec<(char, f32)> {
    let mut words = Vec::new();
    for token in line.split_whitespace() {
        if token.len() < 2 {
            continue;
        }
        let mut chars = token.chars();
        let letter = chars.next().unwrap_or(' ');
        if !letter.is_ascii_alphabetic() {
            continue;
        }
        let value_text: String = chars.collect();
        if let Ok(value) = value_text.parse::<f32>() {
            words.push((letter.to_ascii_lowercase(), value));
        }
    }
    words
}

fn read_gcode(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::new(0x000000));

    let text = String::from_utf8_lossy(data);
    let mut absolute_mode = true;
    let mut scale: f32 = 10.0; // mm -> tenths of mm
    let flip_x: f32 = -1.0;
    let flip_y: f32 = -1.0;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        // Keep pure semicolon comments for thread metadata.
        if let Some(comment) = line.strip_prefix(';') {
            let comment = comment.trim();
            if comment.to_ascii_lowercase().starts_with("thread") {
                let mut thread = EmbThread::new(0x000000);
                thread.description = Some(comment.to_string());
                pattern.add_thread(thread);
            }
            continue;
        }

        // Remove inline comments and parenthetical comments.
        let mut cleaned = line;
        if let Some(idx) = cleaned.find(';') {
            cleaned = &cleaned[..idx];
        }
        if let Some(start) = cleaned.find('(') {
            if let Some(end_rel) = cleaned[start..].find(')') {
                let end = start + end_rel + 1;
                let mut s = String::new();
                s.push_str(&cleaned[..start]);
                s.push(' ');
                s.push_str(&cleaned[end..]);
                let parsed = parse_words(&s);
                if parsed.is_empty() {
                    continue;
                }
                process_words(
                    &mut pattern,
                    &parsed,
                    &mut absolute_mode,
                    &mut scale,
                    flip_x,
                    flip_y,
                );
                continue;
            }
        }

        let words = parse_words(cleaned);
        if words.is_empty() {
            continue;
        }

        process_words(
            &mut pattern,
            &words,
            &mut absolute_mode,
            &mut scale,
            flip_x,
            flip_y,
        );
    }

    if pattern
        .stitches
        .last()
        .map(|s| s.stitch_type != StitchType::End)
        .unwrap_or(true)
    {
        let (end_x, end_y) = current_position(&pattern);
        pattern.add_stitch_absolute(StitchType::End, end_x, end_y);
    }

    Ok(pattern)
}

fn process_words(
    pattern: &mut EmbPattern,
    words: &[(char, f32)],
    absolute_mode: &mut bool,
    scale: &mut f32,
    flip_x: f32,
    flip_y: f32,
) {
    let mut g: Option<f32> = None;
    let mut m: Option<f32> = None;
    let mut x: Option<f32> = None;
    let mut y: Option<f32> = None;

    for (k, v) in words {
        match *k {
            'g' => g = Some(*v),
            'm' => m = Some(*v),
            'x' => x = Some(*v),
            'y' => y = Some(*v),
            _ => {}
        }
    }

    if let Some(gv) = g {
        let gi = gv.round() as i32;
        match gi {
            20 | 70 => {
                *scale = 254.0; // inch -> tenths of mm
            }
            21 | 71 => {
                *scale = 10.0; // mm -> tenths of mm
            }
            90 => {
                *absolute_mode = true;
            }
            91 => {
                *absolute_mode = false;
            }
            0 | 1 => {
                if let (Some(xv), Some(yv)) = (x, y) {
                    let dx = xv * *scale * flip_x;
                    let dy = yv * *scale * flip_y;
                    if *absolute_mode {
                        pattern.add_stitch_absolute(StitchType::Stitch, dx, dy);
                    } else {
                        pattern.add_stitch_relative(StitchType::Stitch, dx, dy);
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(mv) = m {
        let mi = mv.round() as i32;
        match mi {
            30 | 2 => {
                let (x, y) = current_position(pattern);
                pattern.add_stitch_absolute(StitchType::End, x, y);
            }
            0 | 1 => {
                add_command_here(pattern, StitchType::ColorChange);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_bean_gcode() {
        let path = "tests/testdata/Not Mandatory/Bean.gcode";
        let data = fs::read(path).expect("Failed to read test GCODE file");
        let pattern = GcodeReader.read(&data).expect("Failed to parse GCODE file");

        assert!(pattern.stitches.len() > 10, "expected parsed GCODE stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero GCODE coordinates"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
