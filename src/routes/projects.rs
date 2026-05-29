use crate::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProjectSummary {
	pub id: i64,
	pub name: String,
	pub description: Option<String>,
	pub date_created: Option<String>,
	pub design_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProjectRequest {
	pub name: String,
	pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProjectRequest {
	pub name: String,
	pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectMutationResult {
	pub project_id: i64,
	pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDesignCard {
	pub id: i64,
	pub filename: String,
	pub designer_name: Option<String>,
	pub has_image: bool,
	pub image_data_url: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct ProjectDesignCardRow {
	id: i64,
	filename: String,
	designer_name: Option<String>,
	has_image: bool,
	image_data: Option<Vec<u8>>,
	image_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDetailView {
	pub project: ProjectSummary,
	pub designs: Vec<ProjectDesignCard>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveProjectDesignResult {
	pub project_id: i64,
	pub design_id: i64,
	pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectPrintView {
	pub project: ProjectSummary,
	pub designs: Vec<ProjectPrintDesign>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectPrintDesign {
	pub id: i64,
	pub filename: String,
	pub image_data_url: Option<String>,
	pub width_mm: Option<i64>,
	pub height_mm: Option<i64>,
	pub hoop: Option<String>,
	pub stitch_count: Option<i64>,
	pub color_count: Option<i64>,
	pub color_change_count: Option<i64>,
	pub designer_name: Option<String>,
	pub rating: Option<i64>,
	pub is_stitched: bool,
	pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct ProjectPrintDesignRow {
	id: i64,
	filename: String,
	image_data: Option<Vec<u8>>,
	image_type: Option<String>,
	width_mm: Option<f64>,
	height_mm: Option<f64>,
	hoop: Option<String>,
	stitch_count: Option<i64>,
	color_count: Option<i64>,
	color_change_count: Option<i64>,
	designer_name: Option<String>,
	rating: Option<i64>,
	is_stitched: bool,
	notes: Option<String>,
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

fn validate_non_empty(value: &str, label: &str) -> Result<String, String> {
	let trimmed = value.trim();
	if trimmed.is_empty() {
		return Err(format!("{} is required.", label));
	}
	Ok(trimmed.to_string())
}

fn build_data_url(image_data: Option<Vec<u8>>, image_type: Option<&str>) -> Option<String> {
	let bytes = image_data?;
	if bytes.is_empty() {
		return None;
	}

	let mime = match image_type.map(|value| value.trim().to_ascii_lowercase()) {
		Some(kind) if kind == "jpeg" || kind == "jpg" => "image/jpeg",
		Some(kind) if kind == "webp" => "image/webp",
		_ => "image/png",
	};

	Some(format!("data:{};base64,{}", mime, STANDARD.encode(bytes)))
}

fn round_mm_to_i64(value: Option<f64>) -> Option<i64> {
	value.map(|v| v.round() as i64)
}

async fn ensure_project_exists(pool: &SqlitePool, project_id: i64) -> Result<ProjectSummary, String> {
	if project_id <= 0 {
		return Err("Project not found.".to_string());
	}

	sqlx::query_as::<_, ProjectSummary>(
		r#"
		SELECT
			p.id,
			p.name,
			p.description,
			p.date_created,
			COUNT(pd.design_id) AS design_count
		FROM projects p
		LEFT JOIN project_designs pd ON pd.project_id = p.id
		WHERE p.id = ?
		GROUP BY p.id, p.name, p.description, p.date_created
		LIMIT 1
		"#,
	)
	.bind(project_id)
	.fetch_optional(pool)
	.await
	.map_err(|e| e.to_string())?
	.ok_or_else(|| "Project not found.".to_string())
}

async fn ensure_unique_project_name(pool: &SqlitePool, name: &str) -> Result<(), String> {
	let exists = sqlx::query_scalar::<_, i64>(
		"SELECT 1 FROM projects WHERE lower(name) = lower(?) LIMIT 1",
	)
	.bind(name)
	.fetch_optional(pool)
	.await
	.map_err(|e| e.to_string())?
	.is_some();

	if exists {
		Err(format!("Project '{}' already exists.", name))
	} else {
		Ok(())
	}
}

async fn ensure_unique_project_name_except_id(
	pool: &SqlitePool,
	project_id: i64,
	name: &str,
) -> Result<(), String> {
	let exists = sqlx::query_scalar::<_, i64>(
		"SELECT 1 FROM projects WHERE lower(name) = lower(?) AND id <> ? LIMIT 1",
	)
	.bind(name)
	.bind(project_id)
	.fetch_optional(pool)
	.await
	.map_err(|e| e.to_string())?
	.is_some();

	if exists {
		Err(format!("Project '{}' already exists.", name))
	} else {
		Ok(())
	}
}

#[tauri::command]
pub async fn get_projects_list(state: State<'_, AppState>) -> Result<Vec<ProjectSummary>, String> {
	sqlx::query_as::<_, ProjectSummary>(
		r#"
		SELECT
			p.id,
			p.name,
			p.description,
			p.date_created,
			COUNT(pd.design_id) AS design_count
		FROM projects p
		LEFT JOIN project_designs pd ON pd.project_id = p.id
		GROUP BY p.id, p.name, p.description, p.date_created
		ORDER BY p.name COLLATE NOCASE ASC
		"#,
	)
	.fetch_all(&state.db)
	.await
	.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_project(
	state: State<'_, AppState>,
	request: CreateProjectRequest,
) -> Result<ProjectMutationResult, String> {
	let name = validate_non_empty(&request.name, "Project name")?;
	let description = normalize_optional_text(&request.description);

	ensure_unique_project_name(&state.db, &name).await?;

	let result = sqlx::query(
		"INSERT INTO projects (name, description, date_created) VALUES (?, ?, date('now'))",
	)
	.bind(&name)
	.bind(description)
	.execute(&state.db)
	.await
	.map_err(|e| e.to_string())?;

	Ok(ProjectMutationResult {
		project_id: result.last_insert_rowid(),
		message: "Project created.".to_string(),
	})
}

#[tauri::command]
pub async fn get_project_detail(
	state: State<'_, AppState>,
	project_id: i64,
) -> Result<ProjectDetailView, String> {
	let project = ensure_project_exists(&state.db, project_id).await?;

	let design_rows = sqlx::query_as::<_, ProjectDesignCardRow>(
		r#"
		SELECT
			d.id AS id,
			d.filename AS filename,
			des.name AS designer_name,
			CASE WHEN d.image_data IS NOT NULL AND length(d.image_data) > 0 THEN 1 ELSE 0 END AS has_image,
			d.image_data AS image_data,
			d.image_type AS image_type
		FROM project_designs pd
		INNER JOIN designs d ON d.id = pd.design_id
		LEFT JOIN designers des ON des.id = d.designer_id
		WHERE pd.project_id = ?
		ORDER BY d.filename COLLATE NOCASE ASC
		"#,
	)
	.bind(project_id)
	.fetch_all(&state.db)
	.await
	.map_err(|e| e.to_string())?;

	let designs = design_rows
		.into_iter()
		.map(|row| ProjectDesignCard {
			id: row.id,
			filename: row.filename,
			designer_name: row.designer_name,
			has_image: row.has_image,
			image_data_url: build_data_url(row.image_data, row.image_type.as_deref()),
		})
		.collect();

	Ok(ProjectDetailView { project, designs })
}

#[tauri::command]
pub async fn update_project(
	state: State<'_, AppState>,
	project_id: i64,
	request: UpdateProjectRequest,
) -> Result<ProjectMutationResult, String> {
	let name = validate_non_empty(&request.name, "Project name")?;
	let description = normalize_optional_text(&request.description);

	ensure_project_exists(&state.db, project_id).await?;
	ensure_unique_project_name_except_id(&state.db, project_id, &name).await?;

	sqlx::query("UPDATE projects SET name = ?, description = ? WHERE id = ?")
		.bind(name)
		.bind(description)
		.bind(project_id)
		.execute(&state.db)
		.await
		.map_err(|e| e.to_string())?;

	Ok(ProjectMutationResult {
		project_id,
		message: "Project updated.".to_string(),
	})
}

#[tauri::command]
pub async fn delete_project(
	state: State<'_, AppState>,
	project_id: i64,
) -> Result<ProjectMutationResult, String> {
	ensure_project_exists(&state.db, project_id).await?;

	sqlx::query("DELETE FROM projects WHERE id = ?")
		.bind(project_id)
		.execute(&state.db)
		.await
		.map_err(|e| e.to_string())?;

	Ok(ProjectMutationResult {
		project_id,
		message: "Project deleted.".to_string(),
	})
}

#[tauri::command]
pub async fn remove_design_from_project_detail(
	state: State<'_, AppState>,
	project_id: i64,
	design_id: i64,
) -> Result<RemoveProjectDesignResult, String> {
	if design_id <= 0 {
		return Err("Design id must be a positive id.".to_string());
	}

	ensure_project_exists(&state.db, project_id).await?;

	sqlx::query("DELETE FROM project_designs WHERE project_id = ? AND design_id = ?")
		.bind(project_id)
		.bind(design_id)
		.execute(&state.db)
		.await
		.map_err(|e| e.to_string())?;

	Ok(RemoveProjectDesignResult {
		project_id,
		design_id,
		message: "Design removed from project.".to_string(),
	})
}

#[tauri::command]
pub async fn get_project_print_view(
	state: State<'_, AppState>,
	project_id: i64,
) -> Result<ProjectPrintView, String> {
	let project = ensure_project_exists(&state.db, project_id).await?;

	let design_rows = sqlx::query_as::<_, ProjectPrintDesignRow>(
		r#"
		SELECT
			d.id AS id,
			d.filename AS filename,
			d.image_data AS image_data,
			d.image_type AS image_type,
			d.width_mm AS width_mm,
			d.height_mm AS height_mm,
			h.name AS hoop,
			d.stitch_count AS stitch_count,
			d.color_count AS color_count,
			d.color_change_count AS color_change_count,
			des.name AS designer_name,
			d.rating AS rating,
			d.is_stitched AS is_stitched,
			d.notes AS notes
		FROM project_designs pd
		INNER JOIN designs d ON d.id = pd.design_id
		LEFT JOIN hoops h ON h.id = d.hoop_id
		LEFT JOIN designers des ON des.id = d.designer_id
		WHERE pd.project_id = ?
		ORDER BY d.filename COLLATE NOCASE ASC
		"#,
	)
	.bind(project_id)
	.fetch_all(&state.db)
	.await
	.map_err(|e| e.to_string())?;

	let designs = design_rows
		.into_iter()
		.map(|row| ProjectPrintDesign {
			id: row.id,
			filename: row.filename,
			image_data_url: build_data_url(row.image_data, row.image_type.as_deref()),
			width_mm: round_mm_to_i64(row.width_mm),
			height_mm: round_mm_to_i64(row.height_mm),
			hoop: row.hoop,
			stitch_count: row.stitch_count,
			color_count: row.color_count,
			color_change_count: row.color_change_count,
			designer_name: row.designer_name,
			rating: row.rating,
			is_stitched: row.is_stitched,
			notes: normalize_optional_text(&row.notes),
		})
		.collect();

	Ok(ProjectPrintView { project, designs })
}
