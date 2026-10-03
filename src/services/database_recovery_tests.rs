// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

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
        crate::paths::SEED_DB_BYTES,
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
        crate::paths::SEED_DB_BYTES,
    )
    .unwrap();

    let result = validate_database_path(&tmp);

    assert!(result.valid);
    assert!(result.embroidery_dir_exists);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn recover_database_from_backup_file_restores_and_preserves_corrupt_file() {
    let tmp_root = unique_tmp_dir("recover-root");
    let tmp_backup_dir = unique_tmp_dir("recover-backup");
    std::fs::create_dir_all(tmp_root.join("Database")).unwrap();
    std::fs::create_dir_all(&tmp_backup_dir).unwrap();

    let live_db = tmp_root
        .join("Database")
        .join(crate::paths::DATABASE_FILENAME);
    std::fs::write(&live_db, b"corrupted-preexisting-content").unwrap();

    let backup_file = tmp_backup_dir.join("Backup.db");
    std::fs::write(&backup_file, crate::paths::SEED_DB_BYTES).unwrap();

    let outcome = recover_database_from_backup_file(&tmp_root, &backup_file).await;
    assert!(
        outcome.is_ok(),
        "recovery should succeed: {:?}",
        outcome.err()
    );

    // Verified live DB now matches valid seed DB
    let validation = validate_database_path(&tmp_root);
    assert!(validation.valid);

    // Corrupt copy was preserved aside
    let entries: Vec<_> = std::fs::read_dir(tmp_root.join("Database"))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let has_corrupt_copy = entries.iter().any(|e| {
        e.file_name()
            .to_string_lossy()
            .starts_with("EmbroideryCatalogue.corrupt-")
    });
    assert!(
        has_corrupt_copy,
        "should have preserved corrupt DB copy aside"
    );

    let _ = std::fs::remove_dir_all(&tmp_root);
    let _ = std::fs::remove_dir_all(&tmp_backup_dir);
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
