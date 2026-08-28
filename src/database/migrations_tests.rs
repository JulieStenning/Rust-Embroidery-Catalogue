// Tests for the source module.
//
// This module was split out so the production file can stay focused
// on logic. It is included via a #[path] declaration in a
// #[cfg(test)] mod tests; module, retaining full access to the
// private items in the parent module through use super::*;.

use super::*;
use sqlx::sqlite::SqlitePoolOptions;

// â”€â”€â”€ Helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

fn unique_tmp_dir(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "embroidery-migration-test-{}-{}",
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

async fn in_memory_pool() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .expect("create in-memory SQLite pool")
}

/// Convert a filesystem path to forward slashes for use in a `sqlite:///` URL.
fn path_to_sqlite_abs_url(path: &std::path::Path) -> String {
    format!("sqlite:///{}", path.to_string_lossy().replace('\\', "/"))
}

async fn on_disk_pool(path: &std::path::Path) -> SqlitePool {
    let url = path_to_sqlite_abs_url(path);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .expect("create on-disk SQLite pool")
}

// â”€â”€â”€ Helper: seed a database with the initial schema tables â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
async fn seed_schema_tables(pool: &SqlitePool) {
    // Apply the DDL from migration 1 (all use IF NOT EXISTS, so safe to re-run)
    sqlx::query(include_str!(
        "../../migrations/20260503000000_initial.up.sql"
    ))
    .execute(pool)
    .await
    .expect("seed initial schema tables");
}

// â”€â”€â”€ is_already_exists_error â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn already_exists_true_when_database_contains_message() {
    let pool = in_memory_pool().await;
    // Create a table first, then try to create it again without IF NOT EXISTS.
    sqlx::query("CREATE TABLE _test_ae_t(x)")
        .execute(&pool)
        .await
        .expect("first create should succeed");
    let err = sqlx::query("CREATE TABLE _test_ae_t(x)")
        .execute(&pool)
        .await
        .unwrap_err();

    let migrate_err = MigrateError::Execute(err);
    assert!(is_already_exists_error(&migrate_err));
}

#[tokio::test]
async fn already_exists_false_for_non_existent_table_error() {
    let pool = in_memory_pool().await;
    let err = sqlx::query("SELECT * FROM _no_such_table_xyz")
        .execute(&pool)
        .await
        .unwrap_err();

    let migrate_err = MigrateError::Execute(err);
    assert!(!is_already_exists_error(&migrate_err));
}

#[test]
fn already_exists_false_for_protocol_error() {
    let err = sqlx::Error::Protocol("some protocol error".to_string());
    let migrate_err = MigrateError::Execute(err);
    assert!(!is_already_exists_error(&migrate_err));
}

#[test]
fn already_exists_false_for_non_execute_variant() {
    // VersionMismatch is not an Execute or ExecuteMigration variant.
    let err = MigrateError::VersionMismatch(1);
    assert!(!is_already_exists_error(&err));
}

// â”€â”€â”€ is_sqlite_locked_error â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn sqlite_locked_false_for_protocol_variant() {
    let err = sqlx::Error::Protocol("some protocol error".to_string());
    assert!(!is_sqlite_locked_error(&err));
}

#[tokio::test]
async fn sqlite_locked_true_for_concurrent_write_busy() {
    // Open two separate pools to the same on-disk file with busy_timeout=0,
    // so a concurrent write immediately returns SQLITE_BUSY (code "5").
    let tmp = unique_tmp_dir("locked-busy");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let db_path = tmp.join("busy.db");
    std::fs::write(&db_path, []).expect("create empty db file");

    let url = path_to_sqlite_abs_url(&db_path);

    let pool_a = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .expect("pool A");
    let pool_b = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .expect("pool B");

    // Set busy_timeout = 0 on both pools so writes fail immediately on lock.
    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_a)
        .await
        .expect("busy_timeout 0 on A");
    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_b)
        .await
        .expect("busy_timeout 0 on B");

    // Create a table to write to.
    sqlx::query("CREATE TABLE IF NOT EXISTS _busy_test(x)")
        .execute(&pool_a)
        .await
        .expect("create table");

    // Begin an explicit write transaction in pool A and hold it open.
    let mut tx_a = pool_a.begin().await.expect("begin tx A");
    sqlx::query("INSERT INTO _busy_test(x) VALUES(1)")
        .execute(&mut *tx_a)
        .await
        .expect("insert in tx A");

    // Attempt a write in pool B while tx A is still holding a lock.
    let err = sqlx::query("INSERT INTO _busy_test(x) VALUES(2)")
        .execute(&pool_b)
        .await
        .unwrap_err();

    // Roll back tx A so the DB file can be cleaned up.
    tx_a.rollback().await.ok();

    assert!(
        is_sqlite_locked_error(&err),
        "expected is_sqlite_locked_error to be true for SQLITE_BUSY: {:?}",
        err
    );

    pool_a.close().await;
    pool_b.close().await;
    let _ = std::fs::remove_dir_all(&tmp);
}

// â”€â”€â”€ is_locked_migration_error â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn locked_migration_false_for_non_execute_variant() {
    let err = MigrateError::VersionMismatch(1);
    assert!(!is_locked_migration_error(&err));
}

#[test]
fn locked_migration_false_for_execute_with_protocol_inner() {
    let inner = sqlx::Error::Protocol("not a database error".to_string());
    let err = MigrateError::Execute(inner);
    assert!(!is_locked_migration_error(&err));
}

#[tokio::test]
async fn locked_migration_true_for_execute_with_busy_inner() {
    // Use two separate database files to produce two real SQLITE_BUSY errors,
    // one for MigrateError::Execute and one for MigrateError::ExecuteMigration.
    // (sqlx::Error is not Clone, so we cannot reuse the same error.)
    let tmp = unique_tmp_dir("locked-mig-busy");
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    // â”€â”€ Error for MigrateError::Execute â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    let db_path_1 = tmp.join("busy_exec.db");
    std::fs::write(&db_path_1, []).expect("create empty db file");
    let url_1 = path_to_sqlite_abs_url(&db_path_1);

    let pool_a1 = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url_1)
        .await
        .expect("pool A1");
    let pool_b1 = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url_1)
        .await
        .expect("pool B1");

    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_a1)
        .await
        .ok();
    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_b1)
        .await
        .ok();
    sqlx::query("CREATE TABLE IF NOT EXISTS t(x)")
        .execute(&pool_a1)
        .await
        .expect("create table");
    let mut tx1 = pool_a1.begin().await.expect("begin tx");
    sqlx::query("INSERT INTO t(x) VALUES(1)")
        .execute(&mut *tx1)
        .await
        .expect("insert");
    let sqlite_err_exec = sqlx::query("INSERT INTO t(x) VALUES(2)")
        .execute(&pool_b1)
        .await
        .unwrap_err();
    tx1.rollback().await.ok();
    pool_a1.close().await;
    pool_b1.close().await;

    // â”€â”€ Error for MigrateError::ExecuteMigration â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    let db_path_2 = tmp.join("busy_execmig.db");
    std::fs::write(&db_path_2, []).expect("create empty db file");
    let url_2 = path_to_sqlite_abs_url(&db_path_2);

    let pool_a2 = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url_2)
        .await
        .expect("pool A2");
    let pool_b2 = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url_2)
        .await
        .expect("pool B2");

    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_a2)
        .await
        .ok();
    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_b2)
        .await
        .ok();
    sqlx::query("CREATE TABLE IF NOT EXISTS t(x)")
        .execute(&pool_a2)
        .await
        .expect("create table");
    let mut tx2 = pool_a2.begin().await.expect("begin tx");
    sqlx::query("INSERT INTO t(x) VALUES(1)")
        .execute(&mut *tx2)
        .await
        .expect("insert");
    let sqlite_err_execmig = sqlx::query("INSERT INTO t(x) VALUES(2)")
        .execute(&pool_b2)
        .await
        .unwrap_err();
    tx2.rollback().await.ok();
    pool_a2.close().await;
    pool_b2.close().await;

    // â”€â”€ Assertions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    let migrate_err = MigrateError::Execute(sqlite_err_exec);
    assert!(
        is_locked_migration_error(&migrate_err),
        "expected is_locked_migration_error to be true for SQLITE_BUSY"
    );

    let migrate_err_2 = MigrateError::ExecuteMigration(sqlite_err_execmig, 20260730000000_i64);
    assert!(
        is_locked_migration_error(&migrate_err_2),
        "expected is_locked_migration_error to be true for ExecuteMigration with SQLITE_BUSY"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

// â”€â”€â”€ run_migrations â€” happy path on a fresh DB â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn run_migrations_applies_all_migrations_to_fresh_db() {
    let tmp = unique_tmp_dir("fresh-migrate");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let db_path = tmp.join("fresh.db");
    // Touch the file so SQLite can open it.
    std::fs::write(&db_path, []).expect("create empty db file");

    let pool = on_disk_pool(&db_path).await;
    let result = run_migrations(&pool).await;
    assert!(result.is_ok(), "migrations should succeed on fresh DB");

    // Verify the tracking table exists.
    let tracking_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .expect("query sqlite_master");
    assert_eq!(tracking_exists.0, 1, "_sqlx_migrations table must exist");

    // Verify some expected application tables exist.
    for table in &["designs", "tags", "settings", "projects"] {
        let query_str = format!(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'",
            table
        );
        let count: (i64,) = sqlx::query_as(sqlx::AssertSqlSafe(query_str))
            .fetch_one(&pool)
            .await
            .expect("query sqlite_master");
        assert_eq!(count.0, 1, "table '{}' must exist after migrations", table);
    }

    // Verify migration 3's columns were added.
    let cols: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM pragma_table_info('designs') WHERE name IN ('file_size_bytes', 'file_hash_blake3')",
        )
        .fetch_one(&pool)
        .await
        .expect("query pragma_table_info");
    assert_eq!(cols.0, 2, "fingerprint columns should exist on designs");

    pool.close().await;
    let _ = std::fs::remove_dir_all(&tmp);
}

// â”€â”€â”€ run_migrations â€” idempotent (run twice) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn run_migrations_is_idempotent() {
    let tmp = unique_tmp_dir("idempotent");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let db_path = tmp.join("idempotent.db");
    std::fs::write(&db_path, []).expect("create empty db file");

    let pool = on_disk_pool(&db_path).await;

    // First run.
    let r1 = run_migrations(&pool).await;
    assert!(r1.is_ok(), "first migration run should succeed");

    // Second run.
    let r2 = run_migrations(&pool).await;
    assert!(
        r2.is_ok(),
        "second migration run should succeed (idempotent)"
    );

    // Verify all tables still exist after second run.
    let table_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table'")
            .fetch_one(&pool)
            .await
            .expect("query sqlite_master");
    // At minimum: _sqlx_migrations, designers, sources, hoops, tags, designs,
    // projects, settings, design_tags, project_designs = 10
    assert!(
        table_count.0 >= 10,
        "expected at least 10 tables, got {}",
        table_count.0
    );

    pool.close().await;
    let _ = std::fs::remove_dir_all(&tmp);
}

// â”€â”€â”€ run_migrations â€” seed DB scenario (pre-created schema, no tracking) â”€

#[tokio::test]
async fn run_migrations_catches_index_already_exists_on_re_migrate() {
    // Simulate a fully-migrated DB whose _sqlx_migrations tracking table was
    // lost (e.g. seed DB scenario).  When migration 3 (plain CREATE INDEX
    // without IF NOT EXISTS) runs again, it fails with "already exists" and
    // the guard catches it, returning Ok(()) before ever reaching migration 4's
    // ALTER TABLE ADD COLUMN.
    let tmp = unique_tmp_dir("seed-db");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let db_path = tmp.join("seeded.db");
    std::fs::write(&db_path, []).expect("create empty db file");

    let pool = on_disk_pool(&db_path).await;

    // Fully migrate the DB first.
    run_migrations(&pool)
        .await
        .expect("initial full migration should succeed");

    // Drop the _sqlx_migrations tracking table â€” this simulates a seed DB
    // where the schema is present but the tracking metadata is missing.
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
        .execute(&pool)
        .await
        .expect("drop sqlx tracking table");

    // Re-run migrations.
    //   Migration 1 (IF NOT EXISTS):  silently succeeds.
    //   Migration 2 (IF NOT EXISTS):  silently succeeds.
    //   Migration 3 (plain CREATE INDEX):  fails â†’ guard catches "already exists".
    //   Migration 4 is never reached.
    let result = run_migrations(&pool).await;
    assert!(
        result.is_ok(),
        "migrations should succeed: is_already_exists_error catches \
             migration 3's 'already exists' error and bails early: {:?}",
        result
    );

    // The _sqlx_migrations table should have been re-created by SQLx.
    let tracking_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .expect("query sqlite_master");
    assert_eq!(
        tracking_exists.0, 1,
        "_sqlx_migrations table should have been re-created"
    );

    pool.close().await;
    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn run_migrations_handles_seeded_schema_with_only_idempotent_migrations() {
    // If only migrations 1, 2, and the indexes from migration 3 are pre-applied
    // (all use IF NOT EXISTS or produce "already exists" on re-run), the guard
    // catches the plain-CREATE-INDEX errors.  This validates the
    // is_already_exists_error path inside the retry loop.
    let tmp = unique_tmp_dir("seed-idempotent");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let db_path = tmp.join("seeded_idempotent.db");
    std::fs::write(&db_path, []).expect("create empty db file");

    let pool = on_disk_pool(&db_path).await;

    // Apply migrations 1 + 2 (all IF NOT EXISTS â€” safe to pre-apply).
    seed_schema_tables(&pool).await;
    sqlx::query(include_str!(
        "../../migrations/20260526000001_admin_case_insensitive_uniques.up.sql"
    ))
    .execute(&pool)
    .await
    .expect("seed case-insensitive indexes");

    // Pre-apply the plain CREATE INDEX statements from migration 3
    // (20260630000002_indexes.up.sql).  These do NOT use IF NOT EXISTS,
    // so re-running them will produce "already exists" errors.
    sqlx::query(include_str!(
        "../../migrations/20260630000002_indexes.up.sql"
    ))
    .execute(&pool)
    .await
    .expect("seed migration 3 indexes");

    // Run the full migration pipeline.
    // - Migrations 1 & 2: succeed silently (IF NOT EXISTS).
    // - Migration 3:   CREATE INDEX hits "already exists" â†’ guard catches it.
    // - Migration 4:   ALTER TABLE ADD COLUMN succeeds (columns don't exist
    //                  yet, because we did NOT pre-apply migration 4).
    let result = run_migrations(&pool).await;
    assert!(
        result.is_ok(),
        "migrations should succeed: the is_already_exists_error guard catches \
             migration 3's 'already exists' errors: {:?}",
        result
    );

    // The _sqlx_migrations table should have been created by SQLx.
    let tracking_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .expect("query sqlite_master");
    assert_eq!(
        tracking_exists.0, 1,
        "_sqlx_migrations table should have been created"
    );

    pool.close().await;
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// run_migrations — locked-retry loop (transient SQLITE_BUSY then success)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn run_migrations_retries_after_transient_lock_then_succeeds() {
    let tmp = unique_tmp_dir("locked-retry");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let db_path = tmp.join("locked_retry.db");
    std::fs::write(&db_path, []).expect("create empty db file");

    // Two separate single-connection pools on the same on-disk file.
    let pool_a = on_disk_pool(&db_path).await;
    let pool_b = on_disk_pool(&db_path).await;

    // Disable the internal busy timeout so a concurrent write returns
    // SQLITE_BUSY (code 5) immediately instead of blocking.
    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_a)
        .await
        .expect("busy_timeout 0 on pool A");
    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_b)
        .await
        .expect("busy_timeout 0 on pool B");

    // Hold an uncommitted write transaction on pool B so that migration 1's
    // first write (creating _sqlx_migrations / the initial tables) on pool A
    // hits SQLITE_BUSY.
    let mut tx_b = pool_b.begin().await.expect("begin tx B");
    sqlx::query("CREATE TABLE _lock_retry_t(x)")
        .execute(&mut *tx_b)
        .await
        .expect("acquire write lock in tx B");

    // Run migrations in a spawned task. Attempt 1 should fail with a lock
    // error, sleep RETRY_DELAY_MS, then succeed once we release the lock.
    let pool_a_task = pool_a.clone();
    let migrate = tokio::spawn(async move { run_migrations(&pool_a_task).await });

    // Let the first migration attempt hit the lock, then release it well
    // before the 750ms retry delay elapses.
    sleep(Duration::from_millis(100)).await;
    tx_b.commit().await.expect("commit tx B and release lock");

    let result = migrate.await.expect("migration task should not panic");
    assert!(
        result.is_ok(),
        "migrations should succeed after the transient lock is released: {:?}",
        result
    );

    pool_a.close().await;
    pool_b.close().await;
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// run_migrations — exhausted-retry path (persistent lock → AppError)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn run_migrations_returns_error_after_retries_exhausted() {
    let tmp = unique_tmp_dir("locked-exhaust");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let db_path = tmp.join("locked_exhaust.db");
    std::fs::write(&db_path, []).expect("create empty db file");

    let pool_a = on_disk_pool(&db_path).await;
    let pool_b = on_disk_pool(&db_path).await;

    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_a)
        .await
        .expect("busy_timeout 0 on pool A");
    sqlx::query("PRAGMA busy_timeout = 0")
        .execute(&pool_b)
        .await
        .expect("busy_timeout 0 on pool B");

    // Hold the write lock for the entire migration run so every retry
    // (6 attempts × 750ms) fails; on the final attempt attempt < MAX_ATTEMPTS
    // is false and run_migrations returns AppError::Database.
    let mut tx_b = pool_b.begin().await.expect("begin tx B");
    sqlx::query("CREATE TABLE _lock_exhaust_t(x)")
        .execute(&mut *tx_b)
        .await
        .expect("acquire write lock in tx B");

    let pool_a_task = pool_a.clone();
    let migrate = tokio::spawn(async move { run_migrations(&pool_a_task).await });

    let result = migrate.await.expect("migration task should not panic");
    assert!(
        result.is_err(),
        "migrations should fail after exhausting all retry attempts"
    );

    match &result {
        Err(AppError::Database { message }) => assert!(
            message.contains("database migration failed"),
            "unexpected database error message: {}",
            message
        ),
        other => panic!("expected AppError::Database, got: {:?}", other),
    }

    // Release the lock now that the migration task has finished, then clean up.
    tx_b.rollback().await.ok();

    pool_a.close().await;
    pool_b.close().await;
    let _ = std::fs::remove_dir_all(&tmp);
}

