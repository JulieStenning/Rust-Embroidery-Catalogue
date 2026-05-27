use crate::models::{EmbPattern, EmbThread};
use crate::readers::embroidery_reader::EmbroideryReader;
use std::io::{Cursor, BufRead, BufReader};

pub struct ColReader;

impl EmbroideryReader for ColReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
        read_col(data)
    }
}

fn read_col(data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    let cursor = Cursor::new(data);
    let mut reader = BufReader::new(cursor);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let count: usize = line.trim().parse()?;
    for _ in 0..count {
        line.clear();
        reader.read_line(&mut line)?;
        let splits: Vec<&str> = line.trim().split(',').collect();
        if splits.len() < 4 { continue; }
        let catalog_number = splits[0].to_string();
        let r: u8 = splits[1].parse()?;
        let g: u8 = splits[2].parse()?;
        let b: u8 = splits[3].parse()?;
        let mut thread = EmbThread::new(((r as u32) << 16) | ((g as u32) << 8) | (b as u32));
        thread.catalog_number = Some(catalog_number);
        pattern.add_thread(thread);
    }
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_cake3_col() {
        let path = "tests/testdata/Cake 3.col";
        let data = fs::read(path).expect("Failed to read test COL file");
        let pattern = ColReader.read(&data).expect("Failed to parse COL file");
        println!("Stitch count: {}", pattern.stitches.len());
        println!("Number of colours: {}", pattern.threadlist.len());
        let num_colour_changes = pattern.count_color_changes();
        println!("Number of colour changes: {}", num_colour_changes);
        for (i, thread) in pattern.threadlist.iter().take(5).enumerate() {
            println!("Thread {}: color = #{:06x}", i, thread.color);
        }
        assert_eq!(pattern.stitches.len(), 0, "COL files do not contain stitches");
        assert_eq!(pattern.threadlist.len(), 19, "Unexpected number of colours");
        assert_eq!(num_colour_changes, 0, "COL files do not contain color changes");
    }
}
