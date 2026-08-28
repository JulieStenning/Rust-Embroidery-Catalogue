// Tauri command surface for the database-recovery flow.
//
// Exposes the commands the frontend `DatabaseRecoveryView` calls when a
// configured database is missing at startup: drive-letter relocation
// detection, path validation, and the guarded seed fallback.

use crate::services::database_recovery;
use crate::AppState;
use serde::Serialize;
use tauri::State;

/// Result of a drive-letter relocation scan.
#[derive(Debug, Clone, Serialize)]
pub struct DetectedDataRootWire {
    pub data_root: Option<String>,
    pub relative_subpath: String,
}

/// Validation result for a candidate data root.
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseValidationWire {
    pub valid: bool,
    pub data_root: String,
    pub database_path: String,
    pub embroidery_dir: String,
    pub embroidery_dir_exists: bool,
    pub error: Option<String>,
}

/// Detect whether the configured data root can be found at the same relative
/// sub-path on another drive letter (e.g. D: moved to E:).
#[tauri::command]
pub fn detect_relocated_data_root(
    configured_data_root: String,
) -> Result<DetectedDataRootWire, String> {
    let root = std::path::PathBuf::from(configured_data_root.trim());
    let found =
        database_recovery::detect_relocated_data_root(&root).map_err(|err| err.to_string())?;

    Ok(DetectedDataRootWire {
        data_root: found.map(|p| p.to_string_lossy().to_string()),
        relative_subpath: database_recovery::relative_subpath_of_public(&root),
    })
}

/// Validate that `candidate_data_root` contains a real catalogue database.
#[tauri::command]
pub fn validate_database_path(
    candidate_data_root: String,
) -> Result<DatabaseValidationWire, String> {
    let root = std::path::PathBuf::from(candidate_data_root.trim());
    if root.as_os_str().is_empty() {
        return Err("Data root cannot be empty.".to_string());
    }
    let result = database_recovery::validate_database_path(&root);
    Ok(DatabaseValidationWire {
        valid: result.valid,
        data_root: result.data_root,
        database_path: result.database_path,
        embroidery_dir: result.embroidery_dir,
        embroidery_dir_exists: result.embroidery_dir_exists,
        error: result.error,
    })
}

/// Create a fresh empty catalogue at `data_root` (guarded).
///
/// Creates the standard layout (MachineEmbroideryDesigns, logs, Database) and
/// writes the bundled seed database into `Database/`. Refuses to overwrite an
/// existing database unless `overwrite` is explicitly `true` — the frontend
/// only sets that after the user confirms the destructive action.
#[tauri::command]
pub fn seed_database_to_data_root(
    data_root: String,
    overwrite: Option<bool>,
) -> Result<(), String> {
    let root = std::path::PathBuf::from(data_root.trim());
    if root.as_os_str().is_empty() {
        return Err("Data root cannot be empty.".to_string());
    }
    crate::paths::seed_database_if_allowed(&root, overwrite.unwrap_or(false))
        .map_err(|err| err.to_string())
}

// ---------------------------------------------------------------------------
// Inner fns (testable without Tauri state)
// ---------------------------------------------------------------------------

/// Minimal state stub is not needed here — all commands are pure path logic.
#[allow(dead_code)]
fn _state_marker(_state: State<'_, AppState>) {}


#[cfg(test)]
#[path = "database_recovery_tests.rs"]
mod tests;