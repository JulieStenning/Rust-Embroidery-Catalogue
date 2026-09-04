use crate::config::BootstrapConfig;
use crate::error::AppError;
use crate::services::{
    auto_tagging, folder_picker,
    gemini_client::{self, GeminiClient},
    image_generation, scanning, stitch_identifier, tagging, validation,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::Emitter;

const BULK_IMPORT_CONTEXT_TTL: Duration = Duration::from_secs(15 * 60);
const BULK_IMPORT_CONTEXT_MAX_ENTRIES: usize = 128;

static BULK_IMPORT_CONTEXT_STORE: OnceLock<Mutex<HashMap<String, StoredBulkImportContext>>> =
    OnceLock::new();
static BULK_IMPORT_CONTEXT_COUNTER: AtomicU64 = AtomicU64::new(1);
static BULK_IMPORT_DB_POOL: Mutex<Option<SqlitePool>> = Mutex::new(None);
static BULK_IMPORT_APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
static BULK_IMPORT_CONTEXT_RESET_COUNTER: AtomicU64 = AtomicU64::new(0);
static BULK_IMPORT_CONTEXT_LAST_RESET_AT_MILLIS: AtomicU64 = AtomicU64::new(0);
static BULK_IMPORT_STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

const KEY_IMPORT_COMMIT_BATCH_SIZE: &str = "import.commit_batch_size";
const DEFAULT_IMPORT_COMMIT_BATCH_SIZE: usize = 10;
const MAX_IMPORT_COMMIT_BATCH_SIZE: usize = 10_000;
const BULK_IMPORT_PROGRESS_EVENT: &str = "bulk-import-progress";

#[derive(Debug, Clone, Serialize)]
struct BulkImportProgressEvent {
    context_token: Option<String>,
    stage: String,
    processed_count: usize,
    total_count: usize,
    persisted_count: usize,
    committed_count: usize,
    current_file: Option<String>,
    commit_batch_size: usize,
}

#[derive(Debug, Clone)]
struct StoredBulkImportContext {
    confirm_wire: BulkImportConfirmWire,
    created_at_millis: u128,
    sequence: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportRequest {
    #[serde(default)]
    pub root_path: Option<String>,
    #[serde(default)]
    pub root_paths: Vec<String>,
    pub fallback_designer_id: Option<i64>,
    pub fallback_source_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderAssignmentWire {
    pub folder_path: String,
    pub designer_id: Option<i64>,
    pub source_id: Option<i64>,
    pub inferred_designer_id: Option<i64>,
    pub inferred_source_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentFieldSourceWire {
    ExplicitPerFolder,
    Global,
    Inferred,
    Blank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedAssignmentFieldWire {
    pub value: Option<i64>,
    pub source: AssignmentFieldSourceWire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFolderAssignmentWire {
    pub folder_path: String,
    pub designer_id: ResolvedAssignmentFieldWire,
    pub source_id: ResolvedAssignmentFieldWire,
    pub inferred_designer_id: Option<i64>,
    pub inferred_source_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportWire {
    pub root_paths: Vec<String>,
    pub global_designer_id: Option<i64>,
    pub global_source_id: Option<i64>,
    pub per_folder_assignments: Vec<FolderAssignmentWire>,
    pub selected_files: Vec<String>,
    pub create_on_import: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportConfirmWire {
    pub wire: BulkImportWire,
    pub context_token: Option<String>,
    pub canonical_confirm: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportPreview {
    pub discovered_count: usize,
    pub selected_count: usize,
    pub folder_count: usize,
    pub scanned_files: Vec<scanning::ScannedFile>,
    pub resolved_assignments: Vec<ResolvedFolderAssignmentWire>,
    /// True if any selected root path did not exist on disk or was not a directory.
    pub missing_root: bool,
    /// True if any selected root string was empty or relative (shape-invalid).
    pub invalid_root: bool,
    /// True if all selected roots existed but no supported embroidery files were found.
    pub no_supported_files: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportBrowseFolderRequest {
    pub start_dir: Option<String>,
    #[serde(default)]
    pub allow_multi: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportWireSummary {
    pub root_path_count: usize,
    pub folder_assignment_count: usize,
    pub selected_file_count: usize,
    pub create_on_import: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportConfirmSummary {
    pub context_token_present: bool,
    pub root_path_count: usize,
    pub selected_file_count: usize,
    pub per_folder_assignment_count: usize,
    pub canonical_confirm: bool,
    pub resolved_assignment_count: usize,
    pub resolved_assignments: Vec<ResolvedFolderAssignmentWire>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportAssignmentResolutionSummary {
    pub resolved_count: usize,
    pub explicit_field_count: usize,
    pub global_field_count: usize,
    pub inferred_field_count: usize,
    pub blank_field_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportConfirmExecutionResult {
    pub context_token_present: bool,
    pub canonical_confirm: bool,
    pub ready_for_persistence: bool,
    pub persisted_design_count: usize,
    pub root_path_count: usize,
    pub selected_file_count: usize,
    pub resolved_assignments: Vec<ResolvedFolderAssignmentWire>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportPrecheckResult {
    pub context_token: String,
    pub context_token_present: bool,
    pub ready_for_confirm: bool,
    pub is_first_import: bool,
    pub needs_hoop_setup: bool,
    pub root_path_count: usize,
    pub selected_file_count: usize,
    pub resolved_assignments: Vec<ResolvedFolderAssignmentWire>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkImportPrecheckActionWire {
    ReviewHoops,
    ReviewTags,
    ReviewSources,
    ReviewDesigners,
    ImportNow,
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportPrecheckActionRequest {
    pub context_token: String,
    pub action: BulkImportPrecheckActionWire,
    #[serde(default)]
    pub confirm_skip_hoops: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportPrecheckActionResult {
    pub action: BulkImportPrecheckActionWire,
    pub context_token_present: bool,
    pub consumed_context: bool,
    pub requires_skip_hoops_confirmation: bool,
    pub next_route: Option<String>,
    pub confirm_result: Option<BulkImportConfirmExecutionResult>,
}

impl From<BulkImportRequest> for BulkImportWire {
    fn from(request: BulkImportRequest) -> Self {
        let mut root_paths = request
            .root_paths
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        if root_paths.is_empty() {
            if let Some(value) = request
                .root_path
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
            {
                root_paths.push(value);
            }
        }

        Self {
            root_paths,
            global_designer_id: request.fallback_designer_id,
            global_source_id: request.fallback_source_id,
            per_folder_assignments: Vec::new(),
            selected_files: Vec::new(),
            create_on_import: true,
        }
    }
}

impl From<BulkImportRequest> for BulkImportConfirmWire {
    fn from(request: BulkImportRequest) -> Self {
        Self {
            wire: request.into(),
            context_token: None,
            canonical_confirm: false,
        }
    }
}

fn bulk_import_context_store() -> &'static Mutex<HashMap<String, StoredBulkImportContext>> {
    BULK_IMPORT_CONTEXT_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn with_bulk_import_context_store<T, F>(mut f: F) -> Result<T, String>
where
    F: FnMut(&mut HashMap<String, StoredBulkImportContext>) -> T,
{
    let store = bulk_import_context_store();
    match store.lock() {
        Ok(mut guard) => Ok(f(&mut guard)),
        Err(poisoned) => {
            tracing::warn!("bulk import context store mutex poisoned; recovering");
            Ok(f(&mut poisoned.into_inner()))
        }
    }
}

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn prune_bulk_import_context_store(store: &mut HashMap<String, StoredBulkImportContext>) {
    let ttl_millis = BULK_IMPORT_CONTEXT_TTL.as_millis();
    let now = current_timestamp_millis();

    store.retain(|_, context| now.saturating_sub(context.created_at_millis) <= ttl_millis);

    if store.len() <= BULK_IMPORT_CONTEXT_MAX_ENTRIES {
        return;
    }

    let mut entries: Vec<(String, u128, u64)> = store
        .iter()
        .map(|(token, context)| (token.clone(), context.created_at_millis, context.sequence))
        .collect();
    entries.sort_by_key(|(_, created_at_millis, sequence)| (*created_at_millis, *sequence));

    let excess = store.len() - BULK_IMPORT_CONTEXT_MAX_ENTRIES;
    for (token, _, _) in entries.into_iter().take(excess) {
        store.remove(&token);
    }
}

#[cfg(test)]
fn insert_bulk_import_context_for_test(
    token: String,
    confirm_wire: BulkImportConfirmWire,
    created_at_millis: u128,
    sequence: u64,
) {
    let mut store = bulk_import_context_store().lock().unwrap();
    store.insert(
        token,
        StoredBulkImportContext {
            confirm_wire,
            created_at_millis,
            sequence,
        },
    );
}

fn next_bulk_import_context_token() -> (String, u64) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let sequence = BULK_IMPORT_CONTEXT_COUNTER.fetch_add(1, Ordering::Relaxed);
    (format!("bulk-import-{timestamp}-{sequence}"), sequence)
}

fn canonicalize_bulk_import_confirm_wire(
    mut confirm_wire: BulkImportConfirmWire,
) -> BulkImportConfirmWire {
    confirm_wire.canonical_confirm = true;
    confirm_wire
}

pub fn initialize_bulk_import_db_pool(pool: SqlitePool) {
    if let Ok(mut guard) = BULK_IMPORT_DB_POOL.lock() {
        *guard = Some(pool);
    }
}

/// Replace the cached bulk-import pool after a database restore swaps the live
/// connection pool, so subsequent imports target the restored database.
pub fn update_bulk_import_db_pool(pool: SqlitePool) {
    initialize_bulk_import_db_pool(pool);
}

pub fn initialize_bulk_import_app_handle(app_handle: tauri::AppHandle) {
    let _ = BULK_IMPORT_APP_HANDLE.set(app_handle);
}

fn get_bulk_import_db_pool() -> Option<SqlitePool> {
    BULK_IMPORT_DB_POOL
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().cloned())
}

fn get_bulk_import_app_handle() -> Option<&'static tauri::AppHandle> {
    BULK_IMPORT_APP_HANDLE.get()
}

async fn load_catalog_counts(pool: &SqlitePool) -> Result<(i64, i64), String> {
    let design_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM designs")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let hoop_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hoops")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok((design_count, hoop_count))
}

fn normalize_import_commit_batch_size(raw_value: Option<&str>) -> usize {
    let Some(value) = raw_value.map(str::trim).filter(|value| !value.is_empty()) else {
        return DEFAULT_IMPORT_COMMIT_BATCH_SIZE;
    };

    match value.parse::<usize>() {
        Ok(parsed) if parsed > 0 => parsed.min(MAX_IMPORT_COMMIT_BATCH_SIZE),
        _ => DEFAULT_IMPORT_COMMIT_BATCH_SIZE,
    }
}

async fn load_import_commit_batch_size(pool: &SqlitePool) -> Result<usize, String> {
    let raw_batch_size: Option<String> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = ? LIMIT 1")
            .bind(KEY_IMPORT_COMMIT_BATCH_SIZE)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    Ok(normalize_import_commit_batch_size(
        raw_batch_size.as_deref(),
    ))
}

async fn load_tag_catalog(pool: &SqlitePool) -> Result<Vec<(i64, String)>, String> {
    sqlx::query_as::<_, (i64, String)>("SELECT id, description FROM tags ORDER BY id ASC")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

async fn load_stitching_tag_lookup(pool: &SqlitePool) -> Result<HashMap<String, i64>, String> {
    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, description FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(id, description)| (description, id))
        .collect())
}

async fn load_default_stitching_tag_id(pool: &SqlitePool) -> Result<Option<i64>, String> {
    sqlx::query_scalar(
        "SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching' ORDER BY description ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
}

async fn read_string_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, String> {
    sqlx::query_scalar("SELECT value FROM settings WHERE key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
}

async fn read_bool_setting(pool: &SqlitePool, key: &str) -> Result<bool, String> {
    Ok(read_string_setting(pool, key)
        .await?
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "y"
            )
        })
        .unwrap_or(false))
}

async fn read_f64_setting(pool: &SqlitePool, key: &str) -> Result<Option<f64>, String> {
    Ok(read_string_setting(pool, key)
        .await?
        .and_then(|raw| raw.trim().parse::<f64>().ok()))
}

async fn read_i64_setting(pool: &SqlitePool, key: &str) -> Result<Option<i64>, String> {
    Ok(read_string_setting(pool, key)
        .await?
        .and_then(|raw| raw.trim().parse::<i64>().ok()))
}

fn load_import_precheck_state_if_initialized() -> Result<(bool, bool), String> {
    let Some(pool) = get_bulk_import_db_pool() else {
        return Ok((false, false));
    };

    let (design_count, hoop_count) = tauri::async_runtime::block_on(load_catalog_counts(&pool))?;
    let is_first_import = design_count == 0;
    let needs_hoop_setup = is_first_import && hoop_count == 0;
    Ok((is_first_import, needs_hoop_setup))
}

async fn load_import_precheck_state_if_initialized_async() -> Result<(bool, bool), String> {
    let Some(pool) = get_bulk_import_db_pool() else {
        return Ok((false, false));
    };

    let (design_count, hoop_count) = load_catalog_counts(&pool).await?;
    let is_first_import = design_count == 0;
    let needs_hoop_setup = is_first_import && hoop_count == 0;
    Ok((is_first_import, needs_hoop_setup))
}

fn normalize_path_for_match(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

fn strip_sqlite_prefix(database_url: &str) -> &str {
    database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url)
}

fn derive_data_root_from_database_url() -> PathBuf {
    let config = BootstrapConfig::from_env();
    let db_path = Path::new(strip_sqlite_prefix(&config.database_url));

    let root = if let Some(parent) = db_path.parent() {
        if parent
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case("database"))
            .unwrap_or(false)
        {
            parent.parent().unwrap_or(parent)
        } else {
            parent
        }
    } else {
        Path::new("data")
    };

    root.canonicalize().unwrap_or_else(|_| root.to_path_buf())
}

fn get_designs_base_path() -> PathBuf {
    derive_data_root_from_database_url().join("MachineEmbroideryDesigns")
}

/// Returns whether `full_path` resides under the canonical designs base directory.
/// Uses case-insensitive, separator-normalized boundary-safe prefix matching so that
/// only files genuinely under AppRoot/data/MachineEmbroideryDesigns are treated as in-library.
fn is_path_under_designs_base(full_path: &str) -> bool {
    let normalized = full_path.trim().replace('\\', "/");
    let normalized_lower = normalized.to_ascii_lowercase();

    let designs_base = get_designs_base_path();
    let base_norm = designs_base.to_string_lossy().replace('\\', "/");
    let base_lower = base_norm.to_ascii_lowercase();

    if normalized_lower == base_lower {
        return true;
    }

    let base_prefix = format!("{}/", base_lower.trim_end_matches('/'));
    normalized_lower.starts_with(&base_prefix)
}

/// Converts a full on-disk file path under the designs base directory to the
/// canonical stored filepath (e.g. `sub/design.pes`, relative to the library
/// root — forward slashes, no leading slash, no `MachineEmbroideryDesigns`
/// prefix). Uses strict canonical-base-prefix validation instead of substring
/// matching, so unrelated paths containing "machineembroiderydesigns" in their
/// name do not bypass the copy guard.
fn full_path_to_stored_design_filepath(full_path: &str) -> Result<String, String> {
    let normalized_full = full_path.trim().replace('\\', "/");
    if normalized_full.is_empty() {
        return Err("Import filepath is empty.".to_string());
    }

    let designs_base = get_designs_base_path();

    // Use the shared prefix-check helper
    if !is_path_under_designs_base(&normalized_full) {
        return Err(format!(
            "Selected file is outside catalogue design storage. Expected under '{}', got '{}'.",
            designs_base.to_string_lossy(),
            full_path
        ));
    }

    // Reduce the full path under the library root to its canonical relative form.
    crate::paths::design_rel_from_full(&normalized_full, &designs_base).ok_or_else(|| {
        format!(
            "Selected path is the library root itself (not a design file): '{}'",
            full_path
        )
    })
}

/// Pure helper: computes the prospective stored filepath for a file given its
/// absolute path and the selected import root_paths, without touching the filesystem.
/// This is the single source of truth for path mapping used by both preview dedup
/// and confirm-time import.
///
/// Path construction rules:
/// 1. If the file is already under the designs base, its stored path is derived directly.
/// 2. Otherwise, find the longest matching root_path, extract the root folder leaf
///    (the last component of the root), and build the canonical stored path
///    `{root_leaf}/{relative_subpath}` relative to the library root.
/// 3. Drive-letter-only roots (e.g. `C:/`) have no natural leaf; files are placed
///    directly under the library root using the path relative to the drive root.
fn compute_prospective_stored_filepath(
    full_path: &str,
    root_paths: &[String],
) -> Result<String, String> {
    // Fast path: file already under the managed directory
    if let Ok(stored) = full_path_to_stored_design_filepath(full_path) {
        return Ok(stored);
    }

    let source = Path::new(full_path);
    let source_norm = source.to_string_lossy().replace('\\', "/");

    // Find the longest matching import root.
    let rel_path = root_paths
        .iter()
        .map(|root| root.replace('\\', "/").trim_end_matches('/').to_string())
        .filter(|root| {
            let root_lower = root.to_ascii_lowercase();
            let source_lower = source_norm.to_ascii_lowercase();
            if let Some(rest) = source_lower.strip_prefix(&root_lower) {
                rest.is_empty() || rest.starts_with('/')
            } else {
                false
            }
        })
        .max_by_key(|root| root.len())
        .map(|root| {
            let root_folder_name = Path::new(&root)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("import");

            let root_lower = root.to_ascii_lowercase();
            let source_lower = source_norm.to_ascii_lowercase();

            // Detect drive-letter-only root: e.g. "C:" or "C:/" => no natural leaf.
            // Canonical drive-root paths on Windows look like "C:" or "C:/".
            let is_drive_root =
                root.len() <= 3 && root.ends_with(':') || (root.len() <= 4 && root.ends_with(":/"));

            if is_drive_root {
                // Place files directly under /MachineEmbroideryDesigns using
                // the path relative to the drive root.
                if source_lower.len() > root_lower.len() {
                    let after_root = &source_norm[root.len()..];
                    let sub_path = after_root.trim_start_matches('/');
                    if sub_path.is_empty() {
                        source
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string()
                    } else {
                        sub_path.to_string()
                    }
                } else {
                    source
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                }
            } else if source_lower.len() > root_lower.len() {
                let after_root = &source_norm[root.len()..];
                let sub_path = after_root.trim_start_matches('/');
                if sub_path.is_empty() {
                    root_folder_name.to_string()
                } else {
                    format!("{}/{}", root_folder_name, sub_path)
                }
            } else {
                let filename = source
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                format!("{}/{}", root_folder_name, filename)
            }
        })
        .unwrap_or_else(|| {
            source
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    // rel_path is already a library-relative path; canonicalise it (this also
    // strips any stray `MachineEmbroideryDesigns` prefix / separators).
    Ok(crate::paths::canonical_design_rel(&rel_path))
}

/// Compute BLAKE3 hash of a file. Returns hex-encoded string.
pub(crate) fn compute_file_hash_blake3(file_path: &Path) -> Result<String, String> {
    let mut file = File::open(file_path).map_err(|e| {
        format!(
            "Failed to open file for hashing '{}': {}",
            file_path.display(),
            e
        )
    })?;

    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65536]; // 64 KiB buffer
    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| {
            format!(
                "Failed to read file for hashing '{}': {}",
                file_path.display(),
                e
            )
        })?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Get file size in bytes via metadata.
pub(crate) fn compute_file_size(file_path: &Path) -> Result<i64, String> {
    let metadata = fs::metadata(file_path).map_err(|e| {
        format!(
            "Failed to read metadata for '{}': {}",
            file_path.display(),
            e
        )
    })?;
    Ok(metadata.len() as i64)
}

/// Ensures a file is located under the managed MachineEmbroideryDesigns directory.
/// If the file is already under that directory, returns the stored filepath directly
/// (no copy needed for in-library files).
/// If the file is outside, copies it into the managed directory using the path
/// computed by `compute_prospective_stored_filepath`.
///
/// Collision policy (Phase 4):
/// - If destination exists and content matches (same BLAKE3 + size), reuse the
///   existing stored path (no copy).
/// - If destination exists and content differs, auto-rename the new file
///   (stem + _1, _2, etc.) and return the renamed stored path.
/// - The resulting stored filepath is what gets persisted and returned.
fn ensure_file_in_designs_base(full_path: &str, root_paths: &[String]) -> Result<String, String> {
    // Fast path: file is already under MachineEmbroideryDesigns (in-library)
    if let Ok(stored) = full_path_to_stored_design_filepath(full_path) {
        return Ok(stored);
    }

    // Copy path: file is outside the managed directory
    let source = Path::new(full_path);
    if !source.exists() {
        return Err(format!("Import file does not exist: '{}'", full_path));
    }

    // Pre-compute source file content fingerprint for collision detection
    let source_size = compute_file_size(source)?;
    let source_hash = compute_file_hash_blake3(source)?;

    let designs_base = get_designs_base_path();

    // `compute_prospective_stored_filepath` returns the canonical library-relative
    // stored path (e.g. "testdata/Bean.pes"). Rejoin it under the library root to
    // determine the on-disk copy destination.
    let prospective_stored = compute_prospective_stored_filepath(full_path, root_paths)?;
    let rel_path = prospective_stored.trim_start_matches('/');

    let dest = designs_base.join(rel_path);

    // Check if destination already exists
    if dest.exists() {
        // Compute hash of existing destination file
        let dest_size = compute_file_size(&dest).unwrap_or(0);
        let dest_hash = compute_file_hash_blake3(&dest).unwrap_or_default();

        if dest_size == source_size && dest_hash == source_hash {
            // Content matches â€” reuse existing stored filepath, no copy needed
            tracing::info!(
                "Import file '{}' content-identical to existing '{}' â€” reusing stored path",
                source.display(),
                dest.display()
            );
            return Ok(prospective_stored);
        }

        // Content differs â€” auto-rename the new file
        let dest_parent = dest.parent().ok_or_else(|| {
            format!(
                "Cannot determine parent directory for destination: '{}'",
                dest.display()
            )
        })?;

        let stem = dest
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("design");
        let ext = dest.extension().and_then(|n| n.to_str()).unwrap_or("");

        let mut counter = 1u32;
        let final_dest = loop {
            let candidate_name = if ext.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, ext)
            };
            let candidate = dest_parent.join(&candidate_name);
            if !candidate.exists() {
                break candidate;
            }
            counter += 1;
            if counter > 1000 {
                return Err(format!(
                    "Failed to find available auto-rename target for '{}' after 1000 attempts",
                    dest.display()
                ));
            }
        };

        tracing::info!(
            "Import collision: '{}' exists with different content â€” auto-renaming to '{}'",
            dest.display(),
            final_dest.display()
        );

        fs::create_dir_all(dest_parent).map_err(|e| {
            format!(
                "Failed to create directory '{}': {}",
                dest_parent.display(),
                e
            )
        })?;

        fs::copy(source, &final_dest).map_err(|e| {
            format!(
                "Failed to copy '{}' to '{}': {}",
                source.display(),
                final_dest.display(),
                e
            )
        })?;

        // Compute stored filepath from the renamed copy
        return full_path_to_stored_design_filepath(&final_dest.to_string_lossy());
    }

    // No collision: copy to the computed destination
    let dest_parent = dest.parent().ok_or_else(|| {
        format!(
            "Cannot determine parent directory for destination: '{}'",
            dest.display()
        )
    })?;

    fs::create_dir_all(dest_parent).map_err(|e| {
        format!(
            "Failed to create directory '{}': {}",
            dest_parent.display(),
            e
        )
    })?;

    fs::copy(source, &dest).map_err(|e| {
        format!(
            "Failed to copy '{}' to '{}': {}",
            source.display(),
            dest.display(),
            e
        )
    })?;

    // Now compute the stored filepath from the copy destination
    full_path_to_stored_design_filepath(&dest.to_string_lossy())
}

fn normalize_name_for_import_matching(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace(['_', '-', '/', '\\'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn compact_name_for_import_matching(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn strip_web_affixes_for_import_matching(value: &str) -> String {
    let mut compact = compact_name_for_import_matching(value);

    if let Some(stripped) = compact.strip_prefix("www") {
        compact = stripped.to_string();
    }

    for suffix in ["comau", "couk", "com", "net", "org", "co", "uk"] {
        if compact.len() > suffix.len() + 2 && compact.ends_with(suffix) {
            compact.truncate(compact.len() - suffix.len());
            break;
        }
    }

    compact
}

fn suggest_reference_id_from_path(path_value: &str, items: &[(i64, String)]) -> Option<i64> {
    let normalized_path = normalize_name_for_import_matching(path_value);
    let compact_path = compact_name_for_import_matching(path_value);
    if normalized_path.is_empty() && compact_path.is_empty() {
        return None;
    }

    for (item_id, item_name) in items {
        let raw_name = item_name.trim();
        if raw_name.is_empty() {
            continue;
        }

        let lowered = raw_name.to_ascii_lowercase();
        if lowered == "don't know" || lowered == "me" {
            continue;
        }

        let normalized_name = normalize_name_for_import_matching(raw_name);
        let compact_name = compact_name_for_import_matching(raw_name);
        let stripped_compact_name = strip_web_affixes_for_import_matching(raw_name);
        if (!normalized_name.is_empty() && normalized_path.contains(&normalized_name))
            || (!compact_name.is_empty() && compact_path.contains(&compact_name))
            || (!stripped_compact_name.is_empty() && compact_path.contains(&stripped_compact_name))
        {
            return Some(*item_id);
        }
    }

    None
}

fn infer_assignment_ids_from_folder_path(
    folder_path: &str,
    designers: &[(i64, String)],
    sources: &[(i64, String)],
) -> (Option<i64>, Option<i64>) {
    (
        suggest_reference_id_from_path(folder_path, designers),
        suggest_reference_id_from_path(folder_path, sources),
    )
}

fn folder_path_from_file_path(file_path: &str) -> Option<String> {
    let path_text = file_path.trim();
    if path_text.is_empty() {
        return None;
    }

    Path::new(path_text)
        .parent()
        .map(|parent| parent.to_string_lossy().trim().to_string())
        .filter(|parent| !parent.is_empty())
}

fn build_preview_folder_assignments(
    wire: &BulkImportWire,
    scanned_files: &[scanning::ScannedFile],
) -> Vec<FolderAssignmentWire> {
    let mut assignments_by_path = HashMap::<String, FolderAssignmentWire>::new();

    for assignment in &wire.per_folder_assignments {
        assignments_by_path.insert(
            normalize_path_for_match(&assignment.folder_path),
            assignment.clone(),
        );
    }

    for scanned_file in scanned_files {
        if let Some(folder_path) = folder_path_from_file_path(&scanned_file.full_path) {
            let normalized_folder = normalize_path_for_match(&folder_path);
            assignments_by_path
                .entry(normalized_folder)
                .or_insert_with(|| FolderAssignmentWire {
                    folder_path,
                    designer_id: None,
                    source_id: None,
                    inferred_designer_id: None,
                    inferred_source_id: None,
                });
        }
    }

    let mut assignments = assignments_by_path
        .into_values()
        .collect::<Vec<FolderAssignmentWire>>();
    assignments.sort_by(|left, right| {
        left.folder_path
            .to_ascii_lowercase()
            .cmp(&right.folder_path.to_ascii_lowercase())
    });
    assignments
}

async fn load_designers_for_import_inference(
    pool: &SqlitePool,
) -> Result<Vec<(i64, String)>, String> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM designers ORDER BY LENGTH(name) DESC, name ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())
}

async fn load_sources_for_import_inference(
    pool: &SqlitePool,
) -> Result<Vec<(i64, String)>, String> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM sources ORDER BY LENGTH(name) DESC, name ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())
}

fn resolve_assignment_for_file(
    file_path: &str,
    confirm_wire: &BulkImportConfirmWire,
    resolved_assignments: &[ResolvedFolderAssignmentWire],
) -> (Option<i64>, Option<i64>) {
    let normalized_file = normalize_path_for_match(file_path);

    let mut best_match: Option<(&ResolvedFolderAssignmentWire, usize)> = None;
    for assignment in resolved_assignments {
        let normalized_folder = normalize_path_for_match(&assignment.folder_path);
        if normalized_file.starts_with(&normalized_folder) {
            let score = normalized_folder.len();
            if best_match
                .map(|(_, best_score)| score > best_score)
                .unwrap_or(true)
            {
                best_match = Some((assignment, score));
            }
        }
    }

    if let Some((assignment, _)) = best_match {
        return (assignment.designer_id.value, assignment.source_id.value);
    }

    (
        confirm_wire.wire.global_designer_id,
        confirm_wire.wire.global_source_id,
    )
}

async fn persist_bulk_import_confirm_wire(
    pool: &SqlitePool,
    confirm_wire: &BulkImportConfirmWire,
    context_token: Option<&str>,
) -> Result<usize, String> {
    if !confirm_wire.wire.create_on_import {
        return Ok(0);
    }

    let resolved_assignments = resolve_bulk_import_assignments(confirm_wire);
    let preview_3d = false;
    let preview_3d_profile = "balanced".to_string();
    let commit_batch_size = load_import_commit_batch_size(pool).await?;
    let tag_catalog = load_tag_catalog(pool).await?;
    let valid_descriptions: HashSet<String> = tag_catalog
        .iter()
        .map(|(_, description)| description.clone())
        .collect();
    let description_to_tag_id: HashMap<String, i64> = tag_catalog
        .into_iter()
        .map(|(tag_id, description)| (description, tag_id))
        .collect();
    let stitching_tag_lookup = load_stitching_tag_lookup(pool).await?;
    let valid_stitching_descriptions: HashSet<String> =
        stitching_tag_lookup.keys().cloned().collect();
    let default_stitching_tag_id = load_default_stitching_tag_id(pool).await?;
    let total_count = confirm_wire.wire.selected_files.len();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    BULK_IMPORT_STOP_REQUESTED.store(false, Ordering::SeqCst);
    let mut persisted_design_count = 0usize;
    let mut committed_design_count = 0usize;
    let mut persisted_since_last_commit = 0usize;
    let mut processed_count = 0usize;
    let mut stopped = false;
    let mut imported_design_ids: Vec<i64> = Vec::new();

    // Timing accumulators
    let import_start = Instant::now();
    let total_dedup_check_ms = 0u128;
    let mut total_image_gen_ms = 0u128;
    let mut total_db_insert_ms = 0u128;
    let mut total_tagging_ms = 0u128;
    let mut total_commit_ms = 0u128;

    let emit_progress = |stage: &str,
                         processed_count: usize,
                         persisted_count: usize,
                         committed_count: usize,
                         current_file: Option<&str>| {
        if let Some(handle) = get_bulk_import_app_handle() {
            let event = BulkImportProgressEvent {
                context_token: context_token.map(String::from),
                stage: stage.to_string(),
                processed_count,
                total_count,
                persisted_count,
                committed_count,
                current_file: current_file.map(String::from),
                commit_batch_size,
            };

            if let Err(error) = handle.emit(BULK_IMPORT_PROGRESS_EVENT, event) {
                tracing::error!("Failed to emit bulk import progress event: {error}");
            }
        }
    };

    emit_progress(
        "started",
        processed_count,
        persisted_design_count,
        committed_design_count,
        None,
    );

    // Process files in chunks aligned to commit_batch_size.
    // For each chunk: generate previews, insert into DB, then commit.
    // Cancellation takes effect between chunks.
    let all_files = confirm_wire.wire.selected_files.clone();
    let mut chunk_start = 0usize;

    while chunk_start < total_count {
        if BULK_IMPORT_STOP_REQUESTED.load(Ordering::SeqCst) {
            stopped = true;
            break;
        }

        let chunk_end = (chunk_start + commit_batch_size).min(total_count);
        let chunk = &all_files[chunk_start..chunk_end];

        for file_path in chunk {
            if BULK_IMPORT_STOP_REQUESTED.load(Ordering::SeqCst) {
                stopped = true;
                break;
            }

            let stored_filepath =
                ensure_file_in_designs_base(file_path, &confirm_wire.wire.root_paths)?;

            emit_progress(
                "processing_file",
                processed_count,
                persisted_design_count,
                committed_design_count,
                Some(file_path),
            );

            let (designer_id, source_id) =
                resolve_assignment_for_file(file_path, confirm_wire, &resolved_assignments);

            let filename = Path::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(file_path)
                .to_string();

            let t_image = Instant::now();
            let image_result =
                image_generation::generate_preview(&image_generation::ImageGenerationRequest {
                    file_path: file_path.clone(),
                    preview_3d,
                    preview_3d_profile: Some(preview_3d_profile.clone()),
                });
            let image_gen_ms = t_image.elapsed().as_millis();
            total_image_gen_ms += image_gen_ms;
            tracing::debug!(
                "[TIMING] file={} backend={} image_gen={}ms{}",
                filename,
                image_result.backend,
                image_gen_ms,
                image_result
                    .error
                    .as_deref()
                    .map(|e| format!(" error={e}"))
                    .unwrap_or_default(),
            );
            if let Some(error) = image_result.error.as_ref() {
                tracing::error!(
                    "Image generation adapter error for '{}': {}",
                    file_path,
                    error
                );
            }

            let hoop_id = match (image_result.width_mm, image_result.height_mm) {
                (Some(width_mm), Some(height_mm)) => sqlx::query_scalar::<_, i64>(
                    r#"
                        SELECT h.id
                        FROM hoops h
                        WHERE
                            (
                                CAST(h.max_width_mm AS REAL) >= CAST(? AS REAL)
                                AND CAST(h.max_height_mm AS REAL) >= CAST(? AS REAL)
                            )
                            OR (
                                CAST(h.max_width_mm AS REAL) >= CAST(? AS REAL)
                                AND CAST(h.max_height_mm AS REAL) >= CAST(? AS REAL)
                            )
                        ORDER BY
                            (CAST(h.max_width_mm AS REAL) * CAST(h.max_height_mm AS REAL)) ASC,
                            CAST(h.max_width_mm AS REAL) ASC,
                            CAST(h.max_height_mm AS REAL) ASC,
                            h.name COLLATE NOCASE ASC
                        LIMIT 1
                        "#,
                )
                .bind(width_mm)
                .bind(height_mm)
                .bind(height_mm)
                .bind(width_mm)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?,
                _ => None,
            };

            // Compute content fingerprint from the actual stored file for confirm-time persistence
            let designs_base_path = get_designs_base_path();
            let stored_path = designs_base_path.join(
                stored_filepath
                    .strip_prefix("/MachineEmbroideryDesigns/")
                    .unwrap_or(&stored_filepath),
            );
            let file_size_bytes: Option<i64> = compute_file_size(&stored_path).ok();
            let file_hash_blake3: Option<String> = compute_file_hash_blake3(&stored_path).ok();

            let t_insert = Instant::now();
            let insert_result = sqlx::query(
                "INSERT INTO designs (filename, filepath, date_added, designer_id, source_id, hoop_id, image_data, image_type, width_mm, height_mm, stitch_count, color_count, color_change_count, is_stitched, image_tags_verified, stitching_tags_verified, file_size_bytes, file_hash_blake3) VALUES (?, ?, DATE('now'), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, ?, ?)",
            )
            .bind(&filename)
            .bind(&stored_filepath)
            .bind(designer_id)
            .bind(source_id)
            .bind(hoop_id)
            .bind(image_result.image_data)
            .bind(image_result.image_type)
            .bind(image_result.width_mm)
            .bind(image_result.height_mm)
            .bind(image_result.stitch_count)
            .bind(image_result.color_count)
            .bind(image_result.color_change_count)
            .bind(file_size_bytes)
            .bind(file_hash_blake3.as_ref())
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            total_db_insert_ms += t_insert.elapsed().as_millis();

            let design_id = insert_result.last_insert_rowid();
            imported_design_ids.push(design_id);
            let t_tag = Instant::now();
            let matched_descriptions = tagging::suggest_path_rule_descriptions(
                &filename,
                &stored_filepath,
                &valid_descriptions,
            );

            let mut stitching_tag_ids: Vec<i64> = Vec::new();
            if Path::new(file_path).exists() {
                let detected_stitching_descriptions =
                    stitch_identifier::suggest_stitching_from_pattern_file(
                        file_path,
                        &filename,
                        &stored_filepath,
                        &valid_stitching_descriptions,
                        Some(0.70),
                    );

                stitching_tag_ids = detected_stitching_descriptions
                    .iter()
                    .filter_map(|description| stitching_tag_lookup.get(description).copied())
                    .collect();

                if stitching_tag_ids.is_empty() {
                    if let Some(default_tag_id) = default_stitching_tag_id {
                        stitching_tag_ids.push(default_tag_id);
                    }
                }
            }

            stitching_tag_ids.sort_unstable();
            stitching_tag_ids.dedup();

            if !matched_descriptions.is_empty() {
                for description in &matched_descriptions {
                    if let Some(tag_id) = description_to_tag_id.get(description) {
                        sqlx::query(
                            "INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)",
                        )
                        .bind(design_id)
                        .bind(*tag_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    }
                }
            }

            for tag_id in &stitching_tag_ids {
                sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
                    .bind(design_id)
                    .bind(*tag_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            }

            // Import-time path-rule keyword tags are offline local matching, not AI
            // analysis, so no per-mode AI flags (`text_ai_*` / `vision_ai_*`) are set.
            total_tagging_ms += t_tag.elapsed().as_millis();

            persisted_design_count += 1;
            persisted_since_last_commit += 1;
            processed_count += 1;

            emit_progress(
                "processed",
                processed_count,
                persisted_design_count,
                committed_design_count,
                Some(file_path),
            );
        }

        // Commit after each chunk (covers both normal progress and mid-chunk stop).
        let t_commit = Instant::now();
        tx.commit().await.map_err(|e| e.to_string())?;
        let commit_ms = t_commit.elapsed().as_millis();
        total_commit_ms += commit_ms;
        committed_design_count += persisted_since_last_commit;
        if persisted_since_last_commit > 0 {
            tracing::debug!(
                "Bulk import committed chunk [{}-{}]: {} design(s), commit={}ms.",
                chunk_start,
                chunk_end - 1,
                persisted_since_last_commit,
                commit_ms
            );
            emit_progress(
                "batch_committed",
                processed_count,
                persisted_design_count,
                committed_design_count,
                None,
            );
        }
        tx = pool.begin().await.map_err(|e| e.to_string())?;
        persisted_since_last_commit = 0;

        if stopped {
            break;
        }
        chunk_start = chunk_end;
    }

    // Commit the empty transaction left open after the last chunk (or any partial
    // state from an abrupt stop before the chunk-level commit ran).
    let t_final_commit = Instant::now();
    tx.commit().await.map_err(|e| e.to_string())?;
    total_commit_ms += t_final_commit.elapsed().as_millis();
    committed_design_count += persisted_since_last_commit;

    let total_elapsed_ms = import_start.elapsed().as_millis();
    tracing::info!(
        "[TIMING] Bulk import complete: total={}ms | dedup_check={}ms | image_gen={}ms | db_insert={}ms | tagging={}ms | commits={}ms | persisted={} skipped={}",
        total_elapsed_ms,
        total_dedup_check_ms,
        total_image_gen_ms,
        total_db_insert_ms,
        total_tagging_ms,
        total_commit_ms,
        persisted_design_count,
        processed_count.saturating_sub(persisted_design_count),
    );

    // Post-commit Gemini auto-tagging pass (non-fatal, serial, free-tier-paced).
    // Runs only when the import completed and wasn't stopped. The designs are
    // already committed, so this never holds the import transaction open for the
    // duration of slow Gemini network calls.
    if !stopped {
        run_import_ai_tagging_pass(
            pool,
            &imported_design_ids,
            &valid_descriptions,
            &description_to_tag_id,
            &emit_progress,
        )
        .await;
    }

    if stopped {
        emit_progress(
            "stopped",
            processed_count,
            persisted_design_count,
            committed_design_count,
            None,
        );
        return Ok(persisted_design_count);
    }

    emit_progress(
        "completed",
        processed_count,
        persisted_design_count,
        committed_design_count,
        None,
    );
    Ok(persisted_design_count)
}

/// Post-commit Gemini auto-tagging pass for a bulk import. Runs AFTER the import
/// has fully committed, so it never holds the import transaction open for the
/// duration of slow Gemini network calls. Serial (one Gemini request at a time),
/// paced by the configured AI delay (free-tier default 10 s), batching tag writes
/// into a few transactions. The completed import is never failed by this pass —
/// errors are logged and skipped, and a 429 stops the pass hard with a clear
/// "wait" message (mirroring the backfill/Tagging Actions behaviour).
async fn run_import_ai_tagging_pass(
    pool: &SqlitePool,
    imported_design_ids: &[i64],
    valid_descriptions: &HashSet<String>,
    description_to_tag_id: &HashMap<String, i64>,
    emit_progress: &impl Fn(&str, usize, usize, usize, Option<&str>),
) {
    if imported_design_ids.is_empty() {
        return;
    }

    let vision_auto = read_bool_setting(pool, "ai.vision").await.unwrap_or(false);
    if !vision_auto {
        tracing::info!("Bulk import: Visual AI auto-tagging disabled (no vision auto).");
        return;
    }

    let free_tier = read_bool_setting(pool, "ai.free_tier")
        .await
        .unwrap_or(false);
    let Some(api_key) = read_string_setting(pool, "ai.google_api_key")
        .await
        .unwrap_or(None)
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
    else {
        tracing::info!("Bulk import: Gemini auto-tagging enabled but no API key set; skipping.");
        return;
    };

    let gemini = GeminiClient::new(api_key);
    let delay = read_f64_setting(pool, "ai.delay")
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| auto_tagging::default_delay_for(free_tier));
    // `ai.batch_size` caps how many imported designs get AI-tagged this run
    // (blank → tag all imported designs).
    let batch_cap = read_i64_setting(pool, "ai.batch_size")
        .await
        .unwrap_or(None)
        .filter(|value| *value > 0)
        .map(|value| value as usize)
        .unwrap_or(usize::MAX);

    let mode_options = auto_tagging::TaggingModeOptions {
        path_rule_enabled: true,
        text_ai_enabled: false,
        text_ai_network: false,
        visual_ai_enabled: vision_auto,
        visual_ai_delay_seconds: delay,
        // Pace real Gemini network calls only; local-only modes never sleep.
        visual_ai_network: vision_auto,
    };

    tracing::info!(
        "Bulk import AI tagging pass: ids={} visual_ai={} delay={}s free_tier={} batch_cap={}",
        imported_design_ids.len(),
        vision_auto,
        delay,
        free_tier,
        batch_cap
    );

    emit_progress("ai_tagging", 0, 0, 0, None);

    // Batch tag writes into a handful of transactions (one journal + fsync each)
    // rather than one autocommit per design.
    const AI_TAGGING_BATCH: usize = 100;
    let mut pending: Vec<auto_tagging::TagBatchEntry> = Vec::new();
    let mut processed = 0usize;
    let mut applied = 0usize;
    let mut rate_limit_error: Option<AppError> = None;

    let ids: Vec<i64> = imported_design_ids
        .iter()
        .copied()
        .take(batch_cap)
        .collect();
    for design_id in ids {
        if BULK_IMPORT_STOP_REQUESTED.load(Ordering::SeqCst) {
            break;
        }

        let Ok(Some(row)) =
            sqlx::query("SELECT filename, filepath, image_data FROM designs WHERE id = ?")
                .bind(design_id)
                .fetch_optional(pool)
                .await
        else {
            continue;
        };
        let Ok(filename) = row.try_get::<String, _>("filename") else {
            continue;
        };
        let Ok(filepath) = row.try_get::<String, _>("filepath") else {
            continue;
        };
        let Ok(image_data) = row.try_get::<Option<Vec<u8>>, _>("image_data") else {
            continue;
        };

        let result = auto_tagging::compute_tags_for_input(
            &filename,
            &filepath,
            image_data.as_deref(),
            valid_descriptions,
            &mode_options,
            Some(&gemini),
        )
        .await;

        match result {
            Ok(result) => {
                let matched = !result.descriptions.is_empty();
                pending.push(auto_tagging::TagBatchEntry {
                    design_id,
                    descriptions: result.descriptions,
                    text_ai_analyzed: result.text_ai_analyzed,
                    text_ai_matched: result.text_ai_matched,
                    vision_ai_analyzed: result.vision_ai_analyzed,
                    vision_ai_matched: result.vision_ai_matched,
                });
                if matched {
                    applied += 1;
                }
            }
            Err(error) if gemini_client::is_rate_limit_error(&error) => {
                tracing::warn!("Bulk import AI tagging aborted on rate limit (429).");
                rate_limit_error = Some(error);
                break;
            }
            Err(error) => {
                tracing::warn!("Bulk import AI tagging skipped design {design_id}: {error}");
            }
        }

        processed += 1;
        if pending.len() >= AI_TAGGING_BATCH {
            if let Err(error) = auto_tagging::apply_tagging_batch(
                pool,
                description_to_tag_id,
                std::mem::take(&mut pending),
                "reset",
            )
            .await
            {
                tracing::warn!("Bulk import AI tagging batch write failed: {error}");
            }
        }

        emit_progress("ai_tagging", processed, applied, 0, Some(filename.as_str()));
    }

    if !pending.is_empty() {
        if let Err(error) =
            auto_tagging::apply_tagging_batch(pool, description_to_tag_id, pending, "reset").await
        {
            tracing::warn!("Bulk import AI tagging final batch write failed: {error}");
        }
    }

    if let Some(error) = rate_limit_error {
        let message = auto_tagging::rate_limit_message(&error, free_tier);
        tracing::warn!("{message}");
        emit_progress("ai_tagging", processed, applied, 0, None);
        return;
    }

    tracing::info!(
        "Bulk import AI tagging pass finished: processed={} applied={}",
        processed,
        applied
    );
    emit_progress("ai_tagging", processed, applied, 0, None);
}

fn persist_bulk_import_confirm_if_initialized(
    confirm_wire: &BulkImportConfirmWire,
    context_token: Option<&str>,
) -> Result<usize, String> {
    match get_bulk_import_db_pool() {
        Some(pool) => tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            confirm_wire,
            context_token,
        )),
        None => {
            tracing::warn!("Bulk import DB pool not initialized; skipping persistence step.");
            Ok(0)
        }
    }
}

fn clear_bulk_import_context_store_internal(reason: &str) -> BulkImportContextStoreResetResult {
    let cleared_context_count = with_bulk_import_context_store(|store| {
        let count = store.len();
        store.clear();
        count
    })
    .unwrap_or_default();

    let reset_at_millis = current_timestamp_millis() as u64;
    BULK_IMPORT_CONTEXT_LAST_RESET_AT_MILLIS.store(reset_at_millis, Ordering::Relaxed);
    let reset_count = BULK_IMPORT_CONTEXT_RESET_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;

    BulkImportContextStoreResetResult {
        cleared_context_count,
        active_context_count: 0,
        reset_count,
        reset_at_millis,
        reason: reason.to_string(),
    }
}

pub fn reset_bulk_import_context_store_for_startup() -> BulkImportContextStoreResetResult {
    clear_bulk_import_context_store_internal("startup")
}

/// Clear any in-flight bulk-import contexts after a database restore, so a
/// pre-restore context token cannot be reused against the restored database.
pub fn reset_bulk_import_context_store_for_restore() -> BulkImportContextStoreResetResult {
    clear_bulk_import_context_store_internal("restore")
}

pub fn store_bulk_import_context(confirm_wire: BulkImportConfirmWire) -> String {
    let (token, sequence) = next_bulk_import_context_token();
    let _ = with_bulk_import_context_store(|store| {
        prune_bulk_import_context_store(store);
        let mut stored_wire = canonicalize_bulk_import_confirm_wire(confirm_wire.clone());
        stored_wire.context_token = Some(token.clone());
        store.insert(
            token.clone(),
            StoredBulkImportContext {
                confirm_wire: stored_wire,
                created_at_millis: current_timestamp_millis(),
                sequence,
            },
        );
    });
    token
}

pub fn take_bulk_import_context(token: &str) -> Option<BulkImportConfirmWire> {
    with_bulk_import_context_store(|store| {
        prune_bulk_import_context_store(store);
        store.remove(token).map(|context| context.confirm_wire)
    })
    .unwrap_or_default()
}

pub fn get_bulk_import_context(token: &str) -> Option<BulkImportConfirmWire> {
    with_bulk_import_context_store(|store| {
        prune_bulk_import_context_store(store);
        store.get(token).map(|context| context.confirm_wire.clone())
    })
    .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportContextStoreSummary {
    pub active_context_count: usize,
    pub max_entries: usize,
    pub ttl_seconds: u64,
    pub reset_count: u64,
    pub last_reset_at_millis: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportContextStoreResetResult {
    pub cleared_context_count: usize,
    pub active_context_count: usize,
    pub reset_count: u64,
    pub reset_at_millis: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportStopResult {
    pub stop_requested: bool,
}

#[tauri::command]
pub fn debug_bulk_import_context_store() -> Result<BulkImportContextStoreSummary, String> {
    let active_context_count = with_bulk_import_context_store(|store| {
        prune_bulk_import_context_store(store);
        store.len()
    })
    .unwrap_or_default();
    let last_reset_at_millis = BULK_IMPORT_CONTEXT_LAST_RESET_AT_MILLIS.load(Ordering::Relaxed);

    Ok(BulkImportContextStoreSummary {
        active_context_count,
        max_entries: BULK_IMPORT_CONTEXT_MAX_ENTRIES,
        ttl_seconds: BULK_IMPORT_CONTEXT_TTL.as_secs(),
        reset_count: BULK_IMPORT_CONTEXT_RESET_COUNTER.load(Ordering::Relaxed),
        last_reset_at_millis: if last_reset_at_millis == 0 {
            None
        } else {
            Some(last_reset_at_millis)
        },
    })
}

#[tauri::command]
pub fn reset_bulk_import_context_store() -> Result<BulkImportContextStoreResetResult, String> {
    Ok(clear_bulk_import_context_store_internal("manual"))
}

#[tauri::command]
pub fn request_stop_bulk_import() -> Result<BulkImportStopResult, String> {
    BULK_IMPORT_STOP_REQUESTED.store(true, Ordering::SeqCst);
    Ok(BulkImportStopResult {
        stop_requested: true,
    })
}

#[tauri::command]
pub fn preview_bulk_import(request: BulkImportRequest) -> Result<BulkImportPreview, String> {
    let wire: BulkImportWire = request.into();

    preview_bulk_import_wire(wire)
}

#[tauri::command]
pub fn browse_import_folder(
    request: Option<BulkImportBrowseFolderRequest>,
) -> Result<BulkImportBrowseFolderResult, String> {
    let (start_dir, allow_multi) = match request {
        Some(value) => (value.start_dir, value.allow_multi),
        None => (None, false),
    };
    let result = folder_picker::browse_folder_with_error(start_dir.as_deref(), allow_multi)
        .map_err(|err| err.to_string())?;

    Ok(BulkImportBrowseFolderResult {
        path: result.path,
        paths: result.paths,
    })
}

#[tauri::command]
pub fn debug_bulk_import_wire(wire: BulkImportWire) -> Result<BulkImportWireSummary, String> {
    Ok(BulkImportWireSummary {
        root_path_count: wire.root_paths.len(),
        folder_assignment_count: wire.per_folder_assignments.len(),
        selected_file_count: wire.selected_files.len(),
        create_on_import: wire.create_on_import,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportBrowseFolderResult {
    pub path: Option<String>,
    pub paths: Vec<String>,
}

#[tauri::command]
pub fn debug_bulk_import_confirm_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmSummary, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);
    Ok(BulkImportConfirmSummary {
        context_token_present: confirm_wire.context_token.is_some(),
        root_path_count: confirm_wire.wire.root_paths.len(),
        selected_file_count: confirm_wire.wire.selected_files.len(),
        per_folder_assignment_count: confirm_wire.wire.per_folder_assignments.len(),
        canonical_confirm: confirm_wire.canonical_confirm,
        resolved_assignment_count: resolved_assignments.len(),
        resolved_assignments,
    })
}

#[tauri::command]
pub fn debug_bulk_import_assignment_resolution_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportAssignmentResolutionSummary, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);

    let mut explicit_field_count = 0usize;
    let mut global_field_count = 0usize;
    let mut inferred_field_count = 0usize;
    let mut blank_field_count = 0usize;

    for assignment in &resolved_assignments {
        for field in [&assignment.designer_id, &assignment.source_id] {
            match field.source {
                AssignmentFieldSourceWire::ExplicitPerFolder => explicit_field_count += 1,
                AssignmentFieldSourceWire::Global => global_field_count += 1,
                AssignmentFieldSourceWire::Inferred => inferred_field_count += 1,
                AssignmentFieldSourceWire::Blank => blank_field_count += 1,
            }
        }
    }

    Ok(BulkImportAssignmentResolutionSummary {
        resolved_count: resolved_assignments.len(),
        explicit_field_count,
        global_field_count,
        inferred_field_count,
        blank_field_count,
    })
}

#[tauri::command]
pub fn precheck_bulk_import_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportPrecheckResult, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);
    let (is_first_import, needs_hoop_setup) = load_import_precheck_state_if_initialized()?;
    let context_token = store_bulk_import_context(confirm_wire.clone());

    Ok(BulkImportPrecheckResult {
        context_token,
        context_token_present: true,
        ready_for_confirm: true,
        is_first_import,
        needs_hoop_setup,
        root_path_count: confirm_wire.wire.root_paths.len(),
        selected_file_count: confirm_wire.wire.selected_files.len(),
        resolved_assignments,
    })
}

#[tauri::command]
pub async fn precheck_bulk_import_action_wire(
    request: BulkImportPrecheckActionRequest,
) -> Result<BulkImportPrecheckActionResult, String> {
    let context_token = request.context_token.clone();

    match request.action {
        BulkImportPrecheckActionWire::ReviewHoops => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: true,
                consumed_context: false,
                requires_skip_hoops_confirmation: false,
                next_route: Some(format!("/admin/hoops/?import_token={context_token}")),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::ReviewTags => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: true,
                consumed_context: false,
                requires_skip_hoops_confirmation: false,
                next_route: Some(format!("/admin/tags/?import_token={context_token}")),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::ReviewSources => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: true,
                consumed_context: false,
                requires_skip_hoops_confirmation: false,
                next_route: Some(format!("/admin/sources/?import_token={context_token}")),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::ReviewDesigners => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: true,
                consumed_context: false,
                requires_skip_hoops_confirmation: false,
                next_route: Some(format!("/admin/designers/?import_token={context_token}")),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::Cancel => {
            take_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: false,
                consumed_context: true,
                requires_skip_hoops_confirmation: false,
                next_route: Some("/import/".to_string()),
                confirm_result: None,
            })
        }
        BulkImportPrecheckActionWire::ImportNow => {
            get_bulk_import_context(&context_token).ok_or_else(|| {
                format!("Unknown or expired bulk import context token: {context_token}")
            })?;

            let (is_first_import, needs_hoop_setup) =
                load_import_precheck_state_if_initialized_async().await?;
            let requires_skip_hoops_confirmation =
                is_first_import && needs_hoop_setup && !request.confirm_skip_hoops;

            if requires_skip_hoops_confirmation {
                return Ok(BulkImportPrecheckActionResult {
                    action: request.action,
                    context_token_present: true,
                    consumed_context: false,
                    requires_skip_hoops_confirmation: true,
                    next_route: Some("/import/confirm-skip-hoops/".to_string()),
                    confirm_result: None,
                });
            }

            let confirm_result = tauri::async_runtime::spawn_blocking(move || {
                do_confirm_bulk_import_wire_internal(context_token)
            })
            .await
            .map_err(|error| format!("Import task failed to join: {error}"))??;
            Ok(BulkImportPrecheckActionResult {
                action: request.action,
                context_token_present: false,
                consumed_context: true,
                requires_skip_hoops_confirmation: false,
                next_route: Some("/designs/".to_string()),
                confirm_result: Some(confirm_result),
            })
        }
    }
}

#[tauri::command]
pub fn do_confirm_bulk_import_wire(
    context_token: String,
) -> Result<BulkImportConfirmExecutionResult, String> {
    do_confirm_bulk_import_wire_internal(context_token)
}

fn do_confirm_bulk_import_wire_internal(
    context_token: String,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let confirm_wire = take_bulk_import_context(&context_token)
        .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

    let persisted_design_count =
        persist_bulk_import_confirm_if_initialized(&confirm_wire, Some(&context_token))?;
    let mut result = confirm_bulk_import_wire(confirm_wire)?;
    result.persisted_design_count = persisted_design_count;
    Ok(result)
}

#[tauri::command]
pub fn execute_bulk_import_confirm_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let persisted_design_count = persist_bulk_import_confirm_if_initialized(
        &confirm_wire,
        confirm_wire.context_token.as_deref(),
    )?;
    let mut result = confirm_bulk_import_wire(confirm_wire)?;
    result.persisted_design_count = persisted_design_count;
    Ok(result)
}

#[tauri::command]
pub fn confirm_bulk_import_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);

    Ok(BulkImportConfirmExecutionResult {
        context_token_present: confirm_wire.context_token.is_some(),
        canonical_confirm: true,
        ready_for_persistence: true,
        persisted_design_count: 0,
        root_path_count: confirm_wire.wire.root_paths.len(),
        selected_file_count: confirm_wire.wire.selected_files.len(),
        resolved_assignments,
    })
}

#[tauri::command]
pub fn confirm_bulk_import_legacy(
    request: BulkImportRequest,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let precheck = precheck_bulk_import_wire(BulkImportConfirmWire::from(request))?;
    do_confirm_bulk_import_wire(precheck.context_token)
}

pub fn resolve_assignment_field(
    explicit_value: Option<i64>,
    global_value: Option<i64>,
    inferred_value: Option<i64>,
) -> ResolvedAssignmentFieldWire {
    if let Some(value) = explicit_value {
        return ResolvedAssignmentFieldWire {
            value: Some(value),
            source: AssignmentFieldSourceWire::ExplicitPerFolder,
        };
    }

    if let Some(value) = global_value {
        return ResolvedAssignmentFieldWire {
            value: Some(value),
            source: AssignmentFieldSourceWire::Global,
        };
    }

    if let Some(value) = inferred_value {
        return ResolvedAssignmentFieldWire {
            value: Some(value),
            source: AssignmentFieldSourceWire::Inferred,
        };
    }

    ResolvedAssignmentFieldWire {
        value: None,
        source: AssignmentFieldSourceWire::Blank,
    }
}

pub fn resolve_folder_assignment_wire(
    assignment: &FolderAssignmentWire,
    wire: &BulkImportWire,
) -> ResolvedFolderAssignmentWire {
    ResolvedFolderAssignmentWire {
        folder_path: assignment.folder_path.clone(),
        designer_id: resolve_assignment_field(
            assignment.designer_id,
            wire.global_designer_id,
            assignment.inferred_designer_id,
        ),
        source_id: resolve_assignment_field(
            assignment.source_id,
            wire.global_source_id,
            assignment.inferred_source_id,
        ),
        inferred_designer_id: assignment.inferred_designer_id,
        inferred_source_id: assignment.inferred_source_id,
    }
}

pub fn resolve_bulk_import_assignments(
    confirm_wire: &BulkImportConfirmWire,
) -> Vec<ResolvedFolderAssignmentWire> {
    confirm_wire
        .wire
        .per_folder_assignments
        .iter()
        .map(|assignment| resolve_folder_assignment_wire(assignment, &confirm_wire.wire))
        .collect()
}

/// Preview-phase dedupe: excludes scanned files already present in the DB,
/// checking by prospective stored filepath AND by filename-aware content
/// fingerprint (filename + file_size_bytes + file_hash_blake3).
///
/// Duplicate definition: a file on disk is a duplicate only if ALL THREE
/// properties match an existing DB row:
///   - filename (case-insensitive)
///   - file_size_bytes
///   - file_hash_blake3
///
/// Two files with identical content but different filenames are NOT duplicates
/// and both are allowed into the catalogue.
///
/// Performance: BLAKE3 hashing is lazy â€” only computed when a scanned file
/// already matches an existing row on (filename + file_size_bytes).
async fn filter_existing_scanned_files(
    pool: &SqlitePool,
    scanned_files: Vec<scanning::ScannedFile>,
    root_paths: &[String],
) -> Result<Vec<scanning::ScannedFile>, String> {
    if scanned_files.is_empty() {
        return Ok(scanned_files);
    }

    // Stage 0: Load existing stored filepath set for path-based exclusion
    let existing_paths = sqlx::query_scalar::<_, String>("SELECT filepath FROM designs")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let existing_path_set: HashSet<String> = existing_paths
        .into_iter()
        .map(|path| normalize_path_for_match(&path))
        .collect();

    // Stage 1: Load (filename, file_size_bytes, file_hash_blake3) triples
    // for filename-aware duplicate detection.
    let fingerprint_rows: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT filename, file_size_bytes, file_hash_blake3 FROM designs WHERE file_size_bytes IS NOT NULL AND file_hash_blake3 IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Build a set of (filename_lower, size, hash_lower) triples for O(1) lookup.
    // The hash is already stored in the DB â€” never re-computed for existing rows.
    let fingerprint_set: HashSet<(String, i64, String)> = fingerprint_rows
        .into_iter()
        .map(|(filename, size, hash)| {
            (
                filename.to_ascii_lowercase(),
                size,
                hash.to_ascii_lowercase(),
            )
        })
        .collect();

    let mut result: Vec<scanning::ScannedFile> = Vec::with_capacity(scanned_files.len());
    let mut excluded_by_path: usize = 0;
    let mut excluded_by_triple: usize = 0;

    for file in scanned_files {
        // Stage 0: Compute prospective stored filepath and check against DB paths
        let prospective_path = compute_prospective_stored_filepath(&file.full_path, root_paths)
            .unwrap_or_else(|_| format!("/MachineEmbroideryDesigns/{}", file.full_path));

        let normalized_prospective = normalize_path_for_match(&prospective_path);

        if existing_path_set.contains(&normalized_prospective) {
            excluded_by_path += 1;
            continue;
        }

        // Stage 1: Quick (filename + size) match â€” no hashing yet
        let filename_lower = file.filename.to_ascii_lowercase();
        let file_size = match file.file_size_bytes {
            Some(size) => size,
            None => {
                // File metadata unavailable (e.g. deleted during scan).
                // Cannot match any fingerprint; include it (will fail later at persist).
                result.push(file);
                continue;
            }
        };

        // Check if any existing row shares (filename_lower, file_size).
        // If not, this is a genuinely new design â€” fast path, no BLAKE3.
        if fingerprint_set.is_empty() {
            result.push(file);
            continue;
        }

        // Scan through the set for any (filename, size) collision.
        // With ~126K rows we use a single-pass filter. If a collision exists
        // we must compute the BLAKE3 hash of the scanned file to compare.
        let candidate_triples: Vec<&(String, i64, String)> = fingerprint_set
            .iter()
            .filter(|(fname, fsize, _)| *fname == filename_lower && *fsize == file_size)
            .collect();

        if candidate_triples.is_empty() {
            // No (filename, size) match â€” unique design, include without hashing
            result.push(file);
            continue;
        }

        // Stage 2: Lazy BLAKE3 â€” only computed when (filename + size) collide
        let source_path = Path::new(&file.full_path);
        if !source_path.exists() {
            // File vanished; include it (will fail at persist with clear error)
            result.push(file);
            continue;
        }

        let file_hash = match compute_file_hash_blake3(source_path) {
            Ok(hash) => hash.to_ascii_lowercase(),
            Err(_) => {
                // Hashing failed (e.g. permission error); include it
                result.push(file);
                continue;
            }
        };

        // Check if full triple (filename, size, hash) matches any existing row
        let is_duplicate = candidate_triples
            .iter()
            .any(|(_, _, existing_hash)| *existing_hash == file_hash);

        if is_duplicate {
            excluded_by_triple += 1;
            continue;
        }

        // Same filename + same size but different hash â€” modified file, include it
        result.push(file);
    }

    if excluded_by_path > 0 || excluded_by_triple > 0 {
        tracing::info!(
            "Preview dedup: excluded_by_path={} excluded_by_triple={} imported={}",
            excluded_by_path,
            excluded_by_triple,
            result.len()
        );
    }

    Ok(result)
}

fn preview_bulk_import_wire_with_pool(
    wire: BulkImportWire,
    pool: Option<&SqlitePool>,
) -> Result<BulkImportPreview, String> {
    for root_path in &wire.root_paths {
        validation::validate_path(root_path).map_err(|e| format!("{:?}", e))?;
    }

    let mut scanned_files = Vec::new();
    let mut missing_root = false;
    let mut root_had_any_existing_dir = false;
    for root_path in &wire.root_paths {
        let scan_input = scanning::ScanInput {
            root_path: root_path.clone(),
        };
        let scan_result = scanning::scan_with_error(&scan_input).map_err(|err| err.to_string())?;
        missing_root = missing_root || scan_result.missing_root;
        root_had_any_existing_dir = root_had_any_existing_dir || !scan_result.missing_root;
        scanned_files.extend(scan_result.files);
    }
    // Empty-path / relative-path roots were already rejected above by
    // validation::validate_path. Any arrival here means roots were absolute
    // and non-empty. So invalid_root is always false in this function.
    let invalid_root = false;
    // If every selected root was missing, the user needs to know the path is
    // wrong. If at least one root existed but zero files were found, they need
    // to know no supported embroidery extensions were discovered.
    let no_supported_files = scanned_files.is_empty() && root_had_any_existing_dir;

    scanned_files.sort_by(|left, right| {
        left.full_path
            .to_ascii_lowercase()
            .cmp(&right.full_path.to_ascii_lowercase())
    });

    if let Some(active_pool) = pool {
        scanned_files = tauri::async_runtime::block_on(filter_existing_scanned_files(
            active_pool,
            scanned_files,
            &wire.root_paths,
        ))?;
    }

    let discovered_count = scanned_files.len();

    let mut preview_assignments = build_preview_folder_assignments(&wire, &scanned_files);

    if let Some(active_pool) = pool {
        let designers =
            tauri::async_runtime::block_on(load_designers_for_import_inference(active_pool))?;
        let sources =
            tauri::async_runtime::block_on(load_sources_for_import_inference(active_pool))?;

        for assignment in &mut preview_assignments {
            let (inferred_designer_id, inferred_source_id) = infer_assignment_ids_from_folder_path(
                &assignment.folder_path,
                &designers,
                &sources,
            );
            assignment.inferred_designer_id = inferred_designer_id;
            assignment.inferred_source_id = inferred_source_id;
        }
    }

    let resolved_assignments = preview_assignments
        .iter()
        .map(|assignment| {
            let _legacy_resolved = folder_picker::resolve_assignment(
                &folder_picker::FolderAssignment {
                    folder_path: assignment.folder_path.clone(),
                    designer_id: assignment.designer_id,
                    source_id: assignment.source_id,
                },
                &folder_picker::AssignmentFallback {
                    designer_id: wire.global_designer_id,
                    source_id: wire.global_source_id,
                },
            );

            resolve_folder_assignment_wire(assignment, &wire)
        })
        .collect();

    Ok(BulkImportPreview {
        discovered_count,
        selected_count: wire.selected_files.len(),
        folder_count: wire.root_paths.len(),
        scanned_files,
        resolved_assignments,
        missing_root,
        invalid_root,
        no_supported_files,
    })
}

pub fn preview_bulk_import_wire(wire: BulkImportWire) -> Result<BulkImportPreview, String> {
    let pool = get_bulk_import_db_pool();
    preview_bulk_import_wire_with_pool(wire, pool.as_ref())
}
#[cfg(test)]
#[path = "bulk_import_tests.rs"]
mod tests;
