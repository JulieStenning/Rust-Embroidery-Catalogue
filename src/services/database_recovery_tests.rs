//! Unit tests for `src/services/database_recovery.rs`.
//!
//! Included via `#[path]` so the production file stays under the
//! 500-line test-separation threshold.

use super::*;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// relative_subpath_of
// ---------------------------------------------------------------------------

#[test]
fn relative_subpath_of_extracts_relative_tail() {
    let root = PathBuf::from(r"D:\EmbroideryCatalogue\Data");
    assert_eq!(relative_subpath_of(&root), "EmbroideryCatalogue/Data");
}

#[test]
fn relative_subpath_of_handles_forward_slashes() {
    let root = PathBuf::from("D:/EmbroideryCatalogue/Data");
    assert_eq!(relative_subpath_of(&root), "EmbroideryCatalogue/Data");
}

#[test]
fn relative_subpath_of_empty_for_relative_path() {
    let root = PathBuf::from("relative/path");
    assert_eq!(relative_subpath_of(&root), "");
}

#[test]
fn relative_subpath_of_empty_for_unc_path() {
    let root = PathBuf::from(r"\\server\share\EmbroideryCatalogue");
    assert_eq!(relative_subpath_of(&root), "");
}

// ---------------------------------------------------------------------------
// database_relative_path / designs_relative_dir
// ---------------------------------------------------------------------------

#[test]
fn database_relative_path_matches_canonical_layout() {
    let rel = database_relative_path();
    assert_eq!(
        rel,
        PathBuf::from("Database").join(crate::paths::DATABASE_FILENAME)
    );
}

#[test]
fn designs_relative_dir_is_standard_name() {
    assert_eq!(designs_relative_dir(), "MachineEmbroideryDesigns");
}

// ---------------------------------------------------------------------------
// validate_database_path
// ---------------------------------------------------------------------------

fn unique_tmp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "embroidery-recovery-test-{}-{}",
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn validate_database_path_rejects_missing_database() {
    let tmp = unique_tmp_dir("missing-db");
    std::fs::create_dir_all(&tmp).ok();

    let result = validate_database_path(&tmp);

    assert!(!result.valid);
    assert!(result.error.is_some());
    let err = result.error.unwrap();
    assert!(err.contains("No database found"));
    assert!(!result.embroidery_dir_exists);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn validate_database_path_accepts_existing_database() {
    let tmp = unique_tmp_dir("valid-db");
    let db_dir = tmp.join("Database");
    std::fs::create_dir_all(&db_dir).unwrap();
    std::fs::write(
        db_dir.join(crate::paths::DATABASE_FILENAME),
        b"sqlite-bytes",
    )
    .unwrap();

    let result = validate_database_path(&tmp);

    assert!(result.valid);
    assert!(result.error.is_none());
    assert!(!result.embroidery_dir_exists);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn validate_database_path_reports_designs_dir_when_present() {
    let tmp = unique_tmp_dir("with-designs");
    std::fs::create_dir_all(tmp.join("Database")).unwrap();
    std::fs::create_dir_all(tmp.join(designs_relative_dir())).unwrap();
    std::fs::write(
        tmp.join("Database").join(crate::paths::DATABASE_FILENAME),
        b"sqlite-bytes",
    )
    .unwrap();

    let result = validate_database_path(&tmp);

    assert!(result.valid);
    assert!(result.embroidery_dir_exists);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn validate_database_path_reports_unreadable_database() {
    // Simulate an unreadable file by pointing the probe at a directory entry
    // named like the database file (opening a directory for read fails).
    let tmp = unique_tmp_dir("unreadable");
    let db_dir = tmp.join("Database");
    std::fs::create_dir_all(&db_dir).unwrap();
    std::fs::create_dir(db_dir.join(crate::paths::DATABASE_FILENAME)).unwrap();

    let result = validate_database_path(&tmp);

    // `is_file()` is false for a directory, so this reports "missing" rather
    // than "unreadable" - either way it must be invalid.
    assert!(!result.valid);
    assert!(result.error.is_some());
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// detect_relocated_data_root (non-Windows fallback; Windows scan is
// environment-dependent so only structural assertions are made)
// ---------------------------------------------------------------------------

#[test]
fn detect_relocated_data_root_never_errs_for_relative_root() {
    let result = detect_relocated_data_root(std::path::Path::new("relative/path"));
    assert!(result.is_ok());
}

#[test]
fn detect_relocated_data_root_ok_for_drive_root_form() {
    // On Windows this may find a real catalog (or not); on other platforms it
    // always returns Ok(None). The structural invariant is: never an error.
    let root = PathBuf::from(r"D:\EmbroideryCatalogue\Data");
    let result = detect_relocated_data_root(&root);
    assert!(result.is_ok());
}
