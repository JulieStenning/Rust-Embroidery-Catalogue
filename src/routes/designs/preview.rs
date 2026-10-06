// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::launch::{get_design_filepath, resolve_design_full_path};
use super::metadata::round_mm_to_i64;
use crate::services::design_metadata;
use crate::services::image_generation::{generate_preview, ImageGenerationRequest};
use crate::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use tauri::{Emitter, State};

#[derive(Debug, Clone, Serialize)]
pub struct BrowseDesignPreview {
    pub id: i64,
    pub data_url: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub(crate) struct BrowseDesignPreviewRow {
    pub id: i64,
    pub image_data: Option<Vec<u8>>,
    pub image_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Render3dPreviewResult {
    pub design_id: i64,
    pub image_type: Option<String>,
    pub width_mm: Option<i64>,
    pub height_mm: Option<i64>,
    pub stitch_count: Option<i64>,
    pub color_count: Option<i64>,
    pub color_change_count: Option<i64>,
    pub backend: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RenderPreviewRequest {
    pub preview_3d: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReparseDesignResult {
    pub design_id: i64,
    pub width_mm: Option<i64>,
    pub height_mm: Option<i64>,
    pub stitch_count: Option<i64>,
    pub color_count: Option<i64>,
    pub color_change_count: Option<i64>,
    pub hoop_id: Option<i64>,
    pub hoop: Option<String>,
    pub message: String,
}

pub(crate) fn image_mime_from_type(image_type: Option<&str>) -> &'static str {
    match image_type {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        _ => "image/png",
    }
}

pub(crate) fn build_data_url(
    image_data: Option<Vec<u8>>,
    image_type: Option<&str>,
) -> Option<String> {
    let mime = image_mime_from_type(image_type);
    image_data.map(|bytes| {
        let mut url = String::with_capacity(bytes.len() * 4 / 3 + 32);
        url.push_str("data:");
        url.push_str(mime);
        url.push_str(";base64,");
        STANDARD.encode_string(&bytes, &mut url);
        url
    })
}

pub(crate) async fn render_design_3d_preview_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    preview_3d: bool,
) -> Result<Render3dPreviewResult, String> {
    let filepath = get_design_filepath(pool, design_id).await?;
    let full_path = resolve_design_full_path(&filepath);

    if !full_path.is_file() {
        return Err("Design file not found on disk for preview rendering.".to_string());
    }

    let preview_3d_profile: Option<String> = if preview_3d {
        sqlx::query_scalar(
            "SELECT value FROM settings WHERE key = 'image.preview_3d_profile' LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        None
    };

    let preview_3d_profile = preview_3d_profile
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .map(|value| match value.as_str() {
            "soft" => "soft".to_string(),
            "high-contrast" | "high_contrast" | "highcontrast" => "high-contrast".to_string(),
            _ => "balanced".to_string(),
        })
        .unwrap_or_else(|| "balanced".to_string());

    let generation_result = generate_preview(&ImageGenerationRequest {
        file_path: full_path.to_string_lossy().to_string(),
        preview_3d,
        preview_3d_profile: Some(preview_3d_profile),
    });

    if let Some(error) = generation_result.error {
        return Err(error);
    }

    let image_type = generation_result
        .image_type
        .clone()
        .or_else(|| Some(if preview_3d { "3d" } else { "2d" }.to_string()));
    let width_mm = round_mm_to_i64(generation_result.width_mm);
    let height_mm = round_mm_to_i64(generation_result.height_mm);

    sqlx::query(
		"UPDATE designs SET image_data = ?, image_type = ?, width_mm = ?, height_mm = ?, stitch_count = ?, color_count = ?, color_change_count = ? WHERE id = ?",
	)
	.bind(generation_result.image_data)
	.bind(image_type.clone())
	.bind(width_mm)
	.bind(height_mm)
	.bind(generation_result.stitch_count)
	.bind(generation_result.color_count)
	.bind(generation_result.color_change_count)
	.bind(design_id)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;

    let preview_label = if preview_3d { "3D" } else { "2D" };

    Ok(Render3dPreviewResult {
        design_id,
        image_type,
        width_mm,
        height_mm,
        stitch_count: generation_result.stitch_count,
        color_count: generation_result.color_count,
        color_change_count: generation_result.color_change_count,
        backend: generation_result.backend,
        message: format!("{preview_label} preview rendered and saved."),
    })
}

pub(crate) async fn reparse_design_file_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<ReparseDesignResult, String> {
    let filepath = get_design_filepath(pool, design_id).await?;
    let full_path = resolve_design_full_path(&filepath);

    if !full_path.is_file() {
        return Err("Design file not found on disk for metadata recalculation.".to_string());
    }

    let parsed = design_metadata::parse_design_file(&full_path)
        .map_err(|error| format!("Could not re-parse the design file: {}", error))?;

    let width_mm = parsed.width_mm;
    let height_mm = parsed.height_mm;
    let hoop_id = design_metadata::recommend_hoop_for_design(pool, width_mm, height_mm).await?;

    sqlx::query(
        "UPDATE designs SET width_mm = ?, height_mm = ?, stitch_count = ?, color_count = ?, color_change_count = ?, hoop_id = ? WHERE id = ?",
    )
    .bind(width_mm)
    .bind(height_mm)
    .bind(parsed.stitch_count)
    .bind(parsed.color_count)
    .bind(parsed.color_change_count)
    .bind(hoop_id)
    .bind(design_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let hoop = match hoop_id {
        Some(id) => sqlx::query_scalar::<_, String>("SELECT name FROM hoops WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?,
        None => None,
    };

    Ok(ReparseDesignResult {
        design_id,
        width_mm,
        height_mm,
        stitch_count: parsed.stitch_count,
        color_count: parsed.color_count,
        color_change_count: parsed.color_change_count,
        hoop_id,
        hoop,
        message: "Design metadata recalculated from file.".to_string(),
    })
}

#[tauri::command]
pub async fn get_design_previews_for_browse(
    state: State<'_, AppState>,
    design_ids: Vec<i64>,
) -> Result<Vec<BrowseDesignPreview>, String> {
    if design_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder =
        QueryBuilder::<Sqlite>::new("SELECT id, image_data, image_type FROM designs WHERE id IN (");

    let mut separated = builder.separated(", ");
    for id in &design_ids {
        separated.push_bind(*id);
    }
    builder.push(")");

    let rows = builder
        .build_query_as::<BrowseDesignPreviewRow>()
        .fetch_all(&state.db_pool()?)
        .await
        .map_err(|e| e.to_string())?;

    let mut previews = Vec::with_capacity(rows.len());
    for row in rows {
        let mime = image_mime_from_type(row.image_type.as_deref());
        let data_url = row.image_data.map(|bytes| {
            let mut url = String::with_capacity(bytes.len() * 4 / 3 + 32);
            url.push_str("data:");
            url.push_str(mime);
            url.push_str(";base64,");
            STANDARD.encode_string(&bytes, &mut url);
            url
        });

        previews.push(BrowseDesignPreview {
            id: row.id,
            data_url,
        });
    }

    Ok(previews)
}

#[tauri::command]
pub async fn render_design_3d_preview(
    state: State<'_, AppState>,
    design_id: i64,
    request: Option<RenderPreviewRequest>,
) -> Result<Render3dPreviewResult, String> {
    let preview_3d = request.map(|r| r.preview_3d).unwrap_or(true);
    render_design_3d_preview_with_pool(&state.db_pool()?, design_id, preview_3d).await
}

#[tauri::command]
pub async fn reparse_design_file(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<ReparseDesignResult, String> {
    let result = reparse_design_file_with_pool(&state.db_pool()?, design_id).await?;
    let _ = app_handle.emit(
        "design:mutated",
        json!({
            "design_id": design_id,
            "fields": {}
        }),
    );
    Ok(result)
}
