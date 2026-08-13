use crate::AppState;
use crate::services::settings::{
    self,
    BrowseDataRootResult, SaveImportBrowseFolderResult, SaveSettingsRequest,
    SaveSettingsResult, SettingsViewModel,
};
use tauri::State;

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
pub fn browse_settings_data_root(start_dir: Option<String>) -> BrowseDataRootResult {
    settings::browse_settings_data_root(start_dir)
}

#[tauri::command]
pub fn get_google_api_key() -> Result<Option<String>, String> {
    Ok(settings::get_google_api_key())
}

#[tauri::command]
pub fn set_google_api_key(api_key: String) -> Result<bool, String> {
    settings::save_google_api_key_to_env(&api_key)
        .map(|_| true)
        .map_err(|err| err.to_string())
}
#[cfg(test)]
#[path = "settings_route_tests.rs"]
mod tests;

