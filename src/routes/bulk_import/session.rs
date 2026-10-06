// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::types::{
    BulkImportConfirmWire, BulkImportContextStoreResetResult, BulkImportContextStoreSummary,
    BulkImportStopResult,
};
use crate::services::scanning;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const BULK_IMPORT_CONTEXT_TTL: Duration = Duration::from_secs(15 * 60);
pub(crate) const BULK_IMPORT_CONTEXT_MAX_ENTRIES: usize = 128;

pub(crate) static BULK_IMPORT_CONTEXT_STORE: OnceLock<
    Mutex<HashMap<String, StoredBulkImportContext>>,
> = OnceLock::new();
pub(crate) static BULK_IMPORT_CONTEXT_COUNTER: AtomicU64 = AtomicU64::new(1);
pub(crate) static BULK_IMPORT_DB_POOL: Mutex<Option<SqlitePool>> = Mutex::new(None);
pub(crate) static BULK_IMPORT_APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
pub(crate) static BULK_IMPORT_CONTEXT_RESET_COUNTER: AtomicU64 = AtomicU64::new(0);
pub(crate) static BULK_IMPORT_CONTEXT_LAST_RESET_AT_MILLIS: AtomicU64 = AtomicU64::new(0);
pub(crate) static BULK_IMPORT_STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

pub(crate) const DEFAULT_IMPORT_COMMIT_BATCH_SIZE: usize = 100;
pub(crate) const BULK_IMPORT_PROGRESS_EVENT: &str = "bulk-import-progress";

pub(crate) const BULK_IMPORT_SCAN_TTL: Duration = Duration::from_secs(60 * 60);
pub(crate) const BULK_IMPORT_SCAN_MAX_ENTRIES: usize = 64;
pub(crate) const IMPORT_UNKNOWN_FOLDER: &str = "Unknown folder";

#[derive(Debug, Clone)]
pub(crate) struct StoredBulkImportScan {
    pub(crate) root_paths: Vec<String>,
    pub(crate) scanned_files: Vec<scanning::ScannedFile>,
    pub(crate) created_at_millis: u128,
    pub(crate) sequence: u64,
}

pub(crate) static BULK_IMPORT_SCAN_STORE: OnceLock<Mutex<HashMap<String, StoredBulkImportScan>>> =
    OnceLock::new();
pub(crate) static BULK_IMPORT_SCAN_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BulkImportProgressEvent {
    pub(crate) context_token: Option<String>,
    pub(crate) stage: String,
    pub(crate) processed_count: usize,
    pub(crate) total_count: usize,
    pub(crate) persisted_count: usize,
    pub(crate) committed_count: usize,
    pub(crate) failed_count: usize,
    pub(crate) current_file: Option<String>,
    pub(crate) commit_batch_size: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredBulkImportContext {
    pub(crate) confirm_wire: BulkImportConfirmWire,
    pub(crate) created_at_millis: u128,
    pub(crate) sequence: u64,
}

pub(crate) fn bulk_import_context_store() -> &'static Mutex<HashMap<String, StoredBulkImportContext>>
{
    BULK_IMPORT_CONTEXT_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn with_bulk_import_context_store<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut HashMap<String, StoredBulkImportContext>) -> T,
{
    let store = bulk_import_context_store();
    let mut guard = store
        .lock()
        .map_err(|_| "Bulk import context lock is poisoned.".to_string())?;
    Ok(f(&mut guard))
}

pub(crate) fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0)
}

pub(crate) fn prune_bulk_import_context_store(
    store: &mut HashMap<String, StoredBulkImportContext>,
) {
    let now = current_timestamp_millis();
    let ttl_millis = BULK_IMPORT_CONTEXT_TTL.as_millis();
    store.retain(|_, entry| now.saturating_sub(entry.created_at_millis) <= ttl_millis);

    if store.len() > BULK_IMPORT_CONTEXT_MAX_ENTRIES {
        let mut entries = store
            .iter()
            .map(|(key, value)| (key.clone(), value.sequence))
            .collect::<Vec<_>>();
        entries.sort_by_key(|(_, sequence)| *sequence);
        let remove_count = store.len() - BULK_IMPORT_CONTEXT_MAX_ENTRIES;
        for (key, _) in entries.into_iter().take(remove_count) {
            store.remove(&key);
        }
    }
}

pub(crate) fn bulk_import_scan_store() -> &'static Mutex<HashMap<String, StoredBulkImportScan>> {
    BULK_IMPORT_SCAN_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn with_bulk_import_scan_store<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut HashMap<String, StoredBulkImportScan>) -> T,
{
    let store = bulk_import_scan_store();
    let mut guard = store
        .lock()
        .map_err(|_| "Bulk import scan lock is poisoned.".to_string())?;
    Ok(f(&mut guard))
}

pub(crate) fn prune_bulk_import_scan_store(store: &mut HashMap<String, StoredBulkImportScan>) {
    let now = current_timestamp_millis();
    let ttl_millis = BULK_IMPORT_SCAN_TTL.as_millis();
    store.retain(|_, entry| now.saturating_sub(entry.created_at_millis) <= ttl_millis);

    if store.len() > BULK_IMPORT_SCAN_MAX_ENTRIES {
        let mut entries = store
            .iter()
            .map(|(key, value)| (key.clone(), value.sequence))
            .collect::<Vec<_>>();
        entries.sort_by_key(|(_, sequence)| *sequence);
        let remove_count = store.len() - BULK_IMPORT_SCAN_MAX_ENTRIES;
        for (key, _) in entries.into_iter().take(remove_count) {
            store.remove(&key);
        }
    }
}

pub(crate) fn next_bulk_import_scan_token() -> (String, u64) {
    let sequence = BULK_IMPORT_SCAN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0);
    let token = format!("scan-{}-{}", now, sequence);
    (token, sequence)
}

pub(crate) fn store_bulk_import_scan(
    root_paths: Vec<String>,
    scanned_files: Vec<scanning::ScannedFile>,
) -> String {
    let (token, sequence) = next_bulk_import_scan_token();
    let created_at_millis = current_timestamp_millis();
    let stored = StoredBulkImportScan {
        root_paths,
        scanned_files,
        created_at_millis,
        sequence,
    };

    let token_clone = token.clone();
    let _ = with_bulk_import_scan_store(|store| {
        prune_bulk_import_scan_store(store);
        store.insert(token_clone, stored);
    });

    token
}

pub(crate) fn take_bulk_import_scan(token: &str) -> Option<StoredBulkImportScan> {
    with_bulk_import_scan_store(|store| {
        prune_bulk_import_scan_store(store);
        store.remove(token)
    })
    .ok()
    .flatten()
}

pub(crate) fn insert_bulk_import_context_for_test(
    token: String,
    confirm_wire: BulkImportConfirmWire,
    created_at_millis: u128,
    sequence: u64,
) {
    let stored = StoredBulkImportContext {
        confirm_wire,
        created_at_millis,
        sequence,
    };
    let _ = with_bulk_import_context_store(|store| {
        store.insert(token, stored);
    });
}

pub(crate) fn next_bulk_import_context_token() -> (String, u64) {
    let sequence = BULK_IMPORT_CONTEXT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0);
    let token = format!("bulk-import-{}-{}", now, sequence);
    (token, sequence)
}

pub fn initialize_bulk_import_db_pool(pool: SqlitePool) {
    let mut guard = BULK_IMPORT_DB_POOL
        .lock()
        .expect("BULK_IMPORT_DB_POOL lock should not be poisoned during init");
    *guard = Some(pool);
}

pub fn update_bulk_import_db_pool(pool: SqlitePool) {
    initialize_bulk_import_db_pool(pool);
}

pub fn initialize_bulk_import_app_handle(app_handle: tauri::AppHandle) {
    let _ = BULK_IMPORT_APP_HANDLE.set(app_handle);
}

pub(crate) fn get_bulk_import_db_pool() -> Option<SqlitePool> {
    BULK_IMPORT_DB_POOL
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

pub(crate) fn get_bulk_import_app_handle() -> Option<&'static tauri::AppHandle> {
    BULK_IMPORT_APP_HANDLE.get()
}

pub(crate) fn clear_bulk_import_context_store_internal(
    reason: &str,
) -> BulkImportContextStoreResetResult {
    let (cleared_context_count, active_context_count) =
        with_bulk_import_context_store(|store| {
            let cleared = store.len();
            store.clear();
            (cleared, store.len())
        })
        .unwrap_or((0, 0));

    let reset_count = BULK_IMPORT_CONTEXT_RESET_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let reset_at_millis = current_timestamp_millis() as u64;
    BULK_IMPORT_CONTEXT_LAST_RESET_AT_MILLIS.store(reset_at_millis, Ordering::Relaxed);

    BulkImportContextStoreResetResult {
        cleared_context_count,
        active_context_count,
        reset_count,
        reset_at_millis,
        reason: reason.to_string(),
    }
}

pub fn reset_bulk_import_context_store_for_startup() -> BulkImportContextStoreResetResult {
    clear_bulk_import_context_store_internal("startup")
}

pub fn reset_bulk_import_context_store_for_restore() -> BulkImportContextStoreResetResult {
    clear_bulk_import_context_store_internal("restore")
}

pub fn store_bulk_import_context(mut confirm_wire: BulkImportConfirmWire) -> String {
    let (token, sequence) = next_bulk_import_context_token();
    let created_at_millis = current_timestamp_millis();
    confirm_wire.context_token = Some(token.clone());
    let stored = StoredBulkImportContext {
        confirm_wire,
        created_at_millis,
        sequence,
    };

    let token_clone = token.clone();
    let _ = with_bulk_import_context_store(|store| {
        prune_bulk_import_context_store(store);
        store.insert(token_clone, stored);
    });

    token
}

pub fn take_bulk_import_context(token: &str) -> Option<BulkImportConfirmWire> {
    with_bulk_import_context_store(|store| {
        prune_bulk_import_context_store(store);
        store.remove(token).map(|entry| entry.confirm_wire)
    })
    .ok()
    .flatten()
}

pub fn get_bulk_import_context(token: &str) -> Option<BulkImportConfirmWire> {
    with_bulk_import_context_store(|store| {
        prune_bulk_import_context_store(store);
        store.get(token).map(|entry| entry.confirm_wire.clone())
    })
    .ok()
    .flatten()
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
