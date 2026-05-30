use crate::config::BootstrapConfig;
use crate::services::folder_picker;
use crate::settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqliteConnection, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

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

    Ok(DeleteOrphansResult { deleted })
}

#[tauri::command]
pub async fn delete_all_orphans(state: State<'_, AppState>) -> Result<DeleteOrphansResult, String> {
    let pool = &state.db;
    let base_path = derive_designs_source_path();

    let orphan_ids = find_orphan_ids_with_pool(pool, &base_path).await?;
    let deleted = delete_design_ids_with_pool(pool, &orphan_ids).await?;

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
    upsert_setting(&mut conn, KEY_BACKUP_DESIGNS_DESTINATION, &designs_destination)
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
    match folder_picker::browse_folder(start_dir.as_deref(), false) {
        Ok(result) => BrowseBackupFolderResult {
            path: result.path,
            error: None,
        },
        Err(error) => BrowseBackupFolderResult {
            path: None,
            error: Some(error),
        },
    }
}

#[tauri::command]
pub async fn run_database_backup(state: State<'_, AppState>) -> Result<DatabaseBackupResult, String> {
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
            error: Some("No database backup destination configured. Save a destination first.".to_string()),
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
    let destination_path = unique_path_with_suffix(destination_dir.join(format!("catalogue_{}.db", timestamp)));

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
                error: Some(format!(
                    "Could not create database backup: {}",
                    copy_error
                )),
            });
        }
    }

    let size_bytes = fs::metadata(&destination_path).map(|metadata| metadata.len()).unwrap_or(0);

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
            error: Some("No designs backup destination configured. Save a destination first.".to_string()),
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
                eprintln!(
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
                eprintln!(
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
                eprintln!(
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
                eprintln!(
                    "[backup] Could not archive '{}' to '{}': {}",
                    normalize_path_string(&snapshot.full_path),
                    normalize_path_string(&archive_path),
                    error
                );
            }
        }
    }

    if let Err(error) = cleanup_empty_directories(&destination_root, &destination_root.join("_deleted"), true) {
        eprintln!(
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

async fn get_setting_with_default(conn: &mut SqliteConnection, key: &str) -> Result<String, sqlx::Error> {
    if let Some(setting) = settings::get_setting(conn, key).await? {
        return Ok(setting.value);
    }

    upsert_setting(conn, key, "").await?;
    Ok("".to_string())
}

async fn upsert_setting(conn: &mut SqliteConnection, key: &str, value: &str) -> Result<(), sqlx::Error> {
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
    let cleaned = stored_filepath
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();

    if cleaned.is_empty() {
        base_path.to_path_buf()
    } else {
        base_path.join(cleaned)
    }
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

    std::env::var("PYTEST_CURRENT_TEST").is_ok()
}

async fn scan_orphans_with_pool(pool: &SqlitePool, base_path: &Path) -> Result<OrphanScanResult, String> {
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

async fn find_orphan_ids_with_pool(pool: &SqlitePool, base_path: &Path) -> Result<Vec<i64>, String> {
    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, filepath FROM designs ORDER BY filepath",
    )
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
    let offset = usize::try_from((normalized_page - 1) * page_size).map_err(|error| error.to_string())?;
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

async fn delete_design_ids_with_pool(pool: &SqlitePool, design_ids: &[i64]) -> Result<usize, String> {
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
    fs::create_dir_all(path)
        .map_err(|error| format!("Could not create destination '{}': {}", normalize_path_string(path), error))?;

    let probe = path.join(".backup-write-test.tmp");
    fs::write(&probe, b"ok")
        .map_err(|error| format!("Destination is not writable '{}': {}", normalize_path_string(path), error))?;
    let _ = fs::remove_file(&probe);

    Ok(())
}

async fn sqlite_localtime_format(conn: &mut SqliteConnection, format: &str) -> Result<String, String> {
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

fn collect_file_snapshots(root: &Path, skip_deleted_tree: bool) -> Result<HashMap<PathBuf, FileSnapshot>, String> {
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

        let relative = path.strip_prefix(root).map_err(|error| error.to_string())?.to_path_buf();
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

    match (modified_epoch_seconds(left.modified), modified_epoch_seconds(right.modified)) {
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

fn cleanup_empty_directories(root: &Path, preserve_root: &Path, is_root: bool) -> Result<(), String> {
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
    use sqlx::{Connection, Executor, SqliteConnection};
    use sqlx::sqlite::SqlitePoolOptions;
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

    #[test]
    fn strip_sqlite_prefix_handles_supported_formats() {
        assert_eq!(strip_sqlite_prefix("sqlite:///tmp/catalogue.db"), "tmp/catalogue.db");
        assert_eq!(strip_sqlite_prefix("sqlite://tmp/catalogue.db"), "tmp/catalogue.db");
        assert_eq!(strip_sqlite_prefix("sqlite:tmp/catalogue.db"), "tmp/catalogue.db");
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
        fs::write(deleted_dir.join("archived.pes"), b"content").expect("archived file should be created");

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

        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (10, 'first.jef', '/first.jef')")
            .await
            .expect("first insert should succeed");
        pool.execute("INSERT INTO designs (id, filename, filepath) VALUES (11, 'second.jef', '/second.jef')")
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
}