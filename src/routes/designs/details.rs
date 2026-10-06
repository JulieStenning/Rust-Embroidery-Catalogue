// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::metadata::ceil_mm_to_i64;
use super::preview::{build_data_url, BrowseDesignPreviewRow};
use super::types::{BrowseTagOption, DesignLookupOption, ProjectListItem};
use crate::AppState;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use std::collections::HashSet;
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DesignDetail {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub master_filepath: Option<String>,
    pub is_master_only: bool,
    pub image_type: Option<String>,
    pub image_data_url: Option<String>,
    pub width_mm: Option<i64>,
    pub height_mm: Option<i64>,
    pub stitch_count: Option<i64>,
    pub color_count: Option<i64>,
    pub color_change_count: Option<i64>,
    pub designer: String,
    pub designer_id: Option<i64>,
    pub source: String,
    pub source_id: Option<i64>,
    pub hoop: Option<String>,
    pub hoop_id: Option<i64>,
    pub notes: Option<String>,
    pub rating: Option<i64>,
    pub is_stitched: bool,
    pub image_tags_verified: bool,
    pub stitching_tags_verified: bool,
    pub date_added: Option<String>,
    pub tags: Vec<DesignTagDetail>,
    pub projects: Vec<ProjectListItem>,
    pub available_projects: Vec<ProjectListItem>,
    pub all_tags: Vec<BrowseTagOption>,
    pub designers: Vec<DesignLookupOption>,
    pub sources: Vec<DesignLookupOption>,
    pub hoops: Vec<DesignLookupOption>,
}

#[derive(Debug, Clone, FromRow)]
struct DesignDetailRow {
    id: i64,
    filename: String,
    filepath: String,
    master_filepath: Option<String>,
    is_master_only: bool,
    image_data: Option<Vec<u8>>,
    image_type: Option<String>,
    width_mm: Option<f64>,
    height_mm: Option<f64>,
    stitch_count: Option<i64>,
    color_count: Option<i64>,
    color_change_count: Option<i64>,
    designer: String,
    designer_id: Option<i64>,
    source: String,
    source_id: Option<i64>,
    hoop: Option<String>,
    hoop_id: Option<i64>,
    notes: Option<String>,
    rating: Option<i64>,
    is_stitched: bool,
    image_tags_verified: bool,
    stitching_tags_verified: bool,
    date_added: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DesignTagDetail {
    pub id: i64,
    pub description: String,
    pub tag_group: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignImageData {
    pub design_id: i64,
    pub image_type: Option<String>,
    pub data_url: Option<String>,
}

pub(crate) async fn get_design_detail_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<Option<DesignDetail>, String> {
    let detail_row = sqlx::query_as::<_, DesignDetailRow>(
        r#"
		SELECT
			d.id AS id,
			d.filename AS filename,
			d.filepath AS filepath,
			d.master_filepath AS master_filepath,
			COALESCE(d.is_master_only, 0) AS is_master_only,
			d.image_data AS image_data,
			d.image_type AS image_type,
			CAST(d.width_mm AS REAL) AS width_mm,
			CAST(d.height_mm AS REAL) AS height_mm,
			d.stitch_count AS stitch_count,
			d.color_count AS color_count,
			d.color_change_count AS color_change_count,
			COALESCE(designers.name, 'Unknown') AS designer,
			d.designer_id AS designer_id,
			COALESCE(sources.name, 'Unknown') AS source,
			d.source_id AS source_id,
			hoops.name AS hoop,
			d.hoop_id AS hoop_id,
			d.notes AS notes,
			d.rating AS rating,
			d.is_stitched AS is_stitched,
			d.image_tags_verified AS image_tags_verified,
			d.stitching_tags_verified AS stitching_tags_verified,
			d.date_added AS date_added
		FROM designs d
		LEFT JOIN designers ON designers.id = d.designer_id
		LEFT JOIN sources ON sources.id = d.source_id
		LEFT JOIN hoops ON hoops.id = d.hoop_id
		WHERE d.id = ?
		LIMIT 1
		"#,
    )
    .bind(design_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let row = match detail_row {
        Some(value) => value,
        None => return Ok(None),
    };

    let tags = sqlx::query_as::<_, DesignTagDetail>(
        r#"
		SELECT
			t.id AS id,
			t.description AS description,
			t.tag_group AS tag_group
		FROM tags t
		INNER JOIN design_tags dt ON dt.tag_id = t.id
		WHERE dt.design_id = ?
		ORDER BY t.description COLLATE NOCASE ASC
		"#,
    )
    .bind(design_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let projects = sqlx::query_as::<_, ProjectListItem>(
        r#"
		SELECT p.id AS id, p.name AS name
		FROM projects p
		INNER JOIN project_designs pd ON pd.project_id = p.id
		WHERE pd.design_id = ?
		ORDER BY p.name COLLATE NOCASE ASC
		"#,
    )
    .bind(design_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let all_projects = sqlx::query_as::<_, ProjectListItem>(
        r#"
		SELECT p.id AS id, p.name AS name
		FROM projects p
		ORDER BY p.name COLLATE NOCASE ASC
		"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let project_ids: HashSet<i64> = projects.iter().map(|p| p.id).collect();
    let available_projects: Vec<ProjectListItem> = all_projects
        .into_iter()
        .filter(|p| !project_ids.contains(&p.id))
        .collect();

    let all_tags = sqlx::query_as::<_, BrowseTagOption>(
        r#"
		SELECT
			t.id AS id,
			t.description AS description,
			t.tag_group AS tag_group
		FROM tags t
		ORDER BY t.description COLLATE NOCASE ASC
		"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let designers = sqlx::query_as::<_, DesignLookupOption>(
        r#"
		SELECT d.id AS id, d.name AS name
		FROM designers d
		ORDER BY d.name COLLATE NOCASE ASC
		"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let sources = sqlx::query_as::<_, DesignLookupOption>(
        r#"
		SELECT s.id AS id, s.name AS name
		FROM sources s
		ORDER BY s.name COLLATE NOCASE ASC
		"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let hoops = sqlx::query_as::<_, DesignLookupOption>(
        r#"
		SELECT h.id AS id, h.name AS name
		FROM hoops h
		ORDER BY h.name COLLATE NOCASE ASC
		"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Some(DesignDetail {
        id: row.id,
        filename: row.filename,
        filepath: crate::paths::canonical_design_rel(&row.filepath),
        master_filepath: row
            .master_filepath
            .map(|p| crate::paths::canonical_design_rel(&p)),
        is_master_only: row.is_master_only,
        image_type: row.image_type.clone(),
        image_data_url: build_data_url(row.image_data, row.image_type.as_deref()),
        width_mm: ceil_mm_to_i64(row.width_mm),
        height_mm: ceil_mm_to_i64(row.height_mm),
        stitch_count: row.stitch_count,
        color_count: row.color_count,
        color_change_count: row.color_change_count,
        designer: row.designer,
        designer_id: row.designer_id,
        source: row.source,
        source_id: row.source_id,
        hoop: row.hoop,
        hoop_id: row.hoop_id,
        notes: row.notes,
        rating: row.rating,
        is_stitched: row.is_stitched,
        image_tags_verified: row.image_tags_verified,
        stitching_tags_verified: row.stitching_tags_verified,
        date_added: row.date_added,
        tags,
        projects,
        available_projects,
        all_tags,
        designers,
        sources,
        hoops,
    }))
}

pub(crate) async fn get_design_image_data_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<Option<DesignImageData>, String> {
    let row = sqlx::query_as::<_, (Option<Vec<u8>>, Option<String>)>(
        "SELECT image_data, image_type FROM designs WHERE id = ? LIMIT 1",
    )
    .bind(design_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let value = match row {
        Some(v) => BrowseDesignPreviewRow {
            id: design_id,
            image_data: v.0,
            image_type: v.1,
        },
        None => return Ok(None),
    };

    Ok(Some(DesignImageData {
        design_id: value.id,
        image_type: value.image_type.clone(),
        data_url: build_data_url(value.image_data, value.image_type.as_deref()),
    }))
}

#[tauri::command]
pub async fn get_design_detail(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<Option<DesignDetail>, String> {
    get_design_detail_with_pool(&state.db_pool()?, design_id).await
}

#[tauri::command]
pub async fn get_design_image_data_url(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<Option<DesignImageData>, String> {
    get_design_image_data_with_pool(&state.db_pool()?, design_id).await
}
