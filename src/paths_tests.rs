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

// â”€â”€â”€ helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€â”€ DATABASE_FILENAME â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn database_filename_is_correct() {
    assert_eq!(DATABASE_FILENAME, "EmbroideryCatalogue.db");
}

// â”€â”€â”€ ExecutionMode derives â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn execution_mode_debug_fmt() {
    assert_eq!(format!("{:?}", ExecutionMode::Portable), "Portable");
    assert_eq!(format!("{:?}", ExecutionMode::Installed), "Installed");
}

#[test]
fn execution_mode_clone() {
    let a = ExecutionMode::Portable;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn execution_mode_partial_eq() {
    assert_eq!(ExecutionMode::Portable, ExecutionMode::Portable);
    assert_ne!(ExecutionMode::Portable, ExecutionMode::Installed);
}

#[test]
fn execution_mode_serialize() {
    let portable = serde_json::to_value(ExecutionMode::Portable).unwrap();
    assert_eq!(portable, serde_json::json!("Portable"));

    let installed = serde_json::to_value(ExecutionMode::Installed).unwrap();
    assert_eq!(installed, serde_json::json!("Installed"));
}

// â”€â”€â”€ resolve_paths_from_exe_dir â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn portable_mode_when_data_dir_exists() {
    let tmp = tmp_dir("portable_mode_when_data_dir_exists");
    fs::create_dir_all(tmp.join("exe").join("data")).expect("create exe/data");
    fs::create_dir_all(tmp.join("exe").join("other")).expect("create exe/other");

    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    assert_eq!(app_paths.mode, ExecutionMode::Portable);
    assert_eq!(app_paths.data_root, tmp.join("exe").join("data"));
    assert!(app_paths.data_root.exists());
    assert_eq!(
        app_paths.embroidery_designs_dir,
        tmp.join("exe")
            .join("data")
            .join("MachineEmbroideryDesigns")
    );
    assert_eq!(
        app_paths.database_dir,
        tmp.join("exe").join("data").join("Database")
    );
    assert_eq!(
        app_paths.database_path,
        tmp.join("exe")
            .join("data")
            .join("Database")
            .join(DATABASE_FILENAME)
    );
    assert_eq!(
        app_paths.thumbnail_cache_dir,
        tmp.join("exe").join("data").join("thumbnails")
    );
    assert_eq!(app_paths.log_dir, tmp.join("exe").join("data").join("logs"));
    // All dirs created
    assert!(app_paths.embroidery_designs_dir.exists());
    assert!(app_paths.database_dir.exists());
    assert!(app_paths.thumbnail_cache_dir.exists());
    assert!(app_paths.log_dir.exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn portable_mode_with_canonical_data_case_on_case_sensitive_fs() {
    // On Linux/macOS "Data" != "data" (case-sensitive filesystem).
    // "Data" is the canonical folder name and must be detected.
    let tmp = tmp_dir("portable_mode_with_canonical_data");
    fs::create_dir_all(tmp.join("exe").join("Data")).expect("create exe/Data");
    fs::create_dir_all(tmp.join("exe").join("other")).expect("create exe/other");

    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    // "Data" (canonical case) â†’ Portable
    assert_eq!(app_paths.mode, ExecutionMode::Portable);
    assert_eq!(app_paths.data_root, tmp.join("exe").join("Data"));

    let _ = fs::remove_dir_all(&tmp);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn portable_mode_with_lowercase_data_on_case_sensitive_fs() {
    // On Linux/macOS "Data" != "data" (case-sensitive filesystem).
    // The legacy lowercase "data" folder is still accepted.
    let tmp = tmp_dir("portable_mode_with_lowercase_data");
    fs::create_dir_all(tmp.join("exe").join("data")).expect("create exe/data");
    fs::create_dir_all(tmp.join("exe").join("other")).expect("create exe/other");

    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    // "data" (legacy case) â†’ Portable
    assert_eq!(app_paths.mode, ExecutionMode::Portable);
    assert_eq!(app_paths.data_root, tmp.join("exe").join("data"));

    let _ = fs::remove_dir_all(&tmp);
}

/// On Windows "Data" and "data" are the same directory, so we verify
/// that the common case (all-lowercase "data") still works.
#[cfg(target_os = "windows")]
#[test]
fn portable_mode_detected_on_windows() {
    let tmp = tmp_dir("portable_mode_on_windows");
    fs::create_dir_all(tmp.join("exe").join("data")).expect("create exe/data");

    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    assert_eq!(app_paths.mode, ExecutionMode::Portable);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn portable_mode_detected_with_canonical_data_case() {
    // The canonical data folder is `Data/` (matching the runtime layout
    // `Data/Database/EmbroideryCatalogue.db`).
    let tmp = tmp_dir("portable_mode_with_data_caps");
    fs::create_dir_all(tmp.join("exe").join("Data")).expect("create exe/Data");

    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    assert_eq!(app_paths.mode, ExecutionMode::Portable);
    assert_eq!(app_paths.data_root, tmp.join("exe").join("Data"));
    assert_eq!(
        app_paths.database_dir,
        tmp.join("exe").join("Data").join("Database")
    );
    assert!(app_paths.database_dir.exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn legacy_lowercase_data_dir_still_detected() {
    // On case-sensitive filesystems an existing `data/` folder must still
    // resolve to Portable mode rather than falling back to Installed.
    let tmp = tmp_dir("portable_mode_legacy_lowercase");
    fs::create_dir_all(tmp.join("exe").join("data")).expect("create exe/data");

    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    assert_eq!(app_paths.mode, ExecutionMode::Portable);
    assert_eq!(app_paths.data_root, tmp.join("exe").join("data"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn installed_mode_when_no_data_dir() {
    let tmp = tmp_dir("installed_mode_when_no_data_dir");
    fs::create_dir_all(tmp.join("exe")).expect("create exe");

    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    // When there's no "data" directory, should be Installed
    assert_eq!(app_paths.mode, ExecutionMode::Installed);
    // data_root should be the platform data root (not under tmp)
    assert_ne!(app_paths.data_root, tmp.join("exe").join("data"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn empty_exe_dir_does_not_panic() {
    // An exe dir that doesn't exist on disk should not cause a panic
    let tmp = tmp_dir("empty_exe_dir");
    let nonexistent = tmp.join("nonexistent_exe_dir");

    // This should not panic â€” will instantiate Installed mode
    let app_paths = resolve_paths_from_exe_dir(&nonexistent);

    // Should not be Portable since the dir doesn't exist
    assert_eq!(
        app_paths.mode,
        ExecutionMode::Installed,
        "non-existent directory should fall back to Installed mode"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn resolve_paths_creates_all_directories() {
    let tmp = tmp_dir("resolve_paths_creates_all_dirs");
    fs::create_dir_all(tmp.join("exe")).expect("create exe");

    // No data dir exists â†’ creates dirs under platform data root
    let app_paths = resolve_paths_from_exe_dir(&tmp.join("exe"));

    // Verify directories exist
    assert!(app_paths.data_root.exists());
    assert!(app_paths.embroidery_designs_dir.exists());
    assert!(app_paths.database_dir.exists());
    assert!(app_paths.thumbnail_cache_dir.exists());
    assert!(app_paths.log_dir.exists());

    let _ = fs::remove_dir_all(&tmp);
}

// â”€â”€â”€ platform_data_root â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

    // Fallback is "."
    assert_eq!(root, PathBuf::from("."));

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
    assert_eq!(root, PathBuf::from("."));

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
    assert_eq!(root, PathBuf::from("."));

    if let Some(val) = original {
        std::env::set_var("HOME", val);
    }
}

// â”€â”€â”€ to_absolute â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€â”€ to_relative â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
    //  while a non-existent file path does not â€” causing a mismatch.)
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

// â”€â”€â”€ resolve_app_paths (integration via current_exe) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
//
// These tests call the real resolve_app_paths() which uses std::env::current_exe().
// During `cargo test` the binary lives in target/debug/deps/<hash>/<exe>, and
// there won't be a `data/` subdirectory alongside it, so it will always resolve
// to Installed mode.  That is still valuable â€” we verify no panics, path
// consistency, and that all subdirectories are created under the platform data
// root without polluting any filesystem.

#[test]
fn resolve_app_paths_does_not_panic_and_smoke_checks() {
    let app_paths = resolve_app_paths().unwrap();

    // Resolved to some valid mode
    assert!(
        app_paths.mode == ExecutionMode::Portable || app_paths.mode == ExecutionMode::Installed,
        "Expected Portable or Installed, got {:?}",
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

// â”€â”€â”€ round-trip â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn relative_absolute_roundtrip_with_real_dirs() {
    let tmp = tmp_dir("roundtrip_real");
    fs::create_dir_all(tmp.join("root").join("deep").join("deeper")).expect("create dirs");

    let root = tmp.join("root");
    let original_relative = PathBuf::from("deep/deeper/file.dst");
    let absolute = root.join(&original_relative);

    // Touch the file so canonicalize works
    fs::write(&absolute, b"test data").expect("write test file");

    // Round-trip: relative â†’ absolute â†’ relative
    let reconstructed_absolute = to_absolute(&original_relative, &root);
    assert_eq!(reconstructed_absolute, absolute);

    let reconstructed_relative = to_relative(&absolute, &root).unwrap();
    assert_eq!(reconstructed_relative, original_relative);

    let _ = fs::remove_dir_all(&tmp);
}
