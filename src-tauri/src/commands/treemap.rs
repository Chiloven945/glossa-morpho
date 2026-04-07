use crate::models::TreemapNode;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn build_treemap(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TreemapNode>, String> {
    let projects = state.projects.read();
    let project = projects
        .get(&project_id)
        .ok_or_else(|| format!("project not found: {project_id}"))?;
    Ok(project.treemap.clone())
}
