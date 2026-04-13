use crate::editing::{refresh_entry_detail, refresh_project, upsert_history_candidate};
use crate::models::{
    CommitImportInput, EntryDetail, EntrySummary, HistoryEvent, ImportConflict,
    ImportPreviewResponse, ImportPreviewState, ImportPreviewTotals, PreviewImportInput,
    ProjectWorkspace, ResourceFileNode,
};
use crate::parsers;
use crate::project_format::storage::{save_workspace, write_raw_file};
use crate::state::AppState;
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn preview_import(
    input: PreviewImportInput,
    state: State<'_, AppState>,
) -> Result<ImportPreviewResponse, String> {
    let projects = state.projects.read();
    let project = projects
        .get(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    let mut files = parsers::import_files(&input.files)?;
    let mut conflicts = Vec::new();
    let mut preview_entries = Vec::new();

    for imported in files.iter_mut() {
        let mut file_conflicts = 0usize;
        let existing_file = project.files.iter().find(|file| {
            file.locale == imported.item.locale && file.logical_path == imported.item.logical_path
        });

        if existing_file.is_some() {
            conflicts.push(ImportConflict {
                kind: "file".into(),
                logical_path: imported.item.logical_path.clone(),
                locale: imported.item.locale.clone(),
                key: None,
                existing_value: None,
                incoming_value: None,
                message: "A file with the same logical path and locale already exists. Importing will override matching entries.".into(),
            });
            file_conflicts += 1;
        }

        for entry in &imported.imported_entries {
            if let Some(existing_entry) = project.entries.iter().find(|candidate| {
                candidate.key == entry.key
                    && existing_file
                        .as_ref()
                        .map(|file| candidate.file_id == file.id)
                        .unwrap_or(false)
            }) {
                conflicts.push(ImportConflict {
                    kind: "entry".into(),
                    logical_path: imported.item.logical_path.clone(),
                    locale: imported.item.locale.clone(),
                    key: Some(entry.key.clone()),
                    existing_value: Some(existing_entry.target_value.clone()),
                    incoming_value: Some(entry.value.clone()),
                    message: "This key already exists and will be overwritten.".into(),
                });
                file_conflicts += 1;
            }

            preview_entries.push(EntrySummary {
                id: format!("preview:{}:{}", imported.item.preview_file_id, entry.key),
                file_id: imported.item.preview_file_id.clone(),
                key: entry.key.clone(),
                source_value: entry
                    .source_value
                    .clone()
                    .or_else(|| {
                        upstream_value(
                            project,
                            &imported.item.based_on_locale,
                            &imported.item.logical_path,
                            &entry.key,
                        )
                    })
                    .unwrap_or_else(|| entry.value.clone()),
                target_value: entry.value.clone(),
                status: if entry.value.is_empty() {
                    "new".into()
                } else {
                    "translated".into()
                },
                note_count: usize::from(
                    entry
                        .note
                        .as_ref()
                        .map(|item| !item.trim().is_empty())
                        .unwrap_or(false),
                ),
                candidate_count: 0,
                updated_at: Utc::now().to_rfc3339(),
            });
        }

        imported.item.conflict_count = file_conflicts;
    }

    let response = ImportPreviewResponse {
        preview_id: Uuid::new_v4().to_string(),
        items: files.iter().map(|file| file.item.clone()).collect(),
        entries: preview_entries.clone(),
        conflicts: conflicts.clone(),
        totals: ImportPreviewTotals {
            files: files.len(),
            entries: preview_entries.len(),
            conflicts: conflicts.len(),
        },
    };

    state.import_previews.write().insert(
        response.preview_id.clone(),
        ImportPreviewState {
            response: response.clone(),
            files,
        },
    );

    Ok(response)
}

#[tauri::command]
pub fn commit_import(
    input: CommitImportInput,
    state: State<'_, AppState>,
) -> Result<ProjectWorkspace, String> {
    let preview = {
        let previews = state.import_previews.read();
        previews
            .get(&input.preview_id)
            .cloned()
            .ok_or_else(|| format!("import preview not found: {}", input.preview_id))?
    };

    let mut projects = state.projects.write();
    let project = projects
        .get_mut(&input.project_id)
        .ok_or_else(|| format!("project not found: {}", input.project_id))?;

    let workspace_dir = project
        .workspace_dir
        .clone()
        .ok_or_else(|| "project workspace directory is not available".to_string())?;
    let workspace_dir = std::path::PathBuf::from(workspace_dir);

    for imported_file in preview.files {
        let file_name = imported_file.item.name.clone();
        let raw_relative_path = write_raw_file(
            &workspace_dir,
            &imported_file.item.locale,
            &file_name,
            &imported_file.raw_bytes,
        )?;

        let file_id = if let Some(existing_file) = project.files.iter_mut().find(|file| {
            file.locale == imported_file.item.locale
                && file.logical_path == imported_file.item.logical_path
        }) {
            existing_file.name = imported_file.item.name.clone();
            existing_file.format = imported_file.item.format.clone();
            existing_file.based_on_locale = imported_file.item.based_on_locale.clone();
            existing_file.raw_relative_path = Some(raw_relative_path.clone());
            existing_file.id.clone()
        } else {
            let node = ResourceFileNode {
                id: Uuid::new_v4().to_string(),
                name: imported_file.item.name.clone(),
                logical_path: imported_file.item.logical_path.clone(),
                format: imported_file.item.format.clone(),
                locale: imported_file.item.locale.clone(),
                based_on_locale: imported_file.item.based_on_locale.clone(),
                raw_relative_path: Some(raw_relative_path.clone()),
            };
            let id = node.id.clone();
            project.files.push(node);
            id
        };

        for imported_entry in imported_file.imported_entries {
            let source_value = imported_entry
                .source_value
                .clone()
                .or_else(|| {
                    upstream_value(
                        project,
                        &imported_file.item.based_on_locale,
                        &imported_file.item.logical_path,
                        &imported_entry.key,
                    )
                })
                .unwrap_or_else(|| imported_entry.value.clone());

            let fallback_source_locale = imported_file
                .item
                .based_on_locale
                .clone()
                .unwrap_or_else(|| project.primary_locale.clone());

            if let Some(existing_index) = project
                .entries
                .iter()
                .position(|entry| entry.file_id == file_id && entry.key == imported_entry.key)
            {
                let entry_id = project.entries[existing_index].id.clone();
                {
                    let detail = project
                        .details
                        .get_mut(&entry_id)
                        .ok_or_else(|| format!("entry detail not found for {}", entry_id))?;
                    let previous = detail.summary.target_value.clone();
                    if !previous.is_empty() && previous != imported_entry.value {
                        upsert_history_candidate(detail, &previous);
                    }
                    detail.summary.source_value = source_value.clone();
                    detail.summary.target_value = imported_entry.value.clone();
                    detail.summary.status = if imported_entry.value.is_empty() {
                        "new".into()
                    } else {
                        "translated".into()
                    };
                    detail.summary.updated_at = Utc::now().to_rfc3339();
                    detail.source_locale = fallback_source_locale.clone();
                    detail.target_locale = imported_file.item.locale.clone();
                    detail.file_path = imported_file.item.logical_path.clone();
                    if let Some(note) = imported_entry.note.clone() {
                        detail.note = note;
                    }
                    detail.history.insert(
                        0,
                        HistoryEvent {
                            id: Uuid::new_v4().to_string(),
                            action: "import_override".into(),
                            before_value: previous,
                            after_value: imported_entry.value.clone(),
                            operator: "import".into(),
                            created_at: detail.summary.updated_at.clone(),
                        },
                    );
                    refresh_entry_detail(detail);
                }
            } else {
                let entry_id = Uuid::new_v4().to_string();
                let updated_at = Utc::now().to_rfc3339();
                let mut detail = EntryDetail {
                    summary: EntrySummary {
                        id: entry_id.clone(),
                        file_id: file_id.clone(),
                        key: imported_entry.key.clone(),
                        source_value: source_value.clone(),
                        target_value: imported_entry.value.clone(),
                        status: if imported_entry.value.is_empty() {
                            "new".into()
                        } else {
                            "translated".into()
                        },
                        note_count: usize::from(
                            imported_entry
                                .note
                                .as_ref()
                                .map(|item| !item.trim().is_empty())
                                .unwrap_or(false),
                        ),
                        candidate_count: 0,
                        updated_at: updated_at.clone(),
                    },
                    file_path: imported_file.item.logical_path.clone(),
                    source_locale: fallback_source_locale.clone(),
                    target_locale: imported_file.item.locale.clone(),
                    note: imported_entry.note.clone().unwrap_or_default(),
                    issues: vec![],
                    candidates: vec![],
                    history: vec![HistoryEvent {
                        id: Uuid::new_v4().to_string(),
                        action: "import_override".into(),
                        before_value: String::new(),
                        after_value: imported_entry.value.clone(),
                        operator: "import".into(),
                        created_at: updated_at,
                    }],
                };
                refresh_entry_detail(&mut detail);
                project.entries.push(detail.summary.clone());
                project.details.insert(entry_id, detail);
            }
        }
    }

    project.entries.sort_by(|a, b| a.key.cmp(&b.key));
    project.dirty = true;
    refresh_project(project);
    save_workspace(project)?;
    project.dirty = false;

    state.import_previews.write().remove(&input.preview_id);

    Ok(project.clone())
}

fn upstream_value(
    project: &ProjectWorkspace,
    based_on_locale: &Option<String>,
    logical_path: &str,
    key: &str,
) -> Option<String> {
    let locale = based_on_locale.as_ref()?;
    let file_id = project
        .files
        .iter()
        .find(|file| file.locale == *locale && file.logical_path == logical_path)
        .map(|file| file.id.clone())?;
    project
        .entries
        .iter()
        .find(|entry| entry.file_id == file_id && entry.key == key)
        .map(|entry| {
            if entry.target_value.is_empty() {
                entry.source_value.clone()
            } else {
                entry.target_value.clone()
            }
        })
}
