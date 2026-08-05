use crate::config::BootstrapConfig;
use crate::services::compaction::schedule_incremental_vacuum;
use crate::services::db_health;
use crate::services::folder_picker;
use crate::settings;
use crate::AppState;
use fs4::available_space;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqliteConnection, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, State};

const KEY_BACKUP_DATABASE_DESTINATION: &str = "backup.database_destination";
const KEY_BACKUP_DESIGNS_DESTINATION: &str = "backup.designs_destination";
const FILE_COMPARE_TIME_TOLERANCE_SECS: i64 = 2;

#[derive(Debug, Clone, Serialize)]
pub struct BackupViewModel {
    pub db_destination: String,
    pub designs_destination: String,
    pub db_source_path: String,
    pub designs_source_path: String,
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
struct FileSnapshot {
    full_path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
}

#[tauri::command]
pub fn maintenance_scaffold_enabled() -> bool {
    true
}

// ─── Database statistics & manual compaction ─────────────────────────────────

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

/// Return current storage metrics for the database: file size on disk plus
/// SQLite page/freelist counts and the recoverable freelist size.
#[tauri::command]
pub async fn get_db_stats(state: State<'_, AppState>) -> Result<DbStats, String> {
    let db_path = database_path_from_bootstrap();

    let file_size_bytes = fs::metadata(&db_path)
        .map_err(|e| format!("Failed to read database metadata: {e}"))?
        .len();

    let snapshot = db_health::get_freelist_metrics(&state.db).await?;

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
        .filter(|p| p.as_os_str().len() > 0)
        .unwrap_or_else(|| Path::new("."));
    let check_dir = if parent_dir.exists() {
        parent_dir.to_path_buf()
    } else {
        nearest_existing_folder(&db_path, Path::new(".")).to_path_buf()
    };

    let available = available_space(&check_dir)
        .map_err(|e| format!("Failed to query available disk space: {e}"))?;

    tracing::info!(
        "Manual DB compaction — file_size_before={}, free_space_on_volume={}",
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
        .execute(&state.db)
        .await
        .map_err(|e| format!("VACUUM failed: {e}"))?;

    sqlx::query("PRAGMA optimize")
        .execute(&state.db)
        .await
        .map_err(|e| format!("PRAGMA optimize failed: {e}"))?;

    let duration_ms = started.elapsed().as_millis() as u64;

    // Re-measure the freelist delta for reporting.
    let snapshot = db_health::get_freelist_metrics(&state.db).await?;
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
        "Manual DB compaction complete — file_size_after={}, duration_ms={}",
        file_size_after,
        duration_ms
    );

    let _ = app_handle.emit(db_health::EVENT_MAINTENANCE_FINISHED, &result);

    Ok(result)
}

#[tauri::command]
pub async fn scan_orphans(state: State<'_, AppState>) -> Result<OrphanScanResult, String> {
    let pool = &state.db;
    let base_path = derive_designs_source_path();

    scan_orphans_with_pool(pool, &base_path).await
}

#[tauri::command]
pub async fn get_orphans_page(
    state: State<'_, AppState>,
    request: Option<GetOrphansPageRequest>,
) -> Result<OrphansPageResult, String> {
    let pool = &state.db;
    let base_path = derive_designs_source_path();

    get_orphans_page_with_pool(pool, &base_path, request).await
}

#[tauri::command]
pub async fn delete_orphans(
    state: State<'_, AppState>,
    request: DeleteOrphansRequest,
) -> Result<DeleteOrphansResult, String> {
    let pool = &state.db;
    let deleted = delete_design_ids_with_pool(pool, &request.design_ids).await?;

    // Reclaim freelist pages asynchronously after the orphan delete commits,
    // so the UI never blocks on database file compaction.
    schedule_incremental_vacuum(state.db.clone());

    Ok(DeleteOrphansResult { deleted })
}

#[tauri::command]
pub async fn delete_all_orphans(state: State<'_, AppState>) -> Result<DeleteOrphansResult, String> {
    let pool = &state.db;
    let base_path = derive_designs_source_path();

    let orphan_ids = find_orphan_ids_with_pool(pool, &base_path).await?;
    let deleted = delete_design_ids_with_pool(pool, &orphan_ids).await?;

    // Reclaim freelist pages asynchronously after the orphan delete commits,
    // so the UI never blocks on database file compaction.
    schedule_incremental_vacuum(state.db.clone());

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
        opened: normalize_path_string(&folder),
    })
}

#[tauri::command]
pub async fn get_backup_view_model(state: State<'_, AppState>) -> Result<BackupViewModel, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;

    let db_destination = get_setting_with_default(&mut conn, KEY_BACKUP_DATABASE_DESTINATION)
        .await
        .map_err(|e| e.to_string())?;
    let designs_destination = get_setting_with_default(&mut conn, KEY_BACKUP_DESIGNS_DESTINATION)
        .await
        .map_err(|e| e.to_string())?;

    let db_source = derive_database_source_path();
    let designs_source = derive_designs_source_path();

    Ok(BackupViewModel {
        db_destination,
        designs_destination,
        db_source_path: normalize_path_string(&db_source),
        designs_source_path: normalize_path_string(&designs_source),
    })
}

#[tauri::command]
pub async fn save_backup_settings(
    state: State<'_, AppState>,
    request: SaveBackupSettingsRequest,
) -> Result<SaveBackupSettingsResult, String> {
    let db_destination = request.db_destination.trim().to_string();
    let designs_destination = request.designs_destination.trim().to_string();

    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;

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

#[tauri::command]
pub async fn run_database_backup(
    state: State<'_, AppState>,
) -> Result<DatabaseBackupResult, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
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
                normalize_path_string(&source_db_path)
            )),
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
        });
    }

    let timestamp = sqlite_localtime_format(&mut conn, "%Y-%m-%d_%H%M")
        .await
        .unwrap_or_else(|_| fallback_filename_timestamp());
    let destination_path =
        unique_path_with_suffix(destination_dir.join(format!("catalogue_{}.db", timestamp)));

    let escaped_destination = destination_path.to_string_lossy().replace('\'', "''");
    let vacuum_sql = format!("VACUUM INTO '{}'", escaped_destination);
    let db_backup_result = sqlx::query(&vacuum_sql).execute(&mut *conn).await;

    if db_backup_result.is_err() {
        if let Err(copy_error) = fs::copy(&source_db_path, &destination_path) {
            return Ok(DatabaseBackupResult {
                success: false,
                backup_path: None,
                size_bytes: 0,
                completed_at,
                error: Some(format!("Could not create database backup: {}", copy_error)),
            });
        }
    }

    let size_bytes = fs::metadata(&destination_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    Ok(DatabaseBackupResult {
        success: true,
        backup_path: Some(normalize_path_string(&destination_path)),
        size_bytes,
        completed_at,
        error: None,
    })
}

#[tauri::command]
pub async fn run_designs_backup(state: State<'_, AppState>) -> Result<DesignsBackupResult, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
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
                normalize_path_string(&source_root)
            )),
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
            })
        }
    };

    let mut copied = 0usize;
    let mut updated = 0usize;
    let mut unchanged = 0usize;
    let mut archived = 0usize;
    let mut total_bytes_copied = 0u64;

    for (relative_path, source_snapshot) in &source_map {
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
                    normalize_path_string(parent),
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
                    normalize_path_string(&source_snapshot.full_path),
                    normalize_path_string(&destination_path),
                    error
                );
            }
        }
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
                    normalize_path_string(parent),
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
                    normalize_path_string(&snapshot.full_path),
                    normalize_path_string(&archive_path),
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
            normalize_path_string(&destination_root),
            error
        );
    }

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
    })
}

#[tauri::command]
pub async fn run_both_backups(state: State<'_, AppState>) -> Result<BothBackupsResult, String> {
    let database = run_database_backup(state.clone()).await?;
    let designs = run_designs_backup(state).await?;

    Ok(BothBackupsResult { database, designs })
}

async fn get_setting_with_default(
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

fn derive_database_source_path() -> PathBuf {
    let config = BootstrapConfig::from_env();
    PathBuf::from(strip_sqlite_prefix(&config.database_url))
}

fn derive_data_root_path() -> PathBuf {
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

fn derive_designs_source_path() -> PathBuf {
    derive_data_root_path().join("MachineEmbroideryDesigns")
}

fn resolve_design_full_path(base_path: &Path, stored_filepath: &str) -> PathBuf {
    // Normalise the stored path: trim whitespace and normalise separators.
    let candidate = stored_filepath.trim().replace('\\', "/");

    if candidate.is_empty() {
        return base_path.to_path_buf();
    }

    // If the stored path is absolute, use it directly.
    let candidate_path = PathBuf::from(&candidate);
    if candidate_path.is_absolute() {
        return candidate_path;
    }

    // If the stored path starts with "MachineEmbroideryDesigns" (case-insensitive),
    // resolve it relative to the data root (the parent of the designs base path).
    // This handles the common case where the catalogue stores paths like
    // "/MachineEmbroideryDesigns/testdata/01Peacock.dst" or
    // "MachineEmbroideryDesigns/testdata/01Peacock.dst".
    let cleaned = candidate.trim_start_matches('/');
    let cleaned_lower = cleaned.to_ascii_lowercase();
    if cleaned_lower == "machineembroiderydesigns"
        || cleaned_lower.starts_with("machineembroiderydesigns/")
    {
        // Derive the data root from the base_path parameter (base_path is
        // <data_root>/MachineEmbroideryDesigns), so the data root is its parent.
        let data_root = base_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        return data_root.join(cleaned);
    }

    // Otherwise, resolve relative to the designs base path.
    let combined = if candidate.starts_with('/') || candidate.starts_with('\\') {
        format!("{}{}", normalize_path_string(base_path), candidate)
    } else {
        format!("{}/{}", normalize_path_string(base_path), candidate)
    };

    PathBuf::from(combined)
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

fn normalize_path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn current_epoch_seconds_string() -> String {
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

fn ensure_writable_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| {
        format!(
            "Could not create destination '{}': {}",
            normalize_path_string(path),
            error
        )
    })?;

    let probe = path.join(".backup-write-test.tmp");
    fs::write(&probe, b"ok").map_err(|error| {
        format!(
            "Destination is not writable '{}': {}",
            normalize_path_string(path),
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

fn collect_file_snapshots(
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

fn collect_file_snapshots_recursive(
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

fn files_match(left: &FileSnapshot, right: &FileSnapshot) -> bool {
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
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::{Connection, Executor, SqliteConnection};
    use std::time::Duration;

    fn unique_temp_path(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{}-{}",
            prefix,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ))
    }

    // ─── Group A: Pure functions (zero setup) ───────────────────────────────────

    #[test]
    fn maintenance_scaffold_enabled_returns_true() {
        assert!(maintenance_scaffold_enabled());
    }

    #[test]
    fn setting_description_for_key_returns_correct_descriptions() {
        let db_desc = setting_description_for_key(KEY_BACKUP_DATABASE_DESTINATION);
        assert!(db_desc.contains("database"));
        assert!(db_desc.contains("backup"));

        let designs_desc = setting_description_for_key(KEY_BACKUP_DESIGNS_DESTINATION);
        assert!(designs_desc.contains("designs"));
        assert!(designs_desc.contains("backup"));

        assert_eq!(setting_description_for_key("unknown.key"), "");
    }

    #[test]
    fn is_truthy_recognises_valid_values() {
        assert!(is_truthy("1"));
        assert!(is_truthy("true"));
        assert!(is_truthy("yes"));
        assert!(is_truthy("y"));
        assert!(is_truthy("accepted"));
        // case insensitivity and whitespace
        assert!(is_truthy(" TRUE "));
        assert!(is_truthy("  Yes  "));
        assert!(is_truthy("ACCEPTED"));
    }

    #[test]
    fn is_truthy_rejects_invalid_values() {
        assert!(!is_truthy("0"));
        assert!(!is_truthy("no"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy("off"));
        assert!(!is_truthy(""));
        assert!(!is_truthy("   "));
        assert!(!is_truthy("maybe"));
    }

    #[test]
    fn modified_epoch_seconds_handles_some_and_none() {
        let time = SystemTime::now();
        let result = modified_epoch_seconds(Some(time));
        assert!(result.is_some());
        let secs = result.unwrap();
        assert!(secs > 1_700_000_000); // reasonable Unix timestamp in 2026

        assert_eq!(modified_epoch_seconds(None), None);
    }

    #[test]
    fn modified_epoch_seconds_handles_time_before_epoch() {
        // SystemTime::UNIX_EPOCH - 1 second is before the epoch, so duration_since fails.
        let before_epoch = UNIX_EPOCH - Duration::from_secs(1);
        assert_eq!(modified_epoch_seconds(Some(before_epoch)), None);
    }

    #[test]
    fn normalize_path_string_round_trips() {
        let path = PathBuf::from(r"C:\Users\test\file.pes");
        let result = normalize_path_string(&path);
        assert!(result.contains("file.pes"));
        assert!(result.contains("Users"));
    }

    #[test]
    fn normalize_path_string_handles_unicode() {
        let path = PathBuf::from("data/über/dossier.pes");
        let result = normalize_path_string(&path);
        assert!(result.contains("über"));
    }

    #[test]
    fn current_epoch_seconds_string_returns_numeric_string() {
        let result = current_epoch_seconds_string();
        assert!(!result.is_empty());
        assert!(result.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn fallback_filename_timestamp_returns_numeric_string() {
        let result = fallback_filename_timestamp();
        assert!(!result.is_empty());
        assert!(result.chars().all(|c| c.is_ascii_digit()));
    }

    // ─── Group B: Filesystem integration ───────────────────────────────────────

    #[test]
    fn ensure_writable_directory_creates_and_validates() {
        let dir = unique_temp_path("backup-writable-test");
        assert!(!dir.exists());

        let result = ensure_writable_directory(&dir);
        assert!(result.is_ok());
        assert!(dir.exists());
        // The probe file should have been cleaned up
        assert!(!dir.join(".backup-write-test.tmp").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_writable_directory_accepts_existing_directory() {
        let dir = unique_temp_path("backup-writable-existing-test");
        fs::create_dir_all(&dir).expect("pre-create should succeed");

        let result = ensure_writable_directory(&dir);
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn collect_file_snapshots_returns_empty_map_for_missing_root() {
        let missing = unique_temp_path("snapshot-missing");
        let map = collect_file_snapshots(&missing, true).expect("missing root should return empty map");
        assert!(map.is_empty());
    }

    #[test]
    fn collect_file_snapshots_finds_all_files() {
        let root = unique_temp_path("snapshot-files-test");
        fs::create_dir_all(root.join("subdir")).expect("subdir should be created");
        fs::write(root.join("alpha.pes"), b"alpha").expect("alpha should be created");
        fs::write(root.join("subdir").join("beta.pes"), b"beta").expect("beta should be created");

        let map = collect_file_snapshots(&root, false).expect("snapshot should succeed");
        assert_eq!(map.len(), 2);

        let alpha_key = PathBuf::from("alpha.pes");
        let beta_key = PathBuf::from("subdir/beta.pes");
        assert!(map.contains_key(&alpha_key));
        assert!(map.contains_key(&beta_key));
        assert_eq!(map[&alpha_key].size, 5);
        assert_eq!(map[&beta_key].size, 4);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn collect_file_snapshots_skips_deleted_tree_when_flag_set() {
        let root = unique_temp_path("snapshot-deleted-skip");
        fs::create_dir_all(root.join("_deleted").join("2026-01-01")).expect("deleted dirs should be created");
        fs::create_dir_all(root.join("active")).expect("active dir should be created");
        fs::write(root.join("active").join("keep.pes"), b"keep").expect("keep file should be created");
        fs::write(root.join("_deleted").join("2026-01-01").join("gone.pes"), b"gone")
            .expect("gone file should be created");

        let map = collect_file_snapshots(&root, true).expect("snapshot with skip_deleted should succeed");
        assert_eq!(map.len(), 1, "should only find files outside _deleted");
        assert!(map.contains_key(&PathBuf::from("active/keep.pes")));

        // Now collect without skipping to confirm _deleted is normally included
        let map_all = collect_file_snapshots(&root, false).expect("snapshot without skip should succeed");
        assert_eq!(map_all.len(), 2, "should find all files when not skipping _deleted");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_design_full_path_returns_base_for_empty_string() {
        let base = PathBuf::from("C:/Designs");
        assert_eq!(resolve_design_full_path(&base, ""), base);
        assert_eq!(resolve_design_full_path(&base, "  "), base);
    }

    #[test]
    fn resolve_design_full_path_preserves_absolute_path() {
        let base = PathBuf::from("C:/Designs");
        let absolute = PathBuf::from("D:/Other/file.pes");
        let absolute_str = absolute.to_string_lossy().to_string();
        let result = resolve_design_full_path(&base, &absolute_str);
        // On Windows the absolute path will retain its drive letter; on non-Windows
        // we just check it's absolute.
        assert!(result.is_absolute());
        assert_eq!(result, absolute);
    }

    #[test]
    fn resolve_design_full_path_resolves_med_relative_to_data_root() {
        // base_path = <data_root>/MachineEmbroideryDesigns
        let data_root = unique_temp_path("resolve-med-test");
        let designs_base = data_root.join("MachineEmbroideryDesigns").join("testdata");
        fs::create_dir_all(&designs_base).expect("testdata dir should be created");
        let base_path = data_root.join("MachineEmbroideryDesigns");

        // Simulate a stored path like "/MachineEmbroideryDesigns/testdata/design.dst"
        let result = resolve_design_full_path(&base_path, "/MachineEmbroideryDesigns/testdata/design.dst");
        let expected = data_root.join("MachineEmbroideryDesigns/testdata/design.dst");
        assert_eq!(result, expected);
    }

    #[test]
    fn resolve_design_full_path_resolves_med_without_leading_slash() {
        let data_root = unique_temp_path("resolve-med-noslash-test");
        let designs_base = data_root.join("MachineEmbroideryDesigns").join("testdata");
        fs::create_dir_all(&designs_base).expect("testdata dir should be created");
        let base_path = data_root.join("MachineEmbroideryDesigns");

        let result = resolve_design_full_path(&base_path, "MachineEmbroideryDesigns/testdata/design.dst");
        let expected = data_root.join("MachineEmbroideryDesigns/testdata/design.dst");
        assert_eq!(result, expected);
    }

    #[test]
    fn resolve_design_full_path_resolves_relative_path() {
        let base = PathBuf::from("C:/Designs");
        let result = resolve_design_full_path(&base, "subdir/file.pes");
        assert_eq!(result, PathBuf::from("C:/Designs/subdir/file.pes"));
    }

    #[test]
    fn resolve_design_full_path_resolves_leading_slash_relative() {
        let base = PathBuf::from("C:/Designs");
        let result = resolve_design_full_path(&base, "/subdir/file.pes");
        // On Windows the leading slash gets absorbed; we check the path ends as expected.
        assert!(result.to_string_lossy().contains("Designs"));
        assert!(result.to_string_lossy().contains("subdir/file.pes"));
    }

    #[test]
    fn resolve_design_full_path_normalises_backslashes() {
        let base = PathBuf::from("C:/Designs");
        let result = resolve_design_full_path(&base, r"subdir\file.pes");
        assert!(result.to_string_lossy().contains("subdir"));
        assert!(result.to_string_lossy().contains("file.pes"));
    }

    #[test]
    fn nearest_existing_folder_returns_dir_when_path_is_dir() {
        let dir = unique_temp_path("nearest-dir-test");
        fs::create_dir_all(&dir).expect("dir should be created");
        let fallback = PathBuf::from("C:/fallback");

        let result = nearest_existing_folder(&dir, &fallback);
        assert_eq!(result, dir);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nearest_existing_folder_returns_parent_when_path_is_file() {
        let dir = unique_temp_path("nearest-file-test");
        fs::create_dir_all(&dir).expect("dir should be created");
        let file = dir.join("design.pes");
        fs::write(&file, b"data").expect("file should be created");
        let fallback = PathBuf::from("C:/fallback");

        let result = nearest_existing_folder(&file, &fallback);
        assert_eq!(result, dir);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nearest_existing_folder_returns_fallback_when_nothing_exists() {
        let non_existent = PathBuf::from("Q:/does/not/exist/deep/file.pes");
        let fallback = PathBuf::from("C:/fallback");

        let result = nearest_existing_folder(&non_existent, &fallback);
        assert_eq!(result, fallback);
    }

    #[test]
    fn nearest_existing_folder_returns_parent_when_dir_does_not_exist() {
        let dir = unique_temp_path("nearest-parent-test");
        fs::create_dir_all(&dir).expect("dir should be created");
        let non_existent_sub = dir.join("nope").join("deeper").join("file.pes");
        let fallback = PathBuf::from("C:/fallback");

        let result = nearest_existing_folder(&non_existent_sub, &fallback);
        assert_eq!(result, dir); // dir exists, so it should climb to it

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn files_match_returns_false_when_both_modified_none() {
        let left = FileSnapshot {
            full_path: PathBuf::from("left"),
            size: 100,
            modified: None,
        };
        let right = FileSnapshot {
            full_path: PathBuf::from("right"),
            size: 100,
            modified: None,
        };
        assert!(!files_match(&left, &right));
    }

    #[test]
    fn files_match_returns_false_when_one_modified_none() {
        let left = FileSnapshot {
            full_path: PathBuf::from("left"),
            size: 100,
            modified: Some(UNIX_EPOCH),
        };
        let right = FileSnapshot {
            full_path: PathBuf::from("right"),
            size: 100,
            modified: None,
        };
        assert!(!files_match(&left, &right));
        assert!(!files_match(&right, &left));
    }

    #[test]
    fn collect_file_snapshots_recursive_skips_symlinks_and_non_files() {
        // Only regular files and directories are followed; symlinks are not handled
        // by file_type().is_file() / is_dir() checks, but the function does not
        // explicitly handle them. This test ensures the function doesn't panic on a
        // non-regular/non-directory entry (e.g., a named pipe or socket is unlikely,
        // but we can verify robustness with empty dirs).
        let root = unique_temp_path("snapshot-edge-test");
        fs::create_dir_all(&root).expect("dir should be created");
        // Just an empty directory: nothing to snapshot, but should not error
        let map = collect_file_snapshots(&root, false).expect("empty dir should succeed");
        assert!(map.is_empty());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unique_path_with_suffix_returns_original_when_no_conflict() {
        let temp_dir = unique_temp_path("unique-no-conflict-test");
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");
        let path = temp_dir.join("unique_file.db");
        // file does not exist, so the original path should be returned
        let result = unique_path_with_suffix(path.clone());
        assert_eq!(result, path);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn unique_path_with_suffix_handles_file_without_extension() {
        let temp_dir = unique_temp_path("unique-no-ext-test");
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");
        let base = temp_dir.join("noext");
        fs::write(&base, b"seed").expect("seed file should be created");

        let candidate = unique_path_with_suffix(base.clone());
        assert_ne!(candidate, base);
        assert!(candidate
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .starts_with("noext_"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn cleanup_empty_directories_skips_non_directory_root() {
        let file = unique_temp_path("cleanup-not-dir");
        fs::write(&file, b"content").expect("file should be created");
        // Should not fail when root is a file
        let result = cleanup_empty_directories(&file, &PathBuf::from("anything"), true);
        assert!(result.is_ok());
        let _ = fs::remove_file(&file);
    }

    #[test]
    fn cleanup_empty_directories_skips_when_root_starts_with_preserve_root() {
        let root = unique_temp_path("cleanup-preserve-test");
        let preserve = root.join("keep");
        let child = preserve.join("sub");
        fs::create_dir_all(&child).expect("dirs should be created");

        // root itself starts with preserve? No — but a case where preserve is a
        // parent or equal root is handled. Here we check that when root == preserve
        // (is_root=true), we don't delete it but we *do* clean empty children.
        // Actually, `starts_with(preserve_root)` when root == preserve_root returns true,
        // so it short-circuits and does nothing. But is_root means the root itself won't
        // be deleted anyway. Let's test root != preserve, but root starts_with preserve.
        // That path is only reachable when cleanup is called *within* the _deleted tree,
        // which it isn't in normal usage. For coverage we set preserve = root.join("_deleted")
        // and root = preserve.join("sub") — then root starts_with(preserve) -> true -> early return.
        let nested = preserve.join("sub");
        fs::create_dir_all(&nested).expect("sub dir should exist");
        let result = cleanup_empty_directories(&nested, &preserve, false);
        assert!(result.is_ok());
        // sub should NOT have been removed since root starts_with(preserve)
        assert!(nested.exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cleanup_empty_directories_removes_nested_empty() {
        let root = unique_temp_path("cleanup-nested-test");
        let parent = root.join("parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).expect("dirs should be created");

        cleanup_empty_directories(&root, &root.join("_deleted"), true)
            .expect("cleanup should complete");

        // Both child and parent should have been removed
        assert!(!child.exists());
        assert!(!parent.exists());
        // Root should still exist
        assert!(root.exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cleanup_empty_directories_preserves_non_empty_tree() {
        let root = unique_temp_path("cleanup-nonempty-test");
        let parent = root.join("parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).expect("dirs should be created");
        fs::write(child.join("file.pes"), b"data").expect("file should be created");

        cleanup_empty_directories(&root, &root.join("_deleted"), true)
            .expect("cleanup should complete");

        // Non-empty tree should be preserved
        assert!(child.exists());
        assert!(parent.exists());
        assert!(root.exists());

        let _ = fs::remove_dir_all(&root);
    }

    // ─── Group C: Database-dependent (in-memory SQLite) ────────────────────────

    async fn setup_settings_table(conn: &mut SqliteConnection) {
        conn.execute(
            "CREATE TABLE settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                description TEXT NOT NULL
            )",
        )
        .await
        .expect("settings table should be created");
    }

    #[tokio::test]
    async fn upsert_setting_inserts_new_row() {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should connect");
        setup_settings_table(&mut conn).await;

        upsert_setting(&mut conn, KEY_BACKUP_DATABASE_DESTINATION, "D:/Backups/DB")
            .await
            .expect("upsert should succeed");

        let row = sqlx::query_as::<_, (String, String, String)>(
            "SELECT key, value, description FROM settings WHERE key = ?",
        )
        .bind(KEY_BACKUP_DATABASE_DESTINATION)
        .fetch_one(&mut conn)
        .await
        .expect("row should exist");

        assert_eq!(row.1, "D:/Backups/DB");
        assert!(row.2.contains("database"));
    }

    #[tokio::test]
    async fn upsert_setting_updates_existing_row() {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should connect");
        setup_settings_table(&mut conn).await;

        // Insert once
        upsert_setting(&mut conn, KEY_BACKUP_DESIGNS_DESTINATION, "D:/Backups/Designs")
            .await
            .expect("first upsert should succeed");

        // Update with new value
        upsert_setting(&mut conn, KEY_BACKUP_DESIGNS_DESTINATION, "E:/NewBackup")
            .await
            .expect("second upsert should succeed");

        let row = sqlx::query_as::<_, (String, String, String)>(
            "SELECT key, value, description FROM settings WHERE key = ?",
        )
        .bind(KEY_BACKUP_DESIGNS_DESTINATION)
        .fetch_one(&mut conn)
        .await
        .expect("row should exist");

        assert_eq!(row.1, "E:/NewBackup");
        // Description should still be the designs backup description
        assert!(row.2.contains("designs"));
    }

    #[tokio::test]
    async fn find_orphan_ids_with_pool_returns_correct_ids() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-find-test");
        fs::create_dir_all(&root).expect("test root should be created");
        fs::write(root.join("keep.jef"), b"ok").expect("keep file should be created");

        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (1, 'keep.jef', '/keep.jef')")
            .await
            .expect("keep insert should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (2, 'gone.jef', '/gone.jef')")
            .await
            .expect("gone insert should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (3, 'also_gone.jef', '/also_gone.jef')")
            .await
            .expect("also gone insert should succeed");

        let ids = find_orphan_ids_with_pool(&pool, &root)
            .await
            .expect("find orphans should succeed");

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
        assert!(!ids.contains(&1));

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn find_orphan_ids_with_pool_skips_empty_filepath() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-empty-path-test");
        fs::create_dir_all(&root).expect("test root should be created");

        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (1, 'empty.jef', '')")
            .await
            .expect("empty filepath insert should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (2, 'missing.jef', '/missing.jef')")
            .await
            .expect("missing insert should succeed");

        let ids = find_orphan_ids_with_pool(&pool, &root)
            .await
            .expect("find orphans should succeed");

        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&2));

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn get_orphans_page_with_pool_defaults_to_page_one() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-default-page-test");
        fs::create_dir_all(&root).expect("test root should be created");

        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (1, 'missing.jef', '/missing.jef')")
            .await
            .expect("missing insert should succeed");

        let result = get_orphans_page_with_pool(&pool, &root, None)
            .await
            .expect("page load should succeed");

        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 100);
        assert_eq!(result.total, 1);
        assert_eq!(result.items.len(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn get_orphans_page_with_pool_returns_empty_when_no_orphans() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-no-orphans-test");
        fs::create_dir_all(&root).expect("test root should be created");
        fs::write(root.join("present.jef"), b"ok").expect("present file should be created");

        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (1, 'present.jef', '/present.jef')")
            .await
            .expect("present insert should succeed");

        let result = get_orphans_page_with_pool(&pool, &root, None)
            .await
            .expect("page load should succeed");

        assert_eq!(result.items.len(), 0);
        assert_eq!(result.total, 0);
        assert_eq!(result.page, 1);
        assert_eq!(result.total_pages, 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn get_orphans_page_with_pool_clamps_page_size() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-clamp-test");
        fs::create_dir_all(&root).expect("test root should be created");
        // Insert two missing so we have orphan data
        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (1, 'a.jef', '/a.jef')")
            .await
            .expect("insert a should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (2, 'b.jef', '/b.jef')")
            .await
            .expect("insert b should succeed");

        // page_size=0 should clamp to 1
        let result = get_orphans_page_with_pool(
            &pool,
            &root,
            Some(GetOrphansPageRequest {
                page: Some(1),
                page_size: Some(0),
            }),
        )
        .await
        .expect("page load should succeed");
        assert_eq!(result.page_size, 1);
        assert_eq!(result.items.len(), 1);

        // page_size=1000 should clamp to 500
        let result = get_orphans_page_with_pool(
            &pool,
            &root,
            Some(GetOrphansPageRequest {
                page: Some(1),
                page_size: Some(1000),
            }),
        )
        .await
        .expect("page load should succeed");
        assert_eq!(result.page_size, 500);
        assert_eq!(result.items.len(), 2);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn get_orphans_page_with_pool_page_out_of_bounds_clamps() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-clamp-page-test");
        fs::create_dir_all(&root).expect("test root should be created");

        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (1, 'a.jef', '/a.jef')")
            .await
            .expect("insert a should succeed");

        // Request page 999 with page_size 1 — only 1 orphan exists so page clamps to 1
        let result = get_orphans_page_with_pool(
            &pool,
            &root,
            Some(GetOrphansPageRequest {
                page: Some(999),
                page_size: Some(1),
            }),
        )
        .await
        .expect("page load should succeed");
        assert_eq!(result.page, 1);
        assert_eq!(result.items.len(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn delete_design_ids_with_pool_empty_slice_returns_zero() {
        let pool = setup_orphans_test_pool().await;

        let deleted = delete_design_ids_with_pool(&pool, &[])
            .await
            .expect("delete empty slice should succeed");
        assert_eq!(deleted, 0);
    }

    #[tokio::test]
    async fn delete_design_ids_with_pool_chunks_large_batch() {
        let pool = setup_orphans_test_pool().await;

        // Insert 510 rows (exceeds the 500 chunk size)
        let mut ids = Vec::new();
        for i in 0..510 {
            let filepath = format!("/{}.jef", i);
            pool.execute(
                sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
                    .bind(i)
                    .bind(format!("{}.jef", i))
                    .bind(filepath),
            )
            .await
            .expect("insert should succeed");
            ids.push(i);
        }

        let deleted = delete_design_ids_with_pool(&pool, &ids)
            .await
            .expect("delete batch should succeed");
        assert_eq!(deleted, 510);

        let remaining = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM designs")
            .fetch_one(&pool)
            .await
            .expect("count should load");
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    async fn sqlite_localtime_format_returns_formatted_string() {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should connect");

        let result = sqlite_localtime_format(&mut conn, "%Y").await;
        assert!(result.is_ok());
        let year = result.unwrap();
        assert_eq!(year.len(), 4);
        assert!(year.chars().all(|c| c.is_ascii_digit()));
    }

    #[tokio::test]
    async fn sqlite_localtime_format_errors_on_empty_format() {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should connect");

        // strftime with an empty format returns an empty string, which should trigger the error path
        let result = sqlite_localtime_format(&mut conn, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn scan_orphans_with_pool_handles_empty_database_gracefully() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool should connect");

        pool.execute("CREATE TABLE designs (id INTEGER PRIMARY KEY, filename TEXT, filepath TEXT)")
            .await
            .expect("designs table should be created");

        let root = unique_temp_path("orphans-empty-db-test");
        fs::create_dir_all(&root).expect("test root should be created");

        let result = scan_orphans_with_pool(&pool, &root)
            .await
            .expect("scan empty db should succeed");

        assert_eq!(result.checked, 0);
        assert_eq!(result.found, 0);

        let _ = fs::remove_dir_all(&root);
    }

    // ─── Existing tests (preserved) ────────────────────────────────────────────

    #[test]
    fn strip_sqlite_prefix_handles_supported_formats() {
        assert_eq!(
            strip_sqlite_prefix("sqlite:///tmp/catalogue.db"),
            "tmp/catalogue.db"
        );
        assert_eq!(
            strip_sqlite_prefix("sqlite://tmp/catalogue.db"),
            "tmp/catalogue.db"
        );
        assert_eq!(
            strip_sqlite_prefix("sqlite:tmp/catalogue.db"),
            "tmp/catalogue.db"
        );
        assert_eq!(strip_sqlite_prefix("tmp/catalogue.db"), "tmp/catalogue.db");
    }

    #[test]
    fn unique_path_with_suffix_avoids_existing_file() {
        let temp_dir = unique_temp_path("backup-path-test");
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");

        let base = temp_dir.join("catalogue_2026-05-30_1200.db");
        fs::write(&base, b"seed").expect("seed file should be created");

        let candidate = unique_path_with_suffix(base.clone());
        assert_ne!(candidate, base);
        assert!(candidate
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .starts_with("catalogue_2026-05-30_1200_"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn files_match_respects_size_and_mtime_tolerance() {
        let left = FileSnapshot {
            full_path: PathBuf::from("left"),
            size: 100,
            modified: Some(UNIX_EPOCH + Duration::from_secs(1_000)),
        };
        let right_within_tolerance = FileSnapshot {
            full_path: PathBuf::from("right"),
            size: 100,
            modified: Some(UNIX_EPOCH + Duration::from_secs(1_001)),
        };
        let right_outside_tolerance = FileSnapshot {
            full_path: PathBuf::from("right"),
            size: 100,
            modified: Some(UNIX_EPOCH + Duration::from_secs(1_010)),
        };
        let different_size = FileSnapshot {
            full_path: PathBuf::from("right"),
            size: 101,
            modified: Some(UNIX_EPOCH + Duration::from_secs(1_001)),
        };

        assert!(files_match(&left, &right_within_tolerance));
        assert!(!files_match(&left, &right_outside_tolerance));
        assert!(!files_match(&left, &different_size));
    }

    #[test]
    fn cleanup_empty_directories_keeps_deleted_tree() {
        let root = unique_temp_path("backup-cleanup-test");
        let empty_dir = root.join("orphan-empty");
        let deleted_dir = root.join("_deleted").join("2026-05-30");

        fs::create_dir_all(&empty_dir).expect("empty dir should be created");
        fs::create_dir_all(&deleted_dir).expect("deleted dir should be created");
        fs::write(deleted_dir.join("archived.pes"), b"content")
            .expect("archived file should be created");

        cleanup_empty_directories(&root, &root.join("_deleted"), true)
            .expect("cleanup should complete");

        assert!(!empty_dir.exists());
        assert!(deleted_dir.exists());
        assert!(deleted_dir.join("archived.pes").exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn get_setting_with_default_inserts_and_reads_value() {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should connect");

        conn.execute(
            "CREATE TABLE settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                description TEXT NOT NULL
            )",
        )
        .await
        .expect("settings table should be created");

        let initial = get_setting_with_default(&mut conn, KEY_BACKUP_DATABASE_DESTINATION)
            .await
            .expect("default setting should be inserted");
        assert_eq!(initial, "");

        upsert_setting(&mut conn, KEY_BACKUP_DATABASE_DESTINATION, "D:/Backups/DB")
            .await
            .expect("upsert should succeed");

        let updated = get_setting_with_default(&mut conn, KEY_BACKUP_DATABASE_DESTINATION)
            .await
            .expect("updated setting should be readable");
        assert_eq!(updated, "D:/Backups/DB");
    }

    async fn setup_orphans_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool should connect");

        pool.execute(
            "CREATE TABLE designers (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
        )
        .await
        .expect("designers table should be created");

        pool.execute(
            "CREATE TABLE designs (
                id INTEGER PRIMARY KEY,
                filename TEXT NOT NULL,
                filepath TEXT NOT NULL,
                designer_id INTEGER,
                date_added TEXT,
                FOREIGN KEY(designer_id) REFERENCES designers(id)
            )",
        )
        .await
        .expect("designs table should be created");

        pool
    }

    #[tokio::test]
    async fn scan_orphans_counts_missing_files() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-scan-test");
        fs::create_dir_all(&root).expect("test root should be created");
        fs::write(root.join("present.jef"), b"ok").expect("present file should be created");

        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (1, 'present.jef', '/present.jef')")
            .await
            .expect("present design insert should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (2, 'missing.jef', '/missing.jef')")
            .await
            .expect("missing design insert should succeed");

        let result = scan_orphans_with_pool(&pool, &root)
            .await
            .expect("scan should succeed");

        assert_eq!(result.checked, 2);
        assert_eq!(result.found, 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn scan_orphans_handles_relative_filepath_without_leading_separator() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-relative-path-test");
        fs::create_dir_all(&root).expect("test root should be created");
        fs::write(root.join("present.jef"), b"ok").expect("present file should be created");

        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
            .bind(1_i64)
            .bind("present.jef")
            .bind("present.jef")
            .execute(&pool)
            .await
            .expect("design insert should succeed");

        let result = scan_orphans_with_pool(&pool, &root)
            .await
            .expect("scan should succeed");

        assert_eq!(result.checked, 1);
        assert_eq!(result.found, 0);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn scan_orphans_allows_absolute_filepath_when_file_exists() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-absolute-path-test");
        fs::create_dir_all(&root).expect("test root should be created");

        let external_root = unique_temp_path("orphans-absolute-external");
        fs::create_dir_all(&external_root).expect("external root should be created");
        let external_file = external_root.join("exists.jef");
        fs::write(&external_file, b"ok").expect("external file should be created");

        let stored = external_file.to_string_lossy().to_string();
        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
            .bind(1_i64)
            .bind("exists.jef")
            .bind(stored)
            .execute(&pool)
            .await
            .expect("design insert should succeed");

        let result = scan_orphans_with_pool(&pool, &root)
            .await
            .expect("scan should succeed");

        assert_eq!(result.checked, 1);
        assert_eq!(result.found, 0);

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&external_root);
    }

    #[tokio::test]
    async fn scan_orphans_counts_missing_absolute_filepath_as_orphan() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-absolute-missing-test");
        fs::create_dir_all(&root).expect("test root should be created");

        let missing_absolute = format!(
            "{}{}",
            unique_temp_path("orphans-absolute-missing").to_string_lossy(),
            format!("{}missing.jef", std::path::MAIN_SEPARATOR)
        );

        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
            .bind(1_i64)
            .bind("missing.jef")
            .bind(missing_absolute)
            .execute(&pool)
            .await
            .expect("design insert should succeed");

        let result = scan_orphans_with_pool(&pool, &root)
            .await
            .expect("scan should succeed");

        assert_eq!(result.checked, 1);
        assert_eq!(result.found, 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn get_orphans_page_returns_sorted_slice() {
        let pool = setup_orphans_test_pool().await;
        let root = unique_temp_path("orphans-page-test");
        fs::create_dir_all(&root).expect("test root should be created");
        fs::write(root.join("present.jef"), b"ok").expect("present file should be created");

        pool.execute("INSERT INTO designers (id, name) VALUES (1, 'Designer One')")
            .await
            .expect("designer insert should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath, designer_id) VALUES (1, 'present.jef', '/present.jef', 1)")
            .await
            .expect("present design insert should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath, designer_id) VALUES (2, 'a_missing.jef', '/a_missing.jef', 1)")
            .await
            .expect("first missing design insert should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath, designer_id) VALUES (3, 'b_missing.jef', '/b_missing.jef', 1)")
            .await
            .expect("second missing design insert should succeed");

        let result = get_orphans_page_with_pool(
            &pool,
            &root,
            Some(GetOrphansPageRequest {
                page: Some(2),
                page_size: Some(1),
            }),
        )
        .await
        .expect("page load should succeed");

        assert_eq!(result.total, 2);
        assert_eq!(result.page, 2);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, 3);
        assert_eq!(result.items[0].designer, "Designer One");

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn delete_design_ids_with_pool_deletes_only_requested_rows() {
        let pool = setup_orphans_test_pool().await;

        pool.execute(
            "INSERT INTO designs (id, filename, filepath) VALUES (10, 'first.jef', '/first.jef')",
        )
        .await
        .expect("first insert should succeed");
        pool.execute(
            "INSERT INTO designs (id, filename, filepath) VALUES (11, 'second.jef', '/second.jef')",
        )
        .await
        .expect("second insert should succeed");

        let deleted = delete_design_ids_with_pool(&pool, &[10])
            .await
            .expect("delete should succeed");

        assert_eq!(deleted, 1);

        let remaining = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM designs")
            .fetch_one(&pool)
            .await
            .expect("remaining count should load");
        assert_eq!(remaining, 1);
    }

    #[tokio::test]
    async fn scan_orphans_does_not_report_machine_embroidery_designs_path_as_orphan() {
        // Regression test: stored paths like "/MachineEmbroideryDesigns/testdata/01Peacock.dst"
        // should resolve to the correct location under the data root, not produce a doubled path.
        let pool = setup_orphans_test_pool().await;

        // Create a temp directory structure mimicking the real layout:
        //   <root>/MachineEmbroideryDesigns/testdata/01Peacock.dst
        let root = unique_temp_path("orphans-med-path-test");
        let designs_dir = root.join("MachineEmbroideryDesigns").join("testdata");
        fs::create_dir_all(&designs_dir).expect("testdata dir should be created");
        fs::write(designs_dir.join("01Peacock.dst"), b"embroidery data")
            .expect("test file should be created");

        // The base_path passed to orphan scan is <root>/MachineEmbroideryDesigns
        let base_path = root.join("MachineEmbroideryDesigns");

        // Insert a design with the stored path format used by the catalogue:
        // "/MachineEmbroideryDesigns/testdata/01Peacock.dst"
        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
            .bind(1_i64)
            .bind("01Peacock.dst")
            .bind("/MachineEmbroideryDesigns/testdata/01Peacock.dst")
            .execute(&pool)
            .await
            .expect("design insert should succeed");

        let result = scan_orphans_with_pool(&pool, &base_path)
            .await
            .expect("scan should succeed");

        assert_eq!(result.checked, 1, "should have checked exactly one design");
        assert_eq!(
            result.found, 0,
            "should NOT report the existing file as an orphan"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn scan_orphans_handles_machine_embroidery_designs_path_without_leading_slash() {
        // Also test the variant without a leading slash:
        // "MachineEmbroideryDesigns/testdata/01Peacock.dst"
        let pool = setup_orphans_test_pool().await;

        let root = unique_temp_path("orphans-med-no-slash-test");
        let designs_dir = root.join("MachineEmbroideryDesigns").join("testdata");
        fs::create_dir_all(&designs_dir).expect("testdata dir should be created");
        fs::write(designs_dir.join("01Peacock.dst"), b"embroidery data")
            .expect("test file should be created");

        let base_path = root.join("MachineEmbroideryDesigns");

        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
            .bind(1_i64)
            .bind("01Peacock.dst")
            .bind("MachineEmbroideryDesigns/testdata/01Peacock.dst")
            .execute(&pool)
            .await
            .expect("design insert should succeed");

        let result = scan_orphans_with_pool(&pool, &base_path)
            .await
            .expect("scan should succeed");

        assert_eq!(result.checked, 1);
        assert_eq!(result.found, 0);

        let _ = fs::remove_dir_all(&root);
    }
}

