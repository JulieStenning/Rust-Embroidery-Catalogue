use std::cmp::min;
use std::io::{Cursor, Read};

use crate::models::{EmbPattern, EmbThread, StitchType};
use crate::readers::embroidery_reader::EmbroideryReader;

pub struct HusReader;

impl EmbroideryReader for HusReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_hus(data).map_err(|e| e.into())
    }
}

#[derive(Clone)]
struct Huffman {
    default_value: usize,
    lengths: Option<Vec<usize>>,
    table: Option<Vec<usize>>,
    table_width: usize,
}

impl Huffman {
    fn with_default(value: usize) -> Self {
        Self {
            default_value: value,
            lengths: None,
            table: None,
            table_width: 0,
        }
    }

    fn with_lengths(lengths: Vec<usize>) -> Result<Self, String> {
        if lengths.is_empty() {
            return Err("empty Huffman length table".to_string());
        }

        let table_width = *lengths
            .iter()
            .max()
            .ok_or_else(|| "empty Huffman length table".to_string())?;
        if table_width == 0 || table_width > 16 {
            return Err(format!("unsupported Huffman width: {table_width}"));
        }

        let mut table = Vec::new();
        let mut size = 1usize << table_width;
        for bit_length in 1..=table_width {
            size >>= 1;
            for (idx, length) in lengths.iter().enumerate() {
                if *length == bit_length {
                    for _ in 0..size {
                        table.push(idx);
                    }
                }
            }
        }

        if table.is_empty() {
            return Err("failed to build Huffman table".to_string());
        }

        Ok(Self {
            default_value: 0,
            lengths: Some(lengths),
            table: Some(table),
            table_width,
        })
    }

    fn lookup(&self, lookup: u16) -> Result<(usize, usize), String> {
        if self.table.is_none() {
            return Ok((self.default_value, 0));
        }

        let table = self
            .table
            .as_ref()
            .ok_or_else(|| "missing Huffman table".to_string())?;
        let lengths = self
            .lengths
            .as_ref()
            .ok_or_else(|| "missing Huffman lengths".to_string())?;

        let idx = (lookup as usize) >> (16 - self.table_width);
        let value = *table
            .get(idx)
            .ok_or_else(|| "Huffman lookup out of bounds".to_string())?;
        let len = *lengths
            .get(value)
            .ok_or_else(|| "Huffman length index out of bounds".to_string())?;
        Ok((value, len))
    }
}

struct EmbCompress {
    bit_position: usize,
    input_data: Vec<u8>,
    block_elements: isize,
    character_huffman: Option<Huffman>,
    distance_huffman: Option<Huffman>,
}

impl EmbCompress {
    fn new(input_data: Vec<u8>) -> Self {
        Self {
            bit_position: 0,
            input_data,
            block_elements: -1,
            character_huffman: None,
            distance_huffman: None,
        }
    }

    fn get_bits(&self, start_pos_bits: usize, length: usize) -> u32 {
        let end_pos_bits = start_pos_bits + length - 1;
        let start_pos_bytes = start_pos_bits / 8;
        let end_pos_bytes = end_pos_bits / 8;

        let mut value: u32 = 0;
        for i in start_pos_bytes..=end_pos_bytes {
            value <<= 8;
            if let Some(b) = self.input_data.get(i) {
                value |= u32::from(*b);
            }
        }

        let unused_bits = (8 - ((end_pos_bits + 1) % 8)) % 8;
        let mask = (1u32 << length) - 1;
        (value >> unused_bits) & mask
    }

    fn peek(&self, bit_count: usize) -> u32 {
        self.get_bits(self.bit_position, bit_count)
    }

    fn slide(&mut self, bit_count: usize) {
        self.bit_position += bit_count;
    }

    fn pop(&mut self, bit_count: usize) -> u32 {
        let v = self.peek(bit_count);
        self.slide(bit_count);
        v
    }

    fn read_variable_length(&mut self) -> usize {
        let mut m = self.pop(3) as usize;
        if m != 7 {
            return m;
        }

        for _ in 0..13 {
            let s = self.pop(1);
            if s == 1 {
                m += 1;
            } else {
                break;
            }
        }
        m
    }

    fn load_character_length_huffman(&mut self) -> Result<Huffman, String> {
        let count = self.pop(5) as usize;
        if count == 0 {
            return Ok(Huffman::with_default(self.pop(5) as usize));
        }

        let mut lengths = vec![0usize; count];
        let mut index = 0usize;
        while index < count {
            if index == 3 {
                index += self.pop(2) as usize;
                if index >= count {
                    break;
                }
            }
            lengths[index] = self.read_variable_length();
            index += 1;
        }

        Huffman::with_lengths(lengths)
    }

    fn load_character_huffman(&mut self, length_huffman: &Huffman) -> Result<Huffman, String> {
        let count = self.pop(9) as usize;
        if count == 0 {
            return Ok(Huffman::with_default(self.pop(9) as usize));
        }

        let mut lengths = vec![0usize; count];
        let mut index = 0usize;
        while index < count {
            let (mut c, used_bits) = length_huffman.lookup(self.peek(16) as u16)?;
            self.slide(used_bits);

            if c == 0 {
                index += 1;
            } else if c == 1 {
                index += 3 + self.pop(4) as usize;
            } else if c == 2 {
                index += 20 + self.pop(9) as usize;
            } else {
                c -= 2;
                if index >= count {
                    break;
                }
                lengths[index] = c;
                index += 1;
            }
        }

        Huffman::with_lengths(lengths)
    }

    fn load_distance_huffman(&mut self) -> Result<Huffman, String> {
        let count = self.pop(5) as usize;
        if count == 0 {
            return Ok(Huffman::with_default(self.pop(5) as usize));
        }

        let mut lengths = vec![0usize; count];
        for entry in &mut lengths {
            *entry = self.read_variable_length();
        }

        Huffman::with_lengths(lengths)
    }

    fn load_block(&mut self) -> Result<(), String> {
        self.block_elements = self.pop(16) as isize;
        let character_length_huffman = self.load_character_length_huffman()?;
        self.character_huffman = Some(self.load_character_huffman(&character_length_huffman)?);
        self.distance_huffman = Some(self.load_distance_huffman()?);
        Ok(())
    }

    fn get_token(&mut self) -> Result<usize, String> {
        if self.block_elements <= 0 {
            self.load_block()?;
        }
        self.block_elements -= 1;

        let character_huffman = self
            .character_huffman
            .as_ref()
            .ok_or_else(|| "missing character Huffman".to_string())?;
        let (token, used_bits) = character_huffman.lookup(self.peek(16) as u16)?;
        self.slide(used_bits);
        Ok(token)
    }

    fn get_position(&mut self) -> Result<usize, String> {
        let distance_huffman = self
            .distance_huffman
            .as_ref()
            .ok_or_else(|| "missing distance Huffman".to_string())?;
        let (value, used_bits) = distance_huffman.lookup(self.peek(16) as u16)?;
        self.slide(used_bits);
        if value == 0 {
            return Ok(0);
        }

        let v = value - 1;
        Ok((1usize << v) + self.pop(v) as usize)
    }

    fn decompress(&mut self, uncompressed_size: Option<usize>) -> Result<Vec<u8>, String> {
        let bits_total = self.input_data.len() * 8;
        let mut out = Vec::new();

        while bits_total > self.bit_position
            && (uncompressed_size.is_none() || out.len() < uncompressed_size.unwrap_or(0))
        {
            let character = self.get_token()?;
            if character <= 255 {
                out.push(character as u8);
                continue;
            }

            if character == 510 {
                break;
            }

            let length = character - 253;
            let back = self.get_position()? + 1;
            if back > out.len() {
                return Err("compressed stream lookback exceeds output".to_string());
            }

            let position = out.len() - back;
            if back > length {
                let end = position + length;
                if end > out.len() {
                    return Err("compressed stream copy range exceeds output".to_string());
                }
                out.extend_from_within(position..end);
            } else {
                for i in position..(position + length) {
                    let b = *out.get(i).ok_or_else(|| {
                        "compressed stream overlap copy out of bounds".to_string()
                    })?;
                    out.push(b);
                }
            }
        }

        Ok(out)
    }
}

fn expand(data: &[u8], uncompressed_size: Option<usize>) -> Result<Vec<u8>, String> {
    let mut decoder = EmbCompress::new(data.to_vec());
    decoder.decompress(uncompressed_size)
}

fn read_u16_le(cursor: &mut Cursor<&[u8]>) -> Result<u16, String> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| format!("unexpected EOF while reading u16: {e}"))?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, String> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| format!("unexpected EOF while reading u32: {e}"))?;
    Ok(u32::from_le_bytes(buf))
}

fn parse_color(hex: &str) -> u32 {
    u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0)
}

fn hus_thread_set() -> Vec<EmbThread> {
    let make = |hex: &str, description: &str, catalog: &str| EmbThread {
        color: parse_color(hex),
        description: Some(description.to_string()),
        catalog_number: Some(catalog.to_string()),
        details: None,
        brand: Some("Hus".to_string()),
        chart: Some("Hus".to_string()),
        weight: None,
    };

    vec![
        make("#000000", "Black", "026"),
        make("#0000e7", "Blue", "005"),
        make("#00c600", "Green", "002"),
        make("#ff0000", "Red", "014"),
        make("#840084", "Purple", "008"),
        make("#ffff00", "Yellow", "020"),
        make("#848484", "Grey", "024"),
        make("#8484e7", "Light Blue", "006"),
        make("#00ff84", "Light Green", "003"),
        make("#ff7b31", "Orange", "017"),
        make("#ff8ca5", "Pink", "011"),
        make("#845200", "Brown", "028"),
        make("#ffffff", "White", "022"),
        make("#000084", "Dark Blue", "004"),
        make("#008400", "Dark Green", "001"),
        make("#7b0000", "Dark Red", "013"),
        make("#ff6384", "Light Red", "015"),
        make("#522952", "Dark Purple", "007"),
        make("#ff00ff", "Light Purple", "009"),
        make("#ffde00", "Dark Yellow", "019"),
        make("#ffff9c", "Light Yellow", "021"),
        make("#525252", "Dark Grey", "025"),
        make("#d6d6d6", "Light Grey", "023"),
        make("#ff5208", "Dark Orange", "016"),
        make("#ff9c5a", "Light Orange", "018"),
        make("#ff52b5", "Dark Pink", "010"),
        make("#ffc6de", "Light Pink", "012"),
        make("#523100", "Dark Brown", "027"),
        make("#b5a584", "Light Brown", "029"),
    ]
}

fn apply_hus_command(pattern: &mut EmbPattern, cmd: u8, x: f32, y: f32) -> bool {
    match cmd {
        0x80 => pattern.add_stitch_relative(StitchType::Stitch, x, y),
        0x81 => pattern.add_stitch_relative(StitchType::Jump, x, y),
        0x84 => pattern.add_stitch_relative(StitchType::ColorChange, x, y),
        0x88 => {
            if x != 0.0 || y != 0.0 {
                pattern.add_stitch_relative(StitchType::Jump, x, y);
            }
            pattern.add_stitch_relative(StitchType::Trim, 0.0, 0.0);
        }
        0x90 => return false,
        _ => {
            // Ignore unknown commands or flag bytes instead of dropping the loop early.
            return true;
        }
    }

    true
}

pub fn read_hus(data: &[u8]) -> Result<EmbPattern, String> {
    let mut cursor = Cursor::new(data);
    let mut pattern = EmbPattern::new();

    let _magic_code = read_u32_le(&mut cursor)?;
    let number_of_stitches = read_u32_le(&mut cursor)? as usize;
    let number_of_colors = read_u32_le(&mut cursor)? as usize;

    let _extend_pos_x = read_u16_le(&mut cursor)? as i16;
    let _extend_pos_y = read_u16_le(&mut cursor)? as i16;
    let _extend_neg_x = read_u16_le(&mut cursor)? as i16;
    let _extend_neg_y = read_u16_le(&mut cursor)? as i16;

    let command_offset = read_u32_le(&mut cursor)? as usize;
    let x_offset = read_u32_le(&mut cursor)? as usize;
    let y_offset = read_u32_le(&mut cursor)? as usize;

    let mut file_label = [0u8; 8];
    cursor
        .read_exact(&mut file_label)
        .map_err(|e| format!("unexpected EOF while reading HUS label: {e}"))?;
    let label = String::from_utf8_lossy(&file_label)
        .trim_matches(char::from(0))
        .trim()
        .to_string();
    if !label.is_empty() {
        pattern.extras.insert("label".to_string(), label);
    }

    let _unknown_16_bit = read_u16_le(&mut cursor)?;

    let thread_set = hus_thread_set();
    for _ in 0..number_of_colors {
        let idx = read_u16_le(&mut cursor)? as usize;
        let thread = thread_set
            .get(idx)
            .cloned()
            .unwrap_or_else(|| EmbThread::new(0x000000));
        pattern.add_thread(thread);
    }

    if !(command_offset <= x_offset && x_offset <= y_offset && y_offset <= data.len()) {
        return Err("invalid HUS compressed section offsets".to_string());
    }
    let cmd_len = x_offset - command_offset;
    let x_len = y_offset - x_offset;

    let command_compressed = &data[command_offset..(command_offset + cmd_len)];
    let x_compressed = &data[x_offset..(x_offset + x_len)];
    let y_compressed = &data[y_offset..data.len()];

    let command_decompressed = expand(command_compressed, Some(number_of_stitches))?;
    let x_decompressed = expand(x_compressed, Some(number_of_stitches))?;
    let y_decompressed = expand(y_compressed, Some(number_of_stitches))?;

    let stitch_count = min(
        number_of_stitches,
        min(
            command_decompressed.len(),
            min(x_decompressed.len(), y_decompressed.len()),
        ),
    );

    for i in 0..stitch_count {
        let cmd = command_decompressed[i];
        let x = (x_decompressed[i] as i8) as f32;
        let y = -((y_decompressed[i] as i8) as f32);

        if !apply_hus_command(&mut pattern, cmd, x, y) {
            break;
        }

        if !matches!(cmd, 0x80 | 0x81 | 0x84 | 0x88 | 0x90) {
            eprintln!("Warning: Encountered unknown stitch command byte: {cmd:#X} at index {i}");
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
    use std::fs;

    use super::*;

    #[test]
    fn read_hus_fixture_produces_stitches_threads_and_end() {
        let path = "tests/testdata/Bean.hus";
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
        let path = "tests/testdata/Cake 3.hus";
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
    fn hus_jump_command_maps_to_jump() {
        let mut pattern = EmbPattern::new();

        let keep_parsing = apply_hus_command(&mut pattern, 0x81, 5.0, -7.0);

        assert!(keep_parsing);
        assert_eq!(pattern.count_stitch_commands(StitchType::Jump), 1);
        assert_eq!(pattern.count_stitch_commands(StitchType::Stitch), 0);
        assert_eq!(pattern.stitches[0].x, 5.0);
        assert_eq!(pattern.stitches[0].y, -7.0);
    }
}
