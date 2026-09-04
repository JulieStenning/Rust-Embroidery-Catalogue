//! Restore service.
//!
//! Implements the inverse of the in-app backup flow:
//!  - **Database restore** — swaps the live SQLite file for a user-selected
//!    backup snapshot, keeping a `.pre-restore-<ts>` rollback copy and
//!    reverting automatically if integrity verification fails.
//!  - **Designs sync** — an incremental mirror that copies design files from a
//!    backup folder back into `MachineEmbroideryDesigns`, skipping files that
//!    already exist with identical size + mtime.
//!  - **Reconciliation** — detects design files on disk that are absent from
//!    the (restored) database, and can import them as new catalogue records.

use crate::database::connection::establish_connection;
use crate::models::{EmbPattern, StitchType};
use crate::paths::AppPaths;
use crate::readers::{
    DstReader, EmbroideryReader, ExpReader, HusReader, JefReader, PesReader, Vp3Reader,
};
use crate::routes::maintenance as mnt;
use crate::services::storage_migration::verify_database_at;
use crate::PoolHolder;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Tauri event streamed to the frontend during a restore.
pub const RESTORE_PROGRESS_EVENT: &str = "catalogue-restore-progress";

/// Number of sample paths included in detection results / failed imports.
const SAMPLE_LIMIT: usize = 20;

/// Per-callback payload describing the current restore state.
#[derive(Debug, Clone, Serialize)]
pub struct RestoreProgress {
    pub phase: String,
    pub db_status: String,
    pub scanned: u64,
    pub copied: u64,
    pub skipped: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub error: Option<String>,
}

impl RestoreProgress {
    pub fn new(phase: &str, db_status: &str) -> Self {
        Self {
            phase: phase.to_string(),
            db_status: db_status.to_string(),
            scanned: 0,
            copied: 0,
            skipped: 0,
            total_bytes: 0,
            percent: 0.0,
            error: None,
        }
    }
}

/// Result of a database swap + verification attempt.
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseRestoreOutcome {
    pub success: bool,
    pub restored_path: String,
    pub rollback_copy_path: Option<String>,
    pub design_count: u64,
    /// `PRAGMA user_version` of the restored database (schema hint).
    pub schema_version_hint: Option<i64>,
    /// `PRAGMA user_version` of the live database before the swap, so the UI
    /// can warn when a restore changed the schema version.
    pub previous_schema_version_hint: Option<i64>,
    pub rolled_back: bool,
    pub error: Option<String>,
}

/// Result of an incremental designs restore.
#[derive(Debug, Clone, Serialize)]
pub struct DesignsRestoreOutcome {
    pub success: bool,
    pub scanned: u64,
    pub copied: u64,
    pub updated: u64,
    pub skipped: u64,
    pub total_bytes_copied: u64,
    pub error: Option<String>,
}

/// Result of the post-restore reconciliation scan.
#[derive(Debug, Clone, Serialize)]
pub struct DetectUnmatchedFilesResult {
    pub checked: usize,
    pub unmatched: usize,
    pub sample: Vec<String>,
}

/// Result of a batch "import unmatched design files" run.
#[derive(Debug, Clone, Serialize)]
pub struct ImportUnmatchedFilesResult {
    pub detected: usize,
    pub imported: usize,
    pub failed: usize,
    pub failed_samples: Vec<String>,
}


/// Compute the safety rollback copy path next to the live database.
fn rollback_copy_path(live_path: &Path, suffix: &str) -> PathBuf {
    let stem = live_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "catalogue".to_string());
    let ext = live_path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "db".to_string());
    live_path.with_file_name(format!("{stem}.pre-restore-{suffix}.{ext}"))
}

/// Best-effort unique timestamp suffix for rollback copies.
fn timestamp_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

/// Validate that a user-selected file is a plausible SQLite database snapshot.
fn validate_backup_file(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!(
            "Selected restore source is not a file: {}",
            path.display()
        ));
    }
    let name_ok = path
        .file_name()
        .map(|n| n.to_string_lossy().eq_ignore_ascii_case("EmbroideryCatalogue.db"))
        .unwrap_or(false);
    let ext_ok = path
        .extension()
        .map(|e| e.to_string_lossy().eq_ignore_ascii_case("db"))
        .unwrap_or(false);
    if !name_ok && !ext_ok {
        return Err(
            "Please choose an EmbroideryCatalogue.db (or .db) database backup file.".to_string(),
        );
    }
    Ok(())
}

/// Swap the live database file with `backup_path`, keeping a rollback copy.
///
/// The current pool is checkpointed and closed before the file is replaced,
/// then re-opened against the restored file and verified. On verification
/// failure the rollback copy is restored automatically.
pub async fn perform_database_restore(
    holder: &PoolHolder,
    app_paths: &AppPaths,
    backup_path: &Path,
) -> Result<DatabaseRestoreOutcome, String> {
    if let Err(error) = validate_backup_file(backup_path) {
        tracing::error!(
            "[restore] invalid backup file '{}': {error}",
            backup_path.display()
        );
        return Err(error);
    }

    let live_path = &app_paths.database_path;
    if !live_path.is_file() {
        let message = format!("Live database not found at '{}'.", live_path.display());
        tracing::error!("[restore] {message}");
        return Err(message);
    }

    // 1. Flush the live pool (WAL) and close it so the file can be replaced.
    let previous_schema_version_hint = match holder.pool() {
        Some(pool) => sqlx::query_scalar::<_, i64>("PRAGMA user_version")
            .fetch_one(&pool)
            .await
            .ok(),
        None => None,
    };
    if let Some(pool) = holder.pool() {
        let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&pool)
            .await;
    }
    if let Some(pool) = holder.take() {
        pool.close().await;
    }

    // 2. Safety rollback copy of the live file before it is overwritten.
    let suffix = timestamp_suffix();
    let rollback_path = rollback_copy_path(live_path, &suffix);
    if let Err(error) = fs::copy(live_path, &rollback_path) {
        let _ = holder.take(); // close whatever is currently installed
        let message = format!(
            "Could not create safety rollback copy '{}': {error}",
            rollback_path.display()
        );
        tracing::error!("[restore] {message}");
        return Err(message);
    }

    // 3. Copy the selected backup to a temp file, then rename over the live DB.
    let tmp_path = live_path.with_file_name(format!(
        "{}.restore-tmp",
        live_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "catalogue".to_string())
    ));
    if let Err(error) = fs::copy(backup_path, &tmp_path) {
        let _ = fs::copy(&rollback_path, live_path);
        let _ = holder.take();
        let message = format!(
            "Could not copy backup to '{}': {error}",
            tmp_path.display()
        );
        tracing::error!("[restore] {message}");
        return Err(message);
    }
    if let Err(error) = fs::rename(&tmp_path, live_path) {
        let _ = fs::remove_file(&tmp_path);
        let _ = fs::copy(&rollback_path, live_path);
        let _ = holder.take();
        let message = format!(
            "Could not replace live database with '{}': {error}",
            live_path.display()
        );
        tracing::error!("[restore] {message}");
        return Err(message);
    }

    // 4. Re-open the pool against the restored file.
    let new_pool = match establish_connection(app_paths).await {
        Ok(pool) => pool,
        Err(error) => {
            let _ = fs::copy(&rollback_path, live_path);
            let _ = holder.take();
            let message = format!("Could not re-open the database pool: {error}");
            tracing::error!("[restore] {message}");
            return Err(message);
        }
    };
    holder.replace(new_pool);

    // 5. Verify the restored database.
    let valid = verify_database_at(live_path).await.unwrap_or(false);
    tracing::info!(
        "[restore] database restore verification valid={}",
        valid
    );

    if !valid {
        // Automatic rollback using the safety copy.
        if let Some(pool) = holder.take() {
            pool.close().await;
        }
        if let Err(error) = fs::copy(&rollback_path, live_path) {
            let message = format!(
                "Restore verification failed AND automatic rollback failed: {error}"
            );
            tracing::error!("[restore] {message}");
            return Err(message);
        }
        let rolled_pool = establish_connection(app_paths)
            .await
            .map_err(|e| e.to_string())?;
        holder.replace(rolled_pool);
        tracing::warn!(
            "[restore] database restore rolled back to '{}'",
            rollback_path.display()
        );

        return Ok(DatabaseRestoreOutcome {
            success: false,
            restored_path: live_path.to_string_lossy().to_string(),
            rollback_copy_path: Some(rollback_path.to_string_lossy().to_string()),
            design_count: 0,
            schema_version_hint: None,
            previous_schema_version_hint,
            rolled_back: true,
            error: Some(
                "Restored database failed integrity verification and was rolled back.".to_string(),
            ),
        });
    }

    // Gather summary stats from the freshly-opened pool.
    let design_count = match holder.pool() {
        Some(pool) => sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM designs")
            .fetch_one(&pool)
            .await
            .unwrap_or(0),
        None => 0,
    };
    let schema_version_hint = match holder.pool() {
        Some(pool) => sqlx::query_scalar::<_, i64>("PRAGMA user_version")
            .fetch_one(&pool)
            .await
            .ok(),
        None => None,
    };

    tracing::info!(
        "[restore] database restore succeeded restored_path={} design_count={} user_version={:?} previous_user_version={:?}",
        live_path.display(),
        design_count,
        schema_version_hint,
        previous_schema_version_hint,
    );

    Ok(DatabaseRestoreOutcome {
        success: true,
        restored_path: live_path.to_string_lossy().to_string(),
        rollback_copy_path: Some(rollback_path.to_string_lossy().to_string()),
        design_count: design_count.max(0) as u64,
        schema_version_hint,
        previous_schema_version_hint,
        rolled_back: false,
        error: None,
    })
}


/// Incremental mirror restore of design files from `source_root` (a backup
/// folder) into `dest_root` (`MachineEmbroideryDesigns`). Files that already
/// exist at the destination with identical size + mtime are skipped. Additive
/// only — nothing is deleted, and no database records are touched.
pub async fn perform_designs_restore(
    source_root: &Path,
    dest_root: &Path,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(RestoreProgress),
) -> Result<DesignsRestoreOutcome, String> {
    let source_map = mnt::collect_file_snapshots(source_root, true)?;
    let dest_map = mnt::collect_file_snapshots(dest_root, true)?;

    let mut copied = 0u64;
    let mut updated = 0u64;
    let mut skipped = 0u64;
    let mut total_bytes_copied = 0u64;

    for (relative, source) in &source_map {
        if cancel.load(Ordering::SeqCst) {
            break;
        }

        let dest_path = dest_root.join(relative);
        match dest_map.get(relative) {
            Some(existing) if mnt::files_match(existing, source) => {
                skipped += 1;
                let total = source_map.len().max(1) as f64;
                let processed = copied + updated + skipped;
                progress(RestoreProgress {
                    phase: "designs".to_string(),
                    db_status: "syncing".to_string(),
                    scanned: source_map.len() as u64,
                    copied: copied + updated,
                    skipped,
                    total_bytes: total_bytes_copied,
                    percent: (processed as f64 / total).min(1.0),
                    error: None,
                });
                continue;
            }
            _ => {}
        }

        if let Some(parent) = dest_path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                tracing::error!(
                    "[restore] Could not create destination folder '{}': {}",
                    parent.display(),
                    error
                );
                continue;
            }
        }

        match fs::copy(&source.full_path, &dest_path) {
            Ok(bytes) => {
                total_bytes_copied = total_bytes_copied.saturating_add(bytes);
                if dest_map.contains_key(relative) {
                    updated += 1;
                } else {
                    copied += 1;
                }
            }
            Err(error) => {
                tracing::error!(
                    "[restore] Could not copy '{}' to '{}': {}",
                    source.full_path.display(),
                    dest_path.display(),
                    error
                );
            }
        }

        let total = source_map.len().max(1) as f64;
        let processed = copied + updated + skipped;
        progress(RestoreProgress {
            phase: "designs".to_string(),
            db_status: "syncing".to_string(),
            scanned: source_map.len() as u64,
            copied: copied + updated,
            skipped,
            total_bytes: total_bytes_copied,
            percent: (processed as f64 / total).min(1.0),
            error: None,
        });
    }

    let cancelled = cancel.load(Ordering::SeqCst);
    tracing::info!(
        "[restore] designs restore finished scanned={} copied={} updated={} skipped={} bytes={} cancelled={}",
        source_map.len(),
        copied,
        updated,
        skipped,
        total_bytes_copied,
        cancelled,
    );

    Ok(DesignsRestoreOutcome {
        success: true,
        scanned: source_map.len() as u64,
        copied,
        updated,
        skipped,
        total_bytes_copied,
        error: None,
    })
}

/// Build the set of relative design paths referenced by the database, resolved
/// against `designs_root` so it can be compared with on-disk snapshots.
async fn referenced_design_paths(
    pool: &SqlitePool,
    designs_root: &Path,
) -> Result<HashSet<PathBuf>, String> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT filepath FROM designs")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut referenced = HashSet::new();
    for (filepath,) in rows {
        if filepath.trim().is_empty() {
            continue;
        }
        let full = mnt::resolve_design_full_path(designs_root, &filepath);
        if let Ok(relative) = full.strip_prefix(designs_root) {
            referenced.insert(relative.to_path_buf());
        }
    }
    Ok(referenced)
}

/// Return design files present on disk (under `designs_root`) that are absent
/// from the database — the inverse of the existing "orphan" scan.
pub async fn detect_design_files_absent_from_database(
    pool: &SqlitePool,
    designs_root: &Path,
) -> Result<DetectUnmatchedFilesResult, String> {
    let disk = mnt::collect_file_snapshots(designs_root, true)?;
    let referenced = referenced_design_paths(pool, designs_root).await?;

    let mut unmatched: Vec<String> = disk
        .keys()
        .filter(|relative| !referenced.contains(*relative))
        .map(|relative| relative.to_string_lossy().to_string())
        .collect();
    unmatched.sort();

    tracing::info!(
        "[restore] unmatched-file detection checked={} unmatched={}",
        disk.len(),
        unmatched.len()
    );

    Ok(DetectUnmatchedFilesResult {
        checked: disk.len(),
        unmatched: unmatched.len(),
        sample: unmatched.iter().take(SAMPLE_LIMIT).cloned().collect(),
    })
}


/// Drawable bounds in millimetres, matching the preview pipeline.
fn drawable_bounds_mm(pattern: &EmbPattern) -> Option<(f64, f64)> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut found = false;

    for stitch in &pattern.stitches {
        if stitch.stitch_type != StitchType::Stitch {
            continue;
        }
        found = true;
        if stitch.x < min_x {
            min_x = stitch.x;
        }
        if stitch.x > max_x {
            max_x = stitch.x;
        }
        if stitch.y < min_y {
            min_y = stitch.y;
        }
        if stitch.y > max_y {
            max_y = stitch.y;
        }
    }

    if found {
        Some((
            ((max_x - min_x) / 10.0) as f64,
            ((max_y - min_y) / 10.0) as f64,
        ))
    } else {
        None
    }
}

/// Parse an embroidery buffer into a pattern, matching the preview pipeline.
fn parse_pattern(extension: &str, data: &[u8]) -> Result<EmbPattern, String> {
    match extension.to_ascii_lowercase().as_str() {
        "pes" => PesReader.read(data).map_err(|e| e.to_string()),
        "dst" => DstReader.read(data).map_err(|e| e.to_string()),
        "exp" => ExpReader.read(data).map_err(|e| e.to_string()),
        "jef" => JefReader.read(data).map_err(|e| e.to_string()),
        "hus" => HusReader.read(data).map_err(|e| e.to_string()),
        "vp3" => Vp3Reader.read(data).map_err(|e| e.to_string()),
        other => Err(format!("Unsupported embroidery extension '.{other}'")),
    }
}

/// Insert a single unmatched design file as a new catalogue record.
async fn import_single_design(
    pool: &SqlitePool,
    relative: &Path,
    full_path: &Path,
) -> Result<bool, String> {
    let extension = full_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    let supported = matches!(
        extension.as_deref(),
        Some("pes" | "dst" | "exp" | "jef" | "hus" | "vp3")
    );
    if !supported {
        return Ok(false);
    }

    let data = fs::read(full_path)
        .map_err(|e| format!("Could not read '{}': {e}", full_path.display()))?;
    let pattern = parse_pattern(extension.as_deref().unwrap_or(""), &data)?;

    let stitch_count = i64::try_from(pattern.count_stitches()).unwrap_or(i64::MAX);
    let color_count = i64::try_from(pattern.count_distinct_thread_colors()).unwrap_or(i64::MAX);
    let color_change_count = i64::try_from(pattern.count_color_changes()).unwrap_or(i64::MAX);
    let (width_mm, height_mm) = match drawable_bounds_mm(&pattern) {
        Some((width, height)) => (Some(width), Some(height)),
        None => (None, None),
    };

    let file_size_bytes = crate::routes::bulk_import::compute_file_size(full_path).ok();
    let file_hash_blake3 = crate::routes::bulk_import::compute_file_hash_blake3(full_path).ok();

    let filename = relative
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| relative.to_string_lossy().to_string());
    let stored_filepath = crate::paths::canonical_design_rel(&relative.to_string_lossy());

    sqlx::query(
        "INSERT INTO designs (filename, filepath, date_added, width_mm, height_mm, \
         stitch_count, color_count, color_change_count, is_stitched, \
         image_tags_verified, stitching_tags_verified, file_size_bytes, file_hash_blake3) \
         VALUES (?, ?, DATE('now'), ?, ?, ?, ?, ?, 0, 0, 0, ?, ?)",
    )
    .bind(&filename)
    .bind(&stored_filepath)
    .bind(width_mm)
    .bind(height_mm)
    .bind(stitch_count)
    .bind(color_count)
    .bind(color_change_count)
    .bind(file_size_bytes)
    .bind(file_hash_blake3)
    .execute(pool)
    .await
    .map_err(|e| format!("Could not insert '{}': {e}", full_path.display()))?;

    Ok(true)
}

/// Batch import of unmatched design files as new catalogue records.
pub async fn import_unmatched_design_files(
    pool: &SqlitePool,
    designs_root: &Path,
) -> Result<ImportUnmatchedFilesResult, String> {
    let disk = mnt::collect_file_snapshots(designs_root, true)?;
    let referenced = referenced_design_paths(pool, designs_root).await?;

    let mut unmatched: Vec<PathBuf> = disk
        .keys()
        .filter(|relative| !referenced.contains(*relative))
        .cloned()
        .collect();
    unmatched.sort();

    let detected = unmatched.len();
    let mut imported = 0usize;
    let mut failed = 0usize;
    let mut failed_samples = Vec::new();

    for relative in unmatched {
        let full_path = designs_root.join(&relative);
        match import_single_design(pool, &relative, &full_path).await {
            Ok(true) => imported += 1,
            Ok(false) => {}
            Err(error) => {
                failed += 1;
                if failed_samples.len() < SAMPLE_LIMIT {
                    failed_samples.push(error);
                }
            }
        }
    }

    if failed > 0 {
        tracing::error!(
            "[restore] unmatched-file import finished detected={} imported={} failed={} failed_samples={:?}",
            detected,
            imported,
            failed,
            failed_samples,
        );
    } else {
        tracing::info!(
            "[restore] unmatched-file import finished detected={} imported={} failed={}",
            detected,
            imported,
            failed,
        );
    }

    Ok(ImportUnmatchedFilesResult {
        detected,
        imported,
        failed,
        failed_samples,
    })
}

#[cfg(test)]
#[path = "restore_tests.rs"]
mod tests;


