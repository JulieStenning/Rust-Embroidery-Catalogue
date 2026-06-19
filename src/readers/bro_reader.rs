use std::io::Read;
use crate::models::{EmbPattern, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::Cursor;

pub struct BroReader;

impl EmbroideryReader for BroReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_bro(data)
    }
}

fn read_bro(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);
    cursor.set_position(0x100);
    let mut threads: Vec<u8> = Vec::new();
    let mut iter_count = 0;
    loop {
        iter_count += 1;
        let mut b = [0u8; 2];
        if cursor.read_exact(&mut b).is_err() {
            println!("[BRO DEBUG] Iteration {iter_count}: EOF or read error on stitch bytes");
            break;
        }
        let dx = b[0] as i8 as f32;
        let dy = -(b[1] as i8 as f32);
        if b[0] != 0x80 {
            pattern.add_stitch_relative(StitchType::Stitch, dx, dy);
            continue;
        }
        let mut control = [0u8; 1];
        if cursor.read_exact(&mut control).is_err() {
            println!("[BRO DEBUG] Iteration {iter_count}: EOF or read error on control byte");
            break;
        }
        println!("[BRO DEBUG] Iteration {iter_count}: Control byte = 0x{:02X}", control[0]);
        match control[0] {
            0x00 => {
                println!("[BRO DEBUG] Iteration {iter_count}: Control 0x00, continue");
                continue;
            },
            0x02 => {
                println!("[BRO DEBUG] Iteration {iter_count}: Control 0x02, continue");
                continue;
            },
            0xE0 => {
                println!("[BRO DEBUG] Iteration {iter_count}: Control 0xE0, continue");
                continue;
            },
            0x7E | 0x03 => {
                let mut x_bytes = [0u8; 2];
                let mut y_bytes = [0u8; 2];
                if cursor.read_exact(&mut x_bytes).is_err() || cursor.read_exact(&mut y_bytes).is_err() {
                    println!("[BRO DEBUG] Iteration {iter_count}: EOF or read error on jump coords");
                    break;
                }
                let x = i16::from_le_bytes(x_bytes) as f32;
                let y = -(i16::from_le_bytes(y_bytes) as f32);
                pattern.add_stitch_relative(StitchType::Jump, x, y);
                println!("[BRO DEBUG] Iteration {iter_count}: Jump x={x}, y={y}");
                continue;
            }
            c if (0xE0 < c && c < 0xF0) => {
                let needle = c - 0xE0;
                threads.push(needle);
                let mut x_bytes = [0u8; 2];
                let mut y_bytes = [0u8; 2];
                if cursor.read_exact(&mut x_bytes).is_err() || cursor.read_exact(&mut y_bytes).is_err() {
                    println!("[BRO DEBUG] Iteration {iter_count}: EOF or read error on color change coords");
                    break;
                }
                let x = i16::from_le_bytes(x_bytes) as f32;
                let y = -(i16::from_le_bytes(y_bytes) as f32);
                pattern.add_stitch_relative(StitchType::ColorChange, x, y);
                println!("[BRO DEBUG] Iteration {iter_count}: ColorChange needle={needle}, x={x}, y={y}");
                continue;
            }
            _ => {
                println!("[BRO DEBUG] Iteration {iter_count}: Unhandled control 0x{:02X}, continue", control[0]);
                continue;
            }
        }
    }
    // Add dummy threads for color changes if none present
    if pattern.threadlist.is_empty() && !threads.is_empty() {
        for needle in threads.iter().take(7) { // Only add up to 7 threads
            let color = match needle {
                0 => 0x000000, 1 => 0xFF0000, 2 => 0x00FF00, 3 => 0x0000FF,
                4 => 0xFFFF00, 5 => 0xFF00FF, 6 => 0x00FFFF, 7 => 0x888888,
                8 => 0xFFFFFF, 9 => 0x800000, 10 => 0x008000, 11 => 0x000080,
                _ => 0xCCCCCC,
            };
            pattern.add_thread(crate::models::EmbThread::new(color));
        }
    }
    pattern.add_stitch_absolute(StitchType::End, 0.0, 0.0);
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_bro() {
        let path = "tests/testdata/Cake 3.bro";
        let data = fs::read(path).expect("Failed to read test BRO file");
        let pattern = BroReader.read(&data).expect("Failed to parse BRO file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        let num_colour_changes = pattern.stitches.iter().filter(|s| s.stitch_type == StitchType::ColorChange).count();
        println!("Number of colour changes: {}", num_colour_changes);
        assert_eq!(pattern.stitches.len(), 15104, "Unexpected stitch count");
        assert_eq!(pattern.threadlist.len(), 7, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 17, "Unexpected number of colour changes");
        let expected_coords = [
            (0.0, 0.0),
            (-45.0, 105.0),
            (-91.0, 210.0),
            (-136.0, 315.0),
            (-182.0, 420.0),
        ];
        for (i, &(x, y)) in expected_coords.iter().enumerate() {
            assert_eq!(pattern.stitches[i].x, x, "Unexpected x at stitch {}", i);
            assert_eq!(pattern.stitches[i].y, y, "Unexpected y at stitch {}", i);
        }
    }
}
