use crate::config::BootstrapConfig;
use crate::settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use std::path::{Path, PathBuf};
use tauri::State;

const KEY_AI_TIER2_AUTO: &str = "ai.tier2_auto";
const KEY_AI_TIER3_AUTO: &str = "ai.tier3_auto";
const KEY_AI_BATCH_SIZE: &str = "ai.batch_size";
const KEY_AI_DELAY: &str = "ai.delay";
const KEY_IMPORT_COMMIT_BATCH_SIZE: &str = "import.commit_batch_size";
const KEY_IMAGE_PREFERENCE: &str = "image.preference";

#[derive(Debug, Clone, Serialize)]
pub struct SettingsViewModel {
    pub image_preference: String,
    pub google_api_key: String,
    pub has_google_api_key: bool,
    pub ai_tier2_auto: bool,
    pub ai_tier3_auto: bool,
    pub ai_batch_size: String,
    pub ai_delay: String,
    pub import_commit_batch_size: String,
    pub can_configure_data_root: bool,
    pub data_root: String,
    pub log_folder: String,
    pub app_mode: String,
    pub ai_tagging_help_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveSettingsRequest {
    pub image_preference: String,
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
pub struct BrowseDataRootResult {
    pub path: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn get_settings_view_model(state: State<'_, AppState>) -> Result<SettingsViewModel, String> {
    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;

    let image_preference = get_setting_with_default(&mut conn, KEY_IMAGE_PREFERENCE)
        .await
        .map_err(|e| e.to_string())?;
    let ai_tier2_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_TIER2_AUTO)
            .await
            .map_err(|e| e.to_string())?,
    );
    let ai_tier3_auto = is_truthy(
        &get_setting_with_default(&mut conn, KEY_AI_TIER3_AUTO)
            .await
            .map_err(|e| e.to_string())?,
    );
    let ai_batch_size = get_setting_with_default(&mut conn, KEY_AI_BATCH_SIZE)
        .await
        .map_err(|e| e.to_string())?;
    let ai_delay = get_setting_with_default(&mut conn, KEY_AI_DELAY)
        .await
        .map_err(|e| e.to_string())?;
    let import_commit_batch_size = get_setting_with_default(&mut conn, KEY_IMPORT_COMMIT_BATCH_SIZE)
        .await
        .map_err(|e| e.to_string())?;

    let google_api_key = std::env::var("GOOGLE_API_KEY").unwrap_or_default();
    let has_google_api_key = !google_api_key.trim().is_empty();

    let data_root = derive_data_root_from_database_url();
    let log_folder = derive_log_folder_from_data_root(&data_root);

    Ok(SettingsViewModel {
        image_preference,
        google_api_key,
        has_google_api_key,
        ai_tier2_auto,
        ai_tier3_auto,
        ai_batch_size,
        ai_delay,
        import_commit_batch_size,
        can_configure_data_root: false,
        data_root,
        log_folder,
        app_mode: "development".to_string(),
        ai_tagging_help_url: "#/help".to_string(),
    })
}

#[tauri::command]
pub async fn save_settings_view_model(
    state: State<'_, AppState>,
    request: SaveSettingsRequest,
) -> Result<SaveSettingsResult, String> {
    let image_preference = normalize_image_preference(&request.image_preference);
    let ai_batch_size = normalize_optional_batch_size(&request.ai_batch_size);
    let import_commit_batch_size = normalize_optional_batch_size(&request.import_commit_batch_size);
    let ai_delay = normalize_optional_delay(&request.ai_delay);

    let mut conn = state.db.acquire().await.map_err(|e| e.to_string())?;

    upsert_setting(&mut conn, KEY_AI_TIER2_AUTO, bool_to_setting(request.ai_tier2_auto))
        .await
        .map_err(|e| e.to_string())?;
    upsert_setting(&mut conn, KEY_AI_TIER3_AUTO, bool_to_setting(request.ai_tier3_auto))
        .await
        .map_err(|e| e.to_string())?;
    upsert_setting(&mut conn, KEY_AI_BATCH_SIZE, &ai_batch_size)
        .await
        .map_err(|e| e.to_string())?;
    upsert_setting(&mut conn, KEY_AI_DELAY, &ai_delay)
        .await
        .map_err(|e| e.to_string())?;
    upsert_setting(
        &mut conn,
        KEY_IMPORT_COMMIT_BATCH_SIZE,
        &import_commit_batch_size,
    )
    .await
    .map_err(|e| e.to_string())?;
    upsert_setting(&mut conn, KEY_IMAGE_PREFERENCE, &image_preference)
        .await
        .map_err(|e| e.to_string())?;

    save_google_api_key_to_env(&request.google_api_key)?;

    // Data-root persistence is intentionally deferred until desktop mode support is fully wired.
    let _ = request.data_root;

    Ok(SaveSettingsResult {
        saved: true,
        message: "Settings saved successfully.".to_string(),
    })
}

#[tauri::command]
pub fn browse_settings_data_root(start_dir: Option<String>) -> BrowseDataRootResult {
    let from = start_dir.unwrap_or_default();
    BrowseDataRootResult {
        path: None,
        error: Some(format!(
            "Folder picker is not wired yet in this build. Please enter the path manually. Start directory was: {}",
            if from.trim().is_empty() { "(blank)" } else { from.trim() }
        )),
    }
}

async fn get_setting_with_default(conn: &mut SqliteConnection, key: &str) -> Result<String, sqlx::Error> {
    let current = settings::get_setting(conn, key).await?;
    if let Some(setting) = current {
        return Ok(setting.value);
    }

    let fallback = default_for_key(key).to_string();
    upsert_setting(conn, key, &fallback).await?;
    Ok(fallback)
}

async fn upsert_setting(conn: &mut SqliteConnection, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings (key, value, description) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .bind(description_for_key(key))
    .execute(conn)
    .await?;
    Ok(())
}

fn default_for_key(key: &str) -> &'static str {
    match key {
        KEY_AI_TIER2_AUTO => "false",
        KEY_AI_TIER3_AUTO => "false",
        KEY_AI_BATCH_SIZE => "",
        KEY_AI_DELAY => "",
        KEY_IMPORT_COMMIT_BATCH_SIZE => "",
        KEY_IMAGE_PREFERENCE => "2d",
        _ => "",
    }
}

fn description_for_key(key: &str) -> &'static str {
    match key {
        KEY_AI_TIER2_AUTO => "Run Tier 2 (Gemini text AI) automatically during import when a Google API key is present.",
        KEY_AI_TIER3_AUTO => "Run Tier 3 (Gemini vision AI) automatically during import when a Google API key is present.",
        KEY_AI_BATCH_SIZE => "Maximum number of designs to tag with AI per import run. Leave blank to tag all imported designs.",
        KEY_AI_DELAY => "Seconds to wait between Gemini API calls. Leave blank to use the default (5.0 seconds).",
        KEY_IMPORT_COMMIT_BATCH_SIZE => "Maximum number of designs to persist or update before each database commit during import. Leave blank to use the default batch size (1000).",
        KEY_IMAGE_PREFERENCE => "Preferred preview image type for import-created previews: 2d or 3d.",
        _ => "",
    }
}

fn normalize_image_preference(raw: &str) -> String {
    let value = raw.trim().to_ascii_lowercase();
    if value == "3d" {
        "3d".to_string()
    } else {
        "2d".to_string()
    }
}

fn normalize_optional_batch_size(raw: &str) -> String {
    let value = raw.trim();
    if value.is_empty() {
        return "".to_string();
    }

    match value.parse::<i64>() {
        Ok(parsed) => parsed.clamp(1, 10_000).to_string(),
        Err(_) => "".to_string(),
    }
}

fn normalize_optional_delay(raw: &str) -> String {
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

fn bool_to_setting(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "accepted"
    )
}

fn strip_sqlite_prefix(database_url: &str) -> &str {
    database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url)
}

fn derive_data_root_from_database_url() -> String {
    let config = BootstrapConfig::from_env();
    let db_path = Path::new(strip_sqlite_prefix(&config.database_url));

    let root = if let Some(parent) = db_path.parent() {
        if parent
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case("database"))
            .unwrap_or(false)
        {
            parent.parent().unwrap_or(parent)
        } else {
            parent
        }
    } else {
        Path::new("data")
    };

    root.canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn derive_log_folder_from_data_root(data_root: &str) -> String {
    let log_path = PathBuf::from(data_root).join("logs");
    log_path
        .canonicalize()
        .unwrap_or(log_path)
        .to_string_lossy()
        .to_string()
}

fn save_google_api_key_to_env(value: &str) -> Result<(), String> {
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

    std::fs::write(env_path, output).map_err(|e| format!("Failed to update .env: {}", e))?;

    if value.trim().is_empty() {
        std::env::remove_var("GOOGLE_API_KEY");
    } else {
        std::env::set_var("GOOGLE_API_KEY", value.trim());
    }

    Ok(())
}
