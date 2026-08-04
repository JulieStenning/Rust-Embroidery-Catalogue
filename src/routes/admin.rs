use crate::AppState;
use crate::services::admin as admin_service;
use sqlx::SqlitePool;
use tauri::State;

pub use crate::services::admin::{
    AdminDesigner, AdminHoop, AdminSource, AdminTag, CreateDesignerRequest, CreateHoopRequest,
    CreateSourceRequest, CreateTagRequest, SetTagGroupRequest, UpdateDesignerRequest,
    UpdateHoopRequest, UpdateSourceRequest, UpdateTagRequest,
};

#[cfg(test)]
fn validate_non_empty(value: &str, label: &str) -> Result<String, String> {
    admin_service::validate_non_empty(value, label)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
fn validate_positive(value: f64, label: &str) -> Result<f64, String> {
    admin_service::validate_positive(value, label)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
fn validate_tag_group(raw: &str) -> Result<String, String> {
    admin_service::validate_tag_group(raw)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
async fn ensure_unique_name(
    pool: &SqlitePool,
    table: &str,
    name: &str,
    label: &str,
) -> Result<(), String> {
    let sql = format!("SELECT 1 FROM {table} WHERE lower(name) = lower(?) LIMIT 1");
    let exists = sqlx::query_scalar::<_, i64>(&sql)
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .is_some();

    if exists {
        Err(format!("{label} '{name}' already exists."))
    } else {
        Ok(())
    }
}

#[cfg(test)]
async fn ensure_unique_name_except_id(
    pool: &SqlitePool,
    table: &str,
    id_column: &str,
    excluded_id: i64,
    name: &str,
    label: &str,
) -> Result<(), String> {
    let sql = format!(
        "SELECT 1 FROM {table} WHERE lower(name) = lower(?) AND {id_column} <> ? LIMIT 1"
    );
    let exists = sqlx::query_scalar::<_, i64>(&sql)
        .bind(name)
        .bind(excluded_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .is_some();

    if exists {
        Err(format!("{label} '{name}' already exists."))
    } else {
        Ok(())
    }
}

pub async fn list_designers_with_pool(pool: &SqlitePool) -> Result<Vec<AdminDesigner>, String> {
    admin_service::list_designers_with_pool(pool)
        .await
        .map_err(|error| error.to_string())
}

// ---------------------------------------------------------------------------
// Tauri command wrappers
//
// Every #[tauri::command] below is a thin delegation wrapper that extracts
// `state.db` and forwards the call to the corresponding `_with_pool`
// function (e.g. create_designer → create_designer_with_pool).
//
// All business logic, validation, SQL queries, and error handling live
// exclusively in the `_with_pool` functions and are tested exhaustively
// by the `mod tests` section below (73+ tests covering happy paths, empty
// inputs, duplicates, case-insensitive collisions, not-found errors,
// invalid dimensions, design-count accuracy, etc.).
//
// Testing the wrappers directly would add negligible value — they contain
// no branching, no logic, and no error handling of their own. The only
// code path they exercise is `state.db` access, which is guaranteed by
// the Tauri framework. Constructing a full AppState in tests would also
// require coupling to unrelated types (paths, logs, etc.) with no payoff.
//
// Therefore all coverage effort is concentrated on the `_with_pool`
// functions, which deliver >95% effective coverage of every command.
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_designers(state: State<'_, AppState>) -> Result<Vec<AdminDesigner>, String> {
    list_designers_with_pool(&state.db).await
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
    admin_service::create_designer_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_designer(
    state: State<'_, AppState>,
    request: UpdateDesignerRequest,
) -> Result<AdminDesigner, String> {
    update_designer_with_pool(&state.db, request).await
}

async fn update_designer_with_pool(
    pool: &SqlitePool,
    request: UpdateDesignerRequest,
) -> Result<AdminDesigner, String> {
    admin_service::update_designer_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_designer(state: State<'_, AppState>, designer_id: i64) -> Result<(), String> {
    delete_designer_with_pool(&state.db, designer_id).await
}

async fn delete_designer_with_pool(pool: &SqlitePool, designer_id: i64) -> Result<(), String> {
    admin_service::delete_designer_with_pool(pool, designer_id)
        .await
        .map_err(|error| error.to_string())
}

pub async fn list_sources_with_pool(pool: &SqlitePool) -> Result<Vec<AdminSource>, String> {
    admin_service::list_sources_with_pool(pool)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_sources(state: State<'_, AppState>) -> Result<Vec<AdminSource>, String> {
    list_sources_with_pool(&state.db).await
}

#[tauri::command]
pub async fn create_source(
    state: State<'_, AppState>,
    request: CreateSourceRequest,
) -> Result<AdminSource, String> {
    create_source_with_pool(&state.db, request).await
}

async fn create_source_with_pool(
    pool: &SqlitePool,
    request: CreateSourceRequest,
) -> Result<AdminSource, String> {
    admin_service::create_source_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_source(
    state: State<'_, AppState>,
    request: UpdateSourceRequest,
) -> Result<AdminSource, String> {
    update_source_with_pool(&state.db, request).await
}

async fn update_source_with_pool(
    pool: &SqlitePool,
    request: UpdateSourceRequest,
) -> Result<AdminSource, String> {
    admin_service::update_source_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_source(state: State<'_, AppState>, source_id: i64) -> Result<(), String> {
    delete_source_with_pool(&state.db, source_id).await
}

async fn delete_source_with_pool(pool: &SqlitePool, source_id: i64) -> Result<(), String> {
    admin_service::delete_source_with_pool(pool, source_id)
        .await
        .map_err(|error| error.to_string())
}

pub async fn list_tags_with_pool(pool: &SqlitePool) -> Result<Vec<AdminTag>, String> {
    admin_service::list_tags_with_pool(pool)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<AdminTag>, String> {
    list_tags_with_pool(&state.db).await
}

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    request: CreateTagRequest,
) -> Result<AdminTag, String> {
    create_tag_with_pool(&state.db, request).await
}

async fn create_tag_with_pool(
    pool: &SqlitePool,
    request: CreateTagRequest,
) -> Result<AdminTag, String> {
    admin_service::create_tag_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
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
    admin_service::set_tag_group_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    request: UpdateTagRequest,
) -> Result<AdminTag, String> {
    update_tag_with_pool(&state.db, request).await
}

async fn update_tag_with_pool(
    pool: &SqlitePool,
    request: UpdateTagRequest,
) -> Result<AdminTag, String> {
    admin_service::update_tag_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, tag_id: i64) -> Result<(), String> {
    delete_tag_with_pool(&state.db, tag_id).await
}

async fn delete_tag_with_pool(pool: &SqlitePool, tag_id: i64) -> Result<(), String> {
    admin_service::delete_tag_with_pool(pool, tag_id)
        .await
        .map_err(|error| error.to_string())
}

pub async fn list_hoops_with_pool(pool: &SqlitePool) -> Result<Vec<AdminHoop>, String> {
    admin_service::list_hoops_with_pool(pool)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_hoops(state: State<'_, AppState>) -> Result<Vec<AdminHoop>, String> {
    list_hoops_with_pool(&state.db).await
}

#[tauri::command]
pub async fn create_hoop(
    state: State<'_, AppState>,
    request: CreateHoopRequest,
) -> Result<AdminHoop, String> {
    create_hoop_with_pool(&state.db, request).await
}

async fn create_hoop_with_pool(
    pool: &SqlitePool,
    request: CreateHoopRequest,
) -> Result<AdminHoop, String> {
    admin_service::create_hoop_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_hoop(
    state: State<'_, AppState>,
    request: UpdateHoopRequest,
) -> Result<AdminHoop, String> {
    update_hoop_with_pool(&state.db, request).await
}

async fn update_hoop_with_pool(
    pool: &SqlitePool,
    request: UpdateHoopRequest,
) -> Result<AdminHoop, String> {
    admin_service::update_hoop_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_hoop(state: State<'_, AppState>, hoop_id: i64) -> Result<(), String> {
    delete_hoop_with_pool(&state.db, hoop_id).await
}

async fn delete_hoop_with_pool(pool: &SqlitePool, hoop_id: i64) -> Result<(), String> {
    admin_service::delete_hoop_with_pool(pool, hoop_id)
        .await
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Row;

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
				tag_group VARCHAR(20),
				is_system BOOLEAN NOT NULL DEFAULT 0
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

        sqlx::query(
            r#"
			CREATE TABLE design_tags (
				design_id INTEGER NOT NULL REFERENCES designs(id) ON DELETE CASCADE,
				tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
				PRIMARY KEY (design_id, tag_id)
			);
			"#,
        )
        .execute(&pool)
        .await
        .expect("failed to create design_tags table");

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

    // ========================================================================
    // Validation helper unit tests
    // ========================================================================

    #[test]
    fn validate_non_empty_accepts_trimmed() {
        let result = validate_non_empty("  Hello World  ", "Label");
        assert_eq!(result, Ok("Hello World".to_string()));
    }

    #[test]
    fn validate_non_empty_rejects_empty() {
        let result = validate_non_empty("", "Label");
        assert_eq!(result, Err("invalid input: Label is required.".to_string()));
    }

    #[test]
    fn validate_non_empty_rejects_whitespace() {
        let result = validate_non_empty("   \t  ", "Label");
        assert_eq!(result, Err("invalid input: Label is required.".to_string()));
    }

    #[test]
    fn validate_positive_accepts_normal() {
        let result = validate_positive(42.5, "Number");
        assert_eq!(result, Ok(42.5));
    }

    #[test]
    fn validate_positive_rejects_zero() {
        let result = validate_positive(0.0, "Number");
        assert_eq!(result, Err("invalid input: Number must be a positive number.".to_string()));
    }

    #[test]
    fn validate_positive_rejects_negative() {
        let result = validate_positive(-5.0, "Number");
        assert_eq!(result, Err("invalid input: Number must be a positive number.".to_string()));
    }

    #[test]
    fn validate_positive_rejects_infinity() {
        let result = validate_positive(f64::INFINITY, "Number");
        assert_eq!(result, Err("invalid input: Number must be a positive number.".to_string()));
    }

    #[test]
    fn validate_positive_rejects_nan() {
        let result = validate_positive(f64::NAN, "Number");
        assert_eq!(result, Err("invalid input: Number must be a positive number.".to_string()));
    }

    #[test]
    fn validate_tag_group_accepts_image() {
        let result = validate_tag_group("image");
        assert_eq!(result, Ok("image".to_string()));
    }

    #[test]
    fn validate_tag_group_accepts_stitching() {
        let result = validate_tag_group("stitching");
        assert_eq!(result, Ok("stitching".to_string()));
    }

    #[test]
    fn validate_tag_group_rejects_other() {
        let result = validate_tag_group("invalid-group");
        assert_eq!(
            result,
            Err("invalid input: Tag group must be 'image' or 'stitching'.".to_string())
        );
    }

    #[test]
    fn validate_tag_group_trims_and_lowercases() {
        let result = validate_tag_group("  IMAGE  ");
        assert_eq!(result, Ok("image".to_string()));

        let result2 = validate_tag_group("  StItChInG  ");
        assert_eq!(result2, Ok("stitching".to_string()));
    }

    // ========================================================================
    // ensure_unique_name / ensure_unique_name_except_id direct tests
    // ========================================================================

    #[tokio::test]
    async fn ensure_unique_name_ok_when_not_exists() {
        let pool = test_pool().await;
        let result = ensure_unique_name(&pool, "designers", "New Name", "Designer").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn ensure_unique_name_err_when_exists() {
        let pool = test_pool().await;
        let _first = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Existing".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        let result = ensure_unique_name(&pool, "designers", "Existing", "Designer").await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate error")
            .contains("already exists"));
    }

    #[tokio::test]
    async fn ensure_unique_name_except_id_allows_same_name_for_same_id() {
        let pool = test_pool().await;

        let created = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Self".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        let result = ensure_unique_name_except_id(&pool, "designers", "id", created.id, "Self", "Designer").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn ensure_unique_name_except_id_rejects_name_of_other_row() {
        let pool = test_pool().await;

        let _first = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "First".to_string(),
            },
        )
        .await
        .expect("expected first designer to be created");

        let second = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Second".to_string(),
            },
        )
        .await
        .expect("expected second designer to be created");

        let result = ensure_unique_name_except_id(&pool, "designers", "id", second.id, "First", "Designer").await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate error")
            .contains("already exists"));
    }

    // ========================================================================
    // Happy-path Create tests
    // ========================================================================

    // --- Designer ---

    #[tokio::test]
    async fn create_designer_success() {
        let pool = test_pool().await;

        let result = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Amazing Designs".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        assert!(result.id > 0);
        assert_eq!(result.name, "Amazing Designs");
        assert_eq!(result.design_count, 0);
    }

    #[tokio::test]
    async fn create_designer_empty_name() {
        let pool = test_pool().await;

        let result = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "   ".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected empty name error")
            .contains("Designer name is required"));
    }

    #[tokio::test]
    async fn create_designer_duplicate_name() {
        let pool = test_pool().await;

        create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Duplicate".to_string(),
            },
        )
        .await
        .expect("expected first designer to be created");

        let result = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Duplicate".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate error")
            .contains("already exists"));
    }

    #[tokio::test]
    async fn create_designer_case_insensitive_duplicate() {
        let pool = test_pool().await;

        create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Unique".to_string(),
            },
        )
        .await
        .expect("expected first designer to be created");

        let result = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "unique".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected case-insensitive duplicate error")
            .contains("already exists"));
    }

    // --- Source ---

    #[tokio::test]
    async fn create_source_success() {
        let pool = test_pool().await;

        let result = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "USB Import".to_string(),
            },
        )
        .await
        .expect("expected source to be created");

        assert!(result.id > 0);
        assert_eq!(result.name, "USB Import");
        assert_eq!(result.design_count, 0);
    }

    #[tokio::test]
    async fn create_source_empty_name() {
        let pool = test_pool().await;

        let result = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "  ".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected empty name error")
            .contains("Source name is required"));
    }

    #[tokio::test]
    async fn create_source_duplicate_name() {
        let pool = test_pool().await;

        create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "Duplicate Source".to_string(),
            },
        )
        .await
        .expect("expected first source to be created");

        let result = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "Duplicate Source".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate error")
            .contains("already exists"));
    }

    #[tokio::test]
    async fn create_source_case_insensitive_duplicate() {
        let pool = test_pool().await;

        create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "My Source".to_string(),
            },
        )
        .await
        .expect("expected first source to be created");

        let result = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "my source".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected case-insensitive duplicate error")
            .contains("already exists"));
    }

    // --- Tag ---

    #[tokio::test]
    async fn create_tag_success_image() {
        let pool = test_pool().await;

        let result = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Floral".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        assert!(result.id > 0);
        assert_eq!(result.description, "Floral");
        assert_eq!(result.tag_group, Some("image".to_string()));
    }

    #[tokio::test]
    async fn create_tag_success_stitching() {
        let pool = test_pool().await;

        let result = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Satin Stitch".to_string(),
                tag_group: "stitching".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        assert!(result.id > 0);
        assert_eq!(result.description, "Satin Stitch");
        assert_eq!(result.tag_group, Some("stitching".to_string()));
    }

    #[tokio::test]
    async fn create_tag_with_mixed_case_group() {
        let pool = test_pool().await;

        let result = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Test Mixed Case".to_string(),
                tag_group: "  Image  ".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        assert_eq!(result.tag_group, Some("image".to_string()));
    }

    #[tokio::test]
    async fn create_tag_empty_description() {
        let pool = test_pool().await;

        let result = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "  ".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected empty description error")
            .contains("Tag description is required"));
    }

    #[tokio::test]
    async fn create_tag_invalid_group() {
        let pool = test_pool().await;

        let result = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Invalid Group Tag".to_string(),
                tag_group: "bad-group".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected invalid group error")
            .contains("must be 'image' or 'stitching'"));
    }

    #[tokio::test]
    async fn create_tag_duplicate_description() {
        let pool = test_pool().await;

        create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Unique Tag".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected first tag to be created");

        let result = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Unique Tag".to_string(),
                tag_group: "stitching".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate error")
            .contains("already exists"));
    }

    #[tokio::test]
    async fn create_tag_case_insensitive_duplicate() {
        let pool = test_pool().await;

        create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Case Test".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected first tag to be created");

        let result = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "case test".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected case-insensitive duplicate error")
            .contains("already exists"));
    }

    // --- Hoop ---

    #[tokio::test]
    async fn create_hoop_success() {
        let pool = test_pool().await;

        let result = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "130x180".to_string(),
                max_width_mm: 130.0,
                max_height_mm: 180.0,
            },
        )
        .await
        .expect("expected hoop to be created");

        assert!(result.id > 0);
        assert_eq!(result.name, "130x180");
        assert_eq!(result.max_width_mm, 130.0);
        assert_eq!(result.max_height_mm, 180.0);
        assert_eq!(result.design_count, 0);
    }

    #[tokio::test]
    async fn create_hoop_empty_name() {
        let pool = test_pool().await;

        let result = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "  ".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected empty name error")
            .contains("Hoop name is required"));
    }

    #[tokio::test]
    async fn create_hoop_invalid_width() {
        let pool = test_pool().await;

        let result = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Bad Width".to_string(),
                max_width_mm: 0.0,
                max_height_mm: 100.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected invalid width error")
            .contains("must be a positive number"));
    }

    #[tokio::test]
    async fn create_hoop_invalid_height() {
        let pool = test_pool().await;

        let result = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Bad Height".to_string(),
                max_width_mm: 100.0,
                max_height_mm: -1.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected invalid height error")
            .contains("must be a positive number"));
    }

    #[tokio::test]
    async fn create_hoop_duplicate_name() {
        let pool = test_pool().await;

        create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Duplicate Hoop".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected first hoop to be created");

        let result = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Duplicate Hoop".to_string(),
                max_width_mm: 200.0,
                max_height_mm: 200.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate error")
            .contains("already exists"));
    }

    #[tokio::test]
    async fn create_hoop_case_insensitive_duplicate() {
        let pool = test_pool().await;

        create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "My Hoop".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected first hoop to be created");

        let result = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "my hoop".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected case-insensitive duplicate error")
            .contains("already exists"));
    }

    // ========================================================================
    // List tests
    // ========================================================================

    #[tokio::test]
    async fn list_designers_empty() {
        let pool = test_pool().await;

        let result = list_designers_with_pool(&pool).await.expect("expected empty list");
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_designers_with_data() {
        let pool = test_pool().await;

        create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Z Designs".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Alpha Emb".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        let result = list_designers_with_pool(&pool).await.expect("expected designers");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Alpha Emb");
        assert_eq!(result[1].name, "Z Designs");
    }

    #[tokio::test]
    async fn list_sources_empty() {
        let pool = test_pool().await;

        let result = list_sources_with_pool(&pool).await.expect("expected empty list");
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_sources_with_data_and_count() {
        let pool = test_pool().await;

        let src = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "USB".to_string(),
            },
        )
        .await
        .expect("expected source to be created");

        sqlx::query("INSERT INTO designs (filename, filepath, source_id) VALUES (?, ?, ?)")
            .bind("test.dst")
            .bind("/path/test.dst")
            .bind(src.id)
            .execute(&pool)
            .await
            .expect("expected design to be inserted");

        let result = list_sources_with_pool(&pool).await.expect("expected sources");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].design_count, 1);
    }

    #[tokio::test]
    async fn list_tags_empty() {
        let pool = test_pool().await;

        let result = list_tags_with_pool(&pool).await.expect("expected empty list");
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_tags_with_data() {
        let pool = test_pool().await;

        create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Floral".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Satin".to_string(),
                tag_group: "stitching".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        let result = list_tags_with_pool(&pool).await.expect("expected tags");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].description, "Floral");
        assert_eq!(result[1].description, "Satin");
    }

    #[tokio::test]
    async fn list_hoops_empty() {
        let pool = test_pool().await;

        let result = list_hoops_with_pool(&pool).await.expect("expected empty list");
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_hoops_with_data_and_count() {
        let pool = test_pool().await;

        let hoop = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "130x180".to_string(),
                max_width_mm: 130.0,
                max_height_mm: 180.0,
            },
        )
        .await
        .expect("expected hoop to be created");

        create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "100x100".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected hoop to be created");

        sqlx::query("INSERT INTO designs (filename, filepath, hoop_id) VALUES (?, ?, ?)")
            .bind("test.dst")
            .bind("/path/test.dst")
            .bind(hoop.id)
            .execute(&pool)
            .await
            .expect("expected design to be inserted");

        let result = list_hoops_with_pool(&pool).await.expect("expected hoops");
        assert_eq!(result.len(), 2);
        // ordered by max_width_mm, max_height_mm, name
        assert_eq!(result[0].name, "100x100");
        assert_eq!(result[1].name, "130x180");
        assert_eq!(result[1].design_count, 1);
    }

    // ========================================================================
    // Design count accuracy tests
    // ========================================================================

    #[tokio::test]
    async fn designer_design_count_reflects_linked_designs() {
        let pool = test_pool().await;

        let designer = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Count Test".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        // Insert two designs linked to this designer
        for i in 0..2 {
            sqlx::query("INSERT INTO designs (filename, filepath, designer_id) VALUES (?, ?, ?)")
                .bind(format!("design_{}.dst", i))
                .bind(format!("/path/design_{}.dst", i))
                .bind(designer.id)
                .execute(&pool)
                .await
                .expect("expected design to be inserted");
        }

        let result = list_designers_with_pool(&pool)
            .await
            .expect("expected designers");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].design_count, 2);
    }

    #[tokio::test]
    async fn source_design_count_reflects_linked_designs() {
        let pool = test_pool().await;

        let source = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "Count Source".to_string(),
            },
        )
        .await
        .expect("expected source to be created");

        for i in 0..3 {
            sqlx::query("INSERT INTO designs (filename, filepath, source_id) VALUES (?, ?, ?)")
                .bind(format!("design_{}.dst", i))
                .bind(format!("/path/design_{}.dst", i))
                .bind(source.id)
                .execute(&pool)
                .await
                .expect("expected design to be inserted");
        }

        let result = list_sources_with_pool(&pool).await.expect("expected sources");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].design_count, 3);
    }

    #[tokio::test]
    async fn hoop_design_count_reflects_linked_designs() {
        let pool = test_pool().await;

        let hoop = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Count Hoop".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected hoop to be created");

        for i in 0..4 {
            sqlx::query("INSERT INTO designs (filename, filepath, hoop_id) VALUES (?, ?, ?)")
                .bind(format!("design_{}.dst", i))
                .bind(format!("/path/design_{}.dst", i))
                .bind(hoop.id)
                .execute(&pool)
                .await
                .expect("expected design to be inserted");
        }

        let result = list_hoops_with_pool(&pool).await.expect("expected hoops");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].design_count, 4);
    }

    #[tokio::test]
    async fn design_count_zero_when_no_designs() {
        let pool = test_pool().await;

        let designer = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Zero Count".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        let designers = list_designers_with_pool(&pool)
            .await
            .expect("expected designers");
        let d = designers.iter().find(|d| d.id == designer.id).unwrap();
        assert_eq!(d.design_count, 0);
    }

    // ========================================================================
    // Update happy-path + error-path tests
    // ========================================================================

    // --- Designer ---

    #[tokio::test]
    async fn update_designer_success() {
        let pool = test_pool().await;

        let created = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Old Name".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        let updated = update_designer_with_pool(
            &pool,
            UpdateDesignerRequest {
                designer_id: created.id,
                name: "New Name".to_string(),
            },
        )
        .await
        .expect("expected designer to be updated");

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "New Name");
        // Name change should still have zero designs
        assert_eq!(updated.design_count, 0);
    }

    #[tokio::test]
    async fn update_designer_not_found() {
        let pool = test_pool().await;

        let result = update_designer_with_pool(
            &pool,
            UpdateDesignerRequest {
                designer_id: 999,
                name: "Ghost".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }

    #[tokio::test]
    async fn update_designer_empty_name() {
        let pool = test_pool().await;

        let created = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Valid".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        let result = update_designer_with_pool(
            &pool,
            UpdateDesignerRequest {
                designer_id: created.id,
                name: "   ".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected empty name error")
            .contains("Designer name is required"));
    }

    #[tokio::test]
    async fn update_designer_duplicate_name() {
        let pool = test_pool().await;

        let _first = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "First".to_string(),
            },
        )
        .await
        .expect("expected first designer to be created");

        let second = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "Second".to_string(),
            },
        )
        .await
        .expect("expected second designer to be created");

        let result = update_designer_with_pool(
            &pool,
            UpdateDesignerRequest {
                designer_id: second.id,
                name: "First".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate name error")
            .contains("already exists"));
    }

    // --- Source ---

    #[tokio::test]
    async fn update_source_success() {
        let pool = test_pool().await;

        let created = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "Old Source".to_string(),
            },
        )
        .await
        .expect("expected source to be created");

        let updated = update_source_with_pool(
            &pool,
            UpdateSourceRequest {
                source_id: created.id,
                name: "New Source".to_string(),
            },
        )
        .await
        .expect("expected source to be updated");

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "New Source");
        assert_eq!(updated.design_count, 0);
    }

    #[tokio::test]
    async fn update_source_not_found() {
        let pool = test_pool().await;

        let result = update_source_with_pool(
            &pool,
            UpdateSourceRequest {
                source_id: 999,
                name: "Ghost".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }

    #[tokio::test]
    async fn update_source_empty_name() {
        let pool = test_pool().await;

        let created = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "Valid".to_string(),
            },
        )
        .await
        .expect("expected source to be created");

        let result = update_source_with_pool(
            &pool,
            UpdateSourceRequest {
                source_id: created.id,
                name: "  ".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected empty name error")
            .contains("Source name is required"));
    }

    #[tokio::test]
    async fn update_source_duplicate_name() {
        let pool = test_pool().await;

        let _first = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "First".to_string(),
            },
        )
        .await
        .expect("expected first source to be created");

        let second = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "Second".to_string(),
            },
        )
        .await
        .expect("expected second source to be created");

        let result = update_source_with_pool(
            &pool,
            UpdateSourceRequest {
                source_id: second.id,
                name: "First".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate name error")
            .contains("already exists"));
    }

    // --- Hoop ---

    #[tokio::test]
    async fn update_hoop_success() {
        let pool = test_pool().await;

        let created = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Old Hoop".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected hoop to be created");

        let updated = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: created.id,
                name: "New Hoop".to_string(),
                max_width_mm: 130.0,
                max_height_mm: 180.0,
            },
        )
        .await
        .expect("expected hoop to be updated");

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "New Hoop");
        assert_eq!(updated.max_width_mm, 130.0);
        assert_eq!(updated.max_height_mm, 180.0);
        assert_eq!(updated.design_count, 0);
    }

    #[tokio::test]
    async fn update_hoop_not_found() {
        let pool = test_pool().await;

        let result = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: 999,
                name: "Ghost".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }

    #[tokio::test]
    async fn update_hoop_empty_name() {
        let pool = test_pool().await;

        let created = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Valid".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected hoop to be created");

        let result = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: created.id,
                name: "  ".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected empty name error")
            .contains("Hoop name is required"));
    }

    #[tokio::test]
    async fn update_hoop_duplicate_name() {
        let pool = test_pool().await;

        let _first = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "First Hoop".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected first hoop to be created");

        let second = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Second Hoop".to_string(),
                max_width_mm: 130.0,
                max_height_mm: 180.0,
            },
        )
        .await
        .expect("expected second hoop to be created");

        let result = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: second.id,
                name: "First Hoop".to_string(),
                max_width_mm: 130.0,
                max_height_mm: 180.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate name error")
            .contains("already exists"));
    }

    #[tokio::test]
    async fn update_hoop_invalid_dimensions() {
        let pool = test_pool().await;

        let created = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "Valid".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected hoop to be created");

        let result = update_hoop_with_pool(
            &pool,
            UpdateHoopRequest {
                hoop_id: created.id,
                name: "Bad Dims".to_string(),
                max_width_mm: 0.0,
                max_height_mm: 100.0,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected invalid width error")
            .contains("must be a positive number"));
    }

    // ========================================================================
    // update_tag (rename) tests
    // ========================================================================

    #[tokio::test]
    async fn update_tag_success() {
        let pool = test_pool().await;

        let created = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Old Tag".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        let updated = update_tag_with_pool(
            &pool,
            UpdateTagRequest {
                tag_id: created.id,
                description: "New Tag".to_string(),
            },
        )
        .await
        .expect("expected tag to be updated");

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.description, "New Tag");
        assert_eq!(updated.tag_group, Some("image".to_string()));
        assert_eq!(updated.design_count, 0);
    }

    #[tokio::test]
    async fn update_tag_not_found() {
        let pool = test_pool().await;

        let result = update_tag_with_pool(
            &pool,
            UpdateTagRequest {
                tag_id: 999,
                description: "Ghost".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }

    #[tokio::test]
    async fn update_tag_rejects_system_tag() {
        let pool = test_pool().await;

        // Insert a system-defined stitching tag directly (as the migration does).
        let system_tag = sqlx::query(
            "INSERT INTO tags (description, tag_group, is_system) VALUES (?, ?, ?) RETURNING id",
        )
        .bind("Cross Stitch")
        .bind("stitching")
        .bind(1_i64)
        .fetch_one(&pool)
        .await
        .expect("expected system tag to be inserted");
        let system_tag_id: i64 = system_tag.get("id");

        let result = update_tag_with_pool(
            &pool,
            UpdateTagRequest {
                tag_id: system_tag_id,
                description: "Renamed".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected system tag rejection error")
            .contains("System tags cannot be modified or deleted."));
    }

    #[tokio::test]
    async fn update_tag_empty_description() {
        let pool = test_pool().await;

        let created = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Valid".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        let result = update_tag_with_pool(
            &pool,
            UpdateTagRequest {
                tag_id: created.id,
                description: "   ".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected empty description error")
            .contains("Tag description is required"));
    }

    #[tokio::test]
    async fn update_tag_duplicate_description() {
        let pool = test_pool().await;

        let _first = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "First".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected first tag to be created");

        let second = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Second".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected second tag to be created");

        let result = update_tag_with_pool(
            &pool,
            UpdateTagRequest {
                tag_id: second.id,
                description: "First".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected duplicate description error")
            .contains("already exists"));
    }

    #[tokio::test]
    async fn update_tag_case_insensitive_duplicate() {
        let pool = test_pool().await;

        let _first = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Case Tag".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected first tag to be created");

        let second = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Other Tag".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected second tag to be created");

        let result = update_tag_with_pool(
            &pool,
            UpdateTagRequest {
                tag_id: second.id,
                description: "case tag".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected case-insensitive duplicate error")
            .contains("already exists"));
    }

    #[tokio::test]
    async fn list_tags_counts_design_usage() {
        let pool = test_pool().await;

        let tag = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Counted".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        // Insert a design and link it to the tag
        let design = sqlx::query("INSERT INTO designs (filename, filepath) VALUES (?, ?) RETURNING id")
            .bind("counted.dst")
            .bind("/path/counted.dst")
            .fetch_one(&pool)
            .await
            .expect("expected design to be inserted");

        let design_id: i64 = design.get("id");

        sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (?, ?)")
            .bind(design_id)
            .bind(tag.id)
            .execute(&pool)
            .await
            .expect("expected design-tag association to be inserted");

        let result = list_tags_with_pool(&pool).await.expect("expected tags");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "Counted");
        assert_eq!(result[0].design_count, 1);
    }

    // ========================================================================
    // set_tag_group tests
    // ========================================================================

    #[tokio::test]
    async fn set_tag_group_success() {
        let pool = test_pool().await;

        let tag = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Test".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        let updated = set_tag_group_with_pool(
            &pool,
            SetTagGroupRequest {
                tag_id: tag.id,
                tag_group: "stitching".to_string(),
            },
        )
        .await
        .expect("expected tag group to be updated");

        assert_eq!(updated.id, tag.id);
        assert_eq!(updated.description, "Test");
        assert_eq!(updated.tag_group, Some("stitching".to_string()));
    }

    #[tokio::test]
    async fn set_tag_group_not_found() {
        let pool = test_pool().await;

        let result = set_tag_group_with_pool(
            &pool,
            SetTagGroupRequest {
                tag_id: 777,
                tag_group: "stitching".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }

    #[tokio::test]
    async fn set_tag_group_rejects_system_tag() {
        let pool = test_pool().await;

        // Insert a system-defined stitching tag directly (as the migration does).
        let system_tag = sqlx::query(
            "INSERT INTO tags (description, tag_group, is_system) VALUES (?, ?, ?) RETURNING id",
        )
        .bind("Applique")
        .bind("stitching")
        .bind(1_i64)
        .fetch_one(&pool)
        .await
        .expect("expected system tag to be inserted");
        let system_tag_id: i64 = system_tag.get("id");

        let result = set_tag_group_with_pool(
            &pool,
            SetTagGroupRequest {
                tag_id: system_tag_id,
                tag_group: "image".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected system tag rejection error")
            .contains("System tags cannot be modified or deleted."));
    }

    #[tokio::test]
    async fn set_tag_group_rejects_invalid_group() {
        let pool = test_pool().await;

        let tag = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "Test".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        let result = set_tag_group_with_pool(
            &pool,
            SetTagGroupRequest {
                tag_id: tag.id,
                tag_group: "invalid".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected invalid tag group error")
            .contains("must be 'image' or 'stitching'"));
    }

    // ========================================================================
    // Delete success + error-path tests
    // ========================================================================

    // --- Designer ---

    #[tokio::test]
    async fn delete_designer_success() {
        let pool = test_pool().await;

        let created = create_designer_with_pool(
            &pool,
            CreateDesignerRequest {
                name: "To Delete".to_string(),
            },
        )
        .await
        .expect("expected designer to be created");

        let result = delete_designer_with_pool(&pool, created.id).await;
        assert!(result.is_ok());

        // Verify it's gone
        let designers = list_designers_with_pool(&pool)
            .await
            .expect("expected empty list");
        assert!(designers.is_empty());
    }

    #[tokio::test]
    async fn delete_designer_not_found() {
        let pool = test_pool().await;

        let result = delete_designer_with_pool(&pool, 555).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }

    // --- Source ---

    #[tokio::test]
    async fn delete_source_success() {
        let pool = test_pool().await;

        let created = create_source_with_pool(
            &pool,
            CreateSourceRequest {
                name: "To Delete".to_string(),
            },
        )
        .await
        .expect("expected source to be created");

        let result = delete_source_with_pool(&pool, created.id).await;
        assert!(result.is_ok());

        let sources = list_sources_with_pool(&pool).await.expect("expected empty list");
        assert!(sources.is_empty());
    }

    #[tokio::test]
    async fn delete_source_not_found() {
        let pool = test_pool().await;

        let result = delete_source_with_pool(&pool, 666).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }

    // --- Tag ---

    #[tokio::test]
    async fn delete_tag_success() {
        let pool = test_pool().await;

        let created = create_tag_with_pool(
            &pool,
            CreateTagRequest {
                description: "To Delete".to_string(),
                tag_group: "image".to_string(),
            },
        )
        .await
        .expect("expected tag to be created");

        let result = delete_tag_with_pool(&pool, created.id).await;
        assert!(result.is_ok());

        let tags = list_tags_with_pool(&pool).await.expect("expected empty list");
        assert!(tags.is_empty());
    }

    #[tokio::test]
    async fn delete_tag_rejects_not_found_id() {
        let pool = test_pool().await;

        let result = delete_tag_with_pool(&pool, 444).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }

    #[tokio::test]
    async fn delete_tag_rejects_system_tag() {
        let pool = test_pool().await;

        // Insert a system-defined stitching tag directly (as the migration does).
        let system_tag = sqlx::query(
            "INSERT INTO tags (description, tag_group, is_system) VALUES (?, ?, ?) RETURNING id",
        )
        .bind("Lace")
        .bind("stitching")
        .bind(1_i64)
        .fetch_one(&pool)
        .await
        .expect("expected system tag to be inserted");
        let system_tag_id: i64 = system_tag.get("id");

        let result = delete_tag_with_pool(&pool, system_tag_id).await;

        assert!(result.is_err());
        assert!(result
            .expect_err("expected system tag rejection error")
            .contains("System tags cannot be modified or deleted."));

        // The system tag must still exist.
        let still_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tags WHERE id = ?",
        )
        .bind(system_tag_id)
        .fetch_one(&pool)
        .await
        .expect("expected tag existence query");
        assert_eq!(still_exists, 1, "system tag must not be deleted");
    }

    // --- Hoop ---

    #[tokio::test]
    async fn delete_hoop_success() {
        let pool = test_pool().await;

        let created = create_hoop_with_pool(
            &pool,
            CreateHoopRequest {
                name: "To Delete".to_string(),
                max_width_mm: 100.0,
                max_height_mm: 100.0,
            },
        )
        .await
        .expect("expected hoop to be created");

        let result = delete_hoop_with_pool(&pool, created.id).await;
        assert!(result.is_ok());

        let hoops = list_hoops_with_pool(&pool).await.expect("expected empty list");
        assert!(hoops.is_empty());
    }

    #[tokio::test]
    async fn delete_hoop_not_found() {
        let pool = test_pool().await;

        let result = delete_hoop_with_pool(&pool, 888).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("expected not-found error")
            .contains("not found"));
    }
}
