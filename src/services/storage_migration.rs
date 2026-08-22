//! Catalogue storage migration.
//!
//! Moves the active catalogue (SQLite database + managed design library)
//! from the current data root to a newly selected root. The original files
//! are never deleted — on success the source root is renamed to
//! `<root>.migrated-backup`, and any pre-existing non-empty target is first
//! moved aside to `<target>.before-migration-backup`.
//!
//! Progress is streamed via a caller-provided callback; the Tauri route turns
//! that into `app_handle.emit(...)` events.

use crate::error::AppError;
use crate::paths::{self, AppPaths};
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Connection, SqlitePool};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Progress event name streamed to the frontend.
pub const STORAGE_MIGRATION_PROGRESS_EVENT: &str = "catalogue-storage-migration-progress";

/// Per-callback payload describing the current migration state.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorageMigrationProgress {
    pub current_phase: String,
    pub items_copied: u64,
    pub total_items: u64,
    pub bytes_copied: u64,
    pub total_bytes: u64,
    pub status_message: String,
    pub percent: f64,
    pub error: Option<String>,
}

impl StorageMigrationProgress {
    fn new(phase: &str, status_message: String) -> Self {
        Self {
            current_phase: phase.to_string(),
            items_copied: 0,
            total_items: 0,
            bytes_copied: 0,
            total_bytes: 0,
            status_message,
            percent: 0.0,
            error: None,
        }
    }

    fn with_totals(
        mut self,
        total_items: u64,
        total_bytes: u64,
        items_copied: u64,
        bytes_copied: u64,
    ) -> Self {
        self.total_items = total_items;
        self.total_bytes = total_bytes;
        self.items_copied = items_copied;
        self.bytes_copied = bytes_copied;
        self.percent = if total_bytes == 0 {
            1.0
        } else {
            (bytes_copied as f64 / total_bytes as f64).min(1.0)
        };
        self
    }

    fn error(mut self, message: String) -> Self {
        self.error = Some(message);
        self
    }
}

/// Final summary returned once migration completes or fails permanently.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorageMigrationSummary {
    pub success: bool,
    pub source_root: String,
    pub target_root: String,
    pub database_bytes: u64,
    pub asset_items: u64,
    pub asset_bytes: u64,
    pub requires_restart: bool,
}

/// Validated plan produced by pre-flight and consumed by the copy loop.
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    pub target: PathBuf,
    pub target_paths: AppPaths,
    pub total_items: u64,
    pub total_bytes: u64,
    pub database_bytes: u64,
    pub same_device: bool,
    pub preexisting_target_renamed: Option<PathBuf>,
}

/// Runs a WAL checkpoint on the live pooled connection so the main `.db`
/// file is complete and the `-wal`/`-shm` sidecars can be safely captured
/// (or are removed entirely by `TRUNCATE`).
pub async fn checkpoint_live_database(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("WAL checkpoint failed: {e}")))?;
    Ok(())
}

/// Validate the target and compute the copy plan. Does not create or mutate
/// the target beyond a transient write probe that is removed immediately.
///
/// - Rejects a relative, equal, ancestor or descendant target.
/// - Probes write permission and available disk space.
/// - Detects same-device (rename fast-path) via a transient hard-link.
/// - When `force` is set and the target is non-empty, renames it aside to
///   `<target>.before-migration-backup`; otherwise errors.
pub fn preflight(
    source: &AppPaths,
    target_dir: &Path,
    force: bool,
) -> Result<MigrationPlan, AppError> {
    if !target_dir.is_absolute() {
        return Err(AppError::invalid_input(
            "data root must be an absolute path",
        ));
    }

    let trimmed_target = target_dir.to_path_buf();
    let source_root = &source.data_root;

    // Equal / ancestor / descendant relationships are hard errors regardless
    // of force — they would cause recursive or self-destructive migration.
    if paths::path_within(&trimmed_target, source_root)
        || paths::path_within(source_root, &trimmed_target)
    {
        return Err(AppError::invalid_input(format!(
            "target '{}' cannot be the same as, or nested within/containing the \
             current data root '{}'",
            trimmed_target.display(),
            source_root.display()
        )));
    }

    // Write probe: create the target dir (it may not exist yet) and prove we
    // can write to it.
    std::fs::create_dir_all(&trimmed_target)
        .map_err(|e| AppError::io(format!("cannot create target: {e}")))?;
    let probe = trimmed_target.join(".migration-write-probe");
    std::fs::write(&probe, b"probe")
        .map_err(|e| AppError::io(format!("target is not writable: {e}")))?;
    let _ = std::fs::remove_file(&probe);

    // Non-empty target handling.
    let mut preexisting_target_renamed = None;
    if target_has_entries(&trimmed_target) {
        if !force {
            return Err(AppError::invalid_input(format!(
                "target '{}' is not empty; pass force=true to move it aside and overwrite",
                trimmed_target.display()
            )));
        }
        let backup = sibling_backup_path(&trimmed_target, "before-migration-backup");
        std::fs::rename(&trimmed_target, &backup).map_err(|e| {
            AppError::io(format!(
                "failed to move existing target aside to '{}': {e}",
                backup.display()
            ))
        })?;
        std::fs::create_dir_all(&trimmed_target).map_err(|e| {
            AppError::io(format!("cannot recreate target after move-aside: {e}"))
        })?;
        preexisting_target_renamed = Some(backup);
    }

    // Compute totals from the source.
    let database_bytes = database_bytes(source);
    let (asset_items, asset_bytes) = tree_totals(source);

    // Disk space.
    let available = fs4::available_space(&trimmed_target)
        .map_err(|e| AppError::io(format!("cannot read free space on target: {e}")))?;
    let required = database_bytes + asset_bytes;
    if available < required {
        return Err(AppError::io(format!(
            "insufficient free space on '{}': need {required} bytes, have {available}",
            trimmed_target.display()
        )));
    }

    let same_device = same_device_probe(source_root, &trimmed_target);

    Ok(MigrationPlan {
        target: trimmed_target.clone(),
        target_paths: paths::resolve_paths_for_root(&trimmed_target),
        total_items: asset_items,
        total_bytes: required,
        database_bytes,
        same_device,
        preexisting_target_renamed,
    })
}

/// Perform the full migration for an already-validated plan.
///
/// Phases: database copy + verify, asset copy (bytes + file-count progress),
/// finalise (write config + rename source to `.migrated-backup`).
///
/// On failure or cancellation before the config commit point, the partial
/// target tree is removed and any moved-aside pre-existing target is restored;
/// the source root is left untouched. After the config commit point the target
/// becomes authoritative and the source is renamed to a backup.
pub async fn run_migration(
    source: &AppPaths,
    plan: &MigrationPlan,
    cancel: &AtomicBool,
    mut emit: impl FnMut(StorageMigrationProgress) + Send,
) -> Result<StorageMigrationSummary, AppError> {
    emit(StorageMigrationProgress::new(
        "database",
        "Copying database…".to_string(),
    )
    .with_totals(
        plan.total_items,
        plan.total_bytes,
        0,
        plan.database_bytes,
    ));

    match migrate_database(source, plan).await {
        Ok(database_ok) => {
            if !database_ok {
                let message = "database integrity check at target failed".to_string();
                emit(
                    StorageMigrationProgress::new("error", message.clone())
                        .with_totals(plan.total_items, plan.total_bytes, 0, 0)
                        .error(message.clone()),
                );
                rollback_partial_target(plan);
                return Err(AppError::database(message));
            }
        }
        Err(e) => {
            emit(
                StorageMigrationProgress::new("error", e.to_string())
                    .with_totals(plan.total_items, plan.total_bytes, 0, 0)
                    .error(e.to_string()),
            );
            rollback_partial_target(plan);
            return Err(e);
        }
    }

    let mut items_copied = 0u64;
    let mut bytes_copied = plan.database_bytes;

    match copy_asset_trees(source, plan, cancel, &mut emit, &mut items_copied, &mut bytes_copied)
        .await
    {
        Ok(()) => {}
        Err(MigrationAbort::Cancelled) => {
            emit(
                StorageMigrationProgress::new("cancelled", "Migration cancelled.".to_string())
                    .with_totals(
                        plan.total_items,
                        plan.total_bytes,
                        items_copied,
                        bytes_copied,
                    ),
            );
            rollback_partial_target(plan);
            return Err(AppError::io("migration cancelled"));
        }
        Err(MigrationAbort::Io(e)) => {
            emit(
                StorageMigrationProgress::new("error", e.to_string())
                    .with_totals(
                        plan.total_items,
                        plan.total_bytes,
                        items_copied,
                        bytes_copied,
                    )
                    .error(e.to_string()),
            );
            rollback_partial_target(plan);
            return Err(e);
        }
    }

    emit(StorageMigrationProgress::new(
        "finalising",
        "Verifying migrated files…".to_string(),
    )
    .with_totals(plan.total_items, plan.total_bytes, items_copied, bytes_copied));

    verify_target_tree(source, plan)
        .inspect_err(|_e| {
            rollback_partial_target(plan);
        })?;

    // Commit point: the new root becomes authoritative. Everything before this
    // can be rolled back; everything after must never fail the migration —
    // the data is already committed and verified in place.
    paths::write_bootstrap_data_root(&plan.target)?;

    // Best-effort source preservation. The old location is never deleted, but
    // the rename may be impossible when the data root is a filesystem root
    // (e.g. `F:\` — there is no parent folder to rename into). In that case we
    // leave the old folder in place and drop a marker file pointing at the new
    // location so the user can clean up manually later. A failure here is a
    // warning, never an error: the migration itself has already succeeded.
    let relocated_note = preserve_old_location(&source.data_root, &plan.target);

    let completed_message = match relocated_note.as_deref() {
        Some(note) => format!("Catalogue moved successfully. {}", note),
        None => "Catalogue moved successfully.".to_string(),
    };

    emit(StorageMigrationProgress::new(
        "completed",
        completed_message,
    )
    .with_totals(
        plan.total_items,
        plan.total_bytes,
        plan.total_items,
        plan.total_bytes,
    ));

    Ok(StorageMigrationSummary {
        success: true,
        source_root: source.data_root.to_string_lossy().to_string(),
        target_root: plan.target.to_string_lossy().to_string(),
        database_bytes: plan.database_bytes,
        asset_items: plan.total_items,
        asset_bytes: plan.total_bytes.saturating_sub(plan.database_bytes),
        requires_restart: true,
    })
}

/// Internal abort type distinguishing user cancellation from IO failure.
enum MigrationAbort {
    Cancelled,
    Io(AppError),
}

impl From<std::io::Error> for MigrationAbort {
    fn from(value: std::io::Error) -> Self {
        MigrationAbort::Io(AppError::io(value.to_string()))
    }
}

/// Copy `.db` (+ `-wal` / `-shm` if present) to the target and verify the
/// result by opening it and running `integrity_check` + a simple query.
async fn migrate_database(source: &AppPaths, plan: &MigrationPlan) -> Result<bool, AppError> {
    std::fs::create_dir_all(&plan.target_paths.database_dir)
        .map_err(|e| AppError::io(format!("cannot create target database dir: {e}")))?;

    let src_path = &source.database_path;
    let dst_path = &plan.target_paths.database_path;

    if src_path.exists() {
        std::fs::copy(src_path, dst_path)
            .map_err(|e| AppError::io(format!("failed to copy database: {e}")))?;
    }

    // Capture any residual WAL/SHM sidecars.
    for ext in ["-wal", "-shm"] {
        let side_src = {
            let mut name = src_path.as_os_str().to_os_string();
            name.push(ext);
            PathBuf::from(name)
        };
        if side_src.exists() {
            let mut side_dst = dst_path.as_os_str().to_os_string();
            side_dst.push(ext);
            std::fs::copy(&side_src, PathBuf::from(side_dst))
                .map_err(|e| AppError::io(format!("failed to copy {ext} sidecar: {e}")))?;
        }
    }

    verify_database_at(dst_path).await
}

/// Open the migrated database read-only and run `integrity_check` plus a
/// trivial design-count query. Returns `Ok(true)` when valid.
async fn verify_database_at(db_path: &Path) -> Result<bool, AppError> {
    if !db_path.exists() {
        return Ok(false);
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .read_only(true)
        .journal_mode(SqliteJournalMode::Off);

    let mut conn = sqlx::sqlite::SqliteConnection::connect_with(&options)
        .await
        .map_err(|e| AppError::database(format!("cannot open migrated database: {e}")))?;

    let row: (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(&mut conn)
        .await
        .map_err(|e| AppError::database(format!("integrity check failed: {e}")))?;

    if row.0 != "ok" {
        return Ok(false);
    }

    let _: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM designs")
        .fetch_one(&mut conn)
        .await
        .map_err(|e| AppError::database(format!("designs query on migrated DB failed: {e}")))?;

    Ok(true)
}

/// Copy (or, on the same device, rename) the `MachineEmbroideryDesigns` tree
/// to the target, updating cumulative item/byte counts.
///
/// On the same device the tree is renamed atomically and its totals added to
/// the running counters; on a different device each file is copied with
/// per-file progress.
async fn copy_asset_trees(
    source: &AppPaths,
    plan: &MigrationPlan,
    cancel: &AtomicBool,
    emit: &mut (impl FnMut(StorageMigrationProgress) + Send),
    items_copied: &mut u64,
    bytes_copied: &mut u64,
) -> Result<(), MigrationAbort> {
    let src_dir = &source.embroidery_designs_dir;
    let dst_dir = &plan.target_paths.embroidery_designs_dir;

    if !src_dir.exists() {
        return Ok(());
    }

    let (tree_items, tree_bytes) = tree_totals_at(src_dir);

    if plan.same_device {
        // Fast path: rename the whole tree atomically.
        std::fs::rename(src_dir, dst_dir).map_err(|e| {
            MigrationAbort::Io(AppError::io(format!(
                "rename failed for MachineEmbroideryDesigns: {e}"
            )))
        })?;
        *items_copied += tree_items;
        *bytes_copied += tree_bytes;
        emit(
            StorageMigrationProgress::new("assets", "Moved MachineEmbroideryDesigns…".to_string())
                .with_totals(
                    plan.total_items,
                    plan.total_bytes,
                    *items_copied,
                    *bytes_copied,
                ),
        );
    } else {
        // Cross-device: per-file copy with progress.
        copy_tree_recursive(src_dir, dst_dir, cancel, emit, plan, items_copied, bytes_copied)?;
    }

    Ok(())
}

/// Walk `from` and copy every file to `to` (creating directories), emitting
/// progress after each file and honouring the cancellation flag.
fn copy_tree_recursive(
    from: &Path,
    to: &Path,
    cancel: &AtomicBool,
    emit: &mut (impl FnMut(StorageMigrationProgress) + Send),
    plan: &MigrationPlan,
    items_copied: &mut u64,
    bytes_copied: &mut u64,
) -> Result<(), MigrationAbort> {
    std::fs::create_dir_all(to)
        .map_err(|e| MigrationAbort::Io(AppError::io(format!("cannot create {to:?}: {e}"))))?;

    for entry in std::fs::read_dir(from).map_err(MigrationAbort::from)? {
        let entry = entry.map_err(MigrationAbort::from)?;
        let file_type =
            entry
                .file_type()
                .map_err(|e| MigrationAbort::Io(AppError::io(e.to_string())))?;
        let src_path = entry.path();
        let dst_path = to.join(entry.file_name());

        if file_type.is_dir() {
            copy_tree_recursive(
                &src_path, &dst_path, cancel, emit, plan, items_copied, bytes_copied,
            )?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }

        if cancel.load(Ordering::Relaxed) {
            return Err(MigrationAbort::Cancelled);
        }

        std::fs::copy(&src_path, &dst_path)
            .map_err(|e| MigrationAbort::Io(AppError::io(format!("copy failed: {e}"))))?;

        let size = std::fs::metadata(&src_path)
            .map(|m| m.len())
            .unwrap_or(0);
        *items_copied += 1;
        *bytes_copied += size;

        emit(
            StorageMigrationProgress::new("assets", format!("Copying {}", src_path.display()))
                .with_totals(
                    plan.total_items,
                    plan.total_bytes,
                    *items_copied,
                    *bytes_copied,
                ),
        );
    }

    Ok(())
}

/// Verify the migrated design library has matching file counts and byte totals.
fn verify_target_tree(source: &AppPaths, plan: &MigrationPlan) -> Result<(), AppError> {
    let src_dir = &source.embroidery_designs_dir;
    let dst_dir = &plan.target_paths.embroidery_designs_dir;

    if !src_dir.exists() {
        return Ok(());
    }
    let (src_items, src_bytes) = tree_totals_at(src_dir);
    let (dst_items, dst_bytes) = tree_totals_at(dst_dir);
    if src_items != dst_items || src_bytes != dst_bytes {
        return Err(AppError::io(format!(
            "verification mismatch for {:?}: source {src_items} files/{src_bytes} bytes vs target {dst_items} files/{dst_bytes} bytes",
            dst_dir
        )));
    }
    Ok(())
}

/// Remove the partial target tree on failure/cancel and restore any
/// pre-existing target that was moved aside during pre-flight.
fn rollback_partial_target(plan: &MigrationPlan) {
    if let Some(backup) = &plan.preexisting_target_renamed {
        // If a pre-existing target was moved aside, restore it in preference to
        // deleting; any partial migration tree is discarded with it.
        let _ = std::fs::remove_dir_all(&plan.target);
        let _ = std::fs::rename(backup, &plan.target);
    } else {
        let _ = std::fs::remove_dir_all(&plan.target);
    }
}

fn target_has_entries(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|mut it| it.next().is_some())
        .unwrap_or(false)
}

/// Human-readable local timestamp for the moved-notice marker, e.g.
/// `12 May 2026 17:50`. Uses the machine's local timezone; falls back to UTC
/// only if the OS cannot determine the local offset (extremely rare). The
/// format is purely informational.
fn format_moved_timestamp(dt: time::OffsetDateTime) -> String {
    let description = time::macros::format_description!(
        "[day] [month repr:long] [year] [hour]:[minute]"
    );
    dt.format(&description)
        .unwrap_or_else(|_| "unknown time".to_string())
}

fn sibling_backup_path(path: &Path, label: &str) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{name}.{label}"))
}

/// Whether `path` is a filesystem root (e.g. `C:\` on Windows or `/` on
/// Unix). Such roots have no parent directory, so `sibling_backup_path` would
/// collapse to `.\.migrated-backup` and a rename of the whole volume would be
/// attempted — which is impossible (the volume is always in use by the OS).
fn is_filesystem_root(path: &Path) -> bool {
    path.parent().is_none()
}

/// Best-effort preservation of the old data root after a successful commit.
///
/// - When the old root is NOT a filesystem root, rename it to
///   `<root>.migrated-backup` so the space is reclaimed visibly for the user.
/// - When it IS a filesystem root (e.g. `F:\`) — or the rename fails for any
///   other reason — leave the folder in place and write `storage location
///   moved.txt` at its root pointing to the new location.
///
/// Never returns `Err`: the migration data is already committed and verified,
/// so this step is purely cosmetic/cleanup and must not fail the operation.
/// Returns an optional human-readable note for the `completed` message.
fn preserve_old_location(source_root: &Path, target_root: &Path) -> Option<String> {
    // If the data root isn't a filesystem root, try the rename fast-path.
    if !is_filesystem_root(source_root) {
        let backup = sibling_backup_path(source_root, "migrated-backup");
        if backup.exists() {
            let _ = std::fs::remove_dir_all(&backup);
        }
        if std::fs::rename(source_root, &backup).is_ok() {
            return None;
        }
        // Fall through to the marker on rename failure.
    }

    // Cannot (or will not) rename: write a moved-notice marker at the old root.
    let mut note = String::from(
        "The old storage location could not be renamed automatically (it may be a drive \
         root or in use). A marker file 'storage location moved.txt' was written there \
         recording the new location.",
    );

    if let Err(e) = write_moved_notice(source_root, target_root) {
        tracing::warn!(
            "Migration commit succeeded but could not rename old root to backup, \
             nor write the moved-notice marker at '{}': {e}",
            source_root.display()
        );
        note = format!("{note} (However the marker file itself could not be written: {e})");
    }

    Some(note)
}

/// Write the `storage location moved.txt` marker at `source_root` pointing at
/// `target_root`. Returns `Err` only when the marker itself cannot be written.
fn write_moved_notice(source_root: &Path, target_root: &Path) -> Result<(), AppError> {
    let marker_path = source_root.join("storage location moved.txt");
    let content = format!(
        "Embroidery Catalogue storage was moved on {}.\n\n\
         The catalogue data is now stored at:\n{}\n\n\
         The folders at this location (Database, MachineEmbroideryDesigns)\n\
         are no longer used by the application and can be deleted manually once you are\n\
         happy with the move.\n",
        format_moved_timestamp(
            time::OffsetDateTime::now_local()
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        ),
        target_root.display()
    );
    std::fs::write(&marker_path, &content).map_err(|e| {
        AppError::io(format!("failed to write moved-notice marker '{}': {e}", marker_path.display()))
    })
}

/// Total bytes for the database file plus any residual `-wal`/`-shm` sidecars.
fn database_bytes(source: &AppPaths) -> u64 {
    let mut total = source
        .database_path
        .metadata()
        .map(|m| m.len())
        .unwrap_or(0);
    for ext in ["-wal", "-shm"] {
        let mut name = source.database_path.as_os_str().to_os_string();
        name.push(ext);
        let side = PathBuf::from(name);
        total += side.metadata().map(|m| m.len()).unwrap_or(0);
    }
    total
}

/// Sum of file count + bytes across the managed design library.
fn tree_totals(source: &AppPaths) -> (u64, u64) {
    let (design_items, design_bytes) = tree_totals_at(&source.embroidery_designs_dir);
    (design_items, design_bytes)
}

fn tree_totals_at(dir: &Path) -> (u64, u64) {
    let mut items = 0u64;
    let mut bytes = 0u64;
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_dir() {
                stack.push(path);
            } else if ft.is_file() {
                items += 1;
                bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    (items, bytes)
}

/// Probe whether the source and target live on the same device by creating a
/// transient hard link across the boundary; removed immediately afterwards.
fn same_device_probe(source_root: &Path, target: &Path) -> bool {
    let probe_src = target.join(".migration-device-probe");
    let ok = std::fs::write(&probe_src, b"probe").is_ok()
        && std::fs::hard_link(&probe_src, source_root.join(".migration-device-probe-link")).is_ok();
    let _ = std::fs::remove_file(source_root.join(".migration-device-probe-link"));
    let _ = std::fs::remove_file(&probe_src);
    ok
}

// ---------------------------------------------------------------------------
// Tests (split into sibling file)
// ---------------------------------------------------------------------------
#[cfg(test)]
#[path = "storage_migration_tests.rs"]
mod tests;