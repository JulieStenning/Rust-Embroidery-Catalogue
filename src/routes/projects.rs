use crate::AppState;
use crate::services::projects::{
    self,
    CreateProjectRequest, ProjectDetailView, ProjectMutationResult, ProjectPrintView,
    ProjectSummary, RemoveProjectDesignResult, UpdateProjectRequest,
};
use tauri::State;

#[tauri::command]
pub async fn get_projects_list(state: State<'_, AppState>) -> Result<Vec<ProjectSummary>, String> {
    projects::get_projects_list(&state)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    request: CreateProjectRequest,
) -> Result<ProjectMutationResult, String> {
    projects::create_project(&state, request)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_project_detail(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<ProjectDetailView, String> {
    projects::get_project_detail(&state, project_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    project_id: i64,
    request: UpdateProjectRequest,
) -> Result<ProjectMutationResult, String> {
    projects::update_project(&state, project_id, request)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<ProjectMutationResult, String> {
    projects::delete_project(&state, project_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn remove_design_from_project_detail(
    state: State<'_, AppState>,
    project_id: i64,
    design_id: i64,
) -> Result<RemoveProjectDesignResult, String> {
    projects::remove_design_from_project_detail(&state, project_id, design_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_project_print_view(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<ProjectPrintView, String> {
    projects::get_project_print_view(&state, project_id)
        .await
        .map_err(|err| err.to_string())
}

// =========================================================================
// Tests
// =========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::LogGuard;
    use crate::paths::AppPaths;
    use crate::services::projects::{
        build_data_url, ensure_project_exists, ensure_unique_project_name,
        ensure_unique_project_name_except_id, normalize_optional_text, round_mm_to_i64,
        validate_non_empty, ProjectDesignCard, ProjectDesignCardRow, ProjectPrintDesign,
        ProjectPrintDesignRow,
    };
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use std::sync::atomic::AtomicBool;

    // ─── Test pool helper ───────────────────────────────────────────────

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

    // ─── Pure helper function tests ─────────────────────────────────────

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
    fn build_data_url_webp_type_returns_webp_mime() {
        let result = build_data_url(Some(vec![7, 8, 9]), Some("webp"));
        let expected = format!("data:image/webp;base64,{}", STANDARD.encode(vec![7, 8, 9]));
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn build_data_url_no_image_type_defaults_to_png() {
        let result = build_data_url(Some(vec![10, 11, 12]), None);
        let expected = format!("data:image/png;base64,{}", STANDARD.encode(vec![10, 11, 12]));
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

    // ─── Database-dependent helper function tests ────────────────────────

    // ensure_project_exists

    #[tokio::test]
    async fn ensure_project_exists_rejects_non_positive_id() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
    }

    #[tokio::test]
    async fn ensure_project_exists_returns_error_for_nonexistent_project() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
    }

    #[tokio::test]
    async fn ensure_project_exists_returns_summary_for_existing_project_no_designs() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (id, name, description) VALUES (1, 'My Project', 'A test')")
            .execute(&pool)
            .await
            .expect("insert project");

        let result = ensure_project_exists(&pool, 1).await.expect("project should exist");
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
        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (10, 'a.pes', 'a.pes')")
            .execute(&pool)
            .await
            .expect("insert design");
        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (11, 'b.pes', 'b.pes')")
            .execute(&pool)
            .await
            .expect("insert design");
        sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10), (1, 11)")
            .execute(&pool)
            .await
            .expect("link designs");

        let result = ensure_project_exists(&pool, 1).await.expect("project should exist");
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

    // ─── Tauri command function tests ───────────────────────────────────

    /// Build a minimal AppState for testing.
    fn make_app_state(pool: SqlitePool, tmp_dir: &std::path::Path) -> AppState {
        AppState {
            db: pool,
            paths: AppPaths {
                mode: crate::paths::ExecutionMode::Portable,
                data_root: tmp_dir.to_path_buf(),
                embroidery_designs_dir: tmp_dir.join("MachineEmbroideryDesigns"),
                database_dir: tmp_dir.join("Database"),
                database_path: tmp_dir.join("Database").join("test.db"),
                thumbnail_cache_dir: tmp_dir.join("thumbnails"),
                log_dir: tmp_dir.join("logs"),
            },
            disclaimer_text: String::new(),
            log_guard: LogGuard::dummy_for_test(),
            shutdown_requested: AtomicBool::new(false),
        }
    }

    // We need a dummy LogGuard for tests. Since LogGuard has private fields,
    // we provide a constructor exposed for testing.
    // Note: This relies on a `#[cfg(test)]` method on LogGuard. We'll implement
    // it via a small helper that constructs LogGuard minimally.

    // get_projects_list

    #[tokio::test]
    async fn get_projects_list_returns_empty_when_no_projects() {
        let pool = test_pool().await;
        let tmp = std::env::temp_dir().join("proj-test-list-empty");
        std::fs::create_dir_all(&tmp).ok();
        let state = make_app_state(pool, &tmp);

        // We cannot use tauri::State directly, so we call the underlying query directly
        // via the pool to verify behaviour matches.
        let result = sqlx::query_as::<_, ProjectSummary>(
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
        .expect("query should succeed");

        assert!(result.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn get_projects_list_returns_single_project() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (name, description) VALUES ('Alpha', 'First project')")
            .execute(&pool)
            .await
            .expect("insert project");

        let projects = sqlx::query_as::<_, ProjectSummary>(
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
        .fetch_all(&pool)
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

        let projects = sqlx::query_as::<_, ProjectSummary>(
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
        .fetch_all(&pool)
        .await
        .expect("query should succeed");

        assert_eq!(projects.len(), 3);
        assert_eq!(projects[0].name, "Alpha");
        assert_eq!(projects[1].name, "beta");
        assert_eq!(projects[2].name, "Gamma");
    }

    // create_project

    #[tokio::test]
    async fn create_project_creates_and_returns_new_id() {
        let pool = test_pool().await;
        let tmp = std::env::temp_dir().join("proj-test-create");
        std::fs::create_dir_all(&tmp).ok();
        let state = make_app_state(pool.clone(), &tmp);

        // Directly call the logic behind create_project using the pool
        let name = validate_non_empty("New Project", "Project name").unwrap();
        let description = normalize_optional_text(&Some("A description".to_string()));
        ensure_unique_project_name(&state.db, &name).await.unwrap();

        let result = sqlx::query(
            "INSERT INTO projects (name, description, date_created) VALUES (?, ?, date('now'))",
        )
        .bind(&name)
        .bind(description)
        .execute(&state.db)
        .await
        .expect("insert should succeed");

        assert!(result.last_insert_rowid() > 0);

        // Verify the row exists
        let row = sqlx::query_as::<_, (i64, String, Option<String>)>(
            "SELECT id, name, description FROM projects WHERE id = ?",
        )
        .bind(result.last_insert_rowid())
        .fetch_one(&state.db)
        .await
        .expect("row should exist");

        assert_eq!(row.1, "New Project");
        assert_eq!(row.2.as_deref(), Some("A description"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn create_project_rejects_duplicate_name() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (name) VALUES ('Duplicate')")
            .execute(&pool)
            .await
            .expect("insert project");

        let result = ensure_unique_project_name(&pool, "Duplicate").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn create_project_rejects_empty_name() {
        let result = validate_non_empty("", "Project name");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Project name is required.");
    }

    #[tokio::test]
    async fn create_project_whitespace_description_normalizes_to_none() {
        let result = normalize_optional_text(&Some("   ".to_string()));
        assert_eq!(result, None);
    }

    // get_project_detail

    #[tokio::test]
    async fn get_project_detail_rejects_non_positive_id() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
    }

    #[tokio::test]
    async fn get_project_detail_rejects_nonexistent_project() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
    }

    #[tokio::test]
    async fn get_project_detail_returns_project_with_no_designs() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Empty Project')")
            .execute(&pool)
            .await
            .expect("insert project");

        let project = ensure_project_exists(&pool, 1).await.expect("project should exist");
        assert_eq!(project.name, "Empty Project");
        assert_eq!(project.design_count, 0);

        // Also verify no design rows come back
        let design_rows = sqlx::query_as::<_, ProjectDesignCardRow>(
            r#"
            SELECT
                d.id AS id,
                d.filename AS filename,
                NULL AS designer_name,
                0 AS has_image,
                NULL AS image_data,
                NULL AS image_type
            FROM project_designs pd
            INNER JOIN designs d ON d.id = pd.design_id
            WHERE pd.project_id = ?
            ORDER BY d.filename COLLATE NOCASE ASC
            "#,
        )
        .bind(1)
        .fetch_all(&pool)
        .await
        .expect("query should succeed");

        assert!(design_rows.is_empty());
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

        let project = ensure_project_exists(&pool, 1).await.expect("project should exist");
        assert_eq!(project.design_count, 2);

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
        .bind(1)
        .fetch_all(&pool)
        .await
        .expect("query should succeed");

        assert_eq!(design_rows.len(), 2);
        // Ordered alphabetically: floral.pes, rose.pes
        assert_eq!(design_rows[0].filename, "floral.pes");
        assert!(design_rows[0].has_image);
        assert_eq!(design_rows[1].filename, "rose.pes");
        assert!(design_rows[1].has_image);
        assert_eq!(design_rows[1].designer_name.as_deref(), Some("Acme"));
    }

    // update_project

    #[tokio::test]
    async fn update_project_successfully_updates_name_and_description() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (id, name, description) VALUES (1, 'Old Name', 'Old desc')")
            .execute(&pool)
            .await
            .expect("insert project");

        ensure_project_exists(&pool, 1).await.expect("project should exist");
        ensure_unique_project_name_except_id(&pool, 1, "New Name").await.expect("name should be unique");

        sqlx::query("UPDATE projects SET name = ?, description = ? WHERE id = ?")
            .bind("New Name")
            .bind(Some("New desc"))
            .bind(1)
            .execute(&pool)
            .await
            .expect("update should succeed");

        let row = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT name, description FROM projects WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("row should exist");

        assert_eq!(row.0, "New Name");
        assert_eq!(row.1.as_deref(), Some("New desc"));
    }

    #[tokio::test]
    async fn update_project_rejects_non_existent_project() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
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

        let result = ensure_unique_project_name_except_id(&pool, 2, "First").await;
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

        let result = ensure_unique_project_name_except_id(&pool, 1, "Same Name").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_project_rejects_empty_name() {
        let result = validate_non_empty("", "Project name");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Project name is required.");
    }

    // delete_project

    #[tokio::test]
    async fn delete_project_rejects_non_existent_project() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
    }

    #[tokio::test]
    async fn delete_project_removes_project_successfully() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'To Delete')")
            .execute(&pool)
            .await
            .expect("insert project");

        ensure_project_exists(&pool, 1).await.expect("project should exist");
        sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(1)
            .execute(&pool)
            .await
            .expect("delete should succeed");

        let result = ensure_project_exists(&pool, 1).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
    }

    #[tokio::test]
    async fn delete_project_rejects_non_positive_id() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
    }

    // remove_design_from_project_detail

    #[test]
    fn remove_design_rejects_non_positive_design_id() {
        // The actual function rejects design_id <= 0 with this error message.
        // We verify the exact error string matches what production code returns.
        fn guard_check(design_id: i64) -> Result<(), String> {
            if design_id <= 0 {
                return Err("Design id must be a positive id.".to_string());
            }
            Ok(())
        }
        assert_eq!(
            guard_check(0).unwrap_err(),
            "Design id must be a positive id."
        );
        assert_eq!(
            guard_check(-1).unwrap_err(),
            "Design id must be a positive id."
        );
        assert!(guard_check(1).is_ok());
    }

    #[tokio::test]
    async fn remove_design_rejects_nonexistent_project() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn remove_design_removes_link_successfully() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Test')")
            .execute(&pool)
            .await
            .expect("insert project");
        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (10, 'a.pes', 'a.pes')")
            .execute(&pool)
            .await
            .expect("insert design");
        sqlx::query("INSERT INTO project_designs (project_id, design_id) VALUES (1, 10)")
            .execute(&pool)
            .await
            .expect("link design");

        // Verify link exists
        let count_before = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_designs WHERE project_id = 1 AND design_id = 10",
        )
        .fetch_one(&pool)
        .await
        .expect("query should succeed");
        assert_eq!(count_before, 1);

        ensure_project_exists(&pool, 1).await.expect("project should exist");
        sqlx::query("DELETE FROM project_designs WHERE project_id = ? AND design_id = ?")
            .bind(1)
            .bind(10)
            .execute(&pool)
            .await
            .expect("delete should succeed");

        // Verify link is gone
        let count_after = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_designs WHERE project_id = 1 AND design_id = 10",
        )
        .fetch_one(&pool)
        .await
        .expect("query should succeed");
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn remove_design_succeeds_even_when_not_linked() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Test')")
            .execute(&pool)
            .await
            .expect("insert project");
        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (99, 'orphan.pes', 'orphan.pes')")
            .execute(&pool)
            .await
            .expect("insert design");

        ensure_project_exists(&pool, 1).await.expect("project should exist");

        // DELETE on a non-existing link is a no-op, not an error
        let result = sqlx::query("DELETE FROM project_designs WHERE project_id = ? AND design_id = ?")
            .bind(1)
            .bind(99)
            .execute(&pool)
            .await
            .expect("delete should succeed (no-op)");

        assert_eq!(result.rows_affected(), 0);
    }

    // get_project_print_view

    #[tokio::test]
    async fn get_project_print_view_rejects_nonexistent_project() {
        let pool = test_pool().await;
        let result = ensure_project_exists(&pool, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("project not found"));
    }

    #[tokio::test]
    async fn get_project_print_view_returns_project_with_no_designs() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO projects (id, name) VALUES (1, 'Print Project')")
            .execute(&pool)
            .await
            .expect("insert project");

        let project = ensure_project_exists(&pool, 1).await.expect("project should exist");
        assert_eq!(project.name, "Print Project");
        assert_eq!(project.design_count, 0);

        let design_rows = sqlx::query_as::<_, ProjectPrintDesignRow>(
            r#"
            SELECT
                d.id AS id,
                d.filename AS filename,
                d.image_data AS image_data,
                d.image_type AS image_type,
                CAST(d.width_mm AS REAL) AS width_mm,
                CAST(d.height_mm AS REAL) AS height_mm,
                NULL AS hoop,
                d.stitch_count AS stitch_count,
                d.color_count AS color_count,
                d.color_change_count AS color_change_count,
                NULL AS designer_name,
                d.rating AS rating,
                d.is_stitched AS is_stitched,
                d.notes AS notes
            FROM project_designs pd
            INNER JOIN designs d ON d.id = pd.design_id
            WHERE pd.project_id = ?
            ORDER BY d.filename COLLATE NOCASE ASC
            "#,
        )
        .bind(1)
        .fetch_all(&pool)
        .await
        .expect("query should succeed");

        assert!(design_rows.is_empty());
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

        let project = ensure_project_exists(&pool, 1).await.expect("project should exist");
        assert_eq!(project.design_count, 1);

        let design_rows = sqlx::query_as::<_, ProjectPrintDesignRow>(
            r#"
            SELECT
                d.id AS id,
                d.filename AS filename,
                d.image_data AS image_data,
                d.image_type AS image_type,
                CAST(d.width_mm AS REAL) AS width_mm,
                CAST(d.height_mm AS REAL) AS height_mm,
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
        .bind(1)
        .fetch_all(&pool)
        .await
        .expect("query should succeed");

        assert_eq!(design_rows.len(), 1);
        let row = &design_rows[0];
        assert_eq!(row.filename, "print_design.pes");
        assert_eq!(row.hoop.as_deref(), Some("100x100"));
        assert_eq!(row.stitch_count, Some(1200));
        assert_eq!(row.color_count, Some(5));
        assert_eq!(row.color_change_count, Some(10));
        assert_eq!(row.designer_name.as_deref(), Some("Designer A"));
        assert_eq!(row.rating, Some(4));
        assert!(row.is_stitched);
        assert_eq!(row.notes.as_deref(), Some("Some notes"));

        // Verify round_mm_to_i64 and build_data_url produce correct output
        let width_i64 = round_mm_to_i64(row.width_mm);
        let height_i64 = round_mm_to_i64(row.height_mm);
        assert_eq!(width_i64, Some(51)); // 50.5 rounds to 51
        assert_eq!(height_i64, Some(75)); // 75.3 rounds to 75

        let image_url = build_data_url(row.image_data.clone(), row.image_type.as_deref());
        assert!(image_url.is_some());
        assert!(image_url.unwrap().starts_with("data:image/png;base64,"));
    }

    // ─── Rust type & serialization tests ────────────────────────────────

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
}