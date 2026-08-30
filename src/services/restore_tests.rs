use super::*;
use crate::paths::AppPaths;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::fs;
use std::path::{Path, PathBuf};

/// Create a unique temp directory for a test run.
fn unique_temp_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("restore-test-{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Create a SQLite database at `path` with a `designs` table containing the
/// given filepath rows.
async fn make_db(path: &Path, filepaths: &[&str]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create db parent");
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
        )
        .await
        .expect("create db");
    sqlx::query("CREATE TABLE designs (id INTEGER PRIMARY KEY AUTOINCREMENT, filename TEXT NOT NULL, filepath TEXT NOT NULL, date_added TEXT)")
        .execute(&pool)
        .await
        .expect("create table");
    for fp in filepaths {
        sqlx::query(
            "INSERT INTO designs (filename, filepath, date_added) VALUES (?, ?, DATE('now'))",
        )
        .bind(fp.split('/').next_back().unwrap_or(fp))
        .bind(fp)
        .execute(&pool)
        .await
        .expect("insert row");
    }
    pool.close().await;
}

/// Build an `AppPaths` rooted at `root` with the standard layout.
fn make_app_paths(root: &Path) -> AppPaths {
    AppPaths {
        mode: crate::paths::ExecutionMode::Installed,
        data_root: root.to_path_buf(),
        embroidery_designs_dir: root.join("MachineEmbroideryDesigns"),
        database_dir: root.join("Database"),
        database_path: root.join("Database").join("catalogue.db"),
        log_dir: root.join("logs"),
    }
}

/// Force the file's last-modified time to `t` so size+mtime equality can be
/// asserted deterministically (identical files written within the same second
/// are otherwise racy across a second boundary).
fn set_mtime(path: &Path, t: std::time::SystemTime) {
    let file = fs::File::options()
        .write(true)
        .open(path)
        .expect("open file to set mtime");
    file.set_modified(t).expect("set file mtime");
}

/// Set `PRAGMA user_version` on an existing database file (schema version hint).
async fn set_user_version(path: &Path, version: i64) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
        )
        .await
        .expect("open db to set user_version");
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "PRAGMA user_version = {version}"
    )))
    .execute(&pool)
    .await
    .expect("set user_version");
    pool.close().await;
}

/// Read the design count from a database file using a read-only connection.
async fn design_count_at(path: &Path) -> i64 {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(path).read_only(true))
        .await
        .expect("open db read-only");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM designs")
        .fetch_one(&pool)
        .await
        .expect("count designs");
    pool.close().await;
    count
}

/// Create a database with the full `designs` schema required by the
/// unmatched-file import path (mirrors the table shape used across the app).
async fn make_designs_db(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create db parent");
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
        )
        .await
        .expect("create db");
    sqlx::query(
        r#"
        CREATE TABLE designs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            filepath TEXT NOT NULL,
            date_added TEXT,
            designer_id INTEGER,
            source_id INTEGER,
            hoop_id INTEGER,
            image_data BLOB,
            image_type TEXT,
            width_mm REAL,
            height_mm REAL,
            stitch_count INTEGER,
            color_count INTEGER,
            color_change_count INTEGER,
            is_stitched INTEGER NOT NULL DEFAULT 0,
            image_tags_verified INTEGER NOT NULL DEFAULT 0,
            stitching_tags_verified INTEGER NOT NULL DEFAULT 0,
            tagging_mode TEXT,
            file_size_bytes INTEGER,
            file_hash_blake3 TEXT
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("create designs table");
    pool.close().await;
}

#[tokio::test]
async fn validate_backup_file_accepts_db_and_rejects_others() {
    let tmp = unique_temp_dir("validate");
    let db_file = tmp.join("catalogue_2026-08-25_1430.db");
    fs::write(&db_file, b"sqlite").unwrap();
    let named = tmp.join("EmbroideryCatalogue.db");
    fs::write(&named, b"sqlite").unwrap();
    let txt = tmp.join("notes.txt");
    fs::write(&txt, b"hello").unwrap();

    assert!(validate_backup_file(&db_file).is_ok());
    assert!(validate_backup_file(&named).is_ok());
    assert!(validate_backup_file(&txt).is_err());

    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn perform_designs_restore_copies_new_skips_identical_updates_changed() {
    let tmp = unique_temp_dir("designs");
    let source = tmp.join("backup");
    let dest = tmp.join("MachineEmbroideryDesigns");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();

    // New file: only in source -> copied.
    fs::write(source.join("new.pes"), b"new-content").unwrap();
    // Identical file: same content -> copied to dest, then mtimes forced equal
    // so it deterministically skips (a same-second write alone is racy).
    fs::write(source.join("same.pes"), b"same-content").unwrap();
    fs::write(dest.join("same.pes"), b"same-content").unwrap();
    let fixed = std::time::SystemTime::now();
    set_mtime(&source.join("same.pes"), fixed);
    set_mtime(&dest.join("same.pes"), fixed);
    // Changed file: source differs from dest -> updated.
    fs::write(source.join("changed.pes"), b"source-version").unwrap();
    fs::write(dest.join("changed.pes"), b"dest-version").unwrap();

    let cancel = AtomicBool::new(false);
    let mut emitted = 0u64;
    let mut progress = |_p: RestoreProgress| emitted += 1;
    let outcome = perform_designs_restore(&source, &dest, &cancel, &mut progress)
        .await
        .unwrap();

    assert_eq!(outcome.scanned, 3);
    assert_eq!(outcome.copied, 1);
    assert_eq!(outcome.skipped, 1);
    assert_eq!(outcome.updated, 1);
    assert!(outcome.success);
    // Progress is emitted once per iteration (copies, updates, and skips), so the
    // three files in this run each emit an event.
    assert!(emitted >= 2);

    // The changed file on disk now has the source content.
    let changed = fs::read_to_string(dest.join("changed.pes")).unwrap();
    assert_eq!(changed, "source-version");

    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn detect_design_files_absent_from_database_reports_unreferenced_files() {
    let tmp = unique_temp_dir("detect");
    let root = tmp.join("MachineEmbroideryDesigns");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("known.pes"), b"x").unwrap();
    fs::write(root.join("unknown.pes"), b"y").unwrap();

    let db_path = tmp.join("test.db");
    make_db(&db_path, &["/MachineEmbroideryDesigns/known.pes"]).await;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&db_path)
                .read_only(true),
        )
        .await
        .expect("open db");

    let result = detect_design_files_absent_from_database(&pool, &root)
        .await
        .unwrap();
    assert_eq!(result.checked, 2);
    assert_eq!(result.unmatched, 1);
    assert!(result.sample.iter().any(|p| p == "unknown.pes"));
    assert!(!result.sample.iter().any(|p| p == "known.pes"));

    pool.close().await;
    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn perform_database_restore_swaps_and_counts_designs() {
    let tmp = unique_temp_dir("dbrestore");
    let paths = make_app_paths(&tmp);

    // Live DB with one design.
    make_db(&paths.database_path, &["/MachineEmbroideryDesigns/one.pes"]).await;

    // Backup DB with two designs.
    let backup_dir = tmp.join("backups");
    let backup_path = backup_dir.join("catalogue_2026-08-25_1430.db");
    make_db(
        &backup_path,
        &[
            "/MachineEmbroideryDesigns/one.pes",
            "/MachineEmbroideryDesigns/two.pes",
        ],
    )
    .await;

    // Pool connected to the live DB.
    let live_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&paths.database_path))
        .await
        .expect("open live db");
    let holder = PoolHolder::new(live_pool);

    let outcome = perform_database_restore(&holder, &paths, &backup_path)
        .await
        .unwrap();

    assert!(outcome.success);
    assert!(!outcome.rolled_back);
    assert_eq!(outcome.design_count, 2);
    assert!(outcome.rollback_copy_path.is_some());

    // The live file should now contain two designs.
    let verify_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&paths.database_path)
                .read_only(true),
        )
        .await
        .expect("open restored db");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM designs")
        .fetch_one(&verify_pool)
        .await
        .unwrap();
    assert_eq!(count, 2);

    // The safety rollback copy was retained on disk next to the live database,
    // named `catalogue.pre-restore-<timestamp>.db`, and holds the original
    // pre-restore snapshot (one design).
    let rollback_path = PathBuf::from(outcome.rollback_copy_path.clone().unwrap());
    assert!(rollback_path.exists(), "rollback copy should exist on disk");
    let rollback_name = rollback_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    assert!(
        rollback_name.starts_with("catalogue.pre-restore-"),
        "unexpected rollback copy name: {rollback_name}"
    );
    assert!(rollback_name.ends_with(".db"));
    assert_eq!(design_count_at(&rollback_path).await, 1);

    verify_pool.close().await;
    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn perform_database_restore_rolls_back_on_corrupt_file_and_keeps_live_db() {
    let tmp = unique_temp_dir("rollback");
    let paths = make_app_paths(&tmp);

    // Live DB with one design.
    make_db(&paths.database_path, &["/MachineEmbroideryDesigns/one.pes"]).await;

    // A corrupt "database" backup: plain text with a .db extension.
    let corrupt_path = tmp.join("corrupt_test.db");
    fs::write(&corrupt_path, b"Not a database").unwrap();

    let live_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&paths.database_path))
        .await
        .expect("open live db");
    let holder = PoolHolder::new(live_pool);

    let result = perform_database_restore(&holder, &paths, &corrupt_path).await;

    // Whether the service surfaces this as an explicit rollback outcome or an
    // Err (the corrupt file fails verification / pool re-open), the safety
    // property must hold: the live database is restored and uncorrupted.
    if let Ok(outcome) = &result {
        assert!(
            !outcome.success,
            "corrupt file must not restore successfully"
        );
        assert!(outcome.rolled_back, "corrupt file must report rollback");
        assert!(outcome.rollback_copy_path.is_some());
    }

    // Live DB file still holds the original single design (uncorrupted).
    assert_eq!(design_count_at(&paths.database_path).await, 1);

    // A .pre-restore-* safety copy was retained next to the live database.
    let rollback_files: Vec<PathBuf> = fs::read_dir(&paths.database_dir)
        .expect("read database dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().starts_with("catalogue.pre-restore-"))
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(
        rollback_files.len(),
        1,
        "expected exactly one safety rollback copy on disk"
    );
    assert!(rollback_files[0].is_file());
    // The retained copy is a valid DB containing the original design.
    assert_eq!(design_count_at(&rollback_files[0]).await, 1);

    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn perform_database_restore_reports_schema_version_hints() {
    let tmp = unique_temp_dir("schema");
    let paths = make_app_paths(&tmp);

    // Live DB, user_version = 3.
    make_db(&paths.database_path, &["/MachineEmbroideryDesigns/one.pes"]).await;
    set_user_version(&paths.database_path, 3).await;

    // Backup DB, user_version = 7, with two designs.
    let backup_path = tmp.join("backups").join("schema_2026-08-25.db");
    make_db(
        &backup_path,
        &[
            "/MachineEmbroideryDesigns/one.pes",
            "/MachineEmbroideryDesigns/two.pes",
        ],
    )
    .await;
    set_user_version(&backup_path, 7).await;

    let live_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&paths.database_path))
        .await
        .expect("open live db");
    let holder = PoolHolder::new(live_pool);

    let outcome = perform_database_restore(&holder, &paths, &backup_path)
        .await
        .unwrap();

    assert!(outcome.success);
    assert_eq!(outcome.schema_version_hint, Some(7));
    assert_eq!(outcome.previous_schema_version_hint, Some(3));

    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn perform_designs_restore_is_graceful_with_missing_source() {
    let tmp = unique_temp_dir("designs-missing");
    let source = tmp.join("does-not-exist");
    let dest = tmp.join("MachineEmbroideryDesigns");
    fs::create_dir_all(&dest).unwrap();

    let cancel = AtomicBool::new(false);
    let mut progress = |_p: RestoreProgress| {};
    // A missing source is a graceful no-op at the service layer (the route
    // guards the directory-existence check before invoking this function).
    let outcome = perform_designs_restore(&source, &dest, &cancel, &mut progress)
        .await
        .unwrap();

    assert!(outcome.success);
    assert_eq!(outcome.scanned, 0);
    assert_eq!(outcome.copied, 0);
    assert_eq!(outcome.skipped, 0);

    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn perform_designs_restore_honours_cancel_flag() {
    let tmp = unique_temp_dir("designs-cancel");
    let source = tmp.join("backup");
    let dest = tmp.join("MachineEmbroideryDesigns");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();
    fs::write(source.join("a.pes"), b"aaa").unwrap();
    fs::write(source.join("b.pes"), b"bbb").unwrap();

    let cancel = AtomicBool::new(true); // cancellation requested up front
    let mut progress = |_p: RestoreProgress| {};
    let outcome = perform_designs_restore(&source, &dest, &cancel, &mut progress)
        .await
        .unwrap();

    // Nothing is copied or updated when cancellation is already requested.
    assert_eq!(outcome.scanned, 2);
    assert_eq!(outcome.copied, 0);
    assert_eq!(outcome.updated, 0);
    assert_eq!(outcome.skipped, 0);
    assert!(!dest.join("a.pes").exists());
    assert!(!dest.join("b.pes").exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn import_unmatched_design_files_imports_real_design() {
    let tmp = unique_temp_dir("import-unmatched");
    let root = tmp.join("MachineEmbroideryDesigns");
    fs::create_dir_all(&root).unwrap();

    // A real, parseable design file present on disk but absent from the DB.
    let bean = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("Test Designs")
        .join("Bean.pes");
    fs::copy(&bean, root.join("Bean.pes")).expect("copy Bean.pes fixture");

    // Database that does not reference Bean.pes, so it is an unmatched file.
    let db_path = tmp.join("catalogue.db");
    make_designs_db(&db_path).await;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&db_path))
        .await
        .expect("open db");

    let result = import_unmatched_design_files(&pool, &root).await.unwrap();
    assert_eq!(result.detected, 1);
    assert_eq!(result.imported, 1);
    assert_eq!(result.failed, 0);

    // The imported design is now queryable with parsed metadata.
    let (filename, filepath, stitch_count): (String, String, i64) =
        sqlx::query_as("SELECT filename, filepath, stitch_count FROM designs WHERE filepath = ?")
            .bind("/MachineEmbroideryDesigns/Bean.pes")
            .fetch_one(&pool)
            .await
            .expect("imported design row");
    assert_eq!(filename, "Bean.pes");
    assert_eq!(filepath, "/MachineEmbroideryDesigns/Bean.pes");
    assert!(
        stitch_count > 0,
        "stitch_count should be parsed from the fixture"
    );

    pool.close().await;
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn restore_progress_new_initializes_defaults() {
    let p = RestoreProgress::new("designs", "syncing");
    assert_eq!(p.phase, "designs");
    assert_eq!(p.db_status, "syncing");
    assert_eq!(p.scanned, 0);
    assert_eq!(p.copied, 0);
    assert_eq!(p.skipped, 0);
    assert_eq!(p.total_bytes, 0);
    assert_eq!(p.percent, 0.0);
    assert!(p.error.is_none());
}

#[tokio::test]
async fn validate_backup_file_rejects_non_file() {
    let tmp = unique_temp_dir("validate-dir");
    // A directory is not a file → rejected by the "not a file" branch.
    let err = validate_backup_file(&tmp).unwrap_err();
    assert!(err.contains("not a file"), "unexpected error: {err}");
    let _ = fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn perform_database_restore_errors_when_live_db_missing() {
    let tmp = unique_temp_dir("no-live-db");
    let paths = make_app_paths(&tmp);

    let backup = tmp.join("backup.db");
    fs::write(&backup, b"sqlite").unwrap();

    // No live database exists at app_paths.database_path → error before the
    // pool is touched, so an empty holder is sufficient.
    let holder = PoolHolder::default();
    let result = perform_database_restore(&holder, &paths, &backup).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Live database not found"), "unexpected error: {err}");
    let _ = fs::remove_dir_all(&tmp);
}
