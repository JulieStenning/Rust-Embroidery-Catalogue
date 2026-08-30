// Tests for the tagging_actions route.
//
// This module was split out of tagging_actions.rs so the route file can stay
// focused on production logic. It is included via a #[path] declaration in a
// #[cfg(test)] mod tests; module, so it retains full access to the private
// items in the parent module through use super::*;.
use super::*;
use crate::utils::test_support::lock_env;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::atomic::AtomicBool;
use tauri::Manager;

#[test]
fn is_truthy_accepts_expected_variants() {
    assert!(is_truthy("true"));
    assert!(is_truthy("YES"));
    assert!(is_truthy("1"));
    assert!(is_truthy("on"));
    assert!(is_truthy("  true  "));
    assert!(is_truthy("On"));
    assert!(is_truthy("Yes"));

    assert!(!is_truthy("false"));
    assert!(!is_truthy("0"));
    assert!(!is_truthy("no"));
    assert!(!is_truthy("off"));
    assert!(!is_truthy("other"));
}

#[test]
fn preview_tagging_action_prefers_request_override() {
    let preview = preview_tagging_action(TaggingActionRequest {
        request_override: Some(false),
        settings_default: Some(true),
    })
    .expect("preview works");

    assert!(!preview.enabled);
    assert_eq!(preview.mode_order, vec!["FileFolder", "VisualAi"]);

    let preview2 = preview_tagging_action(TaggingActionRequest {
        request_override: Some(true),
        settings_default: Some(false),
    })
    .expect("preview works");
    assert!(preview2.enabled);
}

#[test]
fn preview_tagging_action_falls_back_to_settings_default() {
    let preview = preview_tagging_action(TaggingActionRequest {
        request_override: None,
        settings_default: Some(true),
    })
    .expect("preview works");

    assert!(preview.enabled);

    let preview2 = preview_tagging_action(TaggingActionRequest {
        request_override: None,
        settings_default: Some(false),
    })
    .expect("preview works");
    assert!(!preview2.enabled);
}

#[test]
fn preview_tagging_action_none_defaults() {
    let preview = preview_tagging_action(TaggingActionRequest {
        request_override: None,
        settings_default: None,
    })
    .expect("preview works");
    assert!(preview.enabled);
}

async fn test_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create test sqlite pool");
    crate::database::migrations::run_migrations(&pool)
        .await
        .expect("failed to run migrations");
    pool
}

fn make_app_state(pool: SqlitePool, tmp_dir: &std::path::Path) -> AppState {
    AppState {
        db: crate::PoolHolder::new(pool),
        database_status: crate::DatabaseStatus {
            status: crate::DatabaseStatusKind::Connected,
            configured_data_root: Some(tmp_dir.to_string_lossy().to_string()),
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
            data_root: tmp_dir.to_path_buf(),
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
async fn test_get_setting_with_default() {
    let pool = test_pool().await;
    let mut conn = pool.acquire().await.unwrap();

    // 1. Key doesn't exist, should insert empty string and return it
    let val = get_setting_with_default(&mut conn, "test.new_key")
        .await
        .unwrap();
    assert_eq!(val, "");

    // Verify it was inserted
    let inserted = crate::settings::get_setting(&mut conn, "test.new_key")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inserted.value, "");

    // 2. Key exists, should return it
    sqlx::query("UPDATE settings SET value = ? WHERE key = ?")
        .bind("existing_val")
        .bind("test.new_key")
        .execute(&mut *conn)
        .await
        .unwrap();

    let val2 = get_setting_with_default(&mut conn, "test.new_key")
        .await
        .unwrap();
    assert_eq!(val2, "existing_val");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_get_tagging_actions_view_model() {
    let _guard = lock_env();

    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("tagging-actions-test-vm");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // Set ai.google_api_key in database
    {
        let mut conn = state.db_pool().unwrap().acquire().await.unwrap();
        sqlx::query("UPDATE settings SET value = 'test-api-key' WHERE key = ?")
            .bind(KEY_AI_GOOGLE_API_KEY)
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    let vm = get_tagging_actions_view_model(state.clone()).await.unwrap();
    assert!(vm.has_google_api_key);
    assert!(!vm.ai_vision_auto);
    assert_eq!(vm.ai_batch_size, "");
    assert_eq!(vm.ai_delay, "");
    assert!(!vm.ai_free_tier);
    assert_eq!(vm.import_commit_batch_size, "");
    assert_eq!(vm.default_batch_size, 100);
    assert_eq!(vm.default_commit_every, 100);
    assert_eq!(vm.default_workers, 4);
    assert_eq!(vm.default_delay, 5.0);

    // Update settings in database to check truthiness
    {
        let mut conn = state.db_pool().unwrap().acquire().await.unwrap();
        sqlx::query("UPDATE settings SET value = 'true' WHERE key = ?")
            .bind(KEY_AI_VISION_AUTO)
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query("UPDATE settings SET value = '50' WHERE key = ?")
            .bind(KEY_AI_BATCH_SIZE)
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    let vm2 = get_tagging_actions_view_model(state).await.unwrap();
    assert!(vm2.ai_vision_auto);
    assert_eq!(vm2.ai_batch_size, "50");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn tagging_actions_view_model_free_tier_uses_conservative_defaults() {
    let _guard = lock_env();

    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("tagging-actions-test-free-tier");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // Declare the key is on the free tier.
    {
        let mut conn = state.db_pool().unwrap().acquire().await.unwrap();
        sqlx::query(
            "INSERT OR REPLACE INTO settings (key, value, description) VALUES (?, 'true', '')",
        )
        .bind(KEY_AI_FREE_TIER)
        .execute(&mut *conn)
        .await
        .unwrap();
    }

    let vm = get_tagging_actions_view_model(state.clone()).await.unwrap();
    assert!(vm.ai_free_tier);
    assert_eq!(vm.default_workers, 2);
    assert_eq!(vm.default_delay, 10.0);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_backfills_and_logs() {
    let _guard = lock_env();

    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("tagging-actions-test-backfill");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // Set ai.google_api_key in database
    {
        let mut conn = state.db_pool().unwrap().acquire().await.unwrap();
        sqlx::query("UPDATE settings SET value = 'test-api-key' WHERE key = ?")
            .bind(KEY_AI_GOOGLE_API_KEY)
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    // 1. Run unified backfill (no actions active)
    let request = backfill::UnifiedBackfillRequest {
        actions: None,
        batch_size: None,
        commit_every: None,
        workers: None,
        delay_seconds: None,
        vision_delay_seconds: None,
    };
    let summary = run_unified_backfill(state.clone(), request).await.unwrap();
    assert_eq!(summary.processed, 0);

    // 2. Stop backfill
    let stop_res = stop_unified_backfill();
    assert_eq!(stop_res.status, "stopping");

    // 3. Get backfill log entries
    let logs = get_backfill_log_entries(state.clone(), Some(5))
        .await
        .unwrap();
    assert!(logs.len() <= 5);

    // 4. Run stitching backfill
    let summary_stitch =
        run_stitching_backfill(state.clone(), Some("unverified".to_string()), Some(10))
            .await
            .unwrap();
    assert_eq!(summary_stitch.processed, 0);

    // 5. Run fingerprint backfill
    let summary_fingerprint = run_fingerprint_backfill(state.clone(), Some(5))
        .await
        .unwrap();
    assert_eq!(summary_fingerprint.processed, 0);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn test_run_unified_backfill_errors_when_ai_tagging_requested_without_key() {
    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("tagging-actions-test-no-key");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // Ensure ai.google_api_key in database is empty
    {
        let mut conn = state.db_pool().unwrap().acquire().await.unwrap();
        sqlx::query("UPDATE settings SET value = '' WHERE key = ?")
            .bind(KEY_AI_GOOGLE_API_KEY)
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    let request = backfill::UnifiedBackfillRequest {
        actions: Some(backfill::UnifiedBackfillActions {
            tagging: Some(backfill::TaggingActionOptions {
                action: Some("tag_untagged".to_string()),
                modes: Some(vec!["path_rule".to_string(), "ai_vision".to_string()]),
                enabled: Some(true),
            }),
            stitching: None,
            images: None,
            color_counts: None,
            hoop_dimensions: None,
            fingerprinting: None,
        }),
        batch_size: None,
        commit_every: None,
        workers: None,
        delay_seconds: None,
        vision_delay_seconds: None,
    };

    let result = run_unified_backfill(state.clone(), request).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Google API key is required"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn test_run_unified_backfill_proceeds_without_ai_when_no_ai_modes() {
    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("tagging-actions-test-no-ai-tiers");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    let request = backfill::UnifiedBackfillRequest {
        actions: Some(backfill::UnifiedBackfillActions {
            tagging: Some(backfill::TaggingActionOptions {
                action: Some("tag_untagged".to_string()),
                modes: Some(vec!["path_rule".to_string()]),
                enabled: Some(true),
            }),
            stitching: None,
            images: None,
            color_counts: None,
            hoop_dimensions: None,
            fingerprinting: None,
        }),
        batch_size: None,
        commit_every: None,
        workers: None,
        delay_seconds: None,
        vision_delay_seconds: None,
    };

    // File & Folder Rules does not require an API key, so the guard falls
    // through even with no key configured; the request proceeds to the service.
    let result = run_unified_backfill(state.clone(), request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().processed, 0);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn test_run_unified_backfill_skips_ai_check_when_tagging_disabled() {
    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("tagging-actions-test-tagging-disabled");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // Tagging is explicitly disabled, so the Visual AI guard is skipped even
    // though ai_vision is listed and no API key is configured.
    let request = backfill::UnifiedBackfillRequest {
        actions: Some(backfill::UnifiedBackfillActions {
            tagging: Some(backfill::TaggingActionOptions {
                action: Some("tag_untagged".to_string()),
                modes: Some(vec!["ai_vision".to_string()]),
                enabled: Some(false),
            }),
            stitching: None,
            images: None,
            color_counts: None,
            hoop_dimensions: None,
            fingerprinting: None,
        }),
        batch_size: None,
        commit_every: None,
        workers: None,
        delay_seconds: None,
        vision_delay_seconds: None,
    };

    let result = run_unified_backfill(state.clone(), request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().processed, 0);

    let _ = std::fs::remove_dir_all(&tmp);
}
