use crate::services::{auto_tagging, backfill, fingerprint, maintenance};
use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use std::sync::OnceLock;
use tauri::{Emitter, State};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaggingActionRequest {
    pub request_override: Option<bool>,
    pub settings_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaggingActionsViewModel {
    pub has_google_api_key: bool,
    pub ai_vision_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    pub ai_commit_every: String,
    pub ai_workers: String,
    pub ai_free_tier: bool,
    pub import_commit_batch_size: String,
    pub default_batch_size: i64,
    pub default_commit_every: i64,
    pub default_workers: i64,
    pub default_delay: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaggingActionPreview {
    pub enabled: bool,
    pub mode_order: Vec<String>,
}

const KEY_AI_VISION_AUTO: &str = "ai.vision";
const KEY_AI_BATCH_SIZE: &str = "ai.batch_size";
const KEY_AI_DELAY: &str = "ai.delay";
const KEY_AI_GOOGLE_API_KEY: &str = "ai.google_api_key";
const KEY_AI_COMMIT_EVERY: &str = "ai.commit_every";
const KEY_AI_WORKERS: &str = "ai.workers";
const KEY_AI_FREE_TIER: &str = "ai.free_tier";
const KEY_IMPORT_COMMIT_BATCH_SIZE: &str = "import.commit_batch_size";

/// Tauri event name streamed to the frontend during a unified backfill run.
pub const BACKFILL_PROGRESS_EVENT: &str = "backfill-progress";

/// Global app handle used to emit live progress events during a backfill run.
/// Mirrors the bulk-import pattern; set once at app setup.
static BACKFILL_APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

/// Register the app handle so backfill runs can stream progress events.
pub fn initialize_backfill_app_handle(app_handle: tauri::AppHandle) {
    let _ = BACKFILL_APP_HANDLE.set(app_handle);
}

fn emit_backfill_progress(progress: &backfill::BackfillProgress) {
    if let Some(handle) = BACKFILL_APP_HANDLE.get() {
        if let Err(error) = handle.emit(BACKFILL_PROGRESS_EVENT, progress) {
            tracing::error!("Failed to emit backfill progress event: {error}");
        }
    }
}

#[tauri::command]
pub async fn get_tagging_actions_view_model(
    state: State<'_, AppState>,
) -> Result<TaggingActionsViewModel, String> {
    let pool = state.db_pool()?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;

    let ai_vision_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_VISION_AUTO)
            .await
            .map_err(|e| e.to_string())?,
    );
    let ai_batch_size = get_setting_with_default(&mut conn, KEY_AI_BATCH_SIZE)
        .await
        .map_err(|e| e.to_string())?;
    let ai_delay = get_setting_with_default(&mut conn, KEY_AI_DELAY)
        .await
        .map_err(|e| e.to_string())?;
    let ai_commit_every = get_setting_with_default(&mut conn, KEY_AI_COMMIT_EVERY)
        .await
        .map_err(|e| e.to_string())?;
    let ai_workers = get_setting_with_default(&mut conn, KEY_AI_WORKERS)
        .await
        .map_err(|e| e.to_string())?;
    let ai_free_tier = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_FREE_TIER)
            .await
            .map_err(|e| e.to_string())?,
    );
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
        ai_vision_auto,
        ai_batch_size,
        ai_delay,
        ai_commit_every,
        ai_workers,
        ai_free_tier,
        import_commit_batch_size,
        default_batch_size: 100,
        default_commit_every: 100,
        default_workers: if ai_free_tier { 2 } else { 4 },
        default_delay: if ai_free_tier { 10.0 } else { 5.0 },
    })
}

#[tauri::command]
pub async fn run_unified_backfill(
    state: State<'_, AppState>,
    request: backfill::UnifiedBackfillRequest,
) -> Result<backfill::UnifiedBackfillSummary, String> {
    let pool = state.db_pool()?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let google_api_key = get_setting_with_default(&mut conn, KEY_AI_GOOGLE_API_KEY)
        .await
        .map_err(|e| e.to_string())?;
    let has_api_key = !google_api_key.trim().is_empty();
    drop(conn);

    if let Some(ref actions) = request.actions {
        if let Some(ref tagging) = actions.tagging {
            if tagging.enabled.unwrap_or(true) {
                if let Some(ref modes) = tagging.modes {
                    let requests_ai = modes.iter().any(|m| m == "ai_vision");
                    if requests_ai && !has_api_key {
                        return Err(
                            "Google API key is required for Visual AI tagging. Please configure your API key in Admin -> Settings."
                                .to_string(),
                        );
                    }
                }
            }
        }
    }

    backfill::run_unified_backfill_with_progress(
        &state.db_pool()?,
        request,
        has_api_key,
        if has_api_key {
            Some(google_api_key)
        } else {
            None
        },
        &mut |progress| emit_backfill_progress(&progress),
    )
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
    backfill::get_backfill_log_entries(&state.db_pool()?, limit.unwrap_or(20)).await
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
            hoop_dimensions: None,
            fingerprinting: None,
        }),
        batch_size,
        commit_every: Some(100),
        workers: Some(1),
        delay_seconds: None,
        vision_delay_seconds: None,
    };
    backfill::run_unified_backfill(&state.db_pool()?, request, false)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn run_fingerprint_backfill(
    state: State<'_, AppState>,
    commit_every: Option<i64>,
) -> Result<fingerprint::FingerprintSummary, String> {
    fingerprint::run_fingerprint_backfill(&state.db_pool()?, commit_every.unwrap_or(100))
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

    let mode_order = auto_tagging::ordered_modes()
        .iter()
        .map(|mode| format!("{:?}", mode))
        .collect();

    Ok(TaggingActionPreview {
        enabled: auto_tagging::resolve_enabled(&precedence),
        mode_order,
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
#[path = "tagging_actions_tests.rs"]
mod tests;
