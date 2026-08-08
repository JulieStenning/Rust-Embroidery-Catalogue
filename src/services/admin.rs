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
        return Err(AppError::invalid_input(format!("{label} must be a positive number.")));
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
        Err(AppError::invalid_input(format!("{label} '{name}' already exists.")))
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
    let sql = format!(
        "SELECT 1 FROM {table} WHERE lower(name) = lower(?) AND {id_column} <> ? LIMIT 1"
    );
    let exists = sqlx::query_scalar::<_, i64>(&sql)
        .bind(name)
        .bind(excluded_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?
        .is_some();

    if exists {
        Err(AppError::invalid_input(format!("{label} '{name}' already exists.")))
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
    ensure_unique_name_except_id(pool, "designers", "id", request.designer_id, &name, "Designer").await?;

    let result = sqlx::query("UPDATE designers SET name = ? WHERE id = ?")
        .bind(&name)
        .bind(request.designer_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("designer", Some(request.designer_id.to_string())));
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

pub async fn delete_designer_with_pool(pool: &SqlitePool, designer_id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM designers WHERE id = ?")
        .bind(designer_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if result.rows_affected() == 0 {
        Err(AppError::not_found("designer", Some(designer_id.to_string())))
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
        return Err(AppError::not_found("source", Some(request.source_id.to_string())));
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
        return Err(AppError::invalid_input(format!("Tag '{description}' already exists.")));
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
    let is_system = sqlx::query_scalar::<_, bool>(
        "SELECT is_system FROM tags WHERE id = ? LIMIT 1",
    )
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
        return Err(AppError::invalid_input(format!("Tag '{description}' already exists.")));
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
        return Err(AppError::not_found("hoop", Some(request.hoop_id.to_string())));
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
mod tests {
    use super::*;
    use crate::database::migrations::run_migrations;
    use sqlx::SqlitePool;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    // ---------------------------------------------------------------------
    // validate_non_empty
    // ---------------------------------------------------------------------
    #[test]
    fn validate_non_empty_trims_and_accepts_value() {
        let result = validate_non_empty("  Hello World  ", "Label").unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn validate_non_empty_rejects_blank() {
        let err = validate_non_empty("   ", "Designer name").unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Designer name is required."
        ));
    }

    #[test]
    fn validate_non_empty_rejects_empty() {
        let err = validate_non_empty("", "Source name").unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Source name is required."
        ));
    }

    // ---------------------------------------------------------------------
    // validate_positive
    // ---------------------------------------------------------------------
    #[test]
    fn validate_positive_accepts_value() {
        let result = validate_positive(100.0, "Width").unwrap();
        assert_eq!(result, 100.0);
    }

    #[test]
    fn validate_positive_rejects_zero() {
        let err = validate_positive(0.0, "Max Width (mm)").unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Max Width (mm) must be a positive number."
        ));
    }

    #[test]
    fn validate_positive_rejects_negative() {
        let err = validate_positive(-5.0, "Max Height (mm)").unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Max Height (mm) must be a positive number."
        ));
    }

    #[test]
    fn validate_positive_rejects_nan() {
        let err = validate_positive(f64::NAN, "Width").unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Width must be a positive number."
        ));
    }

    #[test]
    fn validate_positive_rejects_infinity() {
        let err = validate_positive(f64::INFINITY, "Height").unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Height must be a positive number."
        ));
    }

    #[test]
    fn validate_positive_rejects_negative_infinity() {
        let err = validate_positive(f64::NEG_INFINITY, "Width").unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Width must be a positive number."
        ));
    }

    // ---------------------------------------------------------------------
    // validate_tag_group
    // ---------------------------------------------------------------------
    #[test]
    fn validate_tag_group_accepts_image() {
        assert_eq!(validate_tag_group("image").unwrap(), "image".to_string());
    }

    #[test]
    fn validate_tag_group_accepts_stitching() {
        assert_eq!(validate_tag_group("stitching").unwrap(), "stitching".to_string());
    }

    #[test]
    fn validate_tag_group_normalises_case_and_whitespace() {
        assert_eq!(validate_tag_group("  ImAgE  ").unwrap(), "image".to_string());
        assert_eq!(validate_tag_group("StItChInG").unwrap(), "stitching".to_string());
    }

    #[test]
    fn validate_tag_group_rejects_unknown() {
        let err = validate_tag_group("colour").unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Tag group must be 'image' or 'stitching'."
        ));
    }

    // ---------------------------------------------------------------------
    // ensure_unique_name / ensure_unique_name_except_id
    // ---------------------------------------------------------------------
    #[tokio::test]
    async fn ensure_unique_accepts_new_name() {
        let pool = setup().await;
        ensure_unique_name(&pool, "designers", "Fresh", "Designer")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn ensure_unique_rejects_duplicate_case_insensitive() {
        let pool = setup().await;
        let req = CreateDesignerRequest { name: "ACME".to_string() };
        create_designer_with_pool(&pool, req).await.unwrap();
        let err = ensure_unique_name(&pool, "designers", "acme", "Designer")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Designer 'acme' already exists."
        ));
    }

    #[tokio::test]
    async fn ensure_unique_except_id_ignores_excluded_row() {
        let pool = setup().await;
        let a = create_designer_with_pool(&pool, CreateDesignerRequest { name: "Alpha".to_string() })
            .await
            .unwrap();
        create_designer_with_pool(&pool, CreateDesignerRequest { name: "Beta".to_string() })
            .await
            .unwrap();

        // Renaming Alpha to "Alpha" (same row, different case) must be allowed.
        ensure_unique_name_except_id(&pool, "designers", "id", a.id, "alpha", "Designer")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn ensure_unique_except_id_rejects_other_row() {
        let pool = setup().await;
        let a = create_designer_with_pool(&pool, CreateDesignerRequest { name: "Alpha".to_string() })
            .await
            .unwrap();
        create_designer_with_pool(&pool, CreateDesignerRequest { name: "Beta".to_string() })
            .await
            .unwrap();

        let err = ensure_unique_name_except_id(&pool, "designers", "id", a.id, "BETA", "Designer")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::InvalidInput { message } if message == "Designer 'BETA' already exists."
        ));
    }

    // ---------------------------------------------------------------------
    // Designers
    // ---------------------------------------------------------------------
    #[tokio::test]
    async fn list_designers_empty() {
        let pool = setup().await;
        let result = list_designers_with_pool(&pool).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn create_and_list_designers() {
        let pool = setup().await;
        create_designer_with_pool(&pool, CreateDesignerRequest { name: "Zebra".to_string() })
            .await
            .unwrap();
        create_designer_with_pool(&pool, CreateDesignerRequest { name: "Alpha".to_string() })
            .await
            .unwrap();

        let list = list_designers_with_pool(&pool).await.unwrap();
        assert_eq!(list.len(), 2);
        // ORDER BY name COLLATE NOCASE ASC
        assert_eq!(list[0].name, "Alpha");
        assert_eq!(list[1].name, "Zebra");
        assert_eq!(list[0].design_count, 0);
    }

    #[tokio::test]
    async fn create_designer_success_and_count() {
        let pool = setup().await;
        let d = create_designer_with_pool(&pool, CreateDesignerRequest { name: "ACME".to_string() })
            .await
            .unwrap();
        assert!(d.id > 0);
        assert_eq!(d.name, "ACME");
        assert_eq!(d.design_count, 0);
    }

    #[tokio::test]
    async fn create_designer_empty_name_errors() {
        let pool = setup().await;
        let err = create_designer_with_pool(&pool, CreateDesignerRequest { name: "   ".to_string() })
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn create_designer_duplicate_case_insensitive_errors() {
        let pool = setup().await;
        create_designer_with_pool(&pool, CreateDesignerRequest { name: "ACME".to_string() })
            .await
            .unwrap();
        let err = create_designer_with_pool(&pool, CreateDesignerRequest { name: "acme".to_string() })
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn update_designer_success() {
        let pool = setup().await;
        let created = create_designer_with_pool(&pool, CreateDesignerRequest { name: "Old".to_string() })
            .await
            .unwrap();
        let updated = update_designer_with_pool(
            &pool,
            UpdateDesignerRequest { designer_id: created.id, name: "New".to_string() },
        )
        .await
        .unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "New");
    }

    #[tokio::test]
    async fn update_designer_not_found_errors() {
        let pool = setup().await;
        let err = update_designer_with_pool(
            &pool,
            UpdateDesignerRequest { designer_id: 9999, name: "Ghost".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[tokio::test]
    async fn update_designer_duplicate_errors() {
        let pool = setup().await;
        let a = create_designer_with_pool(&pool, CreateDesignerRequest { name: "Alpha".to_string() })
            .await
            .unwrap();
        create_designer_with_pool(&pool, CreateDesignerRequest { name: "Beta".to_string() })
            .await
            .unwrap();
        let err = update_designer_with_pool(
            &pool,
            UpdateDesignerRequest { designer_id: a.id, name: "beta".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn delete_designer_success_and_empty() {
        let pool = setup().await;
        let created = create_designer_with_pool(&pool, CreateDesignerRequest { name: "Temp".to_string() })
            .await
            .unwrap();
        delete_designer_with_pool(&pool, created.id).await.unwrap();
        let list = list_designers_with_pool(&pool).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn delete_designer_not_found_errors() {
        let pool = setup().await;
        let err = delete_designer_with_pool(&pool, 9999).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    // ---------------------------------------------------------------------
    // Sources
    // ---------------------------------------------------------------------
    #[tokio::test]
    async fn list_sources_empty() {
        let pool = setup().await;
        let result = list_sources_with_pool(&pool).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn create_and_list_sources() {
        let pool = setup().await;
        create_source_with_pool(&pool, CreateSourceRequest { name: "Zeta".to_string() })
            .await
            .unwrap();
        create_source_with_pool(&pool, CreateSourceRequest { name: "Alpha".to_string() })
            .await
            .unwrap();
        let list = list_sources_with_pool(&pool).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Alpha");
        assert_eq!(list[1].name, "Zeta");
    }

    #[tokio::test]
    async fn create_source_success() {
        let pool = setup().await;
        let s = create_source_with_pool(&pool, CreateSourceRequest { name: "Etsy".to_string() })
            .await
            .unwrap();
        assert!(s.id > 0);
        assert_eq!(s.name, "Etsy");
        assert_eq!(s.design_count, 0);
    }

    #[tokio::test]
    async fn create_source_empty_name_errors() {
        let pool = setup().await;
        let err = create_source_with_pool(&pool, CreateSourceRequest { name: "  ".to_string() })
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn create_source_duplicate_case_insensitive_errors() {
        let pool = setup().await;
        create_source_with_pool(&pool, CreateSourceRequest { name: "Etsy".to_string() })
            .await
            .unwrap();
        let err = create_source_with_pool(&pool, CreateSourceRequest { name: "ETSY".to_string() })
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn update_source_success() {
        let pool = setup().await;
        let created = create_source_with_pool(&pool, CreateSourceRequest { name: "Old".to_string() })
            .await
            .unwrap();
        let updated = update_source_with_pool(
            &pool,
            UpdateSourceRequest { source_id: created.id, name: "New".to_string() },
        )
        .await
        .unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "New");
    }

    #[tokio::test]
    async fn update_source_not_found_errors() {
        let pool = setup().await;
        let err = update_source_with_pool(
            &pool,
            UpdateSourceRequest { source_id: 9999, name: "Ghost".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[tokio::test]
    async fn update_source_duplicate_errors() {
        let pool = setup().await;
        let a = create_source_with_pool(&pool, CreateSourceRequest { name: "Alpha".to_string() })
            .await
            .unwrap();
        create_source_with_pool(&pool, CreateSourceRequest { name: "Beta".to_string() })
            .await
            .unwrap();
        let err = update_source_with_pool(
            &pool,
            UpdateSourceRequest { source_id: a.id, name: "beta".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn delete_source_success_and_empty() {
        let pool = setup().await;
        let created = create_source_with_pool(&pool, CreateSourceRequest { name: "Temp".to_string() })
            .await
            .unwrap();
        delete_source_with_pool(&pool, created.id).await.unwrap();
        let list = list_sources_with_pool(&pool).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn delete_source_not_found_errors() {
        let pool = setup().await;
        let err = delete_source_with_pool(&pool, 9999).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    // ---------------------------------------------------------------------
    // Tags
    // ---------------------------------------------------------------------
    #[tokio::test]
    async fn list_tags_contains_system_tags() {
        let pool = setup().await;
        let list = list_tags_with_pool(&pool).await.unwrap();
        assert!(!list.is_empty());
        assert!(list.iter().any(|t| t.is_system));
    }

    #[tokio::test]
    async fn create_tag_success() {
        let pool = setup().await;
        let tag = create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "Floral".to_string(), tag_group: "image".to_string() },
        )
        .await
        .unwrap();
        assert!(tag.id > 0);
        assert_eq!(tag.description, "Floral");
        assert_eq!(tag.tag_group, Some("image".to_string()));
        assert!(!tag.is_system);
        assert_eq!(tag.design_count, 0);
    }

    #[tokio::test]
    async fn create_tag_empty_description_errors() {
        let pool = setup().await;
        let err = create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "  ".to_string(), tag_group: "image".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn create_tag_invalid_group_errors() {
        let pool = setup().await;
        let err = create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "Test".to_string(), tag_group: "colour".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn create_tag_duplicate_case_insensitive_errors() {
        let pool = setup().await;
        create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "Floral".to_string(), tag_group: "image".to_string() },
        )
        .await
        .unwrap();
        let err = create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "floral".to_string(), tag_group: "stitching".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn update_tag_success() {
        let pool = setup().await;
        let created = create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "Old".to_string(), tag_group: "image".to_string() },
        )
        .await
        .unwrap();
        let updated = update_tag_with_pool(
            &pool,
            UpdateTagRequest { tag_id: created.id, description: "New".to_string() },
        )
        .await
        .unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.description, "New");
    }

    #[tokio::test]
    async fn update_tag_not_found_errors() {
        let pool = setup().await;
        let err = update_tag_with_pool(
            &pool,
            UpdateTagRequest { tag_id: 9999, description: "Ghost".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[tokio::test]
    async fn update_tag_duplicate_errors() {
        let pool = setup().await;
        let a = create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "Alpha".to_string(), tag_group: "image".to_string() },
        )
        .await
        .unwrap();
        create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "Beta".to_string(), tag_group: "image".to_string() },
        )
        .await
        .unwrap();
        let err = update_tag_with_pool(
            &pool,
            UpdateTagRequest { tag_id: a.id, description: "beta".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn update_tag_system_tag_rejected() {
        let pool = setup().await;
        let all = list_tags_with_pool(&pool).await.unwrap();
        let system = all.iter().find(|t| t.is_system).expect("system tag exists");
        let err = update_tag_with_pool(
            &pool,
            UpdateTagRequest { tag_id: system.id, description: "Hacked".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn set_tag_group_success() {
        let pool = setup().await;
        let created = create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "Group Test".to_string(), tag_group: "image".to_string() },
        )
        .await
        .unwrap();
        let updated = set_tag_group_with_pool(
            &pool,
            SetTagGroupRequest { tag_id: created.id, tag_group: "stitching".to_string() },
        )
        .await
        .unwrap();
        assert_eq!(updated.tag_group, Some("stitching".to_string()));
    }

    #[tokio::test]
    async fn set_tag_group_not_found_errors() {
        let pool = setup().await;
        let err = set_tag_group_with_pool(
            &pool,
            SetTagGroupRequest { tag_id: 9999, tag_group: "image".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[tokio::test]
    async fn set_tag_group_system_tag_rejected() {
        let pool = setup().await;
        let all = list_tags_with_pool(&pool).await.unwrap();
        let system = all.iter().find(|t| t.is_system).expect("system tag exists");
        let err = set_tag_group_with_pool(
            &pool,
            SetTagGroupRequest { tag_id: system.id, tag_group: "image".to_string() },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn delete_tag_success() {
        let pool = setup().await;
        let created = create_tag_with_pool(
            &pool,
            CreateTagRequest { description: "DeleteMe".to_string(), tag_group: "image".to_string() },
        )
        .await
        .unwrap();
        delete_tag_with_pool(&pool, created.id).await.unwrap();
        let list = list_tags_with_pool(&pool).await.unwrap();
        assert!(!list.iter().any(|t| t.id == created.id));
    }

    #[tokio::test]
    async fn delete_tag_not_found_errors() {
        let pool = setup().await;
        let err = delete_tag_with_pool(&pool, 9999).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[tokio::test]
    async fn delete_tag_system_tag_rejected() {
        let pool = setup().await;
        let all = list_tags_with_pool(&pool).await.unwrap();
        let system = all.iter().find(|t| t.is_system).expect("system tag exists");
        let err = delete_tag_with_pool(&pool, system.id).await.unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    // ---------------------------------------------------------------------
    // Hoops
    // ---------------------------------------------------------------------
    #[tokio::test]
    async fn list_hoops_empty() {
        let pool = setup().await;
        let result = list_hoops_with_pool(&pool).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn create_and_list_hoops_ordered_by_size() {
        let pool = setup().await;
        create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "5x7".to_string(), max_width_mm: 200.0, max_height_mm: 300.0 },
        )
        .await
        .unwrap();
        create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "4x4".to_string(), max_width_mm: 100.0, max_height_mm: 100.0 },
        )
        .await
        .unwrap();
        let list = list_hoops_with_pool(&pool).await.unwrap();
        assert_eq!(list.len(), 2);
        // ORDER BY max_width_mm ASC
        assert_eq!(list[0].name, "4x4");
        assert_eq!(list[1].name, "5x7");
    }

    #[tokio::test]
    async fn create_hoop_success() {
        let pool = setup().await;
        let h = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "4x4".to_string(), max_width_mm: 100.0, max_height_mm: 100.0 },
        )
        .await
        .unwrap();
        assert!(h.id > 0);
        assert_eq!(h.name, "4x4");
        assert_eq!(h.max_width_mm, 100.0);
        assert_eq!(h.max_height_mm, 100.0);
        assert_eq!(h.design_count, 0);
    }

    #[tokio::test]
    async fn create_hoop_empty_name_errors() {
        let pool = setup().await;
        let err = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "  ".to_string(), max_width_mm: 100.0, max_height_mm: 100.0 },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn create_hoop_invalid_dimensions_errors() {
        let pool = setup().await;
        let err = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "Bad".to_string(), max_width_mm: 0.0, max_height_mm: 100.0 },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));

        let err = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "Bad2".to_string(), max_width_mm: 100.0, max_height_mm: f64::NAN },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn create_hoop_duplicate_case_insensitive_errors() {
        let pool = setup().await;
        create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "4x4".to_string(), max_width_mm: 100.0, max_height_mm: 100.0 },
        )
        .await
        .unwrap();
        let err = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "4X4".to_string(), max_width_mm: 200.0, max_height_mm: 200.0 },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn update_hoop_success() {
        let pool = setup().await;
        let created = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "Old".to_string(), max_width_mm: 10.0, max_height_mm: 10.0 },
        )
        .await
        .unwrap();
        let updated = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: created.id,
                name: "New Hoop".to_string(),
                max_width_mm: 200.0,
                max_height_mm: 200.0,
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "New Hoop");
        assert_eq!(updated.max_width_mm, 200.0);
        assert_eq!(updated.max_height_mm, 200.0);
    }

    #[tokio::test]
    async fn update_hoop_not_found_errors() {
        let pool = setup().await;
        let err = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: 9999,
                name: "Ghost".to_string(),
                max_width_mm: 10.0,
                max_height_mm: 10.0,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[tokio::test]
    async fn update_hoop_invalid_dimensions_errors() {
        let pool = setup().await;
        let created = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "Valid".to_string(), max_width_mm: 10.0, max_height_mm: 10.0 },
        )
        .await
        .unwrap();
        let err = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: created.id,
                name: "New".to_string(),
                max_width_mm: -1.0,
                max_height_mm: 10.0,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn update_hoop_duplicate_errors() {
        let pool = setup().await;
        let a = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "Alpha".to_string(), max_width_mm: 1.0, max_height_mm: 1.0 },
        )
        .await
        .unwrap();
        create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "Beta".to_string(), max_width_mm: 2.0, max_height_mm: 2.0 },
        )
        .await
        .unwrap();
        let err = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: a.id,
                name: "beta".to_string(),
                max_width_mm: 3.0,
                max_height_mm: 3.0,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn delete_hoop_success_and_empty() {
        let pool = setup().await;
        let created = create_hoop_with_pool(
            &pool,
            CreateHoopRequest { name: "Temp".to_string(), max_width_mm: 1.0, max_height_mm: 1.0 },
        )
        .await
        .unwrap();
        delete_hoop_with_pool(&pool, created.id).await.unwrap();
        let list = list_hoops_with_pool(&pool).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn delete_hoop_not_found_errors() {
        let pool = setup().await;
        let err = delete_hoop_with_pool(&pool, 9999).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }
}
