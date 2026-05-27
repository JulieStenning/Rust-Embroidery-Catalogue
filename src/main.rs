// Embroidery Catalogue — Tauri v2 entry point

pub mod database;
pub mod config;
pub mod settings;
pub mod disclaimer;
pub mod readers;
pub mod models;
pub mod png_writer;
pub mod templating;
pub mod utils;
pub mod routes;
pub mod services;

use sqlx::SqlitePool;
use tauri::State;

/// Shared application state managed by Tauri.
/// `SqlitePool` is `Send + Sync`, so no `Mutex` wrapper is needed.
pub struct AppState {
    /// Connection pool for the SQLite database.
    pub db: SqlitePool,
    /// The disclaimer HTML text, embedded at compile time from DISCLAIMER.html.
    pub disclaimer_text: String,
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
    // Load .env file if present (best-effort; not required in production)
    load_dotenv();

    // Resolve bootstrap configuration from process environment.
    let bootstrap_config = config::BootstrapConfig::from_env();
    println!("Parsed bootstrap configuration: {:#?}", bootstrap_config);

    // Ensure the database directory exists before trying to connect
    config::ensure_database_dir(&bootstrap_config.database_url);

    // Run async setup using Tauri's built-in Tokio runtime
    // This avoids creating a conflicting second runtime alongside Tauri's own
    let (pool, disclaimer_text) = tauri::async_runtime::block_on(async {
        // Establish the SQLite connection pool
        let pool = database::connection::establish_connection().await;

        // Run any pending migrations so the schema is always up to date
        database::migrations::run_migrations(&pool)
            .await
            .expect("Failed to run database migrations");

        // Embed the disclaimer text at compile time from DISCLAIMER.html
        let disclaimer_text = include_str!("../DISCLAIMER.html").to_string();

        (pool, disclaimer_text)
    });

    let app_state = AppState {
        db: pool,
        disclaimer_text,
    };

    routes::bulk_import::initialize_bulk_import_db_pool(app_state.db.clone());
    let startup_reset = routes::bulk_import::reset_bulk_import_context_store_for_startup();
    println!(
        "Bulk import context startup reset: cleared={}, active={}, resets={}, at_ms={}",
        startup_reset.cleared_context_count,
        startup_reset.active_context_count,
        startup_reset.reset_count,
        startup_reset.reset_at_millis
    );

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            routes::bulk_import::initialize_bulk_import_app_handle(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::debug_bootstrap_config,
            check_disclaimer,
            accept_disclaimer,
            get_disclaimer_text,
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
            routes::designs::add_design_to_project,
            routes::designs::remove_design_from_project,
            routes::designs::delete_design,
            routes::designs::open_design_in_editor,
            routes::designs::open_design_in_explorer,
            routes::designs::render_design_3d_preview,
            routes::settings::get_settings_view_model,
            routes::settings::save_settings_view_model,
            routes::settings::save_import_last_browse_folder,
            routes::settings::browse_settings_data_root,
            routes::admin::list_designers,
            routes::admin::create_designer,
            routes::admin::delete_designer,
            routes::admin::list_sources,
            routes::admin::create_source,
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
            routes::tagging_actions::preview_tagging_action,
            routes::maintenance::maintenance_scaffold_enabled,
        ])
        // tauri::generate_context!() reads tauri.conf.json from the project root
        .run(tauri::generate_context!())
        .expect("Error while running the Embroidery Catalogue application");
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

