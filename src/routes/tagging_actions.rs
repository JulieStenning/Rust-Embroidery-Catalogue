use crate::services::auto_tagging;
use crate::services::backfill;
use crate::services::fingerprint;
use crate::settings;
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

    let has_google_api_key = std::env::var("GOOGLE_API_KEY")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);

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
    let has_api_key = std::env::var("GOOGLE_API_KEY")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    backfill::run_unified_backfill(&state.db, request, has_api_key).await
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
    clear_existing_stitching: Option<bool>,
    batch_size: Option<i64>,
) -> Result<backfill::UnifiedBackfillSummary, String> {
    let request = backfill::UnifiedBackfillRequest {
        actions: Some(backfill::UnifiedBackfillActions {
            tagging: None,
            stitching: Some(backfill::StitchingActionOptions {
                clear_existing_stitching,
                enabled: Some(true),
            }),
            images: None,
            color_counts: None,
            fingerprinting: None,
        }),
        batch_size,
        commit_every: Some(100),
        workers: Some(1),
        preview_3d: Some(true),
        delay_seconds: None,
        vision_delay_seconds: None,
    };
    backfill::run_unified_backfill(&state.db, request, false).await
}

#[tauri::command]
pub async fn run_fingerprint_backfill(
    state: State<'_, AppState>,
    commit_every: Option<i64>,
) -> Result<fingerprint::FingerprintSummary, String> {
    fingerprint::run_fingerprint_backfill(&state.db, commit_every.unwrap_or(100)).await
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
    let current = settings::get_setting(conn, key).await?;
    if let Some(setting) = current {
        return Ok(setting.value);
    }

    let fallback = "".to_string();
    sqlx::query(
        "INSERT INTO settings (key, value, description) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(&fallback)
    .bind("Tagging actions default")
    .execute(conn)
    .await?;
    Ok(fallback)
}

fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_tagging_action_prefers_request_override() {
        let preview = preview_tagging_action(TaggingActionRequest {
            request_override: Some(false),
            settings_default: Some(true),
        })
        .expect("preview works");

        assert!(!preview.enabled);
        assert_eq!(preview.tier_order, vec!["Tier1", "Tier2", "Tier3"]);
    }

    #[test]
    fn preview_tagging_action_falls_back_to_settings_default() {
        let preview = preview_tagging_action(TaggingActionRequest {
            request_override: None,
            settings_default: Some(true),
        })
        .expect("preview works");

        assert!(preview.enabled);
    }

    #[test]
    fn is_truthy_accepts_expected_variants() {
        assert!(is_truthy("true"));
        assert!(is_truthy("YES"));
        assert!(is_truthy("1"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy("0"));
    }
}
