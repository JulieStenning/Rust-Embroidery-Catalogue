//! Incremental database compaction.
//!
//! SQLite leaves deleted rows on the freelist rather than returning the space
//! to the operating system immediately. For a catalogue app that regularly
//! bulk-deletes designs and orphan records, this causes the database file to
//! grow without bound. With `PRAGMA auto_vacuum = INCREMENTAL`, the space can
//! be reclaimed incrementally with `PRAGMA incremental_vacuum(N)`.
//!
//! This module provides:
//!  - [`schedule_incremental_vacuum`]: a fire-and-forget background task used
//!    after bulk-delete commit paths so Tauri IPC handlers return immediately.
//!  - [`run_incremental_vacuum`]: the underlying routine, which steps through
//!    the freelist in small page batches, yielding between steps so the
//!    application's single SQLite connection can service UI queries.
//!
//! The connection pool is configured with `max_connections(1)`, so the
//! compaction task borrows the same shared connection between yields. Long
//! `incremental_vacuum` operations are deliberately avoided; each individual
//! `PRAGMA incremental_vacuum(N)` completes and releases its lock quickly.

use sqlx::SqlitePool;
use std::time::Duration;

/// Number of freelist pages to reclaim per `incremental_vacuum` step.
/// At a 4 KiB page size this reclaims ~1 MiB per step, keeping each write
/// transaction short so the UI never observes a long database lock.
const DEFAULT_VACUUM_PAGE_STEP: i64 = 256;

/// Maximum number of steps to run in a single session before giving the
/// runtime a longer breather. Prevents a huge freelist from monopolising the
/// single connection for an extended period.
const MAX_STEPS_PER_SESSION: u32 = 25;

/// Short pause between steps so other queued statements on the shared
/// connection can run (UI queries, event emissions, etc.).
const STEP_YIELD_DELAY: Duration = Duration::from_millis(50);

/// Schedule an incremental vacuum as a fire-and-forget background task.
///
/// This is intended to be called from the *completion path* of bulk-delete
/// Tauri commands, *after* the delete transaction has committed. The caller's
/// IPC response is returned immediately; reclaiming freelist pages happens
/// asynchronously so the Svelte UI stays responsive.
///
/// Errors are logged, never propagated — compaction is best-effort.
pub fn schedule_incremental_vacuum(pool: SqlitePool) {
    tauri::async_runtime::spawn(async move {
        if let Err(err) = run_incremental_vacuum(&pool, DEFAULT_VACUUM_PAGE_STEP).await {
            tracing::warn!("Incremental vacuum failed: {}", err);
        }
    });
}

/// Run an incremental vacuum across the freelist, stepping through pages.
///
/// Returns the total number of pages reclaimed (rounded up to the page step
/// by SQLite's `incremental_vacuum` semantics). Reads the freelist page count
/// before and after, logging both values together with the delta so disk-space
/// reclamation can be monitored.
///
/// The routine:
///  1. Fetches `freelist_count` and `page_count` via the `pragma_*`
///     table-valued functions (reliable under SQLx, unlike bare `PRAGMA
///     name` statements which may not yield a fetchable row).
///  2. If the freelist is empty, logs and returns `Ok(0)`.
///  3. Otherwise loops: `PRAGMA incremental_vacuum(N)` then re-checks the
///     freelist. Stops when the freelist is exhausted or `MAX_STEPS_PER_SESSION`
///     is reached (the remaining pages are reclaimed on the next scheduled
///     run — e.g. after the next bulk delete).
///
/// `max_pages` is the per-step page count passed to `incremental_vacuum`.
pub async fn run_incremental_vacuum(
    pool: &SqlitePool,
    max_pages: i64,
) -> Result<u64, String> {
    let max_pages = max_pages.max(1);
    let before = read_freelist_count(pool).await?;

    if before == 0 {
        tracing::info!(
            "Incremental vacuum: no freelist pages to reclaim (freelist_count=0)"
        );
        return Ok(0);
    }

    let pages_before = read_page_count(pool).await?;
    tracing::info!(
        "Incremental vacuum starting — freelist_pages={}, page_count={}, step_pages={}",
        before,
        pages_before,
        max_pages
    );

    let mut steps: u32 = 0;

    loop {
        let remaining = read_freelist_count(pool).await?;
        if remaining == 0 {
            break;
        }

        // Reclaim up to `max_pages` freelist pages per step. Each call is a
        // short write transaction, so the shared connection is never locked
        // for long; the loop yields between steps below.
        let claimed = remaining.min(max_pages);
        sqlx::query(&format!("PRAGMA incremental_vacuum({})", claimed))
            .execute(pool)
            .await
            .map_err(|err| format!("PRAGMA incremental_vacuum failed: {err}"))?;

        steps = steps.saturating_add(1);

        if steps >= MAX_STEPS_PER_SESSION {
            let after = read_freelist_count(pool).await?;
            tracing::info!(
                "Incremental vacuum capped at {} steps — {} pages remain on the freelist; \
                 will be reclaimed on the next scheduled run",
                MAX_STEPS_PER_SESSION,
                after
            );
            break;
        }

        // Give other consumers of the single connection a chance to run.
        tokio::time::sleep(STEP_YIELD_DELAY).await;
    }

    let after = read_freelist_count(pool).await?;
    let pages_after = read_page_count(pool).await?;

    // SQLite's `sqlite3_changes()` after `PRAGMA incremental_vacuum(N)`
    // reports the freelist page count at execution time rather than the
    // per-call pages moved, so compute the reclaimed total from the freelist
    // delta instead. This also matches the logged before/after figures.
    let reclaimed_pages = before.saturating_sub(after) as u64;

    tracing::info!(
        "Incremental vacuum finished — before_freelist_pages={}, after_freelist_pages={}, \
         pages_reclaimed={}, page_count_before={}, page_count_after={}, steps={}",
        before,
        after,
        reclaimed_pages,
        pages_before,
        pages_after,
        steps
    );

    Ok(reclaimed_pages)
}

/// Read the number of pages currently on the SQLite freelist.
async fn read_freelist_count(pool: &SqlitePool) -> Result<i64, String> {
    let (count,): (i64,) = sqlx::query_as("SELECT freelist_count FROM pragma_freelist_count")
        .fetch_one(pool)
        .await
        .map_err(|err| format!("Failed to read freelist_count: {err}"))?;
    Ok(count)
}

/// Read the total number of pages in the database file.
async fn read_page_count(pool: &SqlitePool) -> Result<i64, String> {
    let (count,): (i64,) = sqlx::query_as("SELECT page_count FROM pragma_page_count")
        .fetch_one(pool)
        .await
        .map_err(|err| format!("Failed to read page_count: {err}"))?;
    Ok(count)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    /// Create an in-memory pool configured with incremental auto-vacuum, the
    /// same setup the production connection applies on startup.
    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create in-memory pool");

        // auto_vacuum must be set on a fresh DB (in-memory counts as fresh),
        // before any tables are created, exactly like the production startup.
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

    /// Populate `count` rows (each with a ~`payload_size`-byte payload), delete
    /// them all, then return the freelist count.
    ///
    /// SQLite packs many tiny rows into a single page, so small payloads leave
    /// very few freelist pages. Using ~2 KiB payloads yields roughly one page
    /// per deleted row, which is needed to build a genuinely large freelist.
    async fn populate_and_delete(pool: &SqlitePool, count: i64, payload_size: usize) -> i64 {
        let filler = "x".repeat(payload_size.max(1));
        for i in 0..count {
            sqlx::query("INSERT INTO items (payload) VALUES (?)")
                .bind(format!("item-{i}-{filler}"))
                .execute(pool)
                .await
                .expect("insert item");
        }
        sqlx::query("DELETE FROM items")
            .execute(pool)
            .await
            .expect("delete all items");

        read_freelist_count(pool).await.expect("read freelist")
    }

    #[tokio::test]
    async fn incremental_vacuum_reclaims_freelist_pages() {
        let pool = test_pool().await;

        // Insert enough rows that deleting them leaves pages on the freelist.
        let freelist_before = populate_and_delete(&pool, 500, 2048).await;
        assert!(
            freelist_before > 0,
            "expected freelist pages after bulk delete, got {freelist_before}"
        );

        let reclaimed = run_incremental_vacuum(&pool, 256)
            .await
            .expect("incremental vacuum should succeed");

        assert!(
            reclaimed > 0,
            "expected at least one page reclaimed, got {reclaimed}"
        );

        let freelist_after = read_freelist_count(&pool)
            .await
            .expect("read freelist after");
        assert_eq!(
            freelist_after, 0,
            "incremental vacuum should empty the freelist entirely"
        );
    }

    #[tokio::test]
    async fn incremental_vacuum_noop_when_freelist_empty() {
        let pool = test_pool().await;

        // Insert a single row but do NOT delete it — no freelist pages.
        sqlx::query("INSERT INTO items (payload) VALUES (?)")
            .bind("keep-me")
            .execute(&pool)
            .await
            .expect("insert item");

        let reclaimed = run_incremental_vacuum(&pool, 256)
            .await
            .expect("incremental vacuum should succeed");

        assert_eq!(
            reclaimed, 0,
            "no pages should be reclaimed when the freelist is empty"
        );
    }

    #[tokio::test]
    async fn incremental_vacuum_respects_step_cap_for_large_freelists() {
        let pool = test_pool().await;

        // Create enough deleted rows to guarantee many freelist pages.
        let freelist_before = populate_and_delete(&pool, 2_000, 2048).await;
        assert!(
            freelist_before > 256,
            "expected a large freelist, got {freelist_before}"
        );

        // Step size of 1 with a cap of 25 means we reclaim at most 25 pages
        // in this session; the remainder stays on the freelist.
        let reclaimed = run_incremental_vacuum(&pool, 1)
            .await
            .expect("incremental vacuum should succeed");

        assert!(
            reclaimed <= 25,
            "step cap should bound pages reclaimed in one session, got {reclaimed}"
        );

        let freelist_after = read_freelist_count(&pool)
            .await
            .expect("read freelist after");
        assert!(
            freelist_after > 0,
            "expected remaining freelist pages after capped session"
        );

        // A subsequent run with a large step should finish the job.
        let _reclaimed_again = run_incremental_vacuum(&pool, 256)
            .await
            .expect("second incremental vacuum should succeed");

        let freelist_final = read_freelist_count(&pool)
            .await
            .expect("read freelist final");
        assert_eq!(
            freelist_final, 0,
            "second run should empty the freelist"
        );
    }

    #[tokio::test]
    async fn schedule_incremental_vacuum_runs_without_panicking() {
        let pool = test_pool().await;
        populate_and_delete(&pool, 200, 2048).await;

        // Fire-and-forget: the task must complete without error, but we
        // cannot await it directly. Give the spawned task a moment to run.
        schedule_incremental_vacuum(pool.clone());
        tokio::time::sleep(Duration::from_millis(300)).await;

        let freelist = read_freelist_count(&pool).await.expect("read freelist");
        assert_eq!(
            freelist, 0,
            "scheduled task should have reclaimed the freelist"
        );
    }
}