use crate::config::BootstrapConfig;
use crate::services::compaction::schedule_incremental_vacuum;
use crate::services::design_metadata;
use crate::services::image_generation::{generate_preview, ImageGenerationRequest};
use crate::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Emitter, State};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct BrowseDesignSummary {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BrowseAdditionalFiltersPayload {
    pub designer_filters: Option<Vec<String>>,
    pub image_tag_filters: Option<Vec<String>>,
    pub stitching_tag_filters: Option<Vec<String>>,
    pub source_filters: Option<Vec<String>>,
    pub hoop_size: Option<String>,
    pub min_rating: Option<i64>,
    pub stitched_status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GetDesignsPayload {
    pub q: Option<String>,
    pub search_file_name: Option<bool>,
    pub search_tags: Option<bool>,
    pub search_folder_name: Option<bool>,
    pub unverified_only: Option<bool>,
    pub additional_filters: Option<BrowseAdditionalFiltersPayload>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowseDesignsPageResult {
    pub items: Vec<BrowseDesignSummary>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// Sentinel for the hoop browse filter: selecting it matches designs whose
/// minimum fitting hoop could not be calculated (`designs.hoop_id IS NULL`).
/// Must stay in sync with `HOOP_UNKNOWN_FILTER` in
/// `frontend/src/lib/utils/hoopConstants.js`.
const HOOP_UNKNOWN_SENTINEL: &str = "__hoop_unknown__";

fn push_where_clause(query_builder: &mut QueryBuilder<Sqlite>, has_where: &mut bool) {
    if *has_where {
        query_builder.push(" AND ");
    } else {
        query_builder.push(" WHERE ");
        *has_where = true;
    }
}

/// Map the frontend's browse sort selection to a deterministic SQL ORDER BY
/// clause. The `filename`/`id` tiebreakers keep pagination stable across pages.
fn browse_sort_clause(sort_by: Option<&str>, sort_dir: Option<&str>) -> String {
    let direction = match sort_dir {
        Some(dir) if dir.eq_ignore_ascii_case("desc") => "DESC",
        _ => "ASC",
    };

    let column = match sort_by {
        Some(sort) if sort.eq_ignore_ascii_case("rating") => "COALESCE(d.rating, -1)",
        Some(sort) if sort.eq_ignore_ascii_case("stitched") => "d.is_stitched",
        // Approximates the frontend's "folder then filename" ordering, because
        // the parent directory name is a path prefix of `filepath`.
        Some(sort) if sort.eq_ignore_ascii_case("folder") => "d.filepath COLLATE NOCASE",
        Some(sort) if sort.eq_ignore_ascii_case("date_added") => "COALESCE(d.date_added, '')",
        _ => "d.filename COLLATE NOCASE",
    };

    format!("{column} {direction}, d.filename COLLATE NOCASE ASC, d.id ASC")
}

/// Push the filter predicates shared by the COUNT, page-id, and aggregate
/// queries. Every tag predicate uses a `d.id IN (SELECT ...)` subquery, so the
/// outer `design_tags`/`tags` join is only needed for aggregation, never for
/// filtering — which is what lets the COUNT and page-id queries stay cheap.
fn push_browse_filters(query_builder: &mut QueryBuilder<Sqlite>, payload: &GetDesignsPayload) {
    let mut has_where = false;

    let q_trimmed = payload
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(q) = q_trimmed {
        let search_file = payload.search_file_name.unwrap_or(true);
        let search_tags = payload.search_tags.unwrap_or(true);
        let search_folder = payload.search_folder_name.unwrap_or(true);
        let general_groups = parse_general_search_groups(q);

        if search_file || search_tags || search_folder {
            push_where_clause(query_builder, &mut has_where);
            push_general_search_clause(
                query_builder,
                search_file,
                search_tags,
                search_folder,
                &general_groups,
            );
        }
    }

    if payload.unverified_only.unwrap_or(false) {
        push_where_clause(query_builder, &mut has_where);
        query_builder.push("(d.image_tags_verified = 0 OR d.stitching_tags_verified = 0)");
    }

    if let Some(ref filters) = payload.additional_filters {
        let designer_filters = filters.designer_filters.as_deref().unwrap_or(&[]);
        if !designer_filters.is_empty() {
            push_where_clause(query_builder, &mut has_where);
            query_builder.push("(");
            for (index, value) in designer_filters.iter().enumerate() {
                if index > 0 {
                    query_builder.push(" OR ");
                }
                query_builder.push("LOWER(COALESCE(designers.name, 'Unknown')) = ");
                query_builder.push_bind(value.trim().to_lowercase());
            }
            query_builder.push(")");
        }

        let image_tag_filters = filters.image_tag_filters.as_deref().unwrap_or(&[]);
        if !image_tag_filters.is_empty() {
            push_where_clause(query_builder, &mut has_where);
            query_builder.push("d.id IN (");
            query_builder.push(
                "SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE ",
            );
            query_builder.push("lower(COALESCE(tags.tag_group, '')) != 'stitching' AND (");
            for (index, value) in image_tag_filters.iter().enumerate() {
                if index > 0 {
                    query_builder.push(" OR ");
                }
                query_builder.push("LOWER(tags.description) = ");
                query_builder.push_bind(value.trim().to_lowercase());
            }
            query_builder.push(")");
            query_builder.push(")");
        }

        let stitching_tag_filters = filters.stitching_tag_filters.as_deref().unwrap_or(&[]);
        if !stitching_tag_filters.is_empty() {
            push_where_clause(query_builder, &mut has_where);
            query_builder.push("d.id IN (");
            query_builder.push(
                "SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE ",
            );
            query_builder.push("lower(COALESCE(tags.tag_group, '')) = 'stitching' AND (");
            for (index, value) in stitching_tag_filters.iter().enumerate() {
                if index > 0 {
                    query_builder.push(" OR ");
                }
                query_builder.push("LOWER(tags.description) = ");
                query_builder.push_bind(value.trim().to_lowercase());
            }
            query_builder.push(")");
            query_builder.push(")");
        }

        let source_filters = filters.source_filters.as_deref().unwrap_or(&[]);
        if !source_filters.is_empty() {
            push_where_clause(query_builder, &mut has_where);
            query_builder.push("(");
            for (index, value) in source_filters.iter().enumerate() {
                if index > 0 {
                    query_builder.push(" OR ");
                }
                query_builder.push("LOWER(COALESCE(sources.name, 'Unknown')) = ");
                query_builder.push_bind(value.trim().to_lowercase());
            }
            query_builder.push(")");
        }

        if let Some(ref hoop_size) = filters.hoop_size {
            let hoop_size_trimmed = hoop_size.trim();
            if hoop_size_trimmed == HOOP_UNKNOWN_SENTINEL {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.hoop_id IS NULL");
            } else if !hoop_size_trimmed.is_empty() {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("LOWER(COALESCE(hoops.name, '')) = ");
                query_builder.push_bind(hoop_size_trimmed.to_lowercase());
            }
        }

        if let Some(min_rating) = filters.min_rating {
            if min_rating >= 1 {
                push_where_clause(query_builder, &mut has_where);
                query_builder.push("d.rating >= ");
                query_builder.push_bind(min_rating);
            }
        }

        if let Some(ref stitched_status) = filters.stitched_status {
            let stitched_status_trimmed = stitched_status.trim();
            if !stitched_status_trimmed.is_empty() && stitched_status_trimmed != "all" {
                push_where_clause(query_builder, &mut has_where);
                if stitched_status_trimmed == "yes" {
                    query_builder.push("d.is_stitched = 1");
                } else {
                    query_builder.push("d.is_stitched = 0");
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkVerifyResult {
    pub requested_count: usize,
    pub verified_count: usize,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProjectListItem {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkAddToProjectResult {
    pub project_id: i64,
    pub requested_count: usize,
    pub added_count: usize,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct BrowseTagOption {
    pub id: i64,
    pub description: String,
    pub tag_group: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkSetTagsResult {
    pub requested_count: usize,
    pub updated_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowseDesignPreview {
    pub id: i64,
    pub data_url: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct BrowseDesignPreviewRow {
    pub id: i64,
    pub image_data: Option<Vec<u8>>,
    pub image_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DesignDetail {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
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

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DesignLookupOption {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignCommandResult {
    pub design_id: i64,
    pub message: String,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct SetDesignTagsRequest {
    pub tag_ids: Vec<i64>,
    #[serde(default)]
    pub image_tags_verified: Option<bool>,
    #[serde(default)]
    pub stitching_tags_verified: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkApplyTagsRequest {
    pub tags_to_add: Vec<i64>,
    pub tags_to_remove: Vec<i64>,
    #[serde(default)]
    pub clear_all_tags: bool,
    #[serde(default)]
    pub image_tags_verified: Option<bool>,
    #[serde(default)]
    pub stitching_tags_verified: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetDesignProjectRequest {
    pub project_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignImageData {
    pub design_id: i64,
    pub image_type: Option<String>,
    pub data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LaunchDesignResult {
    pub design_id: i64,
    pub attempted_path: String,
    pub opened_path: Option<String>,
    pub suppressed: bool,
    pub success: bool,
    pub message: String,
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

fn round_mm_to_i64(value: Option<f64>) -> Option<i64> {
    value.map(|v| v.round() as i64)
}

fn ceil_mm_to_i64(value: Option<f64>) -> Option<i64> {
    value.map(|v| v.ceil() as i64)
}

fn normalize_optional_text(value: &Option<String>) -> Option<String> {
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

fn normalize_optional_fk(value: Option<i64>, label: &str) -> Result<Option<i64>, String> {
    match value {
        Some(id) if id <= 0 => Err(format!("{} must be a positive id.", label)),
        _ => Ok(value),
    }
}

fn validate_rating(rating: Option<i64>) -> Result<Option<i64>, String> {
    match rating {
        Some(value) if !(1..=5).contains(&value) => {
            Err("Rating must be between 1 and 5, or null to clear it.".to_string())
        }
        _ => Ok(rating),
    }
}

async fn ensure_design_exists(pool: &SqlitePool, design_id: i64) -> Result<(), String> {
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

async fn ensure_foreign_key_exists(
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

fn image_mime_from_type(image_type: Option<&str>) -> &'static str {
    match image_type {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        _ => "image/png",
    }
}

fn build_data_url(image_data: Option<Vec<u8>>, image_type: Option<&str>) -> Option<String> {
    let mime = image_mime_from_type(image_type);
    image_data.map(|bytes| format!("data:{};base64,{}", mime, STANDARD.encode(bytes)))
}

fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "accepted"
    )
}

fn external_launches_disabled() -> bool {
    if let Ok(value) = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN") {
        if is_truthy(&value) {
            return true;
        }
    }

    false
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

/// Resolve a stored `filepath` to the absolute on-disk location of the design
/// under the current designs library root. Works for the canonical
/// library-relative form (`Flowers/rose.pes`), any legacy
/// `/MachineEmbroideryDesigns/…` / markerless form, and absolute rows (returned
/// as-is) via the shared single source of truth in `crate::paths`.
fn resolve_design_full_path(relative_file_path: &str) -> PathBuf {
    let designs_base = get_designs_base_path();
    crate::paths::resolve_design_filepath(relative_file_path, &designs_base)
}
fn nearest_existing_folder(path: &Path, fallback: &Path) -> PathBuf {
    let mut candidate = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| fallback.to_path_buf())
    };

    loop {
        if candidate.is_dir() {
            return candidate;
        }

        let Some(parent) = candidate.parent() else {
            break;
        };

        if parent == candidate {
            break;
        }

        candidate = parent.to_path_buf();
    }

    fallback.to_path_buf()
}

#[cfg(target_os = "windows")]
fn normalize_windows_explorer_target(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    let without_verbatim = if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", rest)
    } else if let Some(rest) = raw.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        raw.to_string()
    };

    PathBuf::from(without_verbatim.replace('/', r"\"))
}

fn open_with_default_app(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to launch default app: {}", e))?;
        return Ok(());
    }

    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to launch default app: {}", e))?;
        return Ok(());
    }

    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to launch default app: {}", e))?;

    Ok(())
}

async fn get_design_filepath(pool: &SqlitePool, design_id: i64) -> Result<String, String> {
    let filepath =
        sqlx::query_scalar::<_, String>("SELECT filepath FROM designs WHERE id = ? LIMIT 1")
            .bind(design_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    match filepath {
        Some(value) if !value.trim().is_empty() => Ok(value),
        Some(_) => Err(format!(
            "Design with id={} does not have a stored filepath.",
            design_id
        )),
        None => Err(format!("Design with id={} not found.", design_id)),
    }
}

async fn open_design_in_editor_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    let filepath = get_design_filepath(pool, design_id).await?;
    let full_path = resolve_design_full_path(&filepath);
    let attempted = full_path.to_string_lossy().to_string();

    if external_launches_disabled() {
        return Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: true,
            success: false,
            message: "External launches are disabled in this runtime context.".to_string(),
        });
    }

    if !full_path.is_file() {
        return Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: false,
            success: false,
            message: "Design file was not found on disk.".to_string(),
        });
    }

    match open_with_default_app(&full_path) {
        Ok(()) => Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: Some(full_path.to_string_lossy().to_string()),
            suppressed: false,
            success: true,
            message: "Opened design in the system default app.".to_string(),
        }),
        Err(error) => Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: false,
            success: false,
            message: error,
        }),
    }
}

async fn open_design_in_explorer_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    let filepath = get_design_filepath(pool, design_id).await?;
    let full_path = resolve_design_full_path(&filepath);
    let attempted = full_path.to_string_lossy().to_string();

    if external_launches_disabled() {
        return Ok(LaunchDesignResult {
            design_id,
            attempted_path: attempted,
            opened_path: None,
            suppressed: true,
            success: false,
            message: "External launches are disabled in this runtime context.".to_string(),
        });
    }

    let base = get_designs_base_path();
    let opened_path = if full_path.is_file() {
        if cfg!(target_os = "windows") {
            let select_target = normalize_windows_explorer_target(
                &full_path
                    .canonicalize()
                    .unwrap_or_else(|_| full_path.clone()),
            );
            let _ = Command::new("explorer.exe")
                .arg("/select,")
                .arg(&select_target)
                .spawn()
                .map_err(|e| format!("Failed to open Explorer: {}", e))?;
        } else {
            open_with_default_app(full_path.parent().unwrap_or(&full_path))?;
        }
        full_path
    } else {
        let folder = nearest_existing_folder(&full_path, &base);
        if cfg!(target_os = "windows") {
            let open_target = normalize_windows_explorer_target(
                &folder.canonicalize().unwrap_or_else(|_| folder.clone()),
            );
            let _ = Command::new("explorer.exe")
                .arg(&open_target)
                .spawn()
                .map_err(|e| format!("Failed to open Explorer: {}", e))?;
        } else {
            open_with_default_app(&folder)?;
        }
        folder
    };

    Ok(LaunchDesignResult {
        design_id,
        attempted_path: attempted,
        opened_path: Some(opened_path.to_string_lossy().to_string()),
        suppressed: false,
        success: true,
        message: "Opened Explorer/folder view for design path.".to_string(),
    })
}

async fn render_design_3d_preview_with_pool(
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

/// Re-read the binary design file from disk and refresh the stored technical
/// metadata (dimensions, stitch count, colour counts, recommended hoop).
///
/// The original file is never modified â€” it is only read to extract fresh
/// parameters.  Returns the updated values so the UI can refresh instantly.
async fn reparse_design_file_with_pool(
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
    let hoop_id =
        design_metadata::recommend_hoop_for_design(pool, width_mm, height_mm).await?;

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

async fn get_design_detail_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<Option<DesignDetail>, String> {
    let detail_row = sqlx::query_as::<_, DesignDetailRow>(
        r#"
		SELECT
			d.id AS id,
			d.filename AS filename,
			d.filepath AS filepath,
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

async fn update_design_metadata_with_pool(
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

async fn set_design_rating_with_pool(
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

async fn set_design_stitched_with_pool(
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

async fn set_design_verification_with_pool(
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

/// Classify a set of tag ids into (image tag ids, stitching tag ids) by
/// consulting the `tags.tag_group` column.
async fn classify_tag_ids(
    pool: &SqlitePool,
    tag_ids: &[i64],
) -> Result<(Vec<i64>, Vec<i64>), String> {
    use sqlx::Row;
    let mut image_ids = Vec::new();
    let mut stitching_ids = Vec::new();

    for tag_id in tag_ids {
        let row = sqlx::query("SELECT tag_group FROM tags WHERE id = ? LIMIT 1")
            .bind(*tag_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        let group = row
            .and_then(|record| {
                record
                    .try_get::<Option<String>, _>("tag_group")
                    .ok()
                    .flatten()
            })
            .unwrap_or_default();
        if group.eq_ignore_ascii_case("stitching") {
            stitching_ids.push(*tag_id);
        } else {
            image_ids.push(*tag_id);
        }
    }

    Ok((image_ids, stitching_ids))
}

async fn set_design_tags_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    tag_ids: Vec<i64>,
    request_image: Option<bool>,
    request_stitching: Option<bool>,
) -> Result<DesignCommandResult, String> {
    ensure_design_exists(pool, design_id).await?;

    let mut deduped = Vec::<i64>::new();
    for id in tag_ids {
        if id <= 0 {
            return Err("Tag id values must be positive integers.".to_string());
        }
        if !deduped.contains(&id) {
            deduped.push(id);
        }
    }

    for tag_id in &deduped {
        ensure_foreign_key_exists(pool, "tags", Some(*tag_id), "Tag").await?;
    }

    // Capture the previous image/stitching tag ids so a full-replace can
    // detect whether each domain actually changed.
    fn tag_ids_in_group(rows: &[sqlx::sqlite::SqliteRow], group: &str) -> Vec<i64> {
        use sqlx::Row;
        rows.iter()
            .filter(|row| {
                let g = row
                    .try_get::<Option<String>, _>("tag_group")
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                g.eq_ignore_ascii_case(group)
            })
            .filter_map(|row| row.try_get::<i64, _>("id").ok())
            .collect()
    }

    let existing_rows = sqlx::query(
        "SELECT t.id AS id, t.tag_group AS tag_group
		 FROM tags t
		 INNER JOIN design_tags dt ON dt.tag_id = t.id
		 WHERE dt.design_id = ?",
    )
    .bind(design_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    let existing_image = tag_ids_in_group(&existing_rows, "image");
    let existing_stitching = tag_ids_in_group(&existing_rows, "stitching");

    let (new_image, new_stitching) = classify_tag_ids(pool, &deduped).await?;
    let image_changed = existing_image != new_image;
    let stitching_changed = existing_stitching != new_stitching;

    // Determine the resolved verification values. Explicit request flags win;
    // otherwise any change to a domain marks it verified; unchanged domains
    // stay completely untouched.
    let resolved_image = match request_image {
        Some(value) => Some(value),
        None if image_changed => Some(true),
        None => None,
    };
    let resolved_stitching = match request_stitching {
        Some(value) => Some(value),
        None if stitching_changed => Some(true),
        None => None,
    };

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM design_tags WHERE design_id = ?")
        .bind(design_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for tag_id in &deduped {
        sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
            .bind(design_id)
            .bind(*tag_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(value) = resolved_image {
        sqlx::query("UPDATE designs SET image_tags_verified = ? WHERE id = ?")
            .bind(value)
            .bind(design_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(value) = resolved_stitching {
        sqlx::query("UPDATE designs SET stitching_tags_verified = ? WHERE id = ?")
            .bind(value)
            .bind(design_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Design tags updated and marked as verified.".to_string(),
    })
}

async fn remove_design_tag_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    tag_id: i64,
) -> Result<DesignCommandResult, String> {
    if tag_id <= 0 {
        return Err("Tag id must be a positive integer.".to_string());
    }

    ensure_design_exists(pool, design_id).await?;
    ensure_foreign_key_exists(pool, "tags", Some(tag_id), "Tag").await?;

    sqlx::query("DELETE FROM design_tags WHERE design_id = ? AND tag_id = ?")
        .bind(design_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Tag removed from design.".to_string(),
    })
}

async fn add_design_to_project_with_pool(
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

async fn remove_design_from_project_with_pool(
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

async fn delete_design_with_pool(
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

async fn bulk_delete_designs_with_pool(
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

async fn get_design_image_data_with_pool(
    pool: &SqlitePool,
    design_id: i64,
) -> Result<Option<DesignImageData>, String> {
    let row = sqlx::query_as::<_, BrowseDesignPreviewRow>(
        "SELECT id, image_data, image_type FROM designs WHERE id = ? LIMIT 1",
    )
    .bind(design_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|value| DesignImageData {
        design_id: value.id,
        image_type: value.image_type.clone(),
        data_url: build_data_url(value.image_data, value.image_type.as_deref()),
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralSearchToken {
    pub text: String,
    /// SQLite LIKE fragment with glob semantics applied (see `like_pattern`).
    /// Lowercased so it can be bound directly against `LOWER(...)` columns.
    pub pattern: String,
    pub phrase: bool,
    pub exclude: bool,
    pub is_extension: bool,
}

fn parse_general_search_groups(query: &str) -> Vec<Vec<GeneralSearchToken>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    let mut buffer = String::new();
    let mut in_quotes = false;

    for ch in trimmed.chars() {
        match ch {
            '"' => {
                buffer.push(ch);
                in_quotes = !in_quotes;
            }
            ' ' | '\t' | '\n' if !in_quotes => {
                let token = buffer.trim();
                if !token.is_empty() {
                    if token.eq_ignore_ascii_case("OR") {
                        if !current_group.is_empty() {
                            groups.push(std::mem::take(&mut current_group));
                        }
                    } else {
                        current_group.push(parse_general_token(token));
                    }
                    buffer.clear();
                }
            }
            _ => buffer.push(ch),
        }
    }

    if !buffer.trim().is_empty() {
        let token = buffer.trim();
        if token.eq_ignore_ascii_case("OR") {
            if !current_group.is_empty() {
                groups.push(std::mem::take(&mut current_group));
            }
        } else {
            current_group.push(parse_general_token(token));
        }
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

fn parse_general_token(raw: &str) -> GeneralSearchToken {
    let trimmed = raw.trim();
    let mut exclude = false;
    let mut text = trimmed;

    if let Some(stripped) = trimmed.strip_prefix('-') {
        exclude = true;
        text = stripped;
    }

    let phrase = text.starts_with('"') && text.ends_with('"') || text.contains('"');
    let normalized = text.trim_matches('"').trim();

    let is_extension =
        normalized.starts_with("*") && normalized.len() > 1 && !normalized.contains(' ');
    let final_text = if is_extension {
        normalized
            .trim_start_matches('*')
            .trim_start_matches('.')
            .trim()
            .to_string()
    } else {
        normalized.to_string()
    };

    GeneralSearchToken {
        text: final_text,
        pattern: like_pattern(normalized),
        phrase,
        exclude,
        is_extension,
    }
}

/// Build a SQLite LIKE fragment from a user term using glob semantics.
///
/// - A bare term (no `*`) matches as a substring: `Sig4` → `%sig4%`.
/// - A `*` acts as `%` (zero-or-more characters) and un-anchors the edge it
///   touches:
///   - `Sig4*` → `sig4%` (starts with "Sig4")
///   - `*Sig4` → `%sig4` (ends with "Sig4")
///   - `*Sig4*` → `%sig4%` (contains "Sig4")
///   - `*.hus` → `%.hus` (ends with the ".hus" extension)
fn like_pattern(term: &str) -> String {
    let lower = term.to_lowercase();
    if lower.contains('*') {
        lower.replace('*', "%")
    } else {
        format!("%{lower}%")
    }
}

/// Folder-name search matches the canonical relative `filepath` directly — no
/// container stripping is needed because stored paths no longer carry the
/// `MachineEmbroideryDesigns` marker or a leading slash (see `crate::paths`).
fn library_folder_sql_expr(column: &str) -> String {
    column.to_string()
}

fn push_general_search_clause(
    query_builder: &mut QueryBuilder<Sqlite>,
    search_file: bool,
    search_tags: bool,
    search_folder: bool,
    general_groups: &[Vec<GeneralSearchToken>],
) {
    if general_groups.is_empty() {
        return;
    }

    query_builder.push("(");
    for (group_index, group_tokens) in general_groups.iter().enumerate() {
        if group_index > 0 {
            query_builder.push(" OR ");
        }

        if group_tokens.is_empty() {
            continue;
        }

        query_builder.push("(");
        for (token_index, token) in group_tokens.iter().enumerate() {
            if token_index > 0 {
                query_builder.push(" AND ");
            }

            let pattern = token.pattern.clone();
            if token.exclude {
                query_builder.push("NOT (");
            }

            let mut added = false;
            if search_file {
                query_builder.push("LOWER(d.filename) LIKE ");
                query_builder.push_bind(pattern.clone());
                added = true;
            }

            if search_tags {
                if added {
                    query_builder.push(" OR ");
                }
                query_builder.push("d.id IN (SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE LOWER(tags.description) LIKE ");
                query_builder.push_bind(pattern.clone());
                query_builder.push(")");
                added = true;
            }

            if search_folder {
                if added {
                    query_builder.push(" OR ");
                }
                let folder_expr = library_folder_sql_expr("d.filepath");
                let folder_search_sql = format!("LOWER({folder_expr}) LIKE ");
                query_builder.push(&folder_search_sql);
                query_builder.push_bind(pattern);
            }

            if token.exclude {
                query_builder.push(")");
            }
        }
        query_builder.push(")");
    }
    query_builder.push(")");
}

#[tauri::command]
pub async fn get_designs(
    state: State<'_, AppState>,
    payload: Option<GetDesignsPayload>,
) -> Result<BrowseDesignsPageResult, String> {
    get_designs_page_with_pool(&state.db_pool()?, payload).await
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignIdsResult {
    pub ids: Vec<i64>,
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

async fn get_design_ids_with_pool(
    pool: &SqlitePool,
    payload: Option<GetDesignsPayload>,
) -> Result<DesignIdsResult, String> {
    let payload = payload.unwrap_or_default();
    let sort_clause = browse_sort_clause(payload.sort_by.as_deref(), payload.sort_dir.as_deref());

    let mut ids_builder = QueryBuilder::<Sqlite>::new(
        "SELECT d.id FROM designs d \
         LEFT JOIN designers ON designers.id = d.designer_id \
         LEFT JOIN sources ON sources.id = d.source_id \
         LEFT JOIN hoops ON hoops.id = d.hoop_id",
    );
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

async fn get_designs_page_with_pool(
    pool: &SqlitePool,
    payload: Option<GetDesignsPayload>,
) -> Result<BrowseDesignsPageResult, String> {
    let payload = payload.unwrap_or_default();
    let page = payload.page.unwrap_or(1).max(1);
    let page_size = payload.page_size.unwrap_or(50).clamp(1, 500);
    let sort_clause = browse_sort_clause(payload.sort_by.as_deref(), payload.sort_dir.as_deref());

    // 1. Total count for the pagination controls.
    let mut count_builder = QueryBuilder::<Sqlite>::new(
        "SELECT COUNT(*) FROM designs d \
         LEFT JOIN designers ON designers.id = d.designer_id \
         LEFT JOIN sources ON sources.id = d.source_id \
         LEFT JOIN hoops ON hoops.id = d.hoop_id",
    );
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
    let mut ids_builder = QueryBuilder::<Sqlite>::new(
        "SELECT d.id FROM designs d \
         LEFT JOIN designers ON designers.id = d.designer_id \
         LEFT JOIN sources ON sources.id = d.source_id \
         LEFT JOIN hoops ON hoops.id = d.hoop_id",
    );
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
            "UPDATE designs SET image_tags_verified = 1, stitching_tags_verified = 1 WHERE id = ?",
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
pub async fn get_tags_for_browse(
    state: State<'_, AppState>,
) -> Result<Vec<BrowseTagOption>, String> {
    sqlx::query_as::<_, BrowseTagOption>(
        r#"
		SELECT
			t.id AS id,
			t.description AS description,
			t.tag_group AS tag_group
		FROM tags t
		ORDER BY t.description COLLATE NOCASE ASC
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

async fn bulk_set_tags_for_designs_with_pool(
    pool: &SqlitePool,
    design_ids: &[i64],
    request: BulkApplyTagsRequest,
) -> Result<BulkSetTagsResult, String> {
    if design_ids.is_empty() {
        return Ok(BulkSetTagsResult {
            requested_count: 0,
            updated_count: 0,
        });
    }

    // Deduplicate and validate all tag ids; add wins over remove when both
    // reference the same tag.
    let mut tags_to_add = Vec::<i64>::new();
    for id in request.tags_to_add {
        if id <= 0 {
            return Err("Tag id values must be positive integers.".to_string());
        }
        if !tags_to_add.contains(&id) {
            tags_to_add.push(id);
        }
    }

    let mut tags_to_remove = Vec::<i64>::new();
    for id in request.tags_to_remove {
        if id <= 0 {
            return Err("Tag id values must be positive integers.".to_string());
        }
        if !tags_to_remove.contains(&id) {
            tags_to_remove.push(id);
        }
    }

    for tag_id in tags_to_add.iter().chain(tags_to_remove.iter()) {
        ensure_foreign_key_exists(pool, "tags", Some(*tag_id), "Tag").await?;
    }

    // Verification flags are optional. `None` means "leave this design's flag
    // exactly as it is" — we must NOT clear a prior verified status when a
    // category is left untouched (mixed/indeterminate category preserved).
    // Explicit `Some(value)` always wins and is written regardless of tag diff.
    let resolved_image = request.image_tags_verified;
    let resolved_stitching = request.stitching_tags_verified;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let mut updated_count = 0usize;

    for design_id in design_ids {
        let mut design_changed = false;

        if request.clear_all_tags {
            let result = sqlx::query("DELETE FROM design_tags WHERE design_id = ?")
                .bind(*design_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            design_changed |= result.rows_affected() > 0;
        } else if !tags_to_remove.is_empty() {
            let mut delete_query =
                QueryBuilder::<Sqlite>::new("DELETE FROM design_tags WHERE design_id = ");
            delete_query.push_bind(*design_id);
            delete_query.push(" AND tag_id IN (");
            let mut separated = delete_query.separated(", ");
            for tag_id in &tags_to_remove {
                separated.push_bind(*tag_id);
            }
            delete_query.push(")");
            let result = delete_query
                .build()
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            design_changed |= result.rows_affected() > 0;
        }

        for tag_id in &tags_to_add {
            let result =
                sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
                    .bind(*design_id)
                    .bind(*tag_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            design_changed |= result.rows_affected() > 0;
        }

        if let Some(value) = resolved_image {
            design_changed = true;
            sqlx::query("UPDATE designs SET image_tags_verified = ? WHERE id = ?")
                .bind(value)
                .bind(*design_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }

        if let Some(value) = resolved_stitching {
            design_changed = true;
            sqlx::query("UPDATE designs SET stitching_tags_verified = ? WHERE id = ?")
                .bind(value)
                .bind(*design_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }

        if design_changed {
            updated_count += 1;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(BulkSetTagsResult {
        requested_count: design_ids.len(),
        updated_count,
    })
}

#[tauri::command]
pub async fn bulk_set_tags_for_designs(
    state: State<'_, AppState>,
    design_ids: Vec<i64>,
    request: BulkApplyTagsRequest,
) -> Result<BulkSetTagsResult, String> {
    bulk_set_tags_for_designs_with_pool(&state.db_pool()?, &design_ids, request).await
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

    let previews = rows
        .into_iter()
        .map(|row| {
            let mime = match row.image_type.as_deref() {
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("webp") => "image/webp",
                Some("gif") => "image/gif",
                Some("bmp") => "image/bmp",
                _ => "image/png",
            };

            let data_url = row
                .image_data
                .map(|bytes| format!("data:{};base64,{}", mime, STANDARD.encode(bytes)));

            BrowseDesignPreview {
                id: row.id,
                data_url,
            }
        })
        .collect();

    Ok(previews)
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
    let result = set_design_stitched_with_pool(&state.db_pool()?, design_id, request.is_stitched).await?;
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
pub async fn set_design_tags(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignTagsRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_tags_with_pool(
        &state.db_pool()?,
        design_id,
        request.tag_ids,
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
pub async fn remove_design_tag(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    tag_id: i64,
) -> Result<DesignCommandResult, String> {
    let result = remove_design_tag_with_pool(&state.db_pool()?, design_id, tag_id).await?;
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
pub async fn add_design_to_project(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignProjectRequest,
) -> Result<DesignCommandResult, String> {
    let result = add_design_to_project_with_pool(&state.db_pool()?, design_id, request.project_id).await?;
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
    let result = remove_design_from_project_with_pool(&state.db_pool()?, design_id, project_id).await?;
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
        bulk_delete_designs_with_pool(&state.db_pool()?, &request.design_ids, request.delete_files).await?;
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

#[tauri::command]
pub async fn open_design_in_editor(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    open_design_in_editor_with_pool(&state.db_pool()?, design_id).await
}

#[tauri::command]
pub async fn open_design_in_explorer(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    open_design_in_explorer_with_pool(&state.db_pool()?, design_id).await
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
#[cfg(test)]
#[path = "designs_tests.rs"]
mod tests;
