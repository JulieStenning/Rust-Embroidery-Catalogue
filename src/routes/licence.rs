// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::services::licence::{
    check_stored_licence_status, clear_licence, save_licence, verify_licence_key, LicenceStatus,
};
use crate::AppState;
use tauri::State;

pub(crate) async fn get_licence_status_inner(
    app_state: &AppState,
) -> Result<LicenceStatus, String> {
    let pool = app_state.db_pool().map_err(|e| e.to_string())?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    check_stored_licence_status(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_licence_status(state: State<'_, AppState>) -> Result<LicenceStatus, String> {
    get_licence_status_inner(&state).await
}

pub(crate) async fn activate_licence_inner(
    app_state: &AppState,
    email: String,
    licence_key: String,
) -> Result<LicenceStatus, String> {
    let email_trimmed = email.trim();
    let key_trimmed = licence_key.trim();

    let payload = verify_licence_key(email_trimmed, key_trimmed, None, None)
        .map_err(|err| err.to_string())?;

    let pool = app_state.db_pool().map_err(|e| e.to_string())?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;

    save_licence(&mut conn, email_trimmed, key_trimmed, &payload)
        .await
        .map_err(|e| e.to_string())?;

    check_stored_licence_status(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn activate_licence(
    state: State<'_, AppState>,
    email: String,
    licence_key: String,
) -> Result<LicenceStatus, String> {
    activate_licence_inner(&state, email, licence_key).await
}

pub(crate) async fn deactivate_licence_inner(
    app_state: &AppState,
) -> Result<LicenceStatus, String> {
    let pool = app_state.db_pool().map_err(|e| e.to_string())?;
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;

    clear_licence(&mut conn).await.map_err(|e| e.to_string())?;

    check_stored_licence_status(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn deactivate_licence(state: State<'_, AppState>) -> Result<LicenceStatus, String> {
    deactivate_licence_inner(&state).await
}

#[cfg(test)]
#[path = "licence_tests.rs"]
mod tests;
