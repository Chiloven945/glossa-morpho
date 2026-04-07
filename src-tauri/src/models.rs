use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceFileNode {
    pub id: String,
    pub name: String,
    pub logical_path: String,
    pub format: String,
    pub locale: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateItem {
    pub id: String,
    pub source: String,
    pub value: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEvent {
    pub id: String,
    pub action: String,
    pub before_value: String,
    pub after_value: String,
    pub operator: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub id: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySummary {
    pub id: String,
    pub file_id: String,
    pub key: String,
    pub source_value: String,
    pub target_value: String,
    pub status: String,
    pub note_count: usize,
    pub candidate_count: usize,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryDetail {
    #[serde(flatten)]
    pub summary: EntrySummary,
    pub file_path: String,
    pub source_locale: String,
    pub target_locale: String,
    pub note: String,
    pub issues: Vec<ValidationIssue>,
    pub candidates: Vec<CandidateItem>,
    pub history: Vec<HistoryEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStats {
    pub total: usize,
    pub translated: usize,
    pub missing: usize,
    pub reviewed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreemapNode {
    pub id: String,
    pub label: String,
    pub path: String,
    pub count: usize,
    pub translated_count: usize,
    pub missing_count: usize,
    pub char_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub source_locale: String,
    pub target_locale: String,
    pub target_locales: Vec<String>,
    pub dirty: bool,
    pub files: Vec<ResourceFileNode>,
    pub entries: Vec<EntrySummary>,
    pub details: HashMap<String, EntryDetail>,
    pub treemap: Vec<TreemapNode>,
    pub stats: ProjectStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    pub recent_projects: Vec<String>,
    pub opened_projects: Vec<ProjectWorkspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub name: String,
    pub source_locale: String,
    pub target_locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntryInput {
    pub project_id: String,
    pub entry_id: String,
    pub target_value: Option<String>,
    pub note: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkReplaceInput {
    pub project_id: String,
    pub search: String,
    pub replacement: String,
    pub use_regex: bool,
    pub target_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkReplaceResult {
    pub changed_entry_ids: Vec<String>,
    pub project: ProjectWorkspace,
}
