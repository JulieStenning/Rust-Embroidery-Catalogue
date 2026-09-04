use crate::error::AppError;
use crate::paths::ExecutionMode;
use crate::settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use std::path::{Path, PathBuf};

pub const KEY_AI_VISION_AUTO: &str = "ai.vision";
pub const KEY_AI_GOOGLE_API_KEY: &str = "ai.google_api_key";
pub const KEY_AI_BATCH_SIZE: &str = "ai.batch_size";
pub const KEY_AI_DELAY: &str = "ai.delay";
pub const KEY_AI_GEMINI_MODEL: &str = "ai.gemini_model";
pub const KEY_AI_COMMIT_EVERY: &str = "ai.commit_every";
pub const KEY_AI_WORKERS: &str = "ai.workers";
pub const KEY_AI_FREE_TIER: &str = "ai.free_tier";
pub const KEY_IMPORT_COMMIT_BATCH_SIZE: &str = "import.commit_batch_size";
pub const KEY_IMPORT_LAST_BROWSE_FOLDER: &str = "import.last_browse_folder";
pub const KEY_PREVIEW_3D_PROFILE: &str = "image.preview_3d_profile";
pub const KEY_DB_IDLE_CHECK_INTERVAL_SECS: &str = "db.idle_check_interval_secs";

#[derive(Debug, Clone, Serialize)]
pub struct SettingsViewModel {
    pub preview_3d_profile: String,
    pub google_api_key: String,
    pub has_google_api_key: bool,
    pub ai_vision_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    pub ai_gemini_model: String,
    pub ai_commit_every: String,
    pub ai_workers: String,
    pub ai_free_tier: bool,
    pub import_commit_batch_size: String,
    pub import_last_browse_folder: String,
    pub can_configure_data_root: bool,
    pub data_root: String,
    pub library_root: String,
    pub database_path: String,
    pub log_folder: String,
    pub app_mode: String,
    pub ai_tagging_help_url: String,
    pub db_idle_check_interval_secs: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveSettingsRequest {
    #[serde(default)]
    pub preview_3d_profile: String,
    pub google_api_key: String,
    pub ai_vision_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    #[serde(default)]
    pub ai_gemini_model: String,
    #[serde(default)]
    pub ai_commit_every: String,
    #[serde(default)]
    pub ai_workers: String,
    #[serde(default)]
    pub ai_free_tier: bool,
    pub import_commit_batch_size: String,
    pub data_root: String,
    #[serde(default)]
    pub db_idle_check_interval_secs: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveSettingsResult {
    pub saved: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveImportBrowseFolderResult {
    pub saved: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowseDataRootResult {
    pub path: Option<String>,
    pub error: Option<String>,
}

pub(crate) async fn get_settings_view_model_inner(
    app_state: &AppState,
) -> Result<SettingsViewModel, AppError> {
    let pool = app_state.db_pool().map_err(AppError::database)?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    let preview_3d_profile = get_setting_with_default(&mut conn, KEY_PREVIEW_3D_PROFILE).await?;
    let ai_vision_auto = is_truthy(&get_setting_with_default(&mut conn, KEY_AI_VISION_AUTO).await?);
    let ai_batch_size = get_setting_with_default(&mut conn, KEY_AI_BATCH_SIZE).await?;
    let ai_delay = get_setting_with_default(&mut conn, KEY_AI_DELAY).await?;
    let ai_gemini_model = get_setting_with_default(&mut conn, KEY_AI_GEMINI_MODEL).await?;
    let ai_commit_every = get_setting_with_default(&mut conn, KEY_AI_COMMIT_EVERY).await?;
    let ai_workers = get_setting_with_default(&mut conn, KEY_AI_WORKERS).await?;
    let ai_free_tier = is_truthy(&get_setting_with_default(&mut conn, KEY_AI_FREE_TIER).await?);
    let import_commit_batch_size =
        get_setting_with_default(&mut conn, KEY_IMPORT_COMMIT_BATCH_SIZE).await?;
    let import_last_browse_folder =
        get_setting_with_default(&mut conn, KEY_IMPORT_LAST_BROWSE_FOLDER).await?;
    let db_idle_check_interval_secs =
        get_setting_with_default(&mut conn, KEY_DB_IDLE_CHECK_INTERVAL_SECS).await?;
    let google_api_key = get_setting_with_default(&mut conn, KEY_AI_GOOGLE_API_KEY).await?;
    let has_google_api_key = !google_api_key.trim().is_empty();

    let data_root = app_state.paths.data_root.to_string_lossy().to_string();
    let library_root = app_state
        .paths
        .embroidery_designs_dir
        .to_string_lossy()
        .to_string();
    let database_path = app_state.paths.database_path.to_string_lossy().to_string();
    let log_folder = app_state.paths.log_dir.to_string_lossy().to_string();
    let can_configure_data_root = match app_state.paths.mode {
        ExecutionMode::Dev => true,
        ExecutionMode::Installed => true,
    };
    let app_mode = match app_state.paths.mode {
        ExecutionMode::Dev => "dev".to_string(),
        ExecutionMode::Installed => "installed".to_string(),
    };

    Ok(SettingsViewModel {
        preview_3d_profile,
        google_api_key,
        has_google_api_key,
        ai_vision_auto,
        ai_batch_size,
        ai_delay,
        ai_gemini_model,
        ai_commit_every,
        ai_workers,
        ai_free_tier,
        import_commit_batch_size,
        import_last_browse_folder,
        can_configure_data_root,
        data_root,
        library_root,
        database_path,
        log_folder,
        app_mode,
        ai_tagging_help_url: "#/help".to_string(),
        db_idle_check_interval_secs,
    })
}

pub(crate) async fn save_import_last_browse_folder_inner(
    app_state: &AppState,
    path: String,
) -> Result<SaveImportBrowseFolderResult, AppError> {
    let normalized = path.trim().to_string();
    let pool = app_state.db_pool().map_err(AppError::database)?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    upsert_setting(&mut conn, KEY_IMPORT_LAST_BROWSE_FOLDER, &normalized).await?;

    Ok(SaveImportBrowseFolderResult {
        saved: true,
        path: normalized,
    })
}

pub(crate) async fn save_settings_view_model_inner(
    app_state: &AppState,
    request: SaveSettingsRequest,
) -> Result<SaveSettingsResult, AppError> {
    let preview_3d_profile = normalize_preview_3d_profile(&request.preview_3d_profile);
    let ai_batch_size = normalize_optional_batch_size(&request.ai_batch_size);
    let import_commit_batch_size = normalize_optional_batch_size(&request.import_commit_batch_size);
    let ai_delay = normalize_optional_delay(&request.ai_delay);

    let pool = app_state.db_pool().map_err(AppError::database)?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    upsert_setting(
        &mut conn,
        KEY_AI_VISION_AUTO,
        bool_to_setting(request.ai_vision_auto),
    )
    .await?;
    upsert_setting(&mut conn, KEY_AI_BATCH_SIZE, &ai_batch_size).await?;
    upsert_setting(&mut conn, KEY_AI_DELAY, &ai_delay).await?;
    upsert_setting(
        &mut conn,
        KEY_AI_GEMINI_MODEL,
        request.ai_gemini_model.trim(),
    )
    .await?;
    upsert_setting(
        &mut conn,
        KEY_AI_COMMIT_EVERY,
        request.ai_commit_every.trim(),
    )
    .await?;
    upsert_setting(&mut conn, KEY_AI_WORKERS, request.ai_workers.trim()).await?;
    upsert_setting(
        &mut conn,
        KEY_AI_FREE_TIER,
        bool_to_setting(request.ai_free_tier),
    )
    .await?;
    upsert_setting(
        &mut conn,
        KEY_IMPORT_COMMIT_BATCH_SIZE,
        &import_commit_batch_size,
    )
    .await?;
    upsert_setting(&mut conn, KEY_PREVIEW_3D_PROFILE, &preview_3d_profile).await?;
    upsert_setting(
        &mut conn,
        KEY_DB_IDLE_CHECK_INTERVAL_SECS,
        &normalize_idle_check_interval(&request.db_idle_check_interval_secs),
    )
    .await?;
    upsert_setting(
        &mut conn,
        KEY_AI_GOOGLE_API_KEY,
        request.google_api_key.trim(),
    )
    .await?;

    let _ = request.data_root;

    Ok(SaveSettingsResult {
        saved: true,
        message: "Settings saved successfully.".to_string(),
    })
}

/// Best-effort resolution of the OS-standard user Documents directory.
///
/// This deliberately avoids new dependencies: on Windows it prefers the
/// `USERPROFILE` env var and on macOS/Linux `HOME`. The caller (a Tauri
/// command) normally supplies a more accurate platform Documents path from
/// `app.path().document_dir()`; this function is the fallback seam and is
/// also used directly by tests.
pub(crate) fn standard_documents_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            let candidate = PathBuf::from(profile).join("Documents");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(xdg) = std::env::var("XDG_DOCUMENTS_DIR") {
            let candidate = PathBuf::from(xdg.trim());
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
        if let Some(home) = std::env::var_os("HOME") {
            let candidate = PathBuf::from(home).join("Documents");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Returns `true` when `path` is an existing directory that can be listed
/// (a lightweight read-permission probe). Missing or unreadable paths report
/// `false` so the folder picker can fall back to a safe starting directory.
pub(crate) fn is_readable_dir(path: &Path) -> bool {
    path.is_dir() && std::fs::read_dir(path).is_ok()
}

/// Resolve the starting directory for the native folder picker.
///
/// Priority:
/// 1. The caller's `start_dir` when it is a readable existing directory
///    (trimmed, canonicalised when possible).
/// 2. The supplied fallback Documents directory when it is readable.
/// 3. The process current directory (last resort — always exists).
pub(crate) fn resolve_initial_dir(start_dir: Option<&str>, fallback_docs: &Path) -> PathBuf {
    let trimmed = start_dir.map(str::trim).unwrap_or("");

    if !trimmed.is_empty() {
        let candidate = PathBuf::from(trimmed);
        if is_readable_dir(&candidate) {
            return std::fs::canonicalize(&candidate).unwrap_or(candidate);
        }
    }

    if is_readable_dir(fallback_docs) {
        return std::fs::canonicalize(fallback_docs)
            .unwrap_or_else(|_| fallback_docs.to_path_buf());
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Open a native folder picker for the catalogue data root.
///
/// - The dialog starts at the resolved initial directory (existing value when
///   readable, otherwise the Documents fallback).
/// - A `None` result means the user cancelled — the caller retains state.
/// - The selected native path is returned as a lossy UTF-8 string; the app
///   compares paths via `normalise/canonicalise` elsewhere so this is safe.
pub(crate) fn browse_settings_data_root(
    start_dir: Option<String>,
    fallback_docs: Option<PathBuf>,
) -> BrowseDataRootResult {
    let fallback = fallback_docs
        .or_else(standard_documents_dir)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let initial = resolve_initial_dir(start_dir.as_deref(), &fallback);

    let mut dialog = rfd::FileDialog::new().set_title("Choose the folder for your catalogue data");
    dialog = dialog.set_directory(&initial);

    match dialog.pick_folder() {
        Some(path) => BrowseDataRootResult {
            path: Some(path.to_string_lossy().to_string()),
            error: None,
        },
        None => BrowseDataRootResult {
            path: None,
            error: None,
        },
    }
}

pub(crate) async fn get_setting_with_default(
    conn: &mut SqliteConnection,
    key: &str,
) -> Result<String, AppError> {
    let current = settings::get_setting(conn, key)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;
    if let Some(setting) = current {
        return Ok(setting.value);
    }

    let fallback = default_for_key(key).to_string();
    upsert_setting(conn, key, &fallback).await?;
    Ok(fallback)
}

pub(crate) async fn upsert_setting(
    conn: &mut SqliteConnection,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO settings (key, value, description) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .bind(description_for_key(key))
    .execute(conn)
    .await
    .map_err(|e| AppError::database(e.to_string()))?;
    Ok(())
}

pub(crate) fn default_for_key(key: &str) -> &'static str {
    match key {
        KEY_AI_VISION_AUTO => "false",
        KEY_AI_GOOGLE_API_KEY => "",
        KEY_AI_BATCH_SIZE => "",
        KEY_AI_DELAY => "",
        KEY_AI_GEMINI_MODEL => "",
        KEY_AI_COMMIT_EVERY => "",
        KEY_AI_WORKERS => "",
        KEY_AI_FREE_TIER => "false",
        KEY_IMPORT_COMMIT_BATCH_SIZE => "",
        KEY_PREVIEW_3D_PROFILE => "balanced",
        // Matches crate::services::db_health::DEFAULT_IDLE_CHECK_INTERVAL_SECS.
        KEY_DB_IDLE_CHECK_INTERVAL_SECS => "1800",
        _ => "",
    }
}

pub(crate) fn description_for_key(key: &str) -> &'static str {
    match key {
        KEY_AI_VISION_AUTO => "Run Visual AI (Gemini vision from the preview image) automatically during import when a Google API key is present.",
        KEY_AI_GOOGLE_API_KEY => "Google Gemini API key used for optional automated AI tagging.",
        KEY_AI_BATCH_SIZE => "Maximum number of designs to tag with AI per import run. Leave blank to tag all imported designs.",
        KEY_AI_DELAY => "Seconds to wait between Gemini API calls. Leave blank for no delay on paid, or 10 s on the free tier. Increase if you hit 429 errors.",
        KEY_AI_GEMINI_MODEL => "Gemini model name for Visual AI tagging. Leave blank to auto-select an available model.",
        KEY_AI_COMMIT_EVERY => "How often to report progress/commit during a backfill run (Tagging Actions). Leave blank for the default (100).",
        KEY_AI_WORKERS => "Concurrent designs tagged in parallel by Tagging Actions. Lower this to avoid Gemini rate-limit (429) errors. Leave blank for the default (4).",
        KEY_AI_FREE_TIER => "Whether your Gemini API key is on the free tier. Free-tier keys have strict per-minute and per-day limits; the app stops hard on 429 and tells you how long to wait.",
        KEY_IMPORT_COMMIT_BATCH_SIZE => "Maximum number of designs to persist or update before each database commit during import. Leave blank to use the default batch size (10).",
        KEY_IMPORT_LAST_BROWSE_FOLDER => "Most recently used folder for the bulk import picker.",
        KEY_PREVIEW_3D_PROFILE => "3D preview style profile for native rendering: soft, balanced, or high-contrast.",
        KEY_DB_IDLE_CHECK_INTERVAL_SECS => "Interval in seconds between automatic database fragmentation checks (default 1800).",
        _ => "",
    }
}

pub(crate) fn normalize_idle_check_interval(raw: &str) -> String {
    let value = raw.trim();
    match value.parse::<u64>() {
        Ok(parsed) => parsed.clamp(5, 86_400).to_string(),
        Err(_) => crate::services::db_health::DEFAULT_IDLE_CHECK_INTERVAL_SECS.to_string(),
    }
}

pub(crate) fn normalize_preview_3d_profile(raw: &str) -> String {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "soft" => "soft".to_string(),
        "high-contrast" | "high_contrast" | "highcontrast" => "high-contrast".to_string(),
        _ => "balanced".to_string(),
    }
}

pub(crate) fn normalize_optional_batch_size(raw: &str) -> String {
    let value = raw.trim();
    if value.is_empty() {
        return "".to_string();
    }

    match value.parse::<i64>() {
        Ok(parsed) => parsed.clamp(1, 10_000).to_string(),
        Err(_) => "".to_string(),
    }
}

pub(crate) fn normalize_optional_delay(raw: &str) -> String {
    let value = raw.trim();
    if value.is_empty() {
        return "".to_string();
    }

    match value.parse::<f64>() {
        Ok(parsed) if parsed >= 0.0 => {
            if (parsed.fract() - 0.0).abs() < f64::EPSILON {
                format!("{:.1}", parsed)
            } else {
                parsed.to_string()
            }
        }
        _ => "".to_string(),
    }
}

pub(crate) fn bool_to_setting(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

pub(crate) fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "accepted"
    )
}

pub(crate) async fn get_google_api_key(
    conn: &mut SqliteConnection,
) -> Result<Option<String>, AppError> {
    let key = get_setting_with_default(conn, KEY_AI_GOOGLE_API_KEY).await?;
    let trimmed = key.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

pub(crate) async fn save_google_api_key(
    conn: &mut SqliteConnection,
    value: &str,
) -> Result<(), AppError> {
    upsert_setting(conn, KEY_AI_GOOGLE_API_KEY, value.trim()).await
}
#[cfg(test)]
#[path = "settings_svc_tests.rs"]
mod tests;
