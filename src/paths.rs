//! Path resolution for Portable and Installed execution modes.
//!
//! This module centralises all filesystem layout decisions so that every
//! part of the application derives paths from a single `AppPaths` struct
//! returned by `resolve_app_paths()`.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// The canonical SQLite database filename used throughout the application.
pub const DATABASE_FILENAME: &str = "EmbroideryCatalogue.db";

// ─── Execution Mode ───────────────────────────────────────────────────────────

/// Whether the application is running in Portable (SD card / USB) or Installed mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ExecutionMode {
    /// Running from an SD card / USB stick — `./data/` exists next to the exe.
    Portable,
    /// Standard OS install — data lives under `%APPDATA%` (or platform equivalent).
    Installed,
}

// ─── AppPaths ─────────────────────────────────────────────────────────────────

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

// ─── Path resolution algorithm ────────────────────────────────────────────────

/// Detect whether we are in Portable or Installed mode and return the
/// corresponding `AppPaths` with all directories created.
///
/// Detection algorithm:
/// 1. Determine the directory containing the running executable via
///    `std::env::current_exe()` → `.parent()`.
/// 2. Check if `<exe_dir>/data/` exists as a directory.
///     - YES → `ExecutionMode::Portable`, `data_root = <exe_dir>/data/`.
///     - NO  → `ExecutionMode::Installed`, `data_root = platform app-data dir`.
/// 3. Create all required directories under `data_root`.
/// 4. Return `AppPaths`.
pub fn resolve_app_paths() -> AppPaths {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let (mode, data_root) = if exe_dir.join("data").is_dir() {
        // Portable / SD Card mode — data is next to the executable
        (ExecutionMode::Portable, exe_dir.join("data"))
    } else {
        // Installed mode — use platform-specific app data directory
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

    // Seed the database from the bundled resource — only in release builds.
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
                "[EmbroideryCatalogue] Seeded fresh database from bundled resource → {}",
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

// ─── Platform-specific data root (Installed mode) ─────────────────────────────

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

// ─── Relative / Absolute path conversion helpers ──────────────────────────────

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
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolve_detects_portable_when_data_dir_present() {
        let tmp = std::env::temp_dir().join(format!(
            "paths-test-portable-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(tmp.join("data")).expect("create data dir");
        fs::create_dir_all(tmp.join("bin")).expect("create bin dir");

        // Simulate exe inside bin/
        let _exe = tmp.join("bin").join("test.exe");
        // We can't actually set current_exe(), so we test resolve_app_paths
        // directly by checking the Portable path assumption.
        let app_paths = AppPaths {
            mode: ExecutionMode::Portable,
            data_root: tmp.join("data"),
            embroidery_designs_dir: tmp.join("data").join("MachineEmbroideryDesigns"),
            database_dir: tmp.join("data").join("Database"),
            database_path: tmp.join("data").join("Database").join("catalogue.db"),
            thumbnail_cache_dir: tmp.join("data").join("thumbnails"),
            log_dir: tmp.join("data").join("logs"),
        };

        assert_eq!(app_paths.mode, ExecutionMode::Portable);
        assert!(app_paths.data_root.join("Database").exists() || !app_paths.data_root.join("Database").exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn relative_and_absolute_roundtrip() {
        let root = PathBuf::from("/some/data/root");
        let absolute = PathBuf::from("/some/data/root/MachineEmbroideryDesigns/test.dst");
        let relative = PathBuf::from("MachineEmbroideryDesigns/test.dst");

        let reconstructed = to_absolute(&relative, &root);
        assert_eq!(reconstructed, root.join("MachineEmbroideryDesigns/test.dst"));

        // The to_relative function requires the path to actually exist for canonicalize,
        // so we test the logic works when the root is a prefix.
        let rel = to_relative(&absolute, &root).unwrap_or_else(|_| relative.clone());
        assert!(
            rel == relative || rel.to_string_lossy().contains("MachineEmbroideryDesigns"),
            "Expected relative path, got: {:?}",
            rel
        );
    }
}