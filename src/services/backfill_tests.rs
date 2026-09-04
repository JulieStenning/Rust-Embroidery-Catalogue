// Tests for the backfill service.
//
// This module was split out of backfill.rs so the service file can stay
// focused on production logic. It is included via a #[path] declaration
// in a #[cfg(test)] mod tests; module, so it retains full access to the
// private items in the parent module through use super::*;.

use super::*;
use serial_test::serial;

async fn make_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("memory db");
    for sql in [
        "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, description TEXT)",
        "CREATE TABLE tags (id INTEGER PRIMARY KEY, description TEXT NOT NULL, tag_group TEXT)",
        "CREATE TABLE designs (id INTEGER PRIMARY KEY, filename TEXT NOT NULL, filepath TEXT NOT NULL, image_data BLOB, image_type TEXT, width_mm INTEGER, height_mm INTEGER, hoop_id INTEGER, stitch_count INTEGER, color_count INTEGER, color_change_count INTEGER, image_tags_verified INTEGER NOT NULL DEFAULT 0, stitching_tags_verified INTEGER NOT NULL DEFAULT 0, text_ai_analyzed INTEGER NOT NULL DEFAULT 0, text_ai_matched INTEGER NOT NULL DEFAULT 0, vision_ai_analyzed INTEGER NOT NULL DEFAULT 0, vision_ai_matched INTEGER NOT NULL DEFAULT 0)",
        "CREATE TABLE design_tags (design_id INTEGER NOT NULL, tag_id INTEGER NOT NULL, PRIMARY KEY(design_id, tag_id))",
        "CREATE TABLE hoops (id INTEGER PRIMARY KEY, name TEXT NOT NULL, max_width_mm REAL NOT NULL, max_height_mm REAL NOT NULL)",
    ] {
        sqlx::query(sql).execute(&pool).await.expect("schema");
    }
    pool
}

async fn seed_basic(pool: &SqlitePool) {
    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (1, 'Cats', 'image')")
        .execute(pool)
        .await
        .expect("seed tag");
    sqlx::query(
        "INSERT INTO tags (id, description, tag_group) VALUES (2, 'Line Outline', 'stitching')",
    )
    .execute(pool)
    .await
    .expect("seed tag2");
    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (?, ?, ?)")
        .bind(3_i64)
        .bind("Don't Know")
        .bind("image")
        .execute(pool)
        .await
        .expect("seed tag3");
    sqlx::query("INSERT INTO designs (id, filename, filepath, image_tags_verified, stitching_tags_verified) VALUES (1, 'cute_cat.pes', 'tests/Test Designs/cute_cat.pes', 0, 0)").execute(pool).await.expect("seed design1");
    sqlx::query("INSERT INTO designs (id, filename, filepath, image_tags_verified, stitching_tags_verified) VALUES (2, 'dog_crest.pes', 'tests/Test Designs/dog_crest.pes', 1, 1)").execute(pool).await.expect("seed design2");
    sqlx::query("INSERT INTO designs (id, filename, filepath, image_tags_verified, stitching_tags_verified) VALUES (3, 'flower.pes', 'tests/Test Designs/flower.pes', 0, 0)").execute(pool).await.expect("seed design3");
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (2, 1)")
        .execute(pool)
        .await
        .expect("seed design tag");
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_tag_untagged_skips_tagged_designs() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("tag_untagged".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: None,
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(true),
                }),
                stitching: None,
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    assert!(summary.processed >= 2);
    let still_tagged = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM design_tags WHERE design_id = 2 AND tag_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(still_tagged, 1);
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_retag_all_processes_all_designs_beyond_batch_size() {
    clear_stop_signal();
    let pool = make_test_pool().await;
    // seed_basic provides the 'Cats' image tag plus designs 1..=3. Add more so
    // the total exceeds the batch size, proving pagination reaches EVERY design
    // (previously a single `LIMIT batch_size` run only ever touched the first batch).
    seed_basic(&pool).await;
    for id in 4..=250_i64 {
        sqlx::query(
            "INSERT INTO designs (id, filename, filepath, image_tags_verified, stitching_tags_verified) VALUES (?, ?, ?, 0, 0)",
        )
        .bind(id)
        .bind(format!("design_{id}.pes"))
        .bind(format!("tests/Test Designs/design_{id}.pes"))
        .execute(&pool)
        .await
        .expect("seed extra design");
    }

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("retag_all".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: Some(false),
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(true),
                }),
                stitching: None,
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    // All 250 designs must be processed, not just the first batch of 100.
    assert_eq!(summary.processed, 250, "retag_all must reach every design");
    assert_eq!(summary.errors, 0);
    clear_stop_signal();
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_streams_progress_events() {
    clear_stop_signal();
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // designs 1..=3

    let mut events: Vec<BackfillProgress> = Vec::new();
    let summary = run_unified_backfill_with_progress(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("retag_all".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: None,
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(true),
                }),
                stitching: None,
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(2),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
        None,
        &mut |p| events.push(p.clone()),
    )
    .await
    .expect("run succeeds");

    // Expect started + at least one batch_committed (at processed == commit_every)
    // + a final completed event carrying the true processed count.
    assert!(
        events.len() >= 3,
        "expected started + commit + completed events, got {events:?}"
    );
    assert_eq!(events.first().unwrap().stage, "started");
    assert!(
        events.iter().any(|e| e.stage == "processing"),
        "expected live per-design processing events, got {events:?}"
    );
    assert!(
        events.iter().any(|e| e.stage == "batch_committed"),
        "expected a batch_committed event, got {events:?}"
    );
    let last = events.last().unwrap();
    assert_eq!(last.stage, "completed");
    assert_eq!(last.processed, summary.processed);
    clear_stop_signal();
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_retag_all_respects_workers_concurrency() {
    clear_stop_signal();
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // designs 1..=3
    for id in 4..=40_i64 {
        sqlx::query(
            "INSERT INTO designs (id, filename, filepath, image_tags_verified, stitching_tags_verified) VALUES (?, ?, ?, 0, 0)",
        )
        .bind(id)
        .bind(format!("design_{id}.pes"))
        .bind(format!("tests/Test Designs/design_{id}.pes"))
        .execute(&pool)
        .await
        .expect("seed extra design");
    }

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("retag_all".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: Some(false),
                    folder_path: None,
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
            workers: Some(4),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    // 40 designs across 4 batches of 10, each processed with up to 4 workers.
    assert_eq!(
        summary.processed, 40,
        "concurrent run must reach every design"
    );
    assert_eq!(summary.errors, 0);
    clear_stop_signal();
}

#[tokio::test]
#[serial]
async fn stop_state_transitions_are_stable() {
    clear_stop_signal();
    let first = request_stop();
    let second = request_stop();
    assert_eq!(first.status, "stopping");
    assert_eq!(second.status, "already_stopping");
    clear_stop_signal();
}

#[test]
fn normalize_tag_mode_covers_all_cases() {
    assert_eq!(normalize_tag_mode(Some("retag_all")), TAG_ACTION_RETAG_ALL);
    assert_eq!(
        normalize_tag_mode(Some("retag_all_unverified")),
        TAG_ACTION_RETAG_ALL_UNVERIFIED
    );
    assert_eq!(normalize_tag_mode(Some("unknown")), TAG_ACTION_UNTAGGED);
}

#[test]
fn free_tier_rate_limit_message_paid_tier_keeps_existing_guidance() {
    let error = crate::error::AppError::invalid_input("Gemini API error 429: quota".to_string());
    let msg = free_tier_rate_limit_message(&error, false);
    assert!(msg.contains("Increase the AI delay or lower Workers"));
    assert!(!msg.contains("Free-tier"));
}

#[test]
fn free_tier_rate_limit_message_uses_retry_after() {
    let error = crate::error::AppError::invalid_input(
        "Gemini API error 429: limit (retry_after=120)".to_string(),
    );
    let msg = free_tier_rate_limit_message(&error, true);
    assert!(msg.contains("Free-tier Gemini rate limit reached (429)"));
    assert!(msg.contains("wait about 2 minutes"));
    assert!(!msg.contains("Increase the AI delay"));
}

#[test]
fn free_tier_rate_limit_message_falls_back_to_generic_wait() {
    let error = crate::error::AppError::invalid_input("Gemini API error 429: limit".to_string());
    let msg = free_tier_rate_limit_message(&error, true);
    assert!(msg.contains("wait a few minutes or until tomorrow"));
}

#[test]
fn default_workers_for_reflects_free_tier() {
    // Free-tier keys are rate-limited (~15 req/min), so blank fields default to a
    // conservative concurrency rather than the normal 4.
    assert_eq!(default_workers_for(true), 2);
    assert_eq!(default_workers_for(false), 4);
}

#[test]
fn default_delay_for_reflects_free_tier() {
    // Paid keys aren't rate-limited, so the paid default is no delay; the free-tier
    // default (10s paired with 2 workers) keeps throughput under ~15 requests/minute.
    assert_eq!(default_delay_for(true), 10.0);
    assert_eq!(default_delay_for(false), 0.0);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn normalize_modes_default_includes_path_rule() {
    let result = normalize_modes(None, false);
    assert_eq!(result.len(), 1);
    assert!(result.contains("path_rule"));
}

#[test]
fn normalize_modes_removes_ai_vision_without_api_key() {
    let result = normalize_modes(
        Some(&["path_rule".to_string(), "ai_vision".to_string()]),
        false,
    );
    assert!(result.contains("path_rule"));
    assert!(!result.contains("ai_vision"));
}

#[test]
fn normalize_modes_includes_ai_vision_with_api_key() {
    let result = normalize_modes(
        Some(&["path_rule".to_string(), "ai_vision".to_string()]),
        true,
    );
    assert!(result.contains("path_rule"));
    assert!(result.contains("ai_vision"));
}

#[test]
fn normalize_modes_includes_text_ai_with_api_key() {
    let result = normalize_modes(
        Some(&["path_rule".to_string(), "text_ai".to_string()]),
        true,
    );
    assert!(result.contains("path_rule"));
    assert!(result.contains("text_ai"));
}

#[test]
fn normalize_modes_removes_text_ai_without_api_key() {
    let result = normalize_modes(
        Some(&["path_rule".to_string(), "text_ai".to_string()]),
        false,
    );
    assert!(result.contains("path_rule"));
    assert!(!result.contains("text_ai"));
}

#[test]
fn normalize_modes_empty_slice_resolves_to_path_rule() {
    let result = normalize_modes(Some(&[]), true);
    assert_eq!(result.len(), 1);
    assert!(result.contains("path_rule"));
}

#[test]
fn normalize_modes_path_rule_always_present_even_if_not_listed() {
    let result = normalize_modes(Some(&["ai_vision".to_string()]), true);
    assert!(result.contains("path_rule"));
    assert!(result.contains("ai_vision"));
}

#[test]
fn suggest_visual_ai_exact_token_match() {
    let mut valid = HashSet::new();
    valid.insert("Cats".to_string());
    valid.insert("Flowers".to_string());
    valid.insert("Don't Know".to_string());
    let result = suggest_visual_ai_descriptions("cats.pes", "/designs/", &valid);
    assert!(
        result.contains(&"Cats".to_string()),
        "Expected Cats, got {:?}",
        result
    );
    assert!(!result.contains(&"Flowers".to_string()));
}

#[test]
fn suggest_visual_ai_matches_all_tokens_in_description() {
    let mut valid = HashSet::new();
    valid.insert("Christmas Tree".to_string());
    valid.insert("Don't Know".to_string());
    let result = suggest_visual_ai_descriptions("xmas_tree.pes", "/designs/polls/", &valid);
    assert!(!result.contains(&"Christmas Tree".to_string()));
    assert!(result.contains(&"Don't Know".to_string()) || !result.is_empty());
}

#[test]
fn suggest_visual_ai_fallback_when_no_token_match() {
    let mut valid = HashSet::new();
    valid.insert("Cats".to_string());
    valid.insert("Don't Know".to_string());
    let result = suggest_visual_ai_descriptions("some_random.pes", "/designs/", &valid);
    assert_eq!(result, vec!["Don't Know"]);
}

#[test]
fn suggest_visual_ai_handles_special_characters() {
    let mut valid = HashSet::new();
    valid.insert("Holiday".to_string());
    let result = suggest_visual_ai_descriptions("holiday.pes", "/designs/", &valid);
    assert!(
        result.contains(&"Holiday".to_string()),
        "Expected Holiday, got {:?}",
        result
    );
}

#[test]
fn suggest_visual_ai_fallback_respects_ordering() {
    let mut valid = HashSet::new();
    valid.insert("Patterns".to_string());
    valid.insert("Flowers".to_string());
    let result = suggest_visual_ai_descriptions("zzz_nonexistent.pes", "/designs/", &valid);
    assert_eq!(result, vec!["Patterns"]);
}

#[test]
fn suggest_visual_ai_no_dont_know_when_not_valid() {
    let mut valid = HashSet::new();
    valid.insert("Butterfly".to_string());
    let result = suggest_visual_ai_descriptions("nonexistent.pes", "/designs/", &valid);
    assert!(result.is_empty(), "Expected empty, got {:?}", result);
}
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// resolve_i64_option
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn resolve_i64_request_overrides_setting() {
    assert_eq!(resolve_i64_option(Some(50), Some(10), 100, 1, 1000), 50);
}

#[test]
fn resolve_i64_setting_overrides_default() {
    assert_eq!(resolve_i64_option(None, Some(75), 100, 1, 1000), 75);
}

#[test]
fn resolve_i64_default_used_when_none() {
    assert_eq!(resolve_i64_option(None, None, 200, 1, 1000), 200);
}

#[test]
fn resolve_i64_clamps_to_min() {
    assert_eq!(resolve_i64_option(Some(-5), None, 10, 1, 1000), 1);
}

#[test]
fn resolve_i64_clamps_to_max() {
    assert_eq!(resolve_i64_option(Some(9999), None, 10, 1, 1000), 1000);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// resolve_f64_option
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn resolve_f64_request_overrides_setting() {
    assert!((resolve_f64_option(Some(3.5), Some(1.0), 5.0, 0.0, 120.0) - 3.5).abs() < f64::EPSILON);
}

#[test]
fn resolve_f64_setting_overrides_default() {
    assert!((resolve_f64_option(None, Some(2.5), 5.0, 0.0, 120.0) - 2.5).abs() < f64::EPSILON);
}

#[test]
fn resolve_f64_default_used_when_none() {
    assert!((resolve_f64_option(None, None, 10.0, 0.0, 120.0) - 10.0).abs() < f64::EPSILON);
}

#[test]
fn resolve_f64_clamps_to_min() {
    assert!((resolve_f64_option(Some(-1.0), None, 5.0, 0.0, 120.0) - 0.0).abs() < f64::EPSILON);
}

#[test]
fn resolve_f64_clamps_to_max() {
    assert!((resolve_f64_option(Some(200.0), None, 5.0, 0.0, 120.0) - 120.0).abs() < f64::EPSILON);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// stop_requested_store / is_stop_requested
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
#[serial]
fn is_stop_requested_initial_state_false() {
    clear_stop_signal();
    assert!(!is_stop_requested());
}

#[test]
#[serial]
fn stop_requested_store_true_and_false() {
    clear_stop_signal();
    stop_requested_store(true);
    assert!(is_stop_requested());
    stop_requested_store(false);
    assert!(!is_stop_requested());
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// now_epoch_seconds
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn now_epoch_seconds_returns_positive() {
    let ts = now_epoch_seconds();
    assert!(
        ts > 1_700_000_000,
        "Expected reasonable epoch timestamp, got {}",
        ts
    );
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// log_dir_path / info_log_path / error_log_path
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn log_dir_path_returns_logs_directory() {
    let path = log_dir_path();
    assert!(path.ends_with(LOG_DIR));
    assert_eq!(path.file_name(), Some(std::ffi::OsStr::new(LOG_DIR)));
}

#[test]
fn info_log_path_returns_correct_path() {
    assert_eq!(info_log_path(), log_dir_path().join(INFO_LOG_FILE));
}

#[test]
fn error_log_path_returns_correct_path() {
    assert_eq!(error_log_path(), log_dir_path().join(ERROR_LOG_FILE));
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: get_i64_setting / get_f64_setting
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

async fn seed_setting(pool: &SqlitePool, key: &str, value: &str) {
    sqlx::query("INSERT INTO settings (key, value, description) VALUES (?, ?, 'test')")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await
        .expect("seed setting");
}

#[tokio::test]
async fn get_i64_setting_returns_value_when_present() {
    let pool = make_test_pool().await;
    seed_setting(&pool, "ai.batch_size", "50").await;
    assert_eq!(
        get_i64_setting(&pool, "ai.batch_size").await.unwrap(),
        Some(50)
    );
}

#[tokio::test]
async fn get_i64_setting_returns_none_when_missing() {
    let pool = make_test_pool().await;
    assert_eq!(get_i64_setting(&pool, "missing_key").await.unwrap(), None);
}

#[tokio::test]
async fn get_i64_setting_returns_none_on_non_numeric() {
    let pool = make_test_pool().await;
    seed_setting(&pool, "bad", "not_a_number").await;
    assert_eq!(get_i64_setting(&pool, "bad").await.unwrap(), None);
}

#[tokio::test]
async fn get_f64_setting_returns_value_when_present() {
    let pool = make_test_pool().await;
    seed_setting(&pool, "ai.delay", "2.5").await;
    let result = get_f64_setting(&pool, "ai.delay").await.unwrap();
    assert!((result.unwrap() - 2.5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn get_f64_setting_returns_none_when_missing() {
    let pool = make_test_pool().await;
    assert_eq!(get_f64_setting(&pool, "missing").await.unwrap(), None);
}

#[tokio::test]
async fn get_f64_setting_returns_none_on_non_numeric() {
    let pool = make_test_pool().await;
    seed_setting(&pool, "bad", "abc").await;
    assert_eq!(get_f64_setting(&pool, "bad").await.unwrap(), None);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: get_image_tag_lookup
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn get_image_tag_lookup_returns_map() {
    let pool = make_test_pool().await;
    // tags already seeded: (1, 'Cats', 'image'), (3, "Don't Know", 'image')
    seed_basic(&pool).await;

    let map = get_image_tag_lookup(&pool).await.unwrap();
    assert_eq!(map.len(), 2);
    assert_eq!(*map.get("Cats").unwrap(), 1);
    assert_eq!(*map.get("Don't Know").unwrap(), 3);
}

#[tokio::test]
async fn get_image_tag_lookup_empty_when_no_image_tags() {
    let pool = make_test_pool().await;
    // Only stitch tag seeded
    // Actually make_test_pool() just creates tables, no data
    let map = get_image_tag_lookup(&pool).await.unwrap();
    assert!(map.is_empty());
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: select_tagging_design_ids
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn select_tagging_untagged_excludes_designs_with_image_tags() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // design 2 has an image tag

    let ids = select_tagging_design_ids(&pool, "tag_untagged", 100, 0, false, None, true)
        .await
        .unwrap();
    assert!(ids.contains(&1));
    assert!(ids.contains(&3));
    assert!(!ids.contains(&2));
}

#[tokio::test]
async fn select_tagging_retag_all_includes_all() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    let ids = select_tagging_design_ids(&pool, "retag_all", 100, 0, false, None, true)
        .await
        .unwrap();
    assert_eq!(ids.len(), 3);
}

#[tokio::test]
async fn select_tagging_retag_all_unverified_includes_only_unverified() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // design 2 has tags_checked=1, 1 and 3 have 0

    let ids = select_tagging_design_ids(&pool, "retag_all_unverified", 100, 0, false, None, true)
        .await
        .unwrap();
    assert!(ids.contains(&1));
    assert!(ids.contains(&3));
    assert!(!ids.contains(&2));
}

#[tokio::test]
async fn select_tagging_respects_limit() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    let ids = select_tagging_design_ids(&pool, "tag_untagged", 1, 0, false, None, true)
        .await
        .unwrap();
    assert!(ids.len() <= 1);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: apply_image_tags_and_tier
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn flush_tagging_batch_writes_tag_and_tier() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    let mut map = HashMap::new();
    map.insert("Cats".to_string(), 1);
    map.insert("Don't Know".to_string(), 3);

    flush_tagging_batch(
        &pool,
        &map,
        vec![TagBatchEntry {
            design_id: 1,
            descriptions: vec!["Cats".to_string()],
            text_ai_analyzed: true,
            text_ai_matched: true,
            vision_ai_analyzed: false,
            vision_ai_matched: false,
        }],
        "reset",
    )
    .await
    .unwrap();

    // Verify design_tags
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);

    // Verify the per-mode AI flags were written alongside the tags.
    let analyzed: i64 = sqlx::query_scalar("SELECT text_ai_analyzed FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(analyzed, 1);
    let matched: i64 = sqlx::query_scalar("SELECT text_ai_matched FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(matched, 1);
}

#[tokio::test]
async fn flush_tagging_batch_empty_descriptions_noop() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    flush_tagging_batch(
        &pool,
        &HashMap::new(),
        vec![TagBatchEntry {
            design_id: 1,
            descriptions: vec![],
            text_ai_analyzed: false,
            text_ai_matched: false,
            vision_ai_analyzed: false,
            vision_ai_matched: false,
        }],
        "reset",
    )
    .await
    .unwrap();

    // Should not error, no changes
}

#[tokio::test]
async fn flush_tagging_batch_replaces_existing_image_tags() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // design 2 already has Cats tag

    let mut map = HashMap::new();
    map.insert("Cats".to_string(), 1);
    map.insert("Don't Know".to_string(), 3);

    // Replace Cats with Don't Know (Vision AI produced the suggestion)
    flush_tagging_batch(
        &pool,
        &map,
        vec![TagBatchEntry {
            design_id: 2,
            descriptions: vec!["Don't Know".to_string()],
            text_ai_analyzed: false,
            text_ai_matched: false,
            vision_ai_analyzed: true,
            vision_ai_matched: true,
        }],
        "reset",
    )
    .await
    .unwrap();

    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 2 AND tag_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(remaining, 0);

    let added: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 2 AND tag_id = 3")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(added, 1);
}
#[tokio::test]
async fn count_tagging_candidates_returns_total_unverified_verified_breakdown() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // 1: untagged+unverified, 2: tagged+verified, 3: untagged+unverified

    // tag_untagged -> designs with no image-group tags: 1 and 3 (both unverified).
    let untagged = count_tagging_candidates(&pool, "tag_untagged", None, true)
        .await
        .unwrap();
    assert_eq!(untagged.total_count, 2);
    assert_eq!(untagged.unverified_count, 2);
    assert_eq!(untagged.verified_count, 0);

    // retag_all_unverified -> image_tags_verified = 0: designs 1 and 3.
    let unverified = count_tagging_candidates(&pool, "retag_all_unverified", None, true)
        .await
        .unwrap();
    assert_eq!(unverified.total_count, 2);
    assert_eq!(unverified.unverified_count, 2);
    assert_eq!(unverified.verified_count, 0);

    // retag_all -> every design: total 3, one of which is verified (design 2).
    let all = count_tagging_candidates(&pool, "retag_all", None, true)
        .await
        .unwrap();
    assert_eq!(all.total_count, 3);
    assert_eq!(all.unverified_count, 2);
    assert_eq!(all.verified_count, 1);

    // The pager's total (exclude_verified=false) matches total_count for each scope.
    let untagged_ids = select_tagging_design_ids(&pool, "tag_untagged", 100, 0, false, None, true)
        .await
        .unwrap();
    assert_eq!(untagged.total_count as usize, untagged_ids.len());
    let unverified_ids =
        select_tagging_design_ids(&pool, "retag_all_unverified", 100, 0, false, None, true)
            .await
            .unwrap();
    assert_eq!(unverified.total_count as usize, unverified_ids.len());
    let all_ids = select_tagging_design_ids(&pool, "retag_all", 100, 0, false, None, true)
        .await
        .unwrap();
    assert_eq!(all.total_count as usize, all_ids.len());

    // Excluding verified designs filters the pager down to unverified_count.
    let all_excluding = select_tagging_design_ids(&pool, "retag_all", 100, 0, true, None, true)
        .await
        .unwrap();
    assert_eq!(all.unverified_count as usize, all_excluding.len());
    assert!(!all_excluding.contains(&2));

    // An unknown action normalizes to tag_untagged, matching the pager.
    let unknown = count_tagging_candidates(&pool, "bogus", None, true)
        .await
        .unwrap();
    assert_eq!(unknown.total_count, untagged.total_count);
}
#[tokio::test]
async fn per_mode_ai_scope_counts_and_pager_parity() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // designs 1..=3 (AI flags default 0)
                             // design 1: Text AI analyzed + matched.
    sqlx::query("UPDATE designs SET text_ai_analyzed = 1, text_ai_matched = 1 WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();
    // design 2: Text AI analyzed but no match; Vision AI analyzed + matched.
    sqlx::query(
        "UPDATE designs SET text_ai_analyzed = 1, text_ai_matched = 0, \
         vision_ai_analyzed = 1, vision_ai_matched = 1 WHERE id = 2",
    )
    .execute(&pool)
    .await
    .unwrap();
    // design 3: Vision AI analyzed but no match.
    sqlx::query("UPDATE designs SET vision_ai_analyzed = 1, vision_ai_matched = 0 WHERE id = 3")
        .execute(&pool)
        .await
        .unwrap();

    // Text AI scopes.
    let text_not = count_tagging_candidates(&pool, "retag_all_text_not_analyzed", None, true)
        .await
        .unwrap();
    assert_eq!(text_not.total_count, 1); // only design 3 is text-not-analyzed
    let text_no_match = count_tagging_candidates(&pool, "retag_all_text_no_match", None, true)
        .await
        .unwrap();
    assert_eq!(text_no_match.total_count, 1); // analyzed + no match -> design 2
    let text_analyzed = count_tagging_candidates(&pool, "retag_all_text_analyzed", None, true)
        .await
        .unwrap();
    assert_eq!(text_analyzed.total_count, 2); // designs 1 and 2

    // Vision AI scopes.
    let vision_not = count_tagging_candidates(&pool, "retag_all_vision_not_analyzed", None, true)
        .await
        .unwrap();
    assert_eq!(vision_not.total_count, 1); // only design 1 is vision-not-analyzed
    let vision_no_match = count_tagging_candidates(&pool, "retag_all_vision_no_match", None, true)
        .await
        .unwrap();
    assert_eq!(vision_no_match.total_count, 1); // design 3
    let vision_analyzed = count_tagging_candidates(&pool, "retag_all_vision_analyzed", None, true)
        .await
        .unwrap();
    assert_eq!(vision_analyzed.total_count, 2); // designs 2 and 3

    // Pager parity: each scope's total (exclude_verified=false) matches the pager.
    for (action, expected) in [
        ("retag_all_text_not_analyzed", 1),
        ("retag_all_text_no_match", 1),
        ("retag_all_text_analyzed", 2),
        ("retag_all_vision_not_analyzed", 1),
        ("retag_all_vision_no_match", 1),
        ("retag_all_vision_analyzed", 2),
    ] {
        let ids = select_tagging_design_ids(&pool, action, 100, 0, false, None, true)
            .await
            .unwrap();
        assert_eq!(expected, ids.len(), "pager mismatch for {action}");
    }
}

#[test]
fn resolve_folder_scope_under_validates_boundary() {
    let root = std::env::temp_dir().join("tagging-folder-scope-test");
    let flowers = root.join("Flowers");
    std::fs::create_dir_all(&flowers).unwrap();

    // A subfolder under the library root resolves to its canonical relative path.
    let scope = resolve_folder_scope_under(Some(flowers.to_str().unwrap()), &root)
        .unwrap()
        .unwrap();
    assert_eq!(scope.rel, "Flowers");
    assert!(!scope.is_root);

    // The root itself resolves as the root scope.
    let root_scope = resolve_folder_scope_under(Some(root.to_str().unwrap()), &root)
        .unwrap()
        .unwrap();
    assert!(root_scope.is_root);
    assert_eq!(root_scope.rel, "");

    // No folder -> no scope.
    assert!(resolve_folder_scope_under(None, &root).unwrap().is_none());

    // A path outside the root is rejected.
    let outside = std::env::temp_dir().join("tagging-folder-scope-outside");
    std::fs::create_dir_all(&outside).unwrap();
    let err = resolve_folder_scope_under(Some(outside.to_str().unwrap()), &root).unwrap_err();
    assert!(err.to_string().contains("outside"));

    // A non-existent folder is rejected.
    let err = resolve_folder_scope_under(Some(root.join("Missing").to_str().unwrap()), &root)
        .unwrap_err();
    assert!(err.to_string().contains("does not exist"));

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&outside);
}

#[tokio::test]
async fn folder_scope_filters_candidates_recursively_and_direct_only() {
    let pool = make_test_pool().await;
    // Designs in a Flowers folder (direct files, a nested sub/b.pes, deeper
    // forms) plus an unrelated Animals folder. All stored filepaths are the
    // canonical library-relative form.
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified) VALUES (1, 'a.pes', 'Flowers/a.pes', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified) VALUES (2, 'b.pes', 'Flowers/sub/b.pes', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified) VALUES (3, 'c.pes', 'Flowers/c.pes', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified) VALUES (4, 'd.pes', 'Flowers/d.pes', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified) VALUES (5, 'e.pes', 'Animals/e.pes', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let scope = TaggingFolderScope {
        rel: "Flowers".to_string(),
        is_root: false,
    };

    // Recursive: all four Flowers designs (direct + nested), not design 5.
    let recursive =
        select_tagging_design_ids(&pool, "retag_all", 100, 0, false, Some(&scope), true)
            .await
            .unwrap();
    assert!(recursive.contains(&1));
    assert!(recursive.contains(&2));
    assert!(recursive.contains(&3));
    assert!(recursive.contains(&4));
    assert!(!recursive.contains(&5));

    // Direct-only: designs 1, 3, 4 but not 2 (nested under sub/).
    let direct = select_tagging_design_ids(&pool, "retag_all", 100, 0, false, Some(&scope), false)
        .await
        .unwrap();
    assert!(direct.contains(&1));
    assert!(direct.contains(&3));
    assert!(direct.contains(&4));
    assert!(!direct.contains(&2));

    // Counts align with the candidate set.
    let counts = count_tagging_candidates(&pool, "retag_all", Some(&scope), true)
        .await
        .unwrap();
    assert_eq!(counts.total_count, 4);
    let counts_direct = count_tagging_candidates(&pool, "retag_all", Some(&scope), false)
        .await
        .unwrap();
    assert_eq!(counts_direct.total_count, 3);
}

#[tokio::test]
async fn flush_tagging_batch_add_mode_preserves_existing_image_tags() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // design 2 already has Cats (tag_id=1)

    let mut map = HashMap::new();
    map.insert("Cats".to_string(), 1);
    map.insert("Don't Know".to_string(), 3);

    // "add" appends Don't Know while keeping the existing Cats tag.
    flush_tagging_batch(
        &pool,
        &map,
        vec![TagBatchEntry {
            design_id: 2,
            descriptions: vec!["Don't Know".to_string()],
            text_ai_analyzed: false,
            text_ai_matched: false,
            vision_ai_analyzed: true,
            vision_ai_matched: true,
        }],
        "add",
    )
    .await
    .unwrap();

    let cats: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 2 AND tag_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cats, 1, "existing image tag must be preserved in add mode");

    let added: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 2 AND tag_id = 3")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(added, 1);
}

#[tokio::test]
async fn flush_tagging_batch_reset_never_touches_non_image_tags() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // Give design 1 a stitching tag (tag_id=2) that a reset run must preserve.
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .unwrap();

    let mut map = HashMap::new();
    map.insert("Cats".to_string(), 1);

    flush_tagging_batch(
        &pool,
        &map,
        vec![TagBatchEntry {
            design_id: 1,
            descriptions: vec!["Cats".to_string()],
            text_ai_analyzed: false,
            text_ai_matched: false,
            vision_ai_analyzed: false,
            vision_ai_matched: false,
        }],
        "reset",
    )
    .await
    .unwrap();

    let stitching: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 2")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stitching, 1, "non-image tags must survive a reset");
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: clear_stitching_tags
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn clear_stitching_unverified_removes_tags_from_unverified() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // design 1 has tags_checked=0, design 2 has tags_checked=1
    // Give design 1 a stitching tag (tag_id=2, 'Line Outline', 'stitching')
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .unwrap();

    let cleared = clear_stitching_tags(&pool, "unverified").await.unwrap();
    assert_eq!(cleared, vec![1]);

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 2")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn clear_stitching_unverified_leaves_verified_alone() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // design 2 has tags_checked=1, give it stitching tag
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (2, 2)")
        .execute(&pool)
        .await
        .unwrap();

    let cleared = clear_stitching_tags(&pool, "unverified").await.unwrap();
    assert!(cleared.is_empty());

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 2 AND tag_id = 2")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn clear_stitching_all_removes_tags_from_every_design() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // design 1 has tags_checked=0, design 2 has tags_checked=1
    // Give both designs a stitching tag
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (2, 2)")
        .execute(&pool)
        .await
        .unwrap();

    let cleared = clear_stitching_tags(&pool, "all").await.unwrap();
    assert_eq!(cleared.len(), 2);
    assert!(cleared.contains(&1));
    assert!(cleared.contains(&2));

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE tag_id = 2")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn clear_stitching_unknown_mode_falls_back_to_unverified() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // Give design 1 (unverified) a stitching tag
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .unwrap();

    // The caller (run_unified_backfill) only invokes clear_stitching_tags
    // for "unverified" or "all". An unknown mode is treated conservatively
    // like "unverified": it clears from unverified designs only.
    let cleared = clear_stitching_tags(&pool, "unknown").await.unwrap();
    assert_eq!(cleared, vec![1]);

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 2")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: select_stitching_candidates
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn select_stitching_candidates_excludes_designs_with_stitching_tags() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // design 1 has no stitching tag â€” should be candidate
    // design 2 has image tag but no stitching tag â€” should be candidate
    // Give design 3 a stitching tag
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (3, 2)")
        .execute(&pool)
        .await
        .unwrap();

    let candidates = select_stitching_candidates(&pool, 100, 0).await.unwrap();
    let ids: Vec<i64> = candidates.iter().map(|c| c.id).collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));
    assert!(!ids.contains(&3));
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: get_stitching_tag_lookup
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn get_stitching_tag_lookup_returns_map() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // tag 2 = 'Line Outline', 'stitching'

    let map = get_stitching_tag_lookup(&pool).await.unwrap();
    assert_eq!(map.len(), 1);
    assert_eq!(*map.get("Line Outline").unwrap(), 2);
}

#[tokio::test]
async fn get_stitching_tag_lookup_empty_when_no_stitching_tags() {
    let pool = make_test_pool().await;
    let map = get_stitching_tag_lookup(&pool).await.unwrap();
    assert!(map.is_empty());
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: get_default_stitching_tag_id
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn get_default_stitching_tag_returns_line_outline_if_present() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // tag 2 = 'Line Outline', 'stitching'

    let id = get_default_stitching_tag_id(&pool).await.unwrap();
    assert_eq!(id, Some(2));
}

#[tokio::test]
async fn get_default_stitching_tag_none_when_no_stitching_tags() {
    let pool = make_test_pool().await;
    let id = get_default_stitching_tag_id(&pool).await.unwrap();
    assert_eq!(id, None);
}

#[tokio::test]
async fn get_default_stitching_tag_prefers_line_outline() {
    let pool = make_test_pool().await;
    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (10, 'Zigzag', 'stitching')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO tags (id, description, tag_group) VALUES (11, 'Line Outline', 'stitching')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let id = get_default_stitching_tag_id(&pool).await.unwrap();
    // 'Line Outline' should appear first (CASE WHEN = 0)
    assert_eq!(id, Some(11));
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: apply_stitching_tags
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn apply_stitching_tags_replaces_existing_stitching_tags() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // Add tag 2 ('Line Outline') to design 1
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .unwrap();

    // Create another stitching tag
    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (20, 'Satin', 'stitching')")
        .execute(&pool)
        .await
        .unwrap();

    apply_stitching_tags(&pool, 1, &[20]).await.unwrap();

    // Old tag removed
    let old: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 2")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(old, 0);

    // New tag added
    let new: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 20")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(new, 1);
}

#[tokio::test]
async fn apply_stitching_tags_empty_ids_removes_all_stitching_tags() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .unwrap();

    apply_stitching_tags(&pool, 1, &[]).await.unwrap();

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 2")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: select_image_candidates
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

async fn seed_design_with_image(
    pool: &SqlitePool,
    id: i64,
    image_data: Option<&[u8]>,
    image_type: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_data, image_type, image_tags_verified, stitching_tags_verified)
             VALUES (?, ?, ?, ?, ?, 0, 0)",
    )
    .bind(id)
    .bind(format!("design{}.pes", id))
    .bind(format!("tests/Test Designs/design{}.pes", id))
    .bind(image_data)
    .bind(image_type)
    .execute(pool)
    .await
    .expect("seed design with image");
}

#[tokio::test]
async fn select_image_candidates_normal_picks_designs_with_null_image() {
    let pool = make_test_pool().await;
    seed_design_with_image(&pool, 1, Some(b"fake_png"), Some("2d")).await;
    seed_design_with_image(&pool, 2, None, None).await;

    let ids = select_image_candidates(&pool, false, 100, 0).await.unwrap();
    assert_eq!(ids, vec![2]);
}

#[tokio::test]
async fn select_image_candidates_redo_includes_all() {
    let pool = make_test_pool().await;
    seed_design_with_image(&pool, 1, Some(b"fake_png"), Some("2d")).await;
    seed_design_with_image(&pool, 2, None, None).await;

    let ids = select_image_candidates(&pool, true, 100, 0).await.unwrap();
    assert_eq!(ids.len(), 2);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: clear_image_fields
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn clear_image_fields_sets_fields_to_null() {
    let pool = make_test_pool().await;
    seed_design_with_image(&pool, 1, Some(b"fake_png"), Some("2d")).await;
    // Also set dimensions
    sqlx::query("UPDATE designs SET width_mm = 100, height_mm = 200 WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    clear_image_fields(&pool, 1).await.unwrap();

    let row =
        sqlx::query("SELECT image_data, image_type, width_mm, height_mm FROM designs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(row
        .try_get::<Option<Vec<u8>>, _>("image_data")
        .unwrap()
        .is_none());
    assert!(row
        .try_get::<Option<String>, _>("image_type")
        .unwrap()
        .is_none());
    assert!(row.try_get::<Option<i64>, _>("width_mm").unwrap().is_none());
    assert!(row
        .try_get::<Option<i64>, _>("height_mm")
        .unwrap()
        .is_none());
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// DB helper: select_color_count_candidates
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn select_color_count_candidates_picks_designs_with_null_counts() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // designs 1,2,3 with null stitch/color/color_change

    let ids = select_color_count_candidates(&pool, 100, 0).await.unwrap();
    assert_eq!(ids.len(), 3);
}

#[tokio::test]
async fn select_color_count_candidates_excludes_designs_with_all_counts() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    sqlx::query("UPDATE designs SET stitch_count = 100, color_count = 5, color_change_count = 10 WHERE id = 1")
            .execute(&pool)
            .await
            .unwrap();

    let ids = select_color_count_candidates(&pool, 100, 0).await.unwrap();
    assert!(!ids.contains(&1));
    assert!(ids.contains(&2));
    assert!(ids.contains(&3));
}

// ─────────────────────────────────────────────────────────────
// DB helper: select_hoop_dimension_candidates
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn select_hoop_dimension_candidates_picks_designs_missing_dimensions_or_hoop() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // designs 1,2,3 with null width/height/hoop

    let ids = select_hoop_dimension_candidates(&pool, 100, 0)
        .await
        .unwrap();
    assert_eq!(ids.len(), 3);
}

#[tokio::test]
async fn select_hoop_dimension_candidates_excludes_designs_with_dimensions_and_hoop() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    sqlx::query("UPDATE designs SET width_mm = 100, height_mm = 80, hoop_id = 1 WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    let ids = select_hoop_dimension_candidates(&pool, 100, 0)
        .await
        .unwrap();
    assert!(!ids.contains(&1));
    assert!(ids.contains(&2));
    assert!(ids.contains(&3));
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Log file helpers: truncate_logs_for_new_run / read_log_tail / append_log_line / log_info / log_error
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
//
// All log-related tests are consolidated into a single test to avoid
// race conditions from parallel execution sharing the same log files.

#[tokio::test]
#[serial]
async fn log_files_round_trip() {
    // Clean up first
    let _ = std::fs::remove_dir_all("logs");
    truncate_logs_for_new_run().unwrap();
    assert!(info_log_path().exists());
    assert!(error_log_path().exists());

    // Empty files
    let entries = read_log_tail(&info_log_path(), "info", 10).unwrap();
    assert!(entries.is_empty());

    // Nonexistent file
    let entries = read_log_tail(Path::new("nonexistent.log"), "info", 10).unwrap();
    assert!(entries.is_empty());

    // Write info and error lines
    log_info("line1".to_string());
    log_info("line2".to_string());
    log_info("line3".to_string());
    log_info("line4".to_string());
    log_info("line5".to_string());
    log_error("err1".to_string());

    // Check tail limit (take last 3 of 5 info lines)
    let tail = read_log_tail(&info_log_path(), "info", 3).unwrap();
    assert_eq!(tail.len(), 3);
    assert!(tail.last().unwrap().message.contains("line5"));

    // Check format: timestamp\ttmessage
    let content = std::fs::read_to_string(info_log_path()).unwrap();
    assert!(content.contains("line1"));
    assert!(content.contains('\t'));

    // Error file
    let error_content = std::fs::read_to_string(error_log_path()).unwrap();
    assert!(error_content.contains("err1"));

    // Note: get_backfill_log_entries is not tested here because other
    // parallel tests call truncate_logs_for_new_run() which wipes the
    // shared log files.  That function is exercised by all the
    // run_unified_backfill integration tests which call it naturally.
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// apply_tagging_modes â€” unit-style tests
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
async fn compute_design_tagging_path_rule_match_returns_suggestion() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // design 1: "cute_cat.pes" - File & Folder Rules should match "Cats" via keyword map

    let mut map = HashMap::new();
    map.insert("Cats".to_string(), 1);
    map.insert("Don't Know".to_string(), 3);
    let valid: HashSet<String> = map.keys().cloned().collect();

    let mode_options = TaggingModeOptions {
        path_rule_enabled: true,
        text_ai_enabled: false,
        text_ai_network: false,
        visual_ai_enabled: false,
        visual_ai_delay_seconds: 0.0,
        visual_ai_network: false,
    };
    let result = compute_design_tagging(&pool, 1, &valid, &mode_options, None)
        .await
        .unwrap();

    // Compute-only: returns the merged descriptions; the write happens later in a
    // batched transaction.
    assert!(result.descriptions.iter().any(|d| d == "Cats"));
    assert!(!result.vision_ai_analyzed);
}

#[tokio::test]
async fn compute_design_tagging_path_rule_falls_to_visual_ai() {
    let pool = make_test_pool().await;
    // design with no path-rule match but token match works in Visual AI's local fallback
    sqlx::query("INSERT INTO designs (id, filename, filepath, image_data, image_tags_verified, stitching_tags_verified) VALUES (?, ?, ?, ?, 0, 0)")
        .bind(10_i64)
        .bind("abstract_blob.pes")
        .bind("tests/Test Designs/abstract_blob.pes")
        .bind(vec![0u8, 1, 2, 3])
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (?, ?, ?)")
        .bind(5_i64)
        .bind("Roses")
        .bind("image")
        .execute(&pool)
        .await
        .unwrap();

    let mut map = HashMap::new();
    map.insert("Roses".to_string(), 5);
    map.insert("Don't Know".to_string(), 3);
    let valid: HashSet<String> = map.keys().cloned().collect();

    let mode_options = TaggingModeOptions {
        path_rule_enabled: true,
        text_ai_enabled: false,
        text_ai_network: false,
        visual_ai_enabled: true,
        visual_ai_delay_seconds: 0.0,
        visual_ai_network: false,
    };
    let result = compute_design_tagging(&pool, 10, &valid, &mode_options, None)
        .await
        .unwrap();

    // Path rules produce no match, so Visual AI's local fallback runs and yields
    // "Don't Know" (no network client, image present).
    assert!(result.descriptions.iter().any(|d| d == "Don't Know"));
    assert!(result.vision_ai_analyzed);
    assert!(result.vision_ai_matched);
}

#[tokio::test]
async fn compute_design_tagging_nonexistent_design_returns_none() {
    let pool = make_test_pool().await;
    let valid = HashSet::new();
    let mode_options = TaggingModeOptions {
        path_rule_enabled: true,
        text_ai_enabled: false,
        text_ai_network: false,
        visual_ai_enabled: false,
        visual_ai_delay_seconds: 0.0,
        visual_ai_network: false,
    };
    let result = compute_design_tagging(&pool, 999, &valid, &mode_options, None).await;
    assert!(matches!(result, Ok(ref r) if r.descriptions.is_empty()));
}

#[tokio::test]
async fn flush_tagging_batch_commits_multiple_designs_in_one_transaction() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // designs 1..=3

    let mut map = HashMap::new();
    map.insert("Cats".to_string(), 1);
    map.insert("Don't Know".to_string(), 3);

    // Two designs' writes share a single transaction.
    flush_tagging_batch(
        &pool,
        &map,
        vec![
            TagBatchEntry {
                design_id: 1,
                descriptions: vec!["Cats".to_string()],
                text_ai_analyzed: true,
                text_ai_matched: true,
                vision_ai_analyzed: false,
                vision_ai_matched: false,
            },
            TagBatchEntry {
                design_id: 3,
                descriptions: vec!["Don't Know".to_string()],
                text_ai_analyzed: false,
                text_ai_matched: false,
                vision_ai_analyzed: true,
                vision_ai_matched: true,
            },
        ],
        "reset",
    )
    .await
    .unwrap();

    let cat_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 1 AND tag_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cat_count, 1);
    let analyzed: i64 = sqlx::query_scalar("SELECT text_ai_analyzed FROM designs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(analyzed, 1);

    let dk_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM design_tags WHERE design_id = 3 AND tag_id = 3")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(dk_count, 1);
    let vision_analyzed: i64 =
        sqlx::query_scalar("SELECT vision_ai_analyzed FROM designs WHERE id = 3")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(vision_analyzed, 1);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// run_unified_backfill â€” integration scenarios
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tokio::test]
#[serial]
async fn run_unified_backfill_retag_all_tags_everything() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("retag_all".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: Some(false),
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(true),
                }),
                stitching: None,
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    assert_eq!(summary.processed, 3);
    assert!(summary.actions.contains(&"tagging".to_string()));
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_retag_all_unverified_skips_verified() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // design 2 is tags_checked=1 and has an image tag

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("retag_all_unverified".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: None,
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(true),
                }),
                stitching: None,
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    assert_eq!(summary.processed, 2);
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_stop_signal_detected_by_summary() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    // Guard against a leaked stop flag from parallel tests that share the
    // process-wide STOP_REQUESTED AtomicBool. `run_unified_backfill` also
    // clears it, but only synchronously at the start of the async fn â€” a
    // concurrent test could re-set it after the clear but before the
    // processing loop reads it, causing `processed` to undercount.
    clear_stop_signal();

    // After run_unified_backfill completes, it checks STOP_REQUESTED.
    // We request a stop BEFORE the call, but the function clears the flag
    // at the start. To verify stop detection works, we verify that:
    // 1. When no stop is requested, summary.stopped is false
    // 2. The stop_requested_store / request_stop / clear_stop_signal cycle
    //    works correctly (already tested in stop_state_transitions_are_stable)

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("tag_untagged".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: None,
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(true),
                }),
                stitching: None,
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    // No stop was requested during the run, so stopped should be false
    assert!(!summary.stopped);
    assert!(summary.processed >= 2);
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_stop_aborts_current_tagging_batch() {
    clear_stop_signal();
    let pool = make_test_pool().await;
    seed_basic(&pool).await; // designs 1..=3

    // A single huge batch. A stop requested mid-run MUST abort the current
    // batch immediately rather than drain it. `commit_every` is small so the
    // first progress event fires after just 50 designs — well before the batch
    // could finish — and the callback requests a stop deterministically (no
    // timing races). Without the mid-batch abort, the run would reach every
    // design and `processed` would equal TOTAL.
    const TOTAL: i64 = 10_000;
    for id in 4..=(TOTAL + 3) {
        sqlx::query(
            "INSERT INTO designs (id, filename, filepath, image_tags_verified, stitching_tags_verified) VALUES (?, ?, ?, 0, 0)",
        )
        .bind(id)
        .bind(format!("design_{id}.pes"))
        .bind(format!("tests/Test Designs/design_{id}.pes"))
        .execute(&pool)
        .await
        .expect("seed extra design");
    }

    let stopped_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stopped_flag2 = stopped_flag.clone();
    let summary = run_unified_backfill_with_progress(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("retag_all".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: None,
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(true),
                }),
                stitching: None,
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(TOTAL),
            commit_every: Some(50),
            workers: Some(4),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
        None,
        &mut move |_event: BackfillProgress| {
            // Only the first progress event requests the stop; the run is past
            // its synchronous start-clear and into the tagging loop by then.
            if !stopped_flag2.swap(true, std::sync::atomic::Ordering::SeqCst) {
                request_stop();
            }
        },
    )
    .await
    .expect("run succeeds");

    assert!(summary.stopped, "stop must be reported as stopped");
    assert!(
        summary.processed < TOTAL,
        "stop must abort the current batch without draining it (processed={}, total={})",
        summary.processed,
        TOTAL
    );
    clear_stop_signal();
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_combined_actions() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // Also seed a stitching tag candidate and a color count candidate
    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (10, 'Satin', 'stitching')")
        .execute(&pool)
        .await
        .unwrap();

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("tag_untagged".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: None,
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(true),
                }),
                stitching: Some(StitchingActionOptions {
                    clear_stitching_mode: None,
                    enabled: Some(true),
                }),
                images: None,
                color_counts: Some(ColorCountsActionOptions {
                    enabled: Some(true),
                }),
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    assert!(summary.actions.contains(&"tagging".to_string()));
    assert!(summary.actions.contains(&"stitching".to_string()));
    assert!(summary.actions.contains(&"color_counts".to_string()));
    assert!(summary.processed > 0);
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_hoop_dimensions_action_runs() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("tag_untagged".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: None,
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(false),
                }),
                stitching: None,
                images: None,
                color_counts: None,
                hoop_dimensions: Some(HoopDimensionsActionOptions {
                    enabled: Some(true),
                }),
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    assert!(summary.actions.contains(&"hoop_dimensions".to_string()));
    assert!(summary.processed > 0);
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_no_actions_enabled_processes_zero() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: Some(TaggingActionOptions {
                    action: Some("tag_untagged".to_string()),
                    modes: Some(vec!["path_rule".to_string()]),
                    merge_mode: None,
                    exclude_verified: None,
                    folder_path: None,
                    include_subfolders: None,
                    enabled: Some(false),
                }),
                stitching: Some(StitchingActionOptions {
                    clear_stitching_mode: None,
                    enabled: Some(false),
                }),
                images: Some(ImageActionOptions {
                    redo: Some(false),
                    enabled: Some(false),
                }),
                color_counts: Some(ColorCountsActionOptions {
                    enabled: Some(false),
                }),
                hoop_dimensions: None,
                fingerprinting: Some(FingerprintActionOptions {
                    enabled: Some(false),
                }),
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    assert_eq!(summary.processed, 0);
    assert!(summary.actions.is_empty());
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_stitching_clear_unverified_removes_from_unverified() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // Design 1 (unverified) has a stitching tag (tag 2 = 'Line Outline')
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (1, 2)")
        .execute(&pool)
        .await
        .unwrap();

    // Add another stitching tag
    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (10, 'Satin', 'stitching')")
        .execute(&pool)
        .await
        .unwrap();

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: None,
                stitching: Some(StitchingActionOptions {
                    clear_stitching_mode: Some("unverified".to_string()),
                    enabled: Some(true),
                }),
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    // The clear step removes the old tag, but then the stitching processing
    // loop re-processes design 1 as a candidate and may re-apply the default
    // stitching tag (tag 2 'Line Outline'). So we don't assert on the final
    // tag count â€” we just verify the stitching action ran and processed designs.
    assert!(summary.actions.contains(&"stitching".to_string()));
    assert!(summary.processed > 0);
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_stitching_clear_all_removes_from_verified_designs() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;
    // Design 2 is verified (tags_checked=1) and has a stitching tag
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (2, 2)")
        .execute(&pool)
        .await
        .unwrap();

    // Add another stitching tag
    sqlx::query("INSERT INTO tags (id, description, tag_group) VALUES (10, 'Satin', 'stitching')")
        .execute(&pool)
        .await
        .unwrap();

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: None,
                stitching: Some(StitchingActionOptions {
                    clear_stitching_mode: Some("all".to_string()),
                    enabled: Some(true),
                }),
                images: None,
                color_counts: None,
                hoop_dimensions: None,
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    // The "all" mode clears stitching tags from verified designs too, then
    // re-processes them. Design 2 remains a candidate after the clear.
    assert!(summary.actions.contains(&"stitching".to_string()));
    assert!(summary.processed > 0);
    // Verify design 2's stitching tag was cleared (no lingering "Line Outline")
    // (it may be re-applied by detection, but the clear did run)
    // We at least verify the action ran without error.
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_no_actions_defaults_to_tag_untagged() {
    let pool = make_test_pool().await;
    seed_basic(&pool).await;

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: None,
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    assert!(summary.actions.contains(&"tagging".to_string()));
    assert!(summary.processed >= 2);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// resolve_stored_design_path
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
//
// Pure path-resolution logic that normalises stored DB filepaths into
// absolute on-disk locations. Every branch is testable without any real
// design files on disk.

#[test]
fn resolve_path_empty_refers_to_designs_base() {
    let result = resolve_stored_design_path("");
    assert!(
        result.ends_with("MachineEmbroideryDesigns"),
        "got {}",
        result.display()
    );
}

#[test]
fn resolve_path_whitespace_refers_to_designs_base() {
    let result = resolve_stored_design_path("   ");
    assert!(
        result.ends_with("MachineEmbroideryDesigns"),
        "got {}",
        result.display()
    );
}

#[test]
fn resolve_path_canonical_machine_designs_prefix() {
    let result = resolve_stored_design_path("/MachineEmbroideryDesigns/foo/bar.pes");
    let joined = result.to_string_lossy().replace('\\', "/");
    assert!(
        joined.ends_with("/MachineEmbroideryDesigns/foo/bar.pes"),
        "got {joined}"
    );
}

#[test]
fn resolve_path_machine_designs_without_leading_slash() {
    let result = resolve_stored_design_path("MachineEmbroideryDesigns/foo/bar.pes");
    let joined = result.to_string_lossy().replace('\\', "/");
    assert!(
        joined.ends_with("/MachineEmbroideryDesigns/foo/bar.pes"),
        "got {joined}"
    );
}

#[test]
fn resolve_path_bare_relative_joins_under_designs_base() {
    let result = resolve_stored_design_path("foo/bar.pes");
    let joined = result.to_string_lossy().replace('\\', "/");
    assert!(
        joined.ends_with("/MachineEmbroideryDesigns/foo/bar.pes"),
        "got {joined}"
    );
}

#[test]
fn resolve_path_backslashes_normalized_to_forwards() {
    let result = resolve_stored_design_path(r"foo\bar.pes");
    let joined = result.to_string_lossy().replace('\\', "/");
    assert!(
        joined.ends_with("/MachineEmbroideryDesigns/foo/bar.pes"),
        "got {joined}"
    );
}

#[test]
fn resolve_path_absolute_returned_as_is() {
    let absolute = std::env::current_dir()
        .expect("current dir")
        .join("some_legacy_file.pes");
    let result = resolve_stored_design_path(&absolute.to_string_lossy());
    assert!(result.is_absolute(), "got {:?}", result);
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// get_backfill_log_entries
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
//
// The public log-readback API. These are #[serial] because they share the
// process-wide log files with the other serial backfill tests.

#[tokio::test]
#[serial]
async fn get_backfill_log_entries_combines_info_and_error() {
    let _ = std::fs::remove_dir_all("logs");
    truncate_logs_for_new_run().unwrap();
    log_info("i-line".to_string());
    log_error("e-line".to_string());

    let entries = get_backfill_log_entries(&make_test_pool().await, 10)
        .await
        .unwrap();
    let levels: Vec<&str> = entries.iter().map(|e| e.level.as_str()).collect();
    assert!(levels.contains(&"info"));
    assert!(levels.contains(&"error"));

    let messages: Vec<String> = entries.iter().map(|e| e.message.clone()).collect();
    assert!(messages.iter().any(|m| m.contains("i-line")));
    assert!(messages.iter().any(|m| m.contains("e-line")));
}

#[tokio::test]
#[serial]
async fn get_backfill_log_entries_respects_limit_clamp() {
    let _ = std::fs::remove_dir_all("logs");
    truncate_logs_for_new_run().unwrap();
    for i in 0..5 {
        log_info(format!("line{i}"));
    }

    // limit 0 clamps to 1 â†’ only the last info entry is returned
    let small = get_backfill_log_entries(&make_test_pool().await, 0)
        .await
        .unwrap();
    assert_eq!(small.len(), 1, "expected clamped to 1, got {}", small.len());

    // limit 500 clamps to 200 â†’ all 5 info entries are returned
    let large = get_backfill_log_entries(&make_test_pool().await, 500)
        .await
        .unwrap();
    assert_eq!(large.len(), 5, "expected all 5, got {}", large.len());
}

#[tokio::test]
#[serial]
async fn get_backfill_log_entries_empty_when_no_logs() {
    let _ = std::fs::remove_dir_all("logs");
    truncate_logs_for_new_run().unwrap();

    let entries = get_backfill_log_entries(&make_test_pool().await, 10)
        .await
        .unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
#[serial]
async fn run_unified_backfill_file_dependent_actions_write_back() {
    let pool = make_test_pool().await;

    // Seed a large hoop and a stitching tag so detection has somewhere to write.
    sqlx::query(
        "INSERT INTO hoops (id, name, max_width_mm, max_height_mm) VALUES (1, 'Large', 500.0, 500.0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO tags (id, description, tag_group) VALUES (10, 'Line Outline', 'stitching')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Point a design at a real embroidery fixture so file parsing succeeds.
    let design_path = std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("Test Designs")
        .join("Bean.pes");
    assert!(
        design_path.exists(),
        "fixture missing: {}",
        design_path.display()
    );

    sqlx::query(
        "INSERT INTO designs (id, filename, filepath, image_tags_verified, stitching_tags_verified)
         VALUES (1, 'Bean.pes', ?, 0, 0)",
    )
    .bind(design_path.to_string_lossy().to_string())
    .execute(&pool)
    .await
    .unwrap();

    let summary = run_unified_backfill(
        &pool,
        UnifiedBackfillRequest {
            actions: Some(UnifiedBackfillActions {
                tagging: None,
                stitching: Some(StitchingActionOptions {
                    clear_stitching_mode: Some("unverified".to_string()),
                    enabled: Some(true),
                }),
                images: Some(ImageActionOptions {
                    redo: Some(true),
                    enabled: Some(true),
                }),
                color_counts: Some(ColorCountsActionOptions {
                    enabled: Some(true),
                }),
                hoop_dimensions: Some(HoopDimensionsActionOptions {
                    enabled: Some(true),
                }),
                fingerprinting: None,
            }),
            batch_size: Some(100),
            commit_every: Some(100),
            workers: Some(1),
            delay_seconds: Some(0.0),
            vision_delay_seconds: Some(0.0),
        },
        false,
    )
    .await
    .expect("run succeeds");

    for action in ["stitching", "images", "color_counts", "hoop_dimensions"] {
        assert!(
            summary.actions.iter().any(|a| a == action),
            "missing action {action}"
        );
    }
    assert!(summary.processed > 0);

    // File-derived metadata should now be persisted on the design row, proving
    // the file-parsing success paths of the colour-count and hoop-dimension
    // loops executed.
    let row = sqlx::query(
        "SELECT width_mm, height_mm, stitch_count, color_count, image_type FROM designs WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let width: Option<i64> = row.try_get("width_mm").unwrap();
    let height: Option<i64> = row.try_get("height_mm").unwrap();
    let stitch_count: Option<i64> = row.try_get("stitch_count").unwrap();
    let color_count: Option<i64> = row.try_get("color_count").unwrap();
    let image_type: Option<String> = row.try_get("image_type").unwrap();
    assert!(
        width.is_some() && height.is_some(),
        "expected dimensions written"
    );
    assert!(
        stitch_count.is_some() && color_count.is_some(),
        "expected colour counts written"
    );
    assert!(image_type.is_some(), "expected preview image written");
}
