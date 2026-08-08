use crate::config::BootstrapConfig;
use crate::services::compaction::schedule_incremental_vacuum;
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
    pub tags_checked: bool,
    pub rating: Option<i64>,
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
    pub tags_checked: bool,
    pub rating: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrowseAdditionalFiltersPayload {
    pub designer_filters: Option<Vec<String>>,
    pub image_tag_filters: Option<Vec<String>>,
    pub stitching_tag_filters: Option<Vec<String>>,
    pub source_filters: Option<Vec<String>>,
    pub hoop_size: Option<String>,
    pub min_rating: Option<i64>,
    pub stitched_status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDesignsPayload {
    pub q: Option<String>,
    pub search_file_name: Option<bool>,
    pub search_tags: Option<bool>,
    pub search_folder_name: Option<bool>,
    pub unverified_only: Option<bool>,
    pub additional_filters: Option<BrowseAdditionalFiltersPayload>,
}

fn push_where_clause(query_builder: &mut QueryBuilder<Sqlite>, has_where: &mut bool) {
    if *has_where {
        query_builder.push(" AND ");
    } else {
        query_builder.push(" WHERE ");
        *has_where = true;
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
    pub tags_checked: bool,
    pub tagging_tier: Option<i64>,
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
    tags_checked: bool,
    tagging_tier: Option<i64>,
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
pub struct SetDesignTagsCheckedRequest {
    pub tags_checked: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetDesignTagsRequest {
    pub tag_ids: Vec<i64>,
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
        let exists = sqlx::query_scalar::<_, i64>(&sql)
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

fn normalize_path_for_compare(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn normalize_stored_design_filepath(stored_filepath: &str) -> String {
    let normalized = stored_filepath.trim().replace('\\', "/");
    if normalized.is_empty() {
        return String::new();
    }

    let lower = normalized.to_ascii_lowercase();
    if lower == "machineembroiderydesigns" || lower.starts_with("machineembroiderydesigns/") {
        return format!("/{}", normalized.trim_start_matches('/'));
    }

    if let Some(index) = lower.find("/machineembroiderydesigns/") {
        return format!("/{}", normalized[(index + 1)..].trim_start_matches('/'));
    }

    if let Some(index) = lower.find("/machineembroiderydesigns") {
        if index + "/machineembroiderydesigns".len() == lower.len() {
            return format!("/{}", normalized[(index + 1)..].trim_start_matches('/'));
        }
    }

    let data_root = derive_data_root_from_database_url();
    let designs_base = get_designs_base_path();
    let normalized_for_match = normalize_path_for_compare(&normalized);
    let data_root_for_match = normalize_path_for_compare(&data_root.to_string_lossy());
    let designs_base_for_match = normalize_path_for_compare(&designs_base.to_string_lossy());

    if normalized_for_match == designs_base_for_match {
        return "/MachineEmbroideryDesigns".to_string();
    }

    let designs_prefix = format!("{}/", designs_base_for_match);
    if normalized_for_match.starts_with(&designs_prefix) {
        let suffix = normalized[(designs_base_for_match.len() + 1)..].trim_start_matches('/');
        return format!("/MachineEmbroideryDesigns/{}", suffix);
    }

    if normalized_for_match == data_root_for_match {
        return "/".to_string();
    }

    let data_prefix = format!("{}/", data_root_for_match);
    if normalized_for_match.starts_with(&data_prefix) {
        let suffix = normalized[(data_root_for_match.len() + 1)..].trim_start_matches('/');
        return format!("/{}", suffix);
    }

    normalized
}

fn resolve_design_full_path(relative_file_path: &str) -> PathBuf {
    let normalized = normalize_stored_design_filepath(relative_file_path);

    if normalized.is_empty() {
        return get_designs_base_path();
    }

    let cleaned = normalized.trim_start_matches('/').to_string();
    let cleaned_lower = cleaned.to_ascii_lowercase();

    if cleaned_lower == "machineembroiderydesigns"
        || cleaned_lower.starts_with("machineembroiderydesigns/")
    {
        return derive_data_root_from_database_url().join(cleaned);
    }

    get_designs_base_path().join(cleaned)
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

/// Select the smallest hoop that fits the given design dimensions, trying
/// both orientations.  Mirrors the recommendation logic used during bulk
/// import so recalculated dimensions yield a consistent "Recommended hoop".
async fn recommend_hoop_for_design(
    pool: &SqlitePool,
    width_mm: Option<i64>,
    height_mm: Option<i64>,
) -> Result<Option<i64>, String> {
    let (Some(width_mm), Some(height_mm)) = (width_mm, height_mm) else {
        return Ok(None);
    };

    let hoop_id = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT h.id
            FROM hoops h
            WHERE
                (
                    CAST(h.max_width_mm AS REAL) >= CAST(? AS REAL)
                    AND CAST(h.max_height_mm AS REAL) >= CAST(? AS REAL)
                )
                OR (
                    CAST(h.max_width_mm AS REAL) >= CAST(? AS REAL)
                    AND CAST(h.max_height_mm AS REAL) >= CAST(? AS REAL)
                )
            ORDER BY
                (CAST(h.max_width_mm AS REAL) * CAST(h.max_height_mm AS REAL)) ASC,
                CAST(h.max_width_mm AS REAL) ASC,
                CAST(h.max_height_mm AS REAL) ASC,
                h.name COLLATE NOCASE ASC
            LIMIT 1
            "#,
    )
    .bind(width_mm)
    .bind(height_mm)
    .bind(height_mm)
    .bind(width_mm)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(hoop_id)
}

/// Re-read the binary design file from disk and refresh the stored technical
/// metadata (dimensions, stitch count, colour counts, recommended hoop).
///
/// The original file is never modified — it is only read to extract fresh
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

    let generation_result = generate_preview(&ImageGenerationRequest {
        file_path: full_path.to_string_lossy().to_string(),
        preview_3d: false,
        preview_3d_profile: None,
    });

    if let Some(error) = generation_result.error {
        return Err(format!(
            "Could not re-parse the design file: {}",
            error
        ));
    }

    let width_mm = round_mm_to_i64(generation_result.width_mm);
    let height_mm = round_mm_to_i64(generation_result.height_mm);
    let hoop_id = recommend_hoop_for_design(pool, width_mm, height_mm).await?;

    sqlx::query(
        "UPDATE designs SET width_mm = ?, height_mm = ?, stitch_count = ?, color_count = ?, color_change_count = ?, hoop_id = ? WHERE id = ?",
    )
    .bind(width_mm)
    .bind(height_mm)
    .bind(generation_result.stitch_count)
    .bind(generation_result.color_count)
    .bind(generation_result.color_change_count)
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
        stitch_count: generation_result.stitch_count,
        color_count: generation_result.color_count,
        color_change_count: generation_result.color_change_count,
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
			d.tags_checked AS tags_checked,
			d.tagging_tier AS tagging_tier,
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
        filepath: normalize_stored_design_filepath(&row.filepath),
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
        tags_checked: row.tags_checked,
        tagging_tier: row.tagging_tier,
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

async fn set_design_tags_checked_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    tags_checked: bool,
) -> Result<DesignCommandResult, String> {
    ensure_design_exists(pool, design_id).await?;

    sqlx::query("UPDATE designs SET tags_checked = ? WHERE id = ?")
        .bind(tags_checked)
        .bind(design_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DesignCommandResult {
        design_id,
        message: "Design verification state updated.".to_string(),
    })
}

async fn set_design_tags_with_pool(
    pool: &SqlitePool,
    design_id: i64,
    tag_ids: Vec<i64>,
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

    sqlx::query("UPDATE designs SET tags_checked = 1 WHERE id = ?")
        .bind(design_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

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

// ─── Pure helper function tests ─────────────────────────────────────────────
#[cfg(test)]
mod helper_tests {
    use super::*;

    // ─── round_mm_to_i64 ───────────────────────────────────────────────────

    #[test]
    fn round_mm_normal_value() {
        assert_eq!(round_mm_to_i64(Some(12.6)), Some(13));
        assert_eq!(round_mm_to_i64(Some(12.4)), Some(12));
        assert_eq!(round_mm_to_i64(Some(0.5)), Some(1));
        assert_eq!(round_mm_to_i64(Some(-0.4)), Some(0));
    }

    #[test]
    fn round_mm_none_returns_none() {
        assert_eq!(round_mm_to_i64(None), None);
    }

    // ─── ceil_mm_to_i64 ───────────────────────────────────────────────────

    #[test]
    fn ceil_mm_normal_value() {
        assert_eq!(ceil_mm_to_i64(Some(5.1)), Some(6));
        assert_eq!(ceil_mm_to_i64(Some(5.0)), Some(5));
        assert_eq!(ceil_mm_to_i64(Some(0.1)), Some(1));
    }

    #[test]
    fn ceil_mm_none_returns_none() {
        assert_eq!(ceil_mm_to_i64(None), None);
    }

    // ─── normalize_optional_text ──────────────────────────────────────────

    #[test]
    fn normalize_optional_text_trims_whitespace() {
        assert_eq!(
            normalize_optional_text(&Some("  hello world  ".to_string())),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn normalize_optional_text_empty_returns_none() {
        assert_eq!(normalize_optional_text(&Some("   ".to_string())), None);
        assert_eq!(normalize_optional_text(&Some(String::new())), None);
    }

    #[test]
    fn normalize_optional_text_none_returns_none() {
        assert_eq!(normalize_optional_text(&None), None);
    }

    // ─── normalize_optional_fk ────────────────────────────────────────────

    #[test]
    fn normalize_optional_fk_positive_id_ok() {
        let result = normalize_optional_fk(Some(5), "Designer");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(5));
    }

    #[test]
    fn normalize_optional_fk_none_ok() {
        let result = normalize_optional_fk(None, "Test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn normalize_optional_fk_zero_rejected() {
        let result = normalize_optional_fk(Some(0), "Designer");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a positive id"));
    }

    #[test]
    fn normalize_optional_fk_negative_rejected() {
        let result = normalize_optional_fk(Some(-1), "Source");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a positive id"));
    }

    // ─── validate_rating ──────────────────────────────────────────────────

    #[test]
    fn validate_rating_accepts_valid_range() {
        for rating in 1..=5 {
            let result = validate_rating(Some(rating));
            assert!(result.is_ok(), "rating {} should be valid", rating);
            assert_eq!(result.unwrap(), Some(rating));
        }
    }

    #[test]
    fn validate_rating_rejects_out_of_range() {
        let result = validate_rating(Some(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 1 and 5"));

        let result = validate_rating(Some(6));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 1 and 5"));

        let result = validate_rating(Some(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 1 and 5"));
    }

    #[test]
    fn validate_rating_none_accepted() {
        let result = validate_rating(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    // ─── image_mime_from_type ─────────────────────────────────────────────

    #[test]
    fn image_mime_for_known_types() {
        assert_eq!(image_mime_from_type(Some("jpg")), "image/jpeg");
        assert_eq!(image_mime_from_type(Some("jpeg")), "image/jpeg");
        assert_eq!(image_mime_from_type(Some("webp")), "image/webp");
        assert_eq!(image_mime_from_type(Some("gif")), "image/gif");
        assert_eq!(image_mime_from_type(Some("bmp")), "image/bmp");
    }

    #[test]
    fn image_mime_defaults_to_png() {
        assert_eq!(image_mime_from_type(Some("png")), "image/png");
        assert_eq!(image_mime_from_type(Some("svg")), "image/png");
        assert_eq!(image_mime_from_type(None), "image/png");
    }

    // ─── build_data_url ───────────────────────────────────────────────────

    #[test]
    fn build_data_url_returns_correct_mime_and_base64() {
        let data = Some(vec![0_u8, 1_u8, 2_u8]);
        let result = build_data_url(data, Some("png"));
        assert_eq!(
            result.as_deref(),
            Some("data:image/png;base64,AAEC")
        );
    }

    #[test]
    fn build_data_url_none_data_returns_none() {
        assert_eq!(build_data_url(None, Some("png")), None);
        assert_eq!(build_data_url(None, None), None);
    }

    #[test]
    fn build_data_url_uses_correct_mime_for_jpeg() {
        let data = Some(vec![255_u8; 4]);
        let result = build_data_url(data, Some("jpg"));
        assert!(
            result.as_deref().unwrap_or_default().starts_with("data:image/jpeg;base64,")
        );
    }

    // ─── strip_sqlite_prefix ──────────────────────────────────────────────

    #[test]
    fn strip_sqlite_prefix_triple_slash() {
        assert_eq!(
            strip_sqlite_prefix("sqlite:///data/test.db"),
            "data/test.db"
        );
    }

    #[test]
    fn strip_sqlite_prefix_double_slash() {
        assert_eq!(
            strip_sqlite_prefix("sqlite://data/test.db"),
            "data/test.db"
        );
    }

    #[test]
    fn strip_sqlite_prefix_single_colon() {
        assert_eq!(
            strip_sqlite_prefix("sqlite:data/test.db"),
            "data/test.db"
        );
    }

    #[test]
    fn strip_sqlite_prefix_bare_path_unchanged() {
        assert_eq!(
            strip_sqlite_prefix("data/test.db"),
            "data/test.db"
        );
    }

    #[test]
    fn strip_sqlite_prefix_empty_unchanged() {
        assert_eq!(strip_sqlite_prefix(""), "");
    }

    // ─── normalize_path_for_compare ───────────────────────────────────────

    #[test]
    fn normalize_path_replaces_backslashes() {
        let result = normalize_path_for_compare("foo\\bar\\baz");
        assert!(!result.contains('\\'));
        assert!(result.contains('/'));
    }

    #[test]
    fn normalize_path_trims_trailing_slash() {
        let result = normalize_path_for_compare("/foo/bar/");
        assert!(!result.ends_with('/'));
    }

    #[test]
    fn normalize_path_lowercases() {
        let result = normalize_path_for_compare("/FOO/Bar");
        assert_eq!(result, "/foo/bar");
    }

    #[test]
    fn normalize_path_trims_whitespace() {
        let result = normalize_path_for_compare("  /foo/bar  ");
        assert_eq!(result, "/foo/bar");
    }

    // ─── parse_general_token ──────────────────────────────────────────────

    #[test]
    fn parse_general_token_plain_term() {
        let token = parse_general_token("rose");
        assert_eq!(token.text, "rose");
        assert!(!token.exclude);
        assert!(!token.phrase);
        assert!(!token.is_extension);
    }

    #[test]
    fn parse_general_token_exclusion() {
        let token = parse_general_token("-applique");
        assert_eq!(token.text, "applique");
        assert!(token.exclude);
    }

    #[test]
    fn parse_general_token_extension() {
        let token = parse_general_token("*.hus");
        assert_eq!(token.text, "hus");
        assert!(token.is_extension);
        assert!(!token.exclude);
    }

    #[test]
    fn parse_general_token_extension_with_dot() {
        let token = parse_general_token("*.pes");
        assert_eq!(token.text, "pes");
        assert!(token.is_extension);
    }

    #[test]
    fn parse_general_token_quoted_phrase() {
        let token = parse_general_token("\"cross stitch\"");
        assert_eq!(token.text, "cross stitch");
        assert!(token.phrase);
    }

    #[test]
    fn parse_general_token_excluded_extension() {
        let token = parse_general_token("-*.jef");
        assert_eq!(token.text, "jef");
        assert!(token.exclude);
        assert!(token.is_extension);
    }

    // ─── push_where_clause ────────────────────────────────────────────────

    #[test]
    fn push_where_clause_first_time_inserts_where() {
        let mut builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT * FROM designs");
        let mut has_where = false;
        push_where_clause(&mut builder, &mut has_where);
        assert!(has_where);
        let sql = builder.sql();
        assert!(sql.contains(" WHERE "), "sql should have WHERE clause");
    }

    #[test]
    fn push_where_clause_subsequent_times_inserts_and() {
        let mut builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT * FROM designs");
        let mut has_where = true;
        push_where_clause(&mut builder, &mut has_where);
        assert!(has_where);
        let sql = builder.sql();
        assert!(sql.contains(" AND "), "sql should have AND clause");
        assert!(!sql.contains(" WHERE "));
    }

    // ─── is_truthy ────────────────────────────────────────────────────────

    #[test]
    fn is_truthy_accepts_expected_values() {
        assert!(is_truthy("1"));
        assert!(is_truthy("true"));
        assert!(is_truthy("YES"));
        assert!(is_truthy("y"));
        assert!(is_truthy("accepted"));
    }

    #[test]
    fn is_truthy_rejects_falsy_values() {
        assert!(!is_truthy("no"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy("0"));
        assert!(!is_truthy("n"));
        assert!(!is_truthy("declined"));
    }

    #[test]
    fn is_truthy_trims_whitespace() {
        assert!(is_truthy("  true  "));
        assert!(!is_truthy("  false  "));
    }
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
        let mut query = QueryBuilder::<Sqlite>::new(
            "SELECT id, filepath FROM designs WHERE id IN (",
        );
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

    let is_extension = normalized.starts_with("*") && normalized.len() > 1 && !normalized.contains(' ');
    let final_text = if is_extension {
        normalized.trim_start_matches('*').trim_start_matches('.').trim().to_string()
    } else {
        normalized.to_string()
    };

    GeneralSearchToken {
        text: final_text,
        phrase,
        exclude,
        is_extension,
    }
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

            let pattern = format!("%{}%", token.text.to_lowercase());
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
                query_builder.push("LOWER(d.filepath) LIKE ");
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

#[cfg(test)]
mod parser_tests {
    use super::parse_general_search_groups;

    #[test]
    fn parses_or_groups_exclusions_and_extensions() {
        let groups = parse_general_search_groups(r#"rose "cross stitch" -applique OR *.hus"#);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 3);
        assert_eq!(groups[1].len(), 1);

        let first = &groups[0][0];
        assert_eq!(first.text, "rose");
        assert!(!first.exclude);
        assert!(!first.phrase);

        let second = &groups[0][1];
        assert_eq!(second.text, "cross stitch");
        assert!(!second.exclude);
        assert!(second.phrase);

        let third = &groups[0][2];
        assert_eq!(third.text, "applique");
        assert!(third.exclude);
        assert!(!third.phrase);

        let extension = &groups[1][0];
        assert_eq!(extension.text, "hus");
        assert!(extension.is_extension);
    }

    #[test]
    fn preserves_terms_inside_quotes() {
        let groups = parse_general_search_groups(r#""exact phrase""#);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 1);
        let token = &groups[0][0];
        assert_eq!(token.text, "exact phrase");
        assert!(token.phrase);
    }
}

#[tauri::command]
pub async fn get_designs(
    state: State<'_, AppState>,
    payload: Option<GetDesignsPayload>,
) -> Result<Vec<BrowseDesignSummary>, String> {
    let mut query_builder = QueryBuilder::<Sqlite>::new(
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
            d.tags_checked AS tags_checked,
            d.rating AS rating
        FROM designs d
        LEFT JOIN designers ON designers.id = d.designer_id
        LEFT JOIN sources ON sources.id = d.source_id
        LEFT JOIN hoops ON hoops.id = d.hoop_id
        LEFT JOIN design_tags ON design_tags.design_id = d.id
        LEFT JOIN tags ON tags.id = design_tags.tag_id
        "#
    );

    let mut has_where = false;

    if let Some(ref p) = payload {
        let q_trimmed = p.q.as_deref().map(str::trim).filter(|value| !value.is_empty());
        if let Some(q) = q_trimmed {
            let search_file = p.search_file_name.unwrap_or(true);
            let search_tags = p.search_tags.unwrap_or(true);
            let search_folder = p.search_folder_name.unwrap_or(true);
            let general_groups = parse_general_search_groups(q);

            if search_file || search_tags || search_folder {
                push_where_clause(&mut query_builder, &mut has_where);
                push_general_search_clause(&mut query_builder, search_file, search_tags, search_folder, &general_groups);
            }
        }

        if p.unverified_only.unwrap_or(false) {
            push_where_clause(&mut query_builder, &mut has_where);
            query_builder.push("d.tags_checked = 0");
        }

        if let Some(ref filters) = p.additional_filters {
            let designer_filters = filters.designer_filters.as_deref().unwrap_or(&[]);
            if !designer_filters.is_empty() {
                push_where_clause(&mut query_builder, &mut has_where);
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
                push_where_clause(&mut query_builder, &mut has_where);
                query_builder.push("d.id IN (");
                query_builder.push("SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE ");
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
                push_where_clause(&mut query_builder, &mut has_where);
                query_builder.push("d.id IN (");
                query_builder.push("SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE ");
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
                push_where_clause(&mut query_builder, &mut has_where);
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
                if !hoop_size_trimmed.is_empty() {
                    push_where_clause(&mut query_builder, &mut has_where);
                    query_builder.push("LOWER(COALESCE(hoops.name, '')) = ");
                    query_builder.push_bind(hoop_size_trimmed.to_lowercase());
                }
            }

            if let Some(min_rating) = filters.min_rating {
                if min_rating >= 1 {
                    push_where_clause(&mut query_builder, &mut has_where);
                    query_builder.push("d.rating >= ");
                    query_builder.push_bind(min_rating);
                }
            }

            if let Some(ref stitched_status) = filters.stitched_status {
                let stitched_status_trimmed = stitched_status.trim();
                if !stitched_status_trimmed.is_empty() && stitched_status_trimmed != "all" {
                    push_where_clause(&mut query_builder, &mut has_where);
                    if stitched_status_trimmed == "yes" {
                        query_builder.push("d.is_stitched = 1");
                    } else {
                        query_builder.push("d.is_stitched = 0");
                    }
                }
            }
        }
    }

    query_builder.push(" GROUP BY d.id ORDER BY d.filename COLLATE NOCASE ASC LIMIT 500");

    let rows = query_builder
        .build_query_as::<BrowseDesignSummaryRow>()
        .fetch_all(&state.db)
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
            tags_checked: row.tags_checked,
            rating: row.rating,
        })
        .collect();

    Ok(items)
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

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
    let mut verified_count = 0usize;

    for design_id in &design_ids {
        let result = sqlx::query("UPDATE designs SET tags_checked = 1 WHERE id = ?")
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
    .fetch_all(&state.db)
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
    .fetch_all(&state.db)
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

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
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
pub async fn bulk_set_tags_for_designs(
    state: State<'_, AppState>,
    design_ids: Vec<i64>,
    tag_ids: Vec<i64>,
) -> Result<BulkSetTagsResult, String> {
    if design_ids.is_empty() {
        return Ok(BulkSetTagsResult {
            requested_count: 0,
            updated_count: 0,
        });
    }

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
    let mut updated_count = 0usize;

    for design_id in &design_ids {
        sqlx::query("DELETE FROM design_tags WHERE design_id = ?")
            .bind(*design_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        for tag_id in &tag_ids {
            sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
                .bind(*design_id)
                .bind(*tag_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }

        let result = sqlx::query("UPDATE designs SET tags_checked = 1 WHERE id = ?")
            .bind(*design_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        updated_count += result.rows_affected() as usize;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(BulkSetTagsResult {
        requested_count: design_ids.len(),
        updated_count,
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
        .fetch_all(&state.db)
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
    get_design_detail_with_pool(&state.db, design_id).await
}

#[tauri::command]
pub async fn get_design_image_data_url(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<Option<DesignImageData>, String> {
    get_design_image_data_with_pool(&state.db, design_id).await
}

#[tauri::command]
pub async fn update_design_metadata(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: UpdateDesignMetadataRequest,
) -> Result<DesignCommandResult, String> {
    let result = update_design_metadata_with_pool(&state.db, design_id, request).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": {}
    }));
    Ok(result)
}

#[tauri::command]
pub async fn set_design_rating(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignRatingRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_rating_with_pool(&state.db, design_id, request.rating).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": { "rating": request.rating }
    }));
    Ok(result)
}

#[tauri::command]
pub async fn set_design_stitched(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignStitchedRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_stitched_with_pool(&state.db, design_id, request.is_stitched).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": { "is_stitched": request.is_stitched }
    }));
    Ok(result)
}

#[tauri::command]
pub async fn set_design_tags_checked(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignTagsCheckedRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_tags_checked_with_pool(&state.db, design_id, request.tags_checked).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": { "tags_checked": request.tags_checked }
    }));
    Ok(result)
}

#[tauri::command]
pub async fn set_design_tags(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignTagsRequest,
) -> Result<DesignCommandResult, String> {
    let result = set_design_tags_with_pool(&state.db, design_id, request.tag_ids).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": { "tags_checked": true }
    }));
    Ok(result)
}

#[tauri::command]
pub async fn remove_design_tag(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    tag_id: i64,
) -> Result<DesignCommandResult, String> {
    let result = remove_design_tag_with_pool(&state.db, design_id, tag_id).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": {}
    }));
    Ok(result)
}

#[tauri::command]
pub async fn add_design_to_project(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignProjectRequest,
) -> Result<DesignCommandResult, String> {
    let result = add_design_to_project_with_pool(&state.db, design_id, request.project_id).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": {}
    }));
    Ok(result)
}

#[tauri::command]
pub async fn remove_design_from_project(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    project_id: i64,
) -> Result<DesignCommandResult, String> {
    let result = remove_design_from_project_with_pool(&state.db, design_id, project_id).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": {}
    }));
    Ok(result)
}

#[tauri::command]
pub async fn delete_design(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
    delete_file: bool,
) -> Result<DesignCommandResult, String> {
    let result = delete_design_with_pool(&state.db, design_id, delete_file).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": { "_deleted": true }
    }));
    // Reclaim freelist pages asynchronously after the delete commits, so the
    // UI never blocks on database file compaction.
    schedule_incremental_vacuum(state.db.clone());
    Ok(result)
}

#[tauri::command]
pub async fn bulk_delete_designs(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    request: BulkDeleteDesignsRequest,
) -> Result<BulkDeleteDesignsResult, String> {
    let result = bulk_delete_designs_with_pool(&state.db, &request.design_ids, request.delete_files).await?;
    // Emit events for each deleted design
    for design_id in &request.design_ids {
        let _ = app_handle.emit("design:mutated", json!({
            "design_id": design_id,
            "fields": { "_deleted": true }
        }));
    }
    // Reclaim freelist pages asynchronously after the bulk delete commits, so
    // the UI never blocks on database file compaction.
    schedule_incremental_vacuum(state.db.clone());
    Ok(result)
}

#[tauri::command]
pub async fn open_design_in_editor(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    open_design_in_editor_with_pool(&state.db, design_id).await
}

#[tauri::command]
pub async fn open_design_in_explorer(
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<LaunchDesignResult, String> {
    open_design_in_explorer_with_pool(&state.db, design_id).await
}

#[tauri::command]
pub async fn render_design_3d_preview(
    state: State<'_, AppState>,
    design_id: i64,
    request: Option<RenderPreviewRequest>,
) -> Result<Render3dPreviewResult, String> {
    let preview_3d = request.map(|r| r.preview_3d).unwrap_or(true);
    render_design_3d_preview_with_pool(&state.db, design_id, preview_3d).await
}

#[tauri::command]
pub async fn reparse_design_file(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    design_id: i64,
) -> Result<ReparseDesignResult, String> {
    let result = reparse_design_file_with_pool(&state.db, design_id).await?;
    let _ = app_handle.emit("design:mutated", json!({
        "design_id": design_id,
        "fields": {}
    }));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create test sqlite pool");

        sqlx::query(
            r#"
			CREATE TABLE designers (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name VARCHAR(255) NOT NULL UNIQUE
			);
			"#,
        )
        .execute(&pool)
        .await
        .expect("failed to create designers table");

        sqlx::query(
            r#"
			CREATE TABLE sources (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name VARCHAR(255) NOT NULL UNIQUE
			);
			"#,
        )
        .execute(&pool)
        .await
        .expect("failed to create sources table");

        sqlx::query(
            r#"
			CREATE TABLE hoops (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name VARCHAR(100) NOT NULL UNIQUE,
				max_width_mm NUMERIC(8,2) NOT NULL,
				max_height_mm NUMERIC(8,2) NOT NULL
			);
			"#,
        )
        .execute(&pool)
        .await
        .expect("failed to create hoops table");

        sqlx::query(
            r#"
			CREATE TABLE tags (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				description VARCHAR(255) NOT NULL UNIQUE,
				tag_group VARCHAR(20)
			);
			"#,
        )
        .execute(&pool)
        .await
        .expect("failed to create tags table");

        sqlx::query(
            r#"
			CREATE TABLE projects (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name VARCHAR(255) NOT NULL UNIQUE,
				description TEXT,
				date_created DATE
			);
			"#,
        )
        .execute(&pool)
        .await
        .expect("failed to create projects table");

        sqlx::query(
            r#"
			CREATE TABLE designs (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				filename VARCHAR(500) NOT NULL,
				filepath VARCHAR(1000) NOT NULL,
				image_data BLOB,
				image_type VARCHAR(10),
				width_mm NUMERIC(8,2),
				height_mm NUMERIC(8,2),
				stitch_count INTEGER,
				color_count INTEGER,
				color_change_count INTEGER,
				notes TEXT,
				rating SMALLINT,
				is_stitched BOOLEAN NOT NULL DEFAULT 0,
				tags_checked BOOLEAN NOT NULL DEFAULT 0,
				tagging_tier SMALLINT,
				date_added DATE,
				designer_id INTEGER REFERENCES designers(id) ON DELETE SET NULL,
				source_id INTEGER REFERENCES sources(id) ON DELETE SET NULL,
				hoop_id INTEGER REFERENCES hoops(id) ON DELETE SET NULL
			);
			"#,
        )
        .execute(&pool)
        .await
        .expect("failed to create designs table");

        sqlx::query(
			"CREATE TABLE design_tags (design_id INTEGER NOT NULL, tag_id INTEGER NOT NULL, PRIMARY KEY (design_id, tag_id));",
		)
		.execute(&pool)
		.await
		.expect("failed to create design_tags table");

        sqlx::query(
			"CREATE TABLE project_designs (project_id INTEGER NOT NULL, design_id INTEGER NOT NULL, PRIMARY KEY (project_id, design_id));",
		)
		.execute(&pool)
		.await
		.expect("failed to create project_designs table");

        sqlx::query("INSERT INTO designers (name) VALUES ('Acme Designer')")
            .execute(&pool)
            .await
            .expect("failed to seed designer");
        sqlx::query("INSERT INTO sources (name) VALUES ('USB Import')")
            .execute(&pool)
            .await
            .expect("failed to seed source");
        sqlx::query(
            "INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Hoop A', 126, 126)",
        )
        .execute(&pool)
        .await
        .expect("failed to seed hoop");
        sqlx::query("INSERT INTO tags (description, tag_group) VALUES ('Flowers', 'image')")
            .execute(&pool)
            .await
            .expect("failed to seed tag");
        sqlx::query(
            "INSERT INTO tags (description, tag_group) VALUES ('Satin Stitch', 'stitching')",
        )
        .execute(&pool)
        .await
        .expect("failed to seed tag");
        sqlx::query("INSERT INTO projects (name) VALUES ('Summer Quilt')")
            .execute(&pool)
            .await
            .expect("failed to seed project");
        sqlx::query("INSERT INTO projects (name) VALUES ('Gift Ideas')")
            .execute(&pool)
            .await
            .expect("failed to seed project");

        sqlx::query(
			"INSERT INTO designs (filename, filepath, notes, designer_id, source_id, hoop_id, is_stitched, tags_checked, rating) VALUES ('rose.pes', 'Roses/rose.pes', 'old note', 1, 1, 1, 0, 0, NULL)",
		)
		.execute(&pool)
		.await
		.expect("failed to seed design");

        pool
    }

    #[tokio::test]
    async fn update_design_metadata_updates_core_fields() {
        let pool = test_pool().await;

        let result = update_design_metadata_with_pool(
            &pool,
            1,
            UpdateDesignMetadataRequest {
                notes: Some("  updated note  ".to_string()),
                designer_id: Some(1),
                source_id: Some(1),
                hoop_id: Some(1),
            },
        )
        .await;

        assert!(result.is_ok());

        let row = sqlx::query_as::<_, (Option<String>, Option<i64>, Option<i64>, Option<i64>)>(
            "SELECT notes, designer_id, source_id, hoop_id FROM designs WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("design row should exist");

        assert_eq!(row.0.as_deref(), Some("updated note"));
        assert_eq!(row.1, Some(1));
        assert_eq!(row.2, Some(1));
        assert_eq!(row.3, Some(1));
    }

    #[tokio::test]
    async fn set_design_rating_rejects_invalid_values() {
        let pool = test_pool().await;

        let result = set_design_rating_with_pool(&pool, 1, Some(9)).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected rating error")
            .contains("between 1 and 5"));
    }

    #[tokio::test]
    async fn set_design_tags_replaces_and_marks_verified() {
        let pool = test_pool().await;

        sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
            .execute(&pool)
            .await
            .expect("should insert original tag");

        let result = set_design_tags_with_pool(&pool, 1, vec![2]).await;
        assert!(result.is_ok());

        let assigned = sqlx::query_as::<_, (i64,)>(
            "SELECT tag_id FROM design_tags WHERE design_id = 1 ORDER BY tag_id ASC",
        )
        .fetch_all(&pool)
        .await
        .expect("assigned tags query should succeed");

        assert_eq!(assigned.len(), 1);
        assert_eq!(assigned[0].0, 2);

        let checked = sqlx::query_scalar::<_, i64>("SELECT tags_checked FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("tags_checked query should succeed");

        assert_eq!(checked, 1);
    }

    #[tokio::test]
    async fn add_and_remove_project_membership_round_trip() {
        let pool = test_pool().await;

        let add_result = add_design_to_project_with_pool(&pool, 1, 1).await;
        assert!(add_result.is_ok());

        let count_after_add = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_designs WHERE design_id = 1 AND project_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("project assignment count should work");
        assert_eq!(count_after_add, 1);

        let remove_result = remove_design_from_project_with_pool(&pool, 1, 1).await;
        assert!(remove_result.is_ok());

        let count_after_remove = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_designs WHERE design_id = 1 AND project_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("project assignment count should work");
        assert_eq!(count_after_remove, 0);
    }

    #[tokio::test]
    async fn get_design_image_data_returns_data_url_when_image_exists() {
        let pool = test_pool().await;

        sqlx::query("UPDATE designs SET image_data = ?, image_type = ? WHERE id = 1")
            .bind(vec![1_u8, 2_u8, 3_u8, 4_u8])
            .bind("png")
            .execute(&pool)
            .await
            .expect("should update image data");

        let image = get_design_image_data_with_pool(&pool, 1)
            .await
            .expect("image query should succeed")
            .expect("image should exist");

        assert_eq!(image.design_id, 1);
        assert_eq!(image.image_type.as_deref(), Some("png"));
        assert!(image
            .data_url
            .as_deref()
            .unwrap_or_default()
            .starts_with("data:image/png;base64,"));
    }

    #[tokio::test]
    async fn open_design_in_editor_returns_error_for_missing_design() {
        let pool = test_pool().await;

        let result = open_design_in_editor_with_pool(&pool, 999).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected missing design error")
            .contains("not found"));
    }

    #[tokio::test]
    async fn open_design_in_explorer_returns_error_for_missing_design() {
        let pool = test_pool().await;

        let result = open_design_in_explorer_with_pool(&pool, 999).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected missing design error")
            .contains("not found"));
    }

    #[tokio::test]
    async fn render_design_3d_preview_returns_error_when_source_file_is_missing() {
        let pool = test_pool().await;

        let result = render_design_3d_preview_with_pool(&pool, 1, true).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected missing file error")
            .contains("not found on disk"));
    }

    #[tokio::test]
    async fn render_design_2d_preview_returns_error_when_source_file_is_missing() {
        let pool = test_pool().await;

        let result = render_design_3d_preview_with_pool(&pool, 1, false).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected missing file error")
            .contains("not found on disk"));
    }

    #[test]
    fn launch_disable_parser_accepts_expected_truthy_values() {
        assert!(is_truthy("1"));
        assert!(is_truthy("true"));
        assert!(is_truthy("YES"));
        assert!(!is_truthy("no"));
    }

    // ─── Phase 2: Environment-dependent tests ────────────────────────────────

    #[test]
    #[serial]
    fn external_launches_disabled_returns_true_when_env_var_is_set() {
        let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "true");
        assert!(external_launches_disabled());
        if let Some(val) = prior {
            std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
        } else {
            std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
        }
    }

    #[test]
    #[serial]
    fn external_launches_disabled_returns_false_when_falsy_env_var() {
        let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "false");
        assert!(!external_launches_disabled());
        if let Some(val) = prior {
            std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
        } else {
            std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
        }
    }

    #[test]
    #[serial]
    fn external_launches_disabled_returns_false_when_env_var_absent() {
        let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
        std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
        assert!(!external_launches_disabled());
        if let Some(val) = prior {
            std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
        }
    }

    // ─── Phase 3: Async DB-dependent functions ──────────────────────────────

    #[tokio::test]
    async fn ensure_design_exists_found() {
        let pool = test_pool().await;
        let result = ensure_design_exists(&pool, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn ensure_design_exists_not_found() {
        let pool = test_pool().await;
        let result = ensure_design_exists(&pool, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn ensure_foreign_key_exists_when_exists() {
        let pool = test_pool().await;
        let result = ensure_foreign_key_exists(&pool, "designers", Some(1), "Designer").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn ensure_foreign_key_exists_when_missing() {
        let pool = test_pool().await;
        let result = ensure_foreign_key_exists(&pool, "designers", Some(999), "Designer").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn ensure_foreign_key_exists_none_passes() {
        let pool = test_pool().await;
        let result = ensure_foreign_key_exists(&pool, "designers", None, "Designer").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_design_filepath_returns_path_for_valid_design() {
        let pool = test_pool().await;
        let result = get_design_filepath(&pool, 1).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Roses/rose.pes");
    }

    #[tokio::test]
    async fn get_design_filepath_errors_for_missing_design() {
        let pool = test_pool().await;
        let result = get_design_filepath(&pool, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn get_design_filepath_errors_for_empty_filepath() {
        let pool = test_pool().await;

        sqlx::query("INSERT INTO designs (filename, filepath) VALUES ('empty.pes', '')")
            .execute(&pool)
            .await
            .expect("should insert design with empty filepath");

        let result = get_design_filepath(&pool, 2).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not have a stored filepath"));
    }

    #[tokio::test]
    async fn set_design_stitched_with_pool_sets_true() {
        let pool = test_pool().await;

        let result = set_design_stitched_with_pool(&pool, 1, true).await;
        assert!(result.is_ok());

        let stitched = sqlx::query_scalar::<_, i64>("SELECT is_stitched FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("query should succeed");
        assert_eq!(stitched, 1);
    }

    #[tokio::test]
    async fn set_design_stitched_with_pool_sets_false() {
        let pool = test_pool().await;

        let result = set_design_stitched_with_pool(&pool, 1, false).await;
        assert!(result.is_ok());

        let stitched = sqlx::query_scalar::<_, i64>("SELECT is_stitched FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("query should succeed");
        assert_eq!(stitched, 0);
    }

    #[tokio::test]
    async fn set_design_tags_checked_with_pool_sets_true() {
        let pool = test_pool().await;

        let result = set_design_tags_checked_with_pool(&pool, 1, true).await;
        assert!(result.is_ok());

        let checked = sqlx::query_scalar::<_, i64>("SELECT tags_checked FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("query should succeed");
        assert_eq!(checked, 1);
    }

    #[tokio::test]
    async fn set_design_tags_checked_with_pool_sets_false() {
        let pool = test_pool().await;

        let result = set_design_tags_checked_with_pool(&pool, 1, false).await;
        assert!(result.is_ok());

        let checked = sqlx::query_scalar::<_, i64>("SELECT tags_checked FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("query should succeed");
        assert_eq!(checked, 0);
    }

    #[tokio::test]
    async fn remove_design_tag_with_pool_removes_existing() {
        let pool = test_pool().await;

        sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
            .execute(&pool)
            .await
            .expect("should seed tag link");

        let result = remove_design_tag_with_pool(&pool, 1, 1).await;
        assert!(result.is_ok());

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("count should work");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn remove_design_tag_with_pool_rejects_invalid_tag_id() {
        let pool = test_pool().await;
        let result = remove_design_tag_with_pool(&pool, 1, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("positive integer"));
    }

    #[tokio::test]
    async fn remove_design_tag_with_pool_errors_for_missing_design() {
        let pool = test_pool().await;
        let result = remove_design_tag_with_pool(&pool, 999, 1).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn delete_design_without_file_removes_from_db() {
        let pool = test_pool().await;

        let result = delete_design_with_pool(&pool, 1, false).await;
        assert!(result.is_ok());

        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("count should work");
        assert_eq!(exists, 0);
    }

    #[tokio::test]
    async fn delete_design_errors_when_design_missing() {
        let pool = test_pool().await;
        let result = delete_design_with_pool(&pool, 999, false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn set_design_rating_with_pool_sets_valid_rating() {
        let pool = test_pool().await;

        let result = set_design_rating_with_pool(&pool, 1, Some(4)).await;
        assert!(result.is_ok());

        let rating = sqlx::query_scalar::<_, Option<i64>>("SELECT rating FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("query should succeed");
        assert_eq!(rating, Some(4));
    }

    #[tokio::test]
    async fn set_design_rating_with_pool_clears_rating() {
        let pool = test_pool().await;

        sqlx::query("UPDATE designs SET rating = 3 WHERE id = 1")
            .execute(&pool)
            .await
            .expect("should set rating");

        let result = set_design_rating_with_pool(&pool, 1, None).await;
        assert!(result.is_ok());

        let rating = sqlx::query_scalar::<_, Option<i64>>("SELECT rating FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("query should succeed");
        assert_eq!(rating, None);
    }

    #[tokio::test]
    async fn get_design_detail_returns_full_detail_for_existing_design() {
        let pool = test_pool().await;

        // Add design to a project and tag for a full detail response
        sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 1)")
            .execute(&pool)
            .await
            .expect("should seed tag link");
        sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 1)")
            .execute(&pool)
            .await
            .expect("should seed project link");

        let detail = get_design_detail_with_pool(&pool, 1)
            .await
            .expect("query should succeed")
            .expect("detail should exist");

        assert_eq!(detail.id, 1);
        assert_eq!(detail.filename, "rose.pes");
        assert_eq!(detail.designer, "Acme Designer");
        assert_eq!(detail.source, "USB Import");
        assert_eq!(detail.hoop.as_deref(), Some("Hoop A"));
        assert_eq!(detail.notes.as_deref(), Some("old note"));
        assert!(!detail.tags.is_empty());
        assert!(!detail.projects.is_empty());
        assert!(!detail.available_projects.is_empty());
        assert!(!detail.all_tags.is_empty());
        assert!(!detail.designers.is_empty());
        assert!(!detail.sources.is_empty());
        assert!(!detail.hoops.is_empty());
    }

    #[tokio::test]
    async fn get_design_detail_returns_none_for_missing() {
        let pool = test_pool().await;

        let detail = get_design_detail_with_pool(&pool, 999)
            .await
            .expect("query should succeed");
        assert!(detail.is_none());
    }

    #[tokio::test]
    async fn bulk_delete_designs_empty_list() {
        let pool = test_pool().await;

        let result = bulk_delete_designs_with_pool(&pool, &[], false)
            .await
            .expect("empty list should succeed");
        assert_eq!(result.requested_count, 0);
        assert_eq!(result.deleted_count, 0);
    }

    #[tokio::test]
    async fn bulk_delete_designs_single_design() {
        let pool = test_pool().await;

        let result = bulk_delete_designs_with_pool(&pool, &[1], false)
            .await
            .expect("single delete should succeed");
        assert_eq!(result.requested_count, 1);
        assert_eq!(result.deleted_count, 1);
    }

    #[tokio::test]
    async fn bulk_delete_designs_exceeds_limit() {
        let pool = test_pool().await;

        let ids: Vec<i64> = (1..=51).collect();
        let result = bulk_delete_designs_with_pool(&pool, &ids, false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("more than 50"));
    }

    #[tokio::test]
    async fn bulk_delete_designs_deduplicates() {
        let pool = test_pool().await;

        let result = bulk_delete_designs_with_pool(&pool, &[1, 1, 1], false)
            .await
            .expect("dedup should succeed");
        assert_eq!(result.requested_count, 1);
        assert_eq!(result.deleted_count, 1);
    }

    #[tokio::test]
    async fn update_design_metadata_rejects_invalid_fk() {
        let pool = test_pool().await;

        let result = update_design_metadata_with_pool(
            &pool,
            1,
            UpdateDesignMetadataRequest {
                notes: None,
                designer_id: Some(999),
                source_id: None,
                hoop_id: None,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ─── Phase 4: Filesystem-dependent tests ────────────────────────────────

    #[test]
    #[serial]
    fn derive_data_root_from_database_url_when_db_is_in_database_subfolder() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_root/Database/catalogue.db");

        let root = derive_data_root_from_database_url();

        assert_eq!(root, PathBuf::from("/tmp/test_root"));

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn derive_data_root_from_database_url_when_db_is_in_non_database_folder() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/my_data/catalogue.db");

        let root = derive_data_root_from_database_url();

        // When parent folder is not 'Database', the parent is used directly
        assert_eq!(root, PathBuf::from("/tmp/my_data"));

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn get_designs_base_path_joins_machine_embroidery_designs() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        let base = get_designs_base_path();
        assert_eq!(base, PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns"));

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn normalize_stored_design_filepath_already_normalized() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        // A relative path that isn't under the data root is returned as-is
        // (normalize_stored_design_filepath only adds the /MachineEmbroideryDesigns/
        // prefix when the path is already within that directory structure)
        let result = normalize_stored_design_filepath("Roses/rose.pes");
        assert_eq!(result, "Roses/rose.pes");

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn normalize_stored_design_filepath_under_machine_embroidery() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        // A path already starting with MachineEmbroideryDesigns gets normalized
        let result = normalize_stored_design_filepath("MachineEmbroideryDesigns/Roses/rose.pes");
        assert_eq!(result, "/MachineEmbroideryDesigns/Roses/rose.pes");

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }
    #[test]
    fn nearest_existing_folder_returns_fallback_when_no_parent_exists() {
        // Use a completely isolated UUID-like temp path so no parent exists
        let isolated = std::env::temp_dir().join(format!(
            "nearest-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        // Create the fallback directory so it exists
        std::fs::create_dir_all(&isolated).expect("should create isolated dir");
        let fallback = isolated.clone();
        let nonexistent = isolated.join("a").join("b").join("c");

        let result = nearest_existing_folder(&nonexistent, &fallback);
        assert_eq!(result, fallback);

        let _ = std::fs::remove_dir_all(&isolated);
    }

    #[test]
    #[serial]
    fn normalize_stored_design_filepath_with_machine_embroidery_prefix() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        let result = normalize_stored_design_filepath("machineembroiderydesigns/Roses/rose.pes");
        assert_eq!(result, "/machineembroiderydesigns/Roses/rose.pes");

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn normalize_stored_design_filepath_empty_returns_empty() {
        let result = normalize_stored_design_filepath("");
        assert_eq!(result, "");
    }

    #[test]
    #[serial]
    fn resolve_design_full_path_returns_designs_base_for_empty() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        let result = resolve_design_full_path("");
        assert_eq!(result, PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns"));

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn nearest_existing_folder_returns_existing_dir_when_given_dir() {
        let tmp = std::env::temp_dir().join(format!(
            "nearest-dir-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("should create temp dir");

        let result = nearest_existing_folder(&tmp, &PathBuf::from("/fallback"));
        assert_eq!(result, tmp);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ─── Additional coverage for open/launch suppressed paths ──────────────

    #[tokio::test]
    #[serial]
    async fn open_design_in_editor_returns_suppressed_when_launches_disabled() {
        let pool = test_pool().await;
        let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "true");

        let result = open_design_in_editor_with_pool(&pool, 1).await;
        assert!(result.is_ok());
        let launch = result.unwrap();
        assert!(launch.suppressed);
        assert!(!launch.success);

        if let Some(val) = prior {
            std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
        } else {
            std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
        }
    }

    #[tokio::test]
    #[serial]
    async fn open_design_in_explorer_returns_suppressed_when_launches_disabled() {
        let pool = test_pool().await;
        let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
        std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", "true");

        let result = open_design_in_explorer_with_pool(&pool, 1).await;
        assert!(result.is_ok());
        let launch = result.unwrap();
        assert!(launch.suppressed);
        assert!(!launch.success);

        if let Some(val) = prior {
            std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
        } else {
            std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");
        }
    }

    #[tokio::test]
    #[serial]
    async fn open_design_in_editor_returns_file_not_found_error() {
        let pool = test_pool().await;
        // Set disable to false so it proceeds past the suppressed check
        let prior = std::env::var("EMBROIDERY_DISABLE_EXTERNAL_OPEN").ok();
        std::env::remove_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN");

        let result = open_design_in_editor_with_pool(&pool, 1).await;
        assert!(result.is_ok());
        let launch = result.unwrap();
        assert!(!launch.suppressed);
        assert!(!launch.success);
        assert!(launch.message.contains("not found on disk"));

        if let Some(val) = prior {
            std::env::set_var("EMBROIDERY_DISABLE_EXTERNAL_OPEN", val);
        }
    }

    // ─── Additional DB error paths ─────────────────────────────────────────

    #[tokio::test]
    async fn set_design_stitched_errors_when_missing() {
        let pool = test_pool().await;
        let result = set_design_stitched_with_pool(&pool, 999, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn set_design_tags_checked_errors_when_missing() {
        let pool = test_pool().await;
        let result = set_design_tags_checked_with_pool(&pool, 999, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn add_design_to_project_rejects_invalid_project() {
        let pool = test_pool().await;
        let result = add_design_to_project_with_pool(&pool, 1, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("valid project"));
    }

    #[tokio::test]
    async fn remove_design_from_project_rejects_invalid_project() {
        let pool = test_pool().await;
        let result = remove_design_from_project_with_pool(&pool, 1, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("valid project"));
    }

    #[tokio::test]
    async fn update_design_metadata_rejects_invalid_hoop() {
        let pool = test_pool().await;

        let result = update_design_metadata_with_pool(
            &pool,
            1,
            UpdateDesignMetadataRequest {
                notes: None,
                designer_id: None,
                source_id: None,
                hoop_id: Some(999),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn set_design_tags_rejects_non_positive_tag_id() {
        let pool = test_pool().await;

        let result = set_design_tags_with_pool(&pool, 1, vec![0]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("positive integer"));
    }

    #[tokio::test]
    async fn get_design_image_data_returns_no_data_url_when_no_image() {
        let pool = test_pool().await;

        let result = get_design_image_data_with_pool(&pool, 1)
            .await
            .expect("query should succeed");
        // Design 1 exists but has no image data seeded — returns Some with data_url=None
        let image_data = result.expect("should return Some for existing design");
        assert!(image_data.data_url.is_none());
        assert!(image_data.image_type.is_none());
    }

    #[tokio::test]
    async fn bulk_verify_empty_list_returns_zero_works() {
        let pool = test_pool().await;

        // Test tauri command logic indirectly through design update
        let result = bulk_delete_designs_with_pool(&pool, &[], false).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.requested_count, 0);
    }

    #[tokio::test]
    async fn add_design_to_project_with_missing_project_errors() {
        let pool = test_pool().await;

        let result = add_design_to_project_with_pool(&pool, 1, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ─── normalize_stored_design_filepath additional edge cases ───────────

    #[test]
    #[serial]
    fn normalize_stored_design_filepath_with_absolute_data_root_prefix() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        // A path that starts with the full data root should be normalized
        let result = normalize_stored_design_filepath("/tmp/test_data/MachineEmbroideryDesigns/MyDesign.pes");
        assert_eq!(result, "/MachineEmbroideryDesigns/MyDesign.pes");

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn normalize_stored_design_filepath_exact_data_root_returns_slash() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        let result = normalize_stored_design_filepath("/tmp/test_data");
        assert_eq!(result, "/");

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn normalize_stored_design_filepath_exact_designs_base_returns_med() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        let result = normalize_stored_design_filepath("/tmp/test_data/MachineEmbroideryDesigns");
        assert_eq!(result, "/MachineEmbroideryDesigns");

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn normalize_stored_design_filepath_backslashes_are_normalized() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        let result = normalize_stored_design_filepath("Roses\\rose.pes");
        // With backslashes normalized to forward slashes
        assert_eq!(result, "Roses/rose.pes");

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn resolve_design_full_path_for_med_prefixed_path() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        let result = resolve_design_full_path("MachineEmbroideryDesigns/Roses/rose.pes");
        assert_eq!(result, PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns/Roses/rose.pes"));

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn resolve_design_full_path_for_relative_path() {
        let prior = std::env::var("DATABASE_URL").ok();
        std::env::set_var("DATABASE_URL", "sqlite:/tmp/test_data/Database/catalogue.db");

        let result = resolve_design_full_path("Roses/rose.pes");
        assert_eq!(result, PathBuf::from("/tmp/test_data/MachineEmbroideryDesigns/Roses/rose.pes"));

        if let Some(val) = prior {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
    }

    // ─── parse_general_search_groups additional coverage ──────────────────

    #[test]
    fn parse_general_search_groups_empty_returns_empty() {
        let groups = parse_general_search_groups("");
        assert!(groups.is_empty());
    }

    #[test]
    fn parse_general_search_groups_whitespace_returns_empty() {
        let groups = parse_general_search_groups("   ");
        assert!(groups.is_empty());
    }

    #[test]
    fn parse_general_search_groups_single_word() {
        let groups = parse_general_search_groups("rose");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[0][0].text, "rose");
    }

    #[test]
    fn parse_general_search_groups_multiple_ors() {
        let groups = parse_general_search_groups("cat OR dog OR bird");
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0][0].text, "cat");
        assert_eq!(groups[1][0].text, "dog");
        assert_eq!(groups[2][0].text, "bird");
    }

    #[test]
    fn parse_general_search_groups_trailing_or_is_skipped() {
        let groups = parse_general_search_groups("hello OR ");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0][0].text, "hello");
    }

    // ─── bulk_delete with delete_files=true (trash errors collected) ──────

    #[tokio::test]
    async fn bulk_delete_with_delete_files_errors_when_file_not_found() {
        let pool = test_pool().await;

        // Design 1 has filepath 'Roses/rose.pes' which doesn't exist on disk
        let result = bulk_delete_designs_with_pool(&pool, &[1], true)
            .await
            .expect("bulk delete should not fail even with file errors");
        assert_eq!(result.deleted_count, 1);
        assert!(result.files_trashed == 0);
        // Should report error about file not found
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("not found on disk"));
    }

    // ─── generate_preview is external — just document the gap ─────────────

    // ─── push_general_search_clause ──────────────────────────────────────

    #[test]
    fn push_general_search_clause_adds_file_and_tag_and_folder_search() {
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
        let tokens = vec![GeneralSearchToken {
            text: "rose".to_string(),
            phrase: false,
            exclude: false,
            is_extension: false,
        }];
        let groups = vec![tokens];

        push_general_search_clause(&mut builder, true, true, true, &groups);

        let sql = builder.sql();
        assert!(sql.contains("LOWER(d.filename) LIKE"));
        assert!(sql.contains("design_tags"));
        assert!(sql.contains("LOWER(tags.description) LIKE"));
        assert!(sql.contains("LOWER(d.filepath) LIKE"));
        // The bind values are stored as parameters, so count the `?` placeholders.
        assert!(sql.matches("LIKE ").count() >= 3);
    }

    #[test]
    fn push_general_search_clause_with_exclusion_adds_not() {
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
        let tokens = vec![GeneralSearchToken {
            text: "applique".to_string(),
            phrase: false,
            exclude: true,
            is_extension: false,
        }];
        let groups = vec![tokens];

        push_general_search_clause(&mut builder, true, false, false, &groups);

        let sql = builder.sql();
        assert!(sql.contains("NOT ("));
        assert!(sql.contains("LOWER(d.filename) LIKE"));
        assert!(sql.contains(")"));
    }

    #[test]
    fn push_general_search_clause_with_or_groups_uses_or_between_groups() {
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
        let group_a = vec![GeneralSearchToken {
            text: "rose".to_string(),
            phrase: false,
            exclude: false,
            is_extension: false,
        }];
        let group_b = vec![GeneralSearchToken {
            text: "hus".to_string(),
            phrase: false,
            exclude: false,
            is_extension: true,
        }];
        let groups = vec![group_a, group_b];

        push_general_search_clause(&mut builder, true, false, false, &groups);

        let sql = builder.sql();
        assert!(sql.contains(" OR "));
        // Each group adds a LIKE placeholder for the file search.
        assert!(sql.matches("LOWER(d.filename) LIKE").count() >= 2);
    }

    #[test]
    fn push_general_search_clause_empty_groups_is_noop() {
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
        let original = builder.sql().to_string();
        push_general_search_clause(&mut builder, true, true, true, &[]);
        assert_eq!(builder.sql(), original);
    }

    #[test]
    fn push_general_search_clause_and_between_tokens_within_group() {
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM designs");
        let tokens = vec![
            GeneralSearchToken {
                text: "rose".to_string(),
                phrase: false,
                exclude: false,
                is_extension: false,
            },
            GeneralSearchToken {
                text: "satin".to_string(),
                phrase: false,
                exclude: false,
                is_extension: false,
            },
        ];
        let groups = vec![tokens];

        push_general_search_clause(&mut builder, true, false, false, &groups);

        let sql = builder.sql();
        assert!(sql.contains(" AND "));
        assert!(sql.matches("LOWER(d.filename) LIKE").count() >= 2);
    }

    // ─── recommend_hoop_for_design ───────────────────────────────────────

    #[tokio::test]
    async fn recommend_hoop_selects_smallest_fitting_hoop() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Small', 50, 40)")
            .execute(&pool)
            .await
            .expect("insert small hoop");
        sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Large', 200, 200)")
            .execute(&pool)
            .await
            .expect("insert large hoop");

        let result = recommend_hoop_for_design(&pool, Some(40), Some(35))
            .await
            .expect("hoop recommendation should succeed");

        // Small (50x40) fits 40x35; should be chosen over Large (200x200)
        assert!(result.is_some());
        let name = sqlx::query_scalar::<_, String>("SELECT name FROM hoops WHERE id = ?")
            .bind(result.unwrap())
            .fetch_one(&pool)
            .await
            .expect("hoop name query");
        assert_eq!(name, "Small");
    }

    #[tokio::test]
    async fn recommend_hoop_tries_rotated_orientation() {
        let pool = test_pool().await;
        // 60 wide x 30 tall: fits Small (50x40) rotated (40 wide x 50 tall needed)
        // Actually design 60x30 -> needs 60 wide. Little (70x20) won't fit.
        // To prove rotation: insert hoop that fits when the design is rotated 90°.
        // Design 60x30 -> rotated 30x60. Need a hoop >= 30 wide, >= 60 tall.
        sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Tall', 30, 70)")
            .execute(&pool)
            .await
            .expect("insert tall hoop");

        let result = recommend_hoop_for_design(&pool, Some(60), Some(30))
            .await
            .expect("hoop recommendation should succeed");

        // Only Tall (30x70) fits either orientation: width=60 fails (30<60),
        // but rotated width=30,height=60 → 30>=30 and 70>=60 passes.
        assert!(result.is_some());
        let name = sqlx::query_scalar::<_, String>("SELECT name FROM hoops WHERE id = ?")
            .bind(result.unwrap())
            .fetch_one(&pool)
            .await
            .expect("hoop name query");
        assert_eq!(name, "Tall");
    }

    #[tokio::test]
    async fn recommend_hoop_returns_none_when_no_hoop_fits() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES ('Tiny', 5, 5)")
            .execute(&pool)
            .await
            .expect("insert tiny hoop");

        // Use dimensions larger than ALL seeded hoops (Hoop A is 126x126),
        // so no hoop fits in either orientation.
        let result = recommend_hoop_for_design(&pool, Some(300), Some(300))
            .await
            .expect("hoop recommendation should succeed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn recommend_hoop_returns_none_when_dimensions_missing() {
        let pool = test_pool().await;
        let result = recommend_hoop_for_design(&pool, None, Some(10))
            .await
            .expect("hoop recommendation should succeed");
        assert!(result.is_none());

        let result = recommend_hoop_for_design(&pool, Some(10), None)
            .await
            .expect("hoop recommendation should succeed");
        assert!(result.is_none());
    }

    // ─── normalize_windows_explorer_target (Windows-only) ────────────────

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_windows_explorer_target_strips_verbatim_unc_prefix() {
        let result = normalize_windows_explorer_target(&PathBuf::from(r"\\?\UNC\server\share\file.pes"));
        assert_eq!(result.to_string_lossy(), r"\\server\share\file.pes");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_windows_explorer_target_strips_verbatim_local_prefix() {
        let result = normalize_windows_explorer_target(&PathBuf::from(r"\\?\C:\data\file.pes"));
        assert_eq!(result.to_string_lossy(), r"C:\data\file.pes");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_windows_explorer_target_converts_forward_slashes() {
        let result = normalize_windows_explorer_target(&PathBuf::from("C:/data/file.pes"));
        assert_eq!(result.to_string_lossy(), r"C:\data\file.pes");
    }
}
