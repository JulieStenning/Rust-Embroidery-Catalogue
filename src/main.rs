#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Embroidery Catalogue — Tauri v2 entry point

pub mod config;
pub mod database;
pub mod disclaimer;
pub mod error;
pub mod logging;
pub mod models;
pub mod paths;
pub mod png_writer;
pub mod readers;
pub mod routes;
pub mod services;
pub mod settings;
pub mod templating;
pub mod utils;

use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::atomic::AtomicBool;
use tauri::{Manager, State};

// ─── Shared Application State ─────────────────────────────────────────────────

/// Shared application state managed by Tauri.
/// `SqlitePool` is `Send + Sync`, so no `Mutex` wrapper is needed.
pub struct AppState {
    /// Connection pool for the SQLite database.
    pub db: SqlitePool,
    /// Resolved application paths (Portable vs Installed mode).
    pub paths: paths::AppPaths,
    /// The disclaimer HTML text, embedded at compile time from DISCLAIMER.html.
    pub disclaimer_text: String,
    /// Log guard — kept alive so log writes are flushed on app exit.
    pub log_guard: logging::LogGuard,
    /// Flag signalled when the app is shutting down; background tasks can check it.
    pub shutdown_requested: AtomicBool,
}

// ─── AppStatus (exposed to frontend) ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub execution_mode: String,
    pub data_root: String,
    pub embroidery_dir: String,
    pub database_path: String,
}

/// Pure function to construct an `AppStatus` from `AppPaths`.
/// Extracted for testability — this does not depend on Tauri state.
fn app_status_from_paths(paths: &paths::AppPaths) -> AppStatus {
    let mode_str = match paths.mode {
        paths::ExecutionMode::Portable => "portable".to_string(),
        paths::ExecutionMode::Installed => "installed".to_string(),
    };

    AppStatus {
        execution_mode: mode_str,
        data_root: paths.data_root.to_string_lossy().to_string(),
        embroidery_dir: paths.embroidery_designs_dir.to_string_lossy().to_string(),
        database_path: paths.database_path.to_string_lossy().to_string(),
    }
}

/// Return the current execution mode and path metadata to the frontend.
#[tauri::command]
fn get_app_status(state: State<'_, AppState>) -> AppStatus {
    app_status_from_paths(&state.paths)
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

/// Check whether the disclaimer has already been accepted for this installation.
#[tauri::command]
async fn check_disclaimer(state: State<'_, AppState>) -> Result<bool, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
    disclaimer::is_disclaimer_accepted(&mut conn)
        .await
        .map_err(|err| err.to_string())
}

/// Persist the user's disclaimer acceptance in the database.
#[tauri::command]
async fn accept_disclaimer(state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
    disclaimer::set_disclaimer_accepted(&mut conn, true)
        .await
        .map(|_| ())
        .map_err(|err| format!("Failed to save disclaimer acceptance to the database: {err}"))
}

/// Return the disclaimer HTML text to the frontend.
#[tauri::command]
fn get_disclaimer_text(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.disclaimer_text.clone())
}

// ─── Application entry point ──────────────────────────────────────────────────

fn main() {
    // ── Resolve paths (must be first — before logging and DB) ────────
    let app_paths = match paths::resolve_app_paths() {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("Failed to resolve application paths: {err}");
            std::process::exit(1);
        }
    };
    tracing::info!(
        "Embroidery Catalogue starting — mode={:?}, data_root={}",
        app_paths.mode,
        app_paths.data_root.display()
    );

    // ── Logging ───────────────────────────────────────────────────────
    let log_guard = match logging::init_logging(&app_paths.log_dir) {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to initialize logging: {err}");
            std::process::exit(1);
        }
    };
    tracing::info!("Logging initialised — log_dir={}", app_paths.log_dir.display());

    // Load .env file if present (best-effort; not required in production)
    load_dotenv();

    // Build bootstrap config from resolved paths
    let bootstrap_config = config::BootstrapConfig::from_app_paths(&app_paths);
    tracing::info!("Parsed bootstrap configuration: {:#?}", bootstrap_config);

    // Ensure the database directory exists before trying to connect
    if let Err(err) = config::ensure_database_dir(&bootstrap_config.database_url) {
        eprintln!("Failed to create database directory: {err}");
        std::process::exit(1);
    }

    // Run async setup using Tauri's built-in Tokio runtime
    // This avoids creating a conflicting second runtime alongside Tauri's own
    let (pool, disclaimer_text) = tauri::async_runtime::block_on(async {
        // Establish the SQLite connection pool using resolved paths
        let pool = match database::connection::establish_connection(&app_paths).await {
            Ok(pool) => pool,
            Err(err) => {
                eprintln!("Failed to establish database connection: {err}");
                std::process::exit(1);
            }
        };

        // NOTE: Migration runner is intentionally disabled.
        // Both the seed DB (src-tauri/resources/) and the development DB are
        // pre-migrated. Running sqlx::migrate!() would re-insert all seed data
        // (118 tags, settings, etc.) from the initial migration, overwriting
        // the curated seed DB content.
        //
        // If schema changes are needed in the future, run migrations manually:
        //   - Update the dev DB with new schema
        //   - Compact it and copy to src-tauri/resources/EmbroideryCatalogue.db
        //   - Add the .sql migration file to migrations/ for documentation
        //
        // database::migrations::run_migrations(&pool).await
        //     .expect("Failed to run database migrations");

        // Embed the disclaimer text at compile time from DISCLAIMER.html
        let disclaimer_text = include_str!("../disclaimer.html").to_string();

        (pool, disclaimer_text)
    });

    let app_state = AppState {
        db: pool,
        paths: app_paths,
        disclaimer_text,
        log_guard,
        shutdown_requested: AtomicBool::new(false),
    };

    // Launch a lightweight background backfill for orphan fingerprint data
    // (hash + file size).  This is fire-and-forget — errors are logged, not fatal.
    let fp_pool = app_state.db.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) =
            services::fingerprint::run_fingerprint_backfill(&fp_pool, 100).await
        {
            tracing::error!("Startup fingerprint backfill error: {}", err);
        }
    });

    routes::bulk_import::initialize_bulk_import_db_pool(app_state.db.clone());
    let startup_reset = routes::bulk_import::reset_bulk_import_context_store_for_startup();
    tracing::info!(
        "Bulk import context startup reset: cleared={}, active={}, resets={}, at_ms={}",
        startup_reset.cleared_context_count,
        startup_reset.active_context_count,
        startup_reset.reset_count,
        startup_reset.reset_at_millis
    );

    let app = tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            routes::bulk_import::initialize_bulk_import_app_handle(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            config::debug_bootstrap_config,
            check_disclaimer,
            accept_disclaimer,
            get_disclaimer_text,
            routes::about::get_about_documents,
            routes::about::get_about_document,
            routes::designs::get_designs,
            routes::designs::get_design_detail,
            routes::designs::bulk_verify_designs,
            routes::designs::get_projects_for_browse,
            routes::designs::bulk_add_designs_to_project,
            routes::designs::get_tags_for_browse,
            routes::designs::bulk_set_tags_for_designs,
            routes::designs::get_design_previews_for_browse,
            routes::designs::get_design_image_data_url,
            routes::designs::update_design_metadata,
            routes::designs::set_design_rating,
            routes::designs::set_design_stitched,
            routes::designs::set_design_tags_checked,
            routes::designs::set_design_tags,
            routes::designs::remove_design_tag,
            routes::designs::add_design_to_project,
            routes::designs::remove_design_from_project,
            routes::designs::delete_design,
            routes::designs::bulk_delete_designs,
            routes::designs::open_design_in_editor,
            routes::designs::open_design_in_explorer,
            routes::designs::render_design_3d_preview,
            routes::projects::get_projects_list,
            routes::projects::create_project,
            routes::projects::get_project_detail,
            routes::projects::update_project,
            routes::projects::delete_project,
            routes::projects::remove_design_from_project_detail,
            routes::projects::get_project_print_view,
            routes::settings::get_settings_view_model,
            routes::settings::save_settings_view_model,
            routes::settings::save_import_last_browse_folder,
            routes::settings::browse_settings_data_root,
            routes::admin::list_designers,
            routes::admin::create_designer,
            routes::admin::update_designer,
            routes::admin::delete_designer,
            routes::admin::list_sources,
            routes::admin::create_source,
            routes::admin::update_source,
            routes::admin::delete_source,
            routes::admin::list_tags,
            routes::admin::create_tag,
            routes::admin::set_tag_group,
            routes::admin::delete_tag,
            routes::admin::list_hoops,
            routes::admin::create_hoop,
            routes::admin::update_hoop,
            routes::admin::delete_hoop,
            routes::bulk_import::debug_bulk_import_wire,
            routes::bulk_import::debug_bulk_import_confirm_wire,
            routes::bulk_import::debug_bulk_import_assignment_resolution_wire,
            routes::bulk_import::debug_bulk_import_context_store,
            routes::bulk_import::reset_bulk_import_context_store,
            routes::bulk_import::request_stop_bulk_import,
            routes::bulk_import::precheck_bulk_import_wire,
            routes::bulk_import::precheck_bulk_import_action_wire,
            routes::bulk_import::do_confirm_bulk_import_wire,
            routes::bulk_import::execute_bulk_import_confirm_wire,
            routes::bulk_import::confirm_bulk_import_wire,
            routes::bulk_import::confirm_bulk_import_legacy,
            routes::bulk_import::preview_bulk_import,
            routes::bulk_import::browse_import_folder,
            routes::tagging_actions::get_tagging_actions_view_model,
            routes::tagging_actions::preview_tagging_action,
            routes::tagging_actions::run_unified_backfill,
            routes::tagging_actions::stop_unified_backfill,
            routes::tagging_actions::get_backfill_log_entries,
            routes::tagging_actions::run_stitching_backfill,
            routes::tagging_actions::run_fingerprint_backfill,
            routes::maintenance::maintenance_scaffold_enabled,
            routes::maintenance::get_backup_view_model,
            routes::maintenance::save_backup_settings,
            routes::maintenance::browse_backup_folder,
            routes::maintenance::run_database_backup,
            routes::maintenance::run_designs_backup,
            routes::maintenance::run_both_backups,
            routes::maintenance::scan_orphans,
            routes::maintenance::get_orphans_page,
            routes::maintenance::delete_orphans,
            routes::maintenance::delete_all_orphans,
            routes::maintenance::browse_orphan_path,
        ])
        // tauri::generate_context!() reads tauri.conf.json from the project root
        .build(tauri::generate_context!())
        .expect("Error while building the Embroidery Catalogue application");

    app.run(|app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { code, .. } => {
                tracing::info!(
                    "Exit requested (code: {:?}) — signalling shutdown...",
                    code
                );
                let state = app_handle.state::<AppState>();
                state
                    .shutdown_requested
                    .store(true, std::sync::atomic::Ordering::SeqCst);

                // Close the SQLite connection pool so VACUUM / WAL checkpoints
                // finish cleanly before the process exits.
                let pool = &state.db;
                let pool_clone = pool.clone();
                tauri::async_runtime::spawn(async move {
                    pool_clone.close().await;
                    tracing::info!("SQLite connection pool closed.");
                });
            }
            tauri::RunEvent::Exit => {
                tracing::info!("Embroidery Catalogue exiting.");
                // AppState (including LogGuard) is dropped here,
                // which flushes pending log writes to disk.
            }
            _ => {}
        }
    });
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Load environment variables from a `.env` file if one exists.
fn load_dotenv() {
    let env_path = std::path::Path::new(".env");
    if env_path.exists() {
        if let Ok(content) = std::fs::read_to_string(env_path) {
            load_dotenv_from_str(&content);
        }
    }
}

/// Parse the content of a dotenv-style string, setting environment variables
/// for any `KEY=VALUE` pairs that are not already present in the environment.
///
/// Lines that are empty, whitespace-only, or start with `'#'` are ignored.
/// If a line does not contain `'='` it is silently skipped.
/// Leading/trailing whitespace is trimmed from both the key and the value.
///
/// This is a pure function (no filesystem I/O) extracted from `load_dotenv`
/// so it can be tested without temp files or fixtures.
fn load_dotenv_from_str(content: &str) {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            // Skip lines with an empty key (e.g. "=value")
            if key.is_empty() {
                continue;
            }
            // Only set if not already present in the environment
            if std::env::var(key).is_err() {
                std::env::set_var(key, value);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
use super::*;
use crate::utils::test_support::lock_env;
use std::fs;
use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////
// load_dotenv_from_str — pure parsing logic                                  //
////////////////////////////////////////////////////////////////////////////////

    // ─── load_dotenv_from_str — pure parsing logic ───────────────────────────

    #[test]
    fn parse_empty_string_sets_no_vars() {
        // Act
        load_dotenv_from_str("");

        // Assert — we check that a well-known absent var is still absent
        // (no crash / no side effects).
        assert!(
            std::env::var("EMBROIDERY_TEST_A").is_err(),
            "No vars should have been set from an empty string"
        );
    }

    #[test]
    fn parse_comment_only_lines_sets_no_vars() {
        let content = "# This is a comment\n# Another comment";
        load_dotenv_from_str(content);
        assert!(
            std::env::var("EMBROIDERY_TEST_B").is_err(),
            "Comment lines should not set variables"
        );
    }

    #[test]
    fn parse_whitespace_only_lines_sets_no_vars() {
        let content = "   \n\t\n  ";
        load_dotenv_from_str(content);
        assert!(
            std::env::var("EMBROIDERY_TEST_C").is_err(),
            "Whitespace-only lines should be ignored"
        );
    }

    #[test]
    fn parse_simple_key_value_sets_env_var() {
        // Ensure the var is cleared before the test
        let _prev = std::env::var("EMBROIDERY_TEST_D").ok();
        std::env::remove_var("EMBROIDERY_TEST_D");

        load_dotenv_from_str("EMBROIDERY_TEST_D=hello");
        assert_eq!(
            std::env::var("EMBROIDERY_TEST_D").unwrap_or_default(),
            "hello"
        );
    }

    #[test]
    fn parse_key_with_whitespace_is_trimmed() {
        let _prev = std::env::var("EMBROIDERY_TEST_E").ok();
        std::env::remove_var("EMBROIDERY_TEST_E");

        load_dotenv_from_str("  EMBROIDERY_TEST_E  =  world  ");
        assert_eq!(
            std::env::var("EMBROIDERY_TEST_E").unwrap_or_default(),
            "world"
        );
    }

    #[test]
    fn parse_does_not_overwrite_existing_env_var() {
        // Set an existing value
        let _prev = std::env::var("EMBROIDERY_EXISTING").ok();
        std::env::set_var("EMBROIDERY_EXISTING", "original");

        // Attempt to overwrite via dotenv
        load_dotenv_from_str("EMBROIDERY_EXISTING=overwrite_attempt");

        // The original value must persist
        assert_eq!(
            std::env::var("EMBROIDERY_EXISTING").unwrap_or_default(),
            "original",
            "Should NOT overwrite an already-set environment variable"
        );
    }

    #[test]
    fn parse_multiple_assignments_sets_all() {
        let _prev1 = std::env::var("EMBROIDERY_MULTI_A").ok();
        let _prev2 = std::env::var("EMBROIDERY_MULTI_B").ok();
        std::env::remove_var("EMBROIDERY_MULTI_A");
        std::env::remove_var("EMBROIDERY_MULTI_B");

        load_dotenv_from_str("EMBROIDERY_MULTI_A=foo\nEMBROIDERY_MULTI_B=bar");

        assert_eq!(
            std::env::var("EMBROIDERY_MULTI_A").unwrap_or_default(),
            "foo"
        );
        assert_eq!(
            std::env::var("EMBROIDERY_MULTI_B").unwrap_or_default(),
            "bar"
        );
    }

    #[test]
    fn parse_line_without_equals_sign_is_skipped() {
        let _prev = std::env::var("EMBROIDERY_SKIP_A").ok();
        std::env::remove_var("EMBROIDERY_SKIP_A");

        // Lines without '=' should be silently ignored
        load_dotenv_from_str("EMBROIDERY_SKIP_A");
        assert!(
            std::env::var("EMBROIDERY_SKIP_A").is_err(),
            "Line without '=' should not set a variable"
        );
    }

    #[test]
    fn parse_multiple_equals_signs_uses_only_first() {
        let _prev = std::env::var("EMBROIDERY_MULTI_EQ").ok();
        std::env::remove_var("EMBROIDERY_MULTI_EQ");

        load_dotenv_from_str("EMBROIDERY_MULTI_EQ=val1=val2=val3");

        // Only the first '=' acts as the delimiter; the rest are part of the value
        assert_eq!(
            std::env::var("EMBROIDERY_MULTI_EQ").unwrap_or_default(),
            "val1=val2=val3"
        );
    }

    #[test]
    fn parse_mixed_content_with_comments_and_blanks() {
        let _prev_a = std::env::var("EMBROIDERY_MIXED_A").ok();
        let _prev_b = std::env::var("EMBROIDERY_MIXED_B").ok();
        std::env::remove_var("EMBROIDERY_MIXED_A");
        std::env::remove_var("EMBROIDERY_MIXED_B");

        let content = "# Database config\nEMBROIDERY_MIXED_A=db_host\n\nEMBROIDERY_MIXED_B=db_port\n  ";
        load_dotenv_from_str(content);

        assert_eq!(
            std::env::var("EMBROIDERY_MIXED_A").unwrap_or_default(),
            "db_host"
        );
        assert_eq!(
            std::env::var("EMBROIDERY_MIXED_B").unwrap_or_default(),
            "db_port"
        );
    }

    // ─── load_dotenv_from_str — edge cases ───────────────────────────────────

    #[test]
    fn parse_key_with_empty_value_sets_empty_string() {
        let _prev = std::env::var("EMBROIDERY_EMPTY_VAL").ok();
        std::env::remove_var("EMBROIDERY_EMPTY_VAL");

        load_dotenv_from_str("EMBROIDERY_EMPTY_VAL=");

        assert_eq!(
            std::env::var("EMBROIDERY_EMPTY_VAL").unwrap_or_default(),
            "",
            "A key with '=' and no value should set the variable to empty string"
        );
    }

    #[test]
    fn parse_line_with_empty_key_is_skipped() {
        let _prev = std::env::var("EMBROIDERY_EMPTY_KEY").ok();
        std::env::remove_var("EMBROIDERY_EMPTY_KEY");

        // An empty key after trimming should not set any variable.
        load_dotenv_from_str("=some_value\n  =another");

        assert!(
            std::env::var("EMBROIDERY_EMPTY_KEY").is_err(),
            "Lines with empty keys should be silently skipped (key is empty after trim)"
        );
    }

    // ─── load_dotenv — filesystem integration ────────────────────────────────

    #[test]
    fn load_dotenv_handles_missing_file_gracefully() {
        let _guard = lock_env();

        // Calling load_dotenv() when no .env file exists must not panic.
        // Use a temp dir with a non-existent .env to be safe.
        let tmp = std::env::temp_dir().join(format!(
            "embroidery-main-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&tmp);
        std::env::set_current_dir(&tmp).ok();

        // This should not panic even though there's no .env file.
        load_dotenv();

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn load_dotenv_reads_and_loads_from_file() {
        let _guard = lock_env();

        let tmp = std::env::temp_dir().join(format!(
            "embroidery-main-test-file-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&tmp);

        // Write a temporary .env file
        let env_path = tmp.join(".env");
        let _prev = std::env::var("EMBROIDERY_FILE_TEST").ok();
        std::env::remove_var("EMBROIDERY_FILE_TEST");
        fs::write(&env_path, "EMBROIDERY_FILE_TEST=loaded_from_file\n").expect("write .env");

        // Temporarily change cwd to our temp dir so load_dotenv finds the .env
        let original_cwd = std::env::current_dir().ok();
        std::env::set_current_dir(&tmp).ok();

        load_dotenv();

        assert_eq!(
            std::env::var("EMBROIDERY_FILE_TEST").unwrap_or_default(),
            "loaded_from_file"
        );

        // Restore cwd and clean up
        if let Some(cwd) = original_cwd {
            let _ = std::env::set_current_dir(cwd);
        }
        let _ = fs::remove_dir_all(&tmp);
    }

    // ─── AppStatus struct ──────────────────────────────────────────────────

    #[test]
    fn app_status_from_paths_serializes_portable_mode() {
        let paths = paths::AppPaths {
            mode: paths::ExecutionMode::Portable,
            data_root: PathBuf::from("E:/portable/data"),
            embroidery_designs_dir: PathBuf::from("E:/portable/data/MachineEmbroideryDesigns"),
            database_dir: PathBuf::from("E:/portable/data/Database"),
            database_path: PathBuf::from("E:/portable/data/Database/EmbroideryCatalogue.db"),
            thumbnail_cache_dir: PathBuf::from("E:/portable/data/thumbnails"),
            log_dir: PathBuf::from("E:/portable/data/logs"),
        };

        let status = app_status_from_paths(&paths);

        assert_eq!(status.execution_mode, "portable");
        assert_eq!(status.data_root, "E:/portable/data");
        assert_eq!(status.embroidery_dir, "E:/portable/data/MachineEmbroideryDesigns");
        assert_eq!(status.database_path, "E:/portable/data/Database/EmbroideryCatalogue.db");
    }

    #[test]
    fn app_status_from_paths_serializes_installed_mode() {
        let paths = paths::AppPaths {
            mode: paths::ExecutionMode::Installed,
            data_root: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue"),
            embroidery_designs_dir: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue/MachineEmbroideryDesigns"),
            database_dir: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue/Database"),
            database_path: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue/Database/EmbroideryCatalogue.db"),
            thumbnail_cache_dir: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue/thumbnails"),
            log_dir: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue/logs"),
        };

        let status = app_status_from_paths(&paths);

        assert_eq!(status.execution_mode, "installed");
        assert_eq!(status.data_root, "C:/Users/test/AppData/Roaming/EmbroideryCatalogue");
    }

    #[test]
    fn app_status_from_paths_handles_windows_backslash_paths() {
        // On Windows, to_string_lossy() on a PathBuf constructed from backslashes
        // yields backslashes. The frontend receives these raw values.
        let paths = paths::AppPaths {
            mode: paths::ExecutionMode::Portable,
            data_root: PathBuf::from("D:\\MyData"),
            embroidery_designs_dir: PathBuf::from("D:\\MyData\\MachineEmbroideryDesigns"),
            database_dir: PathBuf::from("D:\\MyData\\Database"),
            database_path: PathBuf::from("D:\\MyData\\Database\\EmbroideryCatalogue.db"),
            thumbnail_cache_dir: PathBuf::from("D:\\MyData\\thumbnails"),
            log_dir: PathBuf::from("D:\\MyData\\logs"),
        };

        let status = app_status_from_paths(&paths);

        // The path strings should contain the backslash separator as originally set.
        assert!(status.data_root.contains('\\'), "Windows paths should retain backslashes");
        assert!(status.embroidery_dir.contains('\\'), "Windows paths should retain backslashes");
        assert!(status.database_path.contains('\\'), "Windows paths should retain backslashes");

        // The execution mode should still be portable.
        assert_eq!(status.execution_mode, "portable");
    }

    // ─── AppStatus struct (serialization) ─────────────────────────────────

    #[test]
    fn app_status_serializes_correct_field_names() {
        let status = AppStatus {
            execution_mode: "installed".to_string(),
            data_root: "/some/data/root".to_string(),
            embroidery_dir: "/some/data/root/MachineEmbroideryDesigns".to_string(),
            database_path: "/some/data/root/Database/EmbroideryCatalogue.db".to_string(),
        };

        let json = serde_json::to_value(&status).expect("serialize AppStatus");
        let map = json.as_object().expect("json should be an object");

        // The frontend expects these exact field names
        assert!(map.contains_key("execution_mode"), "missing 'execution_mode'");
        assert!(map.contains_key("data_root"), "missing 'data_root'");
        assert!(map.contains_key("embroidery_dir"), "missing 'embroidery_dir'");
        assert!(map.contains_key("database_path"), "missing 'database_path'");

        // Exactly 4 fields — no extra, no missing
        assert_eq!(map.len(), 4, "AppStatus should serialize exactly 4 fields");
    }

    #[test]
    fn app_status_serializes_correct_field_values() {
        let status = AppStatus {
            execution_mode: "portable".to_string(),
            data_root: "D:/data".to_string(),
            embroidery_dir: "D:/data/MachineEmbroideryDesigns".to_string(),
            database_path: "D:/data/Database/EmbroideryCatalogue.db".to_string(),
        };

        let json = serde_json::to_value(&status).expect("serialize AppStatus");
        let map = json.as_object().expect("json should be an object");

        assert_eq!(map.get("execution_mode").and_then(|v| v.as_str()), Some("portable"));
        assert_eq!(map.get("data_root").and_then(|v| v.as_str()), Some("D:/data"));
        assert_eq!(map.get("embroidery_dir").and_then(|v| v.as_str()), Some("D:/data/MachineEmbroideryDesigns"));
        assert_eq!(map.get("database_path").and_then(|v| v.as_str()), Some("D:/data/Database/EmbroideryCatalogue.db"));
    }
}
