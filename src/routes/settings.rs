use crate::services::gemini_client::GeminiClient;
use crate::services::settings::{
    self, BrowseDataRootResult, SaveImportBrowseFolderResult, SaveSettingsRequest,
    SaveSettingsResult, SettingsViewModel,
};
use crate::AppState;
use serde::Serialize;
use tauri::{Manager, State};

pub(crate) async fn get_settings_view_model_inner(
    app_state: &AppState,
) -> Result<SettingsViewModel, String> {
    settings::get_settings_view_model_inner(app_state)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_settings_view_model(
    state: State<'_, AppState>,
) -> Result<SettingsViewModel, String> {
    get_settings_view_model_inner(&state).await
}

pub(crate) async fn save_import_last_browse_folder_inner(
    app_state: &AppState,
    path: String,
) -> Result<SaveImportBrowseFolderResult, String> {
    settings::save_import_last_browse_folder_inner(app_state, path)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn save_import_last_browse_folder(
    state: State<'_, AppState>,
    path: String,
) -> Result<SaveImportBrowseFolderResult, String> {
    save_import_last_browse_folder_inner(&state, path).await
}

pub(crate) async fn save_settings_view_model_inner(
    app_state: &AppState,
    request: SaveSettingsRequest,
) -> Result<SaveSettingsResult, String> {
    let result = settings::save_settings_view_model_inner(app_state, request).await;
    if let Err(err) = &result {
        // Capture the underlying cause (e.g. a DB write failure) in the logs so
        // the user can retry and report the exact error when the toast dismisses.
        tracing::error!("Failed to save settings: {}", err);
    }
    result.map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn save_settings_view_model(
    state: State<'_, AppState>,
    request: SaveSettingsRequest,
) -> Result<SaveSettingsResult, String> {
    save_settings_view_model_inner(&state, request).await
}

/// Result of the Settings "Test model" button.
#[derive(Debug, Clone, Serialize)]
pub struct GeminiModelTestResult {
    pub ok: bool,
    pub message: String,
}

/// Client-injectable core for `list_gemini_models`, so the Settings model
/// dropdown can be exercised against a mock server in tests.
pub(crate) async fn list_gemini_models_for_client(
    client: &GeminiClient,
) -> Result<Vec<String>, String> {
    client.list_models().await.map_err(|e| e.to_string())
}

/// List Gemini models available to `api_key` that support `generateContent`
/// (short names, sorted). Populates the Settings model dropdown.
#[tauri::command]
pub async fn list_gemini_models(api_key: String) -> Result<Vec<String>, String> {
    list_gemini_models_for_client(&GeminiClient::new(api_key)).await
}

/// Client-injectable core for `test_gemini_model`, so the ok/err mapping can be
/// exercised against a mock server in tests.
pub(crate) async fn test_gemini_model_for_client(
    client: &GeminiClient,
    model: &str,
) -> Result<GeminiModelTestResult, String> {
    Ok(match client.validate_model(model).await {
        Ok(()) => GeminiModelTestResult {
            ok: true,
            message: format!("Model '{model}' is available and supports generateContent."),
        },
        Err(error) => GeminiModelTestResult {
            ok: false,
            message: error.to_string(),
        },
    })
}

/// Validate that `model` is available to `api_key` and supports
/// `generateContent`. Used by the Settings "Test model" button.
#[tauri::command]
pub async fn test_gemini_model(
    api_key: String,
    model: String,
) -> Result<GeminiModelTestResult, String> {
    test_gemini_model_for_client(&GeminiClient::new(api_key), &model).await
}

#[tauri::command]
pub fn browse_settings_data_root(
    app_handle: tauri::AppHandle,
    start_dir: Option<String>,
) -> BrowseDataRootResult {
    // Best-effort platform Documents path from Tauri; the service helper falls
    // back to an env-derived Documents dir when this is unavailable.
    let fallback_docs = app_handle.path().document_dir().ok();
    settings::browse_settings_data_root(start_dir, fallback_docs)
}

pub(crate) async fn get_google_api_key_inner(
    app_state: &AppState,
) -> Result<Option<String>, String> {
    let pool = app_state.db_pool()?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(|err| err.to_string())?;
    settings::get_google_api_key(&mut conn)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_google_api_key(state: State<'_, AppState>) -> Result<Option<String>, String> {
    get_google_api_key_inner(&state).await
}

pub(crate) async fn set_google_api_key_inner(
    app_state: &AppState,
    api_key: String,
) -> Result<bool, String> {
    let pool = app_state.db_pool()?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(|err| err.to_string())?;
    settings::save_google_api_key(&mut conn, &api_key)
        .await
        .map(|_| true)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn set_google_api_key(
    state: State<'_, AppState>,
    api_key: String,
) -> Result<bool, String> {
    set_google_api_key_inner(&state, api_key).await
}
#[cfg(test)]
#[path = "settings_route_tests.rs"]
mod tests;
