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
pub async fn check_and_schedule_maintenance(
    pool: SqlitePool,
    maintenance_running: Arc<AtomicBool>,
    shutdown_requested: Arc<AtomicBool>,
    app_handle: tauri::AppHandle,
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
mod tests {
    use super::*;

    // ─── should_run_maintenance ─────────────────────────────────────────────

    #[test]
    fn no_maintenance_when_zero_freelist() {
        assert!(!should_run_maintenance(
            10_000,
            0,
            4096,
            0.20,
            20 * 1024 * 1024
        ));
    }

    #[test]
    fn no_maintenance_when_zero_total_pages() {
        assert!(!should_run_maintenance(
            0,
            100,
            4096,
            0.20,
            20 * 1024 * 1024
        ));
    }

    #[test]
    fn no_maintenance_when_zero_page_size() {
        assert!(!should_run_maintenance(10_000, 5000, 0, 0.20, 0));
    }

    #[test]
    fn no_maintenance_below_ratio_threshold() {
        // 1000 free / 10000 total = 10% < 20%, even though > 20MB of free space.
        assert!(!should_run_maintenance(
            10_000,
            1_000,
            4096,
            0.20,
            20 * 1024 * 1024
        ));
    }

    #[test]
    fn no_maintenance_below_byte_floor() {
        // 20% free, but only 100 pages * 4096 = ~0.4 MB < 20 MB.
        assert!(!should_run_maintenance(
            500,
            100,
            4096,
            0.20,
            20 * 1024 * 1024
        ));
    }

    #[test]
    fn maintenance_at_exact_threshold_triggers() {
        // 2000 free / 10000 total = exactly 20%, and 2000*4096 ≈ 8 MB < 20 MB.
        // Byte floor prevents this — bump page size so bytes also pass.
        assert!(should_run_maintenance(
            10_000,
            2_000,
            4096,
            0.20,
            2_000 * 4096 // exactly the reclaimable bytes floor
        ));
    }

    #[test]
    fn maintenance_above_ratio_and_bytes_triggers() {
        // 5,200 free / 10,000 total = 52% ≥ 20%, and 5,200 * 4096 ≈ 21.3 MB
        // ≥ 20 MB floor — both thresholds satisfied.
        assert!(should_run_maintenance(
            10_000,
            5_200,
            4096,
            0.20,
            20 * 1024 * 1024
        ));
    }

    #[test]
    fn zero_ratio_threshold_still_requires_positive_freelist() {
        // Even a 0% ratio threshold must not trigger on a zero freelist.
        assert!(!should_run_maintenance(10_000, 0, 4096, 0.0, 0));
    }

    #[test]
    fn high_ratio_with_low_bytes_does_not_trigger() {
        // 90% free but tiny file — under the byte floor.
        assert!(!should_run_maintenance(
            100,
            90,
            4096,
            0.20,
            20 * 1024 * 1024
        ));
    }

    // ─── MetricsSnapshot helpers ─────────────────────────────────────────────

    #[test]
    fn free_ratio_zero_when_page_count_zero() {
        let snapshot = MetricsSnapshot {
            page_count: 0,
            freelist_count: 50,
            page_size: 4096,
        };
        assert_eq!(snapshot.free_ratio(), 0.0);
    }

    #[test]
    fn free_ratio_calculated_correctly() {
        let snapshot = MetricsSnapshot {
            page_count: 100,
            freelist_count: 25,
            page_size: 4096,
        };
        assert!((snapshot.free_ratio() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn reclaimable_bytes_is_freelist_times_page_size() {
        let snapshot = MetricsSnapshot {
            page_count: 100,
            freelist_count: 25,
            page_size: 4096,
        };
        assert_eq!(snapshot.reclaimable_bytes(), 25 * 4096);
    }

    #[test]
    fn reclaimable_bytes_zero_when_freelist_negative() {
        let snapshot = MetricsSnapshot {
            page_count: 100,
            freelist_count: -5,
            page_size: 4096,
        };
        assert_eq!(snapshot.reclaimable_bytes(), 0);
    }

    // ─── get_freelist_metrics ───────────────────────────────────────────────

    /// Create an in-memory pool, mirroring the production setup used by the
    /// compaction service tests.
    async fn test_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create in-memory pool");

        sqlx::query("PRAGMA auto_vacuum = INCREMENTAL")
            .execute(&pool)
            .await
            .expect("set auto_vacuum");

        sqlx::query(
            "CREATE TABLE items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                payload TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create items table");

        pool
    }

    #[tokio::test]
    async fn get_freelist_metrics_returns_valid_snapshot_on_fresh_pool() {
        let pool = test_pool().await;

        let snapshot = get_freelist_metrics(&pool)
            .await
            .expect("metrics should be readable");

        assert!(snapshot.page_count > 0, "fresh DB has at least one page");
        assert_eq!(snapshot.freelist_count, 0, "fresh DB has no free pages");
        assert!(
            snapshot.page_size > 0,
            "page_size must be a positive number of bytes"
        );
        // 4096 is the common default; assert it is a sane power of two ≥ 512.
        let size = snapshot.page_size;
        let is_power_of_two = size > 0 && (size & (size - 1)) == 0;
        assert!(
            size >= 512 && is_power_of_two,
            "page_size should be a power of two ≥ 512, got {}",
            size
        );
    }

    #[tokio::test]
    async fn get_freelist_metrics_reflects_deleted_pages() {
        let pool = test_pool().await;

        // Insert large rows so each row occupies roughly one page, then delete
        // them all — this leaves pages on the SQLite freelist.
        let filler = "x".repeat(2048);
        for i in 0..200 {
            sqlx::query("INSERT INTO items (payload) VALUES (?)")
                .bind(format!("item-{i}-{filler}"))
                .execute(&pool)
                .await
                .expect("insert item");
        }
        sqlx::query("DELETE FROM items")
            .execute(&pool)
            .await
            .expect("delete all items");

        let snapshot = get_freelist_metrics(&pool)
            .await
            .expect("metrics should be readable");

        assert!(
            snapshot.freelist_count > 0,
            "expected freelist pages after bulk delete, got {}",
            snapshot.freelist_count
        );
        assert!(snapshot.free_ratio() > 0.0);
        assert!(
            snapshot.reclaimable_bytes() > 0,
            "reclaimable bytes should reflect the freelist"
        );
        // Sanity: reclaimable bytes ≈ freelist × page size.
        assert_eq!(
            snapshot.reclaimable_bytes(),
            (snapshot.freelist_count as u64).saturating_mul(snapshot.page_size as u64)
        );
    }

    #[tokio::test]
    async fn get_freelist_metrics_returns_error_when_connection_closed() {
        let pool = test_pool().await;
        pool.close().await;

        let result = get_freelist_metrics(&pool).await;
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("page_count"),
            "error should mention the failing pragma"
        );
    }
}
