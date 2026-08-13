// Tests for the source module.
//
// This module was split out so the production file can stay focused
// on logic. It is included via a #[path] declaration in a
// #[cfg(test)] mod tests; module, retaining full access to the
// private items in the parent module through use super::*;.

use super::*;
use crate::logging::LogGuard;
use crate::utils::test_support::lock_env;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{SqliteConnection, SqlitePool};
use std::sync::atomic::AtomicBool;

// â”€â”€â”€ Helper: create a settings table in an in-memory pool â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€â”€ AppState helpers (Portable / Installed) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

fn make_app_paths_installed(tmp_dir: &std::path::Path) -> crate::paths::AppPaths {
    // Build a sandboxed Installed-mode layout directly so tests do not touch
    // the real platform app-data directory (or the project `dev_data` folder).
    let data_root = tmp_dir.join("installed_data");
    crate::paths::AppPaths {
        mode: crate::paths::ExecutionMode::Installed,
        data_root: data_root.clone(),
        embroidery_designs_dir: data_root.join("MachineEmbroideryDesigns"),
        database_dir: data_root.join("Database"),
        database_path: data_root.join("Database").join(crate::paths::DATABASE_FILENAME),
        thumbnail_cache_dir: data_root.join("thumbnails"),
        log_dir: data_root.join("logs"),
    }
}

fn make_app_paths_portable(tmp_dir: &std::path::Path) -> crate::paths::AppPaths {
    // Build a sandboxed Portable-mode layout with data next to the "exe".
    let data_root = tmp_dir.join("data");
    std::fs::create_dir_all(&data_root).expect("test data dir should be created");
    crate::paths::AppPaths {
        mode: crate::paths::ExecutionMode::Installed,
        data_root: data_root.clone(),
        embroidery_designs_dir: data_root.join("MachineEmbroideryDesigns"),
        database_dir: data_root.join("Database"),
        database_path: data_root.join("Database").join(crate::paths::DATABASE_FILENAME),
        thumbnail_cache_dir: data_root.join("thumbnails"),
        log_dir: data_root.join("logs"),
    }
}

fn make_app_state(pool: SqlitePool, paths: crate::paths::AppPaths) -> AppState {
    AppState {
        db: pool,
        paths,
        log_guard: LogGuard::dummy_for_test(),
        shutdown_requested: AtomicBool::new(false),
        maintenance_running: AtomicBool::new(false),
    }
}

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Category A â€” Pure helper functions
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

#[test]
fn normalize_preview_3d_profile_whitelists_supported_profiles() {
    assert_eq!(settings::normalize_preview_3d_profile("soft"), "soft");
    assert_eq!(settings::normalize_preview_3d_profile("  SOFT "), "soft");
    assert_eq!(
        settings::normalize_preview_3d_profile("balanced"),
        "balanced"
    );
    assert_eq!(
        settings::normalize_preview_3d_profile("high-contrast"),
        "high-contrast"
    );
    assert_eq!(
        settings::normalize_preview_3d_profile("HIGH_CONTRAST"),
        "high-contrast"
    );
    assert_eq!(settings::normalize_preview_3d_profile("other"), "balanced");
}

#[test]
fn normalize_optional_batch_size_clamps_and_rejects_invalid() {
    assert_eq!(settings::normalize_optional_batch_size(""), "");
    assert_eq!(settings::normalize_optional_batch_size("abc"), "");
    assert_eq!(settings::normalize_optional_batch_size("0"), "1");
    assert_eq!(settings::normalize_optional_batch_size("1"), "1");
    assert_eq!(settings::normalize_optional_batch_size("10001"), "10000");
    assert_eq!(settings::normalize_optional_batch_size("  42  "), "42");
}

#[test]
fn normalize_optional_delay_handles_blank_invalid_and_whole_numbers() {
    assert_eq!(settings::normalize_optional_delay(""), "");
    assert_eq!(settings::normalize_optional_delay("nope"), "");
    assert_eq!(settings::normalize_optional_delay("-1"), "");
    assert_eq!(settings::normalize_optional_delay("0"), "0.0");
    assert_eq!(settings::normalize_optional_delay("6"), "6.0");
    assert_eq!(settings::normalize_optional_delay("6.5"), "6.5");
    assert_eq!(settings::normalize_optional_delay(" 2.25 "), "2.25");
}

#[test]
fn truthy_parser_matches_expected_legacy_values() {
    assert!(settings::is_truthy("1"));
    assert!(settings::is_truthy("true"));
    assert!(settings::is_truthy("YES"));
    assert!(settings::is_truthy(" y "));
    assert!(settings::is_truthy("accepted"));

    assert!(!settings::is_truthy("0"));
    assert!(!settings::is_truthy("false"));
    assert!(!settings::is_truthy(""));
    assert!(!settings::is_truthy("maybe"));
}

#[test]
fn bool_to_setting_returns_expected_strings() {
    assert_eq!(settings::bool_to_setting(true), "true");
    assert_eq!(settings::bool_to_setting(false), "false");
}

#[test]
fn default_for_key_returns_correct_defaults() {
    assert_eq!(
        settings::default_for_key(settings::KEY_AI_TIER2_AUTO),
        "false"
    );
    assert_eq!(
        settings::default_for_key(settings::KEY_AI_TIER3_AUTO),
        "false"
    );
    assert_eq!(settings::default_for_key(settings::KEY_AI_BATCH_SIZE), "");
    assert_eq!(settings::default_for_key(settings::KEY_AI_DELAY), "");
    assert_eq!(
        settings::default_for_key(settings::KEY_IMPORT_COMMIT_BATCH_SIZE),
        ""
    );
    assert_eq!(
        settings::default_for_key(settings::KEY_PREVIEW_3D_PROFILE),
        "balanced"
    );
    assert_eq!(settings::default_for_key("unknown_key"), "");
}

#[test]
fn description_for_key_returns_correct_descriptions() {
    assert!(settings::description_for_key(settings::KEY_AI_TIER2_AUTO).contains("Tier 2"));
    assert!(settings::description_for_key(settings::KEY_AI_TIER3_AUTO).contains("Tier 3"));
    assert!(settings::description_for_key(settings::KEY_AI_BATCH_SIZE).contains("designs"));
    assert!(settings::description_for_key(settings::KEY_AI_DELAY).contains("Gemini"));
    assert!(
        settings::description_for_key(settings::KEY_IMPORT_COMMIT_BATCH_SIZE).contains("commit")
    );
    assert!(
        settings::description_for_key(settings::KEY_IMPORT_LAST_BROWSE_FOLDER).contains("picker")
    );
    assert!(settings::description_for_key(settings::KEY_PREVIEW_3D_PROFILE).contains("3D"));
    assert_eq!(settings::description_for_key("unknown_key"), "");
}

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Category B â€” Async DB helper functions
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

#[tokio::test]
async fn upsert_setting_inserts_new_row() {
    let pool = make_pool_and_table().await;
    let mut conn = pool.acquire().await.expect("connection should be acquired");

    settings::upsert_setting(&mut conn, "test.key", "test_value")
        .await
        .expect("upsert should succeed");

    let row: (String, String) =
        sqlx::query_as("SELECT value, description FROM settings WHERE key = ?")
            .bind("test.key")
            .fetch_one(&mut *conn)
            .await
            .expect("row should exist");

    assert_eq!(row.0, "test_value");
    assert_eq!(row.1, settings::description_for_key("test.key"));
}

#[tokio::test]
async fn upsert_setting_updates_existing_row() {
    let pool = make_pool_and_table().await;
    let mut conn = pool.acquire().await.expect("connection should be acquired");

    settings::upsert_setting(&mut conn, "test.key", "original")
        .await
        .expect("first upsert should succeed");
    settings::upsert_setting(&mut conn, "test.key", "updated")
        .await
        .expect("second upsert should succeed");

    let (value, desc): (String, String) =
        sqlx::query_as("SELECT value, description FROM settings WHERE key = ?")
            .bind("test.key")
            .fetch_one(&mut *conn)
            .await
            .expect("row should exist");

    assert_eq!(value, "updated");
    // Description should remain the same (ON CONFLICT only updates value)
    assert_eq!(desc, settings::description_for_key("test.key"));
}

#[tokio::test]
async fn get_setting_with_default_returns_existing_value() {
    let pool = make_pool_and_table().await;
    let mut conn = pool.acquire().await.expect("connection should be acquired");

    settings::upsert_setting(&mut conn, "test.key", "custom_value")
        .await
        .expect("upsert should succeed");

    let result = settings::get_setting_with_default(&mut conn, "test.key")
        .await
        .expect("get should succeed");

    assert_eq!(result, "custom_value");
}

#[tokio::test]
async fn get_setting_with_default_inserts_and_returns_fallback() {
    let pool = make_pool_and_table().await;
    let mut conn = pool.acquire().await.expect("connection should be acquired");

    let result = settings::get_setting_with_default(&mut conn, settings::KEY_PREVIEW_3D_PROFILE)
        .await
        .expect("get should succeed");

    assert_eq!(result, "balanced"); // default_for_key(KEY_PREVIEW_3D_PROFILE)

    // Verify the fallback was persisted
    let (value,): (String,) = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(settings::KEY_PREVIEW_3D_PROFILE)
        .fetch_one(&mut *conn)
        .await
        .expect("row should have been inserted");
    assert_eq!(value, "balanced");
}

#[tokio::test]
async fn get_setting_with_default_returns_empty_string_for_unknown_with_empty_default() {
    let pool = make_pool_and_table().await;
    let mut conn = pool.acquire().await.expect("connection should be acquired");

    let result = settings::get_setting_with_default(&mut conn, settings::KEY_AI_BATCH_SIZE)
        .await
        .expect("get should succeed");

    assert_eq!(result, ""); // default_for_key for KEY_AI_BATCH_SIZE is ""
}

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Category C â€” Tauri command inner functions
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

#[test]
fn browse_data_root_returns_expected_fallback_shapes() {
    let with_start = settings::browse_settings_data_root(Some("D:/catalogue".to_string()));
    assert!(with_start.path.is_none());
    assert!(
        with_start
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("D:/catalogue")
    );

    let without_start = settings::browse_settings_data_root(None);
    assert!(without_start.path.is_none());
    assert!(
        without_start
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("(blank)")
    );
}

#[tokio::test]
async fn get_settings_view_model_inner_has_default_values_in_installed_mode() {
    let _guard = lock_env();

    // Isolate GOOGLE_API_KEY so ambient process state (.env / real env)
    // cannot leak into this default-value assertion.
    let prev_google_api_key = std::env::var("GOOGLE_API_KEY").ok();
    std::env::remove_var("GOOGLE_API_KEY");

    // Helper builds a sandboxed Installed-mode AppPaths directly.
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

    assert_eq!(vm.preview_3d_profile, "balanced");
    assert!(!vm.ai_tier2_auto);
    assert!(!vm.ai_tier3_auto);
    assert_eq!(vm.ai_batch_size, "");
    assert_eq!(vm.ai_delay, "");
    assert_eq!(vm.import_commit_batch_size, "");
    assert_eq!(vm.import_last_browse_folder, "");
    assert_eq!(vm.google_api_key, "");
    assert!(!vm.has_google_api_key);
    assert!(vm.can_configure_data_root); // Installed â†’ true
    assert_eq!(vm.app_mode, "installed");
    assert_eq!(vm.ai_tagging_help_url, "#/help");

    // Restore the ambient GOOGLE_API_KEY (if any) for other tests.
    if let Some(google_api_key) = prev_google_api_key {
        std::env::set_var("GOOGLE_API_KEY", google_api_key);
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn get_settings_view_model_inner_has_installed_mode_defaults() {
    let _guard = lock_env();

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

    assert!(vm.can_configure_data_root); // Installed â†’ true
    assert_eq!(vm.app_mode, "installed");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn get_settings_view_model_inner_reflects_custom_settings() {
    let _guard = lock_env();

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
    settings::upsert_setting(&mut conn, settings::KEY_PREVIEW_3D_PROFILE, "soft")
        .await
        .expect("upsert");
    settings::upsert_setting(&mut conn, settings::KEY_AI_TIER2_AUTO, "true")
        .await
        .expect("upsert");
    settings::upsert_setting(&mut conn, settings::KEY_AI_BATCH_SIZE, "50")
        .await
        .expect("upsert");
    settings::upsert_setting(&mut conn, settings::KEY_AI_DELAY, "2.5")
        .await
        .expect("upsert");
    settings::upsert_setting(
        &mut conn,
        settings::KEY_IMPORT_LAST_BROWSE_FOLDER,
        "D:/imports",
    )
    .await
    .expect("upsert");
    drop(conn);

    let vm = get_settings_view_model_inner(&state)
        .await
        .expect("view model should be retrieved");

    assert_eq!(vm.preview_3d_profile, "soft");
    assert!(vm.ai_tier2_auto);
    assert_eq!(vm.ai_batch_size, "50");
    assert_eq!(vm.ai_delay, "2.5");
    assert_eq!(vm.import_last_browse_folder, "D:/imports");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn get_settings_view_model_inner_reads_google_api_key_from_env() {
    let _guard = lock_env();

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
    let setting = crate::settings::get_setting(&mut conn, settings::KEY_IMPORT_LAST_BROWSE_FOLDER)
        .await
        .expect("get setting")
        .expect("setting should exist");
    assert_eq!(setting.value, "D:/my/folder");
    drop(conn);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn save_settings_view_model_inner_persists_all_fields() {
    let _guard = lock_env();

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
        preview_3d_profile: "HIGH_CONTRAST".to_string(),
        google_api_key: "env-key-abc".to_string(),
        ai_tier2_auto: true,
        ai_tier3_auto: false,
        ai_batch_size: "  25  ".to_string(),
        ai_delay: "  1.5  ".to_string(),
        import_commit_batch_size: "  100  ".to_string(),
        data_root: String::new(),
        db_idle_check_interval_secs: "1800".to_string(),
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
        read_setting(&mut conn, settings::KEY_PREVIEW_3D_PROFILE).await,
        "high-contrast"
    );
    assert_eq!(
        read_setting(&mut conn, settings::KEY_AI_TIER2_AUTO).await,
        "true"
    );
    assert_eq!(
        read_setting(&mut conn, settings::KEY_AI_TIER3_AUTO).await,
        "false"
    );
    assert_eq!(
        read_setting(&mut conn, settings::KEY_AI_BATCH_SIZE).await,
        "25"
    );
    assert_eq!(read_setting(&mut conn, settings::KEY_AI_DELAY).await, "1.5");
    assert_eq!(
        read_setting(&mut conn, settings::KEY_IMPORT_COMMIT_BATCH_SIZE).await,
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

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Category D â€” Type serialization round-trip tests
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

#[test]
fn settings_view_model_serializes_all_fields() {
    let vm = SettingsViewModel {
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
        db_idle_check_interval_secs: "1800".to_string(),
    };
    let json = serde_json::to_value(&vm).expect("serialize");
    let map = json.as_object().expect("should be object");
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
    assert!(map.contains_key("db_idle_check_interval_secs"));
    assert_eq!(map.len(), 16);
}

#[test]
fn save_settings_request_deserializes_all_fields() {
    let json = serde_json::json!({
        "preview_3d_profile": "soft",
        "google_api_key": "xyz",
        "ai_tier2_auto": true,
        "ai_tier3_auto": false,
        "ai_batch_size": "10",
        "ai_delay": "1.0",
        "import_commit_batch_size": "5",
        "data_root": "/custom"
    });
    let req: SaveSettingsRequest = serde_json::from_value(json).expect("deserialize");
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
        "google_api_key": "",
        "ai_tier2_auto": false,
        "ai_tier3_auto": false,
        "ai_batch_size": "",
        "ai_delay": "",
        "import_commit_batch_size": "",
        "data_root": ""
    });
    let req: SaveSettingsRequest = serde_json::from_value(json).expect("deserialize");
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

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Existing env-file test (preserved)
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

#[test]
fn save_google_api_key_updates_and_clears_env_file() {
    let _guard = lock_env();

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

    let write_result = settings::save_google_api_key_to_env("test-key-123");
    assert!(write_result.is_ok());
    let written = std::fs::read_to_string(test_dir.join(".env")).expect(".env should exist");
    assert!(written.contains("GOOGLE_API_KEY=test-key-123"));
    assert_eq!(
        std::env::var("GOOGLE_API_KEY").unwrap_or_default(),
        "test-key-123"
    );

    let clear_result = settings::save_google_api_key_to_env("");
    assert!(clear_result.is_ok());
    let cleared = std::fs::read_to_string(test_dir.join(".env")).expect(".env should still exist");
    assert!(!cleared.contains("GOOGLE_API_KEY="));
    assert!(std::env::var("GOOGLE_API_KEY").is_err());

    std::env::set_current_dir(original_dir).expect("should restore original dir");
    let _ = std::fs::remove_dir_all(test_dir);
}

// ---------------------------------------------------------------------------
// New setup-wizard API key route tests
// ---------------------------------------------------------------------------

#[test]
fn get_google_api_key_returns_none_when_unset() {
    let _guard = lock_env();
    let prev = std::env::var("GOOGLE_API_KEY").ok();
    std::env::remove_var("GOOGLE_API_KEY");

    let result = get_google_api_key();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);

    if let Some(key) = prev {
        std::env::set_var("GOOGLE_API_KEY", key);
    }
}

#[test]
fn get_google_api_key_returns_value_when_set() {
    let _guard = lock_env();
    let prev = std::env::var("GOOGLE_API_KEY").ok();
    std::env::set_var("GOOGLE_API_KEY", "  my-test-key  ");

    let result = get_google_api_key();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("my-test-key".to_string()));

    if let Some(key) = prev {
        std::env::set_var("GOOGLE_API_KEY", key);
    } else {
        std::env::remove_var("GOOGLE_API_KEY");
    }
}

#[test]
fn set_google_api_key_persists_env_and_returns_true() {
    let _guard = lock_env();

    let original_dir = std::env::current_dir().expect("current dir available");
    let test_dir = std::env::temp_dir().join(format!(
        "settings-route-set-key-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time available")
            .as_nanos()
    ));
    std::fs::create_dir_all(&test_dir).expect("test dir should be created");
    std::env::set_current_dir(&test_dir).expect("should switch into test dir");

    let result = set_google_api_key("route-key-456".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let written = std::fs::read_to_string(test_dir.join(".env")).expect(".env should exist");
    assert!(written.contains("GOOGLE_API_KEY=route-key-456"));
    assert_eq!(
        std::env::var("GOOGLE_API_KEY").unwrap_or_default(),
        "route-key-456"
    );

    std::env::set_current_dir(original_dir).expect("should restore original dir");
    std::env::remove_var("GOOGLE_API_KEY");
    let _ = std::fs::remove_dir_all(test_dir);
}
