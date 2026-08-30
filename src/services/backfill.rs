use crate::error::AppError;
use crate::services::auto_tagging::TaggingModeOptions;
use crate::services::design_metadata;
use crate::services::gemini_client::GeminiClient;
use crate::services::stitch_identifier;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinSet;
use tokio::time::interval;

static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

const TAG_ACTION_UNTAGGED: &str = "tag_untagged";
const TAG_ACTION_RETAG_ALL: &str = "retag_all";
const TAG_ACTION_RETAG_ALL_UNVERIFIED: &str = "retag_all_unverified";
const DEFAULT_BATCH_SIZE: i64 = 100;
const DEFAULT_COMMIT_EVERY: i64 = 100;
#[cfg(test)]
const LOG_DIR: &str = "logs";
const ERROR_LOG_FILE: &str = "backfill_errors.log";
const INFO_LOG_FILE: &str = "backfill_info.log";

#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedBackfillRequest {
    pub actions: Option<UnifiedBackfillActions>,
    pub batch_size: Option<i64>,
    pub commit_every: Option<i64>,
    pub workers: Option<i64>,
    pub delay_seconds: Option<f64>,
    pub vision_delay_seconds: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedBackfillActions {
    pub tagging: Option<TaggingActionOptions>,
    pub stitching: Option<StitchingActionOptions>,
    pub images: Option<ImageActionOptions>,
    pub color_counts: Option<ColorCountsActionOptions>,
    pub hoop_dimensions: Option<HoopDimensionsActionOptions>,
    pub fingerprinting: Option<FingerprintActionOptions>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaggingActionOptions {
    pub action: Option<String>,
    /// Tagging modes to run: `"path_rule"` (File & Folder Rules) and/or `"ai_vision"`
    /// (Visual AI). Visual AI additionally requires a configured Google API key.
    pub modes: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StitchingActionOptions {
    /// Values: "none", "unverified", or "all". Controls whether stitching
    /// tags are cleared before re-detection and which designs are affected.
    pub clear_stitching_mode: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageActionOptions {
    pub redo: Option<bool>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorCountsActionOptions {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoopDimensionsActionOptions {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FingerprintActionOptions {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnifiedBackfillSummary {
    pub processed: i64,
    pub errors: i64,
    pub stopped: bool,
    pub actions: Vec<String>,
    pub commit_every: i64,
    pub batch_size: i64,
    pub workers: i64,
    /// Number of stitching-tag rows in `design_tags` before the run started.
    pub stitching_tag_count_before: i64,
    /// Number of stitching-tag rows in `design_tags` after the run finished.
    pub stitching_tag_count_after: i64,
}

/// Live progress streamed to the frontend during a unified backfill run so the
/// Tagging Actions screen can show a "Processed N designs — <action>…" message
/// that updates after each commit. `stage` is one of `started`,
/// `batch_committed`, `stopped` or `completed`.
#[derive(Debug, Clone, Serialize)]
pub struct BackfillProgress {
    pub stage: String,
    pub processed: i64,
    pub errors: i64,
    pub current_action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StopBackfillResult {
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackfillLogEntry {
    pub level: String,
    pub message: String,
}

pub fn request_stop() -> StopBackfillResult {
    let already = STOP_REQUESTED.swap(true, Ordering::SeqCst);
    StopBackfillResult {
        status: if already {
            "already_stopping".to_string()
        } else {
            "stopping".to_string()
        },
    }
}

pub fn clear_stop_signal() {
    STOP_REQUESTED.store(false, Ordering::SeqCst);
}

/// Check whether a stop has been requested.  Used by sibling backfill modules
/// (e.g., `fingerprint`) that share the same atomic flag.
pub fn is_stop_requested() -> bool {
    STOP_REQUESTED.load(Ordering::SeqCst)
}

/// Low-level setter for the stop flag, exposed for use by sibling modules
/// that share this flag.  Prefer `request_stop()` for normal use.
pub fn stop_requested_store(value: bool) {
    STOP_REQUESTED.store(value, Ordering::SeqCst);
}

/// Build and forward a [`BackfillProgress`] snapshot to the caller's callback.
/// `processed`/`errors` are passed by value so the helper does not borrow the
/// counters and therefore never blocks their mutation between commits.
fn emit_progress(
    progress: &mut (dyn FnMut(BackfillProgress) + Send),
    stage: &str,
    current_action: &str,
    processed: i64,
    errors: i64,
) {
    progress(BackfillProgress {
        stage: stage.to_string(),
        processed,
        errors,
        current_action: current_action.to_string(),
    });
}

pub async fn run_unified_backfill(
    pool: &SqlitePool,
    request: UnifiedBackfillRequest,
    has_api_key: bool,
) -> Result<UnifiedBackfillSummary, AppError> {
    run_unified_backfill_with_progress(pool, request, has_api_key, None, &mut |_| {}).await
}

/// Same as [`run_unified_backfill`], but streams live [`BackfillProgress`]
/// updates to `progress` (e.g. after each commit) so the UI can display a
/// running status message, and accepts the actual API key (`api_key`) used to
/// drive real Gemini Visual AI calls when present. Callers that do not need live
/// progress or AI tagging can use the plain [`run_unified_backfill`] wrapper.
pub async fn run_unified_backfill_with_progress(
    pool: &SqlitePool,
    request: UnifiedBackfillRequest,
    has_api_key: bool,
    api_key: Option<String>,
    progress: &mut (dyn FnMut(BackfillProgress) + Send),
) -> Result<UnifiedBackfillSummary, AppError> {
    clear_stop_signal();
    truncate_logs_for_new_run()?;
    let stitching_tag_count_before = count_stitching_tags(pool).await?;

    let actions = request.actions.unwrap_or(UnifiedBackfillActions {
        tagging: Some(TaggingActionOptions {
            action: Some(TAG_ACTION_UNTAGGED.to_string()),
            modes: Some(vec!["path_rule".to_string()]),
            enabled: Some(true),
        }),
        stitching: None,
        images: None,
        color_counts: None,
        hoop_dimensions: None,
        fingerprinting: None,
    });

    let batch_size = resolve_i64_option(
        request.batch_size,
        get_i64_setting(pool, "ai.batch_size").await?,
        DEFAULT_BATCH_SIZE,
        1,
        100_000,
    );
    let commit_every = resolve_i64_option(
        request.commit_every,
        get_i64_setting(pool, "ai.commit_every").await?,
        DEFAULT_COMMIT_EVERY,
        1,
        100_000,
    );
    // Whether the configured Gemini API key is on the free tier (user-declared in
    // Settings). Free-tier keys are rate-limited (~15 req/min, ~1,500/day), so when
    // the tier is free a blank workers/delay default to a conservative pair, and a
    // 429 is a hard stop (never retried) with a clear "wait" message.
    let free_tier = get_string_setting(pool, "ai.free_tier")
        .await?
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "y"
            )
        })
        .unwrap_or(false);
    let workers = resolve_i64_option(
        request.workers,
        get_i64_setting(pool, "ai.workers").await?,
        default_workers_for(free_tier),
        1,
        32,
    );
    let visual_ai_delay_seconds = resolve_f64_option(
        request.vision_delay_seconds,
        get_f64_setting(pool, "ai.delay").await?,
        default_delay_for(free_tier),
        0.0,
        120.0,
    );

    tracing::info!(
        "Backfill run started batch_size={} commit_every={} workers={} visual_ai_delay={} api_key={}",
        batch_size, commit_every, workers, visual_ai_delay_seconds, has_api_key
    );
    log_info(format!(
		"Run started batch_size={} commit_every={} workers={} visual_ai_delay={} api_key={}",
		batch_size, commit_every, workers, visual_ai_delay_seconds, has_api_key
	));

    let mut processed: i64 = 0;
    let mut errors: i64 = 0;
    let mut actions_run: Vec<String> = Vec::new();
    let mut touched_design_ids = HashSet::<i64>::new();

    emit_progress(progress, "started", "backfill", processed, errors);

    if let Some(tagging_action) = actions.tagging {
        if tagging_action.enabled.unwrap_or(true) {
            actions_run.push("tagging".to_string());
            let mode = normalize_tag_mode(tagging_action.action.as_deref());
            let modes = normalize_modes(tagging_action.modes.as_deref(), has_api_key);
            let path_rule_enabled = modes.contains("path_rule");
            let visual_ai_enabled = modes.contains("ai_vision") && has_api_key;

            let image_tag_map = get_image_tag_lookup(pool).await?;
            let valid_descriptions = image_tag_map.keys().cloned().collect::<HashSet<String>>();
            // Shared (read-only) data for the concurrent worker tasks.
            let image_map = Arc::new(image_tag_map);
            let valid = Arc::new(valid_descriptions);
            let pool_arc = Arc::new(pool.clone());
            // Build the Gemini client only when a non-empty API key is present;
            // otherwise Visual AI falls back to a local heuristic and never sleeps.
            let gemini = api_key
                .as_ref()
                .map(|key| key.trim().to_string())
                .filter(|key| !key.is_empty())
                .map(GeminiClient::new)
                .map(Arc::new);
            let mode_options = TaggingModeOptions {
                path_rule_enabled,
                visual_ai_enabled,
                visual_ai_delay_seconds,
                // Pace real Gemini network calls only; local-only modes never sleep.
                visual_ai_network: gemini.is_some() && visual_ai_enabled,
            };
            let workers_usize = workers.max(1) as usize;
            let commit_every_usize = commit_every.max(1) as usize;

            // Resolve the Gemini model up front (fail fast) so a configured model
            // that has been retired/removed is detected before any designs are
            // processed, rather than mid-run after thousands of calls.
            if gemini.is_some() && visual_ai_enabled {
                let configured_model = get_string_setting(pool, "ai.gemini_model").await?;
                if let Some(client) = gemini.as_deref() {
                    let resolved = client
                        .resolve_model(configured_model.as_deref(), visual_ai_enabled)
                        .await?;
                    log_info(format!("Tagging using Gemini model: {resolved}"));
                }
            }

            // Page through ALL matching designs in ascending-id batches of
            // `batch_size`, processing each batch with up to `workers` concurrent
            // tasks. Previously the query ran once with `LIMIT batch_size` and a
            // serial loop, so an "ALL" mode like retag_all could only ever touch
            // the first batch and the Workers control was never used.
            let mut tagging_cursor: i64 = 0;
            let mut tagging_total: i64 = 0;
            loop {
                if STOP_REQUESTED.load(Ordering::SeqCst) {
                    log_info("Stop signal observed during tagging loop".to_string());
                    break;
                }

                let design_ids =
                    select_tagging_design_ids(pool, mode, batch_size, tagging_cursor).await?;
                if design_ids.is_empty() {
                    break;
                }
                if let Some(last) = design_ids.last() {
                    tagging_cursor = *last;
                }
                tagging_total += design_ids.len() as i64;
                log_info(format!(
                    "Tagging batch action={} batch_candidates={} cumulative={} modes={:?}",
                    mode,
                    design_ids.len(),
                    tagging_total,
                    modes
                ));

                // Run this batch's designs concurrently, bounded by `workers`.
                // Worker tasks only COMPUTE suggestions (read-only); the single
                // writer below applies them in one transaction per `commit_every`
                // so a 100k-design run makes ~100k/commit_every commits instead
                // of one autocommit per statement.
                let mut set = JoinSet::new();
                let mut remaining = design_ids.iter().copied();
                for _ in 0..workers_usize {
                    if let Some(design_id) = remaining.next() {
                        spawn_tagging_task(
                            &mut set,
                            design_id,
                            pool_arc.clone(),
                            valid.clone(),
                            mode_options,
                            gemini.clone(),
                        );
                    }
                }
                // Completed suggestions awaiting a batched write.
                let mut pending: Vec<(i64, Vec<String>, String)> = Vec::new();
                // Join results, but keep the loop stop-interruptible so a Stop
                // request aborts the current batch immediately instead of
                // draining it. A short interval polls STOP_REQUESTED while
                // tasks are in flight (e.g. mid Gemini delay); on stop we
                // cancel every in-flight task and break. Suggestions that
                // already finished are flushed in a transaction right after the
                // loop, so nothing completed is lost.
                let mut stop_poll = interval(Duration::from_millis(100));
                loop {
                    tokio::select! {
                        biased;
                        _ = stop_poll.tick() => {
                            if STOP_REQUESTED.load(Ordering::SeqCst) {
                                set.abort_all();
                                log_info("Stop signal observed; aborting current tagging batch".to_string());
                                break;
                            }
                        }
                        joined = set.join_next() => {
                            match joined {
                                None => break, // batch fully drained
                                Some(joined) => {
                                    let before = processed;
                                    match joined {
                                        Ok((design_id, Ok(Some((descriptions, mode))))) => {
                                            touched_design_ids.insert(design_id);
                                            processed += 1;
                                            pending.push((design_id, descriptions, mode));
                                        }
                                        Ok((design_id, Ok(None))) => {
                                            // No mode produced a suggestion — nothing to write.
                                            touched_design_ids.insert(design_id);
                                            processed += 1;
                                        }
                                        Ok((design_id, Err(error))) => {
                                            touched_design_ids.insert(design_id);
                                            processed += 1;
                                            if crate::services::gemini_client::is_rate_limit_error(&error) {
                                                // A 429/quota error is a run-level failure — every
                                                // remaining design would keep failing. Commit what
                                                // we already computed, then abort and point the
                                                // user at the log.
                                                flush_tagging_batch(
                                                    pool,
                                                    &image_map,
                                                    std::mem::take(&mut pending),
                                                )
                                                .await?;
                                                log_error(format!(
                                                    "Tagging aborted on design_id={} due to Gemini rate limit: {error}",
                                                    design_id
                                                ));
                                                return Err(AppError::invalid_input(
                                                    free_tier_rate_limit_message(&error, free_tier),
                                                ));
                                            }
                                            errors += 1;
                                            log_error(format!(
                                                "Tagging failed design_id={} error={}",
                                                design_id, error
                                            ));
                                        }
                                        Err(join_err) => {
                                            errors += 1;
                                            log_error(format!("Tagging task failed: {join_err}"));
                                        }
                                    }
                                    if processed > before {
                                        emit_progress(progress, "processing", "tagging", processed, errors);
                                    }
                                    // Commit every `commit_every` successful writes in a single
                                    // transaction, then report progress.
                                    if pending.len() >= commit_every_usize {
                                        flush_tagging_batch(pool, &image_map, std::mem::take(&mut pending)).await?;
                                    }
                                    if processed % commit_every == 0 {
                                        emit_progress(
                                            progress,
                                            "batch_committed",
                                            "tagging",
                                            processed,
                                            errors,
                                        );
                                    }
                                    if STOP_REQUESTED.load(Ordering::SeqCst) {
                                        set.abort_all();
                                        log_info("Stop signal observed; aborting current tagging batch".to_string());
                                        break;
                                    }
                                    if let Some(design_id) = remaining.next() {
                                        spawn_tagging_task(
                                            &mut set,
                                            design_id,
                                            pool_arc.clone(),
                                            valid.clone(),
                                            mode_options,
                                            gemini.clone(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                // Flush any remaining suggestions (incl. on stop) so the tags
                // computed so far are committed before the next batch / run end.
                if !pending.is_empty() {
                    flush_tagging_batch(pool, &image_map, pending).await?;
                }
            }
        }
    }

    if let Some(stitching_action) = actions.stitching {
        if stitching_action.enabled.unwrap_or(true) {
            actions_run.push("stitching".to_string());

            let stitching_prior_count = count_stitching_tags(pool).await?;
            log_info(format!(
                "Stitching section: {} stitching tags existing before changes",
                stitching_prior_count
            ));

            let clear_mode = stitching_action
                .clear_stitching_mode
                .as_deref()
                .unwrap_or("none");
            if clear_mode == "unverified" || clear_mode == "all" {
                let cleared = clear_stitching_tags(pool, clear_mode).await?;
                let cleared_count = cleared.len();
                touched_design_ids.extend(cleared);
                let after_clear_count = count_stitching_tags(pool).await?;
                log_info(format!(
                    "After clear ({}): {} tags remaining, {} design ids cleared",
                    clear_mode, after_clear_count, cleared_count
                ));
            }
            let stitching_tag_lookup = get_stitching_tag_lookup(pool).await?;
            let valid_stitching_descriptions = stitching_tag_lookup
                .keys()
                .cloned()
                .collect::<HashSet<String>>();
            let default_stitching_tag_id = get_default_stitching_tag_id(pool).await?;
            // Page through ALL matching designs in ascending-id batches of
            // `batch_size`, mirroring the fingerprint backfill.
            let mut stitching_cursor: i64 = 0;
            let mut stitching_total: i64 = 0;
            loop {
                if STOP_REQUESTED.load(Ordering::SeqCst) {
                    break;
                }
                let stitching_candidates =
                    select_stitching_candidates(pool, batch_size, stitching_cursor).await?;
                if stitching_candidates.is_empty() {
                    break;
                }
                if let Some(last) = stitching_candidates.last() {
                    stitching_cursor = last.id;
                }
                stitching_total += stitching_candidates.len() as i64;
                log_info(format!(
                    "{} stitching candidates selected for detection (batch={} cumulative={})",
                    stitching_candidates.len(),
                    stitching_candidates.len(),
                    stitching_total
                ));
                for candidate in stitching_candidates {
                    if STOP_REQUESTED.load(Ordering::SeqCst) {
                        break;
                    }
                    touched_design_ids.insert(candidate.id);
                    processed += 1;

                    let resolved_design_path = resolve_stored_design_path(&candidate.filepath);
                    let detected_descriptions =
                        stitch_identifier::suggest_stitching_from_pattern_file(
                            &resolved_design_path.to_string_lossy(),
                            &candidate.filename,
                            &candidate.filepath,
                            &valid_stitching_descriptions,
                            Some(0.70),
                        );

                    let mut detected_tag_ids = Vec::new();
                    for description in &detected_descriptions {
                        if let Some(tag_id) = stitching_tag_lookup.get(description) {
                            detected_tag_ids.push(*tag_id);
                        }
                    }

                    if detected_tag_ids.is_empty() {
                        if let Some(tag_id) = default_stitching_tag_id {
                            detected_tag_ids.push(tag_id);
                        }
                    }

                    if let Err(error) =
                        apply_stitching_tags(pool, candidate.id, &detected_tag_ids).await
                    {
                        errors += 1;
                        log_error(format!(
                            "Stitching update failed design_id={} error={}",
                            candidate.id, error
                        ));
                    } else if !detected_descriptions.is_empty() {
                        log_info(format!(
                            "Stitching detected design_id={} tags={:?}",
                            candidate.id, detected_descriptions
                        ));
                    }

                    emit_progress(progress, "processing", "stitching", processed, errors);
                    if processed % commit_every == 0 {
                        emit_progress(progress, "batch_committed", "stitching", processed, errors);
                    }
                }
            }

            let stitching_final_count = count_stitching_tags(pool).await?;
            log_info(format!(
                "Stitching section complete: {} stitching tags after processing",
                stitching_final_count
            ));
        }
    }

    if let Some(images_action) = actions.images {
        if images_action.enabled.unwrap_or(true) {
            actions_run.push("images".to_string());
            let mut image_cursor: i64 = 0;
            loop {
                if STOP_REQUESTED.load(Ordering::SeqCst) {
                    break;
                }
                let image_candidates = select_image_candidates(
                    pool,
                    images_action.redo.unwrap_or(false),
                    batch_size,
                    image_cursor,
                )
                .await?;
                if image_candidates.is_empty() {
                    break;
                }
                if let Some(last) = image_candidates.last() {
                    image_cursor = *last;
                }
                for design_id in image_candidates {
                    if STOP_REQUESTED.load(Ordering::SeqCst) {
                        break;
                    }
                    touched_design_ids.insert(design_id);
                    processed += 1;

                    if images_action.redo.unwrap_or(false) {
                        let _ = clear_image_fields(pool, design_id).await;
                    }

                    if let Err(error) = generate_and_store_preview(pool, design_id).await {
                        errors += 1;
                        log_error(format!(
                            "Image action failed design_id={} error={}",
                            design_id, error
                        ));
                    }

                    emit_progress(progress, "processing", "images", processed, errors);
                    if processed % commit_every == 0 {
                        emit_progress(progress, "batch_committed", "images", processed, errors);
                    }
                }
            }
        }
    }

    if let Some(color_counts_action) = actions.color_counts {
        if color_counts_action.enabled.unwrap_or(true) {
            actions_run.push("color_counts".to_string());
            let mut color_cursor: i64 = 0;
            loop {
                if STOP_REQUESTED.load(Ordering::SeqCst) {
                    break;
                }
                let color_candidates =
                    select_color_count_candidates(pool, batch_size, color_cursor).await?;
                if color_candidates.is_empty() {
                    break;
                }
                if let Some(last) = color_candidates.last() {
                    color_cursor = *last;
                }
                for design_id in color_candidates {
                    if STOP_REQUESTED.load(Ordering::SeqCst) {
                        break;
                    }
                    touched_design_ids.insert(design_id);
                    processed += 1;
                    if let Err(error) = update_color_counts_only(pool, design_id).await {
                        errors += 1;
                        log_error(format!(
                            "Colour-count action failed design_id={} error={}",
                            design_id, error
                        ));
                    }

                    emit_progress(progress, "processing", "color_counts", processed, errors);
                    if processed % commit_every == 0 {
                        emit_progress(
                            progress,
                            "batch_committed",
                            "color_counts",
                            processed,
                            errors,
                        );
                    }
                }
            }
        }
    }

    if let Some(hoop_dimensions_action) = actions.hoop_dimensions {
        if hoop_dimensions_action.enabled.unwrap_or(true) {
            actions_run.push("hoop_dimensions".to_string());
            let mut hoop_cursor: i64 = 0;
            loop {
                if STOP_REQUESTED.load(Ordering::SeqCst) {
                    break;
                }
                let hoop_candidates =
                    select_hoop_dimension_candidates(pool, batch_size, hoop_cursor).await?;
                if hoop_candidates.is_empty() {
                    break;
                }
                if let Some(last) = hoop_candidates.last() {
                    hoop_cursor = *last;
                }
                for design_id in hoop_candidates {
                    if STOP_REQUESTED.load(Ordering::SeqCst) {
                        break;
                    }
                    touched_design_ids.insert(design_id);
                    processed += 1;
                    if let Err(error) = update_hoop_dimensions_only(pool, design_id).await {
                        errors += 1;
                        log_error(format!(
                            "Hoop/dimension action failed design_id={} error={}",
                            design_id, error
                        ));
                    }

                    emit_progress(progress, "processing", "hoop_dimensions", processed, errors);
                    if processed % commit_every == 0 {
                        emit_progress(
                            progress,
                            "batch_committed",
                            "hoop_dimensions",
                            processed,
                            errors,
                        );
                    }
                }
            }
        }
    }

    if let Some(fp_action) = actions.fingerprinting {
        if fp_action.enabled.unwrap_or(true) {
            actions_run.push("fingerprinting".to_string());
            let fp_summary =
                crate::services::fingerprint::run_fingerprint_backfill(pool, commit_every)
                    .await
                    .map_err(|e| AppError::database(format!("Fingerprint backfill failed: {e}")))?;
            processed += fp_summary.processed;
            errors += fp_summary.errors;
            if fp_summary.stopped {
                log_info("Fingerprint backfill stopped by user request".to_string());
            }
        }
    }

    if processed == 0 {
        processed = touched_design_ids.len() as i64;
    }

    let stopped = STOP_REQUESTED.load(Ordering::SeqCst);
    log_info(format!(
        "Run complete processed={} errors={} stopped={} actions={:?}",
        processed, errors, stopped, actions_run
    ));

    emit_progress(
        progress,
        if stopped { "stopped" } else { "completed" },
        "backfill",
        processed,
        errors,
    );

    let stitching_tag_count_after = count_stitching_tags(pool).await?;

    Ok(UnifiedBackfillSummary {
        processed,
        errors,
        stopped,
        actions: actions_run,
        commit_every,
        batch_size,
        workers,
        stitching_tag_count_before,
        stitching_tag_count_after,
    })
}

pub async fn get_backfill_log_entries(
    _pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<BackfillLogEntry>, String> {
    let bounded = limit.clamp(1, 200) as usize;
    let mut entries = Vec::new();

    entries.extend(read_log_tail(&info_log_path(), "info", bounded).map_err(|e| e.to_string())?);
    entries.extend(read_log_tail(&error_log_path(), "error", bounded).map_err(|e| e.to_string())?);

    if entries.len() > bounded {
        entries = entries[entries.len() - bounded..].to_vec();
    }

    Ok(entries)
}

fn normalize_tag_mode(raw: Option<&str>) -> &str {
    match raw
        .unwrap_or(TAG_ACTION_UNTAGGED)
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        TAG_ACTION_RETAG_ALL => TAG_ACTION_RETAG_ALL,
        TAG_ACTION_RETAG_ALL_UNVERIFIED => TAG_ACTION_RETAG_ALL_UNVERIFIED,
        _ => TAG_ACTION_UNTAGGED,
    }
}

fn normalize_modes(raw: Option<&[String]>, has_api_key: bool) -> HashSet<String> {
    let mut modes = HashSet::new();
    modes.insert("path_rule".to_string());

    if let Some(values) = raw {
        for mode in values {
            let m = mode.trim().to_ascii_lowercase();
            if m == "path_rule" || (m == "ai_vision" && has_api_key) {
                modes.insert(m);
            }
        }
    }

    modes
}

async fn get_image_tag_lookup(pool: &SqlitePool) -> Result<HashMap<String, i64>, AppError> {
    let rows = sqlx::query(
		"SELECT id, description FROM tags WHERE lower(COALESCE(tag_group, '')) = 'image' ORDER BY description COLLATE NOCASE",
	)
	.fetch_all(pool)
	.await
	.map_err(|e| AppError::database(format!("failed to load image tag lookup: {e}")))?;

    let mut map = HashMap::new();
    for row in rows {
        let tag_id: i64 = row
            .try_get("id")
            .map_err(|e| AppError::database(format!("failed to read tag id: {e}")))?;
        let description: String = row
            .try_get("description")
            .map_err(|e| AppError::database(format!("failed to read tag description: {e}")))?;
        map.insert(description, tag_id);
    }

    Ok(map)
}

async fn select_tagging_design_ids(
    pool: &SqlitePool,
    mode: &str,
    limit: i64,
    min_id: i64,
) -> Result<Vec<i64>, AppError> {
    // `min_id` is a keyset cursor so the caller can page through ALL matching
    // designs in ascending-id batches (see the pagination loop in
    // `run_unified_backfill`). Without it, an "ALL" mode such as `retag_all`
    // re-selects the same first `limit` rows on every run.
    let sql = match mode {
        TAG_ACTION_RETAG_ALL => "SELECT id FROM designs WHERE id > ? ORDER BY id ASC LIMIT ?",
        TAG_ACTION_RETAG_ALL_UNVERIFIED => {
            "SELECT id FROM designs WHERE COALESCE(image_tags_verified, 0) = 0 AND id > ? ORDER BY id ASC LIMIT ?"
        }
        _ => {
            "SELECT d.id
			 FROM designs d
			 WHERE NOT EXISTS (
			   SELECT 1
			   FROM design_tags dt
			   JOIN tags t ON t.id = dt.tag_id
			   WHERE dt.design_id = d.id AND lower(COALESCE(t.tag_group, '')) = 'image'
			 )
			 AND d.id > ?
			 ORDER BY d.id ASC
			 LIMIT ?"
        }
    };

    let rows = sqlx::query(sql)
        .bind(min_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to select tagging design ids: {e}")))?;

    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        ids.push(
            row.try_get::<i64, _>("id").map_err(|e| {
                AppError::database(format!("failed to read tagging design id: {e}"))
            })?,
        );
    }
    Ok(ids)
}

/// Output of a single compute-only tagging worker task: the design id plus the
/// computed `(descriptions, mode)` suggestion, or `None` (no suggestion) / `Err`.
type TaggingWorkerOutput = (i64, Result<Option<(Vec<String>, String)>, AppError>);

/// Spawn a single concurrent tagging task for `design_id`. Workers only COMPUTE
/// the suggestion (a read-only operation — no DB writes); the caller applies the
/// returned result later in a batched transaction. Shared read-only data (valid
/// descriptions) and the connection pool are passed via cheap `Arc` clones so each
/// worker can run independently. Returns `Some((descriptions, mode))` when a mode
/// produced a suggestion, or `None` when none did.
fn spawn_tagging_task(
    set: &mut JoinSet<TaggingWorkerOutput>,
    design_id: i64,
    pool: Arc<SqlitePool>,
    valid_descriptions: Arc<HashSet<String>>,
    mode_options: TaggingModeOptions,
    gemini: Option<Arc<GeminiClient>>,
) {
    set.spawn(async move {
        let result = compute_design_tagging(
            &pool,
            design_id,
            &valid_descriptions,
            &mode_options,
            gemini.as_deref(),
        )
        .await;
        (design_id, result)
    });
}

/// Compute the tagging suggestion for a single design WITHOUT writing to the
/// database. Returns the suggested tag descriptions and the mode that produced
/// them (`path_rule`/`ai_vision`), or `None` if no mode produced a suggestion. The
/// caller applies the result later in a batched transaction (see the tagging
/// loop), so a run can commit thousands of designs' writes in a handful of
/// transactions.
async fn compute_design_tagging(
    pool: &SqlitePool,
    design_id: i64,
    valid_descriptions: &HashSet<String>,
    mode_options: &TaggingModeOptions,
    gemini: Option<&GeminiClient>,
) -> Result<Option<(Vec<String>, String)>, AppError> {
    // Only fetch the preview image when Visual AI (vision) is enabled, so large
    // BLOBs aren't read for every design in a File & Folder Rules-only run.
    let select_sql = if mode_options.visual_ai_enabled {
        "SELECT filename, filepath, image_data FROM designs WHERE id = ?"
    } else {
        "SELECT filename, filepath FROM designs WHERE id = ?"
    };
    let row = sqlx::query(select_sql)
        .bind(design_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to read design row for tagging: {e}")))?;

    let Some(row) = row else {
        return Ok(None);
    };

    let filename: String = row
        .try_get("filename")
        .map_err(|e| AppError::database(format!("failed to read filename: {e}")))?;
    let filepath: String = row
        .try_get("filepath")
        .map_err(|e| AppError::database(format!("failed to read filepath: {e}")))?;
    let image_data: Option<Vec<u8>> = if mode_options.visual_ai_enabled {
        row.try_get("image_data")
            .map_err(|e| AppError::database(format!("failed to read image data: {e}")))?
    } else {
        None
    };

    crate::services::auto_tagging::compute_tags_for_input(
        &filename,
        &filepath,
        image_data.as_deref(),
        valid_descriptions,
        mode_options,
        gemini,
    )
    .await
}

#[cfg(test)]
fn suggest_visual_ai_descriptions(
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
) -> Vec<String> {
    crate::services::auto_tagging::suggest_visual_ai_descriptions(
        filename,
        filepath,
        valid_descriptions,
    )
}

/// Apply a set of computed tagging suggestions in a single SQLite transaction.
/// This is the actual commit batching: `commit_every` designs' writes share one
/// transaction (one journal + fsync) instead of one autocommit per statement.
async fn flush_tagging_batch(
    pool: &SqlitePool,
    image_tag_map: &HashMap<String, i64>,
    results: Vec<(i64, Vec<String>, String)>,
) -> Result<(), AppError> {
    crate::services::auto_tagging::apply_tagging_batch(pool, image_tag_map, results).await
}

async fn count_stitching_tags(pool: &SqlitePool) -> Result<i64, AppError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
		 FROM design_tags dt
		 JOIN tags t ON t.id = dt.tag_id
		 WHERE lower(COALESCE(t.tag_group, '')) = 'stitching'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to count stitching tags: {e}")))?;
    Ok(count)
}

async fn clear_stitching_tags(pool: &SqlitePool, mode: &str) -> Result<Vec<i64>, AppError> {
    // Determine which designs have stitching tags to clear.
    let select_sql = if mode == "all" {
        "SELECT DISTINCT dt.design_id AS id
		 FROM design_tags dt
		 JOIN tags t ON t.id = dt.tag_id
		 WHERE lower(COALESCE(t.tag_group, '')) = 'stitching'"
    } else {
        "SELECT DISTINCT dt.design_id AS id
		 FROM design_tags dt
		 JOIN designs d ON d.id = dt.design_id
		 JOIN tags t ON t.id = dt.tag_id
		 WHERE lower(COALESCE(t.tag_group, '')) = 'stitching'
		   AND COALESCE(d.stitching_tags_verified, 0) = 0"
    };

    let rows = sqlx::query(select_sql)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to load stitching tag candidates: {e}")))?;

    // Delete stitching tags according to the chosen mode.
    let delete_sql = if mode == "all" {
        "DELETE FROM design_tags
		 WHERE tag_id IN (SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching')"
    } else {
        "DELETE FROM design_tags
		 WHERE design_id IN (SELECT id FROM designs WHERE COALESCE(stitching_tags_verified, 0) = 0)
		   AND tag_id IN (SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching')"
    };

    sqlx::query(delete_sql)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to clear stitching tags: {e}")))?;

    let mut ids = Vec::new();
    for row in rows {
        ids.push(
            row.try_get::<i64, _>("id").map_err(|e| {
                AppError::database(format!("failed to read stitching-design id: {e}"))
            })?,
        );
    }
    Ok(ids)
}

async fn select_stitching_candidates(
    pool: &SqlitePool,
    limit: i64,
    min_id: i64,
) -> Result<Vec<StitchingCandidate>, AppError> {
    let rows = sqlx::query(
        "SELECT d.id, d.filename, d.filepath
		 FROM designs d
		 WHERE NOT EXISTS (
		   SELECT 1
		   FROM design_tags dt
		   JOIN tags t ON t.id = dt.tag_id
		   WHERE dt.design_id = d.id AND lower(COALESCE(t.tag_group, '')) = 'stitching'
		 )
		 AND d.id > ?
		 ORDER BY d.id ASC
		 LIMIT ?",
    )
    .bind(min_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to select stitching candidates: {e}")))?;

    let mut candidates = Vec::new();
    for row in rows {
        candidates.push(StitchingCandidate {
            id: row.try_get::<i64, _>("id").map_err(|e| {
                AppError::database(format!("failed to read stitching candidate id: {e}"))
            })?,
            filename: row.try_get::<String, _>("filename").map_err(|e| {
                AppError::database(format!("failed to read stitching filename: {e}"))
            })?,
            filepath: row.try_get::<String, _>("filepath").map_err(|e| {
                AppError::database(format!("failed to read stitching filepath: {e}"))
            })?,
        });
    }
    Ok(candidates)
}

#[derive(Debug, Clone)]
struct StitchingCandidate {
    id: i64,
    filename: String,
    filepath: String,
}

async fn get_stitching_tag_lookup(pool: &SqlitePool) -> Result<HashMap<String, i64>, AppError> {
    let rows = sqlx::query(
        "SELECT id, description
		 FROM tags
		 WHERE lower(COALESCE(tag_group, '')) = 'stitching'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to load stitching tag lookup: {e}")))?;

    let mut map = HashMap::new();
    for row in rows {
        let tag_id = row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::database(format!("failed to read stitching tag id: {e}")))?;
        let description = row.try_get::<String, _>("description").map_err(|e| {
            AppError::database(format!("failed to read stitching tag description: {e}"))
        })?;
        map.insert(description, tag_id);
    }
    Ok(map)
}

async fn get_default_stitching_tag_id(pool: &SqlitePool) -> Result<Option<i64>, AppError> {
    let row = sqlx::query(
		"SELECT id
		 FROM tags
		 WHERE lower(COALESCE(tag_group, '')) = 'stitching'
		 ORDER BY CASE WHEN lower(description) = 'line outline' THEN 0 ELSE 1 END, description COLLATE NOCASE
		 LIMIT 1",
	)
	.fetch_optional(pool)
	.await
	.map_err(|e| AppError::database(format!("failed to read default stitching tag id: {e}")))?;

    Ok(row.and_then(|record| record.try_get::<i64, _>("id").ok()))
}

async fn apply_stitching_tags(
    pool: &SqlitePool,
    design_id: i64,
    tag_ids: &[i64],
) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM design_tags
		 WHERE design_id = ?
		   AND tag_id IN (SELECT id FROM tags WHERE lower(COALESCE(tag_group, '')) = 'stitching')",
    )
    .bind(design_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to clear unverified stitching tags: {e}")))?;

    for tag_id in tag_ids {
        sqlx::query("INSERT OR IGNORE INTO design_tags (design_id, tag_id) VALUES (?, ?)")
            .bind(design_id)
            .bind(*tag_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::database(format!("failed to insert stitching tag: {e}")))?;
    }

    Ok(())
}

async fn select_image_candidates(
    pool: &SqlitePool,
    redo: bool,
    limit: i64,
    min_id: i64,
) -> Result<Vec<i64>, AppError> {
    let sql = if redo {
        "SELECT id FROM designs WHERE id > ? ORDER BY id ASC LIMIT ?"
    } else {
        "SELECT id FROM designs WHERE image_data IS NULL AND id > ? ORDER BY id ASC LIMIT ?"
    };

    let rows = sqlx::query(sql)
        .bind(min_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to select image candidates: {e}")))?;

    let mut ids = Vec::new();
    for row in rows {
        ids.push(
            row.try_get::<i64, _>("id").map_err(|e| {
                AppError::database(format!("failed to read image candidate id: {e}"))
            })?,
        );
    }
    Ok(ids)
}

async fn clear_image_fields(pool: &SqlitePool, design_id: i64) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE designs
		 SET image_data = NULL,
		     image_type = NULL,
		     width_mm = NULL,
		     height_mm = NULL
		 WHERE id = ?",
    )
    .bind(design_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to clear image fields: {e}")))?;
    Ok(())
}

async fn generate_and_store_preview(pool: &SqlitePool, design_id: i64) -> Result<(), AppError> {
    let row = sqlx::query("SELECT filepath FROM designs WHERE id = ?")
        .bind(design_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to read filepath for preview: {e}")))?;

    let Some(row) = row else {
        return Ok(());
    };

    let filepath: String = row
        .try_get("filepath")
        .map_err(|e| AppError::database(format!("failed to read filepath: {e}")))?;
    let resolved_path = resolve_stored_design_path(&filepath);
    let result = design_metadata::parse_design_file(&resolved_path)
        .map_err(|e| AppError::invalid_input(e))?;

    sqlx::query(
        "UPDATE designs
		 SET image_data = ?,
		     image_type = ?,
		     width_mm = ?,
		     height_mm = ?,
		     stitch_count = COALESCE(?, stitch_count),
		     color_count = COALESCE(?, color_count),
		     color_change_count = COALESCE(?, color_change_count)
		 WHERE id = ?",
    )
    .bind(result.image_data)
    .bind(result.image_type)
    .bind(result.width_mm)
    .bind(result.height_mm)
    .bind(result.stitch_count)
    .bind(result.color_count)
    .bind(result.color_change_count)
    .bind(design_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to store generated preview: {e}")))?;

    Ok(())
}

/// Convert a stored DB filepath (e.g. "/MachineEmbroideryDesigns/foo/bar.pes")
/// into an absolute on-disk path under the resolved designs base directory.
///
/// Handles:
/// - `/MachineEmbroideryDesigns/...` (canonical stored form with leading slash)
/// - `MachineEmbroideryDesigns/...` (no leading slash)
/// - Bare relative paths â†’ joined under the designs base directory
/// - Truly absolute paths â†’ returned as-is (e.g. legacy absolute filepaths)
fn resolve_stored_design_path(stored_filepath: &str) -> PathBuf {
    let designs_base = crate::paths::resolve_app_paths()
        .map(|paths| paths.embroidery_designs_dir)
        .unwrap_or_else(|_| PathBuf::from("MachineEmbroideryDesigns"));

    let normalized = stored_filepath.trim().replace('\\', "/");
    if normalized.is_empty() {
        return designs_base;
    }

    let cleaned = normalized.trim_start_matches('/');
    let cleaned_lower = cleaned.to_ascii_lowercase();
    if cleaned_lower == "machineembroiderydesigns"
        || cleaned_lower.starts_with("machineembroiderydesigns/")
    {
        // "/MachineEmbroideryDesigns/..." â†’ "<data_root>/MachineEmbroideryDesigns/..."
        let data_root = designs_base
            .parent()
            .map(|value| value.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        return data_root.join(cleaned);
    }

    let candidate = PathBuf::from(&normalized);
    if candidate.is_absolute() {
        return candidate;
    }

    designs_base.join(cleaned)
}

async fn select_color_count_candidates(
    pool: &SqlitePool,
    limit: i64,
    min_id: i64,
) -> Result<Vec<i64>, AppError> {
    let rows = sqlx::query(
        "SELECT id
		 FROM designs
		 WHERE (stitch_count IS NULL OR color_count IS NULL OR color_change_count IS NULL)
		 AND id > ?
		 ORDER BY id ASC
		 LIMIT ?",
    )
    .bind(min_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to select image candidates: {e}")))?;

    let mut ids = Vec::new();
    for row in rows {
        ids.push(
            row.try_get::<i64, _>("id").map_err(|e| {
                AppError::database(format!("failed to read image candidate id: {e}"))
            })?,
        );
    }
    Ok(ids)
}

async fn update_color_counts_only(pool: &SqlitePool, design_id: i64) -> Result<(), AppError> {
    let row = sqlx::query("SELECT filepath FROM designs WHERE id = ?")
        .bind(design_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            AppError::database(format!("failed to read filepath for color counts: {e}"))
        })?;

    let Some(row) = row else {
        return Ok(());
    };

    let filepath: String = row
        .try_get("filepath")
        .map_err(|e| AppError::database(format!("failed to read filepath: {e}")))?;
    let resolved_path = resolve_stored_design_path(&filepath);
    let result = design_metadata::parse_design_file(&resolved_path)
        .map_err(|e| AppError::invalid_input(e))?;

    sqlx::query(
        "UPDATE designs
		 SET stitch_count = COALESCE(?, stitch_count),
		     color_count = COALESCE(?, color_count),
		     color_change_count = COALESCE(?, color_change_count)
		 WHERE id = ?",
    )
    .bind(result.stitch_count)
    .bind(result.color_count)
    .bind(result.color_change_count)
    .bind(design_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to update color counts: {e}")))?;

    Ok(())
}

async fn get_i64_setting(pool: &SqlitePool, key: &str) -> Result<Option<i64>, AppError> {
    let value = sqlx::query("SELECT value FROM settings WHERE key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to read integer setting {key}: {e}")))?
        .and_then(|row| row.try_get::<String, _>("value").ok());

    Ok(value.and_then(|raw| raw.trim().parse::<i64>().ok()))
}

async fn get_string_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, AppError> {
    let value = sqlx::query("SELECT value FROM settings WHERE key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to read string setting {key}: {e}")))?
        .and_then(|row| row.try_get::<String, _>("value").ok());

    Ok(value)
}

/// Build the message shown when a Gemini run aborts on a rate-limit (429).
/// On the free tier we stop hard (no retry) and tell the user roughly how long
/// to wait, using the API's `Retry-After` seconds when available. On paid tier
/// we keep the existing guidance (raise delay / lower workers).
fn free_tier_rate_limit_message(error: &AppError, free_tier: bool) -> String {
    crate::services::auto_tagging::rate_limit_message(error, free_tier)
}

fn default_workers_for(free_tier: bool) -> i64 {
    crate::services::auto_tagging::default_workers_for(free_tier)
}

fn default_delay_for(free_tier: bool) -> f64 {
    crate::services::auto_tagging::default_delay_for(free_tier)
}

async fn select_hoop_dimension_candidates(
    pool: &SqlitePool,
    limit: i64,
    min_id: i64,
) -> Result<Vec<i64>, AppError> {
    let rows = sqlx::query(
        "SELECT id
\t\t FROM designs
\t\t WHERE (width_mm IS NULL OR height_mm IS NULL OR hoop_id IS NULL) AND id > ?
\t\t ORDER BY id ASC
\t\t LIMIT ?",
    )
    .bind(min_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to select hoop dimension candidates: {e}")))?;

    let mut ids = Vec::new();
    for row in rows {
        ids.push(row.try_get::<i64, _>("id").map_err(|e| {
            AppError::database(format!("failed to read hoop dimension candidate id: {e}"))
        })?);
    }
    Ok(ids)
}

async fn update_hoop_dimensions_only(pool: &SqlitePool, design_id: i64) -> Result<(), AppError> {
    let row = sqlx::query("SELECT filepath FROM designs WHERE id = ?")
        .bind(design_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            AppError::database(format!("failed to read filepath for hoop dimensions: {e}"))
        })?;

    let Some(row) = row else {
        return Ok(());
    };

    let filepath: String = row
        .try_get("filepath")
        .map_err(|e| AppError::database(format!("failed to read filepath: {e}")))?;
    let resolved_path = resolve_stored_design_path(&filepath);
    let parsed = design_metadata::parse_design_file(&resolved_path)
        .map_err(|e| AppError::invalid_input(e))?;

    let hoop_id =
        design_metadata::recommend_hoop_for_design(pool, parsed.width_mm, parsed.height_mm)
            .await
            .map_err(|e| AppError::database(e))?;

    sqlx::query(
        "UPDATE designs
\t\t SET width_mm = ?,
\t\t     height_mm = ?,
\t\t     hoop_id = ?
\t\t WHERE id = ?",
    )
    .bind(parsed.width_mm)
    .bind(parsed.height_mm)
    .bind(hoop_id)
    .bind(design_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::database(format!("failed to update hoop dimensions: {e}")))?;

    Ok(())
}

async fn get_f64_setting(pool: &SqlitePool, key: &str) -> Result<Option<f64>, AppError> {
    let value = sqlx::query("SELECT value FROM settings WHERE key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database(format!("failed to read float setting {key}: {e}")))?
        .and_then(|row| row.try_get::<String, _>("value").ok());

    Ok(value.and_then(|raw| raw.trim().parse::<f64>().ok()))
}

fn resolve_i64_option(
    request: Option<i64>,
    setting: Option<i64>,
    default: i64,
    min: i64,
    max: i64,
) -> i64 {
    request.or(setting).unwrap_or(default).clamp(min, max)
}

fn resolve_f64_option(
    request: Option<f64>,
    setting: Option<f64>,
    default: f64,
    min: f64,
    max: f64,
) -> f64 {
    request.or(setting).unwrap_or(default).clamp(min, max)
}

fn log_dir_path() -> PathBuf {
    static BASE_DIR: OnceLock<PathBuf> = OnceLock::new();

    let base_dir = BASE_DIR.get_or_init(|| {
        crate::paths::resolve_app_paths()
            .map(|paths| paths.log_dir)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    });

    base_dir.clone()
}

fn info_log_path() -> PathBuf {
    log_dir_path().join(INFO_LOG_FILE)
}

fn error_log_path() -> PathBuf {
    log_dir_path().join(ERROR_LOG_FILE)
}

fn truncate_logs_for_new_run() -> Result<(), AppError> {
    let dir = log_dir_path();
    if let Err(err) = fs::create_dir_all(&dir) {
        return Err(AppError::io(format!(
            "failed to create log dir {}: {err}",
            dir.display()
        )));
    }

    let info_path = info_log_path();
    let error_path = error_log_path();

    if let Err(err) = fs::write(&info_path, "") {
        return Err(AppError::io(format!(
            "failed to truncate info log {}: {err}",
            info_path.display()
        )));
    }
    if let Err(err) = fs::write(&error_path, "") {
        return Err(AppError::io(format!(
            "failed to truncate error log {}: {err}",
            error_path.display()
        )));
    }

    Ok(())
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn append_log_line(path: &Path, line: &str) {
    if let Err(err) = fs::create_dir_all(log_dir_path()) {
        eprintln!("failed to create log dir: {err}");
        return;
    }

    let mut file = match fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("failed to open log file {}: {err}", path.display());
            return;
        }
    };

    if let Err(err) = writeln!(file, "{line}") {
        eprintln!("failed to append log line to {}: {err}", path.display());
    }
}

pub fn log_info(message: String) {
    append_log_line(
        &info_log_path(),
        &format!("{}\t{}", now_epoch_seconds(), message),
    );
}

pub fn log_error(message: String) {
    append_log_line(
        &error_log_path(),
        &format!("{}\t{}", now_epoch_seconds(), message),
    );
}

fn read_log_tail(
    path: &Path,
    level: &str,
    limit: usize,
) -> Result<Vec<BackfillLogEntry>, AppError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)
        .map_err(|e| AppError::io(format!("failed to read log file {}: {e}", path.display())))?;
    let mut lines = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| BackfillLogEntry {
            level: level.to_string(),
            message: line.to_string(),
        })
        .collect::<Vec<_>>();

    if lines.len() > limit {
        lines = lines[lines.len() - limit..].to_vec();
    }

    Ok(lines)
}
#[cfg(test)]
#[path = "backfill_tests.rs"]
mod tests;
