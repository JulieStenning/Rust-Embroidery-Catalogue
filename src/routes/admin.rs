use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AdminDesigner {
	pub id: i64,
	pub name: String,
	pub design_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDesignerRequest {
	pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDesignerRequest {
	pub designer_id: i64,
	pub name: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AdminSource {
	pub id: i64,
	pub name: String,
	pub design_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSourceRequest {
	pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSourceRequest {
	pub source_id: i64,
	pub name: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AdminTag {
	pub id: i64,
	pub description: String,
	pub tag_group: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTagRequest {
	pub description: String,
	pub tag_group: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetTagGroupRequest {
	pub tag_id: i64,
	pub tag_group: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AdminHoop {
	pub id: i64,
	pub name: String,
	pub max_width_mm: f64,
	pub max_height_mm: f64,
	pub design_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateHoopRequest {
	pub name: String,
	pub max_width_mm: f64,
	pub max_height_mm: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateHoopRequest {
	pub hoop_id: i64,
	pub name: String,
	pub max_width_mm: f64,
	pub max_height_mm: f64,
}

fn validate_non_empty(value: &str, label: &str) -> Result<String, String> {
	let trimmed = value.trim();
	if trimmed.is_empty() {
		return Err(format!("{} is required.", label));
	}
	Ok(trimmed.to_string())
}

fn validate_positive(value: f64, label: &str) -> Result<f64, String> {
	if !value.is_finite() || value <= 0.0 {
		return Err(format!("{} must be a positive number.", label));
	}
	Ok(value)
}

fn validate_tag_group(raw: &str) -> Result<String, String> {
	let group = raw.trim().to_lowercase();
	if group == "image" || group == "stitching" {
		Ok(group)
	} else {
		Err("Tag group must be 'image' or 'stitching'.".to_string())
	}
}

async fn ensure_unique_name(pool: &SqlitePool, table: &str, name: &str, label: &str) -> Result<(), String> {
	let sql = format!("SELECT 1 FROM {} WHERE lower(name) = lower(?) LIMIT 1", table);
	let exists = sqlx::query_scalar::<_, i64>(&sql)
		.bind(name)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?
		.is_some();

	if exists {
		Err(format!("{} '{}' already exists.", label, name))
	} else {
		Ok(())
	}
}

async fn ensure_unique_name_except_id(
	pool: &SqlitePool,
	table: &str,
	id_column: &str,
	excluded_id: i64,
	name: &str,
	label: &str,
) -> Result<(), String> {
	let sql = format!(
		"SELECT 1 FROM {} WHERE lower(name) = lower(?) AND {} <> ? LIMIT 1",
		table, id_column
	);
	let exists = sqlx::query_scalar::<_, i64>(&sql)
		.bind(name)
		.bind(excluded_id)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?
		.is_some();

	if exists {
		Err(format!("{} '{}' already exists.", label, name))
	} else {
		Ok(())
	}
}

#[tauri::command]
pub async fn list_designers(state: State<'_, AppState>) -> Result<Vec<AdminDesigner>, String> {
	sqlx::query_as::<_, AdminDesigner>(
		r#"
		SELECT
			d.id,
			d.name,
			COUNT(des.id) AS design_count
		FROM designers d
		LEFT JOIN designs des ON des.designer_id = d.id
		GROUP BY d.id, d.name
		ORDER BY d.name COLLATE NOCASE ASC
		"#,
	)
	.fetch_all(&state.db)
	.await
	.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_designer(
	state: State<'_, AppState>,
	request: CreateDesignerRequest,
) -> Result<AdminDesigner, String> {
	create_designer_with_pool(&state.db, request).await
}

async fn create_designer_with_pool(
	pool: &SqlitePool,
	request: CreateDesignerRequest,
) -> Result<AdminDesigner, String> {
	let name = validate_non_empty(&request.name, "Designer name")?;
	ensure_unique_name(pool, "designers", &name, "Designer").await?;

	let result = sqlx::query("INSERT INTO designers (name) VALUES (?)")
		.bind(&name)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	Ok(AdminDesigner {
		id: result.last_insert_rowid(),
		name,
		design_count: 0,
	})
}

#[tauri::command]
pub async fn update_designer(
	state: State<'_, AppState>,
	request: UpdateDesignerRequest,
) -> Result<AdminDesigner, String> {
	update_designer_with_pool(&state.db, request).await
}

async fn update_designer_with_pool(pool: &SqlitePool, request: UpdateDesignerRequest) -> Result<AdminDesigner, String> {
	let name = validate_non_empty(&request.name, "Designer name")?;
	ensure_unique_name_except_id(pool, "designers", "id", request.designer_id, &name, "Designer").await?;

	let result = sqlx::query("UPDATE designers SET name = ? WHERE id = ?")
		.bind(&name)
		.bind(request.designer_id)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	if result.rows_affected() == 0 {
		return Err(format!("Designer with id={} not found.", request.designer_id));
	}

	let row = sqlx::query_as::<_, AdminDesigner>(
		r#"
		SELECT
			d.id,
			d.name,
			COUNT(des.id) AS design_count
		FROM designers d
		LEFT JOIN designs des ON des.designer_id = d.id
		WHERE d.id = ?
		GROUP BY d.id, d.name
		LIMIT 1
		"#,
	)
	.bind(request.designer_id)
	.fetch_one(pool)
	.await
	.map_err(|e| e.to_string())?;

	Ok(row)
}

#[tauri::command]
pub async fn delete_designer(state: State<'_, AppState>, designer_id: i64) -> Result<(), String> {
	delete_designer_with_pool(&state.db, designer_id).await
}

async fn delete_designer_with_pool(pool: &SqlitePool, designer_id: i64) -> Result<(), String> {
	let result = sqlx::query("DELETE FROM designers WHERE id = ?")
		.bind(designer_id)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	if result.rows_affected() == 0 {
		Err(format!("Designer with id={} not found.", designer_id))
	} else {
		Ok(())
	}
}

#[tauri::command]
pub async fn list_sources(state: State<'_, AppState>) -> Result<Vec<AdminSource>, String> {
	sqlx::query_as::<_, AdminSource>(
		r#"
		SELECT
			s.id,
			s.name,
			COUNT(d.id) AS design_count
		FROM sources s
		LEFT JOIN designs d ON d.source_id = s.id
		GROUP BY s.id, s.name
		ORDER BY s.name COLLATE NOCASE ASC
		"#,
	)
	.fetch_all(&state.db)
	.await
	.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_source(
	state: State<'_, AppState>,
	request: CreateSourceRequest,
) -> Result<AdminSource, String> {
	create_source_with_pool(&state.db, request).await
}

async fn create_source_with_pool(pool: &SqlitePool, request: CreateSourceRequest) -> Result<AdminSource, String> {
	let name = validate_non_empty(&request.name, "Source name")?;
	ensure_unique_name(pool, "sources", &name, "Source").await?;

	let result = sqlx::query("INSERT INTO sources (name) VALUES (?)")
		.bind(&name)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	Ok(AdminSource {
		id: result.last_insert_rowid(),
		name,
		design_count: 0,
	})
}

#[tauri::command]
pub async fn update_source(
	state: State<'_, AppState>,
	request: UpdateSourceRequest,
) -> Result<AdminSource, String> {
	update_source_with_pool(&state.db, request).await
}

async fn update_source_with_pool(pool: &SqlitePool, request: UpdateSourceRequest) -> Result<AdminSource, String> {
	let name = validate_non_empty(&request.name, "Source name")?;
	ensure_unique_name_except_id(pool, "sources", "id", request.source_id, &name, "Source").await?;

	let result = sqlx::query("UPDATE sources SET name = ? WHERE id = ?")
		.bind(&name)
		.bind(request.source_id)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	if result.rows_affected() == 0 {
		return Err(format!("Source with id={} not found.", request.source_id));
	}

	let row = sqlx::query_as::<_, AdminSource>(
		r#"
		SELECT
			s.id,
			s.name,
			COUNT(d.id) AS design_count
		FROM sources s
		LEFT JOIN designs d ON d.source_id = s.id
		WHERE s.id = ?
		GROUP BY s.id, s.name
		LIMIT 1
		"#,
	)
	.bind(request.source_id)
	.fetch_one(pool)
	.await
	.map_err(|e| e.to_string())?;

	Ok(row)
}

#[tauri::command]
pub async fn delete_source(state: State<'_, AppState>, source_id: i64) -> Result<(), String> {
	delete_source_with_pool(&state.db, source_id).await
}

async fn delete_source_with_pool(pool: &SqlitePool, source_id: i64) -> Result<(), String> {
	let result = sqlx::query("DELETE FROM sources WHERE id = ?")
		.bind(source_id)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	if result.rows_affected() == 0 {
		Err(format!("Source with id={} not found.", source_id))
	} else {
		Ok(())
	}
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<AdminTag>, String> {
	sqlx::query_as::<_, AdminTag>(
		r#"
		SELECT id, description, tag_group
		FROM tags
		ORDER BY description COLLATE NOCASE ASC
		"#,
	)
	.fetch_all(&state.db)
	.await
	.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_tag(state: State<'_, AppState>, request: CreateTagRequest) -> Result<AdminTag, String> {
	create_tag_with_pool(&state.db, request).await
}

async fn create_tag_with_pool(pool: &SqlitePool, request: CreateTagRequest) -> Result<AdminTag, String> {
	let description = validate_non_empty(&request.description, "Tag description")?;
	let tag_group = validate_tag_group(&request.tag_group)?;

	let existing =
		sqlx::query_scalar::<_, i64>("SELECT 1 FROM tags WHERE lower(description) = lower(?) LIMIT 1")
		.bind(&description)
		.fetch_optional(pool)
		.await
		.map_err(|e| e.to_string())?
		.is_some();

	if existing {
		return Err(format!("Tag '{}' already exists.", description));
	}

	let result = sqlx::query("INSERT INTO tags (description, tag_group) VALUES (?, ?)")
		.bind(&description)
		.bind(&tag_group)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	Ok(AdminTag {
		id: result.last_insert_rowid(),
		description,
		tag_group: Some(tag_group),
	})
}

#[tauri::command]
pub async fn set_tag_group(
	state: State<'_, AppState>,
	request: SetTagGroupRequest,
) -> Result<AdminTag, String> {
	set_tag_group_with_pool(&state.db, request).await
}

async fn set_tag_group_with_pool(
	pool: &SqlitePool,
	request: SetTagGroupRequest,
) -> Result<AdminTag, String> {
	let tag_group = validate_tag_group(&request.tag_group)?;

	let result = sqlx::query("UPDATE tags SET tag_group = ? WHERE id = ?")
		.bind(&tag_group)
		.bind(request.tag_id)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	if result.rows_affected() == 0 {
		return Err(format!("Tag with id={} not found.", request.tag_id));
	}

	let row = sqlx::query_as::<_, AdminTag>(
		r#"
		SELECT id, description, tag_group
		FROM tags
		WHERE id = ?
		LIMIT 1
		"#,
	)
	.bind(request.tag_id)
	.fetch_one(pool)
	.await
	.map_err(|e| e.to_string())?;

	Ok(row)
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, tag_id: i64) -> Result<(), String> {
	delete_tag_with_pool(&state.db, tag_id).await
}

async fn delete_tag_with_pool(pool: &SqlitePool, tag_id: i64) -> Result<(), String> {
	let result = sqlx::query("DELETE FROM tags WHERE id = ?")
		.bind(tag_id)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	if result.rows_affected() == 0 {
		Err(format!("Tag with id={} not found.", tag_id))
	} else {
		Ok(())
	}
}

#[tauri::command]
pub async fn list_hoops(state: State<'_, AppState>) -> Result<Vec<AdminHoop>, String> {
	sqlx::query_as::<_, AdminHoop>(
		r#"
		SELECT
			h.id,
			h.name,
			CAST(h.max_width_mm AS REAL) AS max_width_mm,
			CAST(h.max_height_mm AS REAL) AS max_height_mm,
			COUNT(d.id) AS design_count
		FROM hoops h
		LEFT JOIN designs d ON d.hoop_id = h.id
		GROUP BY h.id, h.name, h.max_width_mm, h.max_height_mm
		ORDER BY h.max_width_mm ASC, h.max_height_mm ASC, h.name COLLATE NOCASE ASC
		"#,
	)
	.fetch_all(&state.db)
	.await
	.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_hoop(
	state: State<'_, AppState>,
	request: CreateHoopRequest,
) -> Result<AdminHoop, String> {
	create_hoop_with_pool(&state.db, request).await
}

async fn create_hoop_with_pool(pool: &SqlitePool, request: CreateHoopRequest) -> Result<AdminHoop, String> {
	let name = validate_non_empty(&request.name, "Hoop name")?;
	let max_width_mm = validate_positive(request.max_width_mm, "Max Width (mm)")?;
	let max_height_mm = validate_positive(request.max_height_mm, "Max Height (mm)")?;
	ensure_unique_name(pool, "hoops", &name, "Hoop").await?;

	let result = sqlx::query(
		"INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES (?, ?, ?)",
	)
	.bind(&name)
	.bind(max_width_mm)
	.bind(max_height_mm)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;

	Ok(AdminHoop {
		id: result.last_insert_rowid(),
		name,
		max_width_mm,
		max_height_mm,
		design_count: 0,
	})
}

#[tauri::command]
pub async fn update_hoop(
	state: State<'_, AppState>,
	request: UpdateHoopRequest,
) -> Result<AdminHoop, String> {
	update_hoop_with_pool(&state.db, request).await
}

async fn update_hoop_with_pool(pool: &SqlitePool, request: UpdateHoopRequest) -> Result<AdminHoop, String> {
	let name = validate_non_empty(&request.name, "Hoop name")?;
	let max_width_mm = validate_positive(request.max_width_mm, "Max Width (mm)")?;
	let max_height_mm = validate_positive(request.max_height_mm, "Max Height (mm)")?;
	ensure_unique_name_except_id(pool, "hoops", "id", request.hoop_id, &name, "Hoop").await?;

	let result = sqlx::query(
		"UPDATE hoops SET name = ?, max_width_mm = ?, max_height_mm = ? WHERE id = ?",
	)
	.bind(&name)
	.bind(max_width_mm)
	.bind(max_height_mm)
	.bind(request.hoop_id)
	.execute(pool)
	.await
	.map_err(|e| e.to_string())?;

	if result.rows_affected() == 0 {
		return Err(format!("Hoop with id={} not found.", request.hoop_id));
	}

	let row = sqlx::query_as::<_, AdminHoop>(
		r#"
		SELECT
			h.id,
			h.name,
			CAST(h.max_width_mm AS REAL) AS max_width_mm,
			CAST(h.max_height_mm AS REAL) AS max_height_mm,
			COUNT(d.id) AS design_count
		FROM hoops h
		LEFT JOIN designs d ON d.hoop_id = h.id
		WHERE h.id = ?
		GROUP BY h.id, h.name, h.max_width_mm, h.max_height_mm
		LIMIT 1
		"#,
	)
	.bind(request.hoop_id)
	.fetch_one(pool)
	.await
	.map_err(|e| e.to_string())?;

	Ok(row)
}

#[tauri::command]
pub async fn delete_hoop(state: State<'_, AppState>, hoop_id: i64) -> Result<(), String> {
	delete_hoop_with_pool(&state.db, hoop_id).await
}

async fn delete_hoop_with_pool(pool: &SqlitePool, hoop_id: i64) -> Result<(), String> {
	let result = sqlx::query("DELETE FROM hoops WHERE id = ?")
		.bind(hoop_id)
		.execute(pool)
		.await
		.map_err(|e| e.to_string())?;

	if result.rows_affected() == 0 {
		Err(format!("Hoop with id={} not found.", hoop_id))
	} else {
		Ok(())
	}
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
			CREATE TABLE designs (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				filename VARCHAR(500) NOT NULL,
				filepath VARCHAR(1000) NOT NULL,
				designer_id INTEGER REFERENCES designers(id) ON DELETE SET NULL,
				source_id INTEGER REFERENCES sources(id) ON DELETE SET NULL,
				hoop_id INTEGER REFERENCES hoops(id) ON DELETE SET NULL
			);
			"#,
		)
		.execute(&pool)
		.await
		.expect("failed to create designs table");

		sqlx::query("CREATE UNIQUE INDEX ux_designers_name_ci ON designers (lower(name));")
			.execute(&pool)
			.await
			.expect("failed to create designers case-insensitive unique index");

		sqlx::query("CREATE UNIQUE INDEX ux_sources_name_ci ON sources (lower(name));")
			.execute(&pool)
			.await
			.expect("failed to create sources case-insensitive unique index");

		sqlx::query("CREATE UNIQUE INDEX ux_hoops_name_ci ON hoops (lower(name));")
			.execute(&pool)
			.await
			.expect("failed to create hoops case-insensitive unique index");

		sqlx::query("CREATE UNIQUE INDEX ux_tags_description_ci ON tags (lower(description));")
			.execute(&pool)
			.await
			.expect("failed to create tags case-insensitive unique index");

		pool
	}

	#[tokio::test]
	async fn create_designer_rejects_duplicate_name() {
		let pool = test_pool().await;

		let first = create_designer_with_pool(
			&pool,
			CreateDesignerRequest {
				name: "Amazing Designs".to_string(),
			},
		)
		.await;
		assert!(first.is_ok());

		let second = create_designer_with_pool(
			&pool,
			CreateDesignerRequest {
				name: "Amazing Designs".to_string(),
			},
		)
		.await;

		assert!(second.is_err());
		assert!(
			second
				.expect_err("expected duplicate designer error")
				.contains("already exists")
		);
	}

	#[tokio::test]
	async fn create_designer_rejects_empty_name() {
		let pool = test_pool().await;

		let result = create_designer_with_pool(
			&pool,
			CreateDesignerRequest {
				name: "   ".to_string(),
			},
		)
		.await;

		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected empty designer name error")
				.contains("Designer name is required")
		);
	}

	#[tokio::test]
	async fn create_designer_rejects_duplicate_name_case_insensitive() {
		let pool = test_pool().await;

		let first = create_designer_with_pool(
			&pool,
			CreateDesignerRequest {
				name: "Amazing Designs".to_string(),
			},
		)
		.await;
		assert!(first.is_ok());

		let second = create_designer_with_pool(
			&pool,
			CreateDesignerRequest {
				name: "amazing designs".to_string(),
			},
		)
		.await;

		assert!(second.is_err());
		assert!(
			second
				.expect_err("expected case-insensitive duplicate designer error")
				.contains("already exists")
		);
	}

	#[tokio::test]
	async fn create_tag_rejects_invalid_tag_group() {
		let pool = test_pool().await;

		let result = create_tag_with_pool(
			&pool,
			CreateTagRequest {
				description: "Animals".to_string(),
				tag_group: "invalid-group".to_string(),
			},
		)
		.await;

		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected invalid tag group error")
				.contains("Tag group must be 'image' or 'stitching'")
		);
	}

	#[tokio::test]
	async fn create_source_rejects_duplicate_name() {
		let pool = test_pool().await;

		let first = create_source_with_pool(
			&pool,
			CreateSourceRequest {
				name: "USB Import".to_string(),
			},
		)
		.await;
		assert!(first.is_ok());

		let second = create_source_with_pool(
			&pool,
			CreateSourceRequest {
				name: "USB Import".to_string(),
			},
		)
		.await;

		assert!(second.is_err());
		assert!(
			second
				.expect_err("expected duplicate source error")
				.contains("already exists")
		);
	}

	#[tokio::test]
	async fn create_source_rejects_empty_name() {
		let pool = test_pool().await;

		let result = create_source_with_pool(
			&pool,
			CreateSourceRequest {
				name: "  \t  ".to_string(),
			},
		)
		.await;

		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected empty source name error")
				.contains("Source name is required")
		);
	}

	#[tokio::test]
	async fn create_source_rejects_duplicate_name_case_insensitive() {
		let pool = test_pool().await;

		let first = create_source_with_pool(
			&pool,
			CreateSourceRequest {
				name: "USB Import".to_string(),
			},
		)
		.await;
		assert!(first.is_ok());

		let second = create_source_with_pool(
			&pool,
			CreateSourceRequest {
				name: "usb import".to_string(),
			},
		)
		.await;

		assert!(second.is_err());
		assert!(
			second
				.expect_err("expected case-insensitive duplicate source error")
				.contains("already exists")
		);
	}

	#[tokio::test]
	async fn update_source_updates_existing_row() {
		let pool = test_pool().await;

		let created = create_source_with_pool(
			&pool,
			CreateSourceRequest {
				name: "USB Import".to_string(),
			},
		)
		.await
		.expect("expected source to be created");

		let updated = update_source_with_pool(
			&pool,
			UpdateSourceRequest {
				source_id: created.id,
				name: "Downloaded".to_string(),
			},
		)
		.await
		.expect("expected source to update");

		assert_eq!(updated.id, created.id);
		assert_eq!(updated.name, "Downloaded");
		assert_eq!(updated.design_count, 0);
	}

	#[tokio::test]
	async fn update_designer_updates_existing_row() {
		let pool = test_pool().await;

		let created = create_designer_with_pool(
			&pool,
			CreateDesignerRequest {
				name: "Amazing Designs".to_string(),
			},
		)
		.await
		.expect("expected designer to be created");

		let updated = update_designer_with_pool(
			&pool,
			UpdateDesignerRequest {
				designer_id: created.id,
				name: "Urban Threads".to_string(),
			},
		)
		.await
		.expect("expected designer to update");

		assert_eq!(updated.id, created.id);
		assert_eq!(updated.name, "Urban Threads");
		assert_eq!(updated.design_count, 0);
	}

	#[tokio::test]
	async fn create_tag_rejects_duplicate_description() {
		let pool = test_pool().await;

		let first = create_tag_with_pool(
			&pool,
			CreateTagRequest {
				description: "Floral".to_string(),
				tag_group: "image".to_string(),
			},
		)
		.await;
		assert!(first.is_ok());

		let second = create_tag_with_pool(
			&pool,
			CreateTagRequest {
				description: "Floral".to_string(),
				tag_group: "stitching".to_string(),
			},
		)
		.await;

		assert!(second.is_err());
		assert!(
			second
				.expect_err("expected duplicate tag description error")
				.contains("already exists")
		);
	}

	#[tokio::test]
	async fn create_tag_rejects_empty_description() {
		let pool = test_pool().await;

		let result = create_tag_with_pool(
			&pool,
			CreateTagRequest {
				description: "   ".to_string(),
				tag_group: "image".to_string(),
			},
		)
		.await;

		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected empty tag description error")
				.contains("Tag description is required")
		);
	}

	#[tokio::test]
	async fn create_tag_rejects_duplicate_description_case_insensitive() {
		let pool = test_pool().await;

		let first = create_tag_with_pool(
			&pool,
			CreateTagRequest {
				description: "Floral".to_string(),
				tag_group: "image".to_string(),
			},
		)
		.await;
		assert!(first.is_ok());

		let second = create_tag_with_pool(
			&pool,
			CreateTagRequest {
				description: "floral".to_string(),
				tag_group: "stitching".to_string(),
			},
		)
		.await;

		assert!(second.is_err());
		assert!(
			second
				.expect_err("expected case-insensitive duplicate tag error")
				.contains("already exists")
		);
	}

	#[tokio::test]
	async fn create_hoop_rejects_duplicate_name() {
		let pool = test_pool().await;

		let first = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "130x180".to_string(),
				max_width_mm: 130.0,
				max_height_mm: 180.0,
			},
		)
		.await;
		assert!(first.is_ok());

		let second = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "130x180".to_string(),
				max_width_mm: 130.0,
				max_height_mm: 180.0,
			},
		)
		.await;

		assert!(second.is_err());
		assert!(
			second
				.expect_err("expected duplicate hoop error")
				.contains("already exists")
		);
	}

	#[tokio::test]
	async fn create_hoop_rejects_empty_name() {
		let pool = test_pool().await;

		let result = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "  ".to_string(),
				max_width_mm: 130.0,
				max_height_mm: 180.0,
			},
		)
		.await;

		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected empty hoop name error")
				.contains("Hoop name is required")
		);
	}

	#[tokio::test]
	async fn create_hoop_rejects_duplicate_name_case_insensitive() {
		let pool = test_pool().await;

		let first = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "130x180".to_string(),
				max_width_mm: 130.0,
				max_height_mm: 180.0,
			},
		)
		.await;
		assert!(first.is_ok());

		let second = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "130X180".to_string(),
				max_width_mm: 130.0,
				max_height_mm: 180.0,
			},
		)
		.await;

		assert!(second.is_err());
		assert!(
			second
				.expect_err("expected case-insensitive duplicate hoop error")
				.contains("already exists")
		);
	}

	#[tokio::test]
	async fn create_hoop_rejects_invalid_dimensions() {
		let pool = test_pool().await;

		let invalid_width = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "Invalid Width".to_string(),
				max_width_mm: 0.0,
				max_height_mm: 180.0,
			},
		)
		.await;

		assert!(invalid_width.is_err());
		assert!(
			invalid_width
				.expect_err("expected invalid width error")
				.contains("Max Width (mm) must be a positive number")
		);

		let invalid_height = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "Invalid Height".to_string(),
				max_width_mm: 130.0,
				max_height_mm: -1.0,
			},
		)
		.await;

		assert!(invalid_height.is_err());
		assert!(
			invalid_height
				.expect_err("expected invalid height error")
				.contains("Max Height (mm) must be a positive number")
		);
	}

	#[tokio::test]
	async fn update_hoop_updates_existing_row() {
		let pool = test_pool().await;

		let created = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "130x180".to_string(),
				max_width_mm: 130.0,
				max_height_mm: 180.0,
			},
		)
		.await
		.expect("expected hoop to be created");

		let updated = update_hoop_with_pool(
			&pool,
			UpdateHoopRequest {
				hoop_id: created.id,
				name: "150x240".to_string(),
				max_width_mm: 150.0,
				max_height_mm: 240.0,
			},
		)
		.await
		.expect("expected hoop to update");

		assert_eq!(updated.id, created.id);
		assert_eq!(updated.name, "150x240");
		assert_eq!(updated.max_width_mm, 150.0);
		assert_eq!(updated.max_height_mm, 240.0);
		assert_eq!(updated.design_count, 0);
	}

	#[tokio::test]
	async fn update_hoop_rejects_duplicate_name_case_insensitive() {
		let pool = test_pool().await;

		let first = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "130x180".to_string(),
				max_width_mm: 130.0,
				max_height_mm: 180.0,
			},
		)
		.await
		.expect("expected first hoop to be created");

		let second = create_hoop_with_pool(
			&pool,
			CreateHoopRequest {
				name: "150x240".to_string(),
				max_width_mm: 150.0,
				max_height_mm: 240.0,
			},
		)
		.await
		.expect("expected second hoop to be created");

		let result = update_hoop_with_pool(
			&pool,
			UpdateHoopRequest {
				hoop_id: second.id,
				name: "130X180".to_string(),
				max_width_mm: 155.0,
				max_height_mm: 245.0,
			},
		)
		.await;

		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected duplicate hoop update error")
				.contains("already exists")
		);

		assert_eq!(first.name, "130x180");
	}

	#[tokio::test]
	async fn set_tag_group_rejects_not_found_tag_id() {
		let pool = test_pool().await;

		let result = set_tag_group_with_pool(
			&pool,
			SetTagGroupRequest {
				tag_id: 999,
				tag_group: "image".to_string(),
			},
		)
		.await;

		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected not-found tag id error")
				.contains("not found")
		);
	}

	#[tokio::test]
	async fn delete_designer_rejects_not_found_id() {
		let pool = test_pool().await;

		let result = delete_designer_with_pool(&pool, 777).await;
		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected not-found designer error")
				.contains("not found")
		);
	}

	#[tokio::test]
	async fn delete_source_rejects_not_found_id() {
		let pool = test_pool().await;

		let result = delete_source_with_pool(&pool, 555).await;
		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected not-found source error")
				.contains("not found")
		);
	}

	#[tokio::test]
	async fn delete_hoop_rejects_not_found_id() {
		let pool = test_pool().await;

		let result = delete_hoop_with_pool(&pool, 333).await;
		assert!(result.is_err());
		assert!(
			result
				.expect_err("expected not-found hoop error")
				.contains("not found")
		);
	}
}
