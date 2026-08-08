use crate::AppState;
use crate::services::projects::{
    self,
    CreateProjectRequest, ProjectDetailView, ProjectMutationResult, ProjectPrintView,
    ProjectSummary, RemoveProjectDesignResult, UpdateProjectRequest,
};
use tauri::State;

#[tauri::command]
pub async fn get_projects_list(state: State<'_, AppState>) -> Result<Vec<ProjectSummary>, String> {
    projects::get_projects_list(&state)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    request: CreateProjectRequest,
) -> Result<ProjectMutationResult, String> {
    projects::create_project(&state, request)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_project_detail(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<ProjectDetailView, String> {
    projects::get_project_detail(&state, project_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    project_id: i64,
    request: UpdateProjectRequest,
) -> Result<ProjectMutationResult, String> {
    projects::update_project(&state, project_id, request)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<ProjectMutationResult, String> {
    projects::delete_project(&state, project_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn remove_design_from_project_detail(
    state: State<'_, AppState>,
    project_id: i64,
    design_id: i64,
) -> Result<RemoveProjectDesignResult, String> {
    projects::remove_design_from_project_detail(&state, project_id, design_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_project_print_view(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<ProjectPrintView, String> {
    projects::get_project_print_view(&state, project_id)
        .await
        .map_err(|err| err.to_string())
}