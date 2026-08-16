// Tests for the source module.
//
// This module was split out so the production file can stay focused
// on logic. It is included via a #[path] declaration in a
// #[cfg(test)] mod tests; module, retaining full access to the
// private items in the parent module through use super::*;.

use super::*;
use crate::utils::test_support::lock_env;
use std::fs;
use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////
// load_dotenv_from_str â€” pure parsing logic                                  //
////////////////////////////////////////////////////////////////////////////////

// â”€â”€â”€ load_dotenv_from_str â€” pure parsing logic â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn parse_empty_string_sets_no_vars() {
    // Act
    load_dotenv_from_str("");

    // Assert â€” we check that a well-known absent var is still absent
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

// â”€â”€â”€ load_dotenv_from_str â€” edge cases â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€â”€ load_dotenv â€” filesystem integration â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€â”€ AppStatus struct â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn app_status_from_paths_serializes_installed_mode_backslash_variant() {
    let paths = paths::AppPaths {
        mode: paths::ExecutionMode::Installed,
        data_root: PathBuf::from("E:/portable/data"),
        embroidery_designs_dir: PathBuf::from("E:/portable/data/MachineEmbroideryDesigns"),
        database_dir: PathBuf::from("E:/portable/data/Database"),
        database_path: PathBuf::from("E:/portable/data/Database/EmbroideryCatalogue.db"),
        thumbnail_cache_dir: PathBuf::from("E:/portable/data/thumbnails"),
        log_dir: PathBuf::from("E:/portable/data/logs"),
    };

    let status = app_status_from_paths(&paths);

    assert_eq!(status.execution_mode, "installed");
    assert_eq!(status.data_root, "E:/portable/data");
    assert_eq!(
        status.embroidery_dir,
        "E:/portable/data/MachineEmbroideryDesigns"
    );
    assert_eq!(
        status.database_path,
        "E:/portable/data/Database/EmbroideryCatalogue.db"
    );
}

#[test]
fn app_status_from_paths_serializes_dev_mode() {
    let paths = paths::AppPaths {
        mode: paths::ExecutionMode::Dev,
        data_root: PathBuf::from("D:/dev/rust-embroidery-catalogue/dev_data"),
        embroidery_designs_dir: PathBuf::from(
            "D:/dev/rust-embroidery-catalogue/dev_data/MachineEmbroideryDesigns",
        ),
        database_dir: PathBuf::from("D:/dev/rust-embroidery-catalogue/dev_data/Database"),
        database_path: PathBuf::from(
            "D:/dev/rust-embroidery-catalogue/dev_data/Database/EmbroideryCatalogue.db",
        ),
        thumbnail_cache_dir: PathBuf::from("D:/dev/rust-embroidery-catalogue/dev_data/thumbnails"),
        log_dir: PathBuf::from("D:/dev/rust-embroidery-catalogue/dev_data/logs"),
    };

    let status = app_status_from_paths(&paths);

    assert_eq!(status.execution_mode, "dev");
    assert_eq!(
        status.data_root,
        "D:/dev/rust-embroidery-catalogue/dev_data"
    );
    assert_eq!(
        status.embroidery_dir,
        "D:/dev/rust-embroidery-catalogue/dev_data/MachineEmbroideryDesigns"
    );
    assert_eq!(
        status.database_path,
        "D:/dev/rust-embroidery-catalogue/dev_data/Database/EmbroideryCatalogue.db"
    );
}

#[test]
fn app_status_from_paths_serializes_installed_mode() {
    let paths = paths::AppPaths {
        mode: paths::ExecutionMode::Installed,
        data_root: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue"),
        embroidery_designs_dir: PathBuf::from(
            "C:/Users/test/AppData/Roaming/EmbroideryCatalogue/MachineEmbroideryDesigns",
        ),
        database_dir: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue/Database"),
        database_path: PathBuf::from(
            "C:/Users/test/AppData/Roaming/EmbroideryCatalogue/Database/EmbroideryCatalogue.db",
        ),
        thumbnail_cache_dir: PathBuf::from(
            "C:/Users/test/AppData/Roaming/EmbroideryCatalogue/thumbnails",
        ),
        log_dir: PathBuf::from("C:/Users/test/AppData/Roaming/EmbroideryCatalogue/logs"),
    };

    let status = app_status_from_paths(&paths);

    assert_eq!(status.execution_mode, "installed");
    assert_eq!(
        status.data_root,
        "C:/Users/test/AppData/Roaming/EmbroideryCatalogue"
    );
}

#[test]
fn app_status_from_paths_handles_windows_backslash_paths() {
    // On Windows, to_string_lossy() on a PathBuf constructed from backslashes
    // yields backslashes. The frontend receives these raw values.
    let paths = paths::AppPaths {
        mode: paths::ExecutionMode::Installed,
        data_root: PathBuf::from("D:\\MyData"),
        embroidery_designs_dir: PathBuf::from("D:\\MyData\\MachineEmbroideryDesigns"),
        database_dir: PathBuf::from("D:\\MyData\\Database"),
        database_path: PathBuf::from("D:\\MyData\\Database\\EmbroideryCatalogue.db"),
        thumbnail_cache_dir: PathBuf::from("D:\\MyData\\thumbnails"),
        log_dir: PathBuf::from("D:\\MyData\\logs"),
    };

    let status = app_status_from_paths(&paths);

    // The path strings should contain the backslash separator as originally set.
    assert!(
        status.data_root.contains('\\'),
        "Windows paths should retain backslashes"
    );
    assert!(
        status.embroidery_dir.contains('\\'),
        "Windows paths should retain backslashes"
    );
    assert!(
        status.database_path.contains('\\'),
        "Windows paths should retain backslashes"
    );

    // The execution mode should still be installed.
    assert_eq!(status.execution_mode, "installed");
}

// â”€â”€â”€ AppStatus struct (serialization) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn app_status_serializes_correct_field_names() {
    let status = AppStatus {
        execution_mode: "installed".to_string(),
        data_root: "/some/data/root".to_string(),
        embroidery_dir: "/some/data/root/MachineEmbroideryDesigns".to_string(),
        database_path: "/some/data/root/Database/EmbroideryCatalogue.db".to_string(),
        data_root_missing: false,
    };

    let json = serde_json::to_value(&status).expect("serialize AppStatus");
    let map = json.as_object().expect("json should be an object");

    // The frontend expects these exact field names
    assert!(
        map.contains_key("execution_mode"),
        "missing 'execution_mode'"
    );
    assert!(map.contains_key("data_root"), "missing 'data_root'");
    assert!(
        map.contains_key("embroidery_dir"),
        "missing 'embroidery_dir'"
    );
    assert!(map.contains_key("database_path"), "missing 'database_path'");
    assert!(
        map.contains_key("data_root_missing"),
        "missing 'data_root_missing'"
    );

    // Exactly 5 fields â€” no extra, no missing
    assert_eq!(map.len(), 5, "AppStatus should serialize exactly 5 fields");
}

#[test]
fn app_status_serializes_correct_field_values() {
    let status = AppStatus {
        execution_mode: "installed".to_string(),
        data_root: "D:/data".to_string(),
        embroidery_dir: "D:/data/MachineEmbroideryDesigns".to_string(),
        database_path: "D:/data/Database/EmbroideryCatalogue.db".to_string(),
        data_root_missing: false,
    };

    let json = serde_json::to_value(&status).expect("serialize AppStatus");
    let map = json.as_object().expect("json should be an object");

    assert_eq!(
        map.get("execution_mode").and_then(|v| v.as_str()),
        Some("installed")
    );
    assert_eq!(
        map.get("data_root").and_then(|v| v.as_str()),
        Some("D:/data")
    );
    assert_eq!(
        map.get("embroidery_dir").and_then(|v| v.as_str()),
        Some("D:/data/MachineEmbroideryDesigns")
    );
    assert_eq!(
        map.get("database_path").and_then(|v| v.as_str()),
        Some("D:/data/Database/EmbroideryCatalogue.db")
    );
}

// â”€â”€â”€ read_idle_interval_from_db â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Create an in-memory pool with (optionally) a settings table.
async fn interval_test_pool(with_settings_table: bool) -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory pool");

    if with_settings_table {
        sqlx::query(
            "CREATE TABLE settings (
                    key VARCHAR(100) PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL
                )",
        )
        .execute(&pool)
        .await
        .expect("create settings table");
    }

    pool
}

#[tokio::test]
async fn read_idle_interval_defaults_when_no_setting_row() {
    let pool = interval_test_pool(true).await;

    let interval = read_idle_interval_from_db(&pool)
        .await
        .expect("missing row should fall back to default");

    assert_eq!(
        interval,
        services::db_health::DEFAULT_IDLE_CHECK_INTERVAL_SECS,
        "missing setting should fall back to the default idle interval"
    );
}

#[tokio::test]
async fn read_idle_interval_parses_valid_value() {
    let pool = interval_test_pool(true).await;
    sqlx::query("INSERT INTO settings (key, value) VALUES ('db.idle_check_interval_secs', '7200')")
        .execute(&pool)
        .await
        .expect("insert idle interval setting");

    let interval = read_idle_interval_from_db(&pool)
        .await
        .expect("valid numeric value should parse");

    assert_eq!(interval, 7200);
}

#[tokio::test]
async fn read_idle_interval_trims_whitespace_around_value() {
    let pool = interval_test_pool(true).await;
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('db.idle_check_interval_secs', '  3600  ')",
    )
    .execute(&pool)
    .await
    .expect("insert idle interval setting");

    let interval = read_idle_interval_from_db(&pool)
        .await
        .expect("whitespace-trimmed numeric value should parse");

    assert_eq!(interval, 3600);
}

#[tokio::test]
async fn read_idle_interval_returns_error_for_invalid_number() {
    let pool = interval_test_pool(true).await;
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('db.idle_check_interval_secs', 'not-a-number')",
    )
    .execute(&pool)
    .await
    .expect("insert idle interval setting");

    let result = read_idle_interval_from_db(&pool).await;
    assert!(result.is_err());
    assert!(
        result.unwrap_err().contains("Invalid idle interval"),
        "error should describe the parse failure"
    );
}

#[tokio::test]
async fn read_idle_interval_returns_error_when_settings_table_missing() {
    let pool = interval_test_pool(false).await;

    let result = read_idle_interval_from_db(&pool).await;
    assert!(
        result.is_err(),
        "missing settings table should surface an error"
    );
}

// â”€â”€â”€ configured data root (Installed-mode bootstrap config) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
//
// These exercise the `get_configured_data_root` / `set_configured_data_root`
// Tauri commands. They redirect APPDATA/HOME to a sandboxed temp dir (and
// hold the shared env lock) so the real user config is never touched.

/// Redirect APPDATA (or HOME on non-Windows) to a fresh temp dir, run `f`,
/// then restore the original env var and clean up.
fn with_sandboxed_app_data<F: FnOnce()>(f: F) {
    #[cfg(target_os = "windows")]
    let (var_name, original) = ("APPDATA", std::env::var("APPDATA").ok());
    #[cfg(not(target_os = "windows"))]
    let (var_name, original) = ("HOME", std::env::var("HOME").ok());

    let sandbox = std::env::temp_dir().join(format!(
        "embroidery-main-data-root-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::env::set_var(var_name, &sandbox);

    f();

    match original {
        Some(val) => std::env::set_var(var_name, val),
        None => std::env::remove_var(var_name),
    }
    let _ = fs::remove_dir_all(&sandbox);
}

#[test]
fn get_configured_data_root_returns_none_on_first_run() {
    let _guard = lock_env();
    with_sandboxed_app_data(|| {
        let result = get_configured_data_root().expect("read should succeed");
        assert_eq!(result, None);
    });
}

#[test]
fn set_then_get_configured_data_root_roundtrips() {
    let _guard = lock_env();
    with_sandboxed_app_data(|| {
        let chosen = "D:/EmbroideryCatalogue/Data".to_string();
        set_configured_data_root(chosen.clone()).expect("write should succeed");

        let read_back = get_configured_data_root()
            .expect("read should succeed")
            .expect("config should exist");
        assert_eq!(read_back, chosen);
    });
}

#[test]
fn set_configured_data_root_rejects_empty_string() {
    let _guard = lock_env();
    with_sandboxed_app_data(|| {
        let err = set_configured_data_root("   ".to_string()).expect_err("empty root should fail");
        assert!(err.contains("cannot be empty"));
    });
}

#[test]
fn set_configured_data_root_rejects_relative_path() {
    let _guard = lock_env();
    with_sandboxed_app_data(|| {
        let err = set_configured_data_root("relative/path".to_string())
            .expect_err("relative root should fail");
        assert!(err.contains("absolute"));
    });
}
