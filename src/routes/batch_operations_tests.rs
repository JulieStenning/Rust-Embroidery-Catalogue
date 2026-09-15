// Tests for the batch_operations route.
//
// This module was split out of batch_operations.rs so the route file can stay
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
async fn test_get_batch_operations_view_model() {
    let _guard = lock_env();

    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("batch-operations-test-vm");
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

    let vm = get_batch_operations_view_model(state.clone())
        .await
        .unwrap();
    assert!(vm.has_google_api_key);
    assert!(!vm.ai_vision_auto);
    assert_eq!(vm.ai_batch_size, "");
    assert_eq!(vm.ai_delay, "");
    assert!(!vm.ai_free_tier);
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

    let vm2 = get_batch_operations_view_model(state).await.unwrap();
    assert!(vm2.ai_vision_auto);
    assert_eq!(vm2.ai_batch_size, "50");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn test_count_tagging_candidates_reports_scope_counts() {
    let _guard = lock_env();

    let pool = test_pool().await;
    // Seed: 3 designs (very high ids to avoid migration-seeded rows); design
    // 900102 carries an image-group tag (migration-seeded id 8, 'Don't Know')
    // and is verified.
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified) VALUES (900101, 'a.pes', 'a.pes', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified) VALUES (900102, 'b.pes', 'b.pes', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified) VALUES (900103, 'c.pes', 'c.pes', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (900102, 8)")
        .execute(&pool)
        .await
        .unwrap();

    let tmp = std::env::temp_dir().join("batch-operations-count-test");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);
    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // tag_untagged -> designs with no image-group tags: 900101 and 900103 (unverified).
    let untagged = count_tagging_candidates(
        state.clone(),
        Some("tag_untagged".to_string()),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(untagged.total_count, 2);
    assert_eq!(untagged.unverified_count, 2);
    assert_eq!(untagged.verified_count, 0);
    // retag_all_unverified -> image_tags_verified = 0: 900101 and 900103.
    let unverified = count_tagging_candidates(
        state.clone(),
        Some("retag_all_unverified".to_string()),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(unverified.total_count, 2);
    assert_eq!(unverified.unverified_count, 2);
    assert_eq!(unverified.verified_count, 0);
    // retag_all -> every design: total 3, one of which is verified (900102).
    let all = count_tagging_candidates(
        state.clone(),
        Some("retag_all".to_string()),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(all.total_count, 3);
    assert_eq!(all.unverified_count, 2);
    assert_eq!(all.verified_count, 1);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // current-thread runtime; guard never crosses threads
async fn batch_operations_view_model_free_tier_uses_conservative_defaults() {
    let _guard = lock_env();

    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("batch-operations-test-free-tier");
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

    let vm = get_batch_operations_view_model(state.clone())
        .await
        .unwrap();
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
                merge_mode: None,
                exclude_verified: None,
                folder_path: None,
                folder_paths: None,
                include_subfolders: None,
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
                merge_mode: None,
                exclude_verified: None,
                folder_path: None,
                folder_paths: None,
                include_subfolders: None,
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
                merge_mode: None,
                exclude_verified: None,
                folder_path: None,
                folder_paths: None,
                include_subfolders: None,
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

#[tokio::test]
async fn browse_tagging_folder_rejects_start_outside_data_root() {
    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("tagging-actions-browse-outside");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // A start path that is NOT under the configured embroidery designs root.
    let outside = std::env::temp_dir()
        .join("tagging-outside-location")
        .join("designs");
    let result = browse_tagging_folder(
        state,
        Some(BrowseTaggingFolderRequest {
            start_dir: Some(outside.to_string_lossy().to_string()),
            allow_multi: None,
        }),
    );

    // The early validation returns an error before the native picker is opened.
    assert_eq!(result.path, None);
    assert!(result.paths.is_empty());
    assert!(result.relative_paths.is_empty());
    assert_eq!(
        result.error.as_deref(),
        Some("Start folder is outside the Data Storage Location.")
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn test_run_maintenance_batch_options() {
    let _guard = lock_env();

    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("batch-operations-test-maint");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // 1. None request
    let summary1 = run_maintenance_batch(state.clone(), None).await.unwrap();
    assert_eq!(summary1.processed, 0);

    // 2. Default request
    let summary2 = run_maintenance_batch(state.clone(), Some(MaintenanceBatchRequest::default()))
        .await
        .unwrap();
    assert_eq!(summary2.processed, 0);

    // 3. missing_previews scope with all maintenance flags enabled
    let summary3 = run_maintenance_batch(
        state.clone(),
        Some(MaintenanceBatchRequest {
            scope: Some("missing_previews".to_string()),
            generate_previews: Some(true),
            recalc_color_counts: Some(true),
            recalc_hoop_dimensions: Some(true),
            batch_size: Some(25),
            commit_every: Some(25),
            workers: Some(2),
        }),
    )
    .await
    .unwrap();
    assert_eq!(summary3.processed, 0);

    // 4. "all" scope with selective flags
    let summary4 = run_maintenance_batch(
        state.clone(),
        Some(MaintenanceBatchRequest {
            scope: Some("all".to_string()),
            generate_previews: Some(false),
            recalc_color_counts: Some(true),
            recalc_hoop_dimensions: Some(false),
            batch_size: Some(10),
            commit_every: Some(10),
            workers: Some(1),
        }),
    )
    .await
    .unwrap();
    assert_eq!(summary4.processed, 0);

    // 5. "all" scope with preview generation only
    let summary5 = run_maintenance_batch(
        state.clone(),
        Some(MaintenanceBatchRequest {
            scope: Some("all".to_string()),
            generate_previews: Some(true),
            recalc_color_counts: Some(false),
            recalc_hoop_dimensions: Some(true),
            batch_size: None,
            commit_every: None,
            workers: None,
        }),
    )
    .await
    .unwrap();
    assert_eq!(summary5.processed, 0);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn test_count_missing_preview_designs() {
    let _guard = lock_env();

    let pool = test_pool().await;
    // Insert one design with missing preview (image_data is NULL)
    sqlx::query("INSERT INTO designs (id, filename, filepath, image_data) VALUES (950001, 'nopreview.pes', 'nopreview.pes', NULL)")
        .execute(&pool)
        .await
        .unwrap();
    // Insert one design with preview
    sqlx::query("INSERT INTO designs (id, filename, filepath, image_data) VALUES (950002, 'haspreview.pes', 'haspreview.pes', X'89504E470D0A1A0A')")
        .execute(&pool)
        .await
        .unwrap();

    let tmp = std::env::temp_dir().join("batch-operations-test-missing-previews");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    let count = count_missing_preview_designs(state).await.unwrap();
    assert!(count >= 1);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_backfill_app_handle_and_emission() {
    let progress = backfill::BackfillProgress {
        stage: "tagging".to_string(),
        processed: 5,
        errors: 0,
        current_action: "visual_ai".to_string(),
    };
    emit_backfill_progress(&progress);
}

#[test]
fn test_dto_derives_and_json_serde() {
    // 1. TaggingActionRequest
    let req: TaggingActionRequest =
        serde_json::from_str(r#"{"request_override": true, "settings_default": false}"#).unwrap();
    assert_eq!(req.request_override, Some(true));
    assert_eq!(req.settings_default, Some(false));
    let debug_str = format!("{:?}", req.clone());
    assert!(debug_str.contains("TaggingActionRequest"));
    let default_req = TaggingActionRequest::default();
    assert_eq!(default_req.request_override, None);

    // 2. BatchOperationsViewModel
    let vm = BatchOperationsViewModel {
        has_google_api_key: true,
        ai_vision_auto: false,
        ai_batch_size: "100".to_string(),
        ai_delay: "5.0".to_string(),
        ai_commit_every: "100".to_string(),
        ai_workers: "4".to_string(),
        ai_free_tier: false,
        default_batch_size: 100,
        default_commit_every: 100,
        default_workers: 4,
        default_delay: 5.0,
        data_storage_location: "/test/designs".to_string(),
    };
    let vm_json = serde_json::to_string(&vm).unwrap();
    assert!(vm_json.contains("has_google_api_key"));
    let vm_debug = format!("{:?}", vm.clone());
    assert!(vm_debug.contains("BatchOperationsViewModel"));

    // 3. TaggingActionPreview
    let preview = TaggingActionPreview {
        enabled: true,
        mode_order: vec!["FileFolder".to_string()],
    };
    let preview_json = serde_json::to_string(&preview).unwrap();
    assert!(preview_json.contains("FileFolder"));
    let preview_debug = format!("{:?}", preview.clone());
    assert!(preview_debug.contains("TaggingActionPreview"));

    // 4. BrowseTaggingFolderRequest
    let parsed_browse_req: BrowseTaggingFolderRequest =
        serde_json::from_str(r#"{"start_dir": "/test", "allow_multi": true}"#).unwrap();
    assert_eq!(parsed_browse_req.start_dir, Some("/test".to_string()));
    assert_eq!(parsed_browse_req.allow_multi, Some(true));
    let browse_req_debug = format!("{:?}", parsed_browse_req.clone());
    assert!(browse_req_debug.contains("BrowseTaggingFolderRequest"));
    let default_browse_req = BrowseTaggingFolderRequest::default();
    assert_eq!(default_browse_req.start_dir, None);

    // 5. BrowseTaggingFolderResult
    let browse_res = BrowseTaggingFolderResult {
        path: Some("/test/folder".to_string()),
        paths: vec!["/test/folder".to_string()],
        relative_paths: vec!["folder".to_string()],
        error: None,
    };
    let browse_res_json = serde_json::to_string(&browse_res).unwrap();
    assert!(browse_res_json.contains("/test/folder"));
    let browse_res_debug = format!("{:?}", browse_res.clone());
    assert!(browse_res_debug.contains("BrowseTaggingFolderResult"));
    let default_browse_res = BrowseTaggingFolderResult::default();
    assert!(default_browse_res.path.is_none());

    // 6. MaintenanceBatchRequest
    let parsed_maint_req: MaintenanceBatchRequest = serde_json::from_str(
        r#"{"scope": "missing_previews", "generate_previews": true, "recalc_color_counts": true, "recalc_hoop_dimensions": true, "batch_size": 100, "commit_every": 100, "workers": 4}"#,
    ).unwrap();
    assert_eq!(parsed_maint_req.scope, Some("missing_previews".to_string()));
    assert_eq!(parsed_maint_req.generate_previews, Some(true));
    assert_eq!(parsed_maint_req.recalc_color_counts, Some(true));
    assert_eq!(parsed_maint_req.recalc_hoop_dimensions, Some(true));
    assert_eq!(parsed_maint_req.batch_size, Some(100));
    assert_eq!(parsed_maint_req.commit_every, Some(100));
    assert_eq!(parsed_maint_req.workers, Some(4));
    let maint_req_debug = format!("{:?}", parsed_maint_req.clone());
    assert!(maint_req_debug.contains("MaintenanceBatchRequest"));
    let default_maint_req = MaintenanceBatchRequest::default();
    assert_eq!(default_maint_req.scope, None);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn test_run_unified_backfill_with_ai_enabled_and_valid_key() {
    let _guard = lock_env();

    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("batch-operations-test-ai-key-run");
    std::fs::create_dir_all(&tmp).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // Set a valid google API key in the settings table
    {
        let mut conn = state.db_pool().unwrap().acquire().await.unwrap();
        sqlx::query("UPDATE settings SET value = 'test-valid-api-key' WHERE key = ?")
            .bind(KEY_AI_GOOGLE_API_KEY)
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    let request = backfill::UnifiedBackfillRequest {
        actions: Some(backfill::UnifiedBackfillActions {
            tagging: Some(backfill::TaggingActionOptions {
                action: Some("tag_untagged".to_string()),
                modes: Some(vec!["path_rule".to_string()]),
                merge_mode: None,
                exclude_verified: None,
                folder_path: None,
                folder_paths: None,
                include_subfolders: None,
                enabled: Some(true),
            }),
            stitching: None,
            images: None,
            color_counts: None,
            hoop_dimensions: None,
            fingerprinting: None,
        }),
        batch_size: Some(10),
        commit_every: Some(10),
        workers: Some(1),
        delay_seconds: None,
        vision_delay_seconds: None,
    };

    let result = run_unified_backfill(state.clone(), request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().processed, 0);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn test_count_tagging_candidates_defaults_and_options() {
    let _guard = lock_env();

    let pool = test_pool().await;
    let tmp = std::env::temp_dir().join("batch-operations-test-candidates-options");
    let folder = tmp.join("subfolder");
    std::fs::create_dir_all(&folder).ok();
    let app_state = make_app_state(pool, &tmp);

    let app = tauri::test::mock_app();
    app.manage(app_state);
    let state = app.state::<AppState>();

    // Test with completely None options (exercises defaults)
    let counts = count_tagging_candidates(state.clone(), None, None, None, None)
        .await
        .unwrap();
    assert_eq!(counts.total_count, 0);

    // Test with outside folder path to exercise error mapping
    let counts2 = count_tagging_candidates(
        state.clone(),
        Some("tag_untagged".to_string()),
        None,
        Some(vec![folder.to_string_lossy().to_string()]),
        Some(false),
    )
    .await;
    assert!(counts2.is_err());

    let _ = std::fs::remove_dir_all(&tmp);
}
