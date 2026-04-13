use crate::editing::{
    refresh_entry_detail, refresh_project, sync_summary_from_detail, upsert_history_candidate,
};
use crate::models::{
    BulkReplaceInput, BulkReplaceResult, CreateEntryInput, DeleteEntriesInput, DeleteEntryInput,
    EntryDetail, EntrySummary, HistoryEvent, ProjectWorkspace, ResourceFileNode, UpdateEntryInput,
};
use crate::state::AppState;
use chrono::Utc;
use regex::Regex;
use tauri::State;
use uuid::Uuid;

fn normalize_resource_group_key(file: &ResourceFileNode) -> String {
    file.logical_path
        .replace('\\', "/")
        .replace(&format!("/{}/", file.locale), "/{locale}/")
        .replace(&format!(".{}.", file.locale), ".{locale}.")
        .to_lowercase()
}

fn file_group_map(project: &ProjectWorkspace, file_id: &str) -> Vec<ResourceFileNode> {
    let Some(source) = project.files.iter().find(|file| file.id == file_id) else {
        return vec![];
    };
    let key = normalize_resource_group_key(source);
    let mut files = project
        .files
        .iter()
        .filter(|file| normalize_resource_group_key(file) == key)
        .cloned()
        .collect::<Vec<_>>();
    files.sort_by(|a, b| {
        a.logical_path
            .cmp(&b.logical_path)
            .then(a.locale.cmp(&b.locale))
    });
    files
}

fn direct_child_files(
    project: &ProjectWorkspace,
    parent_file: &ResourceFileNode,
) -> Vec<ResourceFileNode> {
    let group = file_group_map(project, &parent_file.id);
    group
        .into_iter()
        .filter(|file| file.based_on_locale.as_deref() == Some(parent_file.locale.as_str()))
        .collect()
}

fn entry_exists(project: &ProjectWorkspace, file_id: &str, key: &str) -> bool {
    project
        .entries
        .iter()
        .any(|entry| entry.file_id == file_id && entry.key == key)
}

fn find_entry_id(project: &ProjectWorkspace, file_id: &str, key: &str) -> Option<String> {
    project
        .entries
        .iter()
        .find(|entry| entry.file_id == file_id && entry.key == key)
        .map(|entry| entry.id.clone())
}

fn is_root_file(file: &ResourceFileNode) -> bool {
    file.based_on_locale.is_none()
}

fn build_detail(
    _project: &ProjectWorkspace,
    file: &ResourceFileNode,
    key: String,
    source_value: String,
    target_value: String,
    note: String,
    status: Option<String>,
    updated_at: &str,
    action: &str,
) -> EntryDetail {
    let normalized_status = status.unwrap_or_else(|| {
        if target_value.trim().is_empty() {
            "new".into()
        } else {
            "translated".into()
        }
    });

    let source_locale = if is_root_file(file) {
        String::new()
    } else {
        file.based_on_locale.clone().unwrap_or_default()
    };

    EntryDetail {
        summary: EntrySummary {
            id: Uuid::new_v4().to_string(),
            file_id: file.id.clone(),
            key,
            source_value: if is_root_file(file) {
                String::new()
            } else {
                source_value.clone()
            },
            target_value: target_value.clone(),
            status: normalized_status,
            note_count: usize::from(!note.trim().is_empty()),
            candidate_count: 0,
            updated_at: updated_at.to_string(),
        },
        file_path: file.logical_path.clone(),
        source_locale,
        target_locale: file.locale.clone(),
        note,
        issues: vec![],
        candidates: vec![],
        history: vec![HistoryEvent {
            id: Uuid::new_v4().to_string(),
            action: action.into(),
            before_value: String::new(),
            after_value: target_value,
            operator: "desktop-user".into(),
            created_at: updated_at.to_string(),
        }],
    }
}

fn create_descendant_entries(
    project: &mut ProjectWorkspace,
    parent_file: &ResourceFileNode,
    key: &str,
    parent_target_value: &str,
    updated_at: &str,
) {
    let children = direct_child_files(project, parent_file);

    for child_file in children {
        let child_target_value =
            if let Some(existing_id) = find_entry_id(project, &child_file.id, key) {
                let mut next_target = String::new();
                if let Some(detail) = project.details.get_mut(&existing_id) {
                    let previous_source = detail.summary.source_value.clone();
                    if previous_source != parent_target_value {
                        detail.summary.source_value = parent_target_value.to_string();
                        detail.source_locale = parent_file.locale.clone();
                        detail.summary.updated_at = updated_at.to_string();
                        if detail.summary.target_value.is_empty() {
                            detail.summary.status = "new".into();
                        } else {
                            detail.summary.status = "stale".into();
                        }
                        refresh_entry_detail(detail);
                    }
                    next_target = detail.summary.target_value.clone();
                }
                next_target
            } else {
                let mut detail = build_detail(
                    project,
                    &child_file,
                    key.to_string(),
                    parent_target_value.to_string(),
                    String::new(),
                    String::new(),
                    Some("new".into()),
                    updated_at,
                    "create",
                );
                detail.source_locale = parent_file.locale.clone();
                refresh_entry_detail(&mut detail);
                let child_target_value = detail.summary.target_value.clone();
                let entry_id = detail.summary.id.clone();
                project.entries.push(detail.summary.clone());
                project.details.insert(entry_id, detail);
                child_target_value
            };

        create_descendant_entries(project, &child_file, key, &child_target_value, updated_at);
    }
}

#[tauri::command]
pub fn create_entry(
    input: CreateEntryInput,
    state: State<'_, AppState>,
) -> Result<crate::models::ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;
    let file = project
        .files
        .iter()
        .find(|item| item.id == input.file_id)
        .cloned()
        .ok_or_else(|| format!("resource file not found: {}", input.file_id))?;

    if entry_exists(project, &input.file_id, &input.key) {
        return Err(format!("entry key already exists in file: {}", input.key));
    }

    let updated_at = Utc::now().to_rfc3339();
    let target_value = if is_root_file(&file) {
        input
            .target_value
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| input.source_value.clone().filter(|value| !value.is_empty()))
            .unwrap_or_default()
    } else {
        input.target_value.clone().unwrap_or_default()
    };
    let source_value = if is_root_file(&file) {
        String::new()
    } else {
        input.source_value.clone().unwrap_or_default()
    };
    let mut detail = build_detail(
        project,
        &file,
        input.key.clone(),
        source_value,
        target_value.clone(),
        input.note.unwrap_or_default(),
        input.status,
        &updated_at,
        "create",
    );
    refresh_entry_detail(&mut detail);
    let entry_id = detail.summary.id.clone();
    project.entries.push(detail.summary.clone());
    project.details.insert(entry_id, detail);

    create_descendant_entries(project, &file, &input.key, &target_value, &updated_at);

    project
        .entries
        .sort_by(|a, b| a.key.cmp(&b.key).then(a.file_id.cmp(&b.file_id)));
    project.dirty = true;
    refresh_project(project);
    Ok(project.clone())
}

#[tauri::command]
pub fn delete_entries(
    input: DeleteEntriesInput,
    state: State<'_, AppState>,
) -> Result<crate::models::ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    project
        .entries
        .retain(|entry| !input.entry_ids.contains(&entry.id));
    for entry_id in &input.entry_ids {
        project.details.remove(entry_id);
    }
    project.dirty = true;
    refresh_project(project);
    Ok(project.clone())
}

#[tauri::command]
pub fn delete_entry(
    input: DeleteEntryInput,
    state: State<'_, AppState>,
) -> Result<crate::models::ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    project.entries.retain(|entry| entry.id != input.entry_id);
    project.details.remove(&input.entry_id);
    project.dirty = true;
    refresh_project(project);
    Ok(project.clone())
}

#[tauri::command]
pub fn update_entry(
    input: UpdateEntryInput,
    state: State<'_, AppState>,
) -> Result<crate::models::ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    let (file_id, key) = {
        let detail = project
            .details
            .get(&input.entry_id)
            .ok_or_else(|| format!("entry not found: {}", input.entry_id))?;
        (detail.summary.file_id.clone(), detail.summary.key.clone())
    };

    let file = project
        .files
        .iter()
        .find(|item| item.id == file_id)
        .cloned()
        .ok_or_else(|| format!("resource file not found: {}", file_id))?;

    let updated_at = Utc::now().to_rfc3339();
    let new_target_value = {
        let detail = project
            .details
            .get_mut(&input.entry_id)
            .ok_or_else(|| format!("entry not found: {}", input.entry_id))?;

        let previous_target = detail.summary.target_value.clone();

        if let Some(target_value) = input.target_value.clone() {
            detail.summary.target_value = target_value;
        }
        if let Some(note) = input.note.clone() {
            detail.note = note;
        }
        if let Some(status) = input.status.clone() {
            detail.summary.status = status;
        } else {
            detail.summary.status = if detail.summary.target_value.is_empty() {
                "new".into()
            } else {
                "translated".into()
            };
        }

        if is_root_file(&file) {
            detail.summary.source_value.clear();
            detail.source_locale.clear();
        }

        detail.summary.updated_at = updated_at.clone();

        if !previous_target.is_empty() && previous_target != detail.summary.target_value {
            upsert_history_candidate(detail, &previous_target);
        }

        detail.history.insert(
            0,
            HistoryEvent {
                id: Uuid::new_v4().to_string(),
                action: "edit".into(),
                before_value: previous_target,
                after_value: detail.summary.target_value.clone(),
                operator: "desktop-user".into(),
                created_at: detail.summary.updated_at.clone(),
            },
        );

        refresh_entry_detail(detail);
        detail.summary.target_value.clone()
    };

    sync_summary_from_detail(project, &input.entry_id);
    create_descendant_entries(project, &file, &key, &new_target_value, &updated_at);
    project
        .entries
        .sort_by(|a, b| a.key.cmp(&b.key).then(a.file_id.cmp(&b.file_id)));
    project.dirty = true;
    refresh_project(project);

    Ok(project.clone())
}

#[tauri::command]
pub fn bulk_replace(
    input: BulkReplaceInput,
    state: State<'_, AppState>,
) -> Result<BulkReplaceResult, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    let regex = if input.use_regex {
        Some(Regex::new(&input.search).map_err(|error| format!("invalid regex: {error}"))?)
    } else {
        None
    };

    let mut changed_entry_ids = Vec::new();

    for detail in project.details.values_mut() {
        let original_target = detail.summary.target_value.clone();
        let original_source = detail.summary.source_value.clone();

        let next_target = match &regex {
            Some(regex) => regex
                .replace_all(&original_target, input.replacement.as_str())
                .into_owned(),
            None => original_target.replace(&input.search, &input.replacement),
        };

        let next_source = if input.target_scope == "sourceAndTarget" {
            match &regex {
                Some(regex) => regex
                    .replace_all(&original_source, input.replacement.as_str())
                    .into_owned(),
                None => original_source.replace(&input.search, &input.replacement),
            }
        } else {
            original_source.clone()
        };

        if next_target != original_target || next_source != original_source {
            if !original_target.is_empty() && original_target != next_target {
                upsert_history_candidate(detail, &original_target);
            }

            detail.summary.source_value = next_source;
            detail.summary.target_value = next_target.clone();
            detail.summary.updated_at = Utc::now().to_rfc3339();
            detail.summary.status = if next_target.is_empty() {
                "new".into()
            } else {
                "translated".into()
            };
            detail.history.insert(
                0,
                HistoryEvent {
                    id: Uuid::new_v4().to_string(),
                    action: if input.use_regex {
                        "bulk_edit_regex".into()
                    } else {
                        "bulk_edit".into()
                    },
                    before_value: original_target,
                    after_value: next_target,
                    operator: "bulk-replace".into(),
                    created_at: detail.summary.updated_at.clone(),
                },
            );
            refresh_entry_detail(detail);
            changed_entry_ids.push(detail.summary.id.clone());
        }
    }

    for changed_id in &changed_entry_ids {
        sync_summary_from_detail(project, changed_id);
    }

    project.dirty = true;
    refresh_project(project);

    Ok(BulkReplaceResult {
        changed_entry_ids,
        project: project.clone(),
    })
}
