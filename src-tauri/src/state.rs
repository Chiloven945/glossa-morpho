use crate::models::{
    BootstrapResponse, CandidateItem, EntryDetail, EntrySummary, HistoryEvent, ProjectStats,
    ProjectWorkspace, ResourceFileNode, TreemapNode, ValidationIssue,
};
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

pub struct AppState {
    pub recent_projects: RwLock<Vec<String>>,
    pub projects: RwLock<HashMap<String, ProjectWorkspace>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            recent_projects: RwLock::new(vec!["/Users/you/projects/demo.gmproj".to_string()]),
            projects: RwLock::new(HashMap::new()),
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
}

pub fn demo_project(
    name: &str,
    path: &str,
    source_locale: &str,
    target_locale: &str,
) -> ProjectWorkspace {
    let files = vec![
        ResourceFileNode {
            id: "file-json-en".into(),
            name: "app.en-US.json".into(),
            logical_path: "locale/source/app.en-US.json".into(),
            format: "json".into(),
            locale: source_locale.into(),
            role: "source".into(),
        },
        ResourceFileNode {
            id: "file-json-target".into(),
            name: format!("app.{}.json", target_locale),
            logical_path: format!("locale/target/app.{}.json", target_locale),
            format: "json".into(),
            locale: target_locale.into(),
            role: "target".into(),
        },
        ResourceFileNode {
            id: "file-yaml-en".into(),
            name: "marketing.en-US.yaml".into(),
            logical_path: "locale/source/marketing.en-US.yaml".into(),
            format: "yaml".into(),
            locale: source_locale.into(),
            role: "source".into(),
        },
        ResourceFileNode {
            id: "file-yaml-target".into(),
            name: format!("marketing.{}.yaml", target_locale),
            logical_path: format!("locale/target/marketing.{}.yaml", target_locale),
            format: "yaml".into(),
            locale: target_locale.into(),
            role: "target".into(),
        },
    ];

    let now = Utc::now().to_rfc3339();

    let mut details = HashMap::new();
    let entries_seed = vec![
        (
            "dashboard.welcomeBack",
            "Welcome back, {name}",
            "欢迎回来，{name}",
            "translated",
            "file-json-target",
            "locale/target/app.zh-CN.json",
        ),
        (
            "dashboard.emptyState.title",
            "No records found",
            "",
            "new",
            "file-json-target",
            "locale/target/app.zh-CN.json",
        ),
        (
            "marketing.heroSubtitle",
            "Ship localization projects faster",
            "更快交付本地化项目",
            "reviewed",
            "file-yaml-target",
            "locale/target/marketing.zh-CN.yaml",
        ),
        (
            "settings.autoSaveInterval",
            "Auto save interval",
            "自动保存间隔",
            "translated",
            "file-json-target",
            "locale/target/app.zh-CN.json",
        ),
        (
            "errors.networkTimeout",
            "Network request timed out",
            "",
            "new",
            "file-json-target",
            "locale/target/app.zh-CN.json",
        ),
        (
            "common.ok",
            "OK",
            "确定",
            "approved",
            "file-json-target",
            "locale/target/app.zh-CN.json",
        ),
    ];

    let mut entries = Vec::new();
    for (key, source_value, target_value, status, file_id, file_path) in entries_seed {
        let id = Uuid::new_v4().to_string();
        let summary = EntrySummary {
            id: id.clone(),
            file_id: file_id.into(),
            key: key.into(),
            source_value: source_value.into(),
            target_value: target_value.into(),
            status: status.into(),
            note_count: if target_value.is_empty() { 0 } else { 1 },
            candidate_count: 2,
            updated_at: now.clone(),
        };

        let detail = EntryDetail {
            summary: summary.clone(),
            file_path: file_path.into(),
            source_locale: source_locale.into(),
            target_locale: target_locale.into(),
            note: if target_value.is_empty() {
                String::new()
            } else {
                "Check punctuation and placeholders before review.".into()
            },
            issues: if target_value.contains("{name}") {
                vec![]
            } else {
                vec![ValidationIssue {
                    id: Uuid::new_v4().to_string(),
                    level: "warning".into(),
                    message: "Placeholder consistency not checked yet.".into(),
                }]
            },
            candidates: vec![
                CandidateItem {
                    id: Uuid::new_v4().to_string(),
                    source: "history".into(),
                    value: if target_value.is_empty() {
                        "欢迎回来，{name}".into()
                    } else {
                        target_value.into()
                    },
                    score: 0.92,
                },
                CandidateItem {
                    id: Uuid::new_v4().to_string(),
                    source: "manual".into(),
                    value: if target_value.is_empty() {
                        "你好，{name}".into()
                    } else {
                        target_value.into()
                    },
                    score: 0.61,
                },
            ],
            history: vec![HistoryEvent {
                id: Uuid::new_v4().to_string(),
                action: "edit".into(),
                before_value: String::new(),
                after_value: target_value.into(),
                operator: "system".into(),
                created_at: now.clone(),
            }],
        };

        entries.push(summary);
        details.insert(id, detail);
    }

    let treemap = build_treemap(&entries);
    let translated = entries
        .iter()
        .filter(|entry| !entry.target_value.is_empty())
        .count();
    let reviewed = entries
        .iter()
        .filter(|entry| entry.status == "reviewed" || entry.status == "approved")
        .count();

    ProjectWorkspace {
        id: Uuid::new_v4().to_string(),
        name: name.into(),
        path: path.into(),
        source_locale: source_locale.into(),
        target_locale: target_locale.into(),
        target_locales: vec![target_locale.into(), "ja-JP".into()],
        dirty: false,
        files,
        entries: entries.clone(),
        details,
        treemap,
        stats: ProjectStats {
            total: entries.len(),
            translated,
            missing: entries.len() - translated,
            reviewed,
        },
    }
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
