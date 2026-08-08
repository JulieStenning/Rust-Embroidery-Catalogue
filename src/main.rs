#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Embroidery Catalogue â€” Tauri v2 entry point

pub mod config;
pub mod database;
pub mod disclaimer;
pub mod error;
pub mod initial_setup;
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

// â”€â”€â”€ Shared Application State â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Shared application state managed by Tauri.
/// `SqlitePool` is `Send + Sync`, so no `Mutex` wrapper is needed.
pub struct AppState {
    /// Connection pool for the SQLite database.
    pub db: SqlitePool,
    /// Resolved application paths (Portable vs Installed mode).
    pub paths: paths::AppPaths,
    /// The disclaimer HTML text, embedded at compile time from DISCLAIMER.html.
    pub disclaimer_text: String,
    /// Log guard â€” kept alive so log writes are flushed on app exit.
    pub log_guard: logging::LogGuard,
    /// Flag signalled when the app is shutting down; background tasks can check it.
    pub shutdown_requested: AtomicBool,
    /// Atomic guard preventing overlapping incremental-vacuum maintenance runs.
    pub maintenance_running: AtomicBool,
}

// â”€â”€â”€ AppStatus (exposed to frontend) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub execution_mode: String,
    pub data_root: String,
    pub embroidery_dir: String,
    pub database_path: String,
}

/// Pure function to construct an `AppStatus` from `AppPaths`.
/// Extracted for testability â€” this does not depend on Tauri state.
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

// â”€â”€â”€ Tauri Commands â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

/// Check whether the initial setup wizard has been completed or skipped.
#[tauri::command]
async fn check_initial_setup(state: State<'_, AppState>) -> Result<bool, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
    initial_setup::is_initial_setup_completed(&mut conn)
        .await
        .map_err(|err| err.to_string())
}

/// Persist that the user has completed or skipped the initial setup wizard.
#[tauri::command]
async fn complete_initial_setup(state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
    initial_setup::set_initial_setup_completed(&mut conn, true)
        .await
        .map(|_| ())
        .map_err(|err| format!("Failed to save initial setup status to the database: {err}"))
}

// â”€â”€â”€ Application entry point â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

fn main() {
    // â”€â”€ Resolve paths (must be first â€” before logging and DB) â”€â”€â”€â”€â”€â”€â”€â”€
    let app_paths = match paths::resolve_app_paths() {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("Failed to resolve application paths: {err}");
            std::process::exit(1);
        }
    };
    tracing::info!(
        "Embroidery Catalogue starting â€” mode={:?}, data_root={}",
        app_paths.mode,
        app_paths.data_root.display()
    );

    // â”€â”€ Logging â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    let log_guard = match logging::init_logging(&app_paths.log_dir) {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to initialize logging: {err}");
            std::process::exit(1);
        }
    };
    tracing::info!("Logging initialised â€” log_dir={}", app_paths.log_dir.display());

    // Load .env file if present (best-effort; not required in production)
    load_dotenv();

    // Build bootstrap config from resolved paths
    let bootstrap_config = config::BootstrapConfig::from_app_paths(&app_paths);
    tracing::info!("Parsed bootstrap configuration: {:#?}", bootstrap_config);

    // Export the resolved database URL as DATABASE_URL so that legacy
    // path-derivation helpers (bulk_import, designs, fingerprint, maintenance)
    // which read BootstrapConfig::from_env() agree with the startup connection
    // path. Without this they fall back to the relative default
    // "sqlite:data/database/EmbroideryCatalogue.db", resolving design paths
    // against the project root (./data) instead of the resolved AppPaths data
    // root (e.g. target\debug\Data).
    std::env::set_var("DATABASE_URL", &bootstrap_config.database_url);

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
        maintenance_running: AtomicBool::new(false),
    };

    // Launch a lightweight background backfill for orphan fingerprint data
    // (hash + file size).  This is fire-and-forget â€” errors are logged, not fatal.
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

            // â”€â”€ Database health monitor: startup check + idle interval â”€â”€â”€â”€â”€â”€
            // Reads the configured idle interval from the DB (default 1800s),
            // runs an immediate fragmentation check on startup, then checks on
            // an idle timer. All compaction is fire-and-forget and non-blocking.
            {
                let state = app.state::<AppState>();
                let pool = state.db.clone();
                let maintenance_flag = std::sync::Arc::new(
                    std::sync::atomic::AtomicBool::new(false),
                );
                let shutdown_flag = std::sync::Arc::new(
                    std::sync::atomic::AtomicBool::new(false),
                );

                // NOTE: The immediate startup health check is NOT spawned here.
                // The single-connection pool (max_connections=1) is shared with
                // `check_disclaimer` and the initial UI queries, so running a
                // health check at this exact moment can starve the disclaimer
                // request and cause "pool timed out waiting for an open
                // connection". The startup check is instead launched from the
                // app run loop (see below) after a short delay so first-launch
                // queries complete first.

                // Idle interval task.
                let idle_pool = pool.clone();
                let idle_maintenance = maintenance_flag.clone();
                let idle_shutdown = shutdown_flag.clone();
                let idle_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Start with the default, then pick up the persisted value
                    // from the settings table on each tick so Settings UI
                    // changes take effect without restart.
                    let mut interval_secs =
                        services::db_health::DEFAULT_IDLE_CHECK_INTERVAL_SECS;

                    loop {
                        // Re-read the persisted interval each cycle.
                        if let Ok(secs) = read_idle_interval_from_db(&idle_pool).await {
                            interval_secs = secs;
                        }
                        let mut interval =
                            tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                        interval.tick().await; // consume the first immediate tick
                        interval.tick().await; // wait for the first real interval

                        if idle_shutdown.load(std::sync::atomic::Ordering::SeqCst) {
                            break;
                        }

                        if let Err(err) = services::db_health::check_and_schedule_maintenance(
                            idle_pool.clone(),
                            idle_maintenance.clone(),
                            idle_shutdown.clone(),
                            idle_handle.clone(),
                        )
                        .await
                        {
                            tracing::warn!("Idle DB health check failed: {}", err);
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            config::debug_bootstrap_config,
            check_disclaimer,
            accept_disclaimer,
            get_disclaimer_text,
            check_initial_setup,
            complete_initial_setup,
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
            routes::designs::reparse_design_file,
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
            routes::admin::update_tag,
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
            routes::maintenance::get_db_stats,
            routes::maintenance::compact_database,
            routes::maintenance::scan_orphans,
            routes::maintenance::get_orphans_page,
            routes::maintenance::delete_orphans,
            routes::maintenance::delete_all_orphans,
            routes::maintenance::browse_orphan_path,
        ])
        // tauri::generate_context!() reads tauri.conf.json from the project root
        .build(tauri::generate_context!())
        .expect("Error while building the Embroidery Catalogue application");

    {
        // Startup database health check (fire-and-forget; logged, never fatal).
        // Spawned after the Tauri app is fully running and after a short delay
        // so the disclaimer check and initial UI queries have used the shared
        // single-connection pool first â€” preventing the "pool timed out
        // waiting for an open connection" error.
        let state = app.state::<AppState>();
        let pool = state.db.clone();
        let maintenance_flag = std::sync::Arc::new(
            std::sync::atomic::AtomicBool::new(false),
        );
        let shutdown_flag = std::sync::Arc::new(
            std::sync::atomic::AtomicBool::new(false),
        );
        let startup_handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            if let Err(err) = services::db_health::check_and_schedule_maintenance(
                pool,
                maintenance_flag,
                shutdown_flag,
                startup_handle,
            )
            .await
            {
                tracing::warn!("Startup DB health check failed: {}", err);
            }
        });
    }

    app.run(|app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { code, .. } => {
                tracing::info!(
                    "Exit requested (code: {:?}) â€” signalling shutdown...",
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

// â”€â”€â”€ Helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Read the persisted idle-check interval (seconds) from the settings table.
/// Returns the default on missing/invalid values or DB errors, so the idle
/// monitor never fails due to a transient read failure.
async fn read_idle_interval_from_db(pool: &SqlitePool) -> Result<u64, String> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM settings WHERE key = 'db.idle_check_interval_secs' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some((value,)) => value
            .trim()
            .parse::<u64>()
            .map_err(|e| format!("Invalid idle interval '{}': {}", value.trim(), e)),
        None => Ok(services::db_health::DEFAULT_IDLE_CHECK_INTERVAL_SECS),
    }
}

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
#[path = "main_tests.rs"]
mod tests;

