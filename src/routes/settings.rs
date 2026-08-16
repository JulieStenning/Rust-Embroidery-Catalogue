use crate::services::settings::{
    self, BrowseDataRootResult, SaveImportBrowseFolderResult, SaveSettingsRequest,
    SaveSettingsResult, SettingsViewModel,
};
use crate::AppState;
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
    get_settings_view_model_inner(&*state).await
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
    save_import_last_browse_folder_inner(&*state, path).await
}

pub(crate) async fn save_settings_view_model_inner(
    app_state: &AppState,
    request: SaveSettingsRequest,
) -> Result<SaveSettingsResult, String> {
    settings::save_settings_view_model_inner(app_state, request)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn save_settings_view_model(
    state: State<'_, AppState>,
    request: SaveSettingsRequest,
) -> Result<SaveSettingsResult, String> {
    save_settings_view_model_inner(&*state, request).await
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
    let mut conn = app_state
        .db
        .acquire()
        .await
        .map_err(|err| err.to_string())?;
    settings::get_google_api_key(&mut conn)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_google_api_key(
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    get_google_api_key_inner(&*state).await
}

pub(crate) async fn set_google_api_key_inner(
    app_state: &AppState,
    api_key: String,
) -> Result<bool, String> {
    let mut conn = app_state
        .db
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
    set_google_api_key_inner(&*state, api_key).await
}
#[cfg(test)]
#[path = "settings_route_tests.rs"]
mod tests;
