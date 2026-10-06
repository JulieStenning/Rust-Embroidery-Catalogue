// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::session::IMPORT_UNKNOWN_FOLDER;
use crate::config::BootstrapConfig;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) fn folder_key_from_full_path(full_path: &str) -> String {
    let normalized = full_path.trim().replace('\\', "/");
    if normalized.is_empty() {
        return IMPORT_UNKNOWN_FOLDER.to_string();
    }
    match normalized.rfind('/') {
        Some(index) if index > 0 => normalized[..index].to_string(),
        _ => IMPORT_UNKNOWN_FOLDER.to_string(),
    }
}

pub(crate) fn normalize_path_for_match(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

pub(crate) fn strip_sqlite_prefix(database_url: &str) -> &str {
    database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url)
}

pub(crate) fn derive_data_root_from_database_url() -> PathBuf {
    let config = BootstrapConfig::from_env();
    let db_path = Path::new(strip_sqlite_prefix(&config.database_url));

    let root = if let Some(parent) = db_path.parent() {
        if parent
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case("database"))
            .unwrap_or(false)
        {
            parent.parent().unwrap_or(parent)
        } else {
            parent
        }
    } else {
        Path::new("data")
    };

    root.canonicalize().unwrap_or_else(|_| root.to_path_buf())
}

pub(crate) fn get_designs_base_path() -> PathBuf {
    if let Some(app) = super::session::get_bulk_import_app_handle() {
        use tauri::Manager;
        if let Some(state) = app.try_state::<crate::state::AppState>() {
            return state.paths.embroidery_designs_dir.clone();
        }
    }
    derive_data_root_from_database_url().join("MachineEmbroideryDesigns")
}

pub(crate) fn is_path_under_base(full_path: &str, base: &Path) -> bool {
    crate::paths::path_within(Path::new(full_path.trim()), base)
}

pub(crate) fn is_path_under_designs_base(full_path: &str) -> bool {
    is_path_under_base(full_path, &get_designs_base_path())
}

pub(crate) fn full_path_to_stored_design_filepath(full_path: &str) -> Result<String, String> {
    let normalized_full = full_path.trim().replace('\\', "/");
    if normalized_full.is_empty() {
        return Err("Import filepath is empty.".to_string());
    }

    let designs_base = get_designs_base_path();

    if !is_path_under_designs_base(&normalized_full) {
        return Err(format!(
            "Selected file is outside catalogue design storage. Expected under '{}', got '{}'.",
            designs_base.to_string_lossy(),
            full_path
        ));
    }

    crate::paths::design_rel_from_full(&normalized_full, &designs_base).ok_or_else(|| {
        format!(
            "Selected path is the library root itself (not a design file): '{}'",
            full_path
        )
    })
}

pub(crate) fn compute_prospective_stored_filepath(
    full_path: &str,
    root_paths: &[String],
) -> Result<String, String> {
    if let Ok(stored) = full_path_to_stored_design_filepath(full_path) {
        return Ok(stored);
    }

    let source = Path::new(full_path);
    let source_norm = source.to_string_lossy().replace('\\', "/");

    let rel_path = root_paths
        .iter()
        .map(|root| root.replace('\\', "/").trim_end_matches('/').to_string())
        .filter(|root| {
            let root_lower = root.to_ascii_lowercase();
            let source_lower = source_norm.to_ascii_lowercase();
            if let Some(rest) = source_lower.strip_prefix(&root_lower) {
                rest.is_empty() || rest.starts_with('/')
            } else {
                false
            }
        })
        .max_by_key(|root| root.len())
        .map(|root| {
            let root_folder_name = Path::new(&root)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("import");

            let root_lower = root.to_ascii_lowercase();
            let source_lower = source_norm.to_ascii_lowercase();

            let is_drive_root =
                root.len() <= 3 && root.ends_with(':') || (root.len() <= 4 && root.ends_with(":/"));

            if is_drive_root {
                if source_lower.len() > root_lower.len() {
                    let after_root = &source_norm[root.len()..];
                    let sub_path = after_root.trim_start_matches('/');
                    if sub_path.is_empty() {
                        source
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string()
                    } else {
                        sub_path.to_string()
                    }
                } else {
                    source
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                }
            } else if source_lower.len() > root_lower.len() {
                let after_root = &source_norm[root.len()..];
                let sub_path = after_root.trim_start_matches('/');
                if sub_path.is_empty() {
                    root_folder_name.to_string()
                } else {
                    format!("{}/{}", root_folder_name, sub_path)
                }
            } else {
                let filename = source
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                format!("{}/{}", root_folder_name, filename)
            }
        })
        .unwrap_or_else(|| {
            source
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    Ok(crate::paths::canonical_design_rel(&rel_path))
}

pub fn compute_file_hash_blake3(file_path: impl AsRef<Path>) -> Result<String, String> {
    let file_path = file_path.as_ref();
    let mut file = File::open(file_path).map_err(|e| {
        format!(
            "Failed to open file for hashing '{}': {}",
            file_path.display(),
            e
        )
    })?;

    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65536]; // 64 KiB buffer
    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| {
            format!(
                "Failed to read file for hashing '{}': {}",
                file_path.display(),
                e
            )
        })?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

pub fn compute_file_size(file_path: impl AsRef<Path>) -> Result<i64, String> {
    let file_path = file_path.as_ref();
    let metadata = fs::metadata(file_path).map_err(|e| {
        format!(
            "Failed to read metadata for '{}': {}",
            file_path.display(),
            e
        )
    })?;
    Ok(metadata.len() as i64)
}

pub(crate) fn ensure_file_in_designs_base(
    full_path: &str,
    root_paths: &[String],
) -> Result<String, String> {
    if let Ok(stored) = full_path_to_stored_design_filepath(full_path) {
        return Ok(stored);
    }

    let source = Path::new(full_path);
    if !source.exists() {
        return Err(format!("Import file does not exist: '{}'", full_path));
    }

    let source_size = compute_file_size(source)?;
    let source_hash = compute_file_hash_blake3(source)?;

    let designs_base = get_designs_base_path();
    let prospective_stored = compute_prospective_stored_filepath(full_path, root_paths)?;
    let rel_path = prospective_stored.trim_start_matches('/');

    let dest = designs_base.join(rel_path);

    if dest.exists() {
        let dest_size = compute_file_size(&dest).unwrap_or(0);
        let dest_hash = compute_file_hash_blake3(&dest).unwrap_or_default();

        if dest_size == source_size && dest_hash == source_hash {
            tracing::info!(
                "Import file '{}' content-identical to existing '{}' — reusing stored path",
                source.display(),
                dest.display()
            );
            return Ok(prospective_stored);
        }

        let dest_parent = dest.parent().ok_or_else(|| {
            format!(
                "Cannot determine parent directory for destination: '{}'",
                dest.display()
            )
        })?;

        let stem = dest
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("design");
        let ext = dest.extension().and_then(|n| n.to_str()).unwrap_or("");

        let mut counter = 1u32;
        let final_dest = loop {
            let candidate_name = if ext.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, ext)
            };
            let candidate = dest_parent.join(&candidate_name);
            if !candidate.exists() {
                break candidate;
            }
            counter += 1;
            if counter > 1000 {
                return Err(format!(
                    "Failed to find available auto-rename target for '{}' after 1000 attempts",
                    dest.display()
                ));
            }
        };

        tracing::info!(
            "Import collision: '{}' exists with different content — auto-renaming to '{}'",
            dest.display(),
            final_dest.display()
        );

        fs::create_dir_all(dest_parent).map_err(|e| {
            format!(
                "Failed to create directory '{}': {}",
                dest_parent.display(),
                e
            )
        })?;

        fs::copy(source, &final_dest).map_err(|e| {
            format!(
                "Failed to copy '{}' to '{}': {}",
                source.display(),
                final_dest.display(),
                e
            )
        })?;

        return full_path_to_stored_design_filepath(&final_dest.to_string_lossy());
    }

    let dest_parent = dest.parent().ok_or_else(|| {
        format!(
            "Cannot determine parent directory for destination: '{}'",
            dest.display()
        )
    })?;

    fs::create_dir_all(dest_parent).map_err(|e| {
        format!(
            "Failed to create directory '{}': {}",
            dest_parent.display(),
            e
        )
    })?;

    fs::copy(source, &dest).map_err(|e| {
        format!(
            "Failed to copy '{}' to '{}': {}",
            source.display(),
            dest.display(),
            e
        )
    })?;

    full_path_to_stored_design_filepath(&dest.to_string_lossy())
}
