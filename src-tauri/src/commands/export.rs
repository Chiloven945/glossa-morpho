use crate::models::{BatchExportInput, ExportProjectInput, ExportProjectResult};
use crate::project_format::storage::{export_project_batch_files, export_project_files};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn export_project(
    input: ExportProjectInput,
    state: State<'_, AppState>,
) -> Result<ExportProjectResult, String> {
    let projects = state.projects.read();
    let project = projects
        .get(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;
    export_project_files(project, input)
}

#[tauri::command]
pub fn export_project_batch(
    input: BatchExportInput,
    state: State<'_, AppState>,
) -> Result<ExportProjectResult, String> {
    let projects = state.projects.read();
    let project = projects
        .get(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;
    export_project_batch_files(project, input)
}
