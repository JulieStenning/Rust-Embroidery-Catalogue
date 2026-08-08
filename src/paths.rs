//! Path resolution for Portable and Installed execution modes.
//!
//! This module centralises all filesystem layout decisions so that every
//! part of the application derives paths from a single `AppPaths` struct
//! returned by `resolve_app_paths()`.

use crate::error::AppError;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The canonical SQLite database filename used throughout the application.
pub const DATABASE_FILENAME: &str = "EmbroideryCatalogue.db";

// â”€â”€â”€ Execution Mode â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Whether the application is running in Portable (SD card / USB) or Installed mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ExecutionMode {
    /// Running from an SD card / USB stick â€” `./Data/` exists next to the exe.
    Portable,
    /// Standard OS install â€” data lives under `%APPDATA%` (or platform equivalent).
    Installed,
}

// â”€â”€â”€ AppPaths â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// All resolved application file-system paths.
#[derive(Debug, Clone, Serialize)]
pub struct AppPaths {
    pub mode: ExecutionMode,
    pub data_root: PathBuf,
    pub embroidery_designs_dir: PathBuf,
    pub database_dir: PathBuf,
    pub database_path: PathBuf,
    pub thumbnail_cache_dir: PathBuf,
    pub log_dir: PathBuf,
}

// â”€â”€â”€ Path resolution algorithm â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Detect whether we are in Portable or Installed mode and return the
/// corresponding `AppPaths` with all directories created.
///
/// Detection algorithm:
/// 1. Determine the directory containing the running executable via
///    `std::env::current_exe()` â†’ `.parent()`.
/// 2. Check if `<exe_dir>/Data/` exists as a directory (canonical case; a
///    legacy lowercase `data/` folder is also accepted so existing
///    deployments on case-sensitive filesystems keep working).
///     - YES â†’ `ExecutionMode::Portable`, `data_root = <exe_dir>/Data/`.
///     - NO  â†’ `ExecutionMode::Installed`, `data_root = platform app-data dir`.
/// 3. Create all required directories under `data_root`.
/// 4. Return `AppPaths`.
pub fn resolve_app_paths() -> Result<AppPaths, AppError> {
    let exe_dir = std::env::current_exe()
        .map_err(|err| AppError::io(format!("failed to resolve executable path: {err}")))?
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    Ok(resolve_paths_from_exe_dir(&exe_dir))
}

/// Core path resolution logic, factored out for testability.
///
/// Given a directory path that represents the "executable directory",
/// determines the execution mode and builds the full `AppPaths`.
///
/// The canonical data folder name is `Data/`; a legacy lowercase `data/`
/// folder is also accepted so existing deployments on case-sensitive
/// filesystems keep working.
pub fn resolve_paths_from_exe_dir(exe_dir: &Path) -> AppPaths {
    let (mode, data_root) = if let Some(data_dir) = find_local_data_dir(exe_dir) {
        // Portable / SD Card mode â€” data is next to the executable
        (ExecutionMode::Portable, data_dir)
    } else {
        // Installed mode â€” use platform-specific app data directory
        (ExecutionMode::Installed, platform_data_root())
    };

    let embroidery_designs_dir = data_root.join("MachineEmbroideryDesigns");
    let database_dir = data_root.join("Database");
    let database_path = database_dir.join(DATABASE_FILENAME);
    let thumbnail_cache_dir = data_root.join("thumbnails");
    let log_dir = data_root.join("logs");

    // Create all required directories (best-effort; failures will surface later)
    for dir in [
        &data_root,
        &embroidery_designs_dir,
        &database_dir,
        &thumbnail_cache_dir,
        &log_dir,
    ] {
        let _ = std::fs::create_dir_all(dir);
    }

    // Seed the database from the bundled resource â€” only in release builds.
    // In dev/debug mode the developer's existing database is always used as-is.
    #[cfg(not(debug_assertions))]
    if !database_path.exists() {
        copy_seed_database_if_missing(&database_path);
    }

    AppPaths {
        mode,
        data_root,
        embroidery_designs_dir,
        database_dir,
        database_path,
        thumbnail_cache_dir,
        log_dir,
    }
}

/// Locate the data directory next to the executable.
///
/// The canonical folder name is `Data/`; a legacy lowercase `data/` folder is
/// also accepted so existing deployments on case-sensitive filesystems keep
/// working. The directory name is matched case-insensitively and the path
/// returned reflects the actual case on disk, so round-tripping through the
/// filesystem (e.g. `cargo clean` + copy) is always correct.
fn find_local_data_dir(exe_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(exe_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().eq_ignore_ascii_case("data") {
            return Some(entry.path());
        }
    }
    None
}

/// Attempt to copy the bundled seed database to `database_path`.
///
/// Only compiled in release builds.
#[cfg(not(debug_assertions))]
fn copy_seed_database_if_missing(database_path: &Path) {
    /// Embedded pre-migrated seed database bytes (compacted to ~180 KB).
    /// Path is relative to this source file (`src/paths.rs`).
    const SEED_DB_BYTES: &[u8] = include_bytes!("../src-tauri/resources/EmbroideryCatalogue.db");

    if let Some(parent) = database_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match std::fs::write(database_path, SEED_DB_BYTES) {
        Ok(_) => {
            // Cannot use tracing here because the subscriber might not be set up yet;
            // we print to stderr instead.  In practice this runs once per installation
            // so it is acceptable.
            eprintln!(
                "[EmbroideryCatalogue] Seeded fresh database from bundled resource â†’ {}",
                database_path.display()
            );
        }
        Err(e) => {
            eprintln!(
                "[EmbroideryCatalogue] WARNING: Failed to seed database from bundle: {}. \
                 The app will create an empty database via SQLx migrations.",
                e
            );
        }
    }
}

// â”€â”€â”€ Platform-specific data root (Installed mode) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

fn platform_data_root() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("EmbroideryCatalogue"))
            .unwrap_or_else(|_| PathBuf::from("."))
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Library/Application Support/EmbroideryCatalogue")
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local/share/EmbroideryCatalogue")
    }
}

// â”€â”€â”€ Relative / Absolute path conversion helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Convert an absolute path to a path relative to `root`.
///
/// Returns an error if the path does not have `root` as a prefix (after
/// canonicalisation falls back to string-level prefix matching).
pub fn to_relative(absolute: &Path, root: &Path) -> Result<PathBuf, std::io::Error> {
    // Try canonical forms first for the most reliable result
    let abs_canon = absolute.canonicalize().unwrap_or_else(|_| absolute.to_path_buf());
    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    if let Ok(rest) = abs_canon.strip_prefix(&root_canon) {
        return Ok(rest.to_path_buf());
    }

    // Fall back to string-level matching (handles non-existent paths etc.)
    let abs_str = abs_canon.to_string_lossy().replace('\\', "/");
    let root_str = root_canon.to_string_lossy().replace('\\', "/");

    if let Some(rest) = abs_str.strip_prefix(&root_str) {
        let rest = rest.trim_start_matches('/');
        return Ok(PathBuf::from(rest));
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "Path '{}' is not under root '{}'",
            absolute.display(),
            root.display()
        ),
    ))
}

/// Reconstruct an absolute path from a `root` and a path stored relative to it.
pub fn to_absolute(relative: &Path, root: &Path) -> PathBuf {
    root.join(relative)
}
#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;

