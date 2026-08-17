//! Path resolution for Dev and Installed execution modes.
//!
//! This module centralises all filesystem layout decisions so that every
//! part of the application derives paths from a single `AppPaths` struct
//! returned by `resolve_app_paths()`.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The canonical SQLite database filename used throughout the application.
pub const DATABASE_FILENAME: &str = "EmbroideryCatalogue.db";

// ---------------------------------------------------------------------------
// Execution Mode
// ---------------------------------------------------------------------------

/// Whether the application is running in Dev or Installed mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ExecutionMode {
    /// Debug build - data lives in `<project>/dev_data/`.
    Dev,
    /// Standard OS install. Data lives at the user-configured location (see
    /// `config.json` under the platform app-data dir); until one is chosen,
    /// it falls back to `%APPDATA%` (or platform equivalent).
    Installed,
}

// ---------------------------------------------------------------------------
// AppPaths
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Path resolution algorithm
// ---------------------------------------------------------------------------

/// Detect the execution mode and return the corresponding `AppPaths` with
/// all directories created.
///
/// Detection algorithm:
/// 1. If this is a debug build (`cfg!(debug_assertions)`):
///    - `ExecutionMode::Dev`, `data_root = <project>/dev_data/`.
/// 2. Otherwise -> `ExecutionMode::Installed`, `data_root = platform app-data
///    dir` (or the user-configured location from `config.json`).
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
/// The detection priority is:
/// 1. Debug build (`cfg!(debug_assertions)`) -> Dev.
/// 2. Otherwise -> Installed.
pub fn resolve_paths_from_exe_dir(_exe_dir: &Path) -> AppPaths {
    let (mode, data_root) = if cfg!(debug_assertions) {
        // Dev mode - data lives inside the project root, outside target/
        (ExecutionMode::Dev, dev_data_root())
    } else {
        // Installed mode - platform or user-configured data root
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

    // Seed the database from the bundled resource when no database is present.
    // This runs in every execution mode so a fresh Dev / Installed deployment
    // always starts from the curated seed database.
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

/// Resolve the Dev-mode data root to `<project root>/dev_data/`.
///
/// Uses the compile-time `CARGO_MANIFEST_DIR` environment variable, which
/// points to the directory containing `Cargo.toml`. This is always outside
/// the `target/` directory, so dev data is never wiped by `cargo clean`.
fn dev_data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dev_data")
}

/// Embedded pre-migrated seed database bytes (compacted to ~180 KB).
/// Path is relative to this source file (`src/paths.rs`).
const SEED_DB_BYTES: &[u8] = include_bytes!("../src-tauri/resources/EmbroideryCatalogue.db");

/// Copy the bundled seed database to `data_root/Database/EmbroideryCatalogue.db`,
/// overwriting any existing file.
///
/// This is used only on a **fresh install**: the moment the user answers the
/// first-run "where should your data live?" prompt. Because the seed DB is
/// pre-migrated and contains no design rows, overwriting is safe for a brand
/// new catalogue. Normal launches must NOT call this (it would destroy a real
/// catalogue) — see `ensure_seed_database_if_missing`.
pub fn copy_seed_database_to(data_root: &Path) -> Result<(), AppError> {
    let database_dir = data_root.join("Database");
    let database_path = database_dir.join(DATABASE_FILENAME);

    std::fs::create_dir_all(&database_dir).map_err(|err| {
        AppError::io(format!(
            "failed to create database directory {}: {err}",
            database_dir.display()
        ))
    })?;

    std::fs::write(&database_path, SEED_DB_BYTES).map_err(|err| {
        AppError::io(format!(
            "failed to write seed database {}: {err}",
            database_path.display()
        ))
    })?;

    // Cannot use tracing here because this may run before the subscriber is
    // initialised; print to stderr instead.
    eprintln!(
        "[EmbroideryCatalogue] Seeded fresh database from bundled resource -> {}",
        database_path.display()
    );

    Ok(())
}

/// Attempt to copy the bundled seed database to `database_path` when the file
/// does not already exist.
///
/// This runs on every launch for the standard (non-fresh-install) flow. If a
/// database already exists it is left untouched — never overwritten. On write
/// failure the app falls back to the SQLx migration path (empty schema).
fn copy_seed_database_if_missing(database_path: &Path) {
    if database_path.exists() {
        return;
    }

    if let Some(parent) = database_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match std::fs::write(database_path, SEED_DB_BYTES) {
        Ok(_) => {
            eprintln!(
                "[EmbroideryCatalogue] Seeded fresh database from bundled resource -> {}",
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

// ---------------------------------------------------------------------------
// Platform-specific data root (Installed mode)
// ---------------------------------------------------------------------------

/// The name of the bootstrap config file stored under the platform app-data dir.
///
/// This tiny file lives on the system drive (negligible size) and stores the
/// single path to the user's chosen data root, so large design collections can
/// live on another drive without filling the system drive.
const BOOTSTRAP_CONFIG_FILENAME: &str = "config.json";

/// Runtime bootstrap configuration for Installed mode.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct BootstrapConfig {
    /// The user-chosen absolute path to the data root (e.g. `D:\EmbroideryCatalogue\Data`).
    data_root: PathBuf,
}

/// Bundle path for the bootstrap config in Installed mode.
///
/// This is the fixed location on the system drive (e.g. `%APPDATA%`) that
/// survives uninstalls/reinstalls and records the chosen data-root location.
pub fn bootstrap_config_path() -> PathBuf {
    app_data_base_dir()
        .join("EmbroideryCatalogue")
        .join(BOOTSTRAP_CONFIG_FILENAME)
}

/// The base platform app-data directory (`%APPDATA%` on Windows, etc.).
fn app_data_base_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Library/Application Support")
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local/share")
    }
}

/// Read the persisted user data-root from the bootstrap config.
///
/// Returns `Ok(Some(path))` when a valid config exists, `Ok(None)` when the
/// config is absent (first run), and `Err` when the config is malformed.
pub fn read_bootstrap_data_root() -> Result<Option<PathBuf>, AppError> {
    let path = bootstrap_config_path();
    if !path.is_file() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&path).map_err(|err| {
        AppError::io(format!(
            "failed to read bootstrap config {}: {err}",
            path.display()
        ))
    })?;
    let config: BootstrapConfig = serde_json::from_str(&raw).map_err(|err| {
        AppError::parse(format!(
            "failed to parse bootstrap config {}: {err}",
            path.display()
        ))
    })?;

    Ok(Some(config.data_root))
}

/// Persist the user's chosen data root to the bootstrap config.
///
/// Creates the parent directory and writes the config atomically (write to a
/// temp file then rename) so a crash mid-write cannot corrupt an existing config.
pub fn write_bootstrap_data_root(data_root: &Path) -> Result<(), AppError> {
    if !data_root.is_absolute() {
        return Err(AppError::invalid_input(format!(
            "data root must be an absolute path, got '{}'",
            data_root.display()
        )));
    }

    let path = bootstrap_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            AppError::io(format!(
                "failed to create bootstrap config dir {}: {err}",
                parent.display()
            ))
        })?;
    }

    let config = BootstrapConfig {
        data_root: data_root.to_path_buf(),
    };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|err| AppError::parse(format!("failed to serialize bootstrap config: {err}")))?;

    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|err| {
        AppError::io(format!(
            "failed to write bootstrap config {}: {err}",
            tmp.display()
        ))
    })?;
    std::fs::rename(&tmp, &path).map_err(|err| {
        AppError::io(format!(
            "failed to finalize bootstrap config {}: {err}",
            path.display()
        ))
    })?;

    Ok(())
}

/// Resolve the Installed-mode data root.
///
/// Uses the user-configured data root from `config.json` when present;
/// otherwise falls back to the platform app-data directory so the app still
/// boots on first run (the setup wizard can then relocate it).
fn platform_data_root() -> PathBuf {
    match read_bootstrap_data_root() {
        Ok(Some(root)) => root,
        _ => app_data_base_dir().join("EmbroideryCatalogue"),
    }
}

/// Whether a previously-configured data root has gone missing on disk.
///
/// This is how the app recovers when a portable drive letter changes (or the
/// data folder was moved/deleted): the config still points at the old path,
/// but the folder no longer exists. The frontend prompts the user to reselect
/// a new location in that case.
///
/// Returns `Ok(Some(true))` when a config exists but its path is missing,
/// `Ok(Some(false))` when a config exists and is present, and `Ok(None)` when
/// there is no config at all (first run — the wizard handles it).
pub fn configured_data_root_missing() -> Result<Option<bool>, AppError> {
    match read_bootstrap_data_root()? {
        Some(root) => Ok(Some(!root.exists())),
        None => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Relative / Absolute path conversion helpers
// ---------------------------------------------------------------------------

/// Convert an absolute path to a path relative to `root`.
///
/// Returns an error if the path does not have `root` as a prefix (after
/// canonicalisation falls back to string-level prefix matching).
pub fn to_relative(absolute: &Path, root: &Path) -> Result<PathBuf, std::io::Error> {
    // Try canonical forms first for the most reliable result
    let abs_canon = absolute
        .canonicalize()
        .unwrap_or_else(|_| absolute.to_path_buf());
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

/// Build a full `AppPaths` layout for an arbitrary root directory.
///
/// Used during catalogue storage migration to construct the target layout
/// without touching the live startup resolution. Migration only runs from an
/// installed deployment, so the mode is exposed as `Installed`.
pub fn resolve_paths_for_root(data_root: &Path) -> AppPaths {
    let embroidery_designs_dir = data_root.join("MachineEmbroideryDesigns");
    let database_dir = data_root.join("Database");
    let database_path = database_dir.join(DATABASE_FILENAME);
    let thumbnail_cache_dir = data_root.join("thumbnails");
    let log_dir = data_root.join("logs");

    AppPaths {
        mode: ExecutionMode::Installed,
        data_root: data_root.to_path_buf(),
        embroidery_designs_dir,
        database_dir,
        database_path,
        thumbnail_cache_dir,
        log_dir,
    }
}

/// Best-effort check whether `child` is equal to or nested within `ancestor`.
///
/// When both paths exist they are canonicalised (resolving symlinks and `..`),
/// then compared as normalised strings so drive-letter prefixes like the
/// Windows `\\?\` verbatim marker cannot break the prefix check. Paths that do
/// not exist fall back to raw string normalisation (forward slashes, lowercase
/// on case-insensitive platforms), matching `to_relative`'s fallback behaviour.
pub fn path_within(child: &Path, ancestor: &Path) -> bool {
    let child_canon = child.canonicalize().unwrap_or_else(|_| child.to_path_buf());
    let ancestor_canon = ancestor
        .canonicalize()
        .unwrap_or_else(|_| ancestor.to_path_buf());

    let normalize = |path: &Path| {
        let mut s = path.to_string_lossy().replace('\\', "/");
        while s.ends_with('/') {
            s.pop();
        }
        #[cfg(target_os = "windows")]
        {
            s = s.trim_start_matches("//?/").to_string();
            s = s.to_ascii_lowercase();
        }
        s
    };

    let child_str = normalize(&child_canon);
    let ancestor_str = normalize(&ancestor_canon);

    if ancestor_str.is_empty() {
        return false;
    }

    if child_str == ancestor_str {
        return true;
    }

    child_str.starts_with(&ancestor_str)
        && child_str.as_bytes().get(ancestor_str.len()) == Some(&b'/')
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
