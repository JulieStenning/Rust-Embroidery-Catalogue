use crate::config::BootstrapConfig;
use crate::services::{
    folder_picker, image_generation, scanning, stitch_identifier, tagging, validation,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
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
static BULK_IMPORT_DB_POOL: OnceLock<SqlitePool> = OnceLock::new();
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
    #[serde(default)]
    pub image_preference_override: Option<String>,
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
    let _ = BULK_IMPORT_DB_POOL.set(pool);
}

pub fn initialize_bulk_import_app_handle(app_handle: tauri::AppHandle) {
    let _ = BULK_IMPORT_APP_HANDLE.set(app_handle);
}

fn get_bulk_import_db_pool() -> Option<SqlitePool> {
    BULK_IMPORT_DB_POOL.get().cloned()
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

fn normalize_import_image_preference_override(raw_value: Option<&str>) -> Option<bool> {
    match raw_value
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("2d") => Some(false),
        Some("3d") => Some(true),
        _ => None,
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
/// canonical stored filepath (e.g. `/MachineEmbroideryDesigns/sub/design.pes`).
/// Now uses strict canonical-base-prefix validation instead of substring matching,
/// so unrelated paths containing "machineembroiderydesigns" in their name do not
/// bypass the copy guard.
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

    if normalized_full.to_ascii_lowercase() == designs_base.to_string_lossy().replace('\\', "/").to_ascii_lowercase() {
        return Ok("/MachineEmbroideryDesigns".to_string());
    }

    let base_prefix = format!(
        "{}/",
        designs_base
            .to_string_lossy()
            .replace('\\', "/")
            .trim_end_matches('/')
    );
    let suffix = &normalized_full[(base_prefix.len())..];
    Ok(format!(
        "/MachineEmbroideryDesigns/{}",
        suffix.trim_start_matches('/')
    ))
}

/// Pure helper: computes the prospective stored filepath for a file given its
/// absolute path and the selected import root_paths, without touching the filesystem.
/// This is the single source of truth for path mapping used by both preview dedup
/// and confirm-time import.
///
/// Path construction rules:
/// 1. If the file is already under the designs base, its stored path is derived directly.
/// 2. Otherwise, find the longest matching root_path, extract the root folder leaf
///    (the last component of the root), and build the destination as
///    `/MachineEmbroideryDesigns/{root_leaf}/{relative_subpath}`.
/// 3. Drive-letter-only roots (e.g. `C:/`) have no natural leaf; files are placed
///    directly under `/MachineEmbroideryDesigns/` using the path relative to the drive root.
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
            let is_drive_root = root.len() <= 3
                && root.ends_with(':')
                || (root.len() <= 4 && root.ends_with(":/"));

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

    Ok(format!(
        "/MachineEmbroideryDesigns/{}",
        rel_path.trim_start_matches('/')
    ))
}

/// Compute BLAKE3 hash of a file. Returns hex-encoded string.
fn compute_file_hash_blake3(file_path: &Path) -> Result<String, String> {
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
fn compute_file_size(file_path: &Path) -> Result<i64, String> {
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

    // Use the single-source-of-truth path helper to compute the prospective
    // stored relative path (e.g. "testdata/Bean.pes").
    let prospective_stored =
        compute_prospective_stored_filepath(full_path, root_paths)?;
    // prospective_stored looks like "/MachineEmbroideryDesigns/testdata/Bean.pes"
    let rel_path = prospective_stored
        .strip_prefix("/MachineEmbroideryDesigns/")
        .unwrap_or(&prospective_stored)
        .trim_start_matches('/');

    let dest = designs_base.join(rel_path);

    // Check if destination already exists
    if dest.exists() {
        // Compute hash of existing destination file
        let dest_size = compute_file_size(&dest).unwrap_or(0);
        let dest_hash = compute_file_hash_blake3(&dest).unwrap_or_default();

        if dest_size == source_size && dest_hash == source_hash {
            // Content matches — reuse existing stored filepath, no copy needed
            println!(
                "Import file '{}' content-identical to existing '{}' — reusing stored path",
                source.display(),
                dest.display()
            );
            return Ok(prospective_stored);
        }

        // Content differs — auto-rename the new file
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

        println!(
            "Import collision: '{}' exists with different content — auto-renaming to '{}'",
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

    println!(
        "Copied external file '{}' to managed directory '{}'",
        source.display(),
        dest.display()
    );

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
    image_preference_override: Option<&str>,
) -> Result<usize, String> {
    if !confirm_wire.wire.create_on_import {
        return Ok(0);
    }

    let resolved_assignments = resolve_bulk_import_assignments(confirm_wire);
    let preview_3d = match normalize_import_image_preference_override(image_preference_override) {
        Some(value) => value,
        None => load_import_preview_3d_if_initialized(pool).await?,
    };
    let preview_3d_profile = load_import_preview_3d_profile_if_initialized(pool).await?;
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

    // Timing accumulators
    let import_start = Instant::now();
    let mut total_dedup_check_ms = 0u128;
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
                println!("Failed to emit bulk import progress event: {error}");
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
    // For each chunk: one Python batch call (pyembroidery imported once per chunk),
    // then DB inserts, then commit. Cancellation takes effect between chunks.
    let all_files = confirm_wire.wire.selected_files.clone();
    let mut chunk_start = 0usize;

    while chunk_start < total_count {
        if BULK_IMPORT_STOP_REQUESTED.load(Ordering::SeqCst) {
            stopped = true;
            break;
        }

        let chunk_end = (chunk_start + commit_batch_size).min(total_count);
        let chunk = &all_files[chunk_start..chunk_end];

        // Pre-generate images for Python-only files in this chunk using a single subprocess.
        let python_requests: Vec<image_generation::ImageGenerationRequest> = chunk
            .iter()
            .filter(|fp| image_generation::needs_python_backend(fp))
            .map(|fp| image_generation::ImageGenerationRequest {
                file_path: fp.clone(),
                preview_3d,
                preview_3d_profile: Some(preview_3d_profile.clone()),
            })
            .collect();

        let mut chunk_image_cache: HashMap<String, image_generation::ImageGenerationResult> =
            if !python_requests.is_empty() {
                emit_progress(
                    "generating_images",
                    processed_count,
                    persisted_design_count,
                    committed_design_count,
                    None,
                );
                let t_batch = Instant::now();

                let cache = image_generation::generate_previews_via_python_batch(&python_requests);
                println!(
                    "[TIMING] Python batch done: {}ms for {} file(s)",
                    t_batch.elapsed().as_millis(),
                    python_requests.len()
                );
                cache
            } else {
                HashMap::new()
            };

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

            let t_dedup = Instant::now();
            let existing_design_id: Option<i64> =
                sqlx::query_scalar("SELECT id FROM designs WHERE filepath = ? LIMIT 1")
                    .bind(&stored_filepath)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            total_dedup_check_ms += t_dedup.elapsed().as_millis();

            if existing_design_id.is_some() {
                processed_count += 1;
                emit_progress(
                    "processed",
                    processed_count,
                    persisted_design_count,
                    committed_design_count,
                    Some(file_path),
                );
                continue;
            }

            let (designer_id, source_id) =
                resolve_assignment_for_file(file_path, confirm_wire, &resolved_assignments);

            let filename = Path::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(file_path)
                .to_string();

            let t_image = Instant::now();
            let image_result = chunk_image_cache.remove(file_path).unwrap_or_else(|| {
                image_generation::generate_preview(&image_generation::ImageGenerationRequest {
                    file_path: file_path.clone(),
                    preview_3d,
                    preview_3d_profile: Some(preview_3d_profile.clone()),
                })
            });
            let image_gen_ms = t_image.elapsed().as_millis();
            total_image_gen_ms += image_gen_ms;
            println!(
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
                println!(
                    "Image generation adapter error for '{}': {}",
                    file_path, error
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
            let file_hash_blake3: Option<String> =
                compute_file_hash_blake3(&stored_path).ok();

            // Confirm-time content-based dedup guard (Phase 3.4):
            // If the stored filepath already exists (double-check before insert),
            // or if another row already has the same hash+size, skip.
            let t_confirm_dedup = Instant::now();
            if file_size_bytes.is_some() && file_hash_blake3.is_some() {
                let existing_by_fingerprint: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM designs WHERE file_size_bytes = ? AND file_hash_blake3 = ? LIMIT 1",
                )
                .bind(file_size_bytes)
                .bind(file_hash_blake3.as_ref().unwrap())
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
                if existing_by_fingerprint.is_some() {
                    total_dedup_check_ms += t_confirm_dedup.elapsed().as_millis();
                    println!(
                        "Confirm dedup: '{}' content-match with existing row — skipping",
                        stored_filepath
                    );
                    processed_count += 1;
                    emit_progress(
                        "processed",
                        processed_count,
                        persisted_design_count,
                        committed_design_count,
                        Some(file_path),
                    );
                    continue;
                }
            } else {
                total_dedup_check_ms += t_confirm_dedup.elapsed().as_millis();
            }

            let t_insert = Instant::now();
            let insert_result = sqlx::query(
                "INSERT INTO designs (filename, filepath, date_added, designer_id, source_id, hoop_id, image_data, image_type, width_mm, height_mm, stitch_count, color_count, color_change_count, is_stitched, tags_checked, file_size_bytes, file_hash_blake3) VALUES (?, ?, DATE('now'), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?, ?)",
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
            let t_tag = Instant::now();
            let matched_descriptions = tagging::suggest_tier1_descriptions(
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

            if !matched_descriptions.is_empty() || !stitching_tag_ids.is_empty() {
                sqlx::query("UPDATE designs SET tagging_tier = 1 WHERE id = ?")
                    .bind(design_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            }
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
            println!(
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
    println!(
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

fn persist_bulk_import_confirm_if_initialized(
    confirm_wire: &BulkImportConfirmWire,
    context_token: Option<&str>,
    image_preference_override: Option<&str>,
) -> Result<usize, String> {
    match get_bulk_import_db_pool() {
        Some(pool) => tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            confirm_wire,
            context_token,
            image_preference_override,
        )),
        None => {
            println!("Bulk import DB pool not initialized; skipping persistence step.");
            Ok(0)
        }
    }
}

fn clear_bulk_import_context_store_internal(reason: &str) -> BulkImportContextStoreResetResult {
    let mut store = bulk_import_context_store().lock().unwrap();
    let cleared_context_count = store.len();
    store.clear();

    let reset_at_millis = current_timestamp_millis() as u64;
    BULK_IMPORT_CONTEXT_LAST_RESET_AT_MILLIS.store(reset_at_millis, Ordering::Relaxed);
    let reset_count = BULK_IMPORT_CONTEXT_RESET_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;

    BulkImportContextStoreResetResult {
        cleared_context_count,
        active_context_count: store.len(),
        reset_count,
        reset_at_millis,
        reason: reason.to_string(),
    }
}

pub fn reset_bulk_import_context_store_for_startup() -> BulkImportContextStoreResetResult {
    clear_bulk_import_context_store_internal("startup")
}

pub fn store_bulk_import_context(confirm_wire: BulkImportConfirmWire) -> String {
    let (token, sequence) = next_bulk_import_context_token();
    let mut store = bulk_import_context_store().lock().unwrap();
    prune_bulk_import_context_store(&mut store);
    let mut stored_wire = canonicalize_bulk_import_confirm_wire(confirm_wire);
    stored_wire.context_token = Some(token.clone());
    store.insert(
        token.clone(),
        StoredBulkImportContext {
            confirm_wire: stored_wire,
            created_at_millis: current_timestamp_millis(),
            sequence,
        },
    );
    token
}

pub fn take_bulk_import_context(token: &str) -> Option<BulkImportConfirmWire> {
    let mut store = bulk_import_context_store().lock().unwrap();
    prune_bulk_import_context_store(&mut store);
    store.remove(token).map(|context| context.confirm_wire)
}

pub fn get_bulk_import_context(token: &str) -> Option<BulkImportConfirmWire> {
    let mut store = bulk_import_context_store().lock().unwrap();
    prune_bulk_import_context_store(&mut store);
    store.get(token).map(|context| context.confirm_wire.clone())
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
    let mut store = bulk_import_context_store().lock().unwrap();
    prune_bulk_import_context_store(&mut store);
    let last_reset_at_millis = BULK_IMPORT_CONTEXT_LAST_RESET_AT_MILLIS.load(Ordering::Relaxed);

    Ok(BulkImportContextStoreSummary {
        active_context_count: store.len(),
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
    let result = folder_picker::browse_folder(start_dir.as_deref(), allow_multi)?;

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

            let image_preference_override = request.image_preference_override.clone();
            let confirm_result = tauri::async_runtime::spawn_blocking(move || {
                do_confirm_bulk_import_wire_internal(
                    context_token,
                    image_preference_override.as_deref(),
                )
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
    do_confirm_bulk_import_wire_internal(context_token, None)
}

fn do_confirm_bulk_import_wire_internal(
    context_token: String,
    image_preference_override: Option<&str>,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let confirm_wire = take_bulk_import_context(&context_token)
        .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

    let persisted_design_count = persist_bulk_import_confirm_if_initialized(
        &confirm_wire,
        Some(&context_token),
        image_preference_override,
    )?;
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
        None,
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
/// checking both by prospective stored filepath AND by content fingerprint
/// (BLAKE3 hash + file size).
///
/// For every scanned file we compute its prospective stored filepath using the
/// single-source-of-truth helper so external-source files get the same mapping
/// as they will during confirm. Then we filter against:
///   1. stored filepath match (normalized), OR
///   2. (file_size_bytes, file_hash_blake3) match — with lazy hashing for
///      files that pass the filepath check first.
async fn filter_existing_scanned_files(
    pool: &SqlitePool,
    scanned_files: Vec<scanning::ScannedFile>,
    root_paths: &[String],
) -> Result<Vec<scanning::ScannedFile>, String> {
    if scanned_files.is_empty() {
        return Ok(scanned_files);
    }

    // Load existing path set
    let existing_paths = sqlx::query_scalar::<_, String>("SELECT filepath FROM designs")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let existing_path_set: HashSet<String> = existing_paths
        .into_iter()
        .map(|path| normalize_path_for_match(&path))
        .collect();

    // Load fingerprint pairs for content-based dedup
    let fingerprint_rows: Vec<(Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT file_size_bytes, file_hash_blake3 FROM designs WHERE file_size_bytes IS NOT NULL AND file_hash_blake3 IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Build a set of (size, hash) pairs for O(1) lookup
    let fingerprint_set: HashSet<(i64, String)> = fingerprint_rows
        .into_iter()
        .filter_map(|(size_opt, hash_opt)| {
            match (size_opt, hash_opt) {
                (Some(size), Some(hash)) => Some((size, hash.to_ascii_lowercase())),
                _ => None,
            }
        })
        .collect();

    let mut result: Vec<scanning::ScannedFile> = Vec::with_capacity(scanned_files.len());
    let mut excluded_by_path: usize = 0;
    let mut excluded_by_hash: usize = 0;

    for file in scanned_files {
        // Compute prospective stored filepath using the single-source-of-truth helper
        let prospective_path = compute_prospective_stored_filepath(&file.full_path, root_paths)
            .unwrap_or_else(|_| format!("/MachineEmbroideryDesigns/{}", file.full_path));

        let normalized_prospective = normalize_path_for_match(&prospective_path);

        // Check 1: filepath match
        if existing_path_set.contains(&normalized_prospective) {
            excluded_by_path += 1;
            continue;
        }

        // Check 2: content fingerprint match (lazy hash — only if not excluded by path)
        let source_path = Path::new(&file.full_path);
        if source_path.exists() && !fingerprint_set.is_empty() {
            if let (Ok(size), Ok(hash)) = (
                compute_file_size(source_path),
                compute_file_hash_blake3(source_path),
            ) {
                if fingerprint_set.contains(&(size, hash.to_ascii_lowercase())) {
                    excluded_by_hash += 1;
                    continue;
                }
            }
        }

        result.push(file);
    }

    if excluded_by_path > 0 || excluded_by_hash > 0 {
        println!(
            "Preview dedup: excluded_by_path={} excluded_by_hash={} imported={}",
            excluded_by_path,
            excluded_by_hash,
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
    for root_path in &wire.root_paths {
        let scan_input = scanning::ScanInput {
            root_path: root_path.clone(),
        };
        let scan_result = scanning::scan(&scan_input);
        scanned_files.extend(scan_result.files);
    }

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
    })
}

pub fn preview_bulk_import_wire(wire: BulkImportWire) -> Result<BulkImportPreview, String> {
    let pool = get_bulk_import_db_pool();
    preview_bulk_import_wire_with_pool(wire, pool.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::fs;

    async fn import_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create test sqlite pool");

        sqlx::query(
            r#"
            CREATE TABLE settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                description TEXT
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to create settings table");

        sqlx::query(
            r#"
            CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT NOT NULL UNIQUE,
                tag_group TEXT
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to create tags table");

        sqlx::query(
            r#"
            CREATE TABLE design_tags (
                design_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (design_id, tag_id)
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to create design_tags table");

        sqlx::query(
            r#"
            CREATE TABLE designs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                filepath TEXT NOT NULL,
                date_added TEXT,
                designer_id INTEGER,
                source_id INTEGER,
                hoop_id INTEGER,
                image_data BLOB,
                image_type TEXT,
                width_mm REAL,
                height_mm REAL,
                stitch_count INTEGER,
                color_count INTEGER,
                color_change_count INTEGER,
                is_stitched INTEGER NOT NULL DEFAULT 0,
                tags_checked INTEGER NOT NULL DEFAULT 0,
                tagging_tier INTEGER,
                file_size_bytes INTEGER,
                file_hash_blake3 TEXT
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to create designs table");

        sqlx::query(
            r#"
            CREATE TABLE hoops (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                max_width_mm REAL NOT NULL,
                max_height_mm REAL NOT NULL
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to create hoops table");

        sqlx::query(
            r#"
            CREATE TABLE designers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to create designers table");

        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to create sources table");

        sqlx::query("INSERT INTO settings (key, value, description) VALUES ('image.preference', '2d', 'test preference')")
            .execute(&pool)
            .await
            .expect("failed to seed image preference");

        sqlx::query("INSERT INTO tags (description, tag_group) VALUES ('Alphabets', 'image'), ('Flowers', 'image'), ('Monogram', 'image'), ('Line Outline', 'stitching')")
            .execute(&pool)
            .await
            .expect("failed to seed tags");

        pool
    }

    #[test]
    fn bulk_import_wire_round_trips_through_json() {
        let wire = BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: None,
            per_folder_assignments: vec![FolderAssignmentWire {
                folder_path: "C:/imports/folder-a".to_string(),
                designer_id: None,
                source_id: Some(9),
                inferred_designer_id: Some(11),
                inferred_source_id: None,
            }],
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: true,
        };

        let encoded = serde_json::to_string(&wire).expect("wire should serialize");
        let decoded: BulkImportWire =
            serde_json::from_str(&encoded).expect("wire should deserialize");

        assert_eq!(decoded.root_paths.len(), 1);
        assert_eq!(decoded.per_folder_assignments.len(), 1);
        assert_eq!(decoded.selected_files.len(), 1);
        assert!(decoded.create_on_import);
    }

    #[test]
    fn persist_bulk_import_confirm_wire_writes_image_fields_in_native_mode() {
        let fixture = Path::new("tests").join("testdata").join("Bean.pes");
        assert!(fixture.exists(), "expected Bean.pes fixture to exist");

        let previous_backend = std::env::var("IMPORT_IMAGE_BACKEND").ok();
        std::env::set_var("IMPORT_IMAGE_BACKEND", "native");

        let pool = tauri::async_runtime::block_on(import_test_pool());
        let confirm_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["tests/testdata".to_string()],
                global_designer_id: None,
                global_source_id: None,
                per_folder_assignments: Vec::new(),
                selected_files: vec![fixture.to_string_lossy().to_string()],
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: true,
        };

        let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            &confirm_wire,
            None,
            None,
        ))
        .expect("persist should succeed");
        assert_eq!(persisted, 1);

        // The file is now stored under MachineEmbroideryDesigns/testdata/Bean.pes
        let stored_filepath = "/MachineEmbroideryDesigns/testdata/Bean.pes";
        let row = tauri::async_runtime::block_on(async {
            sqlx::query_as::<_, (Option<Vec<u8>>, Option<String>, Option<f64>, Option<f64>, Option<i64>, Option<i64>, Option<i64>)>(
                "SELECT image_data, image_type, width_mm, height_mm, stitch_count, color_count, color_change_count FROM designs WHERE filepath = ? LIMIT 1"
            )
            .bind(stored_filepath)
            .fetch_one(&pool)
            .await
        })
        .expect("expected persisted design row");

        assert!(row.0.map(|bytes| !bytes.is_empty()).unwrap_or(false));
        assert_eq!(row.1.as_deref(), Some("2d"));
        assert!(row.2.unwrap_or_default() > 0.0);
        assert!(row.3.unwrap_or_default() > 0.0);
        assert!(row.4.unwrap_or_default() > 0);
        assert!(row.5.unwrap_or_default() > 0);
        assert!(row.6.unwrap_or_default() >= 0);

        if let Some(value) = previous_backend {
            std::env::set_var("IMPORT_IMAGE_BACKEND", value);
        } else {
            std::env::remove_var("IMPORT_IMAGE_BACKEND");
        }
    }

    #[test]
    fn persist_bulk_import_confirm_wire_auto_backend_3d_pref_falls_back_safely_without_python() {
        let fixture = Path::new("tests").join("testdata").join("Bean.pes");
        assert!(fixture.exists(), "expected Bean.pes fixture to exist");

        let previous_backend = std::env::var("IMPORT_IMAGE_BACKEND").ok();
        let previous_python = std::env::var("RUST_EMBROIDERY_PYTHON").ok();

        std::env::set_var("IMPORT_IMAGE_BACKEND", "auto");
        // Intentionally point to a missing executable so python path fails and auto must use native fallback.
        std::env::set_var(
            "RUST_EMBROIDERY_PYTHON",
            "__missing_python_for_auto_fallback_test__",
        );

        let pool = tauri::async_runtime::block_on(import_test_pool());
        tauri::async_runtime::block_on(async {
            sqlx::query("UPDATE settings SET value = '3d' WHERE key = 'image.preference'")
                .execute(&pool)
                .await
        })
        .expect("failed to update image preference to 3d");

        let confirm_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["tests/testdata".to_string()],
                global_designer_id: None,
                global_source_id: None,
                per_folder_assignments: Vec::new(),
                selected_files: vec![fixture.to_string_lossy().to_string()],
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: true,
        };

        let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            &confirm_wire,
            None,
            None,
        ))
        .expect("persist should succeed even when python path is unavailable");
        assert_eq!(persisted, 1);

        // The file is now stored under MachineEmbroideryDesigns/testdata/Bean.pes
        let stored_filepath = "/MachineEmbroideryDesigns/testdata/Bean.pes";
        let row = tauri::async_runtime::block_on(async {
            sqlx::query_as::<_, (Option<Vec<u8>>, Option<String>, Option<f64>, Option<f64>)>(
                "SELECT image_data, image_type, width_mm, height_mm FROM designs WHERE filepath = ? LIMIT 1"
            )
            .bind(stored_filepath)
            .fetch_one(&pool)
            .await
        })
        .expect("expected persisted design row");

        assert!(row.0.map(|bytes| !bytes.is_empty()).unwrap_or(false));
        assert_eq!(row.1.as_deref(), Some("3d"));
        assert!(row.2.unwrap_or_default() > 0.0);
        assert!(row.3.unwrap_or_default() > 0.0);

        if let Some(value) = previous_backend {
            std::env::set_var("IMPORT_IMAGE_BACKEND", value);
        } else {
            std::env::remove_var("IMPORT_IMAGE_BACKEND");
        }

        if let Some(value) = previous_python {
            std::env::set_var("RUST_EMBROIDERY_PYTHON", value);
        } else {
            std::env::remove_var("RUST_EMBROIDERY_PYTHON");
        }
    }

    #[test]
    fn persist_bulk_import_confirm_wire_auto_hus_uses_native_backend() {
        let fixture = Path::new("tests").join("testdata").join("Bean.hus");
        assert!(fixture.exists(), "expected Bean.hus fixture to exist");

        let previous_backend = std::env::var("IMPORT_IMAGE_BACKEND").ok();
        std::env::set_var("IMPORT_IMAGE_BACKEND", "auto");

        let generation_result =
            image_generation::generate_preview(&image_generation::ImageGenerationRequest {
                file_path: fixture.to_string_lossy().to_string(),
                preview_3d: true,
                preview_3d_profile: Some("balanced".to_string()),
            });

        assert_eq!(generation_result.backend, "native");
        let error_text = generation_result
            .error
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        assert!(
            error_text.is_empty(),
            "auto mode should generate native HUS previews without adapter errors"
        );

        let pool = tauri::async_runtime::block_on(import_test_pool());
        let confirm_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["tests/testdata".to_string()],
                global_designer_id: None,
                global_source_id: None,
                per_folder_assignments: Vec::new(),
                selected_files: vec![fixture.to_string_lossy().to_string()],
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: true,
        };

        let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            &confirm_wire,
            None,
            None,
        ))
        .expect("persist should succeed for .hus even when preview generation fails");
        assert_eq!(persisted, 1);

        // The file is now stored under MachineEmbroideryDesigns/testdata/Bean.hus
        let stored_filepath = "/MachineEmbroideryDesigns/testdata/Bean.hus";
        let persisted_row_id = tauri::async_runtime::block_on(async {
            sqlx::query_scalar::<_, i64>("SELECT id FROM designs WHERE filepath = ? LIMIT 1")
                .bind(stored_filepath)
                .fetch_optional(&pool)
                .await
        })
        .expect("expected design lookup to succeed");
        assert!(
            persisted_row_id.is_some(),
            "expected .hus design row to be inserted"
        );

        if let Some(value) = previous_backend {
            std::env::set_var("IMPORT_IMAGE_BACKEND", value);
        } else {
            std::env::remove_var("IMPORT_IMAGE_BACKEND");
        }
    }

    #[test]
    fn normalize_import_commit_batch_size_defaults_to_10_and_clamps_high_values() {
        assert_eq!(normalize_import_commit_batch_size(None), 10);
        assert_eq!(normalize_import_commit_batch_size(Some("")), 10);
        assert_eq!(normalize_import_commit_batch_size(Some("abc")), 10);
        assert_eq!(normalize_import_commit_batch_size(Some("0")), 10);
        assert_eq!(normalize_import_commit_batch_size(Some("10")), 10);
        assert_eq!(
            normalize_import_commit_batch_size(Some("1000000")),
            MAX_IMPORT_COMMIT_BATCH_SIZE
        );
    }

    #[test]
    fn normalize_import_image_preference_override_accepts_only_2d_or_3d() {
        assert_eq!(normalize_import_image_preference_override(None), None);
        assert_eq!(normalize_import_image_preference_override(Some("")), None);
        assert_eq!(
            normalize_import_image_preference_override(Some("2d")),
            Some(false)
        );
        assert_eq!(
            normalize_import_image_preference_override(Some(" 3D ")),
            Some(true)
        );
        assert_eq!(
            normalize_import_image_preference_override(Some("unexpected")),
            None
        );
    }

    #[test]
    fn load_import_commit_batch_size_reads_setting_override() {
        let pool = tauri::async_runtime::block_on(import_test_pool());

        let default_batch_size =
            tauri::async_runtime::block_on(load_import_commit_batch_size(&pool))
                .expect("default batch size should load");
        assert_eq!(default_batch_size, 10);

        tauri::async_runtime::block_on(async {
            sqlx::query(
                "INSERT INTO settings (key, value, description) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .bind(KEY_IMPORT_COMMIT_BATCH_SIZE)
            .bind("25")
            .bind("test commit batch size")
            .execute(&pool)
            .await
        })
        .expect("failed to set import commit batch size");

        let configured_batch_size =
            tauri::async_runtime::block_on(load_import_commit_batch_size(&pool))
                .expect("configured batch size should load");
        assert_eq!(configured_batch_size, 25);
    }

    #[test]
    fn persist_bulk_import_confirm_wire_assigns_tier1_keyword_tags() {
        let fixture = Path::new("tests").join("testdata").join("Bean.pes");
        assert!(fixture.exists(), "expected Bean.pes fixture to exist");

        let pool = tauri::async_runtime::block_on(import_test_pool());
        let confirm_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["tests/testdata".to_string()],
                global_designer_id: None,
                global_source_id: None,
                per_folder_assignments: Vec::new(),
                selected_files: vec![fixture.to_string_lossy().to_string()],
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: true,
        };

        let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            &confirm_wire,
            None,
            None,
        ))
        .expect("persist should succeed");
        assert_eq!(persisted, 1);

        let stored_filepath = "/MachineEmbroideryDesigns/testdata/Bean.pes";

        let assigned_tags = tauri::async_runtime::block_on(async {
            sqlx::query_as::<_, (String,)>(
                r#"
                SELECT t.description
                FROM design_tags dt
                JOIN tags t ON t.id = dt.tag_id
                JOIN designs d ON d.id = dt.design_id
                WHERE d.filepath = ?
                ORDER BY t.description ASC
                "#,
            )
            .bind(stored_filepath)
            .fetch_all(&pool)
            .await
        })
        .expect("failed to query assigned tags");

        assert!(
            !assigned_tags.is_empty(),
            "expected at least one tag assignment for imported design"
        );
    }

    #[test]
    fn persist_bulk_import_confirm_wire_assigns_stitching_tags() {
        let fixture = Path::new("tests").join("testdata").join("Bean.pes");
        assert!(fixture.exists(), "expected Bean.pes fixture to exist");

        let pool = tauri::async_runtime::block_on(import_test_pool());
        let confirm_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["tests/testdata".to_string()],
                global_designer_id: None,
                global_source_id: None,
                per_folder_assignments: Vec::new(),
                selected_files: vec![fixture.to_string_lossy().to_string()],
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: true,
        };

        let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            &confirm_wire,
            None,
            None,
        ))
        .expect("persist should succeed");
        assert_eq!(persisted, 1);

        let stored_filepath = "/MachineEmbroideryDesigns/testdata/Bean.pes";

        let stitching_tags = tauri::async_runtime::block_on(async {
            sqlx::query_as::<_, (String,)>(
                r#"
                SELECT t.description
                FROM design_tags dt
                JOIN tags t ON t.id = dt.tag_id
                JOIN designs d ON d.id = dt.design_id
                WHERE d.filepath = ?
                  AND lower(COALESCE(t.tag_group, '')) = 'stitching'
                ORDER BY t.description ASC
                "#,
            )
            .bind(stored_filepath)
            .fetch_all(&pool)
            .await
        })
        .expect("failed to query stitching tags");

        assert!(
            !stitching_tags.is_empty(),
            "expected at least one stitching tag assignment"
        );
    }

    #[test]
    fn persist_bulk_import_confirm_wire_honors_image_preference_override_for_session() {
        let fixture = Path::new("tests").join("testdata").join("Bean.pes");
        assert!(fixture.exists(), "expected Bean.pes fixture to exist");

        let previous_backend = std::env::var("IMPORT_IMAGE_BACKEND").ok();
        std::env::set_var("IMPORT_IMAGE_BACKEND", "native");

        let pool = tauri::async_runtime::block_on(import_test_pool());
        let confirm_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["tests/testdata".to_string()],
                global_designer_id: None,
                global_source_id: None,
                per_folder_assignments: Vec::new(),
                selected_files: vec![fixture.to_string_lossy().to_string()],
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: true,
        };

        let persisted = tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(
            &pool,
            &confirm_wire,
            None,
            Some("3d"),
        ))
        .expect("persist should succeed with explicit session override");
        assert_eq!(persisted, 1);

        let stored_filepath = "/MachineEmbroideryDesigns/testdata/Bean.pes";

        let image_type = tauri::async_runtime::block_on(async {
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT image_type FROM designs WHERE filepath = ? LIMIT 1",
            )
            .bind(stored_filepath)
            .fetch_one(&pool)
            .await
        })
        .expect("expected persisted design row");

        assert_eq!(image_type.as_deref(), Some("3d"));

        if let Some(value) = previous_backend {
            std::env::set_var("IMPORT_IMAGE_BACKEND", value);
        } else {
            std::env::remove_var("IMPORT_IMAGE_BACKEND");
        }
    }

    #[test]
    fn bulk_import_confirm_wire_round_trips_through_json() {
        let confirm_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: vec![FolderAssignmentWire {
                    folder_path: "C:/imports/folder-a".to_string(),
                    designer_id: Some(10),
                    source_id: None,
                    inferred_designer_id: None,
                    inferred_source_id: Some(12),
                }],
                selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
                create_on_import: true,
            },
            context_token: Some("token-123".to_string()),
            canonical_confirm: true,
        };

        let encoded = serde_json::to_string(&confirm_wire).expect("confirm wire should serialize");
        let decoded: BulkImportConfirmWire =
            serde_json::from_str(&encoded).expect("confirm wire should deserialize");

        assert_eq!(decoded.context_token.as_deref(), Some("token-123"));
        assert!(decoded.canonical_confirm);
        assert_eq!(decoded.wire.root_paths.len(), 1);
    }

    #[test]
    fn assignment_field_resolution_prefers_explicit_global_inferred_blank() {
        let explicit = resolve_assignment_field(Some(1), Some(2), Some(3));
        assert_eq!(explicit.value, Some(1));
        assert_eq!(
            explicit.source,
            AssignmentFieldSourceWire::ExplicitPerFolder
        );

        let global = resolve_assignment_field(None, Some(2), Some(3));
        assert_eq!(global.value, Some(2));
        assert_eq!(global.source, AssignmentFieldSourceWire::Global);

        let inferred = resolve_assignment_field(None, None, Some(3));
        assert_eq!(inferred.value, Some(3));
        assert_eq!(inferred.source, AssignmentFieldSourceWire::Inferred);

        let blank = resolve_assignment_field(None, None, None);
        assert_eq!(blank.value, None);
        assert_eq!(blank.source, AssignmentFieldSourceWire::Blank);
    }

    #[test]
    fn suggest_reference_id_from_path_matches_compact_names() {
        let items = vec![
            (1, "www.UrbanThreads.com".to_string()),
            (2, "Another Source".to_string()),
        ];

        let matched = suggest_reference_id_from_path(
            "D:/My Software Development/Rust-Embroidery-Catalogue/data/MachineEmbroideryDesigns/Urban Threads",
            &items,
        );

        assert_eq!(matched, Some(1));
    }

    #[test]
    fn folder_assignment_resolution_uses_wire_defaults_and_inferred_values() {
        let confirm_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: vec![
                    FolderAssignmentWire {
                        folder_path: "C:/imports/folder-a".to_string(),
                        designer_id: Some(10),
                        source_id: None,
                        inferred_designer_id: Some(11),
                        inferred_source_id: Some(12),
                    },
                    FolderAssignmentWire {
                        folder_path: "C:/imports/folder-b".to_string(),
                        designer_id: None,
                        source_id: None,
                        inferred_designer_id: Some(13),
                        inferred_source_id: None,
                    },
                ],
                selected_files: vec![],
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: true,
        };

        let resolved = resolve_bulk_import_assignments(&confirm_wire);
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].designer_id.value, Some(10));
        assert_eq!(
            resolved[0].designer_id.source,
            AssignmentFieldSourceWire::ExplicitPerFolder
        );
        assert_eq!(resolved[0].source_id.value, Some(8));
        assert_eq!(
            resolved[0].source_id.source,
            AssignmentFieldSourceWire::Global
        );

        assert_eq!(resolved[1].designer_id.value, Some(7));
        assert_eq!(
            resolved[1].designer_id.source,
            AssignmentFieldSourceWire::Global
        );
        assert_eq!(resolved[1].source_id.value, Some(8));
        assert_eq!(
            resolved[1].source_id.source,
            AssignmentFieldSourceWire::Global
        );
    }

    #[test]
    fn preview_bulk_import_wire_returns_resolved_assignments() {
        let preview = preview_bulk_import_wire(BulkImportWire {
            root_paths: vec!["C:/imports".to_string()],
            global_designer_id: Some(7),
            global_source_id: Some(8),
            per_folder_assignments: vec![FolderAssignmentWire {
                folder_path: "C:/imports/folder-a".to_string(),
                designer_id: None,
                source_id: Some(9),
                inferred_designer_id: Some(11),
                inferred_source_id: Some(12),
            }],
            selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
            create_on_import: true,
        })
        .expect("preview should resolve");

        assert_eq!(preview.resolved_assignments.len(), 1);
        assert_eq!(preview.resolved_assignments[0].designer_id.value, Some(7));
        assert_eq!(preview.resolved_assignments[0].source_id.value, Some(9));
    }

    #[test]
    fn preview_bulk_import_wire_excludes_already_catalogued_files() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("rec-import-preview-existing-{stamp}"));
        fs::create_dir_all(&root).expect("temp root should be created");

        let file_path = root.join("existing-design.pes");
        fs::write(&file_path, b"dummy").expect("temp pes should be written");
        let file_path_text = file_path.to_string_lossy().to_string();

        let pool = tauri::async_runtime::block_on(import_test_pool());
        tauri::async_runtime::block_on(async {
            sqlx::query(
                "INSERT INTO designs (filename, filepath, date_added, is_stitched, tags_checked) VALUES (?, ?, DATE('now'), 0, 0)",
            )
            .bind("existing-design.pes")
            .bind(&file_path_text)
            .execute(&pool)
            .await
        })
        .expect("seeded existing design should insert");

        let preview = preview_bulk_import_wire_with_pool(
            BulkImportWire {
                root_paths: vec![root.to_string_lossy().to_string()],
                global_designer_id: None,
                global_source_id: None,
                per_folder_assignments: Vec::new(),
                selected_files: Vec::new(),
                create_on_import: true,
            },
            Some(&pool),
        )
        .expect("preview should succeed");

        assert_eq!(preview.discovered_count, 0);
        assert!(preview.scanned_files.is_empty());

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn preview_bulk_import_wire_infers_assignments_from_folder_path_with_pool() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("rec-import-preview-infer-{stamp}"));
        let inferred_folder = root.join("Acme Designs").join("Magazine Source");
        fs::create_dir_all(&inferred_folder).expect("temp inferred folder should be created");

        let file_path = inferred_folder.join("sample-design.pes");
        fs::write(&file_path, b"dummy").expect("temp pes should be written");

        let pool = tauri::async_runtime::block_on(import_test_pool());
        tauri::async_runtime::block_on(async {
            sqlx::query("INSERT INTO designers (id, name) VALUES (1, 'Acme')")
                .execute(&pool)
                .await?;
            sqlx::query("INSERT INTO sources (id, name) VALUES (1, 'Magazine Source')")
                .execute(&pool)
                .await?;
            Ok::<(), sqlx::Error>(())
        })
        .expect("seeded designer/source should insert");

        let preview = preview_bulk_import_wire_with_pool(
            BulkImportWire {
                root_paths: vec![root.to_string_lossy().to_string()],
                global_designer_id: None,
                global_source_id: None,
                per_folder_assignments: Vec::new(),
                selected_files: Vec::new(),
                create_on_import: true,
            },
            Some(&pool),
        )
        .expect("preview should succeed");

        assert_eq!(preview.discovered_count, 1);
        assert_eq!(preview.resolved_assignments.len(), 1);
        assert_eq!(preview.resolved_assignments[0].designer_id.value, Some(1));
        assert_eq!(
            preview.resolved_assignments[0].designer_id.source,
            AssignmentFieldSourceWire::Inferred
        );
        assert_eq!(preview.resolved_assignments[0].source_id.value, Some(1));
        assert_eq!(
            preview.resolved_assignments[0].source_id.source,
            AssignmentFieldSourceWire::Inferred
        );

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn confirm_execution_result_reflects_readiness_and_resolution() {
        let result = execute_bulk_import_confirm_wire(BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: vec![FolderAssignmentWire {
                    folder_path: "C:/imports/folder-a".to_string(),
                    designer_id: Some(10),
                    source_id: None,
                    inferred_designer_id: Some(11),
                    inferred_source_id: Some(12),
                }],
                selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
                create_on_import: true,
            },
            context_token: Some("token-123".to_string()),
            canonical_confirm: true,
        })
        .expect("confirm execution should succeed");

        assert!(result.context_token_present);
        assert!(result.canonical_confirm);
        assert!(result.ready_for_persistence);
        assert_eq!(result.root_path_count, 1);
        assert_eq!(result.selected_file_count, 1);
        assert_eq!(result.resolved_assignments.len(), 1);
        assert_eq!(result.resolved_assignments[0].designer_id.value, Some(10));
    }

    #[test]
    fn canonical_confirm_wire_marks_ready_for_persistence() {
        let result = confirm_bulk_import_wire(BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: vec![FolderAssignmentWire {
                    folder_path: "C:/imports/folder-a".to_string(),
                    designer_id: Some(10),
                    source_id: None,
                    inferred_designer_id: None,
                    inferred_source_id: Some(12),
                }],
                selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
                create_on_import: true,
            },
            context_token: Some("token-456".to_string()),
            canonical_confirm: true,
        })
        .expect("canonical confirm should succeed");

        assert!(result.context_token_present);
        assert!(result.canonical_confirm);
        assert!(result.ready_for_persistence);
        assert_eq!(result.resolved_assignments.len(), 1);
    }

    #[test]
    fn legacy_confirm_wire_shims_into_canonical_confirm() {
        let result = confirm_bulk_import_legacy(BulkImportRequest {
            root_path: Some("C:/imports".to_string()),
            root_paths: Vec::new(),
            fallback_designer_id: Some(7),
            fallback_source_id: Some(8),
        })
        .expect("legacy confirm should succeed");

        assert!(result.canonical_confirm);
        assert!(result.ready_for_persistence);
        assert_eq!(result.root_path_count, 1);
        assert_eq!(result.selected_file_count, 0);
    }

    #[test]
    fn precheck_stores_context_and_do_confirm_consumes_it() {
        let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: vec![FolderAssignmentWire {
                    folder_path: "C:/imports/folder-a".to_string(),
                    designer_id: Some(10),
                    source_id: None,
                    inferred_designer_id: None,
                    inferred_source_id: Some(12),
                }],
                selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
                create_on_import: false,
            },
            context_token: None,
            canonical_confirm: false,
        })
        .expect("precheck should succeed");

        assert!(precheck.context_token_present);
        assert!(precheck.ready_for_confirm);
        assert_eq!(precheck.resolved_assignments.len(), 1);

        let confirm = do_confirm_bulk_import_wire(precheck.context_token)
            .expect("do-confirm should consume stored token");

        assert!(confirm.context_token_present);
        assert!(confirm.canonical_confirm);
        assert!(confirm.ready_for_persistence);
        assert_eq!(confirm.resolved_assignments.len(), 1);
    }

    #[test]
    fn precheck_action_review_tags_keeps_context() {
        let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: Vec::new(),
                selected_files: Vec::new(),
                create_on_import: false,
            },
            context_token: None,
            canonical_confirm: false,
        })
        .expect("precheck should succeed");

        let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
            BulkImportPrecheckActionRequest {
                context_token: precheck.context_token.clone(),
                action: BulkImportPrecheckActionWire::ReviewTags,
                confirm_skip_hoops: false,
                image_preference_override: None,
            },
        ))
        .expect("review action should succeed");

        assert!(!action_result.consumed_context);
        assert!(action_result.context_token_present);
        assert!(action_result
            .next_route
            .unwrap_or_default()
            .contains("/admin/tags/"));
        assert!(take_bulk_import_context(&precheck.context_token).is_some());
    }

    #[test]
    fn precheck_action_cancel_consumes_context() {
        let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: Vec::new(),
                selected_files: Vec::new(),
                create_on_import: false,
            },
            context_token: None,
            canonical_confirm: false,
        })
        .expect("precheck should succeed");

        let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
            BulkImportPrecheckActionRequest {
                context_token: precheck.context_token.clone(),
                action: BulkImportPrecheckActionWire::Cancel,
                confirm_skip_hoops: false,
                image_preference_override: None,
            },
        ))
        .expect("cancel action should succeed");

        assert!(action_result.consumed_context);
        assert!(!action_result.context_token_present);
        assert_eq!(action_result.next_route.as_deref(), Some("/import/"));
        assert!(take_bulk_import_context(&precheck.context_token).is_none());
    }

    #[test]
    fn precheck_action_import_now_consumes_context() {
        let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: Vec::new(),
                selected_files: vec!["C:/imports/folder-a/design.pes".to_string()],
                create_on_import: false,
            },
            context_token: None,
            canonical_confirm: false,
        })
        .expect("precheck should succeed");

        let action_result = tauri::async_runtime::block_on(precheck_bulk_import_action_wire(
            BulkImportPrecheckActionRequest {
                context_token: precheck.context_token.clone(),
                action: BulkImportPrecheckActionWire::ImportNow,
                confirm_skip_hoops: false,
                image_preference_override: None,
            },
        ))
        .expect("import-now action should succeed");

        assert!(action_result.consumed_context);
        assert!(!action_result.context_token_present);
        assert_eq!(action_result.next_route.as_deref(), Some("/designs/"));
        assert!(action_result.confirm_result.is_some());
        assert!(take_bulk_import_context(&precheck.context_token).is_none());
    }

    #[test]
    fn debug_bulk_import_context_store_reports_live_counts() {
        let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: Vec::new(),
                selected_files: Vec::new(),
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: false,
        })
        .expect("precheck should succeed");

        let summary = debug_bulk_import_context_store().expect("debug summary should succeed");

        assert!(summary.active_context_count >= 1);
        assert_eq!(summary.max_entries, BULK_IMPORT_CONTEXT_MAX_ENTRIES);
        assert_eq!(summary.ttl_seconds, BULK_IMPORT_CONTEXT_TTL.as_secs());

        let _ = take_bulk_import_context(&precheck.context_token);
    }

    #[test]
    fn reset_bulk_import_context_store_clears_entries_and_updates_metrics() {
        let precheck = precheck_bulk_import_wire(BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: Vec::new(),
                selected_files: Vec::new(),
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: false,
        })
        .expect("precheck should seed store");

        let before = debug_bulk_import_context_store().expect("summary should succeed");
        assert!(before.active_context_count >= 1);

        let reset = reset_bulk_import_context_store().expect("manual reset should succeed");
        assert!(reset.cleared_context_count >= 1);
        assert_eq!(reset.active_context_count, 0);

        let after = debug_bulk_import_context_store().expect("summary should succeed");
        assert_eq!(after.active_context_count, 0);
        assert!(after.reset_count >= 1);
        assert!(after.last_reset_at_millis.is_some());

        assert!(take_bulk_import_context(&precheck.context_token).is_none());
    }

    #[test]
    fn bulk_import_context_store_evicts_oldest_when_capacity_is_exceeded() {
        let base_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: Vec::new(),
                selected_files: Vec::new(),
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: false,
        };
        let created_at_millis = current_timestamp_millis();

        let first_token = "bulk-import-test-first".to_string();
        insert_bulk_import_context_for_test(
            first_token.clone(),
            base_wire.clone(),
            created_at_millis,
            1,
        );

        for index in 2..=(BULK_IMPORT_CONTEXT_MAX_ENTRIES as u64 + 1) {
            insert_bulk_import_context_for_test(
                format!("bulk-import-test-{index}"),
                base_wire.clone(),
                created_at_millis,
                index,
            );
        }

        assert!(take_bulk_import_context(&first_token).is_none());
        assert!(take_bulk_import_context(&format!(
            "bulk-import-test-{}",
            BULK_IMPORT_CONTEXT_MAX_ENTRIES as u64 + 1
        ))
        .is_some());
    }

    #[test]
    fn bulk_import_context_store_expires_old_entries_on_access() {
        let expired_token = "bulk-import-test-expired".to_string();
        let current_wire = BulkImportConfirmWire {
            wire: BulkImportWire {
                root_paths: vec!["C:/imports".to_string()],
                global_designer_id: Some(7),
                global_source_id: Some(8),
                per_folder_assignments: Vec::new(),
                selected_files: Vec::new(),
                create_on_import: true,
            },
            context_token: None,
            canonical_confirm: false,
        };

        let expired_created_at =
            current_timestamp_millis().saturating_sub(BULK_IMPORT_CONTEXT_TTL.as_millis() + 1);
        insert_bulk_import_context_for_test(
            expired_token.clone(),
            current_wire,
            expired_created_at,
            9999,
        );

        assert!(take_bulk_import_context(&expired_token).is_none());
    }

    // =========================================================================
    // Phase 5 - Path derivation tests (5.1)
    // =========================================================================

    /// Under AppRoot: full_path_to_stored_design_filepath should return
    /// the canonical stored path directly.
    #[test]
    fn stored_filepath_from_designs_base_subdirectory() {
        // We can't change DATABASE_URL at runtime, but we can test the
        // computation against a hypothetical base by constructing a path
        // and checking it is NOT treated as in-library when it clearly isn't.
        let result =
            full_path_to_stored_design_filepath("C:/SomeRandomPath/not-a-design.pes");
        assert!(result.is_err(), "unrelated path must not be in-library");
    }

    /// The old substring-based in-library detection is gone: paths containing
    /// "machineembroiderydesigns" as a substring but not actually under the
    /// canonical designs base must now be treated as external (not in-library).
    #[test]
    fn unrelated_path_containing_sentinel_is_not_in_library() {
        // A path like C:/tmp/machineembroiderydesigns-test/file.pes
        // was previously treated as in-library by the old substring scan.
        // With strict base-prefix validation it must now be external.
        let is_under = is_path_under_designs_base(
            "C:/tmp/machineembroiderydesigns-test/design.pes",
        );
        assert!(!is_under);

        let parsed =
            full_path_to_stored_design_filepath(
                "C:/tmp/machineembroiderydesigns-test/design.pes",
            );
        assert!(parsed.is_err(), "unrelated path must not produce stored path");
    }

    /// compute_prospective_stored_filepath with a standard leaf root.
    #[test]
    fn prospective_path_standard_root_with_leaf() {
        // Simulates: selected root C:/x/d/f, file C:/x/d/f/Babies/Jef Files/design.jef
        let result = compute_prospective_stored_filepath(
            "C:/x/d/f/Babies/Jef Files/design.jef",
            &["C:/x/d/f".to_string()],
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "/MachineEmbroideryDesigns/f/Babies/Jef Files/design.jef"
        );
    }

    /// compute_prospective_stored_filepath with a parent root (leaf = x).
    #[test]
    fn prospective_path_parent_root() {
        // Selected root C:/x, file C:/x/d/f/Babies/Jef Files/design.jef
        let result = compute_prospective_stored_filepath(
            "C:/x/d/f/Babies/Jef Files/design.jef",
            &["C:/x".to_string()],
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "/MachineEmbroideryDesigns/x/d/f/Babies/Jef Files/design.jef"
        );
    }

    /// compute_prospective_stored_filepath with drive-root selection (no leaf).
    #[test]
    fn prospective_path_drive_root() {
        // Selected root C:/, file C:/Designs/Floral/a.pes
        let result = compute_prospective_stored_filepath(
            "C:/Designs/Floral/a.pes",
            &["C:/".to_string()],
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "/MachineEmbroideryDesigns/Designs/Floral/a.pes"
        );
    }

    /// compute_prospective_stored_filepath with mixed slash separators.
    #[test]
    fn prospective_path_mixed_separators() {
        // Backslash in the file path, forward-slash root
        let result = compute_prospective_stored_filepath(
            "C:\\x\\d\\f\\Babies\\Jef Files\\design.jef",
            &["C:/x/d/f".to_string()],
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "/MachineEmbroideryDesigns/f/Babies/Jef Files/design.jef"
        );
    }

    /// Longest-root match must be chosen when multiple roots are provided.
    #[test]
    fn prospective_path_longest_root_wins() {
        let result = compute_prospective_stored_filepath(
            "C:/x/d/f/Babies/Jef Files/design.jef",
            &["C:/x".to_string(), "C:/x/d/f".to_string()],
        );
        assert!(result.is_ok());
        // Longer root "C:/x/d/f" wins => leaf "f"
        assert_eq!(
            result.unwrap(),
            "/MachineEmbroideryDesigns/f/Babies/Jef Files/design.jef"
        );
    }

    // =========================================================================
    // Phase 5 - In-library detection strictness (5.3)
    // =========================================================================

    #[test]
    fn is_path_under_designs_base_accepts_actual_subpath() {
        let designs_base = get_designs_base_path();
        let file_path = designs_base
            .join("some-folder")
            .join("test.pes")
            .to_string_lossy()
            .replace('\\', "/");

        let is_under = is_path_under_designs_base(&file_path);
        assert!(is_under);
    }

    // =========================================================================
    // Phase 5 - file hash + size utilities
    // =========================================================================

    #[test]
    fn compute_blake3_hash_of_known_content() {
        let dir = std::env::temp_dir().join(format!("rec-hash-test-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        let file_path = dir.join("content.bin");
        fs::write(&file_path, b"hello world").expect("file should be written");

        let hash = compute_file_hash_blake3(&file_path).expect("hash should succeed");
        let size = compute_file_size(&file_path).expect("size should succeed");

        assert_eq!(size, 11); // "hello world" = 11 bytes
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // BLAKE3 hex = 64 chars

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir_all(&dir);
    }

    // =========================================================================
    // Phase 5 - Preview dedup with prospective stored path (5.2)
    // =========================================================================

    #[tokio::test]
    async fn preview_dedup_excludes_by_prospective_stored_path() {
        let pool = import_test_pool().await;

        // Seed a design row that maps to a prospective stored path for a
        // hypothetical external file under /MachineEmbroideryDesigns.
        let stored = "/MachineEmbroideryDesigns/some-folder/unique-file.pes";
        sqlx::query(
            "INSERT INTO designs (filename, filepath, date_added, is_stitched, tags_checked) VALUES (?, ?, DATE('now'), 0, 0)",
        )
        .bind("unique-file.pes")
        .bind(stored)
        .execute(&pool)
        .await
        .expect("seed design should insert");

        // Create a temp file outside the designs base that maps to the same
        // prospective stored path.
        let dir = std::env::temp_dir()
            .join(format!("rec-dedup-prospective-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
        let sub = dir.join("some-folder");
        fs::create_dir_all(&sub).expect("temp subdir should be created");
        let file_path = sub.join("unique-file.pes");
        fs::write(&file_path, b"dummy").expect("temp file should be written");
        let file_path_text = file_path.to_string_lossy().to_string();

        let scanned = vec![scanning::ScannedFile {
            full_path: file_path_text,
            extension: "pes".to_string(),
            dedup_group_key: "test".to_string(),
        }];

        let filtered = filter_existing_scanned_files(
            &pool,
            scanned,
            &[dir.to_string_lossy().to_string()],
        )
        .await
        .expect("filter should succeed");

        // The file should be excluded because its prospective stored path
        // "/MachineEmbroideryDesigns/some-folder/unique-file.pes" matches the
        // seeded row.
        assert!(filtered.is_empty());

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir_all(&dir);
    }
}

async fn load_import_preview_3d_if_initialized(pool: &SqlitePool) -> Result<bool, String> {
    let image_preference: Option<String> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = 'image.preference' LIMIT 1")
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    Ok(!matches!(
        image_preference
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("2d")
    ))
}

async fn load_import_preview_3d_profile_if_initialized(
    pool: &SqlitePool,
) -> Result<String, String> {
    let profile: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'image.preview_3d_profile' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let normalized = profile
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "balanced".to_string());

    Ok(match normalized.as_str() {
        "soft" => "soft".to_string(),
        "high-contrast" | "high_contrast" | "highcontrast" => "high-contrast".to_string(),
        _ => "balanced".to_string(),
    })
}
