// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::launch::resolve_design_full_path;
use super::metadata::ensure_design_exists;
use super::types::DesignCommandResult;
use crate::services::compaction::schedule_incremental_vacuum;
use crate::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use tauri::{Emitter, State};

#[derive(Debug, Clone, Deserialize)]
pub struct BulkDeleteDesignsRequest {
    pub design_ids: Vec<i64>,
    pub delete_files: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkDeleteDesignsResult {
    pub requested_count: usize,
    pub deleted_count: usize,
    pub files_trashed: usize,
    pub errors: Vec<String>,
}

pub(crate) async fn delete_design_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    delete_file: bool,
) -> Result<DesignCommandResult, String> {
    ensure_design_exists(pool, design_id).await?;

    let filepath: Option<String> = if delete_file {
        sqlx::query_scalar::<_, String>("SELECT filepath FROM designs WHERE id = ?")
            .bind(design_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    sqlx::query("DELETE FROM designs WHERE id = ?")
        .bind(design_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(stored_path) = filepath {
        let trimmed = stored_path.trim();
        if !trimmed.is_empty() {
            let full_path = resolve_design_full_path(trimmed);
            if full_path.is_file() {
                trash::delete(&full_path).map_err(|e| {
                    format!(
                        "Design deleted from catalogue, but could not move file to recycle bin: {}. File path: {}",
                        e,
                        full_path.display()
                    )
                })?;
            }
        }
    }

    let message = if delete_file {
        "Design and file deleted.".to_string()
    } else {
        "Design deleted.".to_string()
    };

    Ok(DesignCommandResult { design_id, message })
}

pub(crate) async fn bulk_delete_designs_with_pool(
    pool: &SqlitePool,
    design_ids: &[i64],
    delete_files: bool,
) -> Result<BulkDeleteDesignsResult, String> {
    if design_ids.is_empty() {
        return Ok(BulkDeleteDesignsResult {
            requested_count: 0,
            deleted_count: 0,
            files_trashed: 0,
            errors: Vec::new(),
        });
    }

    if design_ids.len() > 50 {
        return Err("Cannot delete more than 50 designs in a single batch operation.".to_string());
    }

    // Deduplicate
    let mut deduped: Vec<i64> = design_ids.to_vec();
    deduped.sort_unstable();
    deduped.dedup();

    let requested_count = deduped.len();

    // Fetch filepaths for all designs (needed if delete_files is true)
    let filepath_rows: Vec<(i64, String)> = if delete_files {
        let mut query =
            QueryBuilder::<Sqlite>::new("SELECT id, filepath FROM designs WHERE id IN (");
        let mut separated = query.separated(", ");
        for id in &deduped {
            separated.push_bind(*id);
        }
        query.push(")");

        query
            .build_query_as::<(i64, String)>()
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    // Batch delete from DB
    let mut delete_query = QueryBuilder::<Sqlite>::new("DELETE FROM designs WHERE id IN (");
    let mut separated = delete_query.separated(", ");
    for id in &deduped {
        separated.push_bind(*id);
    }
    delete_query.push(")");

    let delete_result = delete_query
        .build()
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    let deleted_count = delete_result.rows_affected() as usize;

    // If delete_files is requested, trash each file (collect errors, don't abort)
    let mut files_trashed = 0usize;
    let mut errors: Vec<String> = Vec::new();

    if delete_files {
        for (design_id, filepath) in &filepath_rows {
            let trimmed = filepath.trim();
            if trimmed.is_empty() {
                continue;
            }

            let full_path = resolve_design_full_path(trimmed);
            if !full_path.is_file() {
                errors.push(format!(
                    "Design {} file not found on disk: {}",
                    design_id,
                    full_path.display()
                ));
                continue;
            }

            match trash::delete(&full_path) {
                Ok(()) => files_trashed += 1,
                Err(e) => errors.push(format!(
                    "Could not trash file for design {} ({}): {}",
                    design_id,
                    full_path.display(),
                    e
                )),
            }
        }
    }

    Ok(BulkDeleteDesignsResult {
        requested_count,
        deleted_count,
        files_trashed,
        errors,
    })
}

#[tauri::command]
pub async fn delete_design(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    delete_file: bool,
) -> Result<DesignCommandResult, String> {
    let result = delete_design_with_pool(&state.db_pool()?, design_id, delete_file).await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": { "_deleted": true }
        }),
    );
    // Reclaim freelist pages asynchronously after the delete commits, so the
    // UI never blocks on database file compaction.
    schedule_incremental_vacuum(state.db_pool()?);
    Ok(result)
}

#[tauri::command]
pub async fn bulk_delete_designs(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    request: BulkDeleteDesignsRequest,
) -> Result<BulkDeleteDesignsResult, String> {
    let result =
        bulk_delete_designs_with_pool(&state.db_pool()?, &request.design_ids, request.delete_files)
            .await?;
    // Emit events for each deleted design
    for design_id in &request.design_ids {
        let _ = app_handle.emit(
            "design:mutated",
            json!({
                "design_id": design_id,
                "fields": { "_deleted": true }
            }),
        );
    }
    // Reclaim freelist pages asynchronously after the bulk delete commits, so
    // the UI never blocks on database file compaction.
    schedule_incremental_vacuum(state.db_pool()?);
    Ok(result)
}
