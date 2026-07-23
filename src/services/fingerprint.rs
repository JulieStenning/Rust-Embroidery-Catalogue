// Content fingerprinting backfill: compute BLAKE3 hash and file size for designs
// missing those columns, storing results incrementally into the database.
//
// Design decisions:
// - Uses the same STOP_REQUESTED atomic as backfill.rs for unified stop support.
// - Processes designs in chunks of size `commit_every` to keep commits small and
//   interruptible.
// - Missing files are recorded with sentinel values (file_size_bytes = -1,
//   file_hash_blake3 = ''), preventing re-scanning on subsequent runs.
// - BLAKE3 hashing streams file contents to avoid loading large embroidery files
//   entirely into memory.

use serde::Serialize;
use sqlx::{Row, SqlitePool};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::config::BootstrapConfig;
use crate::services::backfill;

#[derive(Debug, Clone, Serialize)]
pub struct FingerprintSummary {
    pub processed: i64,
    pub errors: i64,
    pub missing_files: i64,
    pub stopped: bool,
}

pub async fn run_fingerprint_backfill(
    pool: &SqlitePool,
    commit_every: i64,
) -> Result<FingerprintSummary, String> {
    let commit_every = commit_every.clamp(1, 100_000);

    let mut processed: i64 = 0;
    let mut errors: i64 = 0;
    let mut missing_files: i64 = 0;

    loop {
        if backfill::is_stop_requested() {
            break;
        }

        // Always fetch from the front of the remaining candidate set.
        // This avoids skipping rows as prior candidates are updated.
        let batch = select_candidates(pool, commit_every).await?;
        if batch.is_empty() {
            break;
        }

        for candidate in batch {
            if backfill::is_stop_requested() {
                break;
            }

            let design_id = candidate.id;
            processed += 1;

            match process_one_design(pool, candidate).await {
                Ok(result) => {
                    if result.was_missing {
                        missing_files += 1;
                    }
                }
                Err(err_msg) => {
                    errors += 1;
                    backfill::log_error(format!(
                        "Fingerprint backfill failed design_id={} error={}",
                        design_id, err_msg
                    ));
                }
            }
        }
    }

    Ok(FingerprintSummary {
        processed,
        errors,
        missing_files,
        stopped: backfill::is_stop_requested(),
    })
}

#[derive(Debug, Clone)]
struct FingerprintCandidate {
    id: i64,
    filepath: String,
}

struct ProcessResult {
    was_missing: bool,
}

async fn select_candidates(pool: &SqlitePool, limit: i64) -> Result<Vec<FingerprintCandidate>, String> {
    let rows = sqlx::query(
        "SELECT id, filepath
         FROM designs
         WHERE file_size_bytes IS NULL
            OR file_hash_blake3 IS NULL
         ORDER BY id ASC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut candidates = Vec::with_capacity(rows.len());
    for row in rows {
        candidates.push(FingerprintCandidate {
            id: row.try_get::<i64, _>("id").map_err(|e| e.to_string())?,
            filepath: row
                .try_get::<String, _>("filepath")
                .map_err(|e| e.to_string())?,
        });
    }

    Ok(candidates)
}

fn strip_sqlite_prefix(database_url: &str) -> &str {
    database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url)
}

fn derive_data_root_path() -> PathBuf {
    let config = BootstrapConfig::from_env();
    let db_path = Path::new(strip_sqlite_prefix(&config.database_url));

    let root = if let Some(parent) = db_path.parent() {
        if parent
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case("database"))
            .unwrap_or(false)
        {
            parent.parent().unwrap_or(parent).to_path_buf()
        } else {
            parent.to_path_buf()
        }
    } else {
        PathBuf::from("data")
    };

    root.canonicalize().unwrap_or(root)
}

fn derive_designs_base_path() -> PathBuf {
    derive_data_root_path().join("MachineEmbroideryDesigns")
}

// Resolve a stored DB path to a concrete on-disk path.
// Handles stored paths such as "/MachineEmbroideryDesigns/foo/bar.pes".
fn resolve_fingerprint_source_path(stored_filepath: &str) -> PathBuf {
    let designs_base = derive_designs_base_path();
    let normalized = stored_filepath.trim().replace('\\', "/");
    if normalized.is_empty() {
        return designs_base;
    }

    let cleaned = normalized.trim_start_matches('/');
    let cleaned_lower = cleaned.to_ascii_lowercase();
    if cleaned_lower == "machineembroiderydesigns"
        || cleaned_lower.starts_with("machineembroiderydesigns/")
    {
        let data_root = designs_base
            .parent()
            .map(|value| value.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        return data_root.join(cleaned);
    }

    let candidate = PathBuf::from(&normalized);
    if candidate.is_absolute() {
        return candidate;
    }

    designs_base.join(cleaned)
}

async fn process_one_design(
    pool: &SqlitePool,
    candidate: FingerprintCandidate,
) -> Result<ProcessResult, String> {
    let source_path = resolve_fingerprint_source_path(&candidate.filepath);
    let source_display = source_path.to_string_lossy().to_string();

    let current_hash: Option<String> =
        sqlx::query_scalar("SELECT file_hash_blake3 FROM designs WHERE id = ?")
            .bind(candidate.id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .flatten();

    if current_hash.as_ref().map_or(false, |h| !h.is_empty()) {
        let current_size: Option<i64> =
            sqlx::query_scalar("SELECT file_size_bytes FROM designs WHERE id = ?")
                .bind(candidate.id)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?
                .flatten();

        if current_size.map_or(false, |s| s > 0) {
            return Ok(ProcessResult { was_missing: false });
        }
    }

    let metadata = match fs::metadata(&source_path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            sqlx::query("UPDATE designs SET file_size_bytes = -1, file_hash_blake3 = '' WHERE id = ?")
                .bind(candidate.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

            backfill::log_error(format!(
                "Fingerprint: file missing on disk design_id={} stored_path={} resolved_path={}",
                candidate.id, candidate.filepath, source_display
            ));

            return Ok(ProcessResult { was_missing: true });
        }
        Err(e) => {
            return Err(format!(
                "Failed to read file metadata for '{}': {}",
                source_display, e
            ));
        }
    };

    let file_size: i64 = metadata.len() as i64;

    let hash_needed = current_hash.as_ref().map_or(true, String::is_empty);
    let hash_string = if hash_needed {
        let mut file = fs::File::open(&source_path)
            .map_err(|e| format!("Failed to open file for hashing '{}': {}", source_display, e))?;

        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 65536];
        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|e| format!("Failed to hash file '{}': {}", source_display, e))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Some(hasher.finalize().to_hex().to_string())
    } else {
        None
    };

    if let Some(ref hash) = hash_string {
        sqlx::query("UPDATE designs SET file_size_bytes = ?, file_hash_blake3 = ? WHERE id = ?")
            .bind(file_size)
            .bind(hash)
            .bind(candidate.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("UPDATE designs SET file_size_bytes = ? WHERE id = ?")
            .bind(file_size)
            .bind(candidate.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(ProcessResult { was_missing: false })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use std::io::Write;

    async fn make_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("memory db");
        sqlx::query(
            "CREATE TABLE designs (
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
    async fn backfill_populates_size_and_hash() {
        let pool = make_test_pool().await;
        let temp_path = write_temp_file("test_design.pes", b"dummy stitch data");

        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (1, 'test_design.pes', ?)")
            .bind(temp_path.to_string_lossy().to_string())
            .execute(&pool)
            .await
            .expect("insert");

        backfill::clear_stop_signal();
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
    async fn backfill_handles_missing_file_with_sentinel() {
        let pool = make_test_pool().await;

        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (2, 'gone.pes', '/nonexistent/gone.pes')")
            .execute(&pool)
            .await
            .expect("insert");

        backfill::clear_stop_signal();
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
    async fn backfill_is_idempotent() {
        let pool = make_test_pool().await;
        let temp_path = write_temp_file("idempotent_test.pes", b"idempotent data");

        sqlx::query("INSERT INTO designs (id, filename, filepath) VALUES (3, 'idempotent_test.pes', ?)")
            .bind(temp_path.to_string_lossy().to_string())
            .execute(&pool)
            .await
            .expect("insert");

        backfill::clear_stop_signal();
        let first = run_fingerprint_backfill(&pool, 10)
            .await
            .expect("first run");
        assert_eq!(first.processed, 1);

        backfill::clear_stop_signal();
        let second = run_fingerprint_backfill(&pool, 10)
            .await
            .expect("second run");
        assert_eq!(second.processed, 0);

        let _ = fs::remove_file(&temp_path);
    }

    #[tokio::test]
    async fn backfill_respects_stop_signal() {
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

        backfill::clear_stop_signal();
        backfill::stop_requested_store(true);

        let summary = run_fingerprint_backfill(&pool, 10)
            .await
            .expect("run succeeds");

        assert!(summary.stopped);
        assert_eq!(summary.processed, 0);

        backfill::clear_stop_signal();
        let _ = fs::remove_file(&temp_path);
    }
}
