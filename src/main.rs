#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Embroidery Catalogue — Tauri v2 entry point

pub mod config;
pub mod database;
pub mod disclaimer;
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

/// Return the current execution mode and path metadata to the frontend.
#[tauri::command]
fn get_app_status(state: State<'_, AppState>) -> AppStatus {
    let mode_str = match state.paths.mode {
        paths::ExecutionMode::Portable => "portable".to_string(),
        paths::ExecutionMode::Installed => "installed".to_string(),
    };

    AppStatus {
        execution_mode: mode_str,
        data_root: state.paths.data_root.to_string_lossy().to_string(),
        embroidery_dir: state
            .paths
            .embroidery_designs_dir
            .to_string_lossy()
            .to_string(),
        database_path: state.paths.database_path.to_string_lossy().to_string(),
    }
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

/// Check whether the disclaimer has already been accepted for this installation.
#[tauri::command]
async fn check_disclaimer(state: State<'_, AppState>) -> Result<bool, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
    Ok(disclaimer::is_disclaimer_accepted(&mut conn).await)
}

/// Persist the user's disclaimer acceptance in the database.
#[tauri::command]
async fn accept_disclaimer(state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
    let ok = disclaimer::set_disclaimer_accepted(&mut conn, true).await;
    if ok {
        Ok(())
    } else {
        Err("Failed to save disclaimer acceptance to the database.".to_string())
    }
}

/// Return the disclaimer HTML text to the frontend.
#[tauri::command]
fn get_disclaimer_text(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.disclaimer_text.clone())
}

// ─── Application entry point ──────────────────────────────────────────────────

fn main() {
    // ── Resolve paths (must be first — before logging and DB) ────────
    let app_paths = paths::resolve_app_paths();
    tracing::info!(
        "Embroidery Catalogue starting — mode={:?}, data_root={}",
        app_paths.mode,
        app_paths.data_root.display()
    );

    // ── Logging ───────────────────────────────────────────────────────
    let log_guard = logging::init_logging(&app_paths.log_dir);
    tracing::info!("Logging initialised — log_dir={}", app_paths.log_dir.display());

    // Load .env file if present (best-effort; not required in production)
    load_dotenv();

    // Build bootstrap config from resolved paths
    let bootstrap_config = config::BootstrapConfig::from_app_paths(&app_paths);
    tracing::info!("Parsed bootstrap configuration: {:#?}", bootstrap_config);

    // Ensure the database directory exists before trying to connect
    config::ensure_database_dir(&bootstrap_config.database_url);

    // Run async setup using Tauri's built-in Tokio runtime
    // This avoids creating a conflicting second runtime alongside Tauri's own
    let (pool, disclaimer_text) = tauri::async_runtime::block_on(async {
        // Establish the SQLite connection pool using resolved paths
        let pool = database::connection::establish_connection(&app_paths)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to establish database connection: {}",
                    e
                )
            });

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
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    // Only set if not already present in the environment
                    if std::env::var(key.trim()).is_err() {
                        std::env::set_var(key.trim(), value.trim());
                    }
                }
            }
        }
    }
}