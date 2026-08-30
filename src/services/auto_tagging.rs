//! Shared Gemini auto-tagging engine, used by both the unified backfill (Tagging
//! Actions) and the bulk import flow. Owns the pure tier computation (tier1 →
//! tier2 → tier3 fallback), the free-tier defaults, the rate-limit (429) message,
//! and the batched tag writer. Callers provide the design inputs
//! (filename/filepath/image_data) and the DB pool for the batched write.

use crate::error::AppError;
use crate::services::gemini_client::GeminiClient;
use crate::services::tagging;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::time::sleep;

/// Auto-tagging orchestration contract (tier selection / precedence) used by the
/// Tagging Actions preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaggingTier {
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone, Default)]
pub struct TaggingPrecedence {
    pub request_override: Option<bool>,
    pub settings_default: Option<bool>,
    pub hard_default: bool,
}

pub fn resolve_enabled(precedence: &TaggingPrecedence) -> bool {
    precedence
        .request_override
        .or(precedence.settings_default)
        .unwrap_or(precedence.hard_default)
}

pub fn ordered_tiers() -> [TaggingTier; 3] {
    [TaggingTier::Tier1, TaggingTier::Tier2, TaggingTier::Tier3]
}

/// Tier configuration for auto-tagging, grouped to keep callers under clippy's
/// `too_many_arguments` limit.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TaggingTierOptions {
    pub tier1_enabled: bool,
    pub tier2_enabled: bool,
    pub tier3_enabled: bool,
    /// Seconds to wait between Tier 2 calls, but only when Tier 2 makes a real
    /// outbound Gemini request (`tier2_network`). For local-only Tier 2 the delay
    /// is skipped so large runs aren't throttled by a fixed sleep.
    pub tier2_delay_seconds: f64,
    pub tier3_delay_seconds: f64,
    /// Whether Tier 2 performs a real network call that needs rate-limit pacing.
    pub tier2_network: bool,
    /// Whether Tier 3 performs a real network call that needs rate-limit pacing.
    pub tier3_network: bool,
}

/// Paid-tier default concurrent workers.
pub(crate) const DEFAULT_WORKERS: i64 = 4;
/// Free-tier default workers (rate-limited keys — keep concurrency low).
pub(crate) const FREE_TIER_WORKERS: i64 = 2;
/// Paid-tier default delay (seconds) between Gemini requests: 0 — paid keys are
/// not rate-limited, so no artificial pacing is needed (concurrency is bounded by
/// Workers). The free tier uses a conservative delay instead.
pub(crate) const DEFAULT_DELAY_SECONDS: f64 = 0.0;
/// Free-tier default delay (seconds) so runs stay under ~15 requests/minute.
pub(crate) const FREE_TIER_DELAY_SECONDS: f64 = 10.0;

/// Default concurrent workers for a run. Free-tier keys are rate-limited, so a
/// lower concurrency (used when the field is blank) keeps the run under roughly
/// 15 requests/minute.
pub(crate) fn default_workers_for(free_tier: bool) -> i64 {
    if free_tier {
        FREE_TIER_WORKERS
    } else {
        DEFAULT_WORKERS
    }
}

/// Default seconds between Gemini requests. Paid keys aren't rate-limited, so the
/// paid default is no delay (0); only the free tier paces requests (`FREE_TIER_DELAY_SECONDS`
/// paired with `FREE_TIER_WORKERS`) to stay under the ~15 requests/minute limit.
pub(crate) fn default_delay_for(free_tier: bool) -> f64 {
    if free_tier {
        FREE_TIER_DELAY_SECONDS
    } else {
        DEFAULT_DELAY_SECONDS
    }
}

/// Compute the tagging suggestion for a single design WITHOUT writing to the
/// database. Returns the suggested tag descriptions and the tier that produced
/// them (1/2/3), or `None` if no tier produced a suggestion. The caller applies
/// the result later in a batched write (see `apply_tagging_batch`).
pub(crate) async fn compute_tiers_for_input(
    filename: &str,
    filepath: &str,
    image_data: Option<&[u8]>,
    valid_descriptions: &HashSet<String>,
    tier_options: &TaggingTierOptions,
    gemini: Option<&GeminiClient>,
) -> Result<Option<(Vec<String>, i64)>, AppError> {
    if tier_options.tier1_enabled {
        let tier1 = tagging::suggest_tier1_descriptions(filename, filepath, valid_descriptions);
        if !tier1.is_empty() {
            return Ok(Some((tier1, 1)));
        }
    }

    if tier_options.tier2_enabled {
        // The delay only paces a real outbound Gemini call. Local-only Tier 2
        // does not sleep, so large runs aren't throttled.
        if tier_options.tier2_network && tier_options.tier2_delay_seconds > 0.0 {
            sleep(Duration::from_secs_f64(tier_options.tier2_delay_seconds)).await;
        }
        let tier2 = if let Some(client) = gemini {
            client
                .suggest_tags_text(filename, valid_descriptions)
                .await?
        } else {
            suggest_tier2_descriptions(filename, filepath, valid_descriptions)
        };
        if !tier2.is_empty() {
            return Ok(Some((tier2, 2)));
        }
    }

    if tier_options.tier3_enabled && image_data.is_some() {
        if tier_options.tier3_network && tier_options.tier3_delay_seconds > 0.0 {
            sleep(Duration::from_secs_f64(tier_options.tier3_delay_seconds)).await;
        }
        let tier3 = if let Some(client) = gemini {
            client
                .suggest_tags_vision(filename, image_data.unwrap_or_default(), valid_descriptions)
                .await?
        } else {
            suggest_tier3_descriptions(filename, filepath, valid_descriptions)
        };
        if !tier3.is_empty() {
            return Ok(Some((tier3, 3)));
        }
    }

    Ok(None)
}

/// Local (offline) Tier 2 heuristic: match a description when every one of its
/// significant tokens appears in the combined filename + filepath.
pub(crate) fn suggest_tier2_descriptions(
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
) -> Vec<String> {
    let combined = format!(
        "{} {}",
        filename.to_ascii_lowercase(),
        filepath.to_ascii_lowercase()
    );
    let mut suggestions = Vec::new();

    for description in valid_descriptions {
        let tokenized = description
            .to_ascii_lowercase()
            .replace(['&', '-', '"'], " ");
        let desc_tokens: Vec<&str> = tokenized
            .split_whitespace()
            .filter(|token| token.len() > 2)
            .collect();
        if !desc_tokens.is_empty() && desc_tokens.iter().all(|token| combined.contains(token)) {
            suggestions.push(description.clone());
        }
    }

    if suggestions.is_empty() {
        for description in ["Don't Know", "Patterns", "Flowers", "Animals"] {
            if valid_descriptions.contains(description) {
                suggestions.push(description.to_string());
                break;
            }
        }
    }

    suggestions.sort();
    suggestions
}

/// Local (offline) Tier 3 heuristic: same as Tier 2, with a "Don't Know" fallback.
pub(crate) fn suggest_tier3_descriptions(
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
) -> Vec<String> {
    let mut tier3 = suggest_tier2_descriptions(filename, filepath, valid_descriptions);
    if tier3.is_empty() && valid_descriptions.contains("Don't Know") {
        tier3.push("Don't Know".to_string());
    }
    tier3
}

/// Message shown when a Gemini run aborts on a rate-limit (429). On the free tier
/// we stop hard (no retry) and tell the user roughly how long to wait, using the
/// API's `Retry-After` seconds when available. On paid tier we keep the existing
/// guidance (raise delay / lower workers).
pub(crate) fn rate_limit_message(error: &AppError, free_tier: bool) -> String {
    if !free_tier {
        return "Gemini tagging aborted: rate limit / quota exceeded (429). No further designs were tagged. Increase the AI delay or lower Workers in Settings. The backfill log has full details."
            .to_string();
    }
    let wait = crate::services::gemini_client::retry_after_seconds(error)
        .map(|seconds| {
            format!(
                "wait about {} minutes (per the API) or until tomorrow",
                seconds.div_ceil(60)
            )
        })
        .unwrap_or_else(|| "wait a few minutes or until tomorrow".to_string());
    format!(
        "Free-tier Gemini rate limit reached (429). No further designs were tagged. {wait} before retrying."
    )
}

/// Apply a set of computed tagging suggestions in a single SQLite transaction.
/// This is the actual commit batching: a batch of designs' writes share one
/// transaction (one journal + fsync) instead of one autocommit per statement.
pub(crate) async fn apply_tagging_batch(
    pool: &SqlitePool,
    image_tag_map: &HashMap<String, i64>,
    results: Vec<(i64, Vec<String>, i64)>,
) -> Result<(), AppError> {
    if results.is_empty() {
        return Ok(());
    }
    let mut tx = pool.begin().await.map_err(|e| {
        AppError::database(format!("failed to begin tagging batch transaction: {e}"))
    })?;

    for (design_id, descriptions, tier) in results {
        if descriptions.is_empty() {
            continue;
        }
        sqlx::query(
            "DELETE FROM design_tags
\t\t WHERE design_id = ?
\t\t   AND tag_id IN (SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'image')",
        )
        .bind(design_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::database(format!("failed to clear existing image tags: {e}")))?;

        for description in descriptions {
            if let Some(tag_id) = image_tag_map.get(&description) {
                sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
                    .bind(design_id)
                    .bind(*tag_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| AppError::database(format!("failed to insert image tag: {e}")))?;
            }
        }

        sqlx::query("UPDATE designs SET tagging_tier = ?, image_tags_verified = 0 WHERE id = ?")
            .bind(tier)
            .bind(design_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("failed to update tagging tier: {e}")))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::database(format!("failed to commit tagging batch: {e}")))?;
    Ok(())
}
