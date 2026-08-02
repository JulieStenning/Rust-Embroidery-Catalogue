use crate::error::AppError;
use crate::paths::ExecutionMode;
use crate::settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use std::path::Path;

pub const KEY_AI_TIER2_AUTO: &str = "ai.tier2_auto";
pub const KEY_AI_TIER3_AUTO: &str = "ai.tier3_auto";
pub const KEY_AI_BATCH_SIZE: &str = "ai.batch_size";
pub const KEY_AI_DELAY: &str = "ai.delay";
pub const KEY_IMPORT_COMMIT_BATCH_SIZE: &str = "import.commit_batch_size";
pub const KEY_IMPORT_LAST_BROWSE_FOLDER: &str = "import.last_browse_folder";
pub const KEY_PREVIEW_3D_PROFILE: &str = "image.preview_3d_profile";

#[derive(Debug, Clone, Serialize)]
pub struct SettingsViewModel {
    pub preview_3d_profile: String,
    pub google_api_key: String,
    pub has_google_api_key: bool,
    pub ai_tier2_auto: bool,
    pub ai_tier3_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    pub import_commit_batch_size: String,
    pub import_last_browse_folder: String,
    pub can_configure_data_root: bool,
    pub data_root: String,
    pub database_path: String,
    pub log_folder: String,
    pub app_mode: String,
    pub ai_tagging_help_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveSettingsRequest {
    #[serde(default)]
    pub preview_3d_profile: String,
    pub google_api_key: String,
    pub ai_tier2_auto: bool,
    pub ai_tier3_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    pub import_commit_batch_size: String,
    pub data_root: String,
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
    let mut conn = app_state
        .db
        .acquire()
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    let preview_3d_profile = get_setting_with_default(&mut conn, KEY_PREVIEW_3D_PROFILE).await?;
    let ai_tier2_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_TIER2_AUTO).await?,
    );
    let ai_tier3_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_TIER3_AUTO).await?,
    );
    let ai_batch_size = get_setting_with_default(&mut conn, KEY_AI_BATCH_SIZE).await?;
    let ai_delay = get_setting_with_default(&mut conn, KEY_AI_DELAY).await?;
    let import_commit_batch_size =
        get_setting_with_default(&mut conn, KEY_IMPORT_COMMIT_BATCH_SIZE).await?;
    let import_last_browse_folder =
        get_setting_with_default(&mut conn, KEY_IMPORT_LAST_BROWSE_FOLDER).await?;

    let google_api_key = std::env::var("GOOGLE_API_KEY").unwrap_or_default();
    let has_google_api_key = !google_api_key.trim().is_empty();

    let data_root = app_state.paths.data_root.to_string_lossy().to_string();
    let database_path = app_state.paths.database_path.to_string_lossy().to_string();
    let log_folder = app_state.paths.log_dir.to_string_lossy().to_string();
    let can_configure_data_root = match app_state.paths.mode {
        ExecutionMode::Portable => false,
        ExecutionMode::Installed => true,
    };
    let app_mode = match app_state.paths.mode {
        ExecutionMode::Portable => "portable".to_string(),
        ExecutionMode::Installed => "installed".to_string(),
    };

    Ok(SettingsViewModel {
        preview_3d_profile,
        google_api_key,
        has_google_api_key,
        ai_tier2_auto,
        ai_tier3_auto,
        ai_batch_size,
        ai_delay,
        import_commit_batch_size,
        import_last_browse_folder,
        can_configure_data_root,
        data_root,
        database_path,
        log_folder,
        app_mode,
        ai_tagging_help_url: "#/help".to_string(),
    })
}

pub(crate) async fn save_import_last_browse_folder_inner(
    app_state: &AppState,
    path: String,
) -> Result<SaveImportBrowseFolderResult, AppError> {
    let normalized = path.trim().to_string();
    let mut conn = app_state
        .db
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

    let mut conn = app_state
        .db
        .acquire()
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    upsert_setting(&mut conn, KEY_AI_TIER2_AUTO, bool_to_setting(request.ai_tier2_auto)).await?;
    upsert_setting(&mut conn, KEY_AI_TIER3_AUTO, bool_to_setting(request.ai_tier3_auto)).await?;
    upsert_setting(&mut conn, KEY_AI_BATCH_SIZE, &ai_batch_size).await?;
    upsert_setting(&mut conn, KEY_AI_DELAY, &ai_delay).await?;
    upsert_setting(&mut conn, KEY_IMPORT_COMMIT_BATCH_SIZE, &import_commit_batch_size).await?;
    upsert_setting(&mut conn, KEY_PREVIEW_3D_PROFILE, &preview_3d_profile).await?;

    save_google_api_key_to_env(&request.google_api_key)?;

    let _ = request.data_root;

    Ok(SaveSettingsResult {
        saved: true,
        message: "Settings saved successfully.".to_string(),
    })
}

pub(crate) fn browse_settings_data_root(start_dir: Option<String>) -> BrowseDataRootResult {
    let from = start_dir.unwrap_or_default();
    BrowseDataRootResult {
        path: None,
        error: Some(format!(
            "Folder picker is not wired yet in this build. Please enter the path manually. Start directory was: {}",
            if from.trim().is_empty() { "(blank)" } else { from.trim() }
        )),
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
        KEY_AI_TIER2_AUTO => "false",
        KEY_AI_TIER3_AUTO => "false",
        KEY_AI_BATCH_SIZE => "",
        KEY_AI_DELAY => "",
        KEY_IMPORT_COMMIT_BATCH_SIZE => "",
        KEY_PREVIEW_3D_PROFILE => "balanced",
        _ => "",
    }
}

pub(crate) fn description_for_key(key: &str) -> &'static str {
    match key {
        KEY_AI_TIER2_AUTO => "Run Tier 2 (Gemini text AI) automatically during import when a Google API key is present.",
        KEY_AI_TIER3_AUTO => "Run Tier 3 (Gemini vision AI) automatically during import when a Google API key is present.",
        KEY_AI_BATCH_SIZE => "Maximum number of designs to tag with AI per import run. Leave blank to tag all imported designs.",
        KEY_AI_DELAY => "Seconds to wait between Gemini API calls. Leave blank to use the default (5.0 seconds).",
        KEY_IMPORT_COMMIT_BATCH_SIZE => "Maximum number of designs to persist or update before each database commit during import. Leave blank to use the default batch size (10).",
        KEY_IMPORT_LAST_BROWSE_FOLDER => "Most recently used folder for the bulk import picker.",
        KEY_PREVIEW_3D_PROFILE => "3D preview style profile for native rendering: soft, balanced, or high-contrast.",
        _ => "",
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

pub(crate) fn save_google_api_key_to_env(value: &str) -> Result<(), AppError> {
    let env_path = Path::new(".env");
    let existing = std::fs::read_to_string(env_path).unwrap_or_default();

    let mut lines = Vec::new();
    let mut replaced = false;

    for line in existing.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("GOOGLE_API_KEY=") {
            if !value.trim().is_empty() {
                lines.push(format!("GOOGLE_API_KEY={}", value.trim()));
            }
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !replaced && !value.trim().is_empty() {
        lines.push(format!("GOOGLE_API_KEY={}", value.trim()));
    }

    let mut output = lines.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }

    std::fs::write(env_path, output)
        .map_err(|e| AppError::io(format!("Failed to update .env: {e}")))?;

    if value.trim().is_empty() {
        std::env::remove_var("GOOGLE_API_KEY");
    } else {
        std::env::set_var("GOOGLE_API_KEY", value.trim());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_preview_3d_profile_whitelists_supported_profiles() {
        assert_eq!(normalize_preview_3d_profile("soft"), "soft");
        assert_eq!(normalize_preview_3d_profile("  SOFT "), "soft");
        assert_eq!(normalize_preview_3d_profile("balanced"), "balanced");
        assert_eq!(
            normalize_preview_3d_profile("high-contrast"),
            "high-contrast"
        );
        assert_eq!(
            normalize_preview_3d_profile("HIGH_CONTRAST"),
            "high-contrast"
        );
        assert_eq!(normalize_preview_3d_profile("other"), "balanced");
    }
}