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
    let log_dir = data_root.join("logs");

    // Seed the database from the bundled resource when no database is present.
    //
    // - Dev mode: always seed — a fresh checkout has no DB yet.
    // - Installed mode: seed only on a genuine first run (no configured root in
    //   config.json). When a configured root simply points at a missing
    //   database (e.g. a portable drive letter changed from D: to E:), we must
    //   NOT silently copy a blank seed into the old path and lose the user's
    //   real catalogue — the database-recovery flow takes over instead and
    //   asks the user to re-point the location.
    //
    // The catalogue layout directories (MachineEmbroideryDesigns, Database,
    // logs) are created ONLY when the database already exists at this root or
    // when we are about to seed a fresh database. In database-recovery mode
    // (configured root present but DB missing) nothing is created at the
    // stale root — the user re-points to the real location instead.
    let seed_on_missing = match mode {
        ExecutionMode::Dev => true,
        ExecutionMode::Installed => read_bootstrap_data_root().ok().flatten().is_none(),
    };
    let layout_needed = database_path.exists() || seed_on_missing;

    if layout_needed {
        // Create all required directories (best-effort; failures will surface later)
        for dir in [&data_root, &embroidery_designs_dir, &database_dir, &log_dir] {
            let _ = std::fs::create_dir_all(dir);
        }
        if !database_path.exists() {
            copy_seed_database_if_missing(&database_path);
        }
    }

    AppPaths {
        mode,
        data_root,
        embroidery_designs_dir,
        database_dir,
        database_path,
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

/// Check if a database file exists within the data root, checking both:
/// 1. `<data_root>/Database/EmbroideryCatalogue.db` (standard layout)
/// 2. `<data_root>/EmbroideryCatalogue.db` (direct in root)
///
/// Returns `Some(PathBuf)` of the existing database file if found, otherwise `None`.
pub fn detect_existing_database_path(data_root: &Path) -> Option<PathBuf> {
    let standard_path = data_root.join("Database").join(DATABASE_FILENAME);
    if standard_path.is_file() {
        return Some(standard_path);
    }

    let root_path = data_root.join(DATABASE_FILENAME);
    if root_path.is_file() {
        return Some(root_path);
    }

    None
}

/// Returns true if an existing database file is present at `data_root`.
pub fn has_existing_database(data_root: &Path) -> bool {
    detect_existing_database_path(data_root).is_some()
}

/// Ensure the standard catalogue layout exists and copy the bundled seed database
/// only if no database already exists.
///
/// If `<data_root>/Database/EmbroideryCatalogue.db` already exists, seed copying
/// is skipped to avoid destroying existing data.
/// If `<data_root>/EmbroideryCatalogue.db` exists (directly in data root), it is
/// moved/preserved under `<data_root>/Database/EmbroideryCatalogue.db`.
///
/// Returns `Ok(false)` if an existing database was detected and preserved.
/// Returns `Ok(true)` if a fresh seed database was written.
pub fn ensure_catalogue_layout_and_seed_if_missing(data_root: &Path) -> Result<bool, AppError> {
    create_catalogue_layout(data_root)?;

    let target_database_path = data_root.join("Database").join(DATABASE_FILENAME);
    if target_database_path.is_file() {
        eprintln!(
            "[EmbroideryCatalogue] Existing database detected at {} — preserving without overwriting.",
            target_database_path.display()
        );
        return Ok(false);
    }

    let direct_root_path = data_root.join(DATABASE_FILENAME);
    if direct_root_path.is_file() {
        eprintln!(
            "[EmbroideryCatalogue] Moving existing database from {} to {}",
            direct_root_path.display(),
            target_database_path.display()
        );
        std::fs::rename(&direct_root_path, &target_database_path).map_err(|err| {
            AppError::io(format!(
                "failed to move database from {} to {}: {err}",
                direct_root_path.display(),
                target_database_path.display()
            ))
        })?;
        return Ok(false);
    }

    std::fs::write(&target_database_path, SEED_DB_BYTES).map_err(|err| {
        AppError::io(format!(
            "failed to write seed database {}: {err}",
            target_database_path.display()
        ))
    })?;

    eprintln!(
        "[EmbroideryCatalogue] Seeded fresh database from bundled resource -> {}",
        target_database_path.display()
    );

    Ok(true)
}

/// Copy the bundled seed database to `data_root/Database/EmbroideryCatalogue.db`,
/// preserving any existing database if already present.
pub fn copy_seed_database_to(data_root: &Path) -> Result<(), AppError> {
    ensure_catalogue_layout_and_seed_if_missing(data_root).map(|_| ())
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

/// Create the standard catalogue layout directories under `data_root`:
/// `<root>/MachineEmbroideryDesigns`, `<root>/logs` and `<root>/Database`.
///
/// Used by the database-recovery "create new catalogue" action so a freshly
/// seeded catalogue has exactly the same folder structure as a normal install
/// (designs root + logs + database directory).
pub fn create_catalogue_layout(data_root: &Path) -> Result<(), AppError> {
    for dir in [
        data_root.join("MachineEmbroideryDesigns"),
        data_root.join("logs"),
        data_root.join("Database"),
    ] {
        std::fs::create_dir_all(&dir).map_err(|err| {
            AppError::io(format!(
                "failed to create catalogue directory {}: {err}",
                dir.display()
            ))
        })?;
    }
    Ok(())
}

/// Guarded seed for the database-recovery flow.
///
/// Creates the standard catalogue layout then writes the bundled seed database
/// into `<root>/Database/EmbroideryCatalogue.db`.
///
/// - With `overwrite = false` (default) this **errors** if a database already
///   exists at the target path, so a real catalogue can never be clobbered.
/// - With `overwrite = true` it replaces any existing file (explicit user
///   confirmation required in the UI).
pub fn seed_database_if_allowed(data_root: &Path, overwrite: bool) -> Result<(), AppError> {
    let database_path = data_root.join("Database").join(DATABASE_FILENAME);

    if !overwrite && database_path.exists() {
        return Err(AppError::invalid_input(format!(
            "a database already exists at {}; refusing to overwrite without explicit confirmation",
            database_path.display()
        )));
    }

    create_catalogue_layout(data_root)?;

    std::fs::write(&database_path, SEED_DB_BYTES).map_err(|err| {
        AppError::io(format!(
            "failed to write seed database {}: {err}",
            database_path.display()
        ))
    })?;

    eprintln!(
        "[EmbroideryCatalogue] Seeded fresh database from bundled resource -> {}",
        database_path.display()
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Canonical design filepath helpers (single source of truth)
// ---------------------------------------------------------------------------

/// The managed designs library container folder name, created under the data
/// root on first install (see [`create_catalogue_layout`]). This name is a
/// *root marker only* — it must never be stored inside `designs.filepath`.
/// Kept private so no caller reconstructs the library root by string-matching
/// the container; resolve it from `AppPaths` instead.
const DESIGNS_CONTAINER: &str = "MachineEmbroideryDesigns";

/// Collapse runs of `/` into a single separator.
fn collapse_dup_slashes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_slash = false;
    for ch in input.chars() {
        if ch == '/' {
            if last_was_slash {
                continue;
            }
            last_was_slash = true;
        } else {
            last_was_slash = false;
        }
        out.push(ch);
    }
    out
}

/// Canonicalise any stored / prospective `filepath` string into the single
/// canonical library-relative form used across the whole application:
///
/// * forward slashes (`/`) only — `\` is converted;
/// * no leading `/` (leading separators are stripped);
/// * a single **leading** `MachineEmbroideryDesigns` container segment is
///   dropped so the result is relative to the designs library root. A nested
///   folder of the same name is preserved (only the first path element is
///   treated as the root);
/// * exact case is preserved — never lower-cased here.
///
/// Inputs that still carry an absolute root (e.g. a legacy `C:/…` path with no
/// container marker) are returned unchanged so a resolver can reproduce the
/// old absolute passthrough behaviour rather than inventing a wrong base.
pub fn canonical_design_rel(input: &str) -> String {
    let slashed = input.trim().replace('\\', "/");
    let collapsed = collapse_dup_slashes(&slashed);
    if collapsed.is_empty() {
        return String::new();
    }

    // A leading '/' marks the library/base root in legacy stored paths, so it is
    // stripped to yield a base-relative path (the canonical form has no leading
    // slash). A leading `MachineEmbroideryDesigns` container segment is dropped too.
    let trimmed = collapsed.trim_start_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    let mut parts = trimmed.splitn(2, '/');
    let head = parts.next().unwrap_or("");
    let tail = parts.next().unwrap_or("");
    if head.eq_ignore_ascii_case(DESIGNS_CONTAINER) {
        return tail.trim_start_matches('/').to_string();
    }

    // A Windows drive prefix (e.g. "C:/...") is a real absolute path and cannot
    // be reduced without a base; preserve it for the legacy absolute passthrough.
    if head.len() == 2 && head.ends_with(':') {
        return collapsed;
    }

    // Otherwise return the base-relative path (no leading slash).
    trimmed.to_string()
}

/// Turn a full on-disk file path that lives under `library_root` into its
/// canonical library-relative form. Returns `None` when the path is not under
/// `library_root` or when `library_root` itself is passed (a directory, not a
/// design file).
pub fn design_rel_from_full(full: &str, library_root: &Path) -> Option<String> {
    let full_norm = full.trim().replace('\\', "/");
    if full_norm.is_empty() {
        return None;
    }
    let root_str = library_root.to_string_lossy().replace('\\', "/");
    let root_trim = root_str.trim_end_matches('/');
    if root_trim.is_empty() {
        return None;
    }

    let full_lower = full_norm.to_ascii_lowercase();
    let root_lower = root_trim.to_ascii_lowercase();
    if full_lower == root_lower {
        return None; // the library root itself → not a design file
    }

    let prefix = format!("{}/", root_lower);
    full_lower.strip_prefix(&prefix)?;

    // `full_norm` is guaranteed at least `root_trim.len() + 1` long because we
    // matched `root_lower` + `/` in the lower-cased form (same lengths).
    let tail = full_norm[root_trim.len() + 1..].trim_start_matches('/');
    let canonical = canonical_design_rel(tail);
    if canonical.is_empty() {
        None
    } else {
        Some(canonical)
    }
}

/// Resolve a stored `filepath` (canonical relative form) to an absolute
/// on-disk path under `library_root`.
///
/// * empty → `library_root` itself;
/// * canonical relative → `library_root.join(rel)`;
/// * a leftover absolute string → returned as-is (`PathBuf::join` semantics
///   reproduce the legacy absolute passthrough for any row the migration could
///   not reduce).
pub fn resolve_design_filepath(stored: &str, library_root: &Path) -> PathBuf {
    let canonical = canonical_design_rel(stored);
    if canonical.is_empty() {
        return library_root.to_path_buf();
    }
    let candidate = PathBuf::from(&canonical);
    if candidate.is_absolute() {
        candidate
    } else {
        library_root.join(candidate)
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

/// Whether the app is in database-recovery mode.
///
/// True for Installed mode when a data root has been configured but the
/// derived database file is missing (e.g. a portable drive letter changed
/// from D: to E: or the data folder was moved). In this state the backend
/// must NOT create catalogue folders, seed the database, or write logs under
/// the stale configured root — the recovery view asks the user to re-point
/// the location first.
pub fn database_recovery_mode(paths: &AppPaths) -> bool {
    matches!(paths.mode, ExecutionMode::Installed)
        && read_bootstrap_data_root().ok().flatten().is_some()
        && !paths.database_path.exists()
}

/// Safe log location used while the app is in database-recovery mode.
///
/// Avoids creating a `logs` directory under the stale configured data root
/// (which may be a drive-letter remnant like `F:\`). Logs are written to the
/// OS temp dir instead; the real logs resume at the re-pointed root after the
/// recovery-triggered restart.
pub fn recovery_log_dir() -> PathBuf {
    std::env::temp_dir()
        .join("EmbroideryCatalogue")
        .join("logs")
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

// ---------------------------------------------------------------------------
// Path normalization helpers (single source of truth)
// ---------------------------------------------------------------------------

/// Returns `full_path` relative to `root` (`""` when they are equal). Assumes
/// `full_path` has already been validated to be under `root`. Separators are
/// normalised to forward slashes and comparisons are case-insensitive (ASCII),
/// so drive-letter casing differences cannot break the derivation.
pub fn relative_path_under_root(full_path: &str, root: &Path) -> String {
    let full_norm = full_path.replace('\\', "/");
    let root_norm = root
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();
    if full_norm.eq_ignore_ascii_case(&root_norm) {
        return String::new();
    }
    let root_lower = root_norm.to_ascii_lowercase();
    if full_norm.to_ascii_lowercase().starts_with(&root_lower) {
        full_norm[root_norm.len()..]
            .trim_start_matches('/')
            .to_string()
    } else {
        full_norm.trim_start_matches('/').to_string()
    }
}

/// Convert a filesystem path to a consistent, readable display string.
///
/// On Windows this strips the extended-length verbatim marker that
/// `canonicalize()` prepends and normalises separators to native backslashes,
/// so a database path from the bootstrap URL and a canonicalised designs path
/// render identically. On other platforms the path is returned as a lossy
/// UTF-8 string unchanged.
///
/// Display-only; the result is never round-tripped into a file operation.
pub fn normalize_path_display(path: &Path) -> String {
    let raw = path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        let without_verbatim = if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{}", rest)
        } else if let Some(rest) = raw.strip_prefix(r"\\?\") {
            rest.to_string()
        } else {
            raw
        };
        without_verbatim.replace('/', r"\")
    }

    #[cfg(not(target_os = "windows"))]
    {
        raw
    }
}

/// Normalise a path for use as an `explorer.exe` command-line target.
///
/// Strips the extended-length verbatim marker that `canonicalize()` adds (which
/// `explorer.exe` does not accept) and converts forward slashes to native
/// backslashes. On non-Windows builds the path is returned unchanged so the
/// helper remains callable on every platform.
pub fn normalize_windows_explorer_target(path: &Path) -> PathBuf {
    PathBuf::from(normalize_path_display(path))
}

pub fn resolve_paths_for_root(data_root: &Path) -> AppPaths {
    let embroidery_designs_dir = data_root.join("MachineEmbroideryDesigns");
    let database_dir = data_root.join("Database");
    let database_path = database_dir.join(DATABASE_FILENAME);
    let log_dir = data_root.join("logs");

    AppPaths {
        mode: ExecutionMode::Installed,
        data_root: data_root.to_path_buf(),
        embroidery_designs_dir,
        database_dir,
        database_path,
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
