use crate::models::{
    BootstrapResponse, EntrySummary, ImportPreviewState, ProjectWorkspace, TreemapNode,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct AppState {
    pub recent_projects: RwLock<Vec<String>>,
    pub projects: RwLock<HashMap<String, ProjectWorkspace>>,
    pub import_previews: RwLock<HashMap<String, ImportPreviewState>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            recent_projects: RwLock::new(load_recent_projects()),
            projects: RwLock::new(HashMap::new()),
            import_previews: RwLock::new(HashMap::new()),
        }
    }
}

impl AppState {
    pub fn bootstrap(&self) -> BootstrapResponse {
        let opened_projects = self.projects.read().values().cloned().collect::<Vec<_>>();
        BootstrapResponse {
            recent_projects: self.recent_projects.read().clone(),
            opened_projects,
        }
    }

    pub fn insert_project(&self, project: ProjectWorkspace) -> ProjectWorkspace {
        self.projects
            .write()
            .insert(project.id.clone(), project.clone());
        project
    }

    pub fn push_recent_project(&self, path: &str) {
        let mut recent_projects = self.recent_projects.write();
        recent_projects.retain(|item| item != path);
        recent_projects.insert(0, path.to_string());
        recent_projects.truncate(12);
        let _ = persist_recent_projects(&recent_projects);
    }
}

fn recent_projects_store_path() -> PathBuf {
    let base_dir = dirs::config_dir()
        .or_else(dirs::home_dir)
        .unwrap_or(std::env::temp_dir());
    base_dir.join("glossa-morpho").join("recent-projects.json")
}

fn load_recent_projects() -> Vec<String> {
    let path = recent_projects_store_path();
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };

    serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default()
}

fn persist_recent_projects(recent_projects: &[String]) -> Result<(), String> {
    let path = recent_projects_store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create config directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let payload = serde_json::to_string_pretty(recent_projects)
        .map_err(|error| format!("failed to serialize recent projects: {error}"))?;
    fs::write(&path, payload).map_err(|error| {
        format!(
            "failed to write recent projects file {}: {error}",
            path.display()
        )
    })
}

pub fn build_treemap(entries: &[EntrySummary]) -> Vec<TreemapNode> {
    let mut groups: HashMap<String, TreemapNode> = HashMap::new();

    for entry in entries {
        let label = entry
            .key
            .split('.')
            .next()
            .map(|item| item.to_string())
            .unwrap_or_else(|| "root".into());

        let group = groups.entry(label.clone()).or_insert(TreemapNode {
            id: label.clone(),
            label: label.clone(),
            path: label.clone(),
            count: 0,
            translated_count: 0,
            missing_count: 0,
            char_count: 0,
        });

        group.count += 1;
        group.char_count += entry.source_value.len() + entry.target_value.len();
        if entry.target_value.is_empty() {
            group.missing_count += 1;
        } else {
            group.translated_count += 1;
        }
    }

    let mut values = groups.into_values().collect::<Vec<_>>();
    values.sort_by(|a, b| b.count.cmp(&a.count));
    values
}
