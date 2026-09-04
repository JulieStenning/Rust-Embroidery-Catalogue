// Tests for the database-recovery route.
//
// Included via #[path] so the route file stays focused on the command surface.
// All three commands are pure path logic (no Tauri State), so they can be
// tested directly.

use super::*;

fn unique_tmp_dir(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "embroidery-db-recovery-route-{}-{}",
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

// ---------------------------------------------------------------------------
// validate_database_path
// ---------------------------------------------------------------------------

#[test]
fn validate_database_path_rejects_empty_root() {
    let result = validate_database_path("   ".to_string());
    let err = result.expect_err("empty root should error");
    assert!(err.contains("cannot be empty"), "unexpected error: {err}");
}

#[test]
fn validate_database_path_accepts_root_with_database() {
    let tmp = unique_tmp_dir("valid-db");
    std::fs::create_dir_all(tmp.join("Database")).unwrap();
    std::fs::write(
        tmp.join("Database").join(crate::paths::DATABASE_FILENAME),
        b"sqlite-bytes",
    )
    .unwrap();

    let wire = validate_database_path(tmp.to_string_lossy().to_string())
        .expect("valid root should not error");
    assert!(wire.valid);
    assert!(wire.error.is_none());
    assert_eq!(wire.data_root, tmp.to_string_lossy().to_string());
    assert_eq!(
        wire.database_path,
        tmp.join("Database")
            .join(crate::paths::DATABASE_FILENAME)
            .to_string_lossy()
            .to_string()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn validate_database_path_reports_missing_database() {
    let tmp = unique_tmp_dir("missing-db");
    std::fs::create_dir_all(&tmp).unwrap();

    let wire = validate_database_path(tmp.to_string_lossy().to_string())
        .expect("missing db should not error at the command layer");
    assert!(!wire.valid);
    let err = wire.error.expect("should carry an error message");
    assert!(err.contains("No database found"), "unexpected error: {err}");

    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// detect_relocated_data_root
// ---------------------------------------------------------------------------

#[test]
fn detect_relocated_data_root_reports_relative_subpath() {
    let wire = detect_relocated_data_root("D:\\EmbroideryCatalogue\\Data".to_string())
        .expect("detect should not error");
    assert_eq!(wire.relative_subpath, "EmbroideryCatalogue/Data");
}

#[test]
fn detect_relocated_data_root_empty_subpath_for_relative_root() {
    let wire = detect_relocated_data_root("relative/path".to_string())
        .expect("relative root should not error");
    assert_eq!(wire.relative_subpath, "");
    assert!(wire.data_root.is_none());
}

// ---------------------------------------------------------------------------
// seed_database_to_data_root
// ---------------------------------------------------------------------------

#[test]
fn seed_database_to_data_root_rejects_empty_root() {
    let result = seed_database_to_data_root("   ".to_string(), None);
    let err = result.expect_err("empty root should error");
    assert!(err.contains("cannot be empty"), "unexpected error: {err}");
}

#[test]
fn seed_database_to_data_root_seeds_fresh_catalogue() {
    let tmp = unique_tmp_dir("seed-fresh");
    std::fs::create_dir_all(&tmp).unwrap();

    seed_database_to_data_root(tmp.to_string_lossy().to_string(), None)
        .expect("seeding should succeed");

    let db = tmp.join("Database").join(crate::paths::DATABASE_FILENAME);
    assert!(db.is_file(), "seed database should be written");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn seed_database_to_data_root_refuses_overwrite() {
    let tmp = unique_tmp_dir("seed-no-overwrite");
    std::fs::create_dir_all(&tmp).unwrap();

    seed_database_to_data_root(tmp.to_string_lossy().to_string(), None)
        .expect("first seed should succeed");

    let second = seed_database_to_data_root(tmp.to_string_lossy().to_string(), None);
    assert!(
        second.is_err(),
        "seeding over an existing database without overwrite should error"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
