// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Alphabet Compression Service
//!
//! Handles creating companion `.zip` archives of full alphabet packages or copying
//! existing companion `.zip` files into the library destination folder (`MachineEmbroideryDesigns/<AlphabetFolder>/`).
//!
//! Follows strict safety invariants:
//! - Original source files are strictly read-only and never modified or deleted.
//! - Destination archives are created idempotently; existing archives are preserved.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Compresses files from an alphabet folder into a `.zip` archive or copies an existing
/// companion `.zip` to the destination folder.
///
/// Returns the path to the resulting `.zip` file in the destination directory.
pub fn compress_or_copy_alphabet(
    source_folder: &Path,
    dest_folder: &Path,
    companion_zip: Option<&Path>,
) -> Result<PathBuf, String> {
    if !source_folder.exists() || !source_folder.is_dir() {
        return Err(format!(
            "Source folder does not exist or is not a directory: '{}'",
            source_folder.display()
        ));
    }

    fs::create_dir_all(dest_folder).map_err(|e| {
        format!(
            "Failed to create destination directory '{}': {}",
            dest_folder.display(),
            e
        )
    })?;

    // Determine target zip file name
    let zip_filename = if let Some(comp_zip) = companion_zip {
        comp_zip
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Alphabet.zip".to_string())
    } else {
        let folder_name = source_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Alphabet");
        format!("{}.zip", folder_name)
    };

    let dest_zip = dest_folder.join(&zip_filename);

    // If destination zip already exists and is non-empty, retain it (idempotent)
    if dest_zip.exists() {
        if let Ok(meta) = fs::metadata(&dest_zip) {
            if meta.len() > 0 {
                tracing::info!(
                    "Destination alphabet archive already exists at '{}' — skipping recompression",
                    dest_zip.display()
                );
                return Ok(dest_zip);
            }
        }
    }

    // If source folder has an existing companion zip, copy it
    if let Some(src_zip) = companion_zip {
        if src_zip.exists() && src_zip.is_file() {
            tracing::info!(
                "Copying existing companion alphabet archive from '{}' to '{}'",
                src_zip.display(),
                dest_zip.display()
            );
            fs::copy(src_zip, &dest_zip).map_err(|e| {
                format!(
                    "Failed to copy companion archive '{}' to '{}': {}",
                    src_zip.display(),
                    dest_zip.display(),
                    e
                )
            })?;
            return Ok(dest_zip);
        }
    }

    // Otherwise, create a new .zip archive from the source folder
    tracing::info!(
        "Compressing alphabet set from '{}' into '{}'",
        source_folder.display(),
        dest_zip.display()
    );

    let zip_file = File::create(&dest_zip)
        .map_err(|e| format!("Failed to create zip file '{}': {}", dest_zip.display(), e))?;

    let mut zip_writer = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    compress_folder_recursive(source_folder, source_folder, &mut zip_writer, options)?;

    zip_writer.finish().map_err(|e| {
        format!(
            "Failed to finalize zip archive '{}': {}",
            dest_zip.display(),
            e
        )
    })?;

    Ok(dest_zip)
}

/// Recursively traverses `current_dir` and adds files to `zip_writer` using relative paths from `base_dir`.
fn compress_folder_recursive<W: Write + io::Seek>(
    base_dir: &Path,
    current_dir: &Path,
    zip_writer: &mut ZipWriter<W>,
    options: SimpleFileOptions,
) -> Result<(), String> {
    let entries = fs::read_dir(current_dir).map_err(|e| {
        format!(
            "Failed to read directory '{}': {}",
            current_dir.display(),
            e
        )
    })?;

    for entry_res in entries {
        let entry = entry_res.map_err(|e| e.to_string())?;
        let path = entry.path();

        // Skip existing .zip archives to avoid recursive bundling
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("zip") {
                continue;
            }
        }

        if path.is_dir() {
            compress_folder_recursive(base_dir, &path, zip_writer, options)?;
        } else if path.is_file() {
            let rel_path = path
                .strip_prefix(base_dir)
                .map_err(|e| format!("Failed to compute relative path: {}", e))?;

            let rel_str = rel_path
                .to_str()
                .ok_or_else(|| "Invalid UTF-8 path in alphabet archive".to_string())?
                .replace('\\', "/");

            zip_writer
                .start_file(rel_str, options)
                .map_err(|e| format!("Failed to add entry '{}' to zip: {}", path.display(), e))?;

            let mut f = File::open(&path).map_err(|e| {
                format!(
                    "Failed to open file '{}' for archiving: {}",
                    path.display(),
                    e
                )
            })?;

            let mut buffer = [0u8; 16384];
            loop {
                let n = f
                    .read(&mut buffer)
                    .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;
                if n == 0 {
                    break;
                }
                zip_writer.write_all(&buffer[..n]).map_err(|e| {
                    format!("Failed to write to zip for '{}': {}", path.display(), e)
                })?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "alphabet_compressor_tests.rs"]
mod tests;
