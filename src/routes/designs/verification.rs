// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct BulkVerifyResult {
    pub requested_count: usize,
    pub verified_count: usize,
}

#[tauri::command]
pub async fn bulk_verify_designs(
    state: State<'_, AppState>,
    design_ids: Vec<i64>,
) -> Result<BulkVerifyResult, String> {
    if design_ids.is_empty() {
        return Ok(BulkVerifyResult {
            requested_count: 0,
            verified_count: 0,
        });
    }

    let pool = state.db_pool()?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let mut verified_count = 0usize;

    for design_id in &design_ids {
        let result = sqlx::query(
            "UPDATE designs SET image_tags_verified = 1, stitching_tags_verified = 1, date_updated = datetime('now') WHERE id = ?",
        )
        .bind(*design_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        verified_count += result.rows_affected() as usize;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(BulkVerifyResult {
        requested_count: design_ids.len(),
        verified_count,
    })
}
