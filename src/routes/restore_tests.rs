// Tests for the restore route (thin IPC over services::restore).
//
// Included via #[path]. Covers the RestoreGuard drop behaviour, the
// request_cancel_restore flag, and the State-only detect-unmatched command.
// The restore_database / restore_designs_incremental / restore_both commands
// take AppHandle<Wry>, which cannot be constructed from a mock runtime, so
// their logic is exercised through the already-tested services::restore layer.

use super::*;
use serial_test::serial;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tauri::Manager;

/// In-memory pool with the settings and designs tables the route touches.
async fn restore_test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create test sqlite pool");
    sqlx::query(
        "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, description TEXT)",
    )
    .execute(&pool)
    .await
    .expect("failed to create settings table");
    sqlx::query(
        "CREATE TABLE designs (id INTEGER PRIMARY KEY AUTOINCREMENT, filename TEXT NOT NULL, filepath TEXT NOT NULL)",
    )
    .execute(&pool)
    .await
    .expect("failed to create designs table");
    pool
}

/// Build a minimal AppState backed by an in-memory SQLite pool.
fn command_app_state(pool: SqlitePool) -> AppState {
    let tmp_dir = std::env::temp_dir().join("restore-route-command-test");
    std::fs::create_dir_all(&tmp_dir).ok();
    AppState {
        db: crate::PoolHolder::new(pool),
        database_status: crate::DatabaseStatus {
            status: crate::DatabaseStatusKind::Connected,
            configured_data_root: Some(tmp_dir.clone().to_string_lossy().to_string()),
            database_path: Some(
                tmp_dir
                    .join("Database")
                    .join("test.db")
                    .to_string_lossy()
                    .to_string(),
            ),
            embroidery_dir: Some(
                tmp_dir
                    .join("MachineEmbroideryDesigns")
                    .to_string_lossy()
                    .to_string(),
            ),
            data_root_missing: false,
        },
        paths: crate::paths::AppPaths {
            mode: crate::paths::ExecutionMode::Installed,
            data_root: tmp_dir.clone(),
            embroidery_designs_dir: tmp_dir.join("MachineEmbroideryDesigns"),
            database_dir: tmp_dir.join("Database"),
            database_path: tmp_dir.join("Database").join("test.db"),
            log_dir: tmp_dir.join("logs"),
        },
        log_guard: crate::logging::LogGuard::dummy_for_test(),
        shutdown_requested: AtomicBool::new(false),
        maintenance_running: AtomicBool::new(false),
        migration_running: AtomicBool::new(false),
        migration_cancel_requested: std::sync::Arc::new(AtomicBool::new(false)),
        restore_in_progress: AtomicBool::new(false),
    }
}

#[test]
#[serial]
fn request_cancel_restore_sets_flag() {
    let result = request_cancel_restore();
    assert!(result.cancel_requested);
}

#[test]
fn restore_guard_resets_flag_on_drop() {
    let flag = AtomicBool::new(true);
    {
        let _guard = RestoreGuard(&flag);
        assert!(flag.load(Ordering::SeqCst));
    }
    assert!(!flag.load(Ordering::SeqCst));
}

#[tokio::test]
#[serial]
async fn detect_design_files_absent_from_database_returns_result() {
    let prior_db = std::env::var("DATABASE_URL").ok();
    let tmp = std::env::temp_dir().join("restore-detect-test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("Database")).unwrap();
    std::fs::create_dir_all(tmp.join("MachineEmbroideryDesigns")).unwrap();
    let url = format!(
        "sqlite:///{}/Database/EmbroideryCatalogue.db",
        tmp.to_string_lossy().replace('\\', "/")
    );
    std::env::set_var("DATABASE_URL", &url);

    let pool = restore_test_pool().await;
    let app = tauri::test::mock_app();
    app.manage(command_app_state(pool));
    let app_state = app.state::<AppState>();
    let result = detect_design_files_absent_from_database(app_state)
        .await
        .expect("detect should succeed");
    assert_eq!(result.checked, 0);
    assert_eq!(result.unmatched, 0);

    match prior_db {
        Some(v) => std::env::set_var("DATABASE_URL", v),
        None => std::env::remove_var("DATABASE_URL"),
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
