use crate::services::{folder_picker, scanning, validation};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BULK_IMPORT_CONTEXT_TTL: Duration = Duration::from_secs(15 * 60);
const BULK_IMPORT_CONTEXT_MAX_ENTRIES: usize = 128;

static BULK_IMPORT_CONTEXT_STORE: OnceLock<Mutex<HashMap<String, StoredBulkImportContext>>> = OnceLock::new();
static BULK_IMPORT_CONTEXT_COUNTER: AtomicU64 = AtomicU64::new(1);
static BULK_IMPORT_DB_POOL: OnceLock<SqlitePool> = OnceLock::new();
static BULK_IMPORT_CONTEXT_RESET_COUNTER: AtomicU64 = AtomicU64::new(0);
static BULK_IMPORT_CONTEXT_LAST_RESET_AT_MILLIS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct StoredBulkImportContext {
    confirm_wire: BulkImportConfirmWire,
    created_at_millis: u128,
    sequence: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportRequest {
    pub root_path: String,
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
        Self {
            root_paths: vec![request.root_path],
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

fn get_bulk_import_db_pool() -> Option<SqlitePool> {
    BULK_IMPORT_DB_POOL.get().cloned()
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

fn load_import_precheck_state_if_initialized() -> Result<(bool, bool), String> {
    let Some(pool) = get_bulk_import_db_pool() else {
        return Ok((false, false));
    };

    let (design_count, hoop_count) = tauri::async_runtime::block_on(load_catalog_counts(&pool))?;
    let is_first_import = design_count == 0;
    let needs_hoop_setup = is_first_import && hoop_count == 0;
    Ok((is_first_import, needs_hoop_setup))
}

fn normalize_path_for_match(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
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
            if best_match.map(|(_, best_score)| score > best_score).unwrap_or(true) {
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
) -> Result<usize, String> {
    if !confirm_wire.wire.create_on_import {
        return Ok(0);
    }

    let resolved_assignments = resolve_bulk_import_assignments(confirm_wire);
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let mut persisted_design_count = 0usize;

    for file_path in &confirm_wire.wire.selected_files {
        let existing_design_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM designs WHERE filepath = ? LIMIT 1",
        )
        .bind(file_path)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        if existing_design_id.is_some() {
            continue;
        }

        let (designer_id, source_id) =
            resolve_assignment_for_file(file_path, confirm_wire, &resolved_assignments);

        let filename = Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(file_path)
            .to_string();

        sqlx::query(
            "INSERT INTO designs (filename, filepath, date_added, designer_id, source_id, is_stitched, tags_checked) VALUES (?, ?, DATE('now'), ?, ?, 0, 0)",
        )
        .bind(filename)
        .bind(file_path)
        .bind(designer_id)
        .bind(source_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        persisted_design_count += 1;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(persisted_design_count)
}

fn persist_bulk_import_confirm_if_initialized(
    confirm_wire: &BulkImportConfirmWire,
) -> Result<usize, String> {
    match get_bulk_import_db_pool() {
        Some(pool) => tauri::async_runtime::block_on(persist_bulk_import_confirm_wire(&pool, confirm_wire)),
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
pub fn preview_bulk_import(request: BulkImportRequest) -> Result<BulkImportPreview, String> {
    let wire: BulkImportWire = request.into();

    preview_bulk_import_wire(wire)
}

#[tauri::command]
pub fn debug_bulk_import_wire(wire: BulkImportWire) -> Result<BulkImportWireSummary, String> {
    println!("Debug bulk import wire: {:#?}", wire);
    Ok(BulkImportWireSummary {
        root_path_count: wire.root_paths.len(),
        folder_assignment_count: wire.per_folder_assignments.len(),
        selected_file_count: wire.selected_files.len(),
        create_on_import: wire.create_on_import,
    })
}

#[tauri::command]
pub fn debug_bulk_import_confirm_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmSummary, String> {
    println!("Debug bulk import confirm wire: {:#?}", confirm_wire);
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
    println!("Debug bulk import assignment resolution: {:#?}", resolved_assignments);

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
    println!("Precheck bulk import stored token: {context_token}");
    println!("Precheck bulk import resolved assignments: {:#?}", resolved_assignments);

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
pub fn precheck_bulk_import_action_wire(
    request: BulkImportPrecheckActionRequest,
) -> Result<BulkImportPrecheckActionResult, String> {
    let context_token = request.context_token.clone();

    match request.action {
        BulkImportPrecheckActionWire::ReviewHoops => {
            get_bulk_import_context(&context_token)
                .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

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
            get_bulk_import_context(&context_token)
                .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

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
            get_bulk_import_context(&context_token)
                .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

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
            get_bulk_import_context(&context_token)
                .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

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
            take_bulk_import_context(&context_token)
                .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

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
            get_bulk_import_context(&context_token)
                .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

            let (is_first_import, needs_hoop_setup) = load_import_precheck_state_if_initialized()?;
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

            let confirm_result = do_confirm_bulk_import_wire(context_token)?;
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
    let confirm_wire = take_bulk_import_context(&context_token)
        .ok_or_else(|| format!("Unknown or expired bulk import context token: {context_token}"))?;

    println!("Do-confirm bulk import using token: {context_token}");
    let persisted_design_count = persist_bulk_import_confirm_if_initialized(&confirm_wire)?;
    let mut result = confirm_bulk_import_wire(confirm_wire)?;
    result.persisted_design_count = persisted_design_count;
    Ok(result)
}

#[tauri::command]
pub fn execute_bulk_import_confirm_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let persisted_design_count = persist_bulk_import_confirm_if_initialized(&confirm_wire)?;
    let mut result = confirm_bulk_import_wire(confirm_wire)?;
    result.persisted_design_count = persisted_design_count;
    Ok(result)
}

#[tauri::command]
pub fn confirm_bulk_import_wire(
    confirm_wire: BulkImportConfirmWire,
) -> Result<BulkImportConfirmExecutionResult, String> {
    let resolved_assignments = resolve_bulk_import_assignments(&confirm_wire);
    println!("Canonical bulk import confirm wire: {:#?}", confirm_wire);
    println!("Canonical bulk import resolved assignments: {:#?}", resolved_assignments);

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

pub fn preview_bulk_import_wire(wire: BulkImportWire) -> Result<BulkImportPreview, String> {
    for root_path in &wire.root_paths {
        validation::validate_path(root_path).map_err(|e| format!("{:?}", e))?;
    }

    let mut discovered_count = 0usize;
    let mut scanned_files = Vec::new();
    for root_path in &wire.root_paths {
        let scan_input = scanning::ScanInput {
            root_path: root_path.clone(),
        };
        let scan_result = scanning::scan(&scan_input);
        discovered_count += scan_result.files.len();
        scanned_files.extend(scan_result.files);
    }

    scanned_files.sort_by(|left, right| {
        left
            .full_path
            .to_ascii_lowercase()
            .cmp(&right.full_path.to_ascii_lowercase())
    });

    let resolved_assignments = wire
        .per_folder_assignments
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let decoded: BulkImportWire = serde_json::from_str(&encoded).expect("wire should deserialize");

        assert_eq!(decoded.root_paths.len(), 1);
        assert_eq!(decoded.per_folder_assignments.len(), 1);
        assert_eq!(decoded.selected_files.len(), 1);
        assert!(decoded.create_on_import);
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
        assert_eq!(explicit.source, AssignmentFieldSourceWire::ExplicitPerFolder);

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
        assert_eq!(resolved[0].designer_id.source, AssignmentFieldSourceWire::ExplicitPerFolder);
        assert_eq!(resolved[0].source_id.value, Some(8));
        assert_eq!(resolved[0].source_id.source, AssignmentFieldSourceWire::Global);

        assert_eq!(resolved[1].designer_id.value, Some(7));
        assert_eq!(resolved[1].designer_id.source, AssignmentFieldSourceWire::Global);
        assert_eq!(resolved[1].source_id.value, Some(8));
        assert_eq!(resolved[1].source_id.source, AssignmentFieldSourceWire::Global);
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
            root_path: "C:/imports".to_string(),
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

        let action_result = precheck_bulk_import_action_wire(BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::ReviewTags,
            confirm_skip_hoops: false,
        })
        .expect("review action should succeed");

        assert!(!action_result.consumed_context);
        assert!(action_result.context_token_present);
        assert!(action_result.next_route.unwrap_or_default().contains("/admin/tags/"));
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

        let action_result = precheck_bulk_import_action_wire(BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::Cancel,
            confirm_skip_hoops: false,
        })
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

        let action_result = precheck_bulk_import_action_wire(BulkImportPrecheckActionRequest {
            context_token: precheck.context_token.clone(),
            action: BulkImportPrecheckActionWire::ImportNow,
            confirm_skip_hoops: false,
        })
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
        assert!(take_bulk_import_context(&format!("bulk-import-test-{}", BULK_IMPORT_CONTEXT_MAX_ENTRIES as u64 + 1)).is_some());
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

        let expired_created_at = current_timestamp_millis()
            .saturating_sub(BULK_IMPORT_CONTEXT_TTL.as_millis() + 1);
        insert_bulk_import_context_for_test(expired_token.clone(), current_wire, expired_created_at, 9999);

        assert!(take_bulk_import_context(&expired_token).is_none());
    }
}