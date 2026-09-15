//! Database fragmentation health monitoring.
//!
//! Periodically measures the SQLite freelist ratio (free pages ÷ total pages)
//! and, when fragmentation exceeds configurable thresholds, schedules a
//! non-blocking incremental vacuum in the background. The UI is notified via
//! Tauri events (`db-maintenance-started` / `db-maintenance-finished`) so it
//! can surface a lightweight, non-intrusive notification.
//!
//! Because `PRAGMA incremental_vacuum(N)` performs short, sub-second write
//! transactions and the compaction routine yields the shared connection
//! between batches, the user can continue using the application normally
//! while maintenance runs — no blocking overlay or nav disabling is required.

use crate::services::compaction::{read_page_size, run_incremental_vacuum};
use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::Emitter;

/// Default free-page ratio threshold (freelist ÷ page_count) above which
/// fragmentation maintenance is considered worthwhile. 20% of the database
/// file being free space is a reasonable, conservative trigger.
pub const DEFAULT_FREE_RATIO_THRESHOLD: f64 = 0.20;

/// Default minimum reclaimable space (bytes) before maintenance triggers.
/// Prevents running compaction for trivial freelists (e.g. a few pages after
/// a single delete). 20 MiB ≈ 5,120 pages at a 4 KiB page size.
pub const DEFAULT_MIN_RECLAIMABLE_BYTES: u64 = 20 * 1024 * 1024;

/// Pages to reclaim per incremental vacuum step during a health-check-triggered
/// run. 1000 pages ≈ 4 MiB at 4 KiB page size.
pub const MAINTENANCE_PAGE_STEP: i64 = 1000;

/// Default interval (seconds) between idle-time health checks.
pub const DEFAULT_IDLE_CHECK_INTERVAL_SECS: u64 = 1800;

/// Tauri event names broadcast to the frontend.
pub const EVENT_MAINTENANCE_STARTED: &str = "db-maintenance-started";
pub const EVENT_MAINTENANCE_FINISHED: &str = "db-maintenance-finished";

/// Snapshot of the database's page and freelist counts.
#[derive(Debug, Clone, Serialize)]
pub struct FreelistMetrics {
    pub page_count: i64,
    pub freelist_count: i64,
}

/// Payload broadcast when maintenance begins.
#[derive(Debug, Clone, Serialize)]
pub struct DbMaintenanceStartedEvent {
    pub page_count: i64,
    pub freelist_pages: i64,
    pub free_ratio: f64,
    pub reclaimable_bytes: u64,
}

/// Payload broadcast when maintenance completes.
#[derive(Debug, Clone, Serialize)]
pub struct DbMaintenanceFinishedEvent {
    pub reclaimed_pages: u64,
    pub reclaimable_bytes_before: u64,
    pub reclaimable_bytes_after: u64,
    pub duration_ms: u64,
}

/// Read the current page count, freelist count, and page size from the pool.
pub async fn get_freelist_metrics(pool: &SqlitePool) -> Result<MetricsSnapshot, String> {
    let (page_count,): (i64,) = sqlx::query_as("SELECT page_count FROM pragma_page_count")
        .fetch_one(pool)
        .await
        .map_err(|err| format!("Failed to read page_count: {err}"))?;

    let (freelist_count,): (i64,) =
        sqlx::query_as("SELECT freelist_count FROM pragma_freelist_count")
            .fetch_one(pool)
            .await
            .map_err(|err| format!("Failed to read freelist_count: {err}"))?;

    let page_size = read_page_size(pool).await?;

    Ok(MetricsSnapshot {
        page_count,
        freelist_count,
        page_size,
    })
}

/// A full metrics snapshot including page size for reclaimable-byte math.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub page_count: i64,
    pub freelist_count: i64,
    pub page_size: i64,
}

impl MetricsSnapshot {
    /// Free pages as a fraction of total pages (0.0..=1.0).
    pub fn free_ratio(&self) -> f64 {
        if self.page_count <= 0 {
            return 0.0;
        }
        (self.freelist_count as f64) / (self.page_count as f64)
    }

    /// Estimated reclaimable space in bytes if the freelist were fully reclaimed.
    pub fn reclaimable_bytes(&self) -> u64 {
        (self.freelist_count.max(0) as u64).saturating_mul(self.page_size.max(0) as u64)
    }
}

/// Pure threshold check: should maintenance run for this fragmentation state?
///
/// Returns true only when:
///  1. There is at least one free page, AND
///  2. Free pages make up at least `ratio_threshold` (as a fraction) of total
///     pages, AND
///  3. Reclaimable space (freelist × page size) is at least `min_reclaimable_bytes`.
///
/// Pure and unit-testable without touching a database.
pub fn should_run_maintenance(
    page_count: i64,
    freelist_count: i64,
    page_size_bytes: i64,
    ratio_threshold: f64,
    min_reclaimable_bytes: u64,
) -> bool {
    if page_count <= 0 || freelist_count <= 0 || page_size_bytes <= 0 {
        return false;
    }
    let ratio = (freelist_count as f64) / (page_count as f64);
    let reclaimable = (freelist_count as u64).saturating_mul(page_size_bytes as u64);
    ratio >= ratio_threshold && reclaimable >= min_reclaimable_bytes
}

/// Orchestrator: measure fragmentation and, if the threshold is exceeded,
/// spawn a non-blocking background compaction and emit lifecycle events.
///
/// Returns `Ok(true)` if maintenance was scheduled, `Ok(false)` if the
/// threshold was not met (or a run is already in progress). Errors are
/// returned only for genuine measurement/emit failures; the compaction task
/// itself logs errors and never propagates.
pub async fn check_and_schedule_maintenance<R: tauri::Runtime>(
    pool: SqlitePool,
    maintenance_running: Arc<AtomicBool>,
    shutdown_requested: Arc<AtomicBool>,
    app_handle: tauri::AppHandle<R>,
) -> Result<bool, String> {
    // Refuse to start if a run is already in progress.

    if maintenance_running.load(Ordering::SeqCst) {
        tracing::info!("DB health check skipped — maintenance already running");
        return Ok(false);
    }

    let snapshot = get_freelist_metrics(&pool).await?;
    let should_run = should_run_maintenance(
        snapshot.page_count,
        snapshot.freelist_count,
        snapshot.page_size,
        DEFAULT_FREE_RATIO_THRESHOLD,
        DEFAULT_MIN_RECLAIMABLE_BYTES,
    );

    tracing::info!(
        "DB health check — page_count={}, freelist_count={}, free_ratio={:.3}, \
         reclaimable_bytes={}",
        snapshot.page_count,
        snapshot.freelist_count,
        snapshot.free_ratio(),
        snapshot.reclaimable_bytes()
    );

    if !should_run {
        return Ok(false);
    }

    // Claim the maintenance slot.
    if maintenance_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        tracing::info!("DB health check — lost race, another run is already active");
        return Ok(false);
    }

    let started_event = DbMaintenanceStartedEvent {
        page_count: snapshot.page_count,
        freelist_pages: snapshot.freelist_count,
        free_ratio: snapshot.free_ratio(),
        reclaimable_bytes: snapshot.reclaimable_bytes(),
    };
    let _ = app_handle.emit(EVENT_MAINTENANCE_STARTED, &started_event);

    let reclaimable_before = snapshot.reclaimable_bytes();

    // Spawn the actual compaction so the caller returns immediately.
    let pool_for_task = pool.clone();
    let running_for_task = maintenance_running.clone();
    let shutdown_for_task = shutdown_requested.clone();
    let handle_for_task = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        let started = Instant::now();
        let reclaimed = run_incremental_vacuum(&pool_for_task, MAINTENANCE_PAGE_STEP).await;

        let reclaimed_pages = match reclaimed {
            Ok(pages) => pages,
            Err(err) => {
                tracing::warn!(
                    "DB maintenance compaction failed (defers to next idle check): {}",
                    err
                );
                running_for_task.store(false, Ordering::SeqCst);
                return;
            }
        };

        // Re-measure for the finished event.
        let reclaimable_after = get_freelist_metrics(&pool_for_task)
            .await
            .map(|s| s.reclaimable_bytes())
            .unwrap_or(reclaimable_before);

        let finished_event = DbMaintenanceFinishedEvent {
            reclaimed_pages,
            reclaimable_bytes_before: reclaimable_before,
            reclaimable_bytes_after: reclaimable_after,
            duration_ms: started.elapsed().as_millis() as u64,
        };

        let _ = handle_for_task.emit(EVENT_MAINTENANCE_FINISHED, &finished_event);

        tracing::info!(
            "DB maintenance finished — reclaimed_pages={}, reclaimable_bytes_before={}, \
             reclaimable_bytes_after={}, duration_ms={}",
            finished_event.reclaimed_pages,
            finished_event.reclaimable_bytes_before,
            finished_event.reclaimable_bytes_after,
            finished_event.duration_ms
        );

        // Release the maintenance slot. Check shutdown so we don't log after exit.
        if shutdown_for_task.load(Ordering::SeqCst) {
            tracing::info!("DB maintenance complete; shutdown requested.");
        }
        running_for_task.store(false, Ordering::SeqCst);
    });

    Ok(true)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
#[path = "db_health_tests.rs"]
mod tests;
