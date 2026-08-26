//! Tauri command surface for catalogue storage migration.
//!
//! `start_catalogue_storage_migration` runs the migration on a blocking task
//! (filesystem I/O) while streaming progress to the frontend via
//! `catalogue-storage-migration-progress` events. `cancel` sets an atomic flag
//! that the copy loop observes between files.

use crate::services::storage_migration::{self, StorageMigrationSummary};
use crate::AppState;
use std::sync::Arc;
use tauri::{Emitter, State};

/// Start migrating the active catalogue to `target_dir`.
///
/// Streams `StorageMigrationProgress` on `catalogue-storage-migration-progress`
/// and resolves with a final summary. `force` moves a pre-existing non-empty
/// target aside (`<target>.before-migration-backup`) before copying.
#[tauri::command]
pub async fn start_catalogue_storage_migration(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    target_dir: String,
    force: Option<bool>,
) -> Result<StorageMigrationSummary, String> {
    // Guard against concurrent migrations.
    if state
        .migration_running
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return Err("A catalogue storage migration is already in progress.".to_string());
    }
    state
        .migration_running
        .store(true, std::sync::atomic::Ordering::SeqCst);
    state
        .migration_cancel_requested
        .store(false, std::sync::atomic::Ordering::SeqCst);

    let result =
        run_migration_blocking(app_handle, &state, &target_dir, force.unwrap_or(false)).await;

    state
        .migration_running
        .store(false, std::sync::atomic::Ordering::SeqCst);

    result
}

/// Request cancellation of a running migration. Cooperative — the copy loop
/// checks the flag between files.
#[tauri::command]
pub fn cancel_catalogue_storage_migration(state: State<'_, AppState>) -> Result<(), String> {
    state
        .migration_cancel_requested
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

async fn run_migration_blocking(
    app_handle: tauri::AppHandle,
    state: &AppState,
    target_dir: &str,
    force: bool,
) -> Result<StorageMigrationSummary, String> {
    let source = &state.paths;
    let target = std::path::PathBuf::from(target_dir.trim());

    // Pre-flight (cheap and can reuse the atomic directly).
    let plan = storage_migration::preflight(source, &target, force).map_err(|e| e.to_string())?;

    // WAL checkpoint so the main .db is complete before the copy.
    storage_migration::checkpoint_live_database(&state.db_pool()?)
        .await
        .map_err(|e| e.to_string())?;

    // Run the migration on a blocking task so the async runtime stays free.
    let cancel = Arc::new(state.migration_cancel_requested.clone());
    let cancel_for_task = cancel.clone();
    let plan_clone = plan.clone();
    let source_clone = source.clone();
    let handle = app_handle.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        runtime.block_on(async move {
            let cancel_flag = cancel_for_task.as_ref();
            storage_migration::run_migration(&source_clone, &plan_clone, cancel_flag, |event| {
                let _ = handle.emit(storage_migration::STORAGE_MIGRATION_PROGRESS_EVENT, &event);
            })
            .await
            .map_err(|e| e.to_string())
        })
    })
    .await
    .map_err(|e| format!("migration thread panicked: {e}"))?
}
