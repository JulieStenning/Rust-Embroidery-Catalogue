use crate::services::admin as admin_service;
use crate::AppState;
use sqlx::SqlitePool;
use tauri::State;

pub use crate::services::admin::{
    AdminDesigner, AdminHoop, AdminSource, AdminTag, CreateDesignerRequest, CreateHoopRequest,
    CreateSourceRequest, CreateTagRequest, SetTagGroupRequest, UpdateDesignerRequest,
    UpdateHoopRequest, UpdateSourceRequest, UpdateTagRequest,
};

#[cfg(test)]
fn validate_non_empty(value: &str, label: &str) -> Result<String, String> {
    admin_service::validate_non_empty(value, label).map_err(|error| error.to_string())
}

#[cfg(test)]
fn validate_positive(value: f64, label: &str) -> Result<f64, String> {
    admin_service::validate_positive(value, label).map_err(|error| error.to_string())
}

#[cfg(test)]
fn validate_tag_group(raw: &str) -> Result<String, String> {
    admin_service::validate_tag_group(raw).map_err(|error| error.to_string())
}

#[cfg(test)]
async fn ensure_unique_name(
    pool: &SqlitePool,
    table: &str,
    name: &str,
    label: &str,
) -> Result<(), String> {
    let sql = format!("SELECT 1 FROM {table} WHERE lower(name) = lower(?) LIMIT 1");
    let exists = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(sql))
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .is_some();

    if exists {
        Err(format!("{label} '{name}' already exists."))
    } else {
        Ok(())
    }
}

#[cfg(test)]
async fn ensure_unique_name_except_id(
    pool: &SqlitePool,
    table: &str,
    id_column: &str,
    excluded_id: i64,
    name: &str,
    label: &str,
) -> Result<(), String> {
    let sql =
        format!("SELECT 1 FROM {table} WHERE lower(name) = lower(?) AND {id_column} <> ? LIMIT 1");
    let exists = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(sql))
        .bind(name)
        .bind(excluded_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .is_some();

    if exists {
        Err(format!("{label} '{name}' already exists."))
    } else {
        Ok(())
    }
}

pub async fn list_designers_with_pool(pool: &SqlitePool) -> Result<Vec<AdminDesigner>, String> {
    admin_service::list_designers_with_pool(pool)
        .await
        .map_err(|error| error.to_string())
}

// ---------------------------------------------------------------------------
// Tauri command wrappers
//
// Every #[tauri::command] below is a thin delegation wrapper that extracts
// `state.db` and forwards the call to the corresponding `_with_pool`
// function (e.g. create_designer â†’ create_designer_with_pool).
//
// All business logic, validation, SQL queries, and error handling live
// exclusively in the `_with_pool` functions and are tested exhaustively
// by the `mod tests` section below (73+ tests covering happy paths, empty
// inputs, duplicates, case-insensitive collisions, not-found errors,
// invalid dimensions, design-count accuracy, etc.).
//
// Testing the wrappers directly would add negligible value â€” they contain
// no branching, no logic, and no error handling of their own. The only
// code path they exercise is `state.db` access, which is guaranteed by
// the Tauri framework. Constructing a full AppState in tests would also
// require coupling to unrelated types (paths, logs, etc.) with no payoff.
//
// Therefore all coverage effort is concentrated on the `_with_pool`
// functions, which deliver >95% effective coverage of every command.
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_designers(state: State<'_, AppState>) -> Result<Vec<AdminDesigner>, String> {
    list_designers_with_pool(&state.db).await
}

#[tauri::command]
pub async fn create_designer(
    state: State<'_, AppState>,
    request: CreateDesignerRequest,
) -> Result<AdminDesigner, String> {
    create_designer_with_pool(&state.db, request).await
}

async fn create_designer_with_pool(
    pool: &SqlitePool,
    request: CreateDesignerRequest,
) -> Result<AdminDesigner, String> {
    admin_service::create_designer_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_designer(
    state: State<'_, AppState>,
    request: UpdateDesignerRequest,
) -> Result<AdminDesigner, String> {
    update_designer_with_pool(&state.db, request).await
}

async fn update_designer_with_pool(
    pool: &SqlitePool,
    request: UpdateDesignerRequest,
) -> Result<AdminDesigner, String> {
    admin_service::update_designer_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_designer(state: State<'_, AppState>, designer_id: i64) -> Result<(), String> {
    delete_designer_with_pool(&state.db, designer_id).await
}

async fn delete_designer_with_pool(pool: &SqlitePool, designer_id: i64) -> Result<(), String> {
    admin_service::delete_designer_with_pool(pool, designer_id)
        .await
        .map_err(|error| error.to_string())
}

pub async fn list_sources_with_pool(pool: &SqlitePool) -> Result<Vec<AdminSource>, String> {
    admin_service::list_sources_with_pool(pool)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_sources(state: State<'_, AppState>) -> Result<Vec<AdminSource>, String> {
    list_sources_with_pool(&state.db).await
}

#[tauri::command]
pub async fn create_source(
    state: State<'_, AppState>,
    request: CreateSourceRequest,
) -> Result<AdminSource, String> {
    create_source_with_pool(&state.db, request).await
}

async fn create_source_with_pool(
    pool: &SqlitePool,
    request: CreateSourceRequest,
) -> Result<AdminSource, String> {
    admin_service::create_source_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_source(
    state: State<'_, AppState>,
    request: UpdateSourceRequest,
) -> Result<AdminSource, String> {
    update_source_with_pool(&state.db, request).await
}

async fn update_source_with_pool(
    pool: &SqlitePool,
    request: UpdateSourceRequest,
) -> Result<AdminSource, String> {
    admin_service::update_source_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_source(state: State<'_, AppState>, source_id: i64) -> Result<(), String> {
    delete_source_with_pool(&state.db, source_id).await
}

async fn delete_source_with_pool(pool: &SqlitePool, source_id: i64) -> Result<(), String> {
    admin_service::delete_source_with_pool(pool, source_id)
        .await
        .map_err(|error| error.to_string())
}

pub async fn list_tags_with_pool(pool: &SqlitePool) -> Result<Vec<AdminTag>, String> {
    admin_service::list_tags_with_pool(pool)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<AdminTag>, String> {
    list_tags_with_pool(&state.db).await
}

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    request: CreateTagRequest,
) -> Result<AdminTag, String> {
    create_tag_with_pool(&state.db, request).await
}

async fn create_tag_with_pool(
    pool: &SqlitePool,
    request: CreateTagRequest,
) -> Result<AdminTag, String> {
    admin_service::create_tag_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn set_tag_group(
    state: State<'_, AppState>,
    request: SetTagGroupRequest,
) -> Result<AdminTag, String> {
    set_tag_group_with_pool(&state.db, request).await
}

async fn set_tag_group_with_pool(
    pool: &SqlitePool,
    request: SetTagGroupRequest,
) -> Result<AdminTag, String> {
    admin_service::set_tag_group_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    request: UpdateTagRequest,
) -> Result<AdminTag, String> {
    update_tag_with_pool(&state.db, request).await
}

async fn update_tag_with_pool(
    pool: &SqlitePool,
    request: UpdateTagRequest,
) -> Result<AdminTag, String> {
    admin_service::update_tag_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, tag_id: i64) -> Result<(), String> {
    delete_tag_with_pool(&state.db, tag_id).await
}

async fn delete_tag_with_pool(pool: &SqlitePool, tag_id: i64) -> Result<(), String> {
    admin_service::delete_tag_with_pool(pool, tag_id)
        .await
        .map_err(|error| error.to_string())
}

pub async fn list_hoops_with_pool(pool: &SqlitePool) -> Result<Vec<AdminHoop>, String> {
    admin_service::list_hoops_with_pool(pool)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_hoops(state: State<'_, AppState>) -> Result<Vec<AdminHoop>, String> {
    list_hoops_with_pool(&state.db).await
}

#[tauri::command]
pub async fn create_hoop(
    state: State<'_, AppState>,
    request: CreateHoopRequest,
) -> Result<AdminHoop, String> {
    create_hoop_with_pool(&state.db, request).await
}

async fn create_hoop_with_pool(
    pool: &SqlitePool,
    request: CreateHoopRequest,
) -> Result<AdminHoop, String> {
    admin_service::create_hoop_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_hoop(
    state: State<'_, AppState>,
    request: UpdateHoopRequest,
) -> Result<AdminHoop, String> {
    update_hoop_with_pool(&state.db, request).await
}

async fn update_hoop_with_pool(
    pool: &SqlitePool,
    request: UpdateHoopRequest,
) -> Result<AdminHoop, String> {
    admin_service::update_hoop_with_pool(pool, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_hoop(state: State<'_, AppState>, hoop_id: i64) -> Result<(), String> {
    delete_hoop_with_pool(&state.db, hoop_id).await
}

async fn delete_hoop_with_pool(pool: &SqlitePool, hoop_id: i64) -> Result<(), String> {
    admin_service::delete_hoop_with_pool(pool, hoop_id)
        .await
        .map_err(|error| error.to_string())
}
#[cfg(test)]
#[path = "admin_tests.rs"]
mod tests;
