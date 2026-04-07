use crate::models::{BulkReplaceInput, BulkReplaceResult, UpdateEntryInput};
use crate::state::{build_treemap, AppState};
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn update_entry(
    input: UpdateEntryInput,
    state: State<'_, AppState>,
) -> Result<crate::models::ProjectWorkspace, String> {
    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    let (entry_id, note_count, updated_at) = {
        let detail = project
            .details
            .get_mut(&input.entry_id)
            .ok_or_else(|| format!("entry not found: {}", input.entry_id))?;

        if let Some(target_value) = input.target_value {
            detail.summary.target_value = target_value;
        }
        if let Some(note) = input.note {
            detail.note = note;
        }
        if let Some(status) = input.status {
            detail.summary.status = status;
        }

        detail.summary.updated_at = Utc::now().to_rfc3339();
        detail.history.insert(
            0,
            crate::models::HistoryEvent {
                id: Uuid::new_v4().to_string(),
                action: "edit".into(),
                before_value: String::new(),
                after_value: detail.summary.target_value.clone(),
                operator: "desktop-user".into(),
                created_at: detail.summary.updated_at.clone(),
            },
        );

        (
            detail.summary.id.clone(),
            usize::from(!detail.note.is_empty()),
            detail.summary.updated_at.clone(),
        )
    };

    if let Some(summary) = project
        .entries
        .iter_mut()
        .find(|entry| entry.id == entry_id)
    {
        if let Some(detail) = project.details.get(&entry_id) {
            *summary = detail.summary.clone();
        }
        summary.note_count = note_count;
        summary.updated_at = updated_at;
    }

    project.dirty = true;
    refresh_project_derived_data(project);

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

    let mut changed_entry_ids = Vec::new();

    for detail in project.details.values_mut() {
        let original = detail.summary.target_value.clone();
        let next = original.replace(&input.search, &input.replacement);

        if next != original {
            detail.summary.target_value = next.clone();
            detail.summary.updated_at = Utc::now().to_rfc3339();
            detail.summary.status = if next.is_empty() {
                "new".into()
            } else {
                "translated".into()
            };
            detail.history.insert(
                0,
                crate::models::HistoryEvent {
                    id: Uuid::new_v4().to_string(),
                    action: if input.use_regex {
                        "bulk_edit_regex".into()
                    } else {
                        "bulk_edit".into()
                    },
                    before_value: original,
                    after_value: next,
                    operator: "bulk-replace".into(),
                    created_at: detail.summary.updated_at.clone(),
                },
            );
            changed_entry_ids.push(detail.summary.id.clone());
        }
    }

    for entry in project.entries.iter_mut() {
        if let Some(detail) = project.details.get(&entry.id) {
            *entry = detail.summary.clone();
        }
    }

    project.dirty = true;
    refresh_project_derived_data(project);

    Ok(BulkReplaceResult {
        changed_entry_ids,
        project: project.clone(),
    })
}

fn refresh_project_derived_data(project: &mut crate::models::ProjectWorkspace) {
    project.treemap = build_treemap(&project.entries);
    project.stats.translated = project
        .entries
        .iter()
        .filter(|entry| !entry.target_value.is_empty())
        .count();
    project.stats.missing = project
        .entries
        .iter()
        .filter(|entry| entry.target_value.is_empty())
        .count();
    project.stats.reviewed = project
        .entries
        .iter()
        .filter(|entry| entry.status == "reviewed" || entry.status == "approved")
        .count();
}
