// Tests for the paths module.
//
// This module was split out of paths.rs so the source file can stay
// focused on production logic. It is included via a #[path] declaration
// in a #[cfg(test)] mod tests; module, so it retains full access to the
// private items in the parent module through use super::*;.

use super::*;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Create a temporary directory with a name derived from the test function.
fn tmp_dir(test_name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "paths-test-{}-{}",
        test_name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

// ---------------------------------------------------------------------------
// DATABASE_FILENAME
// ---------------------------------------------------------------------------

#[test]
fn database_filename_is_correct() {
    assert_eq!(DATABASE_FILENAME, "EmbroideryCatalogue.db");
}

// ---------------------------------------------------------------------------
// ExecutionMode derives
// ---------------------------------------------------------------------------

#[test]
fn execution_mode_debug_fmt() {
    assert_eq!(format!("{:?}", ExecutionMode::Dev), "Dev");
    assert_eq!(format!("{:?}", ExecutionMode::Installed), "Installed");
}

#[test]
fn execution_mode_clone() {
    let a = ExecutionMode::Dev;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn execution_mode_partial_eq() {
    assert_eq!(ExecutionMode::Dev, ExecutionMode::Dev);
    assert_eq!(ExecutionMode::Installed, ExecutionMode::Installed);
    assert_ne!(ExecutionMode::Dev, ExecutionMode::Installed);
}

#[test]
fn execution_mode_serialize() {
    let dev = serde_json::to_value(ExecutionMode::Dev).unwrap();
    assert_eq!(dev, serde_json::json!("Dev"));

    let installed = serde_json::to_value(ExecutionMode::Installed).unwrap();
    assert_eq!(installed, serde_json::json!("Installed"));
}

// ---------------------------------------------------------------------------
// resolve_paths_from_exe_dir
// ---------------------------------------------------------------------------

/// In debug builds, Dev mode is selected and data lives in
/// `<project>/dev_data/`.
#[cfg(debug_assertions)]
#[test]
fn dev_mode_in_debug_builds() {
    let tmp = tmp_dir("dev_mode_in_debug_builds");
    fs::create_dir_all(tmp.join("exe")).expect("create exe");

    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    assert_eq!(app_paths.mode, ExecutionMode::Dev);
    assert_eq!(app_paths.data_root, dev_data_root());
    // The project dev_data directory was created and seeded.
    assert!(app_paths.data_root.exists());
    assert!(app_paths.database_dir.exists());
    assert_eq!(
        app_paths.database_path,
        dev_data_root()
            .join("Database")
            .join(DATABASE_FILENAME)
    );
}

/// `dev_data_root()` resolves to `<CARGO_MANIFEST_DIR>/dev_data`.
#[test]
fn dev_data_root_resolves_to_project_dev_data() {
    let root = dev_data_root();
    assert_eq!(
        root,
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dev_data")
    );
    assert!(root.to_string_lossy().ends_with("dev_data"));
}

/// In release builds, Installed mode is used with the platform app-data root.
/// APPDATA/HOME is redirected to a temp dir so the seeding step stays within
/// the test sandbox.
#[cfg(not(debug_assertions))]
#[test]
#[serial]
fn installed_mode_when_release_build() {
    let tmp = tmp_dir("installed_mode_when_release_build");
    fs::create_dir_all(tmp.join("exe")).expect("create exe");

    #[cfg(target_os = "windows")]
    {
        let original = std::env::var("APPDATA").ok();
        std::env::set_var("APPDATA", tmp.join("fake_appdata"));
        let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));
        assert_eq!(app_paths.mode, ExecutionMode::Installed);
        assert!(app_paths.data_root.starts_with(&tmp.join("fake_appdata")));
        match original {
            Some(val) => std::env::set_var("APPDATA", val),
            None => std::env::remove_var("APPDATA"),
        }
    }
    #[cfg(target_os = "macos")]
    {
        let original = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.join("fake_home"));
        let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));
        assert_eq!(app_paths.mode, ExecutionMode::Installed);
        assert!(app_paths.data_root.starts_with(&tmp.join("fake_home")));
        match original {
            Some(val) => std::env::set_var("HOME", val),
            None => std::env::remove_var("HOME"),
        }
    }
    #[cfg(target_os = "linux")]
    {
        let original = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.join("fake_home"));
        let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));
        assert_eq!(app_paths.mode, ExecutionMode::Installed);
        assert!(app_paths.data_root.starts_with(&tmp.join("fake_home")));
        match original {
            Some(val) => std::env::set_var("HOME", val),
            None => std::env::remove_var("HOME"),
        }
    }

    let _ = fs::remove_dir_all(&tmp);
}

/// A non-existent exe dir should not panic. Detection follows the build
/// profile: debug -> Dev, release -> Installed.
#[test]
fn empty_exe_dir_does_not_panic() {
    let tmp = tmp_dir("empty_exe_dir");
    let nonexistent = tmp.join("nonexistent_exe_dir");

    // This should not panic regardless of mode.
    let app_paths = resolve_paths_from_exe_dir(&nonexistent);

    #[cfg(debug_assertions)]
    assert_eq!(
        app_paths.mode,
        ExecutionMode::Dev,
        "empty exe dir in debug should resolve to Dev"
    );
    #[cfg(not(debug_assertions))]
    assert_eq!(
        app_paths.mode,
        ExecutionMode::Installed,
        "empty exe dir in release should resolve to Installed"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn resolve_paths_creates_all_directories() {
    let tmp = tmp_dir("resolve_paths_creates_all_dirs");
    fs::create_dir_all(tmp.join("exe")).expect("create exe");

    // Resolve (Dev in debug, Installed in release) then verify all dirs exist.
    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    // Verify all directories exist
    assert!(app_paths.data_root.exists());
    assert!(app_paths.embroidery_designs_dir.exists());
    assert!(app_paths.database_dir.exists());
    assert!(app_paths.thumbnail_cache_dir.exists());
    assert!(app_paths.log_dir.exists());
}

// ---------------------------------------------------------------------------
// platform_data_root
// ---------------------------------------------------------------------------

#[test]
fn platform_data_root_has_company_subdirectory() {
    let root = platform_data_root();

    // The root should always end with "EmbroideryCatalogue"
    assert!(
        root.to_string_lossy().contains("EmbroideryCatalogue"),
        "Expected platform_data_root to contain 'EmbroideryCatalogue', got: {:?}",
        root
    );
}

#[cfg(target_os = "windows")]
#[test]
#[serial]
fn windows_platform_data_root_uses_appdata() {
    // Temporarily set APPDATA to a known path
    let original = std::env::var("APPDATA").ok();
    std::env::set_var("APPDATA", "C:\\TestAppData");

    let root = platform_data_root();
    assert_eq!(root, PathBuf::from("C:\\TestAppData\\EmbroideryCatalogue"));

    // Restore original
    match original {
        Some(val) => std::env::set_var("APPDATA", val),
        None => std::env::remove_var("APPDATA"),
    }
}

#[cfg(target_os = "windows")]
#[test]
#[serial]
fn windows_platform_data_root_fallback_when_no_appdata() {
    let original = std::env::var("APPDATA").ok();
    std::env::remove_var("APPDATA");

    let root = platform_data_root();

    // Fallback joins "EmbroideryCatalogue" under the base ("." when APPDATA
    // is missing), so the last segment is always "EmbroideryCatalogue".
    assert!(
        root.to_string_lossy().ends_with("EmbroideryCatalogue"),
        "expected fallback to end with 'EmbroideryCatalogue', got {}",
        root.display()
    );

    // Restore original
    if let Some(val) = original {
        std::env::set_var("APPDATA", val);
    }
}

#[cfg(target_os = "macos")]
#[test]
#[serial]
fn macos_platform_data_root_uses_home() {
    let original = std::env::var("HOME").ok();
    std::env::set_var("HOME", "/Users/testuser");

    let root = platform_data_root();
    assert_eq!(
        root,
        PathBuf::from("/Users/testuser/Library/Application Support/EmbroideryCatalogue")
    );

    match original {
        Some(val) => std::env::set_var("HOME", val),
        None => std::env::remove_var("HOME"),
    }
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn linux_platform_data_root_uses_home() {
    let original = std::env::var("HOME").ok();
    std::env::set_var("HOME", "/home/testuser");

    let root = platform_data_root();
    assert_eq!(
        root,
        PathBuf::from("/home/testuser/.local/share/EmbroideryCatalogue")
    );

    match original {
        Some(val) => std::env::set_var("HOME", val),
        None => std::env::remove_var("HOME"),
    }
}

#[cfg(target_os = "macos")]
#[test]
#[serial]
fn macos_platform_data_root_fallback_when_no_home() {
    let original = std::env::var("HOME").ok();
    std::env::remove_var("HOME");

    let root = platform_data_root();
    assert!(
        root.to_string_lossy().ends_with("EmbroideryCatalogue"),
        "expected fallback to end with 'EmbroideryCatalogue', got {}",
        root.display()
    );

    if let Some(val) = original {
        std::env::set_var("HOME", val);
    }
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn linux_platform_data_root_fallback_when_no_home() {
    let original = std::env::var("HOME").ok();
    std::env::remove_var("HOME");

    let root = platform_data_root();
    assert!(
        root.to_string_lossy().ends_with("EmbroideryCatalogue"),
        "expected fallback to end with 'EmbroideryCatalogue', got {}",
        root.display()
    );

    if let Some(val) = original {
        std::env::set_var("HOME", val);
    }
}

// ---------------------------------------------------------------------------
// to_absolute
// ---------------------------------------------------------------------------

#[test]
fn to_absolute_joins_relative_with_root() {
    let root = PathBuf::from("/some/root");
    let relative = PathBuf::from("subdir/file.txt");

    assert_eq!(
        to_absolute(&relative, &root),
        PathBuf::from("/some/root/subdir/file.txt")
    );
}

#[test]
fn to_absolute_with_empty_relative() {
    let root = PathBuf::from("/some/root");
    assert_eq!(to_absolute(Path::new(""), &root), root);
}

#[test]
fn to_absolute_with_absolute_relative() {
    // If the "relative" path is actually absolute, join still works
    // (root is prepended to an absolute path, which on some platforms yields the absolute path)
    let root = PathBuf::from("/some/root");
    let relative = PathBuf::from("/absolute/path");
    let result = to_absolute(&relative, &root);
    // On Windows this may differ, but on Unix: root.join(absolute) -> absolute
    assert!(
        result == PathBuf::from("/some/root/absolute/path")
            || result == PathBuf::from("/absolute/path")
    );
}

// ---------------------------------------------------------------------------
// to_relative
// ---------------------------------------------------------------------------

#[test]
fn to_relative_returns_ok_for_path_under_root() {
    // Create real directories AND the file so canonicalize works on both
    let tmp = tmp_dir("to_relative_under_root");
    fs::create_dir_all(tmp.join("root").join("sub")).expect("create dirs");
    fs::create_dir_all(tmp.join("root").join("other")).expect("create other");

    let absolute = tmp.join("root").join("sub").join("file.txt");
    let root = tmp.join("root");

    // Touch the file so canonicalize succeeds on the absolute path
    fs::write(&absolute, b"").expect("write test file");

    let result = to_relative(&absolute, &root);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), PathBuf::from("sub").join("file.txt"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn to_relative_returns_err_for_path_not_under_root() {
    let tmp = tmp_dir("to_relative_not_under_root");
    fs::create_dir_all(tmp.join("a")).expect("create dirs");
    fs::create_dir_all(tmp.join("b")).expect("create other");

    let absolute = tmp.join("a").join("file.txt");
    let root = tmp.join("b");

    let result = to_relative(&absolute, &root);
    assert!(result.is_err(), "Expected Err, got {:?}", result);

    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn to_relative_with_nonexistent_path_falls_back_to_string_matching() {
    // Use wholly non-existent paths so canonicalize() fails for BOTH the
    // absolute and root paths, forcing the raw string-level fallback.
    // (On Windows an existing directory canonicalizes with a \\?\ prefix,
    //  while a non-existent file path does not - causing a mismatch.)
    #[cfg(target_os = "windows")]
    let root = PathBuf::from("C:/nonexistent-root");
    #[cfg(not(target_os = "windows"))]
    let root = PathBuf::from("/tmp/nonexistent-root");

    let absolute = root.join("sub/file.txt");

    let result = to_relative(&absolute, &root);

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), PathBuf::from("sub/file.txt"));
}

#[test]
fn to_relative_root_itself_returns_empty() {
    let tmp = tmp_dir("to_relative_root_itself");
    fs::create_dir_all(tmp.join("root")).expect("create root");

    let root = tmp.join("root");

    let result = to_relative(&root, &root);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), PathBuf::from(""));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn to_relative_with_different_drive_letters_on_windows() {
    // On non-Windows this just tests the string-level fallback
    let absolute = PathBuf::from("D:/data/designs/test.dst");
    let root = PathBuf::from("C:/data");

    let result = to_relative(&absolute, &root);
    // Should fail because "D:/data/..." doesn't start with "C:/data"
    assert!(
        result.is_err(),
        "Expected Err for different roots, got {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// resolve_app_paths (integration via current_exe)
// ---------------------------------------------------------------------------
//
// These tests call the real resolve_app_paths() which uses std::env::current_exe().
// During `cargo test` the binary lives in target/debug/deps/<hash>/<exe>.
// In a debug build it resolves to Dev mode and writes to `<project>/dev_data/`.
// That validates the mechanism without polluting any user-visible location.

#[test]
fn resolve_app_paths_does_not_panic_and_smoke_checks() {
    let app_paths = resolve_app_paths().unwrap();

    // Resolved to some valid mode
    assert!(
        app_paths.mode == ExecutionMode::Dev || app_paths.mode == ExecutionMode::Installed,
        "Expected Dev or Installed, got {:?}",
        app_paths.mode
    );

    // All path fields are populated (non-empty)
    assert!(
        !app_paths.data_root.as_os_str().is_empty(),
        "data_root must not be empty"
    );
    assert!(
        !app_paths.embroidery_designs_dir.as_os_str().is_empty(),
        "embroidery_designs_dir must not be empty"
    );
    assert!(
        !app_paths.database_dir.as_os_str().is_empty(),
        "database_dir must not be empty"
    );
    assert!(
        !app_paths.database_path.as_os_str().is_empty(),
        "database_path must not be empty"
    );
    assert!(
        !app_paths.thumbnail_cache_dir.as_os_str().is_empty(),
        "thumbnail_cache_dir must not be empty"
    );
    assert!(
        !app_paths.log_dir.as_os_str().is_empty(),
        "log_dir must not be empty"
    );

    // The database path includes the DATABASE_FILENAME
    assert!(
        app_paths
            .database_path
            .to_string_lossy()
            .ends_with(DATABASE_FILENAME),
        "database_path must end with '{}', got '{}'",
        DATABASE_FILENAME,
        app_paths.database_path.display()
    );

    // The data_root directory was created
    assert!(
        app_paths.data_root.exists(),
        "data_root '{}' was not created",
        app_paths.data_root.display()
    );

    // Subdirectories under data_root were created
    assert!(
        app_paths.embroidery_designs_dir.exists(),
        "embroidery_designs_dir '{}' was not created",
        app_paths.embroidery_designs_dir.display()
    );
    assert!(
        app_paths.database_dir.exists(),
        "database_dir '{}' was not created",
        app_paths.database_dir.display()
    );
    assert!(
        app_paths.thumbnail_cache_dir.exists(),
        "thumbnail_cache_dir '{}' was not created",
        app_paths.thumbnail_cache_dir.display()
    );
    assert!(
        app_paths.log_dir.exists(),
        "log_dir '{}' was not created",
        app_paths.log_dir.display()
    );
}

#[test]
fn resolve_app_paths_is_consistent() {
    let a = resolve_app_paths().unwrap();
    let b = resolve_app_paths().unwrap();

    assert_eq!(a.mode, b.mode, "ExecutionMode differs between calls");
    assert_eq!(a.data_root, b.data_root, "data_root differs between calls");
    assert_eq!(
        a.embroidery_designs_dir, b.embroidery_designs_dir,
        "embroidery_designs_dir differs between calls"
    );
    assert_eq!(
        a.database_dir, b.database_dir,
        "database_dir differs between calls"
    );
    assert_eq!(
        a.database_path, b.database_path,
        "database_path differs between calls"
    );
    assert_eq!(
        a.thumbnail_cache_dir, b.thumbnail_cache_dir,
        "thumbnail_cache_dir differs between calls"
    );
    assert_eq!(a.log_dir, b.log_dir, "log_dir differs between calls");
}

// ---------------------------------------------------------------------------
// round-trip
// ---------------------------------------------------------------------------

#[test]
fn relative_absolute_roundtrip_with_real_dirs() {
    let tmp = tmp_dir("roundtrip_real");
    fs::create_dir_all(tmp.join("root").join("deep").join("deeper")).expect("create dirs");

    let root = tmp.join("root");
    let original_relative = PathBuf::from("deep/deeper/file.dst");
    let absolute = root.join(&original_relative);

    // Touch the file so canonicalize works
    fs::write(&absolute, b"test data").expect("write test file");

    // Round-trip: relative -> absolute -> relative
    let reconstructed_absolute = to_absolute(&original_relative, &root);
    assert_eq!(reconstructed_absolute, absolute);

    let reconstructed_relative = to_relative(&absolute, &root).unwrap();
    assert_eq!(reconstructed_relative, original_relative);

    let _ = fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// bootstrap config (Installed mode)
// ---------------------------------------------------------------------------
//
// These tests manipulate the platform app-data env var (APPDATA on Windows,
// HOME on mac/linux) so the bootstrap config read from / written to a
// sandboxed temp location without touching the real user's config.

/// Redirect the platform app-data env var to a temp dir and return the
/// previous value(s) so tests can restore them.
fn with_sandboxed_app_data<F: FnOnce()>(f: F) {
    #[cfg(target_os = "windows")]
    let (var_name, original) = ("APPDATA", std::env::var("APPDATA").ok());
    #[cfg(not(target_os = "windows"))]
    let (var_name, original) = ("HOME", std::env::var("HOME").ok());

    let sandbox = tmp_dir("sandbox_appdata");
    std::env::set_var(var_name, &sandbox);

    f();

    match original {
        Some(val) => std::env::set_var(var_name, val),
        None => std::env::remove_var(var_name),
    }
    let _ = fs::remove_dir_all(&sandbox);
}

#[test]
#[serial]
fn read_bootstrap_data_root_returns_none_when_no_config() {
    with_sandboxed_app_data(|| {
        assert_eq!(read_bootstrap_data_root().unwrap(), None);
    });
}

#[test]
#[serial]
fn write_then_read_bootstrap_data_root_roundtrips() {
    with_sandboxed_app_data(|| {
        let root = PathBuf::from("D:/EmbroideryCatalogue/Data");
        write_bootstrap_data_root(&root).expect("write should succeed");

        let read_back = read_bootstrap_data_root()
            .expect("read should succeed")
            .expect("config should exist");
        assert_eq!(read_back, root);
    });
}

#[test]
#[serial]
fn write_bootstrap_data_root_rejects_relative_path() {
    with_sandboxed_app_data(|| {
        let result = write_bootstrap_data_root(Path::new("relative/path"));
        assert!(result.is_err());
    });
}

#[test]
#[serial]
fn read_bootstrap_data_root_errors_on_malformed_config() {
    with_sandboxed_app_data(|| {
        // Write a malformed config directly.
        let config_path = bootstrap_config_path();
        fs::create_dir_all(config_path.parent().unwrap()).expect("create dir");
        fs::write(&config_path, "{ not valid json").expect("write malformed config");

        let result = read_bootstrap_data_root();
        assert!(result.is_err());
    });
}

#[test]
#[serial]
fn configured_data_root_missing_returns_none_when_no_config() {
    with_sandboxed_app_data(|| {
        assert_eq!(configured_data_root_missing().unwrap(), None);
    });
}

#[test]
#[serial]
fn configured_data_root_missing_true_when_configured_path_absent() {
    with_sandboxed_app_data(|| {
        // Configure a root that does not exist on disk.
        let missing = tmp_dir("configured_missing_root").join("Data");
        write_bootstrap_data_root(&missing).expect("write config");

        assert_eq!(configured_data_root_missing().unwrap(), Some(true));
    });
}

#[test]
#[serial]
fn configured_data_root_missing_false_when_configured_path_exists() {
    with_sandboxed_app_data(|| {
        // Configure a root that exists on disk.
        let existing = tmp_dir("configured_existing_root").join("Data");
        fs::create_dir_all(&existing).expect("create data dir");
        write_bootstrap_data_root(&existing).expect("write config");

        assert_eq!(configured_data_root_missing().unwrap(), Some(false));
    });
}

#[cfg(not(debug_assertions))]
#[test]
#[serial]
fn platform_data_root_uses_configured_root_when_present() {
    with_sandboxed_app_data(|| {
        let chosen = PathBuf::from("E:/UserData");
        write_bootstrap_data_root(&chosen).expect("write config");

        let root = platform_data_root();
        assert_eq!(root, chosen);
        assert!(
            !root.to_string_lossy().contains("EmbroideryCatalogue"),
            "configured root should not be inside the app-data fallback"
        );
    });
}

#[cfg(not(debug_assertions))]
#[test]
#[serial]
fn platform_data_root_falls_back_to_appdata_when_no_config() {
    with_sandboxed_app_data(|| {
        let root = platform_data_root();
        assert!(
            root.to_string_lossy().contains("EmbroideryCatalogue"),
            "fallback should live under the app-data dir, got {}",
            root.display()
        );
    });
}