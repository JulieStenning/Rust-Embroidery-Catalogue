use crate::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use sqlx::{FromRow, QueryBuilder, Sqlite};
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct BrowseDesignSummary {
	pub id: i64,
	pub filename: String,
	pub designer: String,
	pub source: String,
	pub hoop: Option<String>,
	pub tags: Vec<String>,
	pub is_stitched: bool,
	pub tags_checked: bool,
	pub rating: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
struct BrowseDesignSummaryRow {
	pub id: i64,
	pub filename: String,
	pub designer: String,
	pub source: String,
	pub hoop: Option<String>,
	pub tags_csv: Option<String>,
	pub is_stitched: bool,
	pub tags_checked: bool,
	pub rating: Option<i64>,
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
	pub designer: String,
	pub source: String,
	pub notes: Option<String>,
	pub rating: Option<i64>,
	pub date_added: Option<String>,
}

#[tauri::command]
pub async fn get_designs(state: State<'_, AppState>) -> Result<Vec<BrowseDesignSummary>, String> {
	let rows = sqlx::query_as::<_, BrowseDesignSummaryRow>(
		r#"
		SELECT
			d.id AS id,
			d.filename AS filename,
			COALESCE(designers.name, 'Unknown') AS designer,
			COALESCE(sources.name, 'Unknown') AS source,
			hoops.name AS hoop,
			GROUP_CONCAT(tags.description, '|||') AS tags_csv,
			d.is_stitched AS is_stitched,
			d.tags_checked AS tags_checked,
			d.rating AS rating
		FROM designs d
		LEFT JOIN designers ON designers.id = d.designer_id
		LEFT JOIN sources ON sources.id = d.source_id
		LEFT JOIN hoops ON hoops.id = d.hoop_id
		LEFT JOIN design_tags ON design_tags.design_id = d.id
		LEFT JOIN tags ON tags.id = design_tags.tag_id
		GROUP BY d.id
		ORDER BY d.filename COLLATE NOCASE ASC
		LIMIT 500
		"#,
	)
	.fetch_all(&state.db)
	.await
	.map_err(|e| e.to_string())?;

	let items = rows
		.into_iter()
		.map(|row| BrowseDesignSummary {
			id: row.id,
			filename: row.filename,
			designer: row.designer,
			source: row.source,
			hoop: row.hoop,
			tags: row
				.tags_csv
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
pub async fn get_tags_for_browse(state: State<'_, AppState>) -> Result<Vec<BrowseTagOption>, String> {
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

	let mut builder = QueryBuilder::<Sqlite>::new(
		"SELECT id, image_data, image_type FROM designs WHERE id IN (",
	);

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
	sqlx::query_as::<_, DesignDetail>(
		r#"
		SELECT
			d.id AS id,
			d.filename AS filename,
			d.filepath AS filepath,
			COALESCE(designers.name, 'Unknown') AS designer,
			COALESCE(sources.name, 'Unknown') AS source,
			d.notes AS notes,
			d.rating AS rating,
			d.date_added AS date_added
		FROM designs d
		LEFT JOIN designers ON designers.id = d.designer_id
		LEFT JOIN sources ON sources.id = d.source_id
		WHERE d.id = ?
		LIMIT 1
		"#,
	)
	.bind(design_id)
	.fetch_optional(&state.db)
	.await
	.map_err(|e| e.to_string())
}
