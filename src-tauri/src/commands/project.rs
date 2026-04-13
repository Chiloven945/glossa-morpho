use crate::editing::refresh_project;
use crate::models::{
    BootstrapResponse, CreateProjectInput, CreateResourceFileInput, DeleteResourceFileInput,
    ProjectWorkspace, RenameResourceFileInput, ResourceFileNode, UpdateProjectMetadataInput,
};
use crate::project_format::manifest::normalize_archive_format;
use crate::project_format::storage::{
    create_project_file_path, load_workspace, save_workspace, scaffold_workspace,
};
use crate::state::AppState;
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
    let project_path = create_project_file_path(input.path.as_deref(), &input.name)?
        .to_string_lossy()
        .to_string();

    let project = scaffold_workspace(
        &input.name,
        &project_path,
        input.locale_graph,
        input.primary_locale,
        input.working_locale,
        normalize_archive_format(input.archive_format.as_deref().unwrap_or("lzma2")),
        input
            .key_segmentation_profiles
            .unwrap_or_else(|| vec!["dot".into(), "camel".into()]),
    );

    save_workspace(&project)?;
    state.push_recent_project(&project.path);
    Ok(state.insert_project(project))
}

#[tauri::command]
pub fn open_project(path: String, state: State<'_, AppState>) -> Result<ProjectWorkspace, String> {
    let project = load_workspace(&path)?;
    state.push_recent_project(&project.path);
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

    let save_summary = save_workspace(project)?;
    project.dirty = false;
    state.push_recent_project(&project.path);

    Ok(serde_json::json!({
        "projectId": project_id,
        "path": project.path,
        "savedAt": save_summary.saved_at
    }))
}

#[tauri::command]
pub fn save_project_as(
    project_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let current = {
        let projects = state.projects.read();
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| format!("project not found: {project_id}"))?
    };

    let target_path = create_project_file_path(Some(&path), &current.name)?;
    let mut next = current.clone();
    next.path = target_path.to_string_lossy().to_string();

    let save_summary = save_workspace(&next)?;
    next.dirty = false;
    state.push_recent_project(&next.path);
    let next = state.insert_project(next);

    Ok(serde_json::json!({
        "project": next,
        "savedAt": save_summary.saved_at
    }))
}

fn descendant_locales(project: &ProjectWorkspace, locale_code: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut stack = vec![locale_code.to_string()];

    while let Some(current) = stack.pop() {
        for child in project
            .locale_graph
            .iter()
            .filter(|node| node.parent_code.as_deref() == Some(current.as_str()))
        {
            result.push(child.code.clone());
            stack.push(child.code.clone());
        }
    }

    result
}

fn replace_locale_in_logical_path(
    logical_path: &str,
    from_locale: &str,
    to_locale: &str,
) -> String {
    logical_path
        .replace(&format!("/{from_locale}/"), &format!("/{to_locale}/"))
        .replace(&format!(".{from_locale}."), &format!(".{to_locale}."))
}

fn normalize_resource_group_key(file: &ResourceFileNode) -> String {
    file.logical_path
        .replace('\\', "/")
        .replace(&format!("/{}/", file.locale), "/{locale}/")
        .replace(&format!(".{}.", file.locale), ".{locale}.")
        .to_lowercase()
}

fn related_file_ids(project: &ProjectWorkspace, file_id: &str) -> Vec<String> {
    let Some(source) = project.files.iter().find(|file| file.id == file_id) else {
        return vec![];
    };
    let key = normalize_resource_group_key(source);
    project
        .files
        .iter()
        .filter(|file| normalize_resource_group_key(file) == key)
        .map(|file| file.id.clone())
        .collect()
}

fn build_linked_files(
    project: &ProjectWorkspace,
    input: &CreateResourceFileInput,
) -> Vec<ResourceFileNode> {
    let mut locales = vec![input.locale.clone()];
    if input.include_descendants.unwrap_or(true) {
        locales.extend(descendant_locales(project, &input.locale));
    }

    locales
        .into_iter()
        .filter_map(|locale| {
            let logical_path = if locale == input.locale {
                input.logical_path.clone()
            } else {
                replace_locale_in_logical_path(&input.logical_path, &input.locale, &locale)
            };

            if project
                .files
                .iter()
                .any(|file| file.locale == locale && file.logical_path == logical_path)
            {
                return None;
            }

            let based_on_locale = if locale == input.locale {
                input.based_on_locale.clone().or_else(|| {
                    project
                        .locale_graph
                        .iter()
                        .find(|node| node.code == locale)
                        .and_then(|node| node.parent_code.clone())
                })
            } else {
                project
                    .locale_graph
                    .iter()
                    .find(|node| node.code == locale)
                    .and_then(|node| node.parent_code.clone())
            };

            let name = logical_path
                .rsplit('/')
                .next()
                .map(ToString::to_string)
                .unwrap_or_else(|| input.name.clone());

            Some(ResourceFileNode {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                logical_path,
                format: input.format.clone(),
                locale,
                based_on_locale,
                raw_relative_path: None,
            })
        })
        .collect()
}

#[tauri::command]
pub fn create_resource_file(
    input: CreateResourceFileInput,
    state: State<'_, AppState>,
) -> Result<ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    if project
        .files
        .iter()
        .any(|file| file.locale == input.locale && file.logical_path == input.logical_path)
    {
        return Err("A file with the same locale and logical path already exists".into());
    }

    let new_files = build_linked_files(project, &input);
    project.files.extend(new_files);
    project.files.sort_by(|a, b| {
        a.logical_path
            .cmp(&b.logical_path)
            .then(a.locale.cmp(&b.locale))
    });
    project.dirty = true;
    Ok(project.clone())
}

#[tauri::command]
pub fn rename_resource_file(
    input: RenameResourceFileInput,
    state: State<'_, AppState>,
) -> Result<ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    let source = project
        .files
        .iter()
        .find(|file| file.id == input.file_id)
        .cloned()
        .ok_or_else(|| format!("resource file not found: {}", input.file_id))?;

    let target_ids = if input.include_related.unwrap_or(true) {
        related_file_ids(project, &input.file_id)
    } else {
        vec![input.file_id.clone()]
    };

    for file_id in &target_ids {
        let Some(current) = project
            .files
            .iter()
            .find(|file| &file.id == file_id)
            .cloned()
        else {
            continue;
        };
        let next_path = if current.locale == source.locale {
            input.logical_path.clone()
        } else {
            replace_locale_in_logical_path(&input.logical_path, &source.locale, &current.locale)
        };
        let next_name = next_path
            .rsplit('/')
            .next()
            .map(ToString::to_string)
            .unwrap_or_else(|| input.name.clone());

        let conflict = project.files.iter().any(|file| {
            file.id != current.id && file.locale == current.locale && file.logical_path == next_path
        });
        if conflict {
            return Err(format!(
                "a file already exists at '{}' for locale {}",
                next_path, current.locale
            ));
        }

        if let Some(file) = project.files.iter_mut().find(|file| file.id == current.id) {
            file.logical_path = next_path.clone();
            file.name = next_name;
        }
        for detail in project.details.values_mut() {
            if detail.summary.file_id == current.id {
                detail.file_path = next_path.clone();
            }
        }
    }

    project.files.sort_by(|a, b| {
        a.logical_path
            .cmp(&b.logical_path)
            .then(a.locale.cmp(&b.locale))
    });
    project.dirty = true;
    refresh_project(project);
    Ok(project.clone())
}

#[tauri::command]
pub fn delete_resource_file(
    input: DeleteResourceFileInput,
    state: State<'_, AppState>,
) -> Result<ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    let target_ids = if input.include_related.unwrap_or(true) {
        related_file_ids(project, &input.file_id)
    } else {
        vec![input.file_id.clone()]
    };

    if target_ids.is_empty() {
        return Err("resource file not found".into());
    }

    project.files.retain(|file| !target_ids.contains(&file.id));
    let entry_ids = project
        .entries
        .iter()
        .filter(|entry| target_ids.contains(&entry.file_id))
        .map(|entry| entry.id.clone())
        .collect::<Vec<_>>();
    project
        .entries
        .retain(|entry| !target_ids.contains(&entry.file_id));
    for entry_id in entry_ids {
        project.details.remove(&entry_id);
    }

    project.dirty = true;
    refresh_project(project);
    Ok(project.clone())
}

#[tauri::command]
pub fn update_project_metadata(
    input: UpdateProjectMetadataInput,
    state: State<'_, AppState>,
) -> Result<ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    project.name = input.name;
    project.primary_locale = input.primary_locale;
    project.working_locale = input.working_locale;
    project.archive_format = normalize_archive_format(&input.archive_format);
    project.key_segmentation_profiles = input
        .key_segmentation_profiles
        .into_iter()
        .filter(|item| !item.trim().is_empty())
        .collect();
    if project.key_segmentation_profiles.is_empty() {
        project.key_segmentation_profiles = vec!["dot".into(), "camel".into()];
    }
    project.default_view = input.default_view;
    project.default_sort = input.default_sort;
    project.dirty = true;

    Ok(project.clone())
}
