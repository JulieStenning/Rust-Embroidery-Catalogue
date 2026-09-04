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

// ---------------------------------------------------------------------------
// Pure prompt / tag-list builders
// ---------------------------------------------------------------------------

#[test]
fn format_tag_list_joins_sorted_descriptions() {
    let mut set = HashSet::new();
    set.insert("Flowers".to_string());
    set.insert("Cats".to_string());
    assert_eq!(format_tag_list(&set), "Cats\nFlowers");
}

#[test]
fn build_vision_prompt_contains_filename_and_allowed_tags() {
    let mut set = HashSet::new();
    set.insert("Cats".to_string());
    let prompt = build_vision_prompt("cat.pes", &set);
    assert!(prompt.contains("cat.pes"));
    assert!(prompt.contains("Cats"));
    assert!(prompt.contains("Don't Know"));
}

#[test]
fn build_text_prompt_contains_filename_folder_and_allowed_tags() {
    let mut set = HashSet::new();
    set.insert("Flowers".to_string());
    let prompt = build_text_prompt("rose.pes", "/x/rose.pes", &set);
    assert!(prompt.contains("rose.pes"));
    assert!(prompt.contains("/x/rose.pes"));
    assert!(prompt.contains("Flowers"));
    assert!(prompt.contains("Don't Know"));
}

// ---------------------------------------------------------------------------
// Client state / non-network branches
// ---------------------------------------------------------------------------

#[test]
fn with_model_pins_and_set_model_roundtrips() {
    let client = GeminiClient::new("key");
    assert_eq!(client.current_model(), None);
    let pinned = client.with_model("gemini-2.0-flash");
    assert_eq!(pinned.current_model(), Some("gemini-2.0-flash".to_string()));
    pinned.set_model("gemini-2.5-pro".to_string());
    assert_eq!(pinned.current_model(), Some("gemini-2.5-pro".to_string()));
}

#[test]
fn mark_bad_and_is_bad_track_models() {
    let client = GeminiClient::new("key");
    assert!(!client.is_bad("gemini-a"));
    client.mark_bad("gemini-a");
    assert!(client.is_bad("gemini-a"));
    assert!(!client.is_bad("gemini-b"));
}

#[test]
fn resolve_model_returns_pinned_model_without_network() {
    let client = GeminiClient::new("key").with_model("gemini-2.0-flash");
    let model = tauri::async_runtime::block_on(client.resolve_model(Some("other"), false))
        .expect("pinned model should be returned without a probe");
    assert_eq!(model, "gemini-2.0-flash");
}

#[test]
fn validate_model_rejects_empty_name_without_network() {
    let client = GeminiClient::new("key");
    let err = tauri::async_runtime::block_on(client.validate_model("   "))
        .expect_err("empty model name should error");
    assert!(err.to_string().contains("Enter a Gemini model name"));
}

#[test]
fn suggest_tags_text_errors_when_model_unresolved_without_network() {
    let client = GeminiClient::new("key");
    let valid: HashSet<String> = ["Cats".to_string()].into_iter().collect();
    let err = tauri::async_runtime::block_on(client.suggest_tags_text("a.pes", "/a.pes", &valid))
        .expect_err("unresolved model should error before any network call");
    assert!(err.to_string().contains("has not been resolved"));
}

#[test]
fn suggest_tags_vision_errors_when_model_unresolved_without_network() {
    let client = GeminiClient::new("key");
    let valid: HashSet<String> = ["Cats".to_string()].into_iter().collect();
    let err = tauri::async_runtime::block_on(client.suggest_tags_vision("a.pes", b"PNG", &valid))
        .expect_err("unresolved model should error before any network call");
    assert!(err.to_string().contains("has not been resolved"));
}

// ---------------------------------------------------------------------------
// Network methods (mockito-backed)
// ---------------------------------------------------------------------------

fn allowed_tags() -> HashSet<String> {
    ["Cats", "Flowers"].iter().map(|s| s.to_string()).collect()
}

#[test]
fn list_models_filters_generate_content_and_sorts() {
    let mut server = mockito::Server::new();
    let body = serde_json::json!({
        "models": [
            {"name": "models/text-embedding-004", "supportedGenerationMethods": []},
            {"name": "models/gemini-2.0-pro", "supportedGenerationMethods": ["generateContent"]},
            {"name": "models/gemini-2.0-flash", "supportedGenerationMethods": ["generateContent", "embedContent"]}
        ]
    })
    .to_string();
    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .match_query(mockito::Matcher::Any).create();
    let base = format!("{}/", server.url().trim_end_matches('/'));
    let client = GeminiClient::with_base("key", base);
    let models =
        tauri::async_runtime::block_on(client.list_models()).expect("list models should succeed");
    assert_eq!(
        models,
        vec!["gemini-2.0-flash".to_string(), "gemini-2.0-pro".to_string()]
    );
}

#[test]
fn resolve_model_probes_configured_model_and_succeeds() {
    let mut server = mockito::Server::new();
    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::json!({ "models": [] }).to_string())
        .match_query(mockito::Matcher::Any).create();
    server
        .mock("POST", "/gemini-2.0-flash:generateContent")
        .with_status(200)
        .with_body("{}")
        .match_query(mockito::Matcher::Any).create();
    let base = format!("{}/", server.url().trim_end_matches('/'));
    let client = GeminiClient::with_base("key", base);
    let model =
        tauri::async_runtime::block_on(client.resolve_model(Some("gemini-2.0-flash"), false))
            .expect("configured model should resolve");
    assert_eq!(model, "gemini-2.0-flash");
}

#[test]
fn resolve_model_marks_bad_and_falls_back_to_next_candidate() {
    let mut server = mockito::Server::new();
    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({"models": [
                {"name": "models/gemini-2.0-b", "supportedGenerationMethods": ["generateContent"]}
            ]})
            .to_string(),
        )
        .match_query(mockito::Matcher::Any).create();
    server
        .mock("POST", "/gemini-2.0-a:generateContent")
        .with_status(404)
        .with_body("This model is not found")
        .match_query(mockito::Matcher::Any).create();
    server
        .mock("POST", "/gemini-2.0-b:generateContent")
        .with_status(200)
        .with_body("{}")
        .match_query(mockito::Matcher::Any).create();
    let base = format!("{}/", server.url().trim_end_matches('/'));
    let client = GeminiClient::with_base("key", base);
    let model =
        tauri::async_runtime::block_on(client.resolve_model(Some("gemini-2.0-a"), false))
            .expect("should fall back to the next usable model");
    assert_eq!(model, "gemini-2.0-b");
    assert!(client.is_bad("gemini-2.0-a"));
}

#[test]
fn validate_model_reports_usable_and_unusable() {
    let mut ok_server = mockito::Server::new();
    ok_server
        .mock("POST", "/gemini-2.0-flash:generateContent")
        .with_status(200)
        .with_body("{}")
        .match_query(mockito::Matcher::Any).create();
    let ok = tauri::async_runtime::block_on(
        GeminiClient::with_base("key", format!("{}/", ok_server.url().trim_end_matches('/')))
            .validate_model("gemini-2.0-flash"),
    );
    assert!(ok.is_ok());

    let mut bad_server = mockito::Server::new();
    bad_server
        .mock("POST", "/bad:generateContent")
        .with_status(500)
        .with_body("boom")
        .match_query(mockito::Matcher::Any).create();
    let err = tauri::async_runtime::block_on(
        GeminiClient::with_base("key", format!("{}/", bad_server.url().trim_end_matches('/'))).validate_model("bad"),
    )
    .expect_err("unusable model should error");
    assert!(err.to_string().contains("not usable"));
}

#[test]
fn generate_extracts_text_via_suggest_tags() {
    let mut server = mockito::Server::new();
    let body = serde_json::json!({ "candidates": [{ "content": { "parts": [{ "text": "Cats\nFlowers" }] } }] })
        .to_string();
    server
        .mock("POST", "/gemini-2.0-flash:generateContent")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .match_query(mockito::Matcher::Any).create();
    let base = format!("{}/", server.url().trim_end_matches('/'));
    let client = GeminiClient::with_base("key", base).with_model("gemini-2.0-flash");
    let valid = allowed_tags();
    let tags = tauri::async_runtime::block_on(client.suggest_tags_text("cat.pes", "/cat.pes", &valid))
        .expect("tagging should succeed");
    assert_eq!(tags, vec!["Cats".to_string(), "Flowers".to_string()]);
}

#[test]
fn generate_falls_back_to_another_model_on_model_not_found() {
    let mut server = mockito::Server::new();
    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({"models": [
                {"name": "models/gemini-2.0-b", "supportedGenerationMethods": ["generateContent"]}
            ]})
            .to_string(),
        )
        .match_query(mockito::Matcher::Any).create();
    server
        .mock("POST", "/gemini-2.0-a:generateContent")
        .with_status(404)
        .with_body("models/gemini-2.0-a is not found")
        .match_query(mockito::Matcher::Any).create();
    server
        .mock("POST", "/gemini-2.0-b:generateContent")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({ "candidates": [{ "content": { "parts": [{ "text": "Cats" }] } }] })
                .to_string(),
        )
        .match_query(mockito::Matcher::Any).create();
    let base = format!("{}/", server.url().trim_end_matches('/'));
    let client = GeminiClient::with_base("key", base).with_model("gemini-2.0-a");
    let valid = allowed_tags();
    let tags = tauri::async_runtime::block_on(client.suggest_tags_text("c.pes", "/c.pes", &valid))
        .expect("should fall back to another model");
    assert_eq!(tags, vec!["Cats".to_string()]);
}

#[test]
fn generate_rate_limit_returns_error_with_retry_after() {
    let mut server = mockito::Server::new();
    server
        .mock("POST", "/gemini-2.0-flash:generateContent")
        .with_status(429)
        .with_header("retry-after", "30")
        .with_body("quota exceeded")
        .match_query(mockito::Matcher::Any).create();
    let base = format!("{}/", server.url().trim_end_matches('/'));
    let client = GeminiClient::with_base("key", base).with_model("gemini-2.0-flash");
    let valid = allowed_tags();
    let err = tauri::async_runtime::block_on(client.suggest_tags_text("d.pes", "/d.pes", &valid))
        .expect_err("429 should surface as an error");
    let msg = err.to_string();
    assert!(msg.contains("429"));
    assert!(msg.contains("retry_after=30"));
    assert_eq!(retry_after_seconds(&err), Some(30));
}

#[test]
fn list_models_surfaces_http_error() {
    let mut server = mockito::Server::new();
    server.mock("GET", "/").with_status(500).with_body("boom").match_query(mockito::Matcher::Any).create();
    let client = GeminiClient::with_base("key", format!("{}/", server.url().trim_end_matches('/')));
    let err = tauri::async_runtime::block_on(client.list_models()).expect_err("500 should error");
    assert!(err.to_string().contains("500"));
}

#[test]
fn list_models_surfaces_json_parse_error() {
    let mut server = mockito::Server::new();
    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not-json")
        .match_query(mockito::Matcher::Any).create();
    let client = GeminiClient::with_base("key", format!("{}/", server.url().trim_end_matches('/')));
    let err =
        tauri::async_runtime::block_on(client.list_models()).expect_err("bad json should error");
    assert!(err.to_string().contains("parse failed"));
}
