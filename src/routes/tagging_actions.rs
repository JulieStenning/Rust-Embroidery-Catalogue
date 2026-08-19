use crate::services::{auto_tagging, backfill, fingerprint, maintenance};
use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use tauri::State;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaggingActionRequest {
    pub request_override: Option<bool>,
    pub settings_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaggingActionsViewModel {
    pub has_google_api_key: bool,
    pub ai_tier2_auto: bool,
    pub ai_tier3_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    pub import_commit_batch_size: String,
    pub default_batch_size: i64,
    pub default_commit_every: i64,
    pub default_workers: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaggingActionPreview {
    pub enabled: bool,
    pub tier_order: Vec<String>,
}

const KEY_AI_TIER2_AUTO: &str = "ai.tier2_auto";
const KEY_AI_TIER3_AUTO: &str = "ai.tier3_auto";
const KEY_AI_BATCH_SIZE: &str = "ai.batch_size";
const KEY_AI_DELAY: &str = "ai.delay";
const KEY_AI_GOOGLE_API_KEY: &str = "ai.google_api_key";
const KEY_IMPORT_COMMIT_BATCH_SIZE: &str = "import.commit_batch_size";

#[tauri::command]
pub async fn get_tagging_actions_view_model(
    state: State<'_, AppState>,
) -> Result<TaggingActionsViewModel, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;

    let ai_tier2_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_TIER2_AUTO)
            .await
            .map_err(|e| e.to_string())?,
    );
    let ai_tier3_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_TIER3_AUTO)
            .await
            .map_err(|e| e.to_string())?,
    );
    let ai_batch_size = get_setting_with_default(&mut conn, KEY_AI_BATCH_SIZE)
        .await
        .map_err(|e| e.to_string())?;
    let ai_delay = get_setting_with_default(&mut conn, KEY_AI_DELAY)
        .await
        .map_err(|e| e.to_string())?;
    let import_commit_batch_size =
        get_setting_with_default(&mut conn, KEY_IMPORT_COMMIT_BATCH_SIZE)
            .await
            .map_err(|e| e.to_string())?;

    let google_api_key = get_setting_with_default(&mut conn, KEY_AI_GOOGLE_API_KEY)
        .await
        .map_err(|e| e.to_string())?;
    let has_google_api_key = !google_api_key.trim().is_empty();

    Ok(TaggingActionsViewModel {
        has_google_api_key,
        ai_tier2_auto,
        ai_tier3_auto,
        ai_batch_size,
        ai_delay,
        import_commit_batch_size,
        default_batch_size: 100,
        default_commit_every: 100,
        default_workers: 4,
    })
}

#[tauri::command]
pub async fn run_unified_backfill(
    state: State<'_, AppState>,
    request: backfill::UnifiedBackfillRequest,
) -> Result<backfill::UnifiedBackfillSummary, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;
    let google_api_key = get_setting_with_default(&mut conn, KEY_AI_GOOGLE_API_KEY)
        .await
        .map_err(|e| e.to_string())?;
    let has_api_key = !google_api_key.trim().is_empty();
    drop(conn);

    if let Some(ref actions) = request.actions {
        if let Some(ref tagging) = actions.tagging {
            if tagging.enabled.unwrap_or(true) {
                if let Some(ref tiers) = tagging.tiers {
                    let requests_ai = tiers.contains(&2) || tiers.contains(&3);
                    if requests_ai && !has_api_key {
                        return Err(
                            "Google API key is required for AI tagging (Tier 2 / Tier 3). Please configure your API key in Admin -> Settings."
                                .to_string(),
                        );
                    }
                }
            }
        }
    }

    backfill::run_unified_backfill(&state.db, request, has_api_key)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn stop_unified_backfill() -> backfill::StopBackfillResult {
    backfill::request_stop()
}

#[tauri::command]
pub async fn get_backfill_log_entries(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<backfill::BackfillLogEntry>, String> {
    backfill::get_backfill_log_entries(&state.db, limit.unwrap_or(20)).await
}

#[tauri::command]
pub async fn run_stitching_backfill(
    state: State<'_, AppState>,
    clear_stitching_mode: Option<String>,
    batch_size: Option<i64>,
) -> Result<backfill::UnifiedBackfillSummary, String> {
    let request = backfill::UnifiedBackfillRequest {
        actions: Some(backfill::UnifiedBackfillActions {
            tagging: None,
            stitching: Some(backfill::StitchingActionOptions {
                clear_stitching_mode,
                enabled: Some(true),
            }),
            images: None,
            color_counts: None,
            fingerprinting: None,
        }),
        batch_size,
        commit_every: Some(100),
        workers: Some(1),
        delay_seconds: None,
        vision_delay_seconds: None,
    };
    backfill::run_unified_backfill(&state.db, request, false)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn run_fingerprint_backfill(
    state: State<'_, AppState>,
    commit_every: Option<i64>,
) -> Result<fingerprint::FingerprintSummary, String> {
    fingerprint::run_fingerprint_backfill(&state.db, commit_every.unwrap_or(100))
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn preview_tagging_action(
    request: TaggingActionRequest,
) -> Result<TaggingActionPreview, String> {
    let precedence = auto_tagging::TaggingPrecedence {
        request_override: request.request_override,
        settings_default: request.settings_default,
        hard_default: true,
    };

    let tier_order = auto_tagging::ordered_tiers()
        .iter()
        .map(|tier| format!("{:?}", tier))
        .collect();

    Ok(TaggingActionPreview {
        enabled: auto_tagging::resolve_enabled(&precedence),
        tier_order,
    })
}

async fn get_setting_with_default(
    conn: &mut SqliteConnection,
    key: &str,
) -> Result<String, sqlx::Error> {
    maintenance::get_setting_with_default(conn, key)
        .await
        .map_err(|error| sqlx::Error::Protocol(error.to_string()))
}

fn is_truthy(raw: &str) -> bool {
    maintenance::is_truthy(raw)
}

#[cfg(test)]
mod tests {
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
        assert_eq!(preview.tier_order, vec!["Tier1", "Tier2", "Tier3"]);

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
            db: pool,
            database_status: crate::DatabaseStatus {
                status: crate::DatabaseStatusKind::Connected,
                configured_data_root: Some(tmp_dir.to_string_lossy().to_string()),
                database_path: Some(tmp_dir.join("Database").join("test.db").to_string_lossy().to_string()),
                embroidery_dir: Some(tmp_dir.join("MachineEmbroideryDesigns").to_string_lossy().to_string()),
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
            let mut conn = state.db.acquire().await.unwrap();
            sqlx::query("UPDATE settings SET value = 'test-api-key' WHERE key = ?")
                .bind(KEY_AI_GOOGLE_API_KEY)
                .execute(&mut *conn)
                .await
                .unwrap();
        }

        let vm = get_tagging_actions_view_model(state.clone()).await.unwrap();
        assert!(vm.has_google_api_key);
        assert!(!vm.ai_tier2_auto);
        assert!(!vm.ai_tier3_auto);
        assert_eq!(vm.ai_batch_size, "");
        assert_eq!(vm.ai_delay, "");

        // Update settings in database to check truthiness
        {
            let mut conn = state.db.acquire().await.unwrap();
            sqlx::query("UPDATE settings SET value = '1' WHERE key = ?")
                .bind(KEY_AI_TIER2_AUTO)
                .execute(&mut *conn)
                .await
                .unwrap();
            sqlx::query("UPDATE settings SET value = 'true' WHERE key = ?")
                .bind(KEY_AI_TIER3_AUTO)
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
        assert!(vm2.ai_tier2_auto);
        assert!(vm2.ai_tier3_auto);
        assert_eq!(vm2.ai_batch_size, "50");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
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
            let mut conn = state.db.acquire().await.unwrap();
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
            let mut conn = state.db.acquire().await.unwrap();
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
                    tiers: Some(vec![1, 2]),
                    enabled: Some(true),
                }),
                stitching: None,
                images: None,
                color_counts: None,
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
}
