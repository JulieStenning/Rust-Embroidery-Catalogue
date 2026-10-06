// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::browse_query::{browse_sort_clause, push_browse_filters, GetDesignsPayload};
use crate::AppState;
use serde::Serialize;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct BrowseDesignSummary {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub master_filepath: Option<String>,
    pub is_master_only: bool,
    pub designer: String,
    pub source: String,
    pub hoop: Option<String>,
    pub projects: Vec<String>,
    pub tags: Vec<String>,
    pub image_tags: Vec<String>,
    pub stitching_tags: Vec<String>,
    pub is_stitched: bool,
    pub image_tags_verified: bool,
    pub stitching_tags_verified: bool,
    pub rating: Option<i64>,
    pub date_added: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct BrowseDesignSummaryRow {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub master_filepath: Option<String>,
    pub is_master_only: bool,
    pub designer: String,
    pub source: String,
    pub hoop: Option<String>,
    pub projects_csv: Option<String>,
    pub tags_csv: Option<String>,
    pub image_tags_csv: Option<String>,
    pub stitching_tags_csv: Option<String>,
    pub is_stitched: bool,
    pub image_tags_verified: bool,
    pub stitching_tags_verified: bool,
    pub rating: Option<i64>,
    pub date_added: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowseDesignsPageResult {
    pub items: Vec<BrowseDesignSummary>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignIdsResult {
    pub ids: Vec<i64>,
}

#[tauri::command]
pub async fn get_designs(
    state: State<'_, AppState>,
    payload: Option<GetDesignsPayload>,
) -> Result<BrowseDesignsPageResult, String> {
    get_designs_page_with_pool(&state.db_pool()?, payload).await
}

/// Fetch the full ordered list of design IDs matching the same browse filters
/// and sort as the paginated page query (no LIMIT/OFFSET). This is what the
/// detail view's Prev/Next navigation walks, so it covers the entire filtered
/// result set rather than just the current page.
#[tauri::command]
pub async fn get_design_ids(
    state: State<'_, AppState>,
    payload: Option<GetDesignsPayload>,
) -> Result<DesignIdsResult, String> {
    get_design_ids_with_pool(&state.db_pool()?, payload).await
}

pub(crate) async fn get_design_ids_with_pool(
    pool: &SqlitePool,
    payload: Option<GetDesignsPayload>,
) -> Result<DesignIdsResult, String> {
    let payload = payload.unwrap_or_default();
    let sort_clause = browse_sort_clause(payload.sort_by.as_deref(), payload.sort_dir.as_deref());

    let mut ids_builder = QueryBuilder::<Sqlite>::new("SELECT d.id FROM designs d");
    push_browse_filters(&mut ids_builder, &payload);
    ids_builder.push(" ORDER BY ");
    ids_builder.push(sort_clause.as_str());

    let ids: Vec<i64> = ids_builder
        .build_query_scalar()
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DesignIdsResult { ids })
}

pub(crate) async fn get_designs_page_with_pool(
    pool: &SqlitePool,
    payload: Option<GetDesignsPayload>,
) -> Result<BrowseDesignsPageResult, String> {
    let payload = payload.unwrap_or_default();
    let page = payload.page.unwrap_or(1).max(1);
    let page_size = payload.page_size.unwrap_or(50).clamp(1, 500);
    let sort_clause = browse_sort_clause(payload.sort_by.as_deref(), payload.sort_dir.as_deref());

    // 1. Total count for the pagination controls.
    let mut count_builder = QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM designs d");
    push_browse_filters(&mut count_builder, &payload);
    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let total_pages = if total == 0 {
        1
    } else {
        (total + page_size - 1) / page_size
    };
    let normalized_page = page.min(total_pages.max(1));
    let offset = (normalized_page - 1) * page_size;

    // 2. Page ids (cheap: no tag aggregation).
    let mut ids_builder = QueryBuilder::<Sqlite>::new("SELECT d.id FROM designs d");
    push_browse_filters(&mut ids_builder, &payload);
    ids_builder.push(" ORDER BY ");
    ids_builder.push(sort_clause.as_str());
    ids_builder.push(" LIMIT ");
    ids_builder.push_bind(page_size);
    ids_builder.push(" OFFSET ");
    ids_builder.push_bind(offset);

    let page_ids: Vec<i64> = ids_builder
        .build_query_scalar()
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    if page_ids.is_empty() {
        return Ok(BrowseDesignsPageResult {
            items: Vec::new(),
            page: normalized_page,
            page_size,
            total,
            total_pages,
        });
    }

    // 3. Aggregate tags/projects only for the page's ids.
    let mut agg_builder = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            d.id AS id,
            d.filename AS filename,
            d.filepath AS filepath,
            d.master_filepath AS master_filepath,
            COALESCE(d.is_master_only, 0) AS is_master_only,
            COALESCE(designers.name, 'Unknown') AS designer,
            COALESCE(sources.name, 'Unknown') AS source,
            hoops.name AS hoop,
            (
                SELECT GROUP_CONCAT(projects.name, '|||')
                FROM project_designs
                JOIN projects ON projects.id = project_designs.project_id
                WHERE project_designs.design_id = d.id
            ) AS projects_csv,
            GROUP_CONCAT(tags.description, '|||') AS tags_csv,
            GROUP_CONCAT(CASE WHEN lower(COALESCE(tags.tag_group, '')) = 'stitching' THEN tags.description END, '|||') AS stitching_tags_csv,
            GROUP_CONCAT(CASE WHEN lower(COALESCE(tags.tag_group, '')) != 'stitching' THEN tags.description END, '|||') AS image_tags_csv,
            d.is_stitched AS is_stitched,
            d.image_tags_verified AS image_tags_verified,
            d.stitching_tags_verified AS stitching_tags_verified,
            d.rating AS rating,
            d.date_added AS date_added
        FROM designs d
        LEFT JOIN designers ON designers.id = d.designer_id
        LEFT JOIN sources ON sources.id = d.source_id
        LEFT JOIN hoops ON hoops.id = d.hoop_id
        LEFT JOIN design_tags ON design_tags.design_id = d.id
        LEFT JOIN tags ON tags.id = design_tags.tag_id
        WHERE d.id IN (
        "#,
    );
    {
        let mut separated = agg_builder.separated(", ");
        for design_id in &page_ids {
            separated.push_bind(*design_id);
        }
    }
    agg_builder.push(") GROUP BY d.id ORDER BY ");
    agg_builder.push(sort_clause.as_str());

    let rows = agg_builder
        .build_query_as::<BrowseDesignSummaryRow>()
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let items = rows
        .into_iter()
        .map(|row| BrowseDesignSummary {
            id: row.id,
            filename: row.filename,
            filepath: row.filepath,
            master_filepath: row
                .master_filepath
                .map(|p| crate::paths::canonical_design_rel(&p)),
            is_master_only: row.is_master_only,
            designer: row.designer,
            source: row.source,
            hoop: row.hoop,
            projects: row
                .projects_csv
                .unwrap_or_default()
                .split("|||")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(String::from)
                .collect(),
            tags: row
                .tags_csv
                .unwrap_or_default()
                .split("|||")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(String::from)
                .collect(),
            image_tags: row
                .image_tags_csv
                .unwrap_or_default()
                .split("|||")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(String::from)
                .collect(),
            stitching_tags: row
                .stitching_tags_csv
                .unwrap_or_default()
                .split("|||")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(String::from)
                .collect(),
            is_stitched: row.is_stitched,
            image_tags_verified: row.image_tags_verified,
            stitching_tags_verified: row.stitching_tags_verified,
            rating: row.rating,
            date_added: row.date_added,
        })
        .collect();

    Ok(BrowseDesignsPageResult {
        items,
        page: normalized_page,
        page_size,
        total,
        total_pages,
    })
}
