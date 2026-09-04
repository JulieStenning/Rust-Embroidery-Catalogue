//! Google Gemini API client for Visual AI (vision) tagging.
//!
//! The app is local/offline-first, so Gemini is optional: a client is only
//! constructed when a non-empty API key is configured. The prompt builders and
//! the response parser are pure functions (unit-tested), while the network call
//! is isolated in `GeminiClient::generate`.

use crate::error::AppError;
use base64::Engine as _;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Base URL for the Gemini API.
const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models/";

/// A configured Gemini API client. `reqwest::Client` is `Arc`-backed, so this is
/// cheap to clone and can be shared across concurrent backfill worker tasks.
///
/// The model is resolved lazily and cached (either pinned via
/// [`GeminiClient::with_model`] or auto-selected from the live `ListModels`
/// response). Because Gemini models are renamed/retired over time, `resolve_model`
/// *probes* the chosen model with a real `generateContent` call, and `generate`
/// transparently falls back to another model if the current one later returns a
/// "model not found" 404. Models known to be unusable are remembered so they are
/// not retried within the same process.
#[derive(Clone)]
pub struct GeminiClient {
    api_key: String,
    model: Arc<Mutex<Option<String>>>,
    bad_models: Arc<Mutex<HashSet<String>>>,
    http: reqwest::Client,
    /// Base API endpoint (ends in `/v1beta/models/`). Held as a field so tests can
    /// point the client at a local HTTP mock server; production always uses
    /// [`GEMINI_API_BASE`].
    base_url: String,
}

impl GeminiClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: Arc::new(Mutex::new(None)),
            bad_models: Arc::new(Mutex::new(HashSet::new())),
            http: reqwest::Client::new(),
            base_url: GEMINI_API_BASE.to_string(),
        }
    }

    /// Test-only constructor that points the client at a custom base URL (e.g. a
    /// local HTTP mock server) so the network methods can be exercised without a
    /// real Gemini key. Behaviour is identical to [`GeminiClient::new`] except for
    /// the endpoint base.
    #[cfg(test)]
    pub(crate) fn with_base(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: Arc::new(Mutex::new(None)),
            bad_models: Arc::new(Mutex::new(HashSet::new())),
            http: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Pin a specific model (by short name, e.g. `gemini-2.0-flash`). If set, it
    /// is probed on first use and used as-is (or replaced if unusable).
    pub fn with_model(self, model: impl Into<String>) -> Self {
        self.set_model(model.into());
        self
    }

    /// Return the models available to this API key that support `generateContent`
    /// (short names, sorted). Used to populate the Settings model dropdown.
    pub async fn list_models(&self) -> Result<Vec<String>, AppError> {
        let url = format!("{}?key={}", self.base_url, self.api_key);
        let value = self.get_json(&url).await?;
        let mut names: Vec<String> = value["models"]
            .as_array()
            .map(|models| {
                models
                    .iter()
                    .filter(|m| {
                        m["supportedGenerationMethods"]
                            .as_array()
                            .is_some_and(|arr| {
                                arr.iter().any(|s| s.as_str() == Some("generateContent"))
                            })
                    })
                    .filter_map(|m| {
                        m["name"]
                            .as_str()
                            .map(|name| name.strip_prefix("models/").unwrap_or(name).to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        Ok(names)
    }

    /// Resolve the model to use, caching the result. If `configured` is a
    /// non-empty model name it is tried first; if it is unusable (probe fails) it
    /// is marked bad and the next candidate is auto-selected. The chosen model is
    /// *probed* with a real `generateContent` call so "listed but restricted"
    /// models are rejected up front (fail-fast). `needs_vision` is `true` for
    /// Visual AI and biases auto-selection toward vision-capable models.
    pub async fn resolve_model(
        &self,
        configured: Option<&str>,
        needs_vision: bool,
    ) -> Result<String, AppError> {
        if let Some(model) = self.current_model() {
            if !self.is_bad(&model) {
                return Ok(model);
            }
        }

        // Ordered candidates: the configured model first, then auto-selected ones.
        let mut candidates: Vec<String> = Vec::new();
        if let Some(name) = configured.map(str::trim).filter(|s| !s.is_empty()) {
            candidates.push(name.to_string());
        }
        candidates.extend(self.auto_candidates(needs_vision).await?);

        let mut seen = HashSet::new();
        let mut last_error: Option<AppError> = None;
        for name in candidates {
            if !seen.insert(name.clone()) || self.is_bad(&name) {
                continue;
            }
            match self.probe_model(&name).await {
                Ok(()) => {
                    tracing::info!("Using Gemini model: {name}");
                    self.set_model(name.clone());
                    return Ok(name);
                }
                Err(error) => {
                    self.mark_bad(&name);
                    last_error = Some(error);
                    tracing::warn!("Gemini model '{name}' is unusable; skipping it.");
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AppError::invalid_input(
                "No usable Gemini model could be found for this API key. See the backfill log for per-model errors."
                    .to_string(),
            )
        }))
    }

    /// Validate that `name` works for this API key by sending a real (tiny)
    /// `generateContent` probe. Listing a model does NOT guarantee it is usable —
    /// some models are listed but restricted to new users and return a 404.
    pub async fn validate_model(&self, name: &str) -> Result<(), AppError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::invalid_input(
                "Enter a Gemini model name to test.".to_string(),
            ));
        }
        self.probe_model(name).await.map_err(|error| {
            AppError::invalid_input(format!(
                "Model '{name}' is not usable with this API key: {error}"
            ))
        })
    }

    async fn probe_model(&self, model: &str) -> Result<(), AppError> {
        let payload = json!({ "contents": [{ "parts": [{ "text": "OK" }] }] });
        let url = format!(
            "{}{}:generateContent?key={}",
            self.base_url, model, self.api_key
        );
        let response = self
            .http
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::invalid_input(format!("Gemini probe failed: {e}")))?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(AppError::invalid_input(format!(
                "Gemini API error {status}: {body}"
            )))
        }
    }

    /// Ordered auto-selection candidates (alias-first, then `*-flash`, then any),
    /// excluding models already known to be unusable.
    async fn auto_candidates(&self, needs_vision: bool) -> Result<Vec<String>, AppError> {
        let models = self.list_models().await?;
        let bad = self
            .bad_models
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default();
        Ok(select_auto_models(&models, needs_vision)
            .into_iter()
            .filter(|name| !bad.contains(name))
            .collect())
    }

    fn current_model(&self) -> Option<String> {
        self.model.lock().ok().and_then(|guard| guard.clone())
    }

    fn set_model(&self, model: String) {
        if let Ok(mut guard) = self.model.lock() {
            *guard = Some(model);
        }
    }

    fn mark_bad(&self, model: &str) {
        if let Ok(mut guard) = self.bad_models.lock() {
            guard.insert(model.to_string());
        }
    }

    fn is_bad(&self, model: &str) -> bool {
        self.bad_models
            .lock()
            .ok()
            .is_some_and(|guard| guard.contains(model))
    }

    /// Visual AI: suggest image tags from the design preview image (PNG bytes).
    pub async fn suggest_tags_vision(
        &self,
        filename: &str,
        image_data: &[u8],
        valid_descriptions: &HashSet<String>,
    ) -> Result<Vec<String>, AppError> {
        let inline = json!({
            "mime_type": "image/png",
            "data": base64::engine::general_purpose::STANDARD.encode(image_data),
        });
        let payload = json!({
            "contents": [{
                "parts": [
                    { "inline_data": inline },
                    { "text": build_vision_prompt(filename, valid_descriptions) }
                ]
            }]
        });
        let text = self.generate(payload).await?;
        Ok(parse_tag_list(&text, valid_descriptions))
    }

    /// Text AI: suggest image tags from the file name / folder path alone (no image).
    /// Cheaper and faster than Vision AI since it sends only text tokens.
    pub async fn suggest_tags_text(
        &self,
        filename: &str,
        filepath: &str,
        valid_descriptions: &HashSet<String>,
    ) -> Result<Vec<String>, AppError> {
        let payload = json!({
            "contents": [{
                "parts": [
                    { "text": build_text_prompt(filename, filepath, valid_descriptions) }
                ]
            }]
        });
        let text = self.generate(payload).await?;
        Ok(parse_tag_list(&text, valid_descriptions))
    }

    async fn get_json(&self, url: &str) -> Result<Value, AppError> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::invalid_input(format!("Gemini request failed: {e}")))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::invalid_input(format!(
                "Gemini API error {status}: {body}"
            )));
        }
        response
            .json()
            .await
            .map_err(|e| AppError::invalid_input(format!("Gemini response parse failed: {e}")))
    }

    async fn generate(&self, payload: Value) -> Result<String, AppError> {
        // If the model is retired/unavailable mid-run, fall back to another one
        // and retry the same request, up to a bounded number of attempts.
        let mut attempts = 0u32;
        loop {
            let model = self.current_model().ok_or_else(|| {
                AppError::invalid_input(
                    "Gemini model has not been resolved. Resolve or auto-select a model before tagging."
                        .to_string(),
                )
            })?;
            let url = format!(
                "{}{}:generateContent?key={}",
                self.base_url, model, self.api_key
            );
            let response = self
                .http
                .post(&url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| AppError::invalid_input(format!("Gemini request failed: {e}")))?;

            let status = response.status();
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string());
            if status.is_success() {
                let value: Value = response.json().await.map_err(|e| {
                    AppError::invalid_input(format!("Gemini response parse failed: {e}"))
                })?;
                let text = value["candidates"]
                    .get(0)
                    .and_then(|c| c["content"]["parts"].as_array())
                    .and_then(|parts| parts.first())
                    .and_then(|part| part["text"].as_str())
                    .unwrap_or_default()
                    .to_string();
                return Ok(text);
            }

            let body = response.text().await.unwrap_or_default();
            let error = match retry_after {
                Some(seconds) => AppError::invalid_input(format!(
                    "Gemini API error {status}: {body} (retry_after={seconds})"
                )),
                None => AppError::invalid_input(format!("Gemini API error {status}: {body}")),
            };
            if is_model_not_found(status.as_u16(), &body) {
                self.mark_bad(&model);
                tracing::warn!("Gemini model '{model}' is unavailable; trying another model.");
                attempts += 1;
                if attempts > 3 {
                    return Err(AppError::invalid_input(
                        "Gemini tagging failed: no usable model could be resolved after several attempts. See the backfill log for per-model errors."
                            .to_string(),
                    ));
                }
                // Re-resolve to a non-bad model and retry this same request.
                self.resolve_model(None, false).await?;
                continue;
            }
            return Err(error);
        }
    }
}

/// True when a Gemini `generateContent` error indicates the model itself is not
/// usable for this key (retired, restricted to new users, or simply not found),
/// as opposed to e.g. a rate-limit (429) or quota error that retrying a different
/// model would not fix.
fn is_model_not_found(status: u16, body: &str) -> bool {
    let lower = body.to_lowercase();
    status == 404
        || (lower.contains("model")
            && (lower.contains("no longer available") || lower.contains("is not found")))
}

/// True when a Gemini `AppError` indicates the request quota / rate limit was
/// exceeded (HTTP 429 / `RESOURCE_EXHAUSTED`). These are run-level failures:
/// retrying immediately keeps failing, so the whole tagging run should abort and
/// point the user at the backfill log (and at increasing the AI delay / lowering
/// Workers).
pub fn is_rate_limit_error(error: &crate::error::AppError) -> bool {
    let msg = error.to_string().to_lowercase();
    msg.contains("429")
        || msg.contains("too many requests")
        || msg.contains("resource_exhausted")
        || msg.contains("quota exceeded")
}

/// Extract the `Retry-After` seconds embedded in a rate-limit `AppError` (the
/// `retry_after=<N>` marker written by [`GeminiClient::generate`]) so the UI can
/// tell a free-tier user roughly how long to wait before retrying.
pub fn retry_after_seconds(error: &crate::error::AppError) -> Option<u64> {
    let msg = error.to_string();
    let marker = "retry_after=";
    msg.find(marker).and_then(|idx| {
        let digits: String = msg[idx + marker.len()..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if digits.is_empty() {
            None
        } else {
            digits.parse::<u64>().ok()
        }
    })
}

/// Build the Visual AI (vision) prompt asking for tags from the preview image.
pub fn build_vision_prompt(filename: &str, valid_descriptions: &HashSet<String>) -> String {
    format!(
        "You are tagging embroidery design files. Look at the attached design preview image \
and reply with the image tags, from the allowed list below, that best describe it. Filename \
for context: {filename}. Reply with only the matching tags, one per line, using the exact \
allowed spelling. If none apply, reply exactly: Don't Know.\n\nAllowed tags:\n{}",
        format_tag_list(valid_descriptions)
    )
}

/// Build the Text AI prompt asking for tags from the file name / folder path alone.
pub fn build_text_prompt(
    filename: &str,
    filepath: &str,
    valid_descriptions: &HashSet<String>,
) -> String {
    format!(
        "You are tagging embroidery design files. From the file name and folder path alone, \
reply with the image tags, from the allowed list below, that best describe the design subject. \
Filename: {filename}. Folder: {filepath}. Reply with only the matching tags, one per line, using \
the exact allowed spelling. If none apply, reply exactly: Don't Know.\n\nAllowed tags:\n{}",
        format_tag_list(valid_descriptions)
    )
}

fn format_tag_list(valid_descriptions: &HashSet<String>) -> String {
    let mut list: Vec<&String> = valid_descriptions.iter().collect();
    list.sort();
    list.iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Ordered auto-selection candidates from a sorted list of `generateContent`-capable
/// names, preferring un-versioned `gemini-flash`-style aliases (which track the
/// current model), then versioned `*-flash` models, then any other vision-capable
/// model. When `needs_vision` is set, text-only/embedding families are excluded.
fn select_auto_models(models: &[String], needs_vision: bool) -> Vec<String> {
    let capable: Vec<&String> = models
        .iter()
        .filter(|m| !needs_vision || is_vision_capable(m))
        .collect();
    let mut ordered = Vec::new();
    for name in &capable {
        if is_preferred_alias(name) {
            ordered.push((*name).clone());
        }
    }
    for name in &capable {
        if name.contains("flash") && !is_preferred_alias(name) {
            ordered.push((*name).clone());
        }
    }
    for name in &capable {
        if !name.contains("flash") {
            ordered.push((*name).clone());
        }
    }
    ordered
}

/// True for un-versioned `gemini-flash`-style aliases (e.g. `gemini-flash`,
/// `gemini-flash-lite`) that Gemini keeps pointed at the current flash model.
fn is_preferred_alias(name: &str) -> bool {
    name.starts_with("gemini-flash")
}

/// Heuristic: whether a model name is expected to accept image (vision) input.
/// Gemini's `ListModels` response has no explicit vision flag, so we exclude the
/// obvious text-only families (embeddings).
fn is_vision_capable(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !(lower.contains("embedding") || lower.contains("text-embedding"))
}

/// Extract from a model response the tag descriptions it chose from
/// `valid_descriptions`. Matching is case/whitespace/punctuation-insensitive and
/// tolerant of list separators (newlines, commas, semicolons).
pub fn parse_tag_list(text: &str, valid_descriptions: &HashSet<String>) -> Vec<String> {
    let mut by_normalized: HashMap<String, String> = HashMap::new();
    for description in valid_descriptions {
        by_normalized
            .entry(normalize_tag(description))
            .or_insert_with(|| description.clone());
    }

    let mut matched: HashSet<String> = HashSet::new();
    for raw in text.split(['\n', ',', ';']) {
        let normalized = normalize_tag(raw);
        if normalized.is_empty() {
            continue;
        }
        if let Some(canonical) = by_normalized.get(&normalized) {
            matched.insert(canonical.clone());
        }
    }

    let mut result: Vec<String> = matched.into_iter().collect();
    result.sort();
    result
}

fn normalize_tag(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
#[path = "gemini_client_tests.rs"]
mod tests;
