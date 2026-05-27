use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

pub struct PmvReader;

impl EmbroideryReader for PmvReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_pmv(data)
    }
}

fn read_u16_le(data: &[u8], index: &mut usize) -> Option<u16> {
    if *index + 1 >= data.len() {
        return None;
    }
    let value = u16::from_le_bytes([data[*index], data[*index + 1]]);
    *index += 2;
    Some(value)
}

fn read_u8(data: &[u8], index: &mut usize) -> Option<u8> {
    if *index >= data.len() {
        return None;
    }
    let value = data[*index];
    *index += 1;
    Some(value)
}

fn read_pmv(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::new(0x000000));

    if data.len() <= 0x64 {
        pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);
        return Ok(pattern);
    }

    let mut idx = 0x64usize;
    let mut px: f32 = 0.0;

    loop {
        let stitch_count = match read_u16_le(data, &mut idx) {
            Some(v) => v as usize,
            None => break,
        };
        let block_length = match read_u16_le(data, &mut idx) {
            Some(v) => v,
            None => break,
        };

        if block_length >= 256 {
            break;
        }
        if stitch_count == 0 {
            continue;
        }

        for _ in 0..stitch_count {
            let mut x = match read_u8(data, &mut idx) {
                Some(v) => v as i32,
                None => break,
            };
            let mut y = match read_u8(data, &mut idx) {
                Some(v) => v as i32,
                None => break,
            };

            if y > 16 {
                y = -(32 - y);
            }
            if x > 32 {
                x = -(64 - x);
            }

            let xf = x as f32 * 2.5;
            let yf = y as f32 * -2.5;
            pattern.add_stitch_absolute(StitchType::Stitch, px + xf, yf);
            px += xf;
        }
    }

    let (end_x, end_y) = pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0));
    pattern.add_stitch_absolute(StitchType::End, end_x, end_y);

    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_synthetic_pmv() {
        let mut data = vec![0u8; 0x64];
        // block: 2 stitches, small block length (<256)
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&8u16.to_le_bytes());
        // stitch 1: x=2, y=3
        data.push(2);
        data.push(3);
        // stitch 2: x=4, y=5
        data.push(4);
        data.push(5);
        // terminator block with length >= 256
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&256u16.to_le_bytes());

        let pattern = PmvReader.read(&data).expect("Failed to parse synthetic PMV");

        assert!(pattern.stitches.len() >= 3, "expected stitches plus End");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero PMV coordinates"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
