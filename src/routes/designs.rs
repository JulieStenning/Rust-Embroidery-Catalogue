use crate::config::BootstrapConfig;
use crate::services::image_generation::{generate_preview, ImageGenerationRequest};
use crate::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::State;

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

    std::env::var("PYTEST_CURRENT_TEST").is_ok()
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
) -> Result<Render3dPreviewResult, String> {
    let filepath = get_design_filepath(pool, design_id).await?;
    let full_path = resolve_design_full_path(&filepath);

    if !full_path.is_file() {
        return Err("Design file not found on disk for 3D rendering.".to_string());
    }

    let preview_3d_profile: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'image.preview_3d_profile' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

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
        preview_3d: true,
        preview_3d_profile: Some(preview_3d_profile),
    });

    if let Some(error) = generation_result.error {
        return Err(error);
    }

    let image_type = generation_result
        .image_type
        .clone()
        .or_else(|| Some("3d".to_string()));
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

    Ok(Render3dPreviewResult {
        design_id,
        image_type,
        width_mm,
        height_mm,
        stitch_count: generation_result.stitch_count,
        color_count: generation_result.color_count,
        color_change_count: generation_result.color_change_count,
        backend: generation_result.backend,
        message: "3D preview rendered and saved.".to_string(),
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
            GROUP_CONCAT(CASE WHEN COALESCE(tags.tag_group, '') = 'stitching_type' THEN tags.description END, '|||') AS stitching_tags_csv,
            GROUP_CONCAT(CASE WHEN COALESCE(tags.tag_group, '') != 'stitching_type' THEN tags.description END, '|||') AS image_tags_csv,
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

            if search_file || search_tags || search_folder {
                push_where_clause(&mut query_builder, &mut has_where);
                query_builder.push("(");

                let mut or_added = false;
                let like_pattern = format!("%{}%", q.to_lowercase());

                if search_file {
                    query_builder.push("LOWER(d.filename) LIKE ");
                    query_builder.push_bind(like_pattern.clone());
                    or_added = true;
                }

                if search_tags {
                    if or_added {
                        query_builder.push(" OR ");
                    }
                    query_builder.push("d.id IN (SELECT design_id FROM design_tags JOIN tags ON tags.id = design_tags.tag_id WHERE LOWER(tags.description) LIKE ");
                    query_builder.push_bind(like_pattern.clone());
                    query_builder.push(")");
                    or_added = true;
                }

                if search_folder {
                    if or_added {
                        query_builder.push(" OR ");
                    }
                    query_builder.push("LOWER(d.filepath) LIKE ");
                    query_builder.push_bind(like_pattern);
                }

                query_builder.push(")");
            }
        }

        if p.unverified_only.unwrap_or(false) {
                push_where_clause(&mut query_builder, &mut has_where);
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
                query_builder.push("COALESCE(tags.tag_group, '') != 'stitching_type' AND (");
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
                query_builder.push("COALESCE(tags.tag_group, '') = 'stitching_type' AND (");
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
    state: State<'_, AppState>,
    design_id: i64,
    request: UpdateDesignMetadataRequest,
) -> Result<DesignCommandResult, String> {
    update_design_metadata_with_pool(&state.db, design_id, request).await
}

#[tauri::command]
pub async fn set_design_rating(
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignRatingRequest,
) -> Result<DesignCommandResult, String> {
    set_design_rating_with_pool(&state.db, design_id, request.rating).await
}

#[tauri::command]
pub async fn set_design_stitched(
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignStitchedRequest,
) -> Result<DesignCommandResult, String> {
    set_design_stitched_with_pool(&state.db, design_id, request.is_stitched).await
}

#[tauri::command]
pub async fn set_design_tags_checked(
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignTagsCheckedRequest,
) -> Result<DesignCommandResult, String> {
    set_design_tags_checked_with_pool(&state.db, design_id, request.tags_checked).await
}

#[tauri::command]
pub async fn set_design_tags(
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignTagsRequest,
) -> Result<DesignCommandResult, String> {
    set_design_tags_with_pool(&state.db, design_id, request.tag_ids).await
}

#[tauri::command]
pub async fn add_design_to_project(
    state: State<'_, AppState>,
    design_id: i64,
    request: SetDesignProjectRequest,
) -> Result<DesignCommandResult, String> {
    add_design_to_project_with_pool(&state.db, design_id, request.project_id).await
}

#[tauri::command]
pub async fn remove_design_from_project(
    state: State<'_, AppState>,
    design_id: i64,
    project_id: i64,
) -> Result<DesignCommandResult, String> {
    remove_design_from_project_with_pool(&state.db, design_id, project_id).await
}

#[tauri::command]
pub async fn delete_design(
    state: State<'_, AppState>,
    design_id: i64,
    delete_file: bool,
) -> Result<DesignCommandResult, String> {
    delete_design_with_pool(&state.db, design_id, delete_file).await
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
) -> Result<Render3dPreviewResult, String> {
    render_design_3d_preview_with_pool(&state.db, design_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let result = render_design_3d_preview_with_pool(&pool, 1).await;
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
}
