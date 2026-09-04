// Tests for the fingerprint service.
//
// This module was split out of fingerprint.rs so the service file can
// stay focused on production logic. It is included via a #[path]
// declaration in a #[cfg(test)] mod tests; module, so it retains full
// access to the private items in the parent module through use super::*;.

use super::*;
use sqlx::SqlitePool;
use std::io::Write;

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

async fn make_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("memory db");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS designs (
                id INTEGER PRIMARY KEY,
                filename TEXT NOT NULL,
                filepath TEXT NOT NULL,
                file_size_bytes INTEGER,
                file_hash_blake3 TEXT
            )",
    )
    .execute(&pool)
    .await
    .expect("schema");

    sqlx::query("DELETE FROM designs")
        .execute(&pool)
        .await
        .expect("clean designs table");

    pool
}

fn write_temp_file(name: &str, content: &[u8]) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("fingerprint_test");
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(name);
    let mut file = fs::File::create(&path).expect("create temp file");
    file.write_all(content).expect("write temp file");
    path
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn backfill_populates_size_and_hash() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    let temp_path = write_temp_file("test_design.pes", b"dummy stitch data");

    sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (1, 'test_design.pes', ?)")
        .bind(temp_path.to_string_lossy().to_string())
        .execute(&pool)
        .await
        .expect("insert");

    let summary = run_fingerprint_backfill(&pool, 10)
        .await
        .expect("run succeeds");

    assert_eq!(summary.processed, 1);
    assert_eq!(summary.errors, 0);
    assert!(!summary.stopped);

    let size: i64 = sqlx::query_scalar("SELECT file_size_bytes FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("query size");
    assert!(size > 0);

    let hash: String = sqlx::query_scalar("SELECT file_hash_blake3 FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("query hash");
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 64);

    let _ = fs::remove_file(&temp_path);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn backfill_handles_missing_file_with_sentinel() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;

    sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (2, 'gone.pes', '/nonexistent/gone.pes')")
            .execute(&pool)
            .await
            .expect("insert");

    let summary = run_fingerprint_backfill(&pool, 10)
        .await
        .expect("run succeeds");

    assert_eq!(summary.processed, 1);
    assert_eq!(summary.errors, 0);
    assert_eq!(summary.missing_files, 1);

    let size: i64 = sqlx::query_scalar("SELECT file_size_bytes FROM designs WHERE id = 2")
        .fetch_one(&pool)
        .await
        .expect("query size");
    assert_eq!(size, -1);

    let hash: String = sqlx::query_scalar("SELECT file_hash_blake3 FROM designs WHERE id = 2")
        .fetch_one(&pool)
        .await
        .expect("query hash");
    assert!(hash.is_empty());
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn backfill_is_idempotent() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    let temp_path = write_temp_file("idempotent_test.pes", b"idempotent data");

    sqlx::query(
        "INSERT INTO designs (id, filename, filepath) VALUES (3, 'idempotent_test.pes', ?)",
    )
    .bind(temp_path.to_string_lossy().to_string())
    .execute(&pool)
    .await
    .expect("insert");

    let first = run_fingerprint_backfill(&pool, 10)
        .await
        .expect("first run");
    assert_eq!(first.processed, 1);

    let second = run_fingerprint_backfill(&pool, 10)
        .await
        .expect("second run");
    assert_eq!(second.processed, 0);

    let _ = fs::remove_file(&temp_path);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn backfill_respects_stop_signal() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    let temp_path = write_temp_file("stop_test.pes", b"stop test data");

    for i in 10..20 {
        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
            .bind(i)
            .bind(format!("design_{}.pes", i))
            .bind(temp_path.to_string_lossy().to_string())
            .execute(&pool)
            .await
            .expect("insert");
    }

    backfill::stop_requested_store(true);

    let summary = run_fingerprint_backfill(&pool, 10)
        .await
        .expect("run succeeds");

    assert!(summary.stopped);
    assert_eq!(summary.processed, 0);

    backfill::clear_stop_signal();
    let _ = fs::remove_file(&temp_path);
}

#[test]
fn test_strip_sqlite_prefix() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(strip_sqlite_prefix("sqlite:///path/to/db"), "path/to/db");
    assert_eq!(strip_sqlite_prefix("sqlite://path/to/db"), "path/to/db");
    assert_eq!(strip_sqlite_prefix("sqlite:path/to/db"), "path/to/db");
    assert_eq!(strip_sqlite_prefix("path/to/db"), "path/to/db");
}

#[test]
fn test_resolve_fingerprint_source_path() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let old_val = std::env::var("DATABASE_URL").ok();
    std::env::set_var(
        "DATABASE_URL",
        "sqlite:data/database/EmbroideryCatalogue.db",
    );

    let base = derive_designs_base_path();
    assert_eq!(resolve_fingerprint_source_path(""), base);
    assert_eq!(resolve_fingerprint_source_path("   "), base);

    // Legacy container-prefixed stored forms still resolve under the designs
    // base (the container is canonical-relative'd away and rejoined).
    assert_eq!(
        resolve_fingerprint_source_path("/MachineEmbroideryDesigns/foo/bar.pes"),
        base.join("foo/bar.pes")
    );
    assert_eq!(
        resolve_fingerprint_source_path("machineembroiderydesigns/foo/bar.pes"),
        base.join("foo/bar.pes")
    );

    #[cfg(windows)]
    {
        // A Windows drive path is a true absolute path and passes through as-is.
        let abs = resolve_fingerprint_source_path("C:/some/absolute/path.pes");
        assert_eq!(abs, PathBuf::from("C:/some/absolute/path.pes"));
    }
    #[cfg(not(windows))]
    {
        // On non-Windows a bare leading '/' has no drive and is treated as the
        // base-root marker (legacy stored-filepath convention), so it resolves
        // under the designs base.
        let abs = resolve_fingerprint_source_path("/some/absolute/path.pes");
        assert_eq!(abs, base.join("some/absolute/path.pes"));
    }

    let rel = resolve_fingerprint_source_path("foo/bar.pes");
    assert_eq!(rel, base.join("foo/bar.pes"));

    if let Some(v) = old_val {
        std::env::set_var("DATABASE_URL", v);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[test]
fn test_derive_data_root_path() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let old_val = std::env::var("DATABASE_URL").ok();

    std::env::set_var("DATABASE_URL", "sqlite:some/parent/database/test.db");
    let path1 = derive_data_root_path();
    assert!(path1.to_string_lossy().contains("some/parent"));

    std::env::set_var("DATABASE_URL", "sqlite:some/parent/other/test.db");
    let path2 = derive_data_root_path();
    assert!(path2.to_string_lossy().contains("some/parent/other"));

    std::env::set_var("DATABASE_URL", "sqlite:./test.db");
    let path3 = derive_data_root_path();
    assert!(!path3.to_string_lossy().is_empty());

    if let Some(v) = old_val {
        std::env::set_var("DATABASE_URL", v);
    } else {
        std::env::remove_var("DATABASE_URL");
    }
}

#[tokio::test]
async fn test_select_candidates_error() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("memory db");
    let res = select_candidates(&pool, 10, &std::collections::HashSet::new()).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_select_candidates_type_mismatch() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("memory db");
    sqlx::query("DROP TABLE IF EXISTS designs")
        .execute(&pool)
        .await
        .expect("drop designs");
    sqlx::query(
        "CREATE TABLE designs (
                id TEXT,
                filepath TEXT,
                file_size_bytes INTEGER,
                file_hash_blake3 TEXT
            )",
    )
    .execute(&pool)
    .await
    .expect("schema");

    sqlx::query("INSERT INTO designs (id, filepath, file_size_bytes, file_hash_blake3) VALUES ('not_an_int', 'path', NULL, NULL)")
            .execute(&pool)
            .await
            .expect("insert");

    let res = select_candidates(&pool, 10, &std::collections::HashSet::new()).await;
    assert!(res.is_err());
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_process_one_design_metadata_error() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    // We need fs::metadata to fail with a real (non-NotFound) IO error.
    // A plain missing path is handled as a "missing file" sentinel and
    // returns Ok, so it cannot be used here.
    let blocker = write_temp_file("metadata_blocker.pes", b"x");
    let bad_path = {
        #[cfg(windows)]
        {
            // '<' and '>' are invalid characters in Windows file names.
            // The OS rejects the whole path with ERROR_INVALID_NAME
            // (io::ErrorKind::InvalidInput) before any missing-file
            // logic can kick in. Note that a file used as an intermediate
            // directory component does NOT work on Windows: the path is
            // resolved against the file and the trailing component simply
            // becomes ERROR_PATH_NOT_FOUND (io::ErrorKind::NotFound).
            blocker.with_file_name("bad<>.pes")
        }
        #[cfg(not(windows))]
        {
            // A regular file used as a directory component yields ENOTDIR
            // (io::ErrorKind::NotADirectory) on Unix-likes.
            blocker.join("child.pes")
        }
    };

    sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (1, 'invalid.pes', ?)")
        .bind(bad_path.to_string_lossy().to_string())
        .execute(&pool)
        .await
        .expect("insert");

    let candidate = FingerprintCandidate {
        id: 1,
        filepath: bad_path.to_string_lossy().to_string(),
    };
    let res = process_one_design(&pool, candidate).await;
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), AppError::Io { .. }));

    let _ = fs::remove_file(&blocker);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_process_one_design_open_error() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    let dir = std::env::temp_dir().join("fingerprint_test_dir");
    fs::create_dir_all(&dir).expect("create dir");

    sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (1, 'dir', ?)")
        .bind(dir.to_string_lossy().to_string())
        .execute(&pool)
        .await
        .expect("insert");

    let candidate = FingerprintCandidate {
        id: 1,
        filepath: dir.to_string_lossy().to_string(),
    };
    let res = process_one_design(&pool, candidate).await;
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), AppError::Io { .. }));

    let _ = fs::remove_dir(&dir);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_process_one_design_only_hash_present() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    let temp_path = write_temp_file("hash_only.pes", b"some data");

    sqlx::query("INSERT INTO designs (id, filename, filepath, file_hash_blake3, file_size_bytes) VALUES (1, 'hash_only.pes', ?, 'somehash', NULL)")
            .bind(temp_path.to_string_lossy().to_string())
            .execute(&pool)
            .await
            .expect("insert");

    let candidate = FingerprintCandidate {
        id: 1,
        filepath: temp_path.to_string_lossy().to_string(),
    };
    let res = process_one_design(&pool, candidate).await;
    assert!(res.is_ok());

    let size: i64 = sqlx::query_scalar("SELECT file_size_bytes FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(size, 9);

    let hash: String = sqlx::query_scalar("SELECT file_hash_blake3 FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(hash, "somehash");

    let _ = fs::remove_file(&temp_path);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_clamp_commit_every() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    let summary = run_fingerprint_backfill(&pool, 0).await.unwrap();
    assert_eq!(summary.processed, 0);

    let summary2 = run_fingerprint_backfill(&pool, 200_000).await.unwrap();
    assert_eq!(summary2.processed, 0);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_stop_mid_batch() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    let temp_path = write_temp_file("mid_stop.pes", b"data");

    for i in 1..=50 {
        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (?, ?, ?)")
            .bind(i)
            .bind(format!("mid_{}.pes", i))
            .bind(temp_path.to_string_lossy().to_string())
            .execute(&pool)
            .await
            .unwrap();
    }

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        loop {
            let size: Option<i64> =
                sqlx::query_scalar("SELECT file_size_bytes FROM designs WHERE id = 1")
                    .fetch_optional(&pool_clone)
                    .await
                    .unwrap()
                    .flatten();
            if size.is_some() {
                backfill::stop_requested_store(true);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    });

    let summary = run_fingerprint_backfill(&pool, 1).await.unwrap();
    assert!(summary.stopped);
    assert!(summary.processed >= 1);

    backfill::clear_stop_signal();
    let _ = fs::remove_file(&temp_path);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_backfill_handles_processing_error() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    // A directory path passes fs::metadata but fails fs::File::open, so
    // process_one_design returns an AppError::Io, which the backfill loop
    // counts as a per-design error instead of aborting the whole run.
    let dir = std::env::temp_dir().join("fingerprint_test_dir_backfill");
    fs::create_dir_all(&dir).expect("create dir");

    sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (1, 'error.pes', ?)")
        .bind(dir.to_string_lossy().to_string())
        .execute(&pool)
        .await
        .unwrap();

    let summary = run_fingerprint_backfill(&pool, 10).await.unwrap();
    assert!(summary.processed >= 1);
    assert!(summary.errors >= 1);

    let _ = fs::remove_dir(&dir);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_backfill_select_candidates_error() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("memory db");
    let res = run_fingerprint_backfill(&pool, 10).await;
    assert!(res.is_err());
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_process_one_design_fully_hashed_short_circuits() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    backfill::clear_stop_signal();

    let pool = make_test_pool().await;
    let temp_path = write_temp_file("hashed.pes", b"ignored content");

    // Both hash and size are already populated, so process_one_design must
    // short-circuit and never re-read the on-disk file.
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, file_hash_blake3, file_size_bytes) VALUES (1, 'hashed.pes', ?, 'deadbeef', 42)",
    )
    .bind(temp_path.to_string_lossy().to_string())
    .execute(&pool)
    .await
    .expect("insert");

    let candidate = FingerprintCandidate {
        id: 1,
        filepath: temp_path.to_string_lossy().to_string(),
    };
    let result = process_one_design(&pool, candidate)
        .await
        .expect("should succeed without re-hashing");
    assert!(!result.was_missing);

    // Stored values must be untouched.
    let size: i64 = sqlx::query_scalar("SELECT file_size_bytes FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(size, 42);
    let hash: String = sqlx::query_scalar("SELECT file_hash_blake3 FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(hash, "deadbeef");

    let _ = fs::remove_file(&temp_path);
}
