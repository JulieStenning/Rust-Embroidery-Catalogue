// Tests for the Gemini client. Split out of gemini_client.rs (via `#[path]`) so
// the production file stays focused and under the line-count guideline.

use super::*;
use std::collections::HashSet;

#[test]
fn parse_tag_list_exact_and_case_insensitive() {
    let valid: HashSet<String> = ["Cats", "Flowers", "Monograms"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let result = parse_tag_list("Cats\nflowers", &valid);
    assert_eq!(result, vec!["Cats".to_string(), "Flowers".to_string()]);
}

#[test]
fn parse_tag_list_tolerates_commas_and_unknown() {
    let valid: HashSet<String> = ["Cats", "Dogs"].iter().map(|s| s.to_string()).collect();
    let result = parse_tag_list("cats, dogs, zebras;", &valid);
    assert_eq!(result, vec!["Cats".to_string(), "Dogs".to_string()]);
}

#[test]
fn parse_tag_list_empty_when_nothing_matches() {
    let valid: HashSet<String> = ["Cats"].iter().map(|s| s.to_string()).collect();
    assert!(parse_tag_list("Don't Know", &valid).is_empty());
}

#[test]
fn retry_after_seconds_parses_embedded_marker() {
    let error = crate::error::AppError::invalid_input(
        "Gemini API error 429: rate limit (retry_after=420)".to_string(),
    );
    assert_eq!(retry_after_seconds(&error), Some(420));
}

#[test]
fn retry_after_seconds_none_without_marker() {
    let error =
        crate::error::AppError::invalid_input("Gemini API error 400: bad request".to_string());
    assert_eq!(retry_after_seconds(&error), None);
}

#[test]
fn normalize_tag_strips_punctuation() {
    assert_eq!(normalize_tag("Children & Toys"), "children toys");
}

#[test]
fn select_auto_models_prefers_unversioned_alias() {
    let models = vec![
        "gemini-2.0-pro".to_string(),
        "gemini-2.0-flash".to_string(),
        "gemini-flash".to_string(),
    ];
    assert_eq!(
        select_auto_models(&models, false),
        vec![
            "gemini-flash".to_string(),
            "gemini-2.0-flash".to_string(),
            "gemini-2.0-pro".to_string(),
        ]
    );
}

#[test]
fn select_auto_models_prefers_versioned_flash_when_no_alias() {
    let models = vec!["gemini-2.0-pro".to_string(), "gemini-2.0-flash".to_string()];
    assert_eq!(
        select_auto_models(&models, false),
        vec!["gemini-2.0-flash".to_string(), "gemini-2.0-pro".to_string()]
    );
}

#[test]
fn select_auto_models_falls_back_to_any() {
    let models = vec!["gemini-2.5-pro".to_string()];
    assert_eq!(
        select_auto_models(&models, false),
        vec!["gemini-2.5-pro".to_string()]
    );
    assert!(select_auto_models(&[], false).is_empty());
}

#[test]
fn select_auto_models_vision_excludes_embeddings() {
    let models = vec![
        "text-embedding-004".to_string(),
        "gemini-2.0-flash".to_string(),
    ];
    assert_eq!(
        select_auto_models(&models, true),
        vec!["gemini-2.0-flash".to_string()]
    );
}

#[test]
fn is_preferred_alias_matches_unversioned_flash() {
    assert!(is_preferred_alias("gemini-flash"));
    assert!(is_preferred_alias("gemini-flash-lite"));
    assert!(!is_preferred_alias("gemini-2.0-flash"));
    assert!(!is_preferred_alias("gemini-2.5-pro"));
}

#[test]
fn is_vision_capable_excludes_embedding_families() {
    assert!(!is_vision_capable("text-embedding-004"));
    assert!(is_vision_capable("gemini-2.0-flash"));
}

#[test]
fn is_model_not_found_detects_retired_and_not_found() {
    assert!(is_model_not_found(
        404,
        "This model models/gemini-2.5-flash is no longer available to new users."
    ));
    assert!(is_model_not_found(
        404,
        "models/gemini-1.5-flash is not found for API version v1beta"
    ));
    assert!(!is_model_not_found(429, "quota exceeded"));
    assert!(!is_model_not_found(200, ""));
}

#[test]
fn is_rate_limit_error_detects_quota_and_rate_limits() {
    assert!(is_rate_limit_error(&AppError::invalid_input(
        "Gemini API error 429 Too Many Requests: quota exceeded"
    )));
    assert!(is_rate_limit_error(&AppError::invalid_input(
        "Gemini API error 429: RESOURCE_EXHAUSTED"
    )));
    assert!(!is_rate_limit_error(&AppError::invalid_input(
        "Gemini API error 404 model not found"
    )));
    assert!(!is_rate_limit_error(&AppError::invalid_input(
        "database error"
    )));
}
