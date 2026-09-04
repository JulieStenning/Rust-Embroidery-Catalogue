#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Embroidery Catalogue â€” Tauri v2 entry point

pub mod config;
pub mod database;
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
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::atomic::AtomicBool;
use tauri::{Manager, State};

// â”€â”€â”€ Shared Application State â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

// Shared application state managed by Tauri.
// `SqlitePool` is `Send + Sync`, so no `Mutex` wrapper is needed.
// ---------------------------------------------------------------------------
// Database status (exposed to frontend for the recovery flow)
// ---------------------------------------------------------------------------

/// Tri-state status of the configured database at startup.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseStatusKind {
    /// No configured data root yet - first-run setup wizard handles it.
    Uninitialized,
    /// The configured database file exists and was opened normally.
    Connected,
    /// A configured data root exists but the database file is missing
    /// (e.g. a portable drive letter changed). The recovery view handles it.
    Missing,
}

/// Detailed database status report sent to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStatus {
    pub status: DatabaseStatusKind,
    pub configured_data_root: Option<String>,
    pub database_path: Option<String>,
    pub embroidery_dir: Option<String>,
    pub data_root_missing: bool,
}

/// Compute the database status for the current paths/configuration.
fn database_status_from_paths(paths: &paths::AppPaths) -> DatabaseStatus {
    let configured_root = paths::read_bootstrap_data_root().ok().flatten();

    let configured_str = configured_root
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());
    let database_str = Some(paths.database_path.to_string_lossy().to_string());
    let embroidery_str = Some(paths.embroidery_designs_dir.to_string_lossy().to_string());

    let data_root_missing = matches!(paths.mode, paths::ExecutionMode::Installed)
        && configured_root
            .as_ref()
            .map(|root| !root.exists())
            .unwrap_or(false);

    let status = match configured_root {
        None => DatabaseStatusKind::Uninitialized,
        Some(_) if data_root_missing => DatabaseStatusKind::Missing,
        Some(_) if !paths.database_path.exists() => DatabaseStatusKind::Missing,
        Some(_) => DatabaseStatusKind::Connected,
    };

    DatabaseStatus {
        status,
        configured_data_root: configured_str,
        database_path: database_str,
        embroidery_dir: embroidery_str,
        data_root_missing,
    }
}

// ---------------------------------------------------------------------------

/// Holds the live SQLite pool in a way that can be swapped at runtime for a
/// database restore.
///
/// Cloning the holder is cheap (an `Arc`), and `pool()` clones the underlying
/// `SqlitePool` handle so commands keep a stable pool across `.await` points.
/// A restore takes the current pool out (closing it), replaces the database
/// file on disk, and installs a fresh pool via `replace`.
#[derive(Clone, Default)]
pub struct PoolHolder {
    inner: std::sync::Arc<std::sync::Mutex<Option<SqlitePool>>>,
}

impl PoolHolder {
    /// Wrap a freshly-created pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(Some(pool))),
        }
    }

    /// Clone the currently installed pool, or `None` if a restore has removed it.
    pub fn pool(&self) -> Option<SqlitePool> {
        self.inner
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())
    }

    /// Remove and return the current pool so it can be closed before a restore
    /// swaps the underlying database file. Returns `None` if already absent.
    pub fn take(&self) -> Option<SqlitePool> {
        self.inner.lock().ok().and_then(|mut guard| guard.take())
    }

    /// Install a new pool after a restore, dropping (and thereby closing) any
    /// previous one. Safe to call even if no pool is currently installed.
    pub fn replace(&self, new: SqlitePool) {
        let previous = self
            .inner
            .lock()
            .ok()
            .and_then(|mut guard| guard.replace(new));
        // A `SqlitePool` closes itself when the last handle is dropped.
        drop(previous);
    }
}

/// Shared application state managed by Tauri.
/// The pool is held in a `PoolHolder` so a restore can close and replace it;
/// commands obtain a cheap clone via `AppState::db_pool`.
pub struct AppState {
    /// Connection pool for the SQLite database (swappable at runtime).
    pub db: PoolHolder,
    /// Status of the configured database (Connected / Missing / Uninitialized).
    pub database_status: DatabaseStatus,
    /// Resolved application paths (Portable vs Installed mode).
    pub paths: paths::AppPaths,
    /// Log guard â€” kept alive so log writes are flushed on app exit.
    pub log_guard: logging::LogGuard,
    /// Flag signalled when the app is shutting down; background tasks can check it.
    pub shutdown_requested: AtomicBool,
    /// Atomic guard preventing overlapping incremental-vacuum maintenance runs.
    pub maintenance_running: AtomicBool,
    /// True while a catalogue storage migration is in progress.
    pub migration_running: AtomicBool,
    /// Cooperative cancellation flag observed by the running migration loop.
    pub migration_cancel_requested: std::sync::Arc<AtomicBool>,
    /// True while a database restore is closing/swapping the live pool, so other
    /// commands fail fast instead of acquiring a closed pool.
    pub restore_in_progress: AtomicBool,
}

impl AppState {
    /// Clone the current live database pool for use by a command. Fails fast
    /// while a database restore is swapping the pool so no command touches a
    /// closed pool.
    pub fn db_pool(&self) -> Result<SqlitePool, String> {
        if self
            .restore_in_progress
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err("The database is being restored; please retry shortly.".to_string());
        }
        self.db
            .pool()
            .ok_or_else(|| "The database pool is unavailable.".to_string())
    }
}

// â”€â”€â”€ AppStatus (exposed to frontend) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub execution_mode: String,
    pub data_root: String,
    pub embroidery_dir: String,
    pub database_path: String,
    /// True when a previously-configured data root is no longer present on disk
    /// (e.g. a portable drive letter changed). The frontend offers a recovery
    /// dialog to reselect the location.
    pub data_root_missing: bool,
    /// True when a configured data root exists but the database file is missing.
    pub database_missing: bool,
}

/// Pure function to construct an `AppStatus` from `AppPaths`.
/// Extracted for testability â€” this does not depend on Tauri state.
fn app_status_from_paths(paths: &paths::AppPaths) -> AppStatus {
    let mode_str = match paths.mode {
        paths::ExecutionMode::Dev => "dev".to_string(),
        paths::ExecutionMode::Installed => "installed".to_string(),
    };

    // Only Installed mode can have a configured-then-missing root; Dev mode
    // always resolves to the project dev_data folder.
    let data_root_missing = matches!(paths.mode, paths::ExecutionMode::Installed)
        && paths::configured_data_root_missing()
            .ok()
            .flatten()
            .unwrap_or(false);

    // A database is "missing" when a data root is configured but the derived
    // DB file does not exist. This is the recovery-flow condition.
    let has_configured_root = matches!(paths.mode, paths::ExecutionMode::Installed)
        && paths::read_bootstrap_data_root().ok().flatten().is_some();
    let database_missing = has_configured_root && !paths.database_path.exists();

    AppStatus {
        execution_mode: mode_str,
        data_root: paths.data_root.to_string_lossy().to_string(),
        embroidery_dir: paths.embroidery_designs_dir.to_string_lossy().to_string(),
        database_path: paths.database_path.to_string_lossy().to_string(),
        data_root_missing,
        database_missing,
    }
}

/// Return the current execution mode and path metadata to the frontend.
#[tauri::command]
fn get_app_status(state: State<'_, AppState>) -> AppStatus {
    app_status_from_paths(&state.paths)
}

/// Return the detailed database status used by the recovery flow.
#[tauri::command]
fn get_database_status(state: State<'_, AppState>) -> DatabaseStatus {
    state.database_status.clone()
}

/// Return the persisted, user-configured data root for Installed mode.
///
/// Returns `None` on first run (no config yet) or when running in a non-
/// Installed mode. The frontend uses this to decide whether the setup wizard
/// must prompt for a data location.
#[tauri::command]
fn get_configured_data_root() -> Result<Option<String>, String> {
    match paths::read_bootstrap_data_root() {
        Ok(Some(root)) => Ok(Some(root.to_string_lossy().to_string())),
        Ok(None) => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

/// Persist the user-chosen data root for Installed mode.
///
/// The path must be absolute. This writes the tiny `config.json` under the
/// platform app-data dir so the choice survives reinstalls.
#[tauri::command]
fn set_configured_data_root(data_root: String) -> Result<(), String> {
    let trimmed = data_root.trim();
    if trimmed.is_empty() {
        return Err("Data root cannot be empty.".to_string());
    }
    let path = std::path::PathBuf::from(trimmed);
    paths::write_bootstrap_data_root(&path).map_err(|err| err.to_string())
}

/// Result returned by `configure_fresh_data_root` to report whether an existing
/// database was detected and preserved or a new seed database was copied.
#[derive(Debug, Clone, Serialize)]
pub struct ConfigureDataRootResult {
    pub data_root: String,
    pub existing_database_detected: bool,
    pub database_path: String,
}

/// Persist the user-chosen data root and initialize catalogue storage.
///
/// If an existing database (`EmbroideryCatalogue.db`) is detected within the
/// target directory, it is preserved without copying or overwriting seed data.
/// If no database exists, the seed database is copied to `<data_root>/Database/`.
#[tauri::command]
fn configure_fresh_data_root(data_root: String) -> Result<ConfigureDataRootResult, String> {
    let trimmed = data_root.trim();
    if trimmed.is_empty() {
        return Err("Data root cannot be empty.".to_string());
    }
    let path = std::path::PathBuf::from(trimmed);
    let seeded_fresh =
        paths::ensure_catalogue_layout_and_seed_if_missing(&path).map_err(|err| err.to_string())?;
    paths::write_bootstrap_data_root(&path).map_err(|err| err.to_string())?;

    let database_path = path
        .join("Database")
        .join(paths::DATABASE_FILENAME)
        .to_string_lossy()
        .to_string();

    Ok(ConfigureDataRootResult {
        data_root: path.to_string_lossy().to_string(),
        existing_database_detected: !seeded_fresh,
        database_path,
    })
}

/// Open a native folder picker to choose the data root for Installed mode.
///
/// Follows the existing `browse_backup_folder`/`browse_import_folder` pattern
/// using the `rfd` crate. Returns `Ok(None)` when the user cancels.
#[tauri::command]
fn browse_data_root_folder(start_dir: Option<String>) -> Result<Option<String>, String> {
    let mut dialog =
        rfd::FileDialog::new().set_title("Choose a folder for your Embroidery Catalogue data");
    if let Some(dir) = start_dir.filter(|d| !d.trim().is_empty()) {
        dialog = dialog.set_directory(&dir);
    }
    let picked = dialog.pick_folder();
    Ok(picked.map(|p| p.to_string_lossy().to_string()))
}

/// Restart the application process with the exact same command-line arguments
/// it was started with. Used by the frontend after the initial-setup wizard
/// relocates the data root so the new location takes effect immediately.
///
/// Uses Tauri's built-in `request_restart`, which spawns a fresh copy of the
/// executable with the original arguments and then exits the **current**
/// process. The run loop relaunches the new instance as part of a normal,
/// clean shutdown. This avoids leaving a stale window open with a disabled
/// dialog after a restart.
///
/// Returns `Ok(true)` after the restart has been requested (the current
/// process is about to exit and be replaced).
#[tauri::command]
fn restart_application(app_handle: tauri::AppHandle) -> Result<bool, String> {
    // `request_restart` sets Tauri's restart flag and requests a clean exit;
    // the app's run loop then relaunches the same binary with the original
    // args. The command returns normally so the frontend's promise resolves
    // before the window closes.
    app_handle.request_restart();
    Ok(true)
}

// â”€â”€â”€ Tauri Commands â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Check whether the initial setup wizard has been completed or skipped.
#[tauri::command]
async fn check_initial_setup(state: State<'_, AppState>) -> Result<bool, String> {
    let pool = state.db_pool()?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    initial_setup::is_initial_setup_completed(&mut conn)
        .await
        .map_err(|err| err.to_string())
}

/// Persist that the user has completed or skipped the initial setup wizard.
#[tauri::command]
async fn complete_initial_setup(state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.db_pool()?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
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
        "Embroidery Catalogue starting - mode={:?}, data_root={}",
        app_paths.mode,
        app_paths.data_root.display()
    );

    // Determine whether we are in database-recovery mode. This happens in
    // Installed mode when a data root has been configured but the derived
    // database file is missing (e.g. a portable drive letter changed from D:
    // to E:). In that case we must NOT run schema migrations, create catalogue
    // folders under the stale root, or mount the normal app: the recovery view
    // asks the user to re-point the location before anything else happens.
    let recovery_mode = paths::database_recovery_mode(&app_paths);

    // Logging: while recovering, write logs to a safe temp location instead of
    // the stale configured root so no `logs` folder is created under e.g.
    // `F:\` before the user re-points the real location.
    let log_dir = if recovery_mode {
        let fallback = paths::recovery_log_dir();
        tracing::info!(
            "Database recovery mode: logging to fallback temp dir {} instead of {}",
            fallback.display(),
            app_paths.log_dir.display()
        );
        fallback
    } else {
        app_paths.log_dir.clone()
    };
    let log_guard = match logging::init_logging(&log_dir) {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to initialize logging: {err}");
            std::process::exit(1);
        }
    };
    tracing::info!("Logging initialised - log_dir={}", log_dir.display());

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

    // Compute the database status for state registration.
    let database_status = database_status_from_paths(&app_paths);

    // Run async setup using Tauri's built-in Tokio runtime
    // This avoids creating a conflicting second runtime alongside Tauri's own
    let pool = if recovery_mode {
        tracing::warn!(
            "Database recovery mode: configured database missing at {} - awaiting user re-pointing.",
            app_paths.database_path.display()
        );
        // Throwaway in-memory pool: keeps AppState constructible and commands
        // registered without touching the missing real DB file. The blocking
        // recovery view guarantees no data-touching command runs against it.
        tauri::async_runtime::block_on(async {
            SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .unwrap_or_else(|e| {
                    eprintln!("Failed to establish in-memory recovery pool: {e}");
                    std::process::exit(1);
                })
        })
    } else {
        // Ensure the database directory exists before trying to connect
        if let Err(err) = config::ensure_database_dir(&bootstrap_config.database_url) {
            eprintln!("Failed to create database directory: {err}");
            std::process::exit(1);
        }

        tauri::async_runtime::block_on(async {
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

            pool
        })
    };

    let app_state = AppState {
        db: PoolHolder::new(pool),
        database_status,
        paths: app_paths,
        log_guard,
        shutdown_requested: AtomicBool::new(false),
        maintenance_running: AtomicBool::new(false),
        migration_running: AtomicBool::new(false),
        migration_cancel_requested: std::sync::Arc::new(AtomicBool::new(false)),
        restore_in_progress: AtomicBool::new(false),
    };

    // Launch a lightweight background backfill for orphan fingerprint data
    // (hash + file size).  This is fire-and-forget â€” errors are logged, not fatal.
    if recovery_mode {
        // In recovery mode the frontend is blocked on the DatabaseRecoveryView;
        // none of the normal background DB work (fingerprint backfill, bulk
        // import pool, health monitor) should run against the in-memory pool.
        tracing::info!("Skipping background DB initialisation during database recovery mode.");
    } else {
        let fp_pool = match app_state.db_pool() {
            Ok(pool) => pool,
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        };
        routes::bulk_import::initialize_bulk_import_db_pool(fp_pool.clone());
        tauri::async_runtime::spawn(async move {
            if let Err(err) = services::fingerprint::run_fingerprint_backfill(&fp_pool, 100).await {
                tracing::error!("Startup fingerprint backfill error: {}", err);
            }
        });
        let startup_reset = routes::bulk_import::reset_bulk_import_context_store_for_startup();
        tracing::info!(
            "Bulk import context startup reset: cleared={}, active={}, resets={}, at_ms={}",
            startup_reset.cleared_context_count,
            startup_reset.active_context_count,
            startup_reset.reset_count,
            startup_reset.reset_at_millis
        );
    }

    let app = tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            routes::bulk_import::initialize_bulk_import_app_handle(app.handle().clone());
            routes::tagging_actions::initialize_backfill_app_handle(app.handle().clone());

            // â”€â”€ Database health monitor: startup check + idle interval â”€â”€â”€â”€â”€â”€
            // Reads the configured idle interval from the DB (default 1800s),
            // runs an immediate fragmentation check on startup, then checks on
            // an idle timer. All compaction is fire-and-forget and non-blocking.
            {
                let state = app.state::<AppState>();
                let pool = state
                    .db_pool()
                    .expect("database pool available during setup");
                let maintenance_flag =
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                let shutdown_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

                // NOTE: The immediate startup health check is NOT spawned here.
                // The single-connection pool (max_connections=1) is shared with
                // the initial UI queries, so running a health check at this
                // exact moment can starve a request and cause "pool timed out
                // waiting for an open connection". The startup check is instead
                // launched from the app run loop (see below) after a short
                // delay so first-launch queries complete first.

                // Idle interval task.
                let idle_pool = pool.clone();
                let idle_maintenance = maintenance_flag.clone();
                let idle_shutdown = shutdown_flag.clone();
                let idle_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Start with the default, then pick up the persisted value
                    // from the settings table on each tick so Settings UI
                    // changes take effect without restart.
                    let mut interval_secs = services::db_health::DEFAULT_IDLE_CHECK_INTERVAL_SECS;

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
            get_database_status,
            get_configured_data_root,
            set_configured_data_root,
            configure_fresh_data_root,
            browse_data_root_folder,
            restart_application,
            config::debug_bootstrap_config,
            check_initial_setup,
            complete_initial_setup,
            routes::database_recovery::detect_relocated_data_root,
            routes::database_recovery::validate_database_path,
            routes::database_recovery::seed_database_to_data_root,
            routes::about::get_about_documents,
            routes::about::get_about_document,
            routes::designs::get_designs,
            routes::designs::get_design_ids,
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
            routes::designs::set_design_verification,
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
            routes::settings::list_gemini_models,
            routes::settings::test_gemini_model,
            routes::settings::save_import_last_browse_folder,
            routes::settings::browse_settings_data_root,
            routes::settings::get_google_api_key,
            routes::settings::set_google_api_key,
            routes::storage_migration::start_catalogue_storage_migration,
            routes::storage_migration::cancel_catalogue_storage_migration,
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
            routes::tagging_actions::count_tagging_candidates,
            routes::tagging_actions::browse_tagging_folder,
            routes::tagging_actions::run_stitching_backfill,
            routes::tagging_actions::run_fingerprint_backfill,
            routes::maintenance::maintenance_scaffold_enabled,
            routes::maintenance::get_backup_view_model,
            routes::maintenance::save_backup_settings,
            routes::maintenance::browse_backup_folder,
            routes::maintenance::request_cancel_backup,
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
            routes::restore::browse_restore_file,
            routes::restore::restore_database,
            routes::restore::restore_designs_incremental,
            routes::restore::restore_both,
            routes::restore::detect_design_files_absent_from_database,
            routes::restore::import_unmatched_design_files,
            routes::restore::request_cancel_restore,
        ])
        // tauri::generate_context!() reads tauri.conf.json from the project root
        .build(tauri::generate_context!())
        .expect("Error while building the Embroidery Catalogue application");

    {
        // Startup database health check (fire-and-forget; logged, never fatal).
        // Spawned after the Tauri app is fully running and after a short delay
        // so the initial UI queries have used the shared single-connection
        // pool first â€” preventing the "pool timed out waiting for an open
        // connection" error.
        let state = app.state::<AppState>();
        let pool = state.db_pool().expect("database pool available at startup");
        let maintenance_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
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
                let pool = state.db_pool().expect("database pool available on exit");
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
