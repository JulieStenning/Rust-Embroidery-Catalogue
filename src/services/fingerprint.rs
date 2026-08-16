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
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::config::BootstrapConfig;
use crate::error::AppError;
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
) -> Result<FingerprintSummary, AppError> {
    let commit_every = commit_every.clamp(1, 100_000);

    let mut processed: i64 = 0;
    let mut errors: i64 = 0;
    let mut missing_files: i64 = 0;
    // Rows whose fingerprinting fails mid-run must not be re-selected in the
    // same run, otherwise the loop would re-process them forever. They remain
    // candidates for a future run so transient failures can be retried.
    let mut failed_ids: HashSet<i64> = HashSet::new();

    loop {
        if backfill::is_stop_requested() {
            break;
        }

        // Always fetch from the front of the remaining candidate set.
        // This avoids skipping rows as prior candidates are updated.
        let batch = select_candidates(pool, commit_every, &failed_ids).await?;
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
                    failed_ids.insert(design_id);
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

#[derive(Debug, Clone)]
struct ProcessResult {
    was_missing: bool,
}

async fn select_candidates(
    pool: &SqlitePool,
    limit: i64,
    exclude_ids: &HashSet<i64>,
) -> Result<Vec<FingerprintCandidate>, AppError> {
    let mut sql = String::from(
        "SELECT id, filepath
         FROM designs
         WHERE (file_size_bytes IS NULL
            OR file_hash_blake3 IS NULL)",
    );
    if !exclude_ids.is_empty() {
        let placeholders = vec!["?"; exclude_ids.len()].join(", ");
        sql.push_str(&format!(" AND id NOT IN ({})", placeholders));
    }
    sql.push_str(" ORDER BY id ASC LIMIT ?");

    let mut query = sqlx::query(&sql);
    for id in exclude_ids {
        query = query.bind(*id);
    }
    query = query.bind(limit);

    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to select fingerprint candidates: {e}")))?;

    let mut candidates = Vec::with_capacity(rows.len());
    for row in rows {
        candidates.push(FingerprintCandidate {
            id: row
                .try_get::<i64, _>("id")
                .map_err(|e| AppError::database(format!("failed to read candidate id: {e}")))?,
            filepath: row.try_get::<String, _>("filepath").map_err(|e| {
                AppError::database(format!("failed to read candidate filepath: {e}"))
            })?,
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
) -> Result<ProcessResult, AppError> {
    let source_path = resolve_fingerprint_source_path(&candidate.filepath);
    let source_display = source_path.to_string_lossy().to_string();

    let current_hash: Option<String> =
        sqlx::query_scalar("SELECT file_hash_blake3 FROM designs WHERE id = ?")
            .bind(candidate.id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::database(format!("failed to read existing hash: {e}")))?
            .flatten();

    if current_hash.as_ref().map_or(false, |h| !h.is_empty()) {
        let current_size: Option<i64> =
            sqlx::query_scalar("SELECT file_size_bytes FROM designs WHERE id = ?")
                .bind(candidate.id)
                .fetch_optional(pool)
                .await
                .map_err(|e| AppError::database(format!("failed to read existing size: {e}")))?
                .flatten();

        if current_size.map_or(false, |s| s > 0) {
            return Ok(ProcessResult { was_missing: false });
        }
    }

    let metadata = match fs::metadata(&source_path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            sqlx::query(
                "UPDATE designs SET file_size_bytes = -1, file_hash_blake3 = '' WHERE id = ?",
            )
            .bind(candidate.id)
            .execute(pool)
            .await
            .map_err(|e| AppError::database(format!("failed to mark missing design: {e}")))?;

            backfill::log_error(format!(
                "Fingerprint: file missing on disk design_id={} stored_path={} resolved_path={}",
                candidate.id, candidate.filepath, source_display
            ));

            return Ok(ProcessResult { was_missing: true });
        }
        Err(e) => {
            return Err(AppError::io(format!(
                "failed to read file metadata for '{}': {}",
                source_display, e
            )));
        }
    };

    let file_size: i64 = metadata.len() as i64;

    let hash_needed = current_hash.as_ref().map_or(true, String::is_empty);
    let hash_string = if hash_needed {
        let mut file = fs::File::open(&source_path).map_err(|e| {
            AppError::io(format!(
                "failed to open file for hashing '{}': {}",
                source_display, e
            ))
        })?;

        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 65536];
        loop {
            let bytes_read = file.read(&mut buffer).map_err(|e| {
                AppError::io(format!("failed to hash file '{}': {}", source_display, e))
            })?;
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
            .map_err(|e| AppError::database(format!("failed to update fingerprint data: {e}")))?;
    } else {
        sqlx::query("UPDATE designs SET file_size_bytes = ? WHERE id = ?")
            .bind(file_size)
            .bind(candidate.id)
            .execute(pool)
            .await
            .map_err(|e| AppError::database(format!("failed to update fingerprint size: {e}")))?;
    }

    Ok(ProcessResult { was_missing: false })
}
#[cfg(test)]
#[path = "fingerprint_tests.rs"]
mod tests;
