// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use sqlx::SqlitePool;

#[test]
fn tagging_mode_wire_id() {
    assert_eq!(TaggingMode::FileFolder.wire_id(), "path_rule");
    assert_eq!(TaggingMode::VisualAi.wire_id(), "ai_vision");
    assert_eq!(
        ordered_modes(),
        [TaggingMode::FileFolder, TaggingMode::VisualAi]
    );
}

#[test]
fn resolve_enabled_precedence() {
    let p1 = TaggingPrecedence {
        request_override: Some(true),
        settings_default: Some(false),
        hard_default: false,
    };
    assert!(resolve_enabled(&p1));

    let p2 = TaggingPrecedence {
        request_override: None,
        settings_default: Some(true),
        hard_default: false,
    };
    assert!(resolve_enabled(&p2));

    let p3 = TaggingPrecedence {
        request_override: None,
        settings_default: None,
        hard_default: true,
    };
    assert!(resolve_enabled(&p3));

    let p4 = TaggingPrecedence::default();
    assert!(!resolve_enabled(&p4));
}

#[test]
fn defaults_workers_and_delay() {
    assert_eq!(default_workers_for(true), FREE_TIER_WORKERS);
    assert_eq!(default_workers_for(false), DEFAULT_WORKERS);
    assert_eq!(default_delay_for(true), FREE_TIER_DELAY_SECONDS);
    assert_eq!(default_delay_for(false), DEFAULT_DELAY_SECONDS);
}

#[test]
fn rate_limit_message_formatting() {
    let err = AppError::io("Rate limit exceeded 429");
    let msg_paid = rate_limit_message(&err, false);
    assert!(msg_paid.contains("Gemini tagging aborted"));

    let msg_free = rate_limit_message(&err, true);
    assert!(msg_free.contains("Free-tier Gemini rate limit"));
}

async fn setup_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query(
        "CREATE TABLE tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            tag_group TEXT
        );
        CREATE TABLE designs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            vision_ai_analyzed BOOLEAN,
            vision_ai_matched BOOLEAN,
            image_tags_verified BOOLEAN,
            date_updated TEXT
        );
        CREATE TABLE design_tags (
            design_id INTEGER,
            tag_id INTEGER,
            PRIMARY KEY (design_id, tag_id)
        );",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[tokio::test]
async fn apply_tagging_batch_empty_is_noop() {
    let pool = setup_test_pool().await;
    let map = HashMap::new();
    let res = apply_tagging_batch(&pool, &map, vec![], "reset").await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn apply_tagging_batch_applies_and_resets() {
    let pool = setup_test_pool().await;
    sqlx::query("INSERT INTO tags (id, name, tag_group) VALUES (1, 'Flowers', 'image'), (2, 'Animals', 'image')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO designs (id, vision_ai_analyzed, vision_ai_matched, image_tags_verified, date_updated) VALUES (10, 0, 0, 1, '2026-01-01 10:00:00')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (10, 2)")
        .execute(&pool)
        .await
        .unwrap();

    let mut map = HashMap::new();
    map.insert("Flowers".to_string(), 1i64);

    let entry = TagBatchEntry {
        design_id: 10,
        descriptions: vec!["Flowers".to_string()],
        vision_ai_analyzed: true,
        vision_ai_matched: true,
    };

    apply_tagging_batch(&pool, &map, vec![entry], "reset")
        .await
        .unwrap();

    let tags: Vec<i64> = sqlx::query_scalar("SELECT tag_id FROM design_tags WHERE design_id = 10")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(tags, vec![1]);

    let (analyzed, matched, verified, date_updated): (bool, bool, bool, Option<String>) =
        sqlx::query_as("SELECT vision_ai_analyzed, vision_ai_matched, image_tags_verified, date_updated FROM designs WHERE id = 10")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(analyzed);
    assert!(matched);
    assert!(!verified);
    assert_ne!(date_updated.as_deref(), Some("2026-01-01 10:00:00"));
}

#[tokio::test]
async fn apply_tagging_batch_does_not_stamp_date_updated_when_tags_identical() {
    let pool = setup_test_pool().await;
    sqlx::query("INSERT INTO tags (id, name, tag_group) VALUES (1, 'Flowers', 'image')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO designs (id, vision_ai_analyzed, vision_ai_matched, image_tags_verified, date_updated) VALUES (10, 0, 0, 1, '2026-01-01 10:00:00')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (10, 1)")
        .execute(&pool)
        .await
        .unwrap();

    let mut map = HashMap::new();
    map.insert("Flowers".to_string(), 1i64);

    let entry = TagBatchEntry {
        design_id: 10,
        descriptions: vec!["Flowers".to_string()],
        vision_ai_analyzed: true,
        vision_ai_matched: true,
    };

    // Re-running in reset mode with the exact same tag
    apply_tagging_batch(&pool, &map, vec![entry], "reset")
        .await
        .unwrap();

    let (date_updated, verified): (Option<String>, bool) =
        sqlx::query_as("SELECT date_updated, image_tags_verified FROM designs WHERE id = 10")
            .fetch_one(&pool)
            .await
            .unwrap();
    // date_updated should NOT change since tags did not change
    assert_eq!(date_updated.as_deref(), Some("2026-01-01 10:00:00"));
    assert!(!verified);
}

#[tokio::test]
async fn apply_tagging_batch_does_not_stamp_date_updated_when_no_tags_matched() {
    let pool = setup_test_pool().await;
    sqlx::query("INSERT INTO tags (id, name, tag_group) VALUES (1, 'Flowers', 'image')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO designs (id, vision_ai_analyzed, vision_ai_matched, image_tags_verified, date_updated) VALUES (10, 0, 0, 0, '2026-01-01 10:00:00')")
        .execute(&pool)
        .await
        .unwrap();

    let map = HashMap::new();
    let entry = TagBatchEntry {
        design_id: 10,
        descriptions: vec![],
        vision_ai_analyzed: true,
        vision_ai_matched: false,
    };

    apply_tagging_batch(&pool, &map, vec![entry], "reset")
        .await
        .unwrap();

    let (analyzed, matched, date_updated): (bool, bool, Option<String>) = sqlx::query_as(
        "SELECT vision_ai_analyzed, vision_ai_matched, date_updated FROM designs WHERE id = 10",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(analyzed);
    assert!(!matched);
    assert_eq!(date_updated.as_deref(), Some("2026-01-01 10:00:00"));
}

#[tokio::test]
async fn apply_tagging_batch_add_mode_stamps_date_updated_only_on_new_tags() {
    let pool = setup_test_pool().await;
    sqlx::query("INSERT INTO tags (id, name, tag_group) VALUES (1, 'Flowers', 'image'), (2, 'Birds', 'image')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO designs (id, vision_ai_analyzed, vision_ai_matched, image_tags_verified, date_updated) VALUES (10, 0, 0, 1, '2026-01-01 10:00:00')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO design_tags (design_id, tag_id) VALUES (10, 1)")
        .execute(&pool)
        .await
        .unwrap();

    let mut map = HashMap::new();
    map.insert("Flowers".to_string(), 1i64);
    map.insert("Birds".to_string(), 2i64);

    // In 'add' mode, passing only existing tag 'Flowers' should NOT update date_updated
    let entry_existing = TagBatchEntry {
        design_id: 10,
        descriptions: vec!["Flowers".to_string()],
        vision_ai_analyzed: true,
        vision_ai_matched: true,
    };
    apply_tagging_batch(&pool, &map, vec![entry_existing], "add")
        .await
        .unwrap();

    let date_updated: Option<String> =
        sqlx::query_scalar("SELECT date_updated FROM designs WHERE id = 10")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(date_updated.as_deref(), Some("2026-01-01 10:00:00"));

    // In 'add' mode, passing new tag 'Birds' SHOULD update date_updated
    let entry_new = TagBatchEntry {
        design_id: 10,
        descriptions: vec!["Birds".to_string()],
        vision_ai_analyzed: true,
        vision_ai_matched: true,
    };
    apply_tagging_batch(&pool, &map, vec![entry_new], "add")
        .await
        .unwrap();

    let date_updated2: Option<String> =
        sqlx::query_scalar("SELECT date_updated FROM designs WHERE id = 10")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_ne!(date_updated2.as_deref(), Some("2026-01-01 10:00:00"));
}

#[tokio::test]
async fn compute_tags_for_input_path_rule_only() {
    let mut valid = HashSet::new();
    valid.insert("Flowers".to_string());
    let synonyms = HashMap::new();
    let opts = TaggingModeOptions {
        path_rule_enabled: true,
        visual_ai_enabled: false,
        visual_ai_delay_seconds: 0.0,
        visual_ai_network: false,
    };

    let res = compute_tags_for_input(
        "Flowers_rose.pes",
        "Flowers/rose.pes",
        None,
        &valid,
        &synonyms,
        &opts,
        None,
    )
    .await
    .unwrap();
    assert_eq!(res.descriptions, vec!["Flowers"]);
    assert!(!res.vision_ai_analyzed);
}
