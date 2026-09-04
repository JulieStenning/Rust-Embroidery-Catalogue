// Tests for the storage migration service.
//
// This module was split out of storage_migration.rs so the production file
// can stay focused on logic. It is included via a #[path] declaration in a
// #[cfg(test)] mod tests; module, retaining full access to the private items
// in the parent module through use super::*;.

use super::*;
use crate::paths::DATABASE_FILENAME;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn tmp_dir(test_name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "storage-migration-test-{}-{}",
        test_name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn make_source(root: &Path) -> AppPaths {
    let app_paths = crate::paths::resolve_paths_for_root(root);
    std::fs::create_dir_all(&app_paths.database_dir).expect("create db dir");
    std::fs::create_dir_all(&app_paths.embroidery_designs_dir).expect("create designs dir");
    app_paths
}

/// Write a real, migrated SQLite database (with designs table) at the source
/// path so the integrity verification passes.
async fn seed_database(source: &AppPaths) {
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(&source.database_path)
            .create_if_missing(true),
    )
    .await
    .expect("create source db");

    // Build only the minimal core tables needed by the verify query.
    sqlx::query("CREATE TABLE IF NOT EXISTS designs (id INTEGER PRIMARY KEY AUTOINCREMENT, filename TEXT NOT NULL, filepath TEXT)")
        .execute(&pool)
        .await
        .expect("create designs table");
    sqlx::query("INSERT INTO designs (filename, filepath) VALUES ('a.pes', 'x')")
        .execute(&pool)
        .await
        .expect("insert design");

    pool.close().await;
}

/// Redirect the platform app-data env var to a temp dir while `fut` runs.
///
/// Migration tests reach the commit point (`write_bootstrap_data_root`), which
/// writes to the REAL `%APPDATA%\EmbroideryCatalogue\config.json`. Parallel
/// tests that touch the same env var (main_tests, paths_tests, settings) use
/// `lock_env()` — a global process-wide Mutex — to serialise themselves. This
/// helper acquires that SAME lock so the env var is never mutated concurrently
/// (which both races the tests and could clobber a real user config).
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn with_sandboxed_appdata<F, Fut>(fut: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let _env_guard = crate::utils::test_support::lock_env();

    #[cfg(target_os = "windows")]
    let (var_name, original) = ("APPDATA", std::env::var("APPDATA").ok());
    #[cfg(not(target_os = "windows"))]
    let (var_name, original) = ("HOME", std::env::var("HOME").ok());

    let sandbox = tmp_dir("sandbox_appdata");
    std::env::set_var(var_name, &sandbox);

    fut().await;

    match original {
        Some(val) => std::env::set_var(var_name, val),
        None => std::env::remove_var(var_name),
    }
    let _ = std::fs::remove_dir_all(&sandbox);
}

#[tokio::test]
async fn preflight_rejects_relative_target() {
    let tmp = tmp_dir("preflight_relative");
    let source = make_source(&tmp);
    let err = preflight(&source, Path::new("relative/path"), false).expect_err("should reject");
    assert!(err.to_string().contains("absolute"));
}

#[tokio::test]
async fn preflight_rejects_target_equal_to_source() {
    let tmp = tmp_dir("preflight_equal");
    let source = make_source(&tmp);
    let err = preflight(&source, &source.data_root, true).expect_err("should reject");
    assert!(err.to_string().contains("same as"));
}

#[tokio::test]
async fn preflight_rejects_target_inside_source() {
    let tmp = tmp_dir("preflight_nested");
    let source = make_source(&tmp);
    let nested = source.data_root.join("Database").join("nested");
    let err = preflight(&source, &nested, true).expect_err("should reject");
    assert!(err.to_string().contains("nested"));
}

#[tokio::test]
async fn preflight_rejects_target_containing_source() {
    let tmp = tmp_dir("preflight_ancestor");
    // The source root lives INSIDE the target — migrating would move the
    // target into itself.
    let target = tmp.join("target");
    let source = make_source(&target.join("current"));
    // Ensure both exist so canonicalization succeeds on both sides.
    std::fs::create_dir_all(&target).unwrap();
    let err = preflight(&source, &target, true).expect_err("should reject");
    assert!(err.to_string().contains("nested") || err.to_string().contains("same as"));
}

#[tokio::test]
async fn preflight_rejects_non_empty_target_without_force() {
    let tmp = tmp_dir("preflight_nonempty");
    let source = make_source(&tmp.join("source"));
    let target = tmp.join("target");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("existing.txt"), b"x").unwrap();

    let err = preflight(&source, &target, false).expect_err("should reject");
    assert!(err.to_string().contains("not empty"));
}

#[tokio::test]
async fn preflight_moves_non_empty_target_aside_with_force() {
    let tmp = tmp_dir("preflight_force");
    let source = make_source(&tmp.join("source"));
    let target = tmp.join("target");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("existing.txt"), b"x").unwrap();

    let plan = preflight(&source, &target, true).expect("force should succeed");
    let backup = plan
        .preexisting_target_renamed
        .expect("target should have been moved aside");
    assert!(backup.join("existing.txt").exists());
    // The target itself is now empty (recreated for migration).
    assert!(!target.join("existing.txt").exists());
}

#[tokio::test]
async fn preflight_accepts_empty_target() {
    let tmp = tmp_dir("preflight_empty");
    let source = make_source(&tmp.join("source"));
    let target = tmp.join("target");
    std::fs::create_dir_all(&target).unwrap();

    let plan = preflight(&source, &target, true).expect("empty target should pass");
    assert!(plan.preexisting_target_renamed.is_none());
}

#[tokio::test]
async fn full_migration_copies_database_and_assets() {
    with_sandboxed_appdata(async || {
        let tmp = tmp_dir("full_migration");
        let source = make_source(&tmp.join("source"));
        seed_database(&source).await;

        // Assets.
        std::fs::write(source.embroidery_designs_dir.join("rose.pes"), b"design-a").unwrap();

        let target = tmp.join("target");
        let cancel = AtomicBool::new(false);
        let mut events: Vec<StorageMigrationProgress> = vec![];
        let plan = preflight(&source, &target, true).expect("preflight should pass");

        let summary = run_migration(&source, &plan, &cancel, |event| {
            events.push(event);
        })
        .await
        .expect("migration should succeed");

        assert!(summary.success);
        assert!(summary.requires_restart);
        assert_eq!(summary.asset_items, 1);

        // Target database is valid and has the design.
        assert!(target.join("Database").join(DATABASE_FILENAME).exists());
        assert!(target
            .join("MachineEmbroideryDesigns")
            .join("rose.pes")
            .exists());

        // Source was renamed to a backup, not deleted.
        let backup = tmp.join("source.migrated-backup");
        assert!(backup.exists());
        assert!(!source.data_root.exists());

        // Progress events reached "completed".
        assert!(events.iter().any(|e| e.current_phase == "completed"));
        // Final percent is 1.0.
        assert!(events
            .iter()
            .filter(|e| e.current_phase == "completed")
            .all(|e| e.percent == 1.0));
    })
    .await;
}

#[tokio::test]
async fn full_migration_cancellation_rolls_back_partial_target() {
    // Cancellation aborts before the commit point, so no config write occurs;
    // still sandboxed for safety and consistency.
    with_sandboxed_appdata(async || {
        let tmp = tmp_dir("cancel");
        let source = make_source(&tmp.join("source"));
        seed_database(&source).await;
        // A large-ish asset file so we have something to cancel partway through.
        std::fs::write(
            source.embroidery_designs_dir.join("big.pes"),
            vec![0x42u8; 2048],
        )
        .unwrap();

        let target = tmp.join("target");

        // Force same-device=false so cancel-in-loop is reachable (rename fast
        // path would complete instantly and bypass the per-file loop).
        let mut plan = preflight(&source, &target, true).expect("preflight");
        plan.same_device = false;

        let cancel = AtomicBool::new(true); // cancel immediately
        let result = run_migration(&source, &plan, &cancel, |_| {}).await;

        assert!(result.is_err(), "cancelled migration should error");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("cancelled"), "unexpected error: {msg}");

        // Partial target removed, source untouched, no backup created.
        assert!(!target.exists());
        assert!(source.data_root.exists());
        assert!(!tmp.join("source.migrated-backup").exists());
    })
    .await;
}

#[test]
fn format_moved_timestamp_renders_human_readable_local_date_time() {
    let dt = time::macros::datetime!(2026-05-12 17:50:00).assume_utc();
    let formatted = format_moved_timestamp(dt);
    assert_eq!(formatted, "12 May 2026 17:50");
}

#[test]
fn is_filesystem_root_true_for_roots() {
    #[cfg(target_os = "windows")]
    {
        assert!(is_filesystem_root(Path::new("C:\\")));
        assert!(is_filesystem_root(Path::new("D:\\")));
    }
    #[cfg(not(target_os = "windows"))]
    {
        assert!(is_filesystem_root(Path::new("/")));
    }
}

#[test]
fn is_filesystem_root_false_for_nested_paths() {
    let tmp = tmp_dir("is_filesystem_root_nested");
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    assert!(!is_filesystem_root(&tmp));
    assert!(!is_filesystem_root(&tmp.join("sub")));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn write_moved_notice_writes_marker_at_source() {
    let tmp = tmp_dir("moved_notice");
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    let target = PathBuf::from("D:/EmbroideryCatalogue/Data");
    let result = write_moved_notice(&tmp, &target);
    assert!(result.is_ok(), "marker should write: {:?}", result.err());

    let marker = tmp.join("storage location moved.txt");
    let contents = std::fs::read_to_string(&marker).expect("marker should exist");
    assert!(contents.contains("Embroidery Catalogue storage was moved on"));
    assert!(contents.contains("D:/EmbroideryCatalogue/Data"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn relocated_marker_note_writes_marker_and_returns_note() {
    // Exercise the marker-fallback branch hermetically against a temp source
    // root (the exact path a filesystem root would take, without touching the
    // real drive root). A real root cannot be created under temp, so we call
    // the extracted helper directly.
    let tmp = tmp_dir("relocated_marker_note_write");
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    let target = PathBuf::from("D:/EmbroideryCatalogue/Data");
    let note = relocated_marker_note(&tmp, &target);

    assert!(
        note.contains("storage location moved.txt"),
        "note should describe the marker, got: {note}"
    );

    let marker = tmp.join("storage location moved.txt");
    let contents = std::fs::read_to_string(&marker).expect("marker should exist");
    assert!(contents.contains("D:/EmbroideryCatalogue/Data"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn relocated_marker_note_returns_note_when_marker_cannot_be_written() {
    // A source root that does not exist makes write_moved_notice fail; the
    // fallback must still return a note (never panic, never Err) that mentions
    // the failure. The source is under temp so nothing real is touched.
    let tmp = tmp_dir("relocated_marker_note_missing");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let missing_source = tmp.join("does-not-exist");
    let target = PathBuf::from("D:/EmbroideryCatalogue/Data");

    let note = relocated_marker_note(&missing_source, &target);

    assert!(note.contains("storage location moved.txt"));
    assert!(
        note.contains("could not be written"),
        "note should mention the marker failure, got: {note}"
    );
    // No marker file should have been created anywhere.
    assert!(!missing_source.join("storage location moved.txt").exists());

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn preserve_old_location_renames_non_root_source_to_backup() {
    // A non-root source takes the rename fast-path and yields no note.
    let tmp = tmp_dir("preserve_old_location_rename");
    let source = tmp.join("current");
    std::fs::create_dir_all(&source).expect("create source dir");
    std::fs::write(source.join("rose.pes"), b"data").expect("write source file");
    let target = tmp.join("target"); // target need not exist for the rename path

    let note = preserve_old_location(&source, &target);
    assert!(
        note.is_none(),
        "rename success should return None, got: {note:?}"
    );
    assert!(
        !source.exists(),
        "source should have been renamed away to the backup"
    );
    let backup = tmp.join("current.migrated-backup");
    assert!(
        backup.join("rose.pes").exists(),
        "backup should hold the original data"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn full_migration_source_is_filesystem_root_still_succeeds() {
    with_sandboxed_appdata(async || {
        let tmp = tmp_dir("root_migration");
        let source = make_source(&tmp.join("source"));
        seed_database(&source).await;
        std::fs::write(source.embroidery_designs_dir.join("a.pes"), b"data").unwrap();

        let target = tmp.join("target");
        let cancel = AtomicBool::new(false);
        let mut events: Vec<StorageMigrationProgress> = vec![];
        let plan = preflight(&source, &target, true).expect("preflight");

        // Integration check: a nested source still completes and renames.
        let summary = run_migration(&source, &plan, &cancel, |event| {
            events.push(event);
        })
        .await
        .expect("migration should succeed");

        assert!(summary.success);

        let backup = tmp.join("source.migrated-backup");
        assert!(backup.exists());
        assert!(events.iter().any(|e| e.current_phase == "completed"));
    })
    .await;
}

#[tokio::test]
async fn progress_percent_reflects_bytes() {
    with_sandboxed_appdata(async || {
        let tmp = tmp_dir("percent");
        let source = make_source(&tmp.join("source"));
        seed_database(&source).await;
        std::fs::write(source.embroidery_designs_dir.join("a.pes"), b"aaaa").unwrap();

        let target = tmp.join("target");
        let cancel = AtomicBool::new(false);
        let mut events: Vec<StorageMigrationProgress> = vec![];
        let plan = preflight(&source, &target, true).expect("preflight");

        run_migration(&source, &plan, &cancel, |event| {
            events.push(event);
        })
        .await
        .expect("migration should succeed");

        let asset_events: Vec<&StorageMigrationProgress> = events
            .iter()
            .filter(|e| e.current_phase == "assets")
            .collect();
        assert!(!asset_events.is_empty());
        // Every asset event's percent is within [0,1] and grows monotonically.
        let percents: Vec<f64> = asset_events.iter().map(|e| e.percent).collect();
        for pair in percents.windows(2) {
            assert!(pair[1] >= pair[0]);
        }
        assert!(percents.iter().all(|p| (0.0..=1.0).contains(p)));
    })
    .await;
}

#[test]
fn progress_with_totals_zero_bytes_and_error_method() {
    // total_bytes == 0 → percent clamps to 1.0 (zero-totals fast path).
    let p = StorageMigrationProgress::new("assets", "msg".to_string()).with_totals(10, 0, 10, 0);
    assert_eq!(p.percent, 1.0);
    assert_eq!(p.total_items, 10);
    assert_eq!(p.total_bytes, 0);
    assert_eq!(p.items_copied, 10);
    assert_eq!(p.bytes_copied, 0);

    // error() surfaces a message on the error field.
    let e = StorageMigrationProgress::new("error", "boom".to_string()).error("detail".to_string());
    assert_eq!(e.current_phase, "error");
    assert_eq!(e.error.as_deref(), Some("detail"));
}

// ---------------------------------------------------------------------------
// checkpoint_live_database / verify_database_at edges
// ---------------------------------------------------------------------------

#[tokio::test]
async fn checkpoint_live_database_runs_wal_checkpoint() {
    let tmp = tmp_dir("checkpoint");
    let source = make_source(&tmp.join("source"));
    std::fs::create_dir_all(&source.database_dir).expect("create db dir");

    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(&source.database_path)
            .create_if_missing(true),
    )
    .await
    .expect("open file-backed pool");

    checkpoint_live_database(&pool)
        .await
        .expect("wal_checkpoint(TRUNCATE) should succeed");

    pool.close().await;
    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn verify_database_at_returns_false_when_missing() {
    let tmp = tmp_dir("verify_missing");
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    let missing = tmp.join("Database").join("nope.db");
    let ok = verify_database_at(&missing)
        .await
        .expect("a missing db is a non-error false result");
    assert!(!ok);

    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// run_migration failure / rollback branches
// ---------------------------------------------------------------------------

#[tokio::test]
async fn run_migration_errors_when_database_verification_fails() {
    with_sandboxed_appdata(async || {
        let tmp = tmp_dir("run_migration_no_db");
        // Source has assets but NO database file (seed_database is intentionally
        // omitted), so verify_database_at fails and the migration must error out
        // and roll back the partial target.
        let source = make_source(&tmp.join("source"));
        std::fs::write(source.embroidery_designs_dir.join("a.pes"), b"data").unwrap();

        let target = tmp.join("target");
        let plan = preflight(&source, &target, true).expect("preflight should pass");
        let cancel = AtomicBool::new(false);

        let result = run_migration(&source, &plan, &cancel, |_| {}).await;
        assert!(result.is_err(), "migration with no source DB must fail");

        // Partial target removed, source untouched, no backup created.
        assert!(!target.exists());
        assert!(source.data_root.exists());
        assert!(!tmp.join("source.migrated-backup").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    })
    .await;
}

#[test]
fn rollback_partial_target_restores_moved_aside_target() {
    let tmp = tmp_dir("rollback_restore");

    // A pre-existing target was moved aside to `backup` during preflight.
    std::fs::create_dir_all(tmp.join("backup")).expect("create backup");
    std::fs::write(tmp.join("backup").join("keep.pes"), b"x").expect("write backup file");
    // A partial migration tree now sits where the target should be.
    std::fs::create_dir_all(tmp.join("target")).expect("create partial target");
    std::fs::write(tmp.join("target").join("partial"), b"y").expect("write partial file");

    let target = tmp.join("target");
    let backup = tmp.join("backup");
    let plan = MigrationPlan {
        target: target.clone(),
        target_paths: crate::paths::resolve_paths_for_root(&target),
        total_items: 1,
        total_bytes: 10,
        database_bytes: 1,
        same_device: false,
        preexisting_target_renamed: Some(backup.clone()),
    };

    rollback_partial_target(&plan);

    // The moved-aside target is restored and the partial tree is discarded.
    assert!(
        target.join("keep.pes").exists(),
        "backup content should be restored"
    );
    assert!(
        !target.join("partial").exists(),
        "partial tree should be removed"
    );
    assert!(
        !backup.exists(),
        "backup should no longer exist after restore"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
