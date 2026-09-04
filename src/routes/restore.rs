//! Restore Tauri commands.
//!
//! Thin IPC layer over `services::restore`. Resolves backup destinations from
//! settings, manages the `restore_in_progress` pool-swap gate, and streams
//! progress on `catalogue-restore-progress`.

use crate::routes::maintenance as mnt;
use crate::services::folder_picker;
use crate::services::restore;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};

/// Cooperative cancellation flag observed by the designs-restore copy loop.
static RESTORE_CANCEL_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Deserialize)]
pub struct RestoreDatabaseRequest {
    pub db_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RestoreDesignsRequest {
    pub designs_source_dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RestoreBothRequest {
    pub db_file: String,
    pub designs_source_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowseRestoreFileResponse {
    pub path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RestoreBothResult {
    pub database: restore::DatabaseRestoreOutcome,
    pub designs: restore::DesignsRestoreOutcome,
    pub unmatched: Option<restore::DetectUnmatchedFilesResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CancelRestoreResult {
    pub cancel_requested: bool,
}

/// Resets `restore_in_progress` when dropped, even on error.
struct RestoreGuard<'a>(&'a AtomicBool);

impl Drop for RestoreGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// After a database restore swaps the live pool, point the bulk-import module's
/// cached pool at the current pool and clear any in-flight import contexts.
fn refresh_bulk_import_after_restore(state: &State<'_, AppState>) {
    if let Some(pool) = state.db.pool() {
        crate::routes::bulk_import::update_bulk_import_db_pool(pool);
    }
    let _ = crate::routes::bulk_import::reset_bulk_import_context_store_for_restore();
}

/// Broadcast a progress event to the frontend (fire-and-forget).
fn emit_progress(app_handle: &AppHandle, progress: restore::RestoreProgress) {
    let _ = app_handle.emit(restore::RESTORE_PROGRESS_EVENT, progress);
}

/// Read a stored backup destination setting from the settings table.
async fn read_backup_setting(state: &State<'_, AppState>, key: &str) -> Result<String, String> {
    let pool = state.db_pool()?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    mnt::get_setting_with_default(&mut conn, key)
        .await
        .map_err(|e| e.to_string())
}

/// File picker for a database backup snapshot, defaulting to the configured
/// `Database backup folder` and restricted to `.db` files.
#[tauri::command]
pub fn browse_restore_file(start_dir: Option<String>) -> BrowseRestoreFileResponse {
    match folder_picker::pick_db_backup_file(start_dir.as_deref()) {
        Ok(result) => BrowseRestoreFileResponse {
            path: result.path,
            error: None,
        },
        Err(error) => BrowseRestoreFileResponse {
            path: None,
            error: Some(error.to_string()),
        },
    }
}

/// Raise the cooperative restore cancellation flag.
#[tauri::command]
pub fn request_cancel_restore() -> CancelRestoreResult {
    RESTORE_CANCEL_REQUESTED.store(true, Ordering::SeqCst);
    CancelRestoreResult {
        cancel_requested: true,
    }
}

/// Swap the live database for a user-selected backup snapshot, with automatic
/// rollback on verification failure.
#[tauri::command]
pub async fn restore_database(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: RestoreDatabaseRequest,
) -> Result<restore::DatabaseRestoreOutcome, String> {
    RESTORE_CANCEL_REQUESTED.store(false, Ordering::SeqCst);
    if state.restore_in_progress.load(Ordering::SeqCst) {
        tracing::error!("[restore] restore_database rejected: a restore is already in progress");
        return Err("A restore is already in progress.".to_string());
    }
    state.restore_in_progress.store(true, Ordering::SeqCst);
    let _guard = RestoreGuard(&state.restore_in_progress);

    let backup_path = PathBuf::from(request.db_file.trim());
    if backup_path.as_os_str().is_empty() {
        tracing::error!("[restore] restore_database rejected: no backup file selected");
        return Err("No database backup file selected.".to_string());
    }
    tracing::info!(
        "[restore] restore_database requested file='{}'",
        backup_path.display()
    );

    emit_progress(
        &app_handle,
        restore::RestoreProgress::new("db-swap", "starting"),
    );
    let outcome =
        match restore::perform_database_restore(&state.db, &state.paths, &backup_path).await {
            Ok(outcome) => outcome,
            Err(error) => {
                tracing::error!(
                    "[restore] restore_database failed file='{}': {error}",
                    backup_path.display()
                );
                return Err(error);
            }
        };
    refresh_bulk_import_after_restore(&state);
    tracing::info!(
        "[restore] restore_database outcome success={} rolled_back={} designs={} error={:?}",
        outcome.success,
        outcome.rolled_back,
        outcome.design_count,
        outcome.error,
    );

    emit_progress(
        &app_handle,
        restore::RestoreProgress {
            phase: "completed".to_string(),
            db_status: if outcome.success {
                "restored".to_string()
            } else {
                "rolled-back".to_string()
            },
            scanned: 0,
            copied: 0,
            skipped: 0,
            total_bytes: 0,
            percent: 1.0,
            error: outcome.error.clone(),
        },
    );
    Ok(outcome)
}

/// Incremental mirror restore of design files from the designs backup folder.
#[tauri::command]
pub async fn restore_designs_incremental(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: Option<RestoreDesignsRequest>,
) -> Result<restore::DesignsRestoreOutcome, String> {
    RESTORE_CANCEL_REQUESTED.store(false, Ordering::SeqCst);

    let source_raw = match request.and_then(|r| r.designs_source_dir) {
        Some(dir) if !dir.trim().is_empty() => dir.trim().to_string(),
        _ => read_backup_setting(&state, mnt::KEY_BACKUP_DESIGNS_DESTINATION).await?,
    };
    let source_root = PathBuf::from(source_raw.trim());
    if source_root.as_os_str().is_empty() {
        let message =
            "No designs backup folder configured. Save a designs backup destination first."
                .to_string();
        tracing::error!("[restore] {message}");
        return Err(message);
    }
    if !source_root.is_dir() {
        let message = format!("Designs backup folder not found: {}", source_root.display());
        tracing::error!("[restore] {message}");
        return Err(message);
    }
    tracing::info!(
        "[restore] restore_designs_incremental requested source='{}'",
        source_root.display()
    );

    let dest_root = mnt::derive_designs_source_path();
    let mut progress = |p: restore::RestoreProgress| emit_progress(&app_handle, p);
    let outcome = match restore::perform_designs_restore(
        &source_root,
        &dest_root,
        &RESTORE_CANCEL_REQUESTED,
        &mut progress,
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            tracing::error!(
                "[restore] restore_designs_incremental failed source='{}': {error}",
                source_root.display()
            );
            return Err(error);
        }
    };
    tracing::info!(
        "[restore] restore_designs_incremental outcome scanned={} copied={} skipped={}",
        outcome.scanned,
        outcome.copied,
        outcome.skipped,
    );

    emit_progress(
        &app_handle,
        restore::RestoreProgress {
            phase: "completed".to_string(),
            db_status: "restored".to_string(),
            scanned: outcome.scanned,
            copied: outcome.copied + outcome.updated,
            skipped: outcome.skipped,
            total_bytes: outcome.total_bytes_copied,
            percent: 1.0,
            error: None,
        },
    );

    Ok(outcome)
}

/// Restore the database then sync design files, then reconcile unmatched files.
#[tauri::command]
pub async fn restore_both(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: RestoreBothRequest,
) -> Result<RestoreBothResult, String> {
    RESTORE_CANCEL_REQUESTED.store(false, Ordering::SeqCst);
    if state.restore_in_progress.load(Ordering::SeqCst) {
        return Err("A restore is already in progress.".to_string());
    }
    // Resolve + validate the designs backup folder BEFORE restoring the database,
    // so "Restore Both" fails fast with a clear message instead of half-restoring
    // the database and then failing on the designs phase. This runs before the
    // restore gate is set so read_backup_setting can still read the pool.
    let source_raw = match request.designs_source_dir.map(|dir| dir.trim().to_string()) {
        Some(dir) if !dir.is_empty() => dir,
        _ => read_backup_setting(&state, mnt::KEY_BACKUP_DESIGNS_DESTINATION).await?,
    };
    let source_root = PathBuf::from(source_raw.trim());
    if source_root.as_os_str().is_empty() {
        let message =
            "No designs backup folder configured. Save a designs backup destination first."
                .to_string();
        tracing::error!("[restore] {message}");
        return Err(message);
    }
    if !source_root.is_dir() {
        let message = format!("Designs backup folder not found: {}", source_root.display());
        tracing::error!("[restore] {message}");
        return Err(message);
    }
    let dest_root = mnt::derive_designs_source_path();

    state.restore_in_progress.store(true, Ordering::SeqCst);
    let _guard = RestoreGuard(&state.restore_in_progress);

    let backup_path = PathBuf::from(request.db_file.trim());
    if backup_path.as_os_str().is_empty() {
        tracing::error!("[restore] restore_both rejected: no backup file selected");
        return Err("No database backup file selected.".to_string());
    }
    tracing::info!(
        "[restore] restore_both requested file='{}'",
        backup_path.display()
    );

    // Database first — abort the designs phase if it fails so we never sync
    // designs into a mismatched database (which would create orphans).
    emit_progress(
        &app_handle,
        restore::RestoreProgress::new("db-swap", "starting"),
    );
    let database =
        match restore::perform_database_restore(&state.db, &state.paths, &backup_path).await {
            Ok(outcome) => outcome,
            Err(error) => {
                tracing::error!(
                    "[restore] restore_both database phase failed file='{}': {error}",
                    backup_path.display()
                );
                return Err(error);
            }
        };
    refresh_bulk_import_after_restore(&state);
    if !database.success {
        tracing::error!(
            "[restore] restore_both database restore failed; skipping designs sync (error={:?})",
            database.error
        );
        return Ok(RestoreBothResult {
            database,
            designs: restore::DesignsRestoreOutcome {
                success: false,
                scanned: 0,
                copied: 0,
                updated: 0,
                skipped: 0,
                total_bytes_copied: 0,
                error: Some(
                    "Designs sync skipped because the database restore failed.".to_string(),
                ),
            },
            unmatched: None,
        });
    }

    // Designs sync.
    let mut progress = |p: restore::RestoreProgress| emit_progress(&app_handle, p);
    let designs = match restore::perform_designs_restore(
        &source_root,
        &dest_root,
        &RESTORE_CANCEL_REQUESTED,
        &mut progress,
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            tracing::error!(
                "[restore] restore_both designs phase failed source='{}': {error}",
                source_root.display()
            );
            return Err(error);
        }
    };

    // Reconciliation: files on disk absent from the restored database.
    // Use the holder directly (not `db_pool`) because `restore_in_progress` is
    // still set for the duration of this command.
    let pool = state
        .db
        .pool()
        .ok_or_else(|| "The database pool is unavailable.".to_string())?;
    let unmatched = restore::detect_design_files_absent_from_database(&pool, &dest_root)
        .await
        .ok();
    tracing::info!(
        "[restore] restore_both completed designs_copied={} designs_skipped={} unmatched={:?}",
        designs.copied,
        designs.skipped,
        unmatched.as_ref().map(|u| u.unmatched),
    );

    emit_progress(
        &app_handle,
        restore::RestoreProgress {
            phase: "completed".to_string(),
            db_status: "restored".to_string(),
            scanned: designs.scanned,
            copied: designs.copied + designs.updated,
            skipped: designs.skipped,
            total_bytes: designs.total_bytes_copied,
            percent: 1.0,
            error: None,
        },
    );

    Ok(RestoreBothResult {
        database,
        designs,
        unmatched,
    })
}

/// Post-restore reconciliation scan: files on disk absent from the database.
#[tauri::command]
pub async fn detect_design_files_absent_from_database(
    state: State<'_, AppState>,
) -> Result<restore::DetectUnmatchedFilesResult, String> {
    let pool = state.db_pool()?;
    let dest_root = mnt::derive_designs_source_path();
    match restore::detect_design_files_absent_from_database(&pool, &dest_root).await {
        Ok(result) => {
            tracing::info!(
                "[restore] detect_design_files_absent_from_database checked={} unmatched={}",
                result.checked,
                result.unmatched,
            );
            Ok(result)
        }
        Err(error) => {
            tracing::error!("[restore] detect_design_files_absent_from_database failed: {error}");
            Err(error)
        }
    }
}

/// Batch import of unmatched design files as new catalogue records.
#[tauri::command]
pub async fn import_unmatched_design_files(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<restore::ImportUnmatchedFilesResult, String> {
    RESTORE_CANCEL_REQUESTED.store(false, Ordering::SeqCst);
    tracing::info!("[restore] import_unmatched_design_files requested");
    emit_progress(
        &app_handle,
        restore::RestoreProgress::new("import-unmatched", "starting"),
    );
    let pool = state.db_pool()?;
    let dest_root = mnt::derive_designs_source_path();
    match restore::import_unmatched_design_files(&pool, &dest_root).await {
        Ok(result) => {
            tracing::info!(
                "[restore] import_unmatched_design_files imported={} failed={}",
                result.imported,
                result.failed,
            );
            Ok(result)
        }
        Err(error) => {
            tracing::error!("[restore] import_unmatched_design_files failed: {error}");
            Err(error)
        }
    }
}

#[cfg(test)]
#[path = "restore_tests.rs"]
mod tests;
