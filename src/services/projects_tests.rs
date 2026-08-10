// Tests for the projects service.
//
// This module was split out of projects.rs so the service file can stay
// focused on production logic. It is included via a #[path] declaration
// in a #[cfg(test)] mod tests; module, so it retains full access to the
// private items in the parent module through use super::*;.

use super::*;
use crate::logging::LogGuard;
use crate::paths::AppPaths;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::atomic::AtomicBool;

// â”€â”€â”€ Test infrastructure â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Create an in-memory SQLite pool with the minimal schema needed for
/// project tests: `projects`, `project_designs`, `designs`, `designers`,
/// and `hoops`.
async fn test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory test pool");

    // Enable foreign keys so DELETE CASCADE etc. work in tests.
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("failed to enable foreign keys");

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
                date_added DATE,
                designer_id INTEGER REFERENCES designers(id) ON DELETE SET NULL,
                hoop_id INTEGER REFERENCES hoops(id) ON DELETE SET NULL
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("failed to create designs table");

    sqlx::query(
        r#"
            CREATE TABLE project_designs (
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                design_id INTEGER NOT NULL REFERENCES designs(id) ON DELETE CASCADE,
                PRIMARY KEY (project_id, design_id)
            );
            "#,
    )
    .execute(&pool)
    .await
    .expect("failed to create project_designs table");

    pool
}

/// Build a minimal AppState for testing.
fn make_app_state(pool: SqlitePool) -> AppState {
    let tmp_dir = std::env::temp_dir().join("proj-service-test");
    std::fs::create_dir_all(&tmp_dir).ok();
    AppState {
        db: pool,
        paths: AppPaths {
            mode: crate::paths::ExecutionMode::Installed,
            data_root: tmp_dir.clone(),
            embroidery_designs_dir: tmp_dir.join("MachineEmbroideryDesigns"),
            database_dir: tmp_dir.join("Database"),
            database_path: tmp_dir.join("Database").join("test.db"),
            thumbnail_cache_dir: tmp_dir.join("thumbnails"),
            log_dir: tmp_dir.join("logs"),
        },
        disclaimer_text: String::new(),
        log_guard: LogGuard::dummy_for_test(),
        shutdown_requested: AtomicBool::new(false),
        maintenance_running: AtomicBool::new(false),
    }
}

/// Helper to insert a minimal design row and return its id.
async fn insert_design(pool: &SqlitePool, id: i64, filename: &str) {
    sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
        .bind(id)
        .bind(filename)
        .bind(filename)
        .execute(pool)
        .await
        .expect("insert design");
}

// â”€â”€â”€ Pure helper function tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

// normalize_optional_text

#[test]
fn normalize_optional_text_none_returns_none() {
    assert_eq!(normalize_optional_text(&None), None);
}

#[test]
fn normalize_optional_text_empty_string_returns_none() {
    assert_eq!(normalize_optional_text(&Some(String::new())), None);
}

#[test]
fn normalize_optional_text_whitespace_only_returns_none() {
    assert_eq!(normalize_optional_text(&Some("   \t  ".to_string())), None);
}

#[test]
fn normalize_optional_text_valid_returns_trimmed() {
    assert_eq!(
        normalize_optional_text(&Some("  Hello World  ".to_string())),
        Some("Hello World".to_string())
    );
}

#[test]
fn normalize_optional_text_no_trim_needed_preserves() {
    assert_eq!(
        normalize_optional_text(&Some("Hello".to_string())),
        Some("Hello".to_string())
    );
}

// validate_non_empty

#[test]
fn validate_non_empty_accepts_trimmed_text() {
    assert_eq!(
        validate_non_empty("  Project Alpha  ", "Project name"),
        Ok("Project Alpha".to_string())
    );
}

#[test]
fn validate_non_empty_rejects_empty_string() {
    assert_eq!(
        validate_non_empty("", "Project name"),
        Err("Project name is required.".to_string())
    );
}

#[test]
fn validate_non_empty_rejects_whitespace_only() {
    assert_eq!(
        validate_non_empty("   \n  ", "Project name"),
        Err("Project name is required.".to_string())
    );
}

// build_data_url

#[test]
fn build_data_url_none_image_data_returns_none() {
    assert_eq!(build_data_url(None, Some("png")), None);
}

#[test]
fn build_data_url_empty_bytes_returns_none() {
    assert_eq!(build_data_url(Some(vec![]), Some("png")), None);
}

#[test]
fn build_data_url_png_type_returns_png_mime() {
    let result = build_data_url(Some(vec![1, 2, 3]), Some("png"));
    let expected = format!("data:image/png;base64,{}", STANDARD.encode(vec![1, 2, 3]));
    assert_eq!(result, Some(expected));
}

#[test]
fn build_data_url_jpeg_type_returns_jpeg_mime() {
    let result = build_data_url(Some(vec![4, 5, 6]), Some("jpeg"));
    let expected = format!("data:image/jpeg;base64,{}", STANDARD.encode(vec![4, 5, 6]));
    assert_eq!(result, Some(expected));
}

#[test]
fn build_data_url_jpg_type_returns_jpeg_mime() {
    let result = build_data_url(Some(vec![13, 14, 15]), Some("JPG"));
    let expected = format!(
        "data:image/jpeg;base64,{}",
        STANDARD.encode(vec![13, 14, 15])
    );
    assert_eq!(result, Some(expected));
}

#[test]
fn build_data_url_webp_type_returns_webp_mime() {
    let result = build_data_url(Some(vec![7, 8, 9]), Some("webp"));
    let expected = format!("data:image/webp;base64,{}", STANDARD.encode(vec![7, 8, 9]));
    assert_eq!(result, Some(expected));
}

#[test]
fn build_data_url_no_image_type_defaults_to_png() {
    let result = build_data_url(Some(vec![10, 11, 12]), None);
    let expected = format!(
        "data:image/png;base64,{}",
        STANDARD.encode(vec![10, 11, 12])
    );
    assert_eq!(result, Some(expected));
}

#[test]
fn build_data_url_unknown_image_type_defaults_to_png() {
    let result = build_data_url(Some(vec![16, 17, 18]), Some("bmp"));
    let expected = format!(
        "data:image/png;base64,{}",
        STANDARD.encode(vec![16, 17, 18])
    );
    assert_eq!(result, Some(expected));
}

// round_mm_to_i64

#[test]
fn round_mm_to_i64_none_returns_none() {
    assert_eq!(round_mm_to_i64(None), None);
}

#[test]
fn round_mm_to_i64_rounds_down() {
    assert_eq!(round_mm_to_i64(Some(12.3)), Some(12));
}

#[test]
fn round_mm_to_i64_rounds_up() {
    assert_eq!(round_mm_to_i64(Some(12.7)), Some(13));
}

#[test]
fn round_mm_to_i64_rounds_half_up() {
    assert_eq!(round_mm_to_i64(Some(12.5)), Some(13));
}

// â”€â”€â”€ Database-dependent helper function tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

// ensure_project_exists

#[tokio::test]
async fn ensure_project_exists_rejects_non_positive_id() {
    let pool = test_pool().await;
    let result = ensure_project_exists(&pool, 0).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn ensure_project_exists_returns_error_for_nonexistent_project() {
    let pool = test_pool().await;
    let result = ensure_project_exists(&pool, 999).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn ensure_project_exists_returns_summary_for_existing_project_no_designs() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name, description) VALUES (1, 'My Project', 'A test')")
        .execute(&pool)
        .await
        .expect("insert project");

    let result = ensure_project_exists(&pool, 1)
        .await
        .expect("project should exist");
    assert_eq!(result.id, 1);
    assert_eq!(result.name, "My Project");
    assert_eq!(result.description.as_deref(), Some("A test"));
    assert_eq!(result.design_count, 0);
}

#[tokio::test]
async fn ensure_project_exists_counts_designs() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Project With Designs')")
        .execute(&pool)
        .await
        .expect("insert project");
    insert_design(&pool, 10, "a.pes").await;
    insert_design(&pool, 11, "b.pes").await;
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10), (1, 11)")
        .execute(&pool)
        .await
        .expect("link designs");

    let result = ensure_project_exists(&pool, 1)
        .await
        .expect("project should exist");
    assert_eq!(result.design_count, 2);
}

// ensure_unique_project_name

#[tokio::test]
async fn ensure_unique_project_name_ok_when_name_does_not_exist() {
    let pool = test_pool().await;
    let result = ensure_unique_project_name(&pool, "New Project").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn ensure_unique_project_name_err_when_name_exists() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (name) VALUES ('Existing')")
        .execute(&pool)
        .await
        .expect("insert project");

    let result = ensure_unique_project_name(&pool, "Existing").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn ensure_unique_project_name_is_case_insensitive() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (name) VALUES ('My Project')")
        .execute(&pool)
        .await
        .expect("insert project");

    let result = ensure_unique_project_name(&pool, "my project").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

// ensure_unique_project_name_except_id

#[tokio::test]
async fn ensure_unique_project_name_except_id_allows_same_name_for_same_id() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'My Project')")
        .execute(&pool)
        .await
        .expect("insert project");

    let result = ensure_unique_project_name_except_id(&pool, 1, "My Project").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn ensure_unique_project_name_except_id_rejects_name_of_other_project() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'First')")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO projects (id, name) VALUES (2, 'Second')")
        .execute(&pool)
        .await
        .expect("insert project");

    let result = ensure_unique_project_name_except_id(&pool, 2, "First").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn ensure_unique_project_name_except_id_ok_when_name_not_taken_by_other() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Existing')")
        .execute(&pool)
        .await
        .expect("insert project");

    let result = ensure_unique_project_name_except_id(&pool, 1, "Updated Name").await;
    assert!(result.is_ok());
}

// â”€â”€â”€ Service function tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

// get_projects_list

#[tokio::test]
async fn get_projects_list_returns_empty_when_no_projects() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = get_projects_list(&state)
        .await
        .expect("query should succeed");
    assert!(result.is_empty());
}

#[tokio::test]
async fn get_projects_list_returns_single_project() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (name, description) VALUES ('Alpha', 'First project')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let projects = get_projects_list(&state)
        .await
        .expect("query should succeed");

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "Alpha");
    assert_eq!(projects[0].description.as_deref(), Some("First project"));
    assert_eq!(projects[0].design_count, 0);
}

#[tokio::test]
async fn get_projects_list_sorts_case_insensitively() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (name) VALUES ('beta')")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO projects (name) VALUES ('Alpha')")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO projects (name) VALUES ('Gamma')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let projects = get_projects_list(&state)
        .await
        .expect("query should succeed");

    assert_eq!(projects.len(), 3);
    assert_eq!(projects[0].name, "Alpha");
    assert_eq!(projects[1].name, "beta");
    assert_eq!(projects[2].name, "Gamma");
}

#[tokio::test]
async fn get_projects_list_includes_design_counts() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'With Designs')")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO projects (id, name) VALUES (2, 'Empty')")
        .execute(&pool)
        .await
        .expect("insert project");
    insert_design(&pool, 10, "a.pes").await;
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10)")
        .execute(&pool)
        .await
        .expect("link design");
    let state = make_app_state(pool);

    let projects = get_projects_list(&state)
        .await
        .expect("query should succeed");

    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "Empty");
    assert_eq!(projects[0].design_count, 0);
    assert_eq!(projects[1].name, "With Designs");
    assert_eq!(projects[1].design_count, 1);
}

// create_project

#[tokio::test]
async fn create_project_creates_and_returns_new_id() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = create_project(
        &state,
        CreateProjectRequest {
            name: "New Project".to_string(),
            description: Some("A description".to_string()),
        },
    )
    .await
    .expect("create should succeed");

    assert!(result.project_id > 0);
    assert_eq!(result.message, "Project created.");

    // Verify the row exists
    let row = sqlx::query_as::<_, (i64, String, Option<String>)>(
        "SELECT id, name, description FROM projects WHERE id = ?",
    )
    .bind(result.project_id)
    .fetch_one(&state.db)
    .await
    .expect("row should exist");

    assert_eq!(row.1, "New Project");
    assert_eq!(row.2.as_deref(), Some("A description"));
}

#[tokio::test]
async fn create_project_trims_name() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = create_project(
        &state,
        CreateProjectRequest {
            name: "  Trimmed Name  ".to_string(),
            description: None,
        },
    )
    .await
    .expect("create should succeed");

    let name = sqlx::query_scalar::<_, String>("SELECT name FROM projects WHERE id = ?")
        .bind(result.project_id)
        .fetch_one(&state.db)
        .await
        .expect("name should exist");

    assert_eq!(name, "Trimmed Name");
}

#[tokio::test]
async fn create_project_normalizes_whitespace_description_to_none() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = create_project(
        &state,
        CreateProjectRequest {
            name: "No Desc".to_string(),
            description: Some("   ".to_string()),
        },
    )
    .await
    .expect("create should succeed");

    let description =
        sqlx::query_scalar::<_, Option<String>>("SELECT description FROM projects WHERE id = ?")
            .bind(result.project_id)
            .fetch_one(&state.db)
            .await
            .expect("row should exist");

    assert_eq!(description, None);
}

#[tokio::test]
async fn create_project_rejects_duplicate_name() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (name) VALUES ('Duplicate')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let result = create_project(
        &state,
        CreateProjectRequest {
            name: "Duplicate".to_string(),
            description: None,
        },
    )
    .await;

    let err = result.expect_err("should fail");
    assert!(err.to_string().contains("already exists"));
}

#[tokio::test]
async fn create_project_rejects_empty_name() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = create_project(
        &state,
        CreateProjectRequest {
            name: "   ".to_string(),
            description: None,
        },
    )
    .await;

    let err = result.expect_err("should fail");
    assert_eq!(err.to_string(), "invalid input: Project name is required.");
}

// get_project_detail

#[tokio::test]
async fn get_project_detail_rejects_non_positive_id() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = get_project_detail(&state, 0).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn get_project_detail_rejects_nonexistent_project() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = get_project_detail(&state, 999).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn get_project_detail_returns_project_with_no_designs() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Empty Project')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let view = get_project_detail(&state, 1).await.expect("should succeed");

    assert_eq!(view.project.name, "Empty Project");
    assert_eq!(view.project.design_count, 0);
    assert!(view.designs.is_empty());
}

#[tokio::test]
async fn get_project_detail_includes_designs_with_images() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Project With Designs')")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO designers (id, name) VALUES (1, 'Acme')")
        .execute(&pool)
        .await
        .expect("insert designer");
    sqlx::query(
            "INSERT INTO designs (id, filename, filepath, image_data, image_type, designer_id) VALUES (10, 'rose.pes', 'rose.pes', X'010203', 'png', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert design");
    sqlx::query("INSERT INTO designs (id, filename, filepath, image_data, image_type) VALUES (11, 'floral.pes', 'floral.pes', X'040506', 'jpeg')")
            .execute(&pool)
            .await
            .expect("insert design");
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10), (1, 11)")
        .execute(&pool)
        .await
        .expect("link designs");
    let state = make_app_state(pool);

    let view = get_project_detail(&state, 1).await.expect("should succeed");

    assert_eq!(view.project.design_count, 2);
    assert_eq!(view.designs.len(), 2);
    // Ordered alphabetically: floral.pes, rose.pes
    assert_eq!(view.designs[0].filename, "floral.pes");
    assert!(view.designs[0].has_image);
    assert!(view.designs[0].image_data_url.is_some());
    assert!(
        view.designs[0]
            .image_data_url
            .as_deref()
            .unwrap()
            .starts_with("data:image/jpeg;base64,")
    );
    assert_eq!(view.designs[1].filename, "rose.pes");
    assert!(view.designs[1].has_image);
    assert_eq!(view.designs[1].designer_name.as_deref(), Some("Acme"));
    assert!(
        view.designs[1]
            .image_data_url
            .as_deref()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
}

#[tokio::test]
async fn get_project_detail_marks_design_without_image() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Project')")
        .execute(&pool)
        .await
        .expect("insert project");
    insert_design(&pool, 50, "no_image.pes").await;
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 50)")
        .execute(&pool)
        .await
        .expect("link design");
    let state = make_app_state(pool);

    let view = get_project_detail(&state, 1).await.expect("should succeed");

    assert_eq!(view.designs.len(), 1);
    assert!(!view.designs[0].has_image);
    assert_eq!(view.designs[0].image_data_url, None);
}

// update_project

#[tokio::test]
async fn update_project_successfully_updates_name_and_description() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name, description) VALUES (1, 'Old Name', 'Old desc')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let result = update_project(
        &state,
        1,
        UpdateProjectRequest {
            name: "New Name".to_string(),
            description: Some("New desc".to_string()),
        },
    )
    .await
    .expect("update should succeed");

    assert_eq!(result.project_id, 1);
    assert_eq!(result.message, "Project updated.");

    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, description FROM projects WHERE id = 1",
    )
    .fetch_one(&state.db)
    .await
    .expect("row should exist");

    assert_eq!(row.0, "New Name");
    assert_eq!(row.1.as_deref(), Some("New desc"));
}

#[tokio::test]
async fn update_project_normalizes_whitespace_description_to_none() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Project')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    update_project(
        &state,
        1,
        UpdateProjectRequest {
            name: "Project".to_string(),
            description: Some("   ".to_string()),
        },
    )
    .await
    .expect("update should succeed");

    let description =
        sqlx::query_scalar::<_, Option<String>>("SELECT description FROM projects WHERE id = 1")
            .fetch_one(&state.db)
            .await
            .expect("row should exist");

    assert_eq!(description, None);
}

#[tokio::test]
async fn update_project_rejects_non_existent_project() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = update_project(
        &state,
        999,
        UpdateProjectRequest {
            name: "Any".to_string(),
            description: None,
        },
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn update_project_rejects_name_conflict_with_another_project() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'First')")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO projects (id, name) VALUES (2, 'Second')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let result = update_project(
        &state,
        2,
        UpdateProjectRequest {
            name: "First".to_string(),
            description: None,
        },
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn update_project_allows_same_name_on_same_project() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Same Name')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let result = update_project(
        &state,
        1,
        UpdateProjectRequest {
            name: "Same Name".to_string(),
            description: None,
        },
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn update_project_rejects_empty_name() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Project')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let result = update_project(
        &state,
        1,
        UpdateProjectRequest {
            name: "  ".to_string(),
            description: None,
        },
    )
    .await;

    let err = result.expect_err("should fail");
    assert_eq!(err.to_string(), "invalid input: Project name is required.");
}

// delete_project

#[tokio::test]
async fn delete_project_rejects_non_existent_project() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = delete_project(&state, 999).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn delete_project_rejects_non_positive_id() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = delete_project(&state, 0).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn delete_project_removes_project_successfully() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'To Delete')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let result = delete_project(&state, 1)
        .await
        .expect("delete should succeed");

    assert_eq!(result.project_id, 1);
    assert_eq!(result.message, "Project deleted.");

    let remaining = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects WHERE id = 1")
        .fetch_one(&state.db)
        .await
        .expect("query should succeed");
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn delete_project_cascades_project_design_links() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Cascade Project')")
        .execute(&pool)
        .await
        .expect("insert project");
    insert_design(&pool, 10, "a.pes").await;
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10)")
        .execute(&pool)
        .await
        .expect("link design");
    let state = make_app_state(pool);

    delete_project(&state, 1)
        .await
        .expect("delete should succeed");

    let links =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM project_designs WHERE project_id = 1")
            .fetch_one(&state.db)
            .await
            .expect("query should succeed");
    assert_eq!(links, 0);
}

// remove_design_from_project_detail

#[tokio::test]
async fn remove_design_rejects_non_positive_design_id() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = remove_design_from_project_detail(&state, 1, 0).await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: Design id must be a positive id."
    );
}

#[tokio::test]
async fn remove_design_rejects_nonexistent_project() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = remove_design_from_project_detail(&state, 999, 10).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn remove_design_removes_link_successfully() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Test')")
        .execute(&pool)
        .await
        .expect("insert project");
    insert_design(&pool, 10, "a.pes").await;
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10)")
        .execute(&pool)
        .await
        .expect("link design");
    let state = make_app_state(pool);

    let result = remove_design_from_project_detail(&state, 1, 10)
        .await
        .expect("removal should succeed");

    assert_eq!(result.project_id, 1);
    assert_eq!(result.design_id, 10);
    assert_eq!(result.message, "Design removed from project.");

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_designs WHERE project_id = 1 AND design_id = 10",
    )
    .fetch_one(&state.db)
    .await
    .expect("query should succeed");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn remove_design_succeeds_even_when_not_linked() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Test')")
        .execute(&pool)
        .await
        .expect("insert project");
    insert_design(&pool, 99, "orphan.pes").await;
    let state = make_app_state(pool);

    // Removing a non-existing link is a no-op, not an error.
    let result = remove_design_from_project_detail(&state, 1, 99)
        .await
        .expect("no-op should still succeed");

    assert_eq!(result.project_id, 1);
    assert_eq!(result.design_id, 99);
}

// get_project_print_view

#[tokio::test]
async fn get_project_print_view_rejects_nonexistent_project() {
    let pool = test_pool().await;
    let state = make_app_state(pool);

    let result = get_project_print_view(&state, 999).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("project not found")
    );
}

#[tokio::test]
async fn get_project_print_view_returns_project_with_no_designs() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Print Project')")
        .execute(&pool)
        .await
        .expect("insert project");
    let state = make_app_state(pool);

    let view = get_project_print_view(&state, 1)
        .await
        .expect("should succeed");

    assert_eq!(view.project.name, "Print Project");
    assert!(view.designs.is_empty());
}

#[tokio::test]
async fn get_project_print_view_includes_designs_with_all_print_fields() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Print View Project')")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO designers (id, name) VALUES (1, 'Designer A')")
        .execute(&pool)
        .await
        .expect("insert designer");
    sqlx::query("INSERT INTO hoops (id, name, max_width_mm, max_height_mm) VALUES (1, '100x100', 100.0, 100.0)")
            .execute(&pool)
            .await
            .expect("insert hoop");
    sqlx::query(
            "INSERT INTO designs (id, filename, filepath, image_data, image_type, width_mm, height_mm, stitch_count, color_count, color_change_count, designer_id, hoop_id, rating, is_stitched, notes) VALUES (10, 'print_design.pes', 'print_design.pes', X'AABB', 'png', 50.5, 75.3, 1200, 5, 10, 1, 1, 4, 1, 'Some notes')",
        )
        .execute(&pool)
        .await
        .expect("insert design with all fields");
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10)")
        .execute(&pool)
        .await
        .expect("link design");
    let state = make_app_state(pool);

    let view = get_project_print_view(&state, 1)
        .await
        .expect("should succeed");

    assert_eq!(view.project.design_count, 1);
    assert_eq!(view.designs.len(), 1);
    let design = &view.designs[0];
    assert_eq!(design.id, 10);
    assert_eq!(design.filename, "print_design.pes");
    assert_eq!(design.hoop.as_deref(), Some("100x100"));
    assert_eq!(design.stitch_count, Some(1200));
    assert_eq!(design.color_count, Some(5));
    assert_eq!(design.color_change_count, Some(10));
    assert_eq!(design.designer_name.as_deref(), Some("Designer A"));
    assert_eq!(design.rating, Some(4));
    assert!(design.is_stitched);
    assert_eq!(design.notes.as_deref(), Some("Some notes"));
    // 50.5 rounds to 51, 75.3 rounds to 75
    assert_eq!(design.width_mm, Some(51));
    assert_eq!(design.height_mm, Some(75));
    // X'AABB' + 'png' -> data URL
    assert!(
        design
            .image_data_url
            .as_deref()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
}

#[tokio::test]
async fn get_project_print_view_normalizes_whitespace_notes_to_none() {
    let pool = test_pool().await;
    sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Project')")
        .execute(&pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO designs (id, filename, filepath, notes) VALUES (10, 'note.pes', 'note.pes', '   ')")
            .execute(&pool)
            .await
            .expect("insert design");
    sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10)")
        .execute(&pool)
        .await
        .expect("link design");
    let state = make_app_state(pool);

    let view = get_project_print_view(&state, 1)
        .await
        .expect("should succeed");

    assert_eq!(view.designs.len(), 1);
    assert_eq!(view.designs[0].notes, None);
}

// â”€â”€â”€ Rust type & serialization tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn project_mutation_result_serializes_correct_field_names() {
    let result = ProjectMutationResult {
        project_id: 42,
        message: "Created.".to_string(),
    };
    let json = serde_json::to_value(&result).expect("serialize");
    let map = json.as_object().expect("should be object");
    assert!(map.contains_key("project_id"));
    assert!(map.contains_key("message"));
    assert_eq!(map.len(), 2);
}

#[test]
fn project_design_card_serializes_correct_field_names() {
    let card = ProjectDesignCard {
        id: 1,
        filename: "test.pes".to_string(),
        designer_name: Some("Designer".to_string()),
        has_image: true,
        image_data_url: Some("data:image/png;base64,abc".to_string()),
    };
    let json = serde_json::to_value(&card).expect("serialize");
    let map = json.as_object().expect("should be object");
    assert!(map.contains_key("id"));
    assert!(map.contains_key("filename"));
    assert!(map.contains_key("designer_name"));
    assert!(map.contains_key("has_image"));
    assert!(map.contains_key("image_data_url"));
    assert_eq!(map.len(), 5);
}

#[test]
fn project_detail_view_serializes_correct_field_names() {
    let view = ProjectDetailView {
        project: ProjectSummary {
            id: 1,
            name: "P".to_string(),
            description: None,
            date_created: None,
            design_count: 0,
        },
        designs: vec![],
    };
    let json = serde_json::to_value(&view).expect("serialize");
    let map = json.as_object().expect("should be object");
    assert!(map.contains_key("project"));
    assert!(map.contains_key("designs"));
    assert_eq!(map.len(), 2);
}

#[test]
fn project_print_design_serializes_correct_field_names() {
    let design = ProjectPrintDesign {
        id: 1,
        filename: "test.pes".to_string(),
        image_data_url: None,
        width_mm: Some(100),
        height_mm: Some(200),
        hoop: Some("100x100".to_string()),
        stitch_count: Some(5000),
        color_count: Some(8),
        color_change_count: Some(16),
        designer_name: None,
        rating: Some(3),
        is_stitched: true,
        notes: Some("Nice".to_string()),
    };
    let json = serde_json::to_value(&design).expect("serialize");
    let map = json.as_object().expect("should be object");
    assert!(map.contains_key("id"));
    assert!(map.contains_key("filename"));
    assert!(map.contains_key("image_data_url"));
    assert!(map.contains_key("width_mm"));
    assert!(map.contains_key("height_mm"));
    assert!(map.contains_key("hoop"));
    assert!(map.contains_key("stitch_count"));
    assert!(map.contains_key("color_count"));
    assert!(map.contains_key("color_change_count"));
    assert!(map.contains_key("designer_name"));
    assert!(map.contains_key("rating"));
    assert!(map.contains_key("is_stitched"));
    assert!(map.contains_key("notes"));
    assert_eq!(map.len(), 13);
}

#[test]
fn remove_project_design_result_serializes_correct_field_names() {
    let result = RemoveProjectDesignResult {
        project_id: 1,
        design_id: 10,
        message: "Removed.".to_string(),
    };
    let json = serde_json::to_value(&result).expect("serialize");
    let map = json.as_object().expect("should be object");
    assert!(map.contains_key("project_id"));
    assert!(map.contains_key("design_id"));
    assert!(map.contains_key("message"));
    assert_eq!(map.len(), 3);
}
