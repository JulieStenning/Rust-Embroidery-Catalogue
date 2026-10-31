// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use std::fs::{self, File};
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{}_{}", prefix, now));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_compress_creates_valid_zip_with_all_files() {
    let src_path = unique_temp_dir("alphabet_test_src");
    let dest_path = unique_temp_dir("alphabet_test_dest");

    // Create sample alphabet files
    fs::write(src_path.join("Upper_A.pes"), b"dummy pes A content").unwrap();
    fs::write(src_path.join("Upper_B.pes"), b"dummy pes B content").unwrap();
    fs::write(src_path.join("Font.bx"), b"dummy embrilliance bx").unwrap();

    // Create a subfolder for size
    let sub_dir = src_path.join("1.5inch");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join("Lower_a.pes"), b"dummy sub lower a").unwrap();

    let result = compress_or_copy_alphabet(&src_path, &dest_path, None);
    assert!(result.is_ok(), "Compression failed: {:?}", result.err());

    let zip_path = result.unwrap();
    assert!(zip_path.exists());
    assert_eq!(zip_path.extension().and_then(|e| e.to_str()), Some("zip"));

    // Verify zip contents
    let file = File::open(&zip_path).unwrap();
    let archive = zip::ZipArchive::new(file).unwrap();

    let file_names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
    assert!(file_names.contains(&"Upper_A.pes".to_string()));
    assert!(file_names.contains(&"Upper_B.pes".to_string()));
    assert!(file_names.contains(&"Font.bx".to_string()));
    assert!(file_names.contains(&"1.5inch/Lower_a.pes".to_string()));

    // Verify source files remain intact (read-only invariant)
    assert!(src_path.join("Upper_A.pes").exists());
    assert!(src_path.join("Upper_B.pes").exists());
    assert!(src_path.join("Font.bx").exists());
    assert!(sub_dir.join("Lower_a.pes").exists());

    let _ = fs::remove_dir_all(&src_path);
    let _ = fs::remove_dir_all(&dest_path);
}

#[test]
fn test_copies_existing_companion_zip() {
    let src_path = unique_temp_dir("alphabet_existing_zip_src");
    let dest_path = unique_temp_dir("alphabet_existing_zip_dest");

    // Create an existing companion zip
    let existing_zip = src_path.join("OriginalFontPackage.zip");
    fs::write(&existing_zip, b"PK existing zip dummy content").unwrap();

    let result = compress_or_copy_alphabet(&src_path, &dest_path, Some(&existing_zip));
    assert!(result.is_ok());

    let target_zip = dest_path.join("OriginalFontPackage.zip");
    assert!(target_zip.exists());

    let mut content = Vec::new();
    File::open(&target_zip)
        .unwrap()
        .read_to_end(&mut content)
        .unwrap();
    assert_eq!(content, b"PK existing zip dummy content");

    let _ = fs::remove_dir_all(&src_path);
    let _ = fs::remove_dir_all(&dest_path);
}

#[test]
fn test_destination_existing_zip_is_idempotent() {
    let src_path = unique_temp_dir("alphabet_idemp_src");
    let dest_path = unique_temp_dir("alphabet_idemp_dest");

    let target_zip = dest_path.join(format!(
        "{}.zip",
        src_path.file_name().unwrap().to_str().unwrap()
    ));
    fs::write(&target_zip, b"pre-existing archive content").unwrap();

    // Call compress
    let result = compress_or_copy_alphabet(&src_path, &dest_path, None).unwrap();
    assert_eq!(result, target_zip);

    // Verify content was not overwritten
    let mut content = Vec::new();
    File::open(&target_zip)
        .unwrap()
        .read_to_end(&mut content)
        .unwrap();
    assert_eq!(content, b"pre-existing archive content");

    let _ = fs::remove_dir_all(&src_path);
    let _ = fs::remove_dir_all(&dest_path);
}
