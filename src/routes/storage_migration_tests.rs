// Tests for the storage-migration route.
//
// Included via #[path]. `cancel_catalogue_storage_migration` is a State-only
// command and is testable via tauri::test::mock_app. The start command and
// run_migration_blocking take AppHandle<Wry>, which cannot be constructed from
// a mock runtime, so their logic is exercised through the tested
// services::storage_migration layer.

use super::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tauri::Manager;

fn make_app_state() -> AppState {
    let tmp_dir = std::env::temp_dir().join("storage-migration-route-test");
    AppState {
        db: crate::PoolHolder::default(),
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

#[tokio::test]
async fn cancel_catalogue_storage_migration_sets_cancel_flag() {
    let app = tauri::test::mock_app();
    app.manage(make_app_state());
    let state = app.state::<AppState>();
    let result = cancel_catalogue_storage_migration(state);
    assert!(result.is_ok());
    let app_state = app.state::<AppState>();
    assert!(app_state.migration_cancel_requested.load(Ordering::SeqCst));
}
