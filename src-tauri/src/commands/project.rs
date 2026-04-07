use crate::models::{BootstrapResponse, CreateProjectInput, ProjectWorkspace};
use crate::state::{demo_project, AppState};
use tauri::State;

#[tauri::command]
pub fn bootstrap_workspace(state: State<'_, AppState>) -> BootstrapResponse {
    state.bootstrap()
}

#[tauri::command]
pub fn create_project(
    input: CreateProjectInput,
    state: State<'_, AppState>,
) -> Result<ProjectWorkspace, String> {
    let project = demo_project(
        &input.name,
        "/Users/you/projects/starter.gmproj",
        &input.source_locale,
        &input.target_locale,
    );
    Ok(state.insert_project(project))
}

#[tauri::command]
pub fn open_project(path: String, state: State<'_, AppState>) -> Result<ProjectWorkspace, String> {
    let project = demo_project("Opened Demo Project", &path, "en-US", "zh-CN");
    Ok(state.insert_project(project))
}

#[tauri::command]
pub fn save_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&project_id)
        .ok_or_else(|| format!("project not found: {project_id}"))?;
    project.dirty = false;
    Ok(serde_json::json!({
        "projectId": project_id,
        "savedAt": chrono::Utc::now().to_rfc3339()
    }))
}
