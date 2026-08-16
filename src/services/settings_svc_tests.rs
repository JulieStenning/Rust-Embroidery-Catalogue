// Tests for the source module.
//
// This module was split out so the production file can stay focused
// on logic. It is included via a #[path] declaration in a
// #[cfg(test)] mod tests; module, retaining full access to the
// private items in the parent module through use super::*;.

use super::*;
use crate::database::migrations::run_migrations;
use sqlx::SqlitePool;

// ---------------------------------------------------------------------
// default_for_key
// ---------------------------------------------------------------------
#[test]
fn default_for_key_known_keys() {
    assert_eq!(default_for_key(KEY_AI_TIER2_AUTO), "false");
    assert_eq!(default_for_key(KEY_AI_TIER3_AUTO), "false");
    assert_eq!(default_for_key(KEY_AI_GOOGLE_API_KEY), "");
    assert_eq!(default_for_key(KEY_AI_BATCH_SIZE), "");
    assert_eq!(default_for_key(KEY_AI_DELAY), "");
    assert_eq!(default_for_key(KEY_IMPORT_COMMIT_BATCH_SIZE), "");
    assert_eq!(default_for_key(KEY_IMPORT_LAST_BROWSE_FOLDER), "");
    assert_eq!(default_for_key(KEY_PREVIEW_3D_PROFILE), "balanced");
    assert_eq!(default_for_key(KEY_DB_IDLE_CHECK_INTERVAL_SECS), "1800");
}

#[test]
fn default_for_key_unknown_key_falls_back_to_empty() {
    assert_eq!(default_for_key("unknown.key"), "");
}

// ---------------------------------------------------------------------
// description_for_key
// ---------------------------------------------------------------------
#[test]
fn description_for_key_known_keys() {
    assert!(description_for_key(KEY_AI_TIER2_AUTO).contains("Tier 2"));
    assert!(description_for_key(KEY_AI_TIER3_AUTO).contains("Tier 3"));
    assert!(description_for_key(KEY_AI_GOOGLE_API_KEY).contains("Google Gemini API key"));
    assert!(description_for_key(KEY_AI_BATCH_SIZE).contains("Maximum number of designs"));
    assert!(description_for_key(KEY_AI_DELAY).contains("Seconds to wait"));
    assert!(description_for_key(KEY_IMPORT_COMMIT_BATCH_SIZE)
        .contains("Maximum number of designs to persist"));
    assert!(
        description_for_key(KEY_IMPORT_LAST_BROWSE_FOLDER).contains("Most recently used folder")
    );
    assert!(description_for_key(KEY_PREVIEW_3D_PROFILE).contains("3D preview style"));
    assert!(description_for_key(KEY_DB_IDLE_CHECK_INTERVAL_SECS).contains("Interval in seconds"));
}

#[test]
fn description_for_key_unknown_key_falls_back_to_empty() {
    assert_eq!(description_for_key("unknown.key"), "");
}

// ---------------------------------------------------------------------
// normalize_idle_check_interval
// ---------------------------------------------------------------------
#[test]
fn normalize_idle_check_interval_valid() {
    assert_eq!(normalize_idle_check_interval("  60 "), "60");
}

#[test]
fn normalize_idle_check_interval_clamps_min() {
    assert_eq!(normalize_idle_check_interval("4"), "5");
}

#[test]
fn normalize_idle_check_interval_clamps_max() {
    assert_eq!(normalize_idle_check_interval("90000"), "86400");
}

#[test]
fn normalize_idle_check_interval_invalid_uses_default() {
    assert_eq!(
        normalize_idle_check_interval("abc"),
        crate::services::db_health::DEFAULT_IDLE_CHECK_INTERVAL_SECS.to_string()
    );
}

#[test]
fn normalize_idle_check_interval_negative_uses_default() {
    assert_eq!(
        normalize_idle_check_interval("-5"),
        crate::services::db_health::DEFAULT_IDLE_CHECK_INTERVAL_SECS.to_string()
    );
}

// ---------------------------------------------------------------------
// normalize_optional_batch_size
// ---------------------------------------------------------------------
#[test]
fn normalize_optional_batch_size_valid() {
    assert_eq!(normalize_optional_batch_size("  50 "), "50");
}

#[test]
fn normalize_optional_batch_size_empty_stays_empty() {
    assert_eq!(normalize_optional_batch_size("   "), "");
}

#[test]
fn normalize_optional_batch_size_invalid_stays_empty() {
    assert_eq!(normalize_optional_batch_size("abc"), "");
}

#[test]
fn normalize_optional_batch_size_clamps_min() {
    assert_eq!(normalize_optional_batch_size("0"), "1");
    assert_eq!(normalize_optional_batch_size("-5"), "1");
}

#[test]
fn normalize_optional_batch_size_clamps_max() {
    assert_eq!(normalize_optional_batch_size("20000"), "10000");
}

// ---------------------------------------------------------------------
// normalize_optional_delay
// ---------------------------------------------------------------------
#[test]
fn normalize_optional_delay_whole_number_formats_with_decimal() {
    assert_eq!(normalize_optional_delay("5"), "5.0");
}

#[test]
fn normalize_optional_delay_fractional_preserved() {
    assert_eq!(normalize_optional_delay("2.75"), "2.75");
}

#[test]
fn normalize_optional_delay_empty_stays_empty() {
    assert_eq!(normalize_optional_delay("  "), "");
}

#[test]
fn normalize_optional_delay_negative_stays_empty() {
    assert_eq!(normalize_optional_delay("-1"), "");
}

#[test]
fn normalize_optional_delay_invalid_stays_empty() {
    assert_eq!(normalize_optional_delay("abc"), "");
}

// ---------------------------------------------------------------------
// bool_to_setting
// ---------------------------------------------------------------------
#[test]
fn bool_to_setting_true_returns_true_str() {
    assert_eq!(bool_to_setting(true), "true");
}

#[test]
fn bool_to_setting_false_returns_false_str() {
    assert_eq!(bool_to_setting(false), "false");
}

// ---------------------------------------------------------------------
// is_truthy
// ---------------------------------------------------------------------
#[test]
fn is_truthy_accepts_all_truthy_variants() {
    for value in ["1", "true", "yes", "y", "accepted", "TRUE", " Yes ", "Y"] {
        assert!(is_truthy(value), "expected {value:?} to be truthy");
    }
}

#[test]
fn is_truthy_rejects_falsy_values() {
    for value in ["0", "false", "no", "n", "declined", "", "   "] {
        assert!(!is_truthy(value), "expected {value:?} to be falsy");
    }
}

// ---------------------------------------------------------------------
// get_setting_with_default / upsert_setting (in-memory SQLite)
// ---------------------------------------------------------------------
async fn setup_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("memory db");
    run_migrations(&pool).await.expect("migrations");
    pool
}

#[tokio::test]
async fn get_setting_with_default_returns_existing_value() {
    let pool = setup_pool().await;
    let mut conn = pool.acquire().await.unwrap();
    upsert_setting(&mut conn, KEY_AI_BATCH_SIZE, "42")
        .await
        .unwrap();

    let value = get_setting_with_default(&mut conn, KEY_AI_BATCH_SIZE)
        .await
        .unwrap();
    assert_eq!(value, "42");
}

#[tokio::test]
async fn get_setting_with_default_creates_and_returns_default_when_missing() {
    let pool = setup_pool().await;
    let mut conn = pool.acquire().await.unwrap();

    // KEY_PREVIEW_3D_PROFILE defaults to "balanced"
    let value = get_setting_with_default(&mut conn, KEY_PREVIEW_3D_PROFILE)
        .await
        .unwrap();
    assert_eq!(value, "balanced");

    // Verify the default was actually persisted
    let stored = settings::get_setting(&mut conn, KEY_PREVIEW_3D_PROFILE)
        .await
        .unwrap()
        .expect("default should be persisted");
    assert_eq!(stored.value, "balanced");
}

#[tokio::test]
async fn upsert_setting_inserts_new_row() {
    let pool = setup_pool().await;
    let mut conn = pool.acquire().await.unwrap();
    upsert_setting(&mut conn, KEY_AI_DELAY, "3.5")
        .await
        .unwrap();

    let stored = settings::get_setting(&mut conn, KEY_AI_DELAY)
        .await
        .unwrap()
        .expect("row exists");
    assert_eq!(stored.value, "3.5");
    let description = stored.description.as_deref().expect("description exists");
    assert!(description.contains("Seconds to wait"));
}

#[tokio::test]
async fn upsert_setting_updates_existing_row() {
    let pool = setup_pool().await;
    let mut conn = pool.acquire().await.unwrap();
    upsert_setting(&mut conn, KEY_AI_DELAY, "3.5")
        .await
        .unwrap();
    upsert_setting(&mut conn, KEY_AI_DELAY, "7.0")
        .await
        .unwrap();

    let stored = settings::get_setting(&mut conn, KEY_AI_DELAY)
        .await
        .unwrap()
        .expect("row exists");
    assert_eq!(stored.value, "7.0");
}

#[tokio::test]
async fn get_google_api_key_returns_none_when_empty() {
    let pool = setup_pool().await;
    let mut conn = pool.acquire().await.unwrap();

    let key = get_google_api_key(&mut conn).await.unwrap();
    assert_eq!(key, None);
}

#[tokio::test]
async fn save_and_get_google_api_key_roundtrips() {
    let pool = setup_pool().await;
    let mut conn = pool.acquire().await.unwrap();

    save_google_api_key(&mut conn, "  my-secret-key  ")
        .await
        .unwrap();

    let key = get_google_api_key(&mut conn).await.unwrap();
    assert_eq!(key, Some("my-secret-key".to_string()));

    save_google_api_key(&mut conn, "   ")
        .await
        .unwrap();

    let key_cleared = get_google_api_key(&mut conn).await.unwrap();
    assert_eq!(key_cleared, None);
}

