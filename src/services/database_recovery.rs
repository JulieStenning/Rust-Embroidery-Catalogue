//! Pure service helpers for the database-recovery flow.
//!
//! These functions are deliberately free of Tauri/IPC concerns so they can be
//! unit-tested in isolation. They cover:
//! - Detecting a relocated data root across drive letters (Windows).
//! - Validating a candidate data root has a readable catalogue database.
//!
//! The frontend recovery view calls into these via `routes::database_recovery`.

use crate::error::AppError;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Report whether a candidate data root is a valid catalogue location.
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseValidation {
    pub valid: bool,
    pub data_root: String,
    pub database_path: String,
    pub embroidery_dir: String,
    pub embroidery_dir_exists: bool,
    pub error: Option<String>,
}

/// Result of scanning other drive letters for a relocated catalogue.
#[derive(Debug, Clone, Serialize)]
pub struct DetectedDataRoot {
    pub data_root: Option<String>,
    pub relative_subpath: String,
}

/// The standard catalogue database path relative to a data root.
pub fn database_relative_path() -> PathBuf {
    PathBuf::from("Database").join(crate::paths::DATABASE_FILENAME)
}

/// The standard designs directory name relative to a data root.
pub fn designs_relative_dir() -> &'static str {
    "MachineEmbroideryDesigns"
}

/// Compute the relative sub-path of a configured data root below the
/// filesystem root (e.g. `D:\EmbroideryCatalogue\Data` -> `EmbroideryCatalogue/Data`).
///
/// Returns an empty string when the root cannot be represented as a drive
/// relative path (e.g. it is itself a filesystem root or a UNC path).
/// Public exposure of the drive-relative sub-path computation so the route
/// layer can report it to the frontend without reimplementing the logic.
pub fn relative_subpath_of_public(configured_root: &Path) -> String {
    relative_subpath_of(configured_root)
}

fn relative_subpath_of(configured_root: &Path) -> String {
    let raw = configured_root.to_string_lossy();
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() < 2 {
        // UNC or relative path - cannot scan drive letters meaningfully.
        return "".to_string();
    }
    let tail = parts[1..].join(":");
    let tail = tail.trim_start_matches(&['\\', '/'][..]);
    tail.replace('\\', "/")
}

/// Probe every drive letter A..Z (excluding the configured drive) for a
/// database at `<letter>:\<relative>\Database\EmbroideryCatalogue.db`.
///
/// On non-Windows platforms there are no drive letters, so this always
/// returns `Ok(None)`.
pub fn detect_relocated_data_root(configured_root: &Path) -> Result<Option<PathBuf>, AppError> {
    #[cfg(target_os = "windows")]
    {
        let relative = relative_subpath_of(configured_root);
        if relative.is_empty() {
            return Ok(None);
        }

        let configured_drive_letter = configured_root
            .to_string_lossy()
            .chars()
            .next()
            .unwrap_or(' ')
            .to_ascii_uppercase();

        for letter in b'A'..=b'Z' {
            let drive = (letter as char).to_ascii_uppercase();
            if drive == configured_drive_letter {
                continue;
            }
            let candidate_root = PathBuf::from(format!("{drive}:\\")).join(&relative);
            let candidate_db = candidate_root.join(database_relative_path());
            if candidate_db.is_file() {
                tracing::info!(
                    "Database recovery: found relocated catalogue at {} (drive {} -> {})",
                    candidate_root.display(),
                    configured_drive_letter,
                    drive
                );
                return Ok(Some(candidate_root));
            }
        }
        Ok(None)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

/// Validate that `data_root` is a real catalogue location: the derived
/// database file must exist and be readable. The designs directory is reported
/// separately as a warning (its absence is not a blocker).
pub fn validate_database_path(data_root: &Path) -> DatabaseValidation {
    let data_root_str = data_root.to_string_lossy().to_string();
    let database_path = data_root.join(database_relative_path());
    let database_path_str = database_path.to_string_lossy().to_string();
    let embroidery_dir = data_root.join(designs_relative_dir());
    let embroidery_dir_str = embroidery_dir.to_string_lossy().to_string();
    let embroidery_dir_exists = embroidery_dir.is_dir();

    if !database_path.is_file() {
        return DatabaseValidation {
            valid: false,
            data_root: data_root_str,
            database_path: database_path_str,
            embroidery_dir: embroidery_dir_str,
            embroidery_dir_exists,
            error: Some(format!(
                "No database found at {}. This folder does not look like an Embroidery Catalogue data location.",
                database_path.display()
            )),
        };
    }

    // Probe readability by attempting to open the file for read.
    match std::fs::File::open(&database_path) {
        Ok(_) => DatabaseValidation {
            valid: true,
            data_root: data_root_str,
            database_path: database_path_str,
            embroidery_dir: embroidery_dir_str,
            embroidery_dir_exists,
            error: None,
        },
        Err(err) => DatabaseValidation {
            valid: false,
            data_root: data_root_str,
            database_path: database_path_str,
            embroidery_dir: embroidery_dir_str,
            embroidery_dir_exists,
            error: Some(format!(
                "The database at {} could not be read: {err}",
                database_path.display()
            )),
        },
    }
}

#[cfg(test)]
#[path = "database_recovery_tests.rs"]
mod tests;
