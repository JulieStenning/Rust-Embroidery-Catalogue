use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct PhbReader;

impl EmbroideryReader for PhbReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_phb(data)
    }
}

const FLAG_LONG: u8 = 0x80;
const JUMP_CODE: u8 = 0x10;
const TRIM_CODE: u8 = 0x20;

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, std::io::Error> {
    let mut b = [0u8; 1];
    cursor.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_u16_le(cursor: &mut Cursor<&[u8]>) -> Result<u16, std::io::Error> {
    let mut b = [0u8; 2];
    cursor.read_exact(&mut b)?;
    Ok(u16::from_le_bytes(b))
}

fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
    let mut b = [0u8; 4];
    cursor.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn signed12(b: u16) -> i32 {
    let b = b & 0x0FFF;
    if b > 0x07FF {
        -0x1000 + b as i32
    } else {
        b as i32
    }
}

fn signed7(b: u8) -> i32 {
    if b > 63 {
        -128 + b as i32
    } else {
        b as i32
    }
}

fn thread_color_from_index(idx: u8) -> u32 {
    const PALETTE: [u32; 16] = [
        0x000000, 0x000080, 0x0000FF, 0x008080, 0x00FFFF, 0x800080, 0xFF00FF, 0x800000,
        0xFF0000, 0x008000, 0x00FF00, 0x808000, 0xFFFF00, 0x808080, 0xC0C0C0, 0xFFFFFF,
    ];
    PALETTE[idx as usize % PALETTE.len()]
}

fn read_pec_stitches(cursor: &mut Cursor<&[u8]>, pattern: &mut EmbPattern) -> Result<(), std::io::Error> {
    loop {
        let val1 = match read_u8(cursor) {
            Ok(v) => v,
            Err(_) => break,
        };
        let val2 = match read_u8(cursor) {
            Ok(v) => v,
            Err(_) => break,
        };

        if val1 == 0xFF && val2 == 0x00 {
            break;
        }

        if val1 == 0xFE && val2 == 0xB0 {
            cursor.seek(SeekFrom::Current(1))?;
            pattern.add_stitch_absolute(StitchType::ColorChange, 0.0, 0.0);
            continue;
        }

        let mut jump = false;
        let mut trim = false;
        let x: i32;
        let y: i32;

        if val1 & FLAG_LONG != 0 {
            if val1 & TRIM_CODE != 0 {
                trim = true;
            }
            if val1 & JUMP_CODE != 0 {
                jump = true;
            }
            let code = ((val1 as u16) << 8) | (val2 as u16);
            x = signed12(code);
            let _ = read_u8(cursor)?;
        } else {
            x = signed7(val1);
        }

        let y_byte1 = if val1 & FLAG_LONG != 0 { read_u8(cursor)? } else { val2 };
        if y_byte1 & FLAG_LONG != 0 {
            if y_byte1 & TRIM_CODE != 0 {
                trim = true;
            }
            if y_byte1 & JUMP_CODE != 0 {
                jump = true;
            }
            let y_byte2 = read_u8(cursor)?;
            let code = ((y_byte1 as u16) << 8) | (y_byte2 as u16);
            y = signed12(code);
        } else {
            y = signed7(y_byte1);
        }

        if jump {
            pattern.add_stitch_relative(StitchType::Jump, x as f32, y as f32);
        } else if trim {
            pattern.add_stitch_absolute(StitchType::Trim, 0.0, 0.0);
            pattern.add_stitch_relative(StitchType::Jump, x as f32, y as f32);
        } else {
            pattern.add_stitch_relative(StitchType::Stitch, x as f32, y as f32);
        }
    }

    let (end_x, end_y) = pattern
        .stitches
        .last()
        .map(|s| (s.x, s.y))
        .unwrap_or((0.0, 0.0));
    pattern.add_stitch_absolute(StitchType::End, end_x, end_y);
    Ok(())
}

fn read_phb(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let mut cursor = Cursor::new(data);

    cursor.seek(SeekFrom::Start(0x71))?;
    let color_count = read_u16_le(&mut cursor)? as usize;
    for _ in 0..color_count {
        let idx = read_u8(&mut cursor)?;
        pattern.add_thread(EmbThread::new(thread_color_from_index(idx)));
    }

    let mut file_offset: u64 = 0x52;
    cursor.seek(SeekFrom::Start(0x54))?;
    file_offset += read_u32_le(&mut cursor)? as u64;

    cursor.seek(SeekFrom::Start(file_offset))?;
    file_offset += read_u32_le(&mut cursor)? as u64 + 2;

    cursor.seek(SeekFrom::Start(file_offset))?;
    file_offset += read_u32_le(&mut cursor)? as u64;

    cursor.seek(SeekFrom::Start(file_offset + 14))?;
    let color_count2 = read_u8(&mut cursor)? as i64;
    cursor.seek(SeekFrom::Current(color_count2 + 0x15))?;

    read_pec_stitches(&mut cursor, &mut pattern)?;

    if pattern.threadlist.is_empty() {
        pattern.add_thread(EmbThread::new(0x000000));
    }

    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_phb() {
        let path = "tests/testdata/Not Mandatory/Bean.phb";
        let data = fs::read(path).expect("Failed to read test PHB file");
        let pattern = PhbReader.read(&data).expect("Failed to parse PHB file");

        assert!(pattern.stitches.len() > 10, "expected parsed PHB stitches");
        assert!(
            pattern.stitches.iter().any(|s| s.x != 0.0 || s.y != 0.0),
            "expected non-zero PHB coordinates"
        );
        assert_eq!(
            pattern.stitches.last().map(|s| s.stitch_type),
            Some(StitchType::End),
            "expected terminal End command"
        );
    }
}
