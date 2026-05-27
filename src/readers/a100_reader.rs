use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

pub struct A100Reader;

impl EmbroideryReader for A100Reader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_100(data)
    }
}

fn current_position(pattern: &EmbPattern) -> (f32, f32) {
    pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0))
}

fn read_100(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::new(0x000000));

    let mut i = 0usize;
    while i + 3 < data.len() {
        let b0 = data[i];
        let _b1 = data[i + 1];
        let mut x = data[i + 2] as i32;
        let mut y = data[i + 3] as i32;

        if x > 0x80 {
            x -= 0x80;
            x = -x;
        }
        if y > 0x80 {
            y -= 0x80;
            y = -y;
        }

        if b0 == 0x61 {
            pattern.add_stitch_relative(StitchType::Stitch, x as f32, -(y as f32));
        } else if (b0 & 0x01) != 0 {
            pattern.add_stitch_relative(StitchType::Jump, x as f32, -(y as f32));
        } else {
            let (cx, cy) = current_position(&pattern);
            pattern.add_stitch_absolute(StitchType::ColorChange, cx, cy);
        }

        i += 4;
    }

    let (end_x, end_y) = current_position(&pattern);
    pattern.add_stitch_absolute(StitchType::End, end_x, end_y);
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_synthetic_100() {
        // 0x61 => stitch by (+5, -3)
        // 0x01 => jump by (+2, +4)
        // 0x00 => color-change command
        let data = vec![
            0x61, 0x00, 0x05, 0x03, // stitch
            0x01, 0x00, 0x02, 0xFC, // jump (y encoded to produce +4 after inversion)
            0x00, 0x00, 0x00, 0x00, // color change
        ];

        let pattern = A100Reader.read(&data).expect("Failed to parse synthetic .100");

        assert!(pattern.stitches.len() >= 4, "expected commands plus end");
        assert_eq!(
            pattern.stitches[0].stitch_type,
            StitchType::Stitch,
            "first command should be stitch"
        );
        assert!(
            pattern.stitches.iter().any(|s| s.stitch_type == StitchType::Jump),
            "expected jump command"
        );
        assert!(
            pattern
                .stitches
                .iter()
                .any(|s| s.stitch_type == StitchType::ColorChange),
            "expected color change command"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
