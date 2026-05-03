pub mod readers;
pub mod models;

use std::fs::{self, File};
use std::io::Write;
use crate::models::EmbPattern;
use crate::readers::*;

fn main() {
    let testdata_dir = "tests/testdata";
    let out_path = "rust.csv";
    let mut wtr = File::create(out_path).expect("Failed to create rust.csv");
    writeln!(wtr, "format,stitch_count,width_mm,height_mm,begin_x,begin_y,end_x,end_y,thread_changes,colour_changes").unwrap();

    let entries = fs::read_dir(testdata_dir).expect("Failed to read testdata dir");
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() { continue; }
        let fname = path.file_name().unwrap().to_string_lossy();
        if !fname.starts_with("Bean.") { continue; }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();

        // Dynamically map extension to XxxReader struct (e.g., pes -> PesReader)
        let reader: Option<Box<dyn EmbroideryReader>> = match ext.as_str() {
            "dst" => Some(Box::new(DstReader)),
            "exp" => Some(Box::new(ExpReader)),
            "jef" => Some(Box::new(JefReader)),
            "pes" => Some(Box::new(PesReader)),
            "vp3" => Some(Box::new(Vp3Reader)),
            _ => None,
        };
        if let Some(reader) = reader {
            let data = fs::read(&path).expect("Failed to read file");
            match reader.read(&data) {
                Ok(pattern) => write_report_row(&mut wtr, &ext, &pattern),
                Err(e) => {
                    eprintln!("ERROR reading {}: {}", path.display(), e);
                    writeln!(wtr, "{},{},ERROR,ERROR,ERROR,ERROR,ERROR,ERROR,ERROR", ext, fname).unwrap();
                }
            }
        }
        // Files without readers are skipped (no row added to CSV)
    }
}

fn write_report_row(wtr: &mut File, ext: &str, pattern: &EmbPattern) {
    let stitch_count = pattern.count_stitches();
    let (min_x, min_y, max_x, max_y) = pattern.bounds();
    let width = (max_x - min_x).abs();
    let height = (max_y - min_y).abs();
    let begin = pattern.stitches.first().map(|s| (s.x, s.y)).unwrap_or((0.0, 0.0));
    let end = pattern.stitches.last().map(|s| (s.x, s.y)).unwrap_or((0.0, 0.0));
    let thread_changes = pattern.count_threads();
    let colour_changes = pattern.count_color_changes();
    writeln!(wtr, "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{}",
        ext,
        stitch_count,
        width,
        height,
        begin.0,
        begin.1,
        end.0,
        end.1,
        thread_changes,
        colour_changes
    ).unwrap();
}
