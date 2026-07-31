use crate::paths::ExecutionMode;
use crate::settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use std::path::Path;
use tauri::State;

const KEY_AI_TIER2_AUTO: &str = "ai.tier2_auto";
const KEY_AI_TIER3_AUTO: &str = "ai.tier3_auto";
const KEY_AI_BATCH_SIZE: &str = "ai.batch_size";
const KEY_AI_DELAY: &str = "ai.delay";
const KEY_IMPORT_COMMIT_BATCH_SIZE: &str = "import.commit_batch_size";
const KEY_IMPORT_LAST_BROWSE_FOLDER: &str = "import.last_browse_folder";
const KEY_IMAGE_PREFERENCE: &str = "image.preference";
const KEY_PREVIEW_3D_PROFILE: &str = "image.preview_3d_profile";

#[derive(Debug, Clone, Serialize)]
pub struct SettingsViewModel {
    pub image_preference: String,
    pub preview_3d_profile: String,
    pub google_api_key: String,
    pub has_google_api_key: bool,
    pub ai_tier2_auto: bool,
    pub ai_tier3_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    pub import_commit_batch_size: String,
    pub import_last_browse_folder: String,
    pub can_configure_data_root: bool,
    pub data_root: String,
    pub database_path: String,
    pub log_folder: String,
    pub app_mode: String,
    pub ai_tagging_help_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveSettingsRequest {
    pub image_preference: String,
    #[serde(default)]
    pub preview_3d_profile: String,
    pub google_api_key: String,
    pub ai_tier2_auto: bool,
    pub ai_tier3_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    pub import_commit_batch_size: String,
    pub data_root: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveSettingsResult {
    pub saved: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveImportBrowseFolderResult {
    pub saved: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowseDataRootResult {
    pub path: Option<String>,
    pub error: Option<String>,
}

pub(crate) async fn get_settings_view_model_inner(
    app_state: &AppState,
) -> Result<SettingsViewModel, String> {
    let mut conn = app_state.db.acquire().await.map_err(|e| e.to_string())?;

    let image_preference = get_setting_with_default(&mut conn, KEY_IMAGE_PREFERENCE)
        .await
        .map_err(|e| e.to_string())?;
    let preview_3d_profile = get_setting_with_default(&mut conn, KEY_PREVIEW_3D_PROFILE)
        .await
        .map_err(|e| e.to_string())?;
    let ai_tier2_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_TIER2_AUTO)
            .await
            .map_err(|e| e.to_string())?,
    );
    let ai_tier3_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_TIER3_AUTO)
            .await
            .map_err(|e| e.to_string())?,
    );
    let ai_batch_size = get_setting_with_default(&mut conn, KEY_AI_BATCH_SIZE)
        .await
        .map_err(|e| e.to_string())?;
    let ai_delay = get_setting_with_default(&mut conn, KEY_AI_DELAY)
        .await
        .map_err(|e| e.to_string())?;
    let import_commit_batch_size =
        get_setting_with_default(&mut conn, KEY_IMPORT_COMMIT_BATCH_SIZE)
            .await
            .map_err(|e| e.to_string())?;
    let import_last_browse_folder =
        get_setting_with_default(&mut conn, KEY_IMPORT_LAST_BROWSE_FOLDER)
            .await
            .map_err(|e| e.to_string())?;

    let google_api_key = std::env::var("GOOGLE_API_KEY").unwrap_or_default();
    let has_google_api_key = !google_api_key.trim().is_empty();

    // Derive paths and mode from the centrally resolved AppPaths in state
    let data_root = app_state.paths.data_root.to_string_lossy().to_string();
    let database_path = app_state.paths.database_path.to_string_lossy().to_string();
    let log_folder = app_state.paths.log_dir.to_string_lossy().to_string();
    let can_configure_data_root = match app_state.paths.mode {
        ExecutionMode::Portable => false,
        ExecutionMode::Installed => true,
    };
    let app_mode = match app_state.paths.mode {
        ExecutionMode::Portable => "portable".to_string(),
        ExecutionMode::Installed => "installed".to_string(),
    };

    Ok(SettingsViewModel {
        image_preference,
        preview_3d_profile,
        google_api_key,
        has_google_api_key,
        ai_tier2_auto,
        ai_tier3_auto,
        ai_batch_size,
        ai_delay,
        import_commit_batch_size,
        import_last_browse_folder,
        can_configure_data_root,
        data_root,
        database_path,
        log_folder,
        app_mode,
        ai_tagging_help_url: "#/help".to_string(),
    })
}

#[tauri::command]
pub async fn get_settings_view_model(
    state: State<'_, AppState>,
) -> Result<SettingsViewModel, String> {
    get_settings_view_model_inner(&*state).await
}

pub(crate) async fn save_import_last_browse_folder_inner(
    app_state: &AppState,
    path: String,
) -> Result<SaveImportBrowseFolderResult, String> {
    let normalized = path.trim().to_string();
    let mut conn = app_state.db.acquire().await.map_err(|e| e.to_string())?;

    upsert_setting(&mut conn, KEY_IMPORT_LAST_BROWSE_FOLDER, &normalized)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SaveImportBrowseFolderResult {
        saved: true,
        path: normalized,
    })
}

#[tauri::command]
pub async fn save_import_last_browse_folder(
    state: State<'_, AppState>,
    path: String,
) -> Result<SaveImportBrowseFolderResult, String> {
    save_import_last_browse_folder_inner(&*state, path).await
}

pub(crate) async fn save_settings_view_model_inner(
    app_state: &AppState,
    request: SaveSettingsRequest,
) -> Result<SaveSettingsResult, String> {
    let image_preference = normalize_image_preference(&request.image_preference);
    let preview_3d_profile = normalize_preview_3d_profile(&request.preview_3d_profile);
    let ai_batch_size = normalize_optional_batch_size(&request.ai_batch_size);
    let import_commit_batch_size = normalize_optional_batch_size(&request.import_commit_batch_size);
    let ai_delay = normalize_optional_delay(&request.ai_delay);

    let mut conn = app_state.db.acquire().await.map_err(|e| e.to_string())?;

    upsert_setting(
        &mut conn,
        KEY_AI_TIER2_AUTO,
        bool_to_setting(request.ai_tier2_auto),
    )
    .await
    .map_err(|e| e.to_string())?;
    upsert_setting(
        &mut conn,
        KEY_AI_TIER3_AUTO,
        bool_to_setting(request.ai_tier3_auto),
    )
    .await
    .map_err(|e| e.to_string())?;
    upsert_setting(&mut conn, KEY_AI_BATCH_SIZE, &ai_batch_size)
        .await
        .map_err(|e| e.to_string())?;
    upsert_setting(&mut conn, KEY_AI_DELAY, &ai_delay)
        .await
        .map_err(|e| e.to_string())?;
    upsert_setting(
        &mut conn,
        KEY_IMPORT_COMMIT_BATCH_SIZE,
        &import_commit_batch_size,
    )
    .await
    .map_err(|e| e.to_string())?;
    upsert_setting(&mut conn, KEY_IMAGE_PREFERENCE, &image_preference)
        .await
        .map_err(|e| e.to_string())?;
    upsert_setting(&mut conn, KEY_PREVIEW_3D_PROFILE, &preview_3d_profile)
        .await
        .map_err(|e| e.to_string())?;

    save_google_api_key_to_env(&request.google_api_key)?;

    // Data-root persistence is intentionally deferred until desktop mode support is fully wired.
    let _ = request.data_root;

    Ok(SaveSettingsResult {
        saved: true,
        message: "Settings saved successfully.".to_string(),
    })
}

#[tauri::command]
pub async fn save_settings_view_model(
    state: State<'_, AppState>,
    request: SaveSettingsRequest,
) -> Result<SaveSettingsResult, String> {
    save_settings_view_model_inner(&*state, request).await
}

#[tauri::command]
pub fn browse_settings_data_root(start_dir: Option<String>) -> BrowseDataRootResult {
    let from = start_dir.unwrap_or_default();
    BrowseDataRootResult {
        path: None,
        error: Some(format!(
            "Folder picker is not wired yet in this build. Please enter the path manually. Start directory was: {}",
            if from.trim().is_empty() { "(blank)" } else { from.trim() }
        )),
    }
}

async fn get_setting_with_default(
    conn: &mut SqliteConnection,
    key: &str,
) -> Result<String, sqlx::Error> {
    let current = settings::get_setting(conn, key).await?;
    if let Some(setting) = current {
        return Ok(setting.value);
    }

    let fallback = default_for_key(key).to_string();
    upsert_setting(conn, key, &fallback).await?;
    Ok(fallback)
}

async fn upsert_setting(
    conn: &mut SqliteConnection,
    key: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings (key, value, description) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .bind(description_for_key(key))
    .execute(conn)
    .await?;
    Ok(())
}

fn default_for_key(key: &str) -> &'static str {
    match key {
        KEY_AI_TIER2_AUTO => "false",
        KEY_AI_TIER3_AUTO => "false",
        KEY_AI_BATCH_SIZE => "",
        KEY_AI_DELAY => "",
        KEY_IMPORT_COMMIT_BATCH_SIZE => "",
        KEY_IMAGE_PREFERENCE => "2d",
        KEY_PREVIEW_3D_PROFILE => "balanced",
        _ => "",
    }
}

fn description_for_key(key: &str) -> &'static str {
    match key {
        KEY_AI_TIER2_AUTO => "Run Tier 2 (Gemini text AI) automatically during import when a Google API key is present.",
        KEY_AI_TIER3_AUTO => "Run Tier 3 (Gemini vision AI) automatically during import when a Google API key is present.",
        KEY_AI_BATCH_SIZE => "Maximum number of designs to tag with AI per import run. Leave blank to tag all imported designs.",
        KEY_AI_DELAY => "Seconds to wait between Gemini API calls. Leave blank to use the default (5.0 seconds).",
        KEY_IMPORT_COMMIT_BATCH_SIZE => "Maximum number of designs to persist or update before each database commit during import. Leave blank to use the default batch size (10).",
        KEY_IMPORT_LAST_BROWSE_FOLDER => "Most recently used folder for the bulk import picker.",
        KEY_IMAGE_PREFERENCE => "Preferred preview image type for import-created previews: 2d or 3d.",
        KEY_PREVIEW_3D_PROFILE => "3D preview style profile for native rendering: soft, balanced, or high-contrast.",
        _ => "",
    }
}

fn normalize_image_preference(raw: &str) -> String {
    let value = raw.trim().to_ascii_lowercase();
    if value == "3d" {
        "3d".to_string()
    } else {
        "2d".to_string()
    }
}

fn normalize_preview_3d_profile(raw: &str) -> String {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "soft" => "soft".to_string(),
        "high-contrast" | "high_contrast" | "highcontrast" => "high-contrast".to_string(),
        _ => "balanced".to_string(),
    }
}

fn normalize_optional_batch_size(raw: &str) -> String {
    let value = raw.trim();
    if value.is_empty() {
        return "".to_string();
    }

    match value.parse::<i64>() {
        Ok(parsed) => parsed.clamp(1, 10_000).to_string(),
        Err(_) => "".to_string(),
    }
}

fn normalize_optional_delay(raw: &str) -> String {
    let value = raw.trim();
    if value.is_empty() {
        return "".to_string();
    }

    match value.parse::<f64>() {
        Ok(parsed) if parsed >= 0.0 => {
            if (parsed.fract() - 0.0).abs() < f64::EPSILON {
                format!("{:.1}", parsed)
            } else {
                parsed.to_string()
            }
        }
        _ => "".to_string(),
    }
}

fn bool_to_setting(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "accepted"
    )
}

fn save_google_api_key_to_env(value: &str) -> Result<(), String> {
    let env_path = Path::new(".env");
    let existing = std::fs::read_to_string(env_path).unwrap_or_default();

    let mut lines = Vec::new();
    let mut replaced = false;

    for line in existing.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("GOOGLE_API_KEY=") {
            if !value.trim().is_empty() {
                lines.push(format!("GOOGLE_API_KEY={}", value.trim()));
            }
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !replaced && !value.trim().is_empty() {
        lines.push(format!("GOOGLE_API_KEY={}", value.trim()));
    }

    let mut output = lines.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }

    std::fs::write(env_path, output).map_err(|e| format!("Failed to update .env: {}", e))?;

    if value.trim().is_empty() {
        std::env::remove_var("GOOGLE_API_KEY");
    } else {
        std::env::set_var("GOOGLE_API_KEY", value.trim());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::LogGuard;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    // ─── Helper: create a settings table in an in-memory pool ──────────

    async fn setup_settings_table(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                description TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await
        .expect("settings table should be created");
    }

    /// Create an in-memory SqlitePool with the settings table and a Pool-based
    /// connection for direct queries.
    async fn make_pool_and_table() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool should connect");
        setup_settings_table(&pool).await;
        pool
    }

    // ─── AppState helpers (Portable / Installed) ───────────────────────

    fn make_app_paths_installed(tmp_dir: &std::path::Path) -> crate::paths::AppPaths {
        // Without a `data/` subdirectory under `tmp_dir`, resolve_paths_from_exe_dir
        // uses Installed mode.
        crate::paths::resolve_paths_from_exe_dir(tmp_dir)
    }

    fn make_app_paths_portable(tmp_dir: &std::path::Path) -> crate::paths::AppPaths {
        // With a `data/` subdirectory, Portable mode is activated.
        let data_dir = tmp_dir.join("data");
        std::fs::create_dir_all(&data_dir).expect("test data dir should be created");
        crate::paths::resolve_paths_from_exe_dir(tmp_dir)
    }

    fn make_app_state(pool: SqlitePool, paths: crate::paths::AppPaths) -> AppState {
        AppState {
            db: pool,
            paths,
            disclaimer_text: String::new(),
            log_guard: LogGuard::dummy_for_test(),
            shutdown_requested: AtomicBool::new(false),
        }
    }

    // ════════════════════════════════════════════════════════════════════
    // Category A — Pure helper functions
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn normalize_image_preference_whitelists_to_2d_or_3d() {
        assert_eq!(normalize_image_preference("3d"), "3d");
        assert_eq!(normalize_image_preference(" 3D "), "3d");
        assert_eq!(normalize_image_preference("2d"), "2d");
        assert_eq!(normalize_image_preference("unexpected"), "2d");
        assert_eq!(normalize_image_preference(""), "2d");
    }

    #[test]
    fn normalize_preview_3d_profile_whitelists_supported_profiles() {
        assert_eq!(normalize_preview_3d_profile("soft"), "soft");
        assert_eq!(normalize_preview_3d_profile("  SOFT "), "soft");
        assert_eq!(normalize_preview_3d_profile("balanced"), "balanced");
        assert_eq!(
            normalize_preview_3d_profile("high-contrast"),
            "high-contrast"
        );
        assert_eq!(
            normalize_preview_3d_profile("HIGH_CONTRAST"),
            "high-contrast"
        );
        assert_eq!(normalize_preview_3d_profile("other"), "balanced");
    }

    #[test]
    fn normalize_optional_batch_size_clamps_and_rejects_invalid() {
        assert_eq!(normalize_optional_batch_size(""), "");
        assert_eq!(normalize_optional_batch_size("abc"), "");
        assert_eq!(normalize_optional_batch_size("0"), "1");
        assert_eq!(normalize_optional_batch_size("1"), "1");
        assert_eq!(normalize_optional_batch_size("10001"), "10000");
        assert_eq!(normalize_optional_batch_size("  42  "), "42");
    }

    #[test]
    fn normalize_optional_delay_handles_blank_invalid_and_whole_numbers() {
        assert_eq!(normalize_optional_delay(""), "");
        assert_eq!(normalize_optional_delay("nope"), "");
        assert_eq!(normalize_optional_delay("-1"), "");
        assert_eq!(normalize_optional_delay("0"), "0.0");
        assert_eq!(normalize_optional_delay("6"), "6.0");
        assert_eq!(normalize_optional_delay("6.5"), "6.5");
        assert_eq!(normalize_optional_delay(" 2.25 "), "2.25");
    }

    #[test]
    fn truthy_parser_matches_expected_legacy_values() {
        assert!(is_truthy("1"));
        assert!(is_truthy("true"));
        assert!(is_truthy("YES"));
        assert!(is_truthy(" y "));
        assert!(is_truthy("accepted"));

        assert!(!is_truthy("0"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy(""));
        assert!(!is_truthy("maybe"));
    }

    #[test]
    fn bool_to_setting_returns_expected_strings() {
        assert_eq!(bool_to_setting(true), "true");
        assert_eq!(bool_to_setting(false), "false");
    }

    #[test]
    fn default_for_key_returns_correct_defaults() {
        assert_eq!(default_for_key(KEY_AI_TIER2_AUTO), "false");
        assert_eq!(default_for_key(KEY_AI_TIER3_AUTO), "false");
        assert_eq!(default_for_key(KEY_AI_BATCH_SIZE), "");
        assert_eq!(default_for_key(KEY_AI_DELAY), "");
        assert_eq!(default_for_key(KEY_IMPORT_COMMIT_BATCH_SIZE), "");
        assert_eq!(default_for_key(KEY_IMAGE_PREFERENCE), "2d");
        assert_eq!(default_for_key(KEY_PREVIEW_3D_PROFILE), "balanced");
        assert_eq!(default_for_key("unknown_key"), "");
    }

    #[test]
    fn description_for_key_returns_correct_descriptions() {
        assert!(description_for_key(KEY_AI_TIER2_AUTO).contains("Tier 2"));
        assert!(description_for_key(KEY_AI_TIER3_AUTO).contains("Tier 3"));
        assert!(description_for_key(KEY_AI_BATCH_SIZE).contains("designs"));
        assert!(description_for_key(KEY_AI_DELAY).contains("Gemini"));
        assert!(description_for_key(KEY_IMPORT_COMMIT_BATCH_SIZE).contains("commit"));
        assert!(description_for_key(KEY_IMPORT_LAST_BROWSE_FOLDER).contains("picker"));
        assert!(description_for_key(KEY_IMAGE_PREFERENCE).contains("preview"));
        assert!(description_for_key(KEY_PREVIEW_3D_PROFILE).contains("3D"));
        assert_eq!(description_for_key("unknown_key"), "");
    }

    // ════════════════════════════════════════════════════════════════════
    // Category B — Async DB helper functions
    // ════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn upsert_setting_inserts_new_row() {
        let pool = make_pool_and_table().await;
        let mut conn = pool.acquire().await.expect("connection should be acquired");

        upsert_setting(&mut conn, "test.key", "test_value")
            .await
            .expect("upsert should succeed");

        let row: (String, String) = sqlx::query_as(
            "SELECT value, description FROM settings WHERE key = ?",
        )
        .bind("test.key")
        .fetch_one(&mut *conn)
        .await
        .expect("row should exist");

        assert_eq!(row.0, "test_value");
        assert_eq!(row.1, description_for_key("test.key"));
    }

    #[tokio::test]
    async fn upsert_setting_updates_existing_row() {
        let pool = make_pool_and_table().await;
        let mut conn = pool.acquire().await.expect("connection should be acquired");

        upsert_setting(&mut conn, "test.key", "original")
            .await
            .expect("first upsert should succeed");
        upsert_setting(&mut conn, "test.key", "updated")
            .await
            .expect("second upsert should succeed");

        let (value, desc): (String, String) = sqlx::query_as(
            "SELECT value, description FROM settings WHERE key = ?",
        )
        .bind("test.key")
        .fetch_one(&mut *conn)
        .await
        .expect("row should exist");

        assert_eq!(value, "updated");
        // Description should remain the same (ON CONFLICT only updates value)
        assert_eq!(desc, description_for_key("test.key"));
    }

    #[tokio::test]
    async fn get_setting_with_default_returns_existing_value() {
        let pool = make_pool_and_table().await;
        let mut conn = pool.acquire().await.expect("connection should be acquired");

        upsert_setting(&mut conn, KEY_IMAGE_PREFERENCE, "3d")
            .await
            .expect("upsert should succeed");

        let result = get_setting_with_default(&mut conn, KEY_IMAGE_PREFERENCE)
            .await
            .expect("get should succeed");

        assert_eq!(result, "3d");
    }

    #[tokio::test]
    async fn get_setting_with_default_inserts_and_returns_fallback() {
        let pool = make_pool_and_table().await;
        let mut conn = pool.acquire().await.expect("connection should be acquired");

        let result = get_setting_with_default(&mut conn, KEY_IMAGE_PREFERENCE)
            .await
            .expect("get should succeed");

        assert_eq!(result, "2d"); // default_for_key(KEY_IMAGE_PREFERENCE)

        // Verify the fallback was persisted
        let (value,): (String,) = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(KEY_IMAGE_PREFERENCE)
            .fetch_one(&mut *conn)
            .await
            .expect("row should have been inserted");
        assert_eq!(value, "2d");
    }

    #[tokio::test]
    async fn get_setting_with_default_returns_empty_string_for_unknown_with_empty_default() {
        let pool = make_pool_and_table().await;
        let mut conn = pool.acquire().await.expect("connection should be acquired");

        let result = get_setting_with_default(&mut conn, KEY_AI_BATCH_SIZE)
            .await
            .expect("get should succeed");

        assert_eq!(result, ""); // default_for_key for KEY_AI_BATCH_SIZE is ""
    }

    // ════════════════════════════════════════════════════════════════════
    // Category C — Tauri command inner functions
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn browse_data_root_returns_expected_fallback_shapes() {
        let with_start = browse_settings_data_root(Some("D:/catalogue".to_string()));
        assert!(with_start.path.is_none());
        assert!(with_start
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("D:/catalogue"));

        let without_start = browse_settings_data_root(None);
        assert!(without_start.path.is_none());
        assert!(without_start
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("(blank)"));
    }

    #[tokio::test]
    async fn get_settings_view_model_inner_has_default_values_in_installed_mode() {
        // Use the env lock to isolate from parallel tests that set GOOGLE_API_KEY
        let _guard = env_lock().lock().unwrap();

        // Use a temp directory without a `data/` child → Installed mode.
        let tmp = std::env::temp_dir().join(format!(
            "settings-test-get-vm-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).ok();

        let pool = make_pool_and_table().await;
        let paths = make_app_paths_installed(&tmp);
        let state = make_app_state(pool, paths);

        let vm = get_settings_view_model_inner(&state)
            .await
            .expect("view model should be retrieved");

        assert_eq!(vm.image_preference, "2d");
        assert_eq!(vm.preview_3d_profile, "balanced");
        assert!(!vm.ai_tier2_auto);
        assert!(!vm.ai_tier3_auto);
        assert_eq!(vm.ai_batch_size, "");
        assert_eq!(vm.ai_delay, "");
        assert_eq!(vm.import_commit_batch_size, "");
        assert_eq!(vm.import_last_browse_folder, "");
        assert_eq!(vm.google_api_key, "");
        assert!(!vm.has_google_api_key);
        assert!(vm.can_configure_data_root); // Installed → true
        assert_eq!(vm.app_mode, "installed");
        assert_eq!(vm.ai_tagging_help_url, "#/help");

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn get_settings_view_model_inner_has_portable_mode_defaults() {
        // Use the env lock to isolate from parallel tests that set GOOGLE_API_KEY
        let _guard = env_lock().lock().unwrap();

        let tmp = std::env::temp_dir().join(format!(
            "settings-test-portable-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).ok();

        let pool = make_pool_and_table().await;
        let paths = make_app_paths_portable(&tmp);
        let state = make_app_state(pool, paths);

        let vm = get_settings_view_model_inner(&state)
            .await
            .expect("view model should be retrieved");

        assert!(!vm.can_configure_data_root); // Portable → false
        assert_eq!(vm.app_mode, "portable");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn get_settings_view_model_inner_reflects_custom_settings() {
        // Use the env lock to isolate from parallel tests that set GOOGLE_API_KEY
        let _guard = env_lock().lock().unwrap();

        let tmp = std::env::temp_dir().join(format!(
            "settings-test-custom-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).ok();

        let pool = make_pool_and_table().await;
        let paths = make_app_paths_installed(&tmp);
        let state = make_app_state(pool.clone(), paths);

        // Pre-load some custom settings
        let mut conn = pool.acquire().await.expect("connection");
        upsert_setting(&mut conn, KEY_IMAGE_PREFERENCE, "3d")
            .await
            .expect("upsert");
        upsert_setting(&mut conn, KEY_PREVIEW_3D_PROFILE, "soft")
            .await
            .expect("upsert");
        upsert_setting(&mut conn, KEY_AI_TIER2_AUTO, "true")
            .await
            .expect("upsert");
        upsert_setting(&mut conn, KEY_AI_BATCH_SIZE, "50")
            .await
            .expect("upsert");
        upsert_setting(&mut conn, KEY_AI_DELAY, "2.5")
            .await
            .expect("upsert");
        upsert_setting(&mut conn, KEY_IMPORT_LAST_BROWSE_FOLDER, "D:/imports")
            .await
            .expect("upsert");
        drop(conn);

        let vm = get_settings_view_model_inner(&state)
            .await
            .expect("view model should be retrieved");

        assert_eq!(vm.image_preference, "3d");
        assert_eq!(vm.preview_3d_profile, "soft");
        assert!(vm.ai_tier2_auto);
        assert_eq!(vm.ai_batch_size, "50");
        assert_eq!(vm.ai_delay, "2.5");
        assert_eq!(vm.import_last_browse_folder, "D:/imports");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn get_settings_view_model_inner_reads_google_api_key_from_env() {
        let _guard = env_lock().lock().unwrap();

        let tmp = std::env::temp_dir().join(format!(
            "settings-test-gapi-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).ok();

        let pool = make_pool_and_table().await;
        let paths = make_app_paths_installed(&tmp);
        let state = make_app_state(pool, paths);

        // Set the env var before calling
        std::env::set_var("GOOGLE_API_KEY", "my-test-key");

        let vm = get_settings_view_model_inner(&state)
            .await
            .expect("view model should be retrieved");
        assert_eq!(vm.google_api_key, "my-test-key");
        assert!(vm.has_google_api_key);

        // Clean up env
        std::env::remove_var("GOOGLE_API_KEY");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn save_import_last_browse_folder_inner_persists_and_trims() {
        let tmp = std::env::temp_dir().join(format!(
            "settings-test-save-last-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).ok();

        let pool = make_pool_and_table().await;
        let paths = make_app_paths_installed(&tmp);
        let state = make_app_state(pool.clone(), paths);

        let result = save_import_last_browse_folder_inner(&state, "  D:/my/folder  ".to_string())
            .await
            .expect("save should succeed");
        assert!(result.saved);
        assert_eq!(result.path, "D:/my/folder");

        // Verify in DB
        let mut conn = state.db.acquire().await.expect("connection");
        let setting = crate::settings::get_setting(&mut conn, KEY_IMPORT_LAST_BROWSE_FOLDER)
            .await
            .expect("get setting")
            .expect("setting should exist");
        assert_eq!(setting.value, "D:/my/folder");
        drop(conn);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn save_settings_view_model_inner_persists_all_fields() {
        let _guard = env_lock().lock().unwrap();

        let tmp = std::env::temp_dir().join(format!(
            "settings-test-save-all-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).ok();

        let pool = make_pool_and_table().await;
        let paths = make_app_paths_installed(&tmp);

        // Temporarily cd into tmp so save_google_api_key_to_env writes .env there
        let original_dir = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(&tmp).expect("switch to tmp");

        let state = make_app_state(pool, paths);

        let request = SaveSettingsRequest {
            image_preference: " 3D ".to_string(),
            preview_3d_profile: "HIGH_CONTRAST".to_string(),
            google_api_key: "env-key-abc".to_string(),
            ai_tier2_auto: true,
            ai_tier3_auto: false,
            ai_batch_size: "  25  ".to_string(),
            ai_delay: "  1.5  ".to_string(),
            import_commit_batch_size: "  100  ".to_string(),
            data_root: String::new(),
        };

        let result = save_settings_view_model_inner(&state, request)
            .await
            .expect("save should succeed");
        assert!(result.saved);
        assert_eq!(result.message, "Settings saved successfully.");

        // Verify settings in DB
        let mut conn = state.db.acquire().await.expect("connection");

        async fn read_setting(conn: &mut SqliteConnection, key: &str) -> String {
            crate::settings::get_setting(conn, key)
                .await
                .expect("get setting")
                .expect("setting should exist")
                .value
        }

        assert_eq!(
            read_setting(&mut conn, KEY_IMAGE_PREFERENCE).await,
            "3d"
        );
        assert_eq!(
            read_setting(&mut conn, KEY_PREVIEW_3D_PROFILE).await,
            "high-contrast"
        );
        assert_eq!(
            read_setting(&mut conn, KEY_AI_TIER2_AUTO).await,
            "true"
        );
        assert_eq!(
            read_setting(&mut conn, KEY_AI_TIER3_AUTO).await,
            "false"
        );
        assert_eq!(
            read_setting(&mut conn, KEY_AI_BATCH_SIZE).await,
            "25"
        );
        assert_eq!(
            read_setting(&mut conn, KEY_AI_DELAY).await,
            "1.5"
        );
        assert_eq!(
            read_setting(&mut conn, KEY_IMPORT_COMMIT_BATCH_SIZE).await,
            "100"
        );
        drop(conn);

        // Verify .env file was written
        let env_content = std::fs::read_to_string(tmp.join(".env")).unwrap_or_default();
        assert!(env_content.contains("GOOGLE_API_KEY=env-key-abc"));
        assert_eq!(
            std::env::var("GOOGLE_API_KEY").unwrap_or_default(),
            "env-key-abc"
        );

        // Cleanup
        std::env::set_current_dir(&original_dir).expect("restore dir");
        std::env::remove_var("GOOGLE_API_KEY");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ════════════════════════════════════════════════════════════════════
    // Category D — Type serialization round-trip tests
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn settings_view_model_serializes_all_fields() {
        let vm = SettingsViewModel {
            image_preference: "2d".to_string(),
            preview_3d_profile: "balanced".to_string(),
            google_api_key: "".to_string(),
            has_google_api_key: false,
            ai_tier2_auto: false,
            ai_tier3_auto: false,
            ai_batch_size: "".to_string(),
            ai_delay: "".to_string(),
            import_commit_batch_size: "".to_string(),
            import_last_browse_folder: "".to_string(),
            can_configure_data_root: true,
            data_root: "/data".to_string(),
            database_path: "/data/db.sqlite".to_string(),
            log_folder: "/data/logs".to_string(),
            app_mode: "installed".to_string(),
            ai_tagging_help_url: "#/help".to_string(),
        };
        let json = serde_json::to_value(&vm).expect("serialize");
        let map = json.as_object().expect("should be object");
        assert!(map.contains_key("image_preference"));
        assert!(map.contains_key("preview_3d_profile"));
        assert!(map.contains_key("google_api_key"));
        assert!(map.contains_key("has_google_api_key"));
        assert!(map.contains_key("ai_tier2_auto"));
        assert!(map.contains_key("ai_tier3_auto"));
        assert!(map.contains_key("ai_batch_size"));
        assert!(map.contains_key("ai_delay"));
        assert!(map.contains_key("import_commit_batch_size"));
        assert!(map.contains_key("import_last_browse_folder"));
        assert!(map.contains_key("can_configure_data_root"));
        assert!(map.contains_key("data_root"));
        assert!(map.contains_key("database_path"));
        assert!(map.contains_key("log_folder"));
        assert!(map.contains_key("app_mode"));
        assert!(map.contains_key("ai_tagging_help_url"));
        assert_eq!(map.len(), 16);
    }

    #[test]
    fn save_settings_request_deserializes_all_fields() {
        let json = serde_json::json!({
            "image_preference": "3d",
            "preview_3d_profile": "soft",
            "google_api_key": "xyz",
            "ai_tier2_auto": true,
            "ai_tier3_auto": false,
            "ai_batch_size": "10",
            "ai_delay": "1.0",
            "import_commit_batch_size": "5",
            "data_root": "/custom"
        });
        let req: SaveSettingsRequest =
            serde_json::from_value(json).expect("deserialize");
        assert_eq!(req.image_preference, "3d");
        assert_eq!(req.preview_3d_profile, "soft");
        assert_eq!(req.google_api_key, "xyz");
        assert!(req.ai_tier2_auto);
        assert!(!req.ai_tier3_auto);
        assert_eq!(req.ai_batch_size, "10");
        assert_eq!(req.ai_delay, "1.0");
        assert_eq!(req.import_commit_batch_size, "5");
        assert_eq!(req.data_root, "/custom");
    }

    #[test]
    fn save_settings_request_preview_3d_profile_defaults_to_empty() {
        let json = serde_json::json!({
            "image_preference": "2d",
            "google_api_key": "",
            "ai_tier2_auto": false,
            "ai_tier3_auto": false,
            "ai_batch_size": "",
            "ai_delay": "",
            "import_commit_batch_size": "",
            "data_root": ""
        });
        let req: SaveSettingsRequest =
            serde_json::from_value(json).expect("deserialize");
        assert_eq!(req.preview_3d_profile, ""); // #[serde(default)]
    }

    #[test]
    fn save_settings_result_serializes_correctly() {
        let result = SaveSettingsResult {
            saved: true,
            message: "Done.".to_string(),
        };
        let json = serde_json::to_value(&result).expect("serialize");
        let map = json.as_object().expect("should be object");
        assert!(map.contains_key("saved"));
        assert!(map.contains_key("message"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn save_import_browse_folder_result_serializes_correctly() {
        let result = SaveImportBrowseFolderResult {
            saved: true,
            path: "D:/test".to_string(),
        };
        let json = serde_json::to_value(&result).expect("serialize");
        let map = json.as_object().expect("should be object");
        assert!(map.contains_key("saved"));
        assert!(map.contains_key("path"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn browse_data_root_result_serializes_correctly() {
        let with_path = BrowseDataRootResult {
            path: Some("D:/data".to_string()),
            error: None,
        };
        let json = serde_json::to_value(&with_path).expect("serialize");
        let map = json.as_object().expect("should be object");
        assert!(map.contains_key("path"));
        assert!(map.contains_key("error"));
        assert_eq!(map.len(), 2);
        assert_eq!(map["path"], serde_json::json!("D:/data"));
        assert_eq!(map["error"], serde_json::json!(null));

        let none_path = BrowseDataRootResult {
            path: None,
            error: Some("failed".to_string()),
        };
        let json2 = serde_json::to_value(&none_path).expect("serialize");
        assert_eq!(json2["path"], serde_json::json!(null));
        assert_eq!(json2["error"], serde_json::json!("failed"));
    }

    // ════════════════════════════════════════════════════════════════════
    // Existing env-file test (preserved)
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn save_google_api_key_updates_and_clears_env_file() {
        let _guard = env_lock().lock().unwrap();

        let original_dir = std::env::current_dir().expect("current dir available");
        let test_dir = std::env::temp_dir().join(format!(
            "settings-route-tests-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time available")
                .as_nanos()
        ));

        std::fs::create_dir_all(&test_dir).expect("test dir should be created");
        std::env::set_current_dir(&test_dir).expect("should switch into test dir");

        let write_result = save_google_api_key_to_env("test-key-123");
        assert!(write_result.is_ok());
        let written = std::fs::read_to_string(test_dir.join(".env")).expect(".env should exist");
        assert!(written.contains("GOOGLE_API_KEY=test-key-123"));
        assert_eq!(
            std::env::var("GOOGLE_API_KEY").unwrap_or_default(),
            "test-key-123"
        );

        let clear_result = save_google_api_key_to_env("");
        assert!(clear_result.is_ok());
        let cleared =
            std::fs::read_to_string(test_dir.join(".env")).expect(".env should still exist");
        assert!(!cleared.contains("GOOGLE_API_KEY="));
        assert!(std::env::var("GOOGLE_API_KEY").is_err());

        std::env::set_current_dir(original_dir).expect("should restore original dir");
        let _ = std::fs::remove_dir_all(test_dir);
    }
}
