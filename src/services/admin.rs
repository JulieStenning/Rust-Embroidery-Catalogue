use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

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
    pub design_count: i64,
    pub is_system: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTagRequest {
    pub description: String,
    pub tag_group: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTagRequest {
    pub tag_id: i64,
    pub description: String,
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

pub fn validate_non_empty(value: &str, label: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid_input(format!("{label} is required.")));
    }
    Ok(trimmed.to_string())
}

pub fn validate_positive(value: f64, label: &str) -> Result<f64, AppError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(AppError::invalid_input(format!(
            "{label} must be a positive number."
        )));
    }
    Ok(value)
}

pub fn validate_tag_group(raw: &str) -> Result<String, AppError> {
    let group = raw.trim().to_lowercase();
    if group == "image" || group == "stitching" {
        Ok(group)
    } else {
        Err(AppError::invalid_input(
            "Tag group must be 'image' or 'stitching'.".to_string(),
        ))
    }
}

async fn ensure_unique_name(
    pool: &SqlitePool,
    table: &str,
    name: &str,
    label: &str,
) -> Result<(), AppError> {
    let sql = format!("SELECT 1 FROM {table} WHERE lower(name) = lower(?) LIMIT 1");
    let exists = sqlx::query_scalar::<_, i64>(&sql)
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?
        .is_some();

    if exists {
        Err(AppError::invalid_input(format!(
            "{label} '{name}' already exists."
        )))
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
) -> Result<(), AppError> {
    let sql =
        format!("SELECT 1 FROM {table} WHERE lower(name) = lower(?) AND {id_column} <> ? LIMIT 1");
    let exists = sqlx::query_scalar::<_, i64>(&sql)
        .bind(name)
        .bind(excluded_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?
        .is_some();

    if exists {
        Err(AppError::invalid_input(format!(
            "{label} '{name}' already exists."
        )))
    } else {
        Ok(())
    }
}

pub async fn list_designers_with_pool(pool: &SqlitePool) -> Result<Vec<AdminDesigner>, AppError> {
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
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(e.to_string()))
}

pub async fn create_designer_with_pool(
    pool: &SqlitePool,
    request: CreateDesignerRequest,
) -> Result<AdminDesigner, AppError> {
    let name = validate_non_empty(&request.name, "Designer name")?;
    ensure_unique_name(pool, "designers", &name, "Designer").await?;

    let result = sqlx::query("INSERT INTO designers (name) VALUES (?)")
        .bind(&name)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    Ok(AdminDesigner {
        id: result.last_insert_rowid(),
        name,
        design_count: 0,
    })
}

pub async fn update_designer_with_pool(
    pool: &SqlitePool,
    request: UpdateDesignerRequest,
) -> Result<AdminDesigner, AppError> {
    let name = validate_non_empty(&request.name, "Designer name")?;
    ensure_unique_name_except_id(
        pool,
        "designers",
        "id",
        request.designer_id,
        &name,
        "Designer",
    )
    .await?;

    let result = sqlx::query("UPDATE designers SET name = ? WHERE id = ?")
        .bind(&name)
        .bind(request.designer_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(
            "designer",
            Some(request.designer_id.to_string()),
        ));
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
    .map_err(|e| AppError::database(e.to_string()))?;

    Ok(row)
}

pub async fn delete_designer_with_pool(
    pool: &SqlitePool,
    designer_id: i64,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM designers WHERE id = ?")
        .bind(designer_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        Err(AppError::not_found(
            "designer",
            Some(designer_id.to_string()),
        ))
    } else {
        Ok(())
    }
}

pub async fn list_sources_with_pool(pool: &SqlitePool) -> Result<Vec<AdminSource>, AppError> {
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
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(e.to_string()))
}

pub async fn create_source_with_pool(
    pool: &SqlitePool,
    request: CreateSourceRequest,
) -> Result<AdminSource, AppError> {
    let name = validate_non_empty(&request.name, "Source name")?;
    ensure_unique_name(pool, "sources", &name, "Source").await?;

    let result = sqlx::query("INSERT INTO sources (name) VALUES (?)")
        .bind(&name)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    Ok(AdminSource {
        id: result.last_insert_rowid(),
        name,
        design_count: 0,
    })
}

pub async fn update_source_with_pool(
    pool: &SqlitePool,
    request: UpdateSourceRequest,
) -> Result<AdminSource, AppError> {
    let name = validate_non_empty(&request.name, "Source name")?;
    ensure_unique_name_except_id(pool, "sources", "id", request.source_id, &name, "Source").await?;

    let result = sqlx::query("UPDATE sources SET name = ? WHERE id = ?")
        .bind(&name)
        .bind(request.source_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(
            "source",
            Some(request.source_id.to_string()),
        ));
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
    .map_err(|e| AppError::database(e.to_string()))?;

    Ok(row)
}

pub async fn delete_source_with_pool(pool: &SqlitePool, source_id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM sources WHERE id = ?")
        .bind(source_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        Err(AppError::not_found("source", Some(source_id.to_string())))
    } else {
        Ok(())
    }
}

pub async fn list_tags_with_pool(pool: &SqlitePool) -> Result<Vec<AdminTag>, AppError> {
    sqlx::query_as::<_, AdminTag>(
        r#"
		SELECT
			t.id,
			t.description,
			t.tag_group,
			COUNT(dt.design_id) AS design_count,
			t.is_system AS is_system
		FROM tags t
		LEFT JOIN design_tags dt ON dt.tag_id = t.id
		GROUP BY t.id, t.description, t.tag_group, t.is_system
		ORDER BY t.description COLLATE NOCASE ASC
		"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(e.to_string()))
}

pub async fn create_tag_with_pool(
    pool: &SqlitePool,
    request: CreateTagRequest,
) -> Result<AdminTag, AppError> {
    let description = validate_non_empty(&request.description, "Tag description")?;
    let tag_group = validate_tag_group(&request.tag_group)?;

    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM tags WHERE lower(description) = lower(?) LIMIT 1",
    )
    .bind(&description)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::database(e.to_string()))?
    .is_some();

    if existing {
        return Err(AppError::invalid_input(format!(
            "Tag '{description}' already exists."
        )));
    }

    let result = sqlx::query("INSERT INTO tags (description, tag_group) VALUES (?, ?)")
        .bind(&description)
        .bind(&tag_group)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    Ok(AdminTag {
        id: result.last_insert_rowid(),
        description,
        tag_group: Some(tag_group),
        design_count: 0,
        is_system: false,
    })
}

/// Fetch the `is_system` flag for a tag and reject requests to modify or delete
/// system-defined tags. Returns `NotFound` if the tag does not exist.
async fn ensure_tag_not_system(pool: &SqlitePool, tag_id: i64) -> Result<(), AppError> {
    let is_system =
        sqlx::query_scalar::<_, bool>("SELECT is_system FROM tags WHERE id = ? LIMIT 1")
            .bind(tag_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::database(e.to_string()))?;

    match is_system {
        None => Err(AppError::not_found("tag", Some(tag_id.to_string()))),
        Some(true) => Err(AppError::invalid_input(
            "System tags cannot be modified or deleted.".to_string(),
        )),
        Some(false) => Ok(()),
    }
}

pub async fn update_tag_with_pool(
    pool: &SqlitePool,
    request: UpdateTagRequest,
) -> Result<AdminTag, AppError> {
    let description = validate_non_empty(&request.description, "Tag description")?;

    ensure_tag_not_system(pool, request.tag_id).await?;

    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM tags WHERE lower(description) = lower(?) AND id <> ? LIMIT 1",
    )
    .bind(&description)
    .bind(request.tag_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::database(e.to_string()))?
    .is_some();

    if existing {
        return Err(AppError::invalid_input(format!(
            "Tag '{description}' already exists."
        )));
    }

    let result = sqlx::query("UPDATE tags SET description = ? WHERE id = ?")
        .bind(&description)
        .bind(request.tag_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("tag", Some(request.tag_id.to_string())));
    }

    let row = sqlx::query_as::<_, AdminTag>(
        r#"
		SELECT
			t.id,
			t.description,
			t.tag_group,
			COUNT(dt.design_id) AS design_count,
			t.is_system AS is_system
		FROM tags t
		LEFT JOIN design_tags dt ON dt.tag_id = t.id
		WHERE t.id = ?
		GROUP BY t.id, t.description, t.tag_group, t.is_system
		LIMIT 1
		"#,
    )
    .bind(request.tag_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::database(e.to_string()))?;

    Ok(row)
}

pub async fn set_tag_group_with_pool(
    pool: &SqlitePool,
    request: SetTagGroupRequest,
) -> Result<AdminTag, AppError> {
    let tag_group = validate_tag_group(&request.tag_group)?;

    ensure_tag_not_system(pool, request.tag_id).await?;

    let result = sqlx::query("UPDATE tags SET tag_group = ? WHERE id = ?")
        .bind(&tag_group)
        .bind(request.tag_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("tag", Some(request.tag_id.to_string())));
    }

    let row = sqlx::query_as::<_, AdminTag>(
        r#"
		SELECT
			t.id,
			t.description,
			t.tag_group,
			COUNT(dt.design_id) AS design_count,
			t.is_system AS is_system
		FROM tags t
		LEFT JOIN design_tags dt ON dt.tag_id = t.id
		WHERE t.id = ?
		GROUP BY t.id, t.description, t.tag_group, t.is_system
		LIMIT 1
		"#,
    )
    .bind(request.tag_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::database(e.to_string()))?;

    Ok(row)
}

pub async fn delete_tag_with_pool(pool: &SqlitePool, tag_id: i64) -> Result<(), AppError> {
    ensure_tag_not_system(pool, tag_id).await?;

    let result = sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        Err(AppError::not_found("tag", Some(tag_id.to_string())))
    } else {
        Ok(())
    }
}

pub async fn list_hoops_with_pool(pool: &SqlitePool) -> Result<Vec<AdminHoop>, AppError> {
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
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(e.to_string()))
}

pub async fn create_hoop_with_pool(
    pool: &SqlitePool,
    request: CreateHoopRequest,
) -> Result<AdminHoop, AppError> {
    let name = validate_non_empty(&request.name, "Hoop name")?;
    let max_width_mm = validate_positive(request.max_width_mm, "Max Width (mm)")?;
    let max_height_mm = validate_positive(request.max_height_mm, "Max Height (mm)")?;
    ensure_unique_name(pool, "hoops", &name, "Hoop").await?;

    let result =
        sqlx::query("INSERT INTO hoops (name, max_width_mm, max_height_mm) VALUES (?, ?, ?)")
            .bind(&name)
            .bind(max_width_mm)
            .bind(max_height_mm)
            .execute(pool)
            .await
            .map_err(|e| AppError::database(e.to_string()))?;

    Ok(AdminHoop {
        id: result.last_insert_rowid(),
        name,
        max_width_mm,
        max_height_mm,
        design_count: 0,
    })
}

pub async fn update_hoop_with_pool(
    pool: &SqlitePool,
    request: UpdateHoopRequest,
) -> Result<AdminHoop, AppError> {
    let name = validate_non_empty(&request.name, "Hoop name")?;
    let max_width_mm = validate_positive(request.max_width_mm, "Max Width (mm)")?;
    let max_height_mm = validate_positive(request.max_height_mm, "Max Height (mm)")?;
    ensure_unique_name_except_id(pool, "hoops", "id", request.hoop_id, &name, "Hoop").await?;

    let result =
        sqlx::query("UPDATE hoops SET name = ?, max_width_mm = ?, max_height_mm = ? WHERE id = ?")
            .bind(&name)
            .bind(max_width_mm)
            .bind(max_height_mm)
            .bind(request.hoop_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(
            "hoop",
            Some(request.hoop_id.to_string()),
        ));
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
    .map_err(|e| AppError::database(e.to_string()))?;

    Ok(row)
}

pub async fn delete_hoop_with_pool(pool: &SqlitePool, hoop_id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM hoops WHERE id = ?")
        .bind(hoop_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        Err(AppError::not_found("hoop", Some(hoop_id.to_string())))
    } else {
        Ok(())
    }
}
#[cfg(test)]
#[path = "admin_tests.rs"]
mod tests;
