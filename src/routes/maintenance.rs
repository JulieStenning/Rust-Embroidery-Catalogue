use crate::config::BootstrapConfig;
use crate::services::compaction::schedule_incremental_vacuum;
use crate::services::db_health;
use crate::services::folder_picker;
use crate::settings;
use crate::paths::normalize_path_display;
use crate::AppState;
use fs4::available_space;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqliteConnection, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, State};

pub(crate) const KEY_BACKUP_DATABASE_DESTINATION: &str = "backup.database_destination";
pub(crate) const KEY_BACKUP_DESIGNS_DESTINATION: &str = "backup.designs_destination";
const KEY_BACKUP_DATABASE_LAST_RUN_AT: &str = "backup.database_last_run_at";
const KEY_BACKUP_DESIGNS_LAST_RUN_AT: &str = "backup.designs_last_run_at";
const FILE_COMPARE_TIME_TOLERANCE_SECS: i64 = 2;

/// Cooperative cancellation flag observed by the running backup loops.
///
/// Follows the same process-wide atomic pattern as `backfill::STOP_REQUESTED`
/// and `bulk_import::BULK_IMPORT_STOP_REQUESTED`. Each public run command
/// clears the signal once at entry; the inner helpers observe but never clear
/// it so `run_both_backups` sees a single cancellation across both phases.
static BACKUP_CANCEL_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Event emitted to the frontend the moment the database phase of a combined
/// ("both") backup finishes successfully. The designs phase runs afterwards, so
/// the frontend uses this signal to switch its cancel-confirmation wording from
/// "the database copy is currently running" to "the database copy has
/// completed" while the modal is still open.
pub const EVENT_DATABASE_BACKUP_COMPLETED: &str = "database-backup-completed";

#[derive(Debug, Clone, Serialize)]
pub struct BackupViewModel {
    pub db_destination: String,
    pub designs_destination: String,
    pub db_source_path: String,
    pub designs_source_path: String,
    /// Epoch-seconds string of the last successful database backup (if any).
    pub db_last_backup_at: String,
    /// Epoch-seconds string of the last successful designs backup (if any).
    pub designs_last_backup_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveBackupSettingsRequest {
    pub db_destination: String,
    pub designs_destination: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveBackupSettingsResult {
    pub saved: bool,
    pub message: String,
    pub db_destination: String,
    pub designs_destination: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowseBackupFolderResult {
    pub path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseBackupResult {
    pub success: bool,
    pub backup_path: Option<String>,
    pub size_bytes: u64,
    pub completed_at: String,
    pub error: Option<String>,
    /// True when the run was aborted by the user via `request_cancel_backup`.
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignsBackupResult {
    pub success: bool,
    pub scanned: usize,
    pub copied: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub archived: usize,
    pub total_bytes_copied: u64,
    pub completed_at: String,
    pub error: Option<String>,
    /// True when the run was aborted by the user via `request_cancel_backup`.
    pub cancelled: bool,
}

/// Result of raising the cooperative backup cancellation flag.
#[derive(Debug, Clone, Serialize)]
pub struct CancelBackupResult {
    pub cancel_requested: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BothBackupsResult {
    pub database: DatabaseBackupResult,
    pub designs: DesignsBackupResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrphanScanResult {
    pub checked: usize,
    pub found: usize,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct OrphanDesignItem {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub designer: String,
    pub date_added: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrphansPageResult {
    pub items: Vec<OrphanDesignItem>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrphansPageRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteOrphansRequest {
    pub design_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteOrphansResult {
    pub deleted: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowseOrphanPathResult {
    pub ok: bool,
    pub opened: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrphanPathDebugItem {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub resolved_path: String,
    pub exists: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct FileSnapshot {
    pub(crate) full_path: PathBuf,
    pub(crate) size: u64,
    pub(crate) modified: Option<SystemTime>,
}

#[tauri::command]
pub fn maintenance_scaffold_enabled() -> bool {
    true
}

// â”€â”€â”€ Database statistics & manual compaction â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Storage metrics for the catalogue database, for the Settings UI.
#[derive(Debug, Clone, Serialize)]
pub struct DbStats {
    pub file_size_bytes: u64,
    pub page_count: i64,
    pub freelist_count: i64,
    pub page_size: i64,
    pub free_ratio: f64,
    pub reclaimable_bytes: u64,
}

/// Result of a successful manual `VACUUM` + `PRAGMA optimize` run.
#[derive(Debug, Clone, Serialize)]
pub struct CompactResult {
    pub file_size_before: u64,
    pub file_size_after: u64,
    pub pages_reclaimed: u64,
    pub duration_ms: u64,
}

/// Read the live database path from the bootstrap config.
fn database_path_from_bootstrap() -> PathBuf {
    let config = BootstrapConfig::from_env();
    PathBuf::from(strip_sqlite_prefix(&config.database_url))
}

/// Raise the cooperative backup cancellation flag.
///
/// The running backup loop observes the flag and aborts at the next safe
/// boundary: database partial files are removed; already-copied design files
/// are left intact.
#[tauri::command]
pub fn request_cancel_backup() -> Result<CancelBackupResult, String> {
    BACKUP_CANCEL_REQUESTED.store(true, Ordering::SeqCst);
    tracing::info!("[backup] Cancel requested by user");
    Ok(CancelBackupResult {
        cancel_requested: true,
    })
}

/// Clear the cooperative backup cancellation flag. Called once at the start
/// of each public run command so a fresh run never inherits a stale flag.
fn clear_backup_cancel_signal() {
    BACKUP_CANCEL_REQUESTED.store(false, Ordering::SeqCst);
}

/// Check whether a backup cancellation has been requested.
fn is_backup_cancel_requested() -> bool {
    BACKUP_CANCEL_REQUESTED.load(Ordering::SeqCst)
}

/// Return current storage metrics for the database: file size on disk plus
/// SQLite page/freelist counts and the recoverable freelist size.
#[tauri::command]
pub async fn get_db_stats(state: State<'_, AppState>) -> Result<DbStats, String> {
    let db_path = database_path_from_bootstrap();

    let file_size_bytes = fs::metadata(&db_path)
        .map_err(|e| format!("Failed to read database metadata: {e}"))?
        .len();

    let snapshot = db_health::get_freelist_metrics(&state.db_pool()?).await?;

    Ok(DbStats {
        file_size_bytes,
        page_count: snapshot.page_count,
        freelist_count: snapshot.freelist_count,
        page_size: snapshot.page_size,
        free_ratio: snapshot.free_ratio(),
        reclaimable_bytes: snapshot.reclaimable_bytes(),
    })
}

/// Manually compact the database: run a full `VACUUM` followed by
/// `PRAGMA optimize`, guarded by a disk-space safety check.
#[tauri::command]
pub async fn compact_database(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CompactResult, String> {
    let started = Instant::now();
    let db_path = database_path_from_bootstrap();

    let file_size_before = fs::metadata(&db_path)
        .map_err(|e| format!("Failed to read database metadata: {e}"))?
        .len();

    // Determine the parent directory to check disk space.
    let parent_dir = db_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let check_dir = if parent_dir.exists() {
        parent_dir.to_path_buf()
    } else {
        nearest_existing_folder(&db_path, Path::new(".")).to_path_buf()
    };

    let available = available_space(&check_dir)
        .map_err(|e| format!("Failed to query available disk space: {e}"))?;

    tracing::info!(
        "Manual DB compaction â€” file_size_before={}, free_space_on_volume={}",
        file_size_before,
        available
    );

    // Safety check: VACUUM needs headroom for its temporary rewrite.
    if available < file_size_before {
        let msg = format!(
            "Insufficient disk space to compact the database. Need at least {} bytes free but only {} are available.",
            file_size_before, available
        );
        tracing::warn!("{}", msg);
        let _ = app_handle.emit(
            db_health::EVENT_MAINTENANCE_FINISHED,
            serde_json::json!({ "error": msg }),
        );
        return Err(msg);
    }

    // Run the full VACUUM (blocking rewrite) then PRAGMA optimize.
    sqlx::query("VACUUM")
        .execute(&state.db_pool()?)
        .await
        .map_err(|e| format!("VACUUM failed: {e}"))?;

    sqlx::query("PRAGMA optimize")
        .execute(&state.db_pool()?)
        .await
        .map_err(|e| format!("PRAGMA optimize failed: {e}"))?;

    let duration_ms = started.elapsed().as_millis() as u64;

    // Re-measure the freelist delta for reporting.
    let snapshot = db_health::get_freelist_metrics(&state.db_pool()?).await?;
    let file_size_after = fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(file_size_before);
    let pages_reclaimed = snapshot.freelist_count.max(0) as u64;

    let result = CompactResult {
        file_size_before,
        file_size_after,
        pages_reclaimed,
        duration_ms,
    };

    tracing::info!(
        "Manual DB compaction complete â€” file_size_after={}, duration_ms={}",
        file_size_after,
        duration_ms
    );

    let _ = app_handle.emit(db_health::EVENT_MAINTENANCE_FINISHED, &result);

    Ok(result)
}

#[tauri::command]
pub async fn scan_orphans(state: State<'_, AppState>) -> Result<OrphanScanResult, String> {
    let pool = state.db_pool()?;
    let base_path = derive_designs_source_path();

    scan_orphans_with_pool(&pool, &base_path).await
}

#[tauri::command]
pub async fn get_orphans_page(
    state: State<'_, AppState>,
    request: Option<GetOrphansPageRequest>,
) -> Result<OrphansPageResult, String> {
    let pool = state.db_pool()?;
    let base_path = derive_designs_source_path();

    get_orphans_page_with_pool(&pool, &base_path, request).await
}

#[tauri::command]
pub async fn delete_orphans(
    state: State<'_, AppState>,
    request: DeleteOrphansRequest,
) -> Result<DeleteOrphansResult, String> {
    let pool = state.db_pool()?;
    let deleted = delete_design_ids_with_pool(&pool, &request.design_ids).await?;

    // Reclaim freelist pages asynchronously after the orphan delete commits,
    // so the UI never blocks on database file compaction.
    schedule_incremental_vacuum(pool.clone());

    Ok(DeleteOrphansResult { deleted })
}

#[tauri::command]
pub async fn delete_all_orphans(state: State<'_, AppState>) -> Result<DeleteOrphansResult, String> {
    let pool = state.db_pool()?;
    let base_path = derive_designs_source_path();

    let orphan_ids = find_orphan_ids_with_pool(&pool, &base_path).await?;
    let deleted = delete_design_ids_with_pool(&pool, &orphan_ids).await?;

    // Reclaim freelist pages asynchronously after the orphan delete commits,
    // so the UI never blocks on database file compaction.
    schedule_incremental_vacuum(pool.clone());

    Ok(DeleteOrphansResult { deleted })
}

#[tauri::command]
pub fn browse_orphan_path(filepath: String) -> Result<BrowseOrphanPathResult, String> {
    let base_path = derive_designs_source_path();
    let target = resolve_design_full_path(&base_path, &filepath);
    let folder = nearest_existing_folder(&target, &base_path);

    if !external_launches_disabled() {
        let _ = open_folder_in_explorer(&folder);
    }

    Ok(BrowseOrphanPathResult {
        ok: true,
        opened: normalize_path_display(&folder),
    })
}

#[tauri::command]
pub async fn get_backup_view_model(state: State<'_, AppState>) -> Result<BackupViewModel, String> {
    let pool = state.db_pool()?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;

    let db_destination = get_setting_with_default(&mut conn, KEY_BACKUP_DATABASE_DESTINATION)
        .await
        .map_err(|e| e.to_string())?;
    let designs_destination = get_setting_with_default(&mut conn, KEY_BACKUP_DESIGNS_DESTINATION)
        .await
        .map_err(|e| e.to_string())?;
    let db_last_backup_at = get_setting_with_default(&mut conn, KEY_BACKUP_DATABASE_LAST_RUN_AT)
        .await
        .map_err(|e| e.to_string())?;
    let designs_last_backup_at = get_setting_with_default(&mut conn, KEY_BACKUP_DESIGNS_LAST_RUN_AT)
        .await
        .map_err(|e| e.to_string())?;

    let db_source = derive_database_source_path();
    let designs_source = derive_designs_source_path();

    Ok(BackupViewModel {
        db_destination,
        designs_destination,
        db_source_path: normalize_path_display(&db_source),
        designs_source_path: normalize_path_display(&designs_source),
        db_last_backup_at,
        designs_last_backup_at,
    })
}

#[tauri::command]
pub async fn save_backup_settings(
    state: State<'_, AppState>,
    request: SaveBackupSettingsRequest,
) -> Result<SaveBackupSettingsResult, String> {
    let db_destination = request.db_destination.trim().to_string();
    let designs_destination = request.designs_destination.trim().to_string();

    let pool = state.db_pool()?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;

    upsert_setting(&mut conn, KEY_BACKUP_DATABASE_DESTINATION, &db_destination)
        .await
        .map_err(|e| e.to_string())?;
    upsert_setting(
        &mut conn,
        KEY_BACKUP_DESIGNS_DESTINATION,
        &designs_destination,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(SaveBackupSettingsResult {
        saved: true,
        message: "Backup destinations saved.".to_string(),
        db_destination,
        designs_destination,
    })
}

#[tauri::command]
pub fn browse_backup_folder(start_dir: Option<String>) -> BrowseBackupFolderResult {
    match folder_picker::browse_folder_with_error(start_dir.as_deref(), false) {
        Ok(result) => BrowseBackupFolderResult {
            path: result.path,
            error: None,
        },
        Err(error) => BrowseBackupFolderResult {
            path: None,
            error: Some(error.to_string()),
        },
    }
}

/// Identifier helpers used by the cancellation-aware result constructors.
fn cancelled_database_backup(completed_at: &str, error: Option<String>) -> DatabaseBackupResult {
    DatabaseBackupResult {
        success: false,
        backup_path: None,
        size_bytes: 0,
        completed_at: completed_at.to_string(),
        error,
        cancelled: true,
    }
}

fn cancelled_designs_backup(completed_at: &str, error: Option<String>) -> DesignsBackupResult {
    DesignsBackupResult {
        success: false,
        scanned: 0,
        copied: 0,
        updated: 0,
        unchanged: 0,
        archived: 0,
        total_bytes_copied: 0,
        completed_at: completed_at.to_string(),
        error,
        cancelled: true,
    }
}

/// Clear the cancellation flag and run the database backup.
#[tauri::command]
pub async fn run_database_backup(
    state: State<'_, AppState>,
) -> Result<DatabaseBackupResult, String> {
    clear_backup_cancel_signal();
    run_database_backup_inner(&state.db_pool()?).await
}

/// Core database backup logic. Observes (but never clears) the cancellation
/// flag so `run_both_backups` can share one signal across both phases.
///
/// On cancellation the partially written `.db` file in the destination is
/// removed immediately and a `cancelled` result is returned.
async fn run_database_backup_inner(pool: &SqlitePool) -> Result<DatabaseBackupResult, String> {
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let completed_at = current_epoch_seconds_string();

    let db_destination_raw = get_setting_with_default(&mut conn, KEY_BACKUP_DATABASE_DESTINATION)
        .await
        .map_err(|e| e.to_string())?;
    let db_destination = db_destination_raw.trim();
    if db_destination.is_empty() {
        return Ok(DatabaseBackupResult {
            success: false,
            backup_path: None,
            size_bytes: 0,
            completed_at,
            error: Some(
                "No database backup destination configured. Save a destination first.".to_string(),
            ),
            cancelled: false,
        });
    }

    let source_db_path = derive_database_source_path();
    if !source_db_path.exists() {
        return Ok(DatabaseBackupResult {
            success: false,
            backup_path: None,
            size_bytes: 0,
            completed_at,
            error: Some(format!(
                "Database source not found: {}",
                normalize_path_display(&source_db_path)
            )),
            cancelled: false,
        });
    }

    let destination_dir = PathBuf::from(db_destination);
    if let Err(error) = ensure_writable_directory(&destination_dir) {
        return Ok(DatabaseBackupResult {
            success: false,
            backup_path: None,
            size_bytes: 0,
            completed_at,
            error: Some(error),
            cancelled: false,
        });
    }

    let timestamp = sqlite_localtime_format(&mut conn, "%Y-%m-%d_%H%M")
        .await
        .unwrap_or_else(|_| fallback_filename_timestamp());
    let destination_path =
        unique_path_with_suffix(destination_dir.join(format!("catalogue_{}.db", timestamp)));

    // Bail out before writing anything if cancellation was requested early.
    if is_backup_cancel_requested() {
        tracing::info!("[backup] Database backup cancelled before copy");
        return Ok(cancelled_database_backup(
            &completed_at,
            Some("Database backup cancelled.".to_string()),
        ));
    }

    let escaped_destination = destination_path.to_string_lossy().replace('\'', "''");
    let vacuum_sql = format!("VACUUM INTO '{}'", escaped_destination);
    let db_backup_result = sqlx::query(sqlx::AssertSqlSafe(vacuum_sql))
        .execute(&mut *conn)
        .await;

    if db_backup_result.is_err() {
        if is_backup_cancel_requested() {
            cleanup_maybe_partial_backup(&destination_path);
            tracing::info!("[backup] Database backup cancelled during fallback copy");
            return Ok(cancelled_database_backup(
                &completed_at,
                Some("Database backup cancelled.".to_string()),
            ));
        }

        if let Err(copy_error) = fs::copy(&source_db_path, &destination_path) {
            if is_backup_cancel_requested() {
                cleanup_maybe_partial_backup(&destination_path);
                tracing::info!("[backup] Database backup cancelled after failed copy");
                return Ok(cancelled_database_backup(
                    &completed_at,
                    Some("Database backup cancelled.".to_string()),
                ));
            }

            return Ok(DatabaseBackupResult {
                success: false,
                backup_path: None,
                size_bytes: 0,
                completed_at,
                error: Some(format!("Could not create database backup: {}", copy_error)),
                cancelled: false,
            });
        }
    }

    // Cancellation after the write finished: remove the partial file and report.
    if is_backup_cancel_requested() {
        cleanup_maybe_partial_backup(&destination_path);
        tracing::info!("[backup] Database backup cancelled after copy completed");
        return Ok(cancelled_database_backup(
            &completed_at,
            Some("Database backup cancelled.".to_string()),
        ));
    }

    let size_bytes = fs::metadata(&destination_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    // Record the last-run timestamp so the UI can display it (persisted in settings).
    let _ = upsert_setting(&mut conn, KEY_BACKUP_DATABASE_LAST_RUN_AT, &completed_at).await;

    Ok(DatabaseBackupResult {
        success: true,
        backup_path: Some(normalize_path_display(&destination_path)),
        size_bytes,
        completed_at,
        error: None,
        cancelled: false,
    })
}

/// Best-effort removal of a partially written database backup file.
fn cleanup_maybe_partial_backup(destination_path: &Path) {
    if destination_path.exists() {
        if let Err(error) = fs::remove_file(destination_path) {
            tracing::error!(
                "[backup] Could not remove partial database backup '{}': {}",
                normalize_path_display(destination_path),
                error
            );
        } else {
            tracing::info!(
                "[backup] Removed partial database backup '{}'",
                normalize_path_display(destination_path)
            );
        }
    }
}

/// Clear the cancellation flag and run the designs backup.
#[tauri::command]
pub async fn run_designs_backup(state: State<'_, AppState>) -> Result<DesignsBackupResult, String> {
    clear_backup_cancel_signal();
    run_designs_backup_inner(&state.db_pool()?).await
}

/// Core designs backup logic. Observes (but never clears) the cancellation
/// flag so `run_both_backups` can share one signal across both phases.
///
/// On cancellation the copy loop stops at the next file boundary. Files
/// already copied remain in the destination; the archive and directory
/// cleanup phases are skipped so cancellation never moves or removes output.
async fn run_designs_backup_inner(pool: &SqlitePool) -> Result<DesignsBackupResult, String> {
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let completed_at = current_epoch_seconds_string();

    let destination_raw = get_setting_with_default(&mut conn, KEY_BACKUP_DESIGNS_DESTINATION)
        .await
        .map_err(|e| e.to_string())?;
    let destination_str = destination_raw.trim();
    if destination_str.is_empty() {
        return Ok(DesignsBackupResult {
            success: false,
            scanned: 0,
            copied: 0,
            updated: 0,
            unchanged: 0,
            archived: 0,
            total_bytes_copied: 0,
            completed_at,
            error: Some(
                "No designs backup destination configured. Save a destination first.".to_string(),
            ),
            cancelled: false,
        });
    }

    let source_root = derive_designs_source_path();
    if !source_root.exists() {
        return Ok(DesignsBackupResult {
            success: false,
            scanned: 0,
            copied: 0,
            updated: 0,
            unchanged: 0,
            archived: 0,
            total_bytes_copied: 0,
            completed_at,
            error: Some(format!(
                "Designs source folder not found: {}",
                normalize_path_display(&source_root)
            )),
            cancelled: false,
        });
    }

    let destination_root = PathBuf::from(destination_str);
    if let Err(error) = ensure_writable_directory(&destination_root) {
        return Ok(DesignsBackupResult {
            success: false,
            scanned: 0,
            copied: 0,
            updated: 0,
            unchanged: 0,
            archived: 0,
            total_bytes_copied: 0,
            completed_at,
            error: Some(error),
            cancelled: false,
        });
    }

    let source_map = match collect_file_snapshots(&source_root, true) {
        Ok(map) => map,
        Err(error) => {
            return Ok(DesignsBackupResult {
                success: false,
                scanned: 0,
                copied: 0,
                updated: 0,
                unchanged: 0,
                archived: 0,
                total_bytes_copied: 0,
                completed_at,
                error: Some(format!("Could not scan designs source: {}", error)),
                cancelled: false,
            })
        }
    };

    let backup_map = match collect_file_snapshots(&destination_root, true) {
        Ok(map) => map,
        Err(error) => {
            return Ok(DesignsBackupResult {
                success: false,
                scanned: 0,
                copied: 0,
                updated: 0,
                unchanged: 0,
                archived: 0,
                total_bytes_copied: 0,
                completed_at,
                error: Some(format!("Could not scan backup destination: {}", error)),
                cancelled: false,
            })
        }
    };

    let mut copied = 0usize;
    let mut updated = 0usize;
    let mut unchanged = 0usize;
    let mut archived = 0usize;
    let mut total_bytes_copied = 0u64;

    for (relative_path, source_snapshot) in &source_map {
        // Stop copying further files as soon as cancellation is requested;
        // already-copied files are left in place.
        if is_backup_cancel_requested() {
            tracing::info!("[backup] Designs backup cancellation observed during copy loop");
            break;
        }

        let destination_path = destination_root.join(relative_path);

        let should_copy = match backup_map.get(relative_path) {
            Some(existing_snapshot) => {
                if files_match(existing_snapshot, source_snapshot) {
                    unchanged += 1;
                    false
                } else {
                    updated += 1;
                    true
                }
            }
            None => {
                copied += 1;
                true
            }
        };

        if !should_copy {
            continue;
        }

        if let Some(parent) = destination_path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                tracing::error!(
                    "[backup] Could not create destination folder '{}': {}",
                    normalize_path_display(parent),
                    error
                );
                continue;
            }
        }

        match fs::copy(&source_snapshot.full_path, &destination_path) {
            Ok(bytes) => total_bytes_copied = total_bytes_copied.saturating_add(bytes),
            Err(error) => {
                tracing::error!(
                    "[backup] Could not copy '{}' to '{}': {}",
                    normalize_path_display(&source_snapshot.full_path),
                    normalize_path_display(&destination_path),
                    error
                );
            }
        }
    }

    // If cancellation arrived, skip the archive (rename/move) phase entirely
    // and report a cancelled result. Never move or remove destination output.
    if is_backup_cancel_requested() {
        tracing::info!("[backup] Designs backup cancelled; skipping archive phase");
        return Ok(DesignsBackupResult {
            success: false,
            scanned: source_map.len(),
            copied,
            updated,
            unchanged,
            archived: 0,
            total_bytes_copied,
            completed_at,
            error: Some("Designs backup cancelled.".to_string()),
            cancelled: true,
        });
    }

    let source_keys = source_map.keys().cloned().collect::<HashSet<PathBuf>>();
    let archive_date = sqlite_localtime_format(&mut conn, "%Y-%m-%d")
        .await
        .unwrap_or_else(|_| "unknown-date".to_string());
    let archive_root = destination_root.join("_deleted").join(archive_date);

    for (relative_path, snapshot) in &backup_map {
        if source_keys.contains(relative_path) {
            continue;
        }

        let archive_path = archive_root.join(relative_path);
        if let Some(parent) = archive_path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                tracing::error!(
                    "[backup] Could not create archive folder '{}': {}",
                    normalize_path_display(parent),
                    error
                );
                continue;
            }
        }

        if archive_path.exists() {
            let _ = fs::remove_file(&archive_path);
        }

        match fs::rename(&snapshot.full_path, &archive_path) {
            Ok(_) => archived += 1,
            Err(error) => {
                tracing::error!(
                    "[backup] Could not archive '{}' to '{}': {}",
                    normalize_path_display(&snapshot.full_path),
                    normalize_path_display(&archive_path),
                    error
                );
            }
        }
    }

    if let Err(error) =
        cleanup_empty_directories(&destination_root, &destination_root.join("_deleted"), true)
    {
        tracing::error!(
            "[backup] Could not clean up empty directories under '{}': {}",
            normalize_path_display(&destination_root),
            error
        );
    }

    // Record the last-run timestamp so the UI can display it (persisted in settings).
    let _ = upsert_setting(&mut conn, KEY_BACKUP_DESIGNS_LAST_RUN_AT, &completed_at).await;

    Ok(DesignsBackupResult {
        success: true,
        scanned: source_map.len(),
        copied,
        updated,
        unchanged,
        archived,
        total_bytes_copied,
        completed_at,
        error: None,
        cancelled: false,
    })
}

/// Clear the cancellation flag once, then run both phases sequentially against
/// the same shared flag. If cancellation is requested during the database
/// phase, the designs phase is skipped entirely and reported as cancelled.
///
/// Emits [`EVENT_DATABASE_BACKUP_COMPLETED`] as soon as the database phase
/// finishes successfully, before the designs phase starts, so the frontend can
/// reflect the now-completed database copy in its cancellation prompt.
#[tauri::command]
pub async fn run_both_backups(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<BothBackupsResult, String> {
    clear_backup_cancel_signal();

    let database = run_database_backup_inner(&state.db_pool()?).await?;

    if database.success {
        let _ = app_handle.emit(EVENT_DATABASE_BACKUP_COMPLETED, ());
    }

    let designs = if is_backup_cancel_requested() {
        cancelled_designs_backup(&current_epoch_seconds_string(), None)
    } else {
        run_designs_backup_inner(&state.db_pool()?).await?
    };

    Ok(BothBackupsResult { database, designs })
}

pub(crate) async fn get_setting_with_default(
    conn: &mut SqliteConnection,
    key: &str,
) -> Result<String, sqlx::Error> {
    if let Some(setting) = settings::get_setting(conn, key).await? {
        return Ok(setting.value);
    }

    upsert_setting(conn, key, "").await?;
    Ok("".to_string())
}

async fn upsert_setting(
    conn: &mut SqliteConnection,
    key: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings (key, value, description) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .bind(setting_description_for_key(key))
    .execute(conn)
    .await?;

    Ok(())
}

fn setting_description_for_key(key: &str) -> &'static str {
    match key {
        KEY_BACKUP_DATABASE_DESTINATION => "Destination folder for timestamped database backups.",
        KEY_BACKUP_DESIGNS_DESTINATION => "Destination folder for incremental designs backups.",
        _ => "",
    }
}

pub(crate) fn derive_database_source_path() -> PathBuf {
    let config = BootstrapConfig::from_env();
    PathBuf::from(strip_sqlite_prefix(&config.database_url))
}

pub(crate) fn derive_data_root_path() -> PathBuf {
    let db_path = derive_database_source_path();

    let root = if let Some(parent) = db_path.parent() {
        if parent
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case("database"))
            .unwrap_or(false)
        {
            parent.parent().unwrap_or(parent).to_path_buf()
        } else {
            parent.to_path_buf()
        }
    } else {
        PathBuf::from("data")
    };

    root.canonicalize().unwrap_or(root)
}

pub(crate) fn derive_designs_source_path() -> PathBuf {
    derive_data_root_path().join("MachineEmbroideryDesigns")
}

pub(crate) fn resolve_design_full_path(base_path: &Path, stored_filepath: &str) -> PathBuf {
    // Delegate to the shared single source of truth: handles canonical
    // library-relative paths, legacy `/MachineEmbroideryDesigns/…` forms, and
    // absolute rows (returned as-is).
    crate::paths::resolve_design_filepath(stored_filepath, base_path)
}

fn nearest_existing_folder(path: &Path, fallback: &Path) -> PathBuf {
    let mut candidate = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| fallback.to_path_buf())
    };

    loop {
        if candidate.is_dir() {
            return candidate;
        }

        let Some(parent) = candidate.parent() else {
            break;
        };

        if parent == candidate {
            break;
        }

        candidate = parent.to_path_buf();
    }

    fallback.to_path_buf()
}

fn open_folder_in_explorer(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|error| format!("Failed to open Explorer: {}", error))?;
        return Ok(());
    }

    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|error| format!("Failed to open folder: {}", error))?;
        return Ok(());
    }

    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|error| format!("Failed to open folder: {}", error))?;

    Ok(())
}

fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "accepted"
    )
}

fn external_launches_disabled() -> bool {
    if let Ok(value) = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN") {
        if is_truthy(&value) {
            return true;
        }
    }

    false
}

async fn scan_orphans_with_pool(
    pool: &SqlitePool,
    base_path: &Path,
) -> Result<OrphanScanResult, String> {
    let rows = sqlx::query_as::<_, (String,)>("SELECT filepath FROM designs")
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?;

    let mut checked = 0usize;
    let mut found = 0usize;
    for (filepath,) in rows {
        if filepath.trim().is_empty() {
            continue;
        }

        checked = checked.saturating_add(1);
        let full_path = resolve_design_full_path(base_path, &filepath);
        if !full_path.is_file() {
            found = found.saturating_add(1);
        }
    }

    Ok(OrphanScanResult { checked, found })
}

async fn find_orphan_ids_with_pool(
    pool: &SqlitePool,
    base_path: &Path,
) -> Result<Vec<i64>, String> {
    let rows =
        sqlx::query_as::<_, (i64, String)>("SELECT id, filepath FROM designs ORDER BY filepath")
            .fetch_all(pool)
            .await
            .map_err(|error| error.to_string())?;

    let mut orphan_ids = Vec::new();
    for (id, filepath) in rows {
        if filepath.trim().is_empty() {
            continue;
        }

        let full_path = resolve_design_full_path(base_path, &filepath);
        if !full_path.is_file() {
            orphan_ids.push(id);
        }
    }

    Ok(orphan_ids)
}

async fn get_orphans_page_with_pool(
    pool: &SqlitePool,
    base_path: &Path,
    request: Option<GetOrphansPageRequest>,
) -> Result<OrphansPageResult, String> {
    let page = request
        .as_ref()
        .and_then(|item| item.page)
        .unwrap_or(1)
        .max(1);
    let page_size = request
        .as_ref()
        .and_then(|item| item.page_size)
        .unwrap_or(100)
        .clamp(1, 500);

    let orphan_ids = find_orphan_ids_with_pool(pool, base_path).await?;
    let total = i64::try_from(orphan_ids.len()).map_err(|error| error.to_string())?;
    let total_pages = if total == 0 {
        1
    } else {
        (total + page_size - 1) / page_size
    };

    let normalized_page = page.min(total_pages.max(1));
    let offset =
        usize::try_from((normalized_page - 1) * page_size).map_err(|error| error.to_string())?;
    let take = usize::try_from(page_size).map_err(|error| error.to_string())?;

    let page_ids: Vec<i64> = orphan_ids.into_iter().skip(offset).take(take).collect();
    if page_ids.is_empty() {
        return Ok(OrphansPageResult {
            items: Vec::new(),
            page: normalized_page,
            page_size,
            total,
            total_pages,
        });
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT d.id, d.filename, d.filepath, COALESCE(designers.name, '') AS designer, d.date_added AS date_added \
         FROM designs d \
         LEFT JOIN designers ON designers.id = d.designer_id \
         WHERE d.id IN (",
    );

    {
        let mut separated = query.separated(", ");
        for design_id in &page_ids {
            separated.push_bind(*design_id);
        }
    }

    query.push(") ORDER BY d.filepath");

    let items = query
        .build_query_as::<OrphanDesignItem>()
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?;

    Ok(OrphansPageResult {
        items,
        page: normalized_page,
        page_size,
        total,
        total_pages,
    })
}

async fn delete_design_ids_with_pool(
    pool: &SqlitePool,
    design_ids: &[i64],
) -> Result<usize, String> {
    if design_ids.is_empty() {
        return Ok(0);
    }

    let mut deleted = 0usize;
    for chunk in design_ids.chunks(500) {
        let mut query = QueryBuilder::<Sqlite>::new("DELETE FROM designs WHERE id IN (");
        {
            let mut separated = query.separated(", ");
            for design_id in chunk {
                separated.push_bind(*design_id);
            }
        }
        query.push(")");

        let result = query
            .build()
            .execute(pool)
            .await
            .map_err(|error| error.to_string())?;
        deleted = deleted.saturating_add(result.rows_affected() as usize);
    }

    Ok(deleted)
}

fn strip_sqlite_prefix(database_url: &str) -> &str {
    database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url)
}

pub(crate) fn current_epoch_seconds_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn fallback_filename_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn unique_path_with_suffix(base_path: PathBuf) -> PathBuf {
    if !base_path.exists() {
        return base_path;
    }

    let stem = base_path
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "catalogue_backup".to_string());
    let extension = base_path
        .extension()
        .map(|value| value.to_string_lossy().to_string());
    let parent = base_path
        .parent()
        .map(|value| value.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    for index in 1..=10_000 {
        let mut candidate_name = format!("{}_{}", stem, index);
        if let Some(ext) = &extension {
            candidate_name.push('.');
            candidate_name.push_str(ext);
        }

        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    base_path
}

pub(crate) fn ensure_writable_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| {
        format!(
            "Could not create destination '{}': {}",
            normalize_path_display(path),
            error
        )
    })?;

    let probe = path.join(".backup-write-test.tmp");
    fs::write(&probe, b"ok").map_err(|error| {
        format!(
            "Destination is not writable '{}': {}",
            normalize_path_display(path),
            error
        )
    })?;
    let _ = fs::remove_file(&probe);

    Ok(())
}

async fn sqlite_localtime_format(
    conn: &mut SqliteConnection,
    format: &str,
) -> Result<String, String> {
    let value = sqlx::query_scalar::<_, Option<String>>("SELECT strftime(?, 'now', 'localtime')")
        .bind(format)
        .fetch_one(conn)
        .await
        .map_err(|error| error.to_string())?
        .unwrap_or_default();

    if value.trim().is_empty() {
        return Err("Could not format local timestamp".to_string());
    }

    Ok(value)
}

pub(crate) fn collect_file_snapshots(
    root: &Path,
    skip_deleted_tree: bool,
) -> Result<HashMap<PathBuf, FileSnapshot>, String> {
    let mut map = HashMap::new();
    if !root.exists() {
        return Ok(map);
    }

    collect_file_snapshots_recursive(root, root, skip_deleted_tree, &mut map)?;
    Ok(map)
}

pub(crate) fn collect_file_snapshots_recursive(
    root: &Path,
    current: &Path,
    skip_deleted_tree: bool,
    map: &mut HashMap<PathBuf, FileSnapshot>,
) -> Result<(), String> {
    for entry in fs::read_dir(current).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;

        let relative = path
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_path_buf();
        if skip_deleted_tree && relative.starts_with("_deleted") {
            continue;
        }

        if file_type.is_dir() {
            collect_file_snapshots_recursive(root, &path, skip_deleted_tree, map)?;
            continue;
        }

        if file_type.is_file() {
            let metadata = entry.metadata().map_err(|error| error.to_string())?;
            map.insert(
                relative,
                FileSnapshot {
                    full_path: path,
                    size: metadata.len(),
                    modified: metadata.modified().ok(),
                },
            );
        }
    }

    Ok(())
}

pub(crate) fn files_match(left: &FileSnapshot, right: &FileSnapshot) -> bool {
    if left.size != right.size {
        return false;
    }

    match (
        modified_epoch_seconds(left.modified),
        modified_epoch_seconds(right.modified),
    ) {
        (Some(left_secs), Some(right_secs)) => {
            (left_secs - right_secs).abs() <= FILE_COMPARE_TIME_TOLERANCE_SECS
        }
        _ => false,
    }
}

fn modified_epoch_seconds(value: Option<SystemTime>) -> Option<i64> {
    value.and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| i64::try_from(duration.as_secs()).ok())
    })
}

fn cleanup_empty_directories(
    root: &Path,
    preserve_root: &Path,
    is_root: bool,
) -> Result<(), String> {
    if !root.is_dir() {
        return Ok(());
    }

    if root.starts_with(preserve_root) {
        return Ok(());
    }

    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            cleanup_empty_directories(&path, preserve_root, false)?;
        }
    }

    if !is_root {
        let mut iter = fs::read_dir(root).map_err(|error| error.to_string())?;
        if iter.next().is_none() {
            fs::remove_dir(root).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}
#[cfg(test)]
#[path = "maintenance_tests.rs"]
mod tests;
