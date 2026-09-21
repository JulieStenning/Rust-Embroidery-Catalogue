// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

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

#[test]
fn test_db_health_derives_and_edge_cases() {
    let fm = FreelistMetrics {
        page_count: 100,
        freelist_count: 20,
    };
    assert!(format!("{:?}", fm).contains("page_count: 100"));
    let _fm_clone = fm.clone();
    let _fm_json = serde_json::to_value(&fm).unwrap();

    let started = DbMaintenanceStartedEvent {
        page_count: 100,
        freelist_pages: 20,
        free_ratio: 0.2,
        reclaimable_bytes: 81920,
    };
    assert!(format!("{:?}", started).contains("reclaimable_bytes: 81920"));
    let _st_clone = started.clone();
    let _st_json = serde_json::to_value(&started).unwrap();

    let finished = DbMaintenanceFinishedEvent {
        reclaimed_pages: 20,
        reclaimable_bytes_before: 81920,
        reclaimable_bytes_after: 0,
        duration_ms: 15,
    };
    assert!(format!("{:?}", finished).contains("duration_ms: 15"));
    let _fn_clone = finished.clone();
    let _fn_json = serde_json::to_value(&finished).unwrap();

    let snap_empty = MetricsSnapshot {
        page_count: 0,
        freelist_count: 0,
        page_size: 4096,
    };
    assert_eq!(snap_empty.free_ratio(), 0.0);
    let snap_neg = MetricsSnapshot {
        page_count: -1,
        freelist_count: -5,
        page_size: -4096,
    };
    assert_eq!(snap_neg.free_ratio(), 0.0);
    assert_eq!(snap_neg.reclaimable_bytes(), 0);
    assert!(format!("{:?}", snap_neg).contains("page_size: -4096"));
    let _snap_clone = snap_neg.clone();
}

#[tokio::test]
async fn check_and_schedule_maintenance_skips_when_running() {
    let pool = test_pool().await;
    let running = Arc::new(AtomicBool::new(true));
    let shutdown = Arc::new(AtomicBool::new(false));
    let app = tauri::test::mock_app();
    let res = check_and_schedule_maintenance(pool, running, shutdown, app.handle().clone()).await;
    assert_eq!(res, Ok(false));
}

#[tokio::test]
async fn check_and_schedule_maintenance_skips_when_below_threshold() {
    let pool = test_pool().await;
    let running = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::new(AtomicBool::new(false));
    let app = tauri::test::mock_app();
    let res = check_and_schedule_maintenance(pool, running, shutdown, app.handle().clone()).await;
    assert_eq!(res, Ok(false));
}
