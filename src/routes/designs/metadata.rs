// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::types::{BulkAddToProjectResult, DesignCommandResult, ProjectListItem, SetDesignProjectRequest};
use crate::AppState;
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use tauri::{Emitter, State};

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDesignMetadataRequest {
    pub notes: Option<String>,
    pub designer_id: Option<i64>,
    pub source_id: Option<i64>,
    pub hoop_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetDesignRatingRequest {
    pub rating: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetDesignStitchedRequest {
    pub is_stitched: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetDesignVerificationRequest {
    pub image_tags_verified: Option<bool>,
    pub stitching_tags_verified: Option<bool>,
}

pub(crate) fn round_mm_to_i64(value: Option<f64>) -> Option<i64> {
    value.map(|v| v.round() as i64)
}

pub(crate) fn ceil_mm_to_i64(value: Option<f64>) -> Option<i64> {
    value.map(|v| v.ceil() as i64)
}

pub(crate) fn normalize_optional_text(value: &Option<String>) -> Option<String> {
    match value {
        Some(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => None,
    }
}

pub(crate) fn normalize_optional_fk(value: Option<i64>, label: &str) -> Result<Option<i64>, String> {
    match value {
        Some(id) if id <= 0 => Err(format!("{} must be a positive id.", label)),
        _ => Ok(value),
    }
}

pub(crate) fn validate_rating(rating: Option<i64>) -> Result<Option<i64>, String> {
    match rating {
        Some(value) if !(1..=5).contains(&value) => {
            Err("Rating must be between 1 and 5, or null to clear it.".to_string())
        }
        _ => Ok(rating),
    }
}

pub(crate) async fn ensure_design_exists(pool: &SqlitePool, design_id: i64) -> Result<(), String> {
    let exists = sqlx::query_scalar::<_, i64>("SELECT 1 FROM designs WHERE id = ? LIMIT 1")
        .bind(design_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .is_some();

    if exists {
        Ok(())
    } else {
        Err(format!("Design with id={} not found.", design_id))
    }
}

pub(crate) async fn ensure_foreign_key_exists(
    pool: &SqlitePool,
    table: &str,
    id: Option<i64>,
    label: &str,
) -> Result<(), String> {
    if let Some(value) = id {
        let sql = format!("SELECT 1 FROM {} WHERE id = ? LIMIT 1", table);
        let exists = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(sql))
            .bind(value)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .is_some();

        if !exists {
            return Err(format!("{} with id={} not found.", label, value));
        }
    }

    Ok(())
}

pub(crate) async fn update_design_metadata_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    request: UpdateDesignMetadataRequest,
) -> Result<DesignCommandResult, String> {
    ensure_design_exists(pool, design_id).await?;

    let designer_id = normalize_optional_fk(request.designer_id, "Designer")?;
    let source_id = normalize_optional_fk(request.source_id, "Source")?;
    let hoop_id = normalize_optional_fk(request.hoop_id, "Hoop")?;

    ensure_foreign_key_exists(pool, "designers", designer_id, "Designer").await?;
    ensure_foreign_key_exists(pool, "sources", source_id, "Source").await?;
    ensure_foreign_key_exists(pool, "hoops", hoop_id, "Hoop").await?;

    let notes = normalize_optional_text(&request.notes);

    sqlx::query(
        "UPDATE designs SET notes = ?, designer_id = ?, source_id = ?, hoop_id = ? WHERE id = ?",
    )
    .bind(notes)
    .bind(designer_id)
    .bind(source_id)
    .bind(hoop_id)
    .bind(design_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Design metadata updated.".to_string(),
    })
}

pub(crate) async fn set_design_rating_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    rating: Option<i64>,
) -> Result<DesignCommandResult, String> {
    ensure_design_exists(pool, design_id).await?;
    let normalized = validate_rating(rating)?;

    sqlx::query("UPDATE designs SET rating = ? WHERE id = ?")
        .bind(normalized)
        .bind(design_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Design rating updated.".to_string(),
    })
}

pub(crate) async fn set_design_stitched_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    is_stitched: bool,
) -> Result<DesignCommandResult, String> {
    ensure_design_exists(pool, design_id).await?;

    sqlx::query("UPDATE designs SET is_stitched = ? WHERE id = ?")
        .bind(is_stitched)
        .bind(design_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Design stitched state updated.".to_string(),
    })
}

pub(crate) async fn set_design_verification_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    image_tags_verified: Option<bool>,
    stitching_tags_verified: Option<bool>,
) -> Result<DesignCommandResult, String> {
    ensure_design_exists(pool, design_id).await?;

    if let Some(value) = image_tags_verified {
        sqlx::query("UPDATE designs SET image_tags_verified = ? WHERE id = ?")
            .bind(value)
            .bind(design_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(value) = stitching_tags_verified {
        sqlx::query("UPDATE designs SET stitching_tags_verified = ? WHERE id = ?")
            .bind(value)
            .bind(design_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(DesignCommandResult {
        design_id,
        message: "Design verification state updated.".to_string(),
    })
}

pub(crate) async fn add_design_to_project_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    project_id: i64,
) -> Result<DesignCommandResult, String> {
    if project_id <= 0 {
        return Err("A valid project must be selected.".to_string());
    }

    ensure_design_exists(pool, design_id).await?;
    ensure_foreign_key_exists(pool, "projects", Some(project_id), "Project").await?;

    sqlx::query("INSERT OR IGNORE INTO project_designs (project_id, design_id) VALUES (?, ?)")
        .bind(project_id)
        .bind(design_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Design added to project.".to_string(),
    })
}

pub(crate) async fn remove_design_from_project_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    project_id: i64,
) -> Result<DesignCommandResult, String> {
    if project_id <= 0 {
        return Err("A valid project must be selected.".to_string());
    }

    ensure_design_exists(pool, design_id).await?;
    ensure_foreign_key_exists(pool, "projects", Some(project_id), "Project").await?;

    sqlx::query("DELETE FROM project_designs WHERE project_id = ? AND design_id = ?")
        .bind(project_id)
        .bind(design_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Design removed from project.".to_string(),
    })
}

#[tauri::command]
pub async fn get_projects_for_browse(
    state: State<'_, AppState>,
) -> Result<Vec<ProjectListItem>, String> {
    sqlx::query_as::<_, ProjectListItem>(
        r#"
		SELECT
			p.id AS id,
			p.name AS name
		FROM projects p
		ORDER BY p.name COLLATE NOCASE ASC
		"#,
    )
    .fetch_all(&state.db_pool()?)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_add_designs_to_project(
    state: State<'_, AppState>,
    project_id: i64,
    design_ids: Vec<i64>,
) -> Result<BulkAddToProjectResult, String> {
    if project_id <= 0 {
        return Err("A valid project must be selected.".to_string());
    }

    if design_ids.is_empty() {
        return Ok(BulkAddToProjectResult {
            project_id,
            requested_count: 0,
            added_count: 0,
        });
    }

    let pool = state.db_pool()?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let mut added_count = 0usize;

    for design_id in &design_ids {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO project_designs (project_id, design_id) VALUES (?, ?)",
        )
        .bind(project_id)
        .bind(*design_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        added_count += result.rows_affected() as usize;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(BulkAddToProjectResult {
        project_id,
        requested_count: design_ids.len(),
        added_count,
    })
}

#[tauri::command]
pub async fn update_design_metadata(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: UpdateDesignMetadataRequest,
) -> Result<DesignCommandResult, String> {
    let result = update_design_metadata_with_pool(&state.db_pool()?, design_id, request).await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": {}
        }),
    );
    Ok(result)
}

#[tauri::command]
pub async fn set_design_rating(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignRatingRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_rating_with_pool(&state.db_pool()?, design_id, request.rating).await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": { "rating": request.rating }
        }),
    );
    Ok(result)
}

#[tauri::command]
pub async fn set_design_stitched(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignStitchedRequest,
) -> Result<DesignCommandResult, String> {
    let result =
        set_design_stitched_with_pool(&state.db_pool()?, design_id, request.is_stitched).await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": { "is_stitched": request.is_stitched }
        }),
    );
    Ok(result)
}

#[tauri::command]
pub async fn set_design_verification(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignVerificationRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_verification_with_pool(
        &state.db_pool()?,
        design_id,
        request.image_tags_verified,
        request.stitching_tags_verified,
    )
    .await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": {
                "image_tags_verified": request.image_tags_verified,
                "stitching_tags_verified": request.stitching_tags_verified,
            }
        }),
    );
    Ok(result)
}

#[tauri::command]
pub async fn add_design_to_project(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignProjectRequest,
) -> Result<DesignCommandResult, String> {
    let result =
        add_design_to_project_with_pool(&state.db_pool()?, design_id, request.project_id).await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": {}
        }),
    );
    Ok(result)
}

#[tauri::command]
pub async fn remove_design_from_project(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    project_id: i64,
) -> Result<DesignCommandResult, String> {
    let result =
        remove_design_from_project_with_pool(&state.db_pool()?, design_id, project_id).await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": {}
        }),
    );
    Ok(result)
}
