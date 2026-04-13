use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaleDependencyNode {
    pub code: String,
    pub label: String,
    pub parent_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceFileNode {
    pub id: String,
    pub name: String,
    pub logical_path: String,
    pub format: String,
    pub locale: String,
    pub based_on_locale: Option<String>,
    pub raw_relative_path: Option<String>,
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
    pub workspace_dir: Option<String>,
    pub locale_graph: Vec<LocaleDependencyNode>,
    pub primary_locale: String,
    pub working_locale: String,
    pub archive_format: String,
    pub key_segmentation_profiles: Vec<String>,
    pub default_view: String,
    pub default_sort: String,
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
    pub path: Option<String>,
    pub locale_graph: Vec<LocaleDependencyNode>,
    pub primary_locale: String,
    pub working_locale: String,
    pub archive_format: Option<String>,
    pub key_segmentation_profiles: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectMetadataInput {
    pub project_id: String,
    pub name: String,
    pub primary_locale: String,
    pub working_locale: String,
    pub archive_format: String,
    pub key_segmentation_profiles: Vec<String>,
    pub default_view: String,
    pub default_sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResourceFileInput {
    pub project_id: String,
    pub name: String,
    pub logical_path: String,
    pub format: String,
    pub locale: String,
    pub based_on_locale: Option<String>,
    pub include_descendants: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameResourceFileInput {
    pub project_id: String,
    pub file_id: String,
    pub name: String,
    pub logical_path: String,
    pub include_related: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResourceFileInput {
    pub project_id: String,
    pub file_id: String,
    pub include_related: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntryInput {
    pub project_id: String,
    pub file_id: String,
    pub key: String,
    pub source_value: Option<String>,
    pub target_value: Option<String>,
    pub note: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteEntryInput {
    pub project_id: String,
    pub entry_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteEntriesInput {
    pub project_id: String,
    pub entry_ids: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFileInput {
    pub path: String,
    pub locale: String,
    pub based_on_locale: Option<String>,
    pub logical_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewImportInput {
    pub project_id: String,
    pub files: Vec<ImportFileInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitImportInput {
    pub project_id: String,
    pub preview_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewItem {
    pub preview_file_id: String,
    pub path: String,
    pub logical_path: String,
    pub name: String,
    pub format: String,
    pub locale: String,
    pub based_on_locale: Option<String>,
    pub entry_count: usize,
    pub conflict_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportConflict {
    pub kind: String,
    pub logical_path: String,
    pub locale: String,
    pub key: Option<String>,
    pub existing_value: Option<String>,
    pub incoming_value: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewResponse {
    pub preview_id: String,
    pub items: Vec<ImportPreviewItem>,
    pub entries: Vec<EntrySummary>,
    pub conflicts: Vec<ImportConflict>,
    pub totals: ImportPreviewTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewTotals {
    pub files: usize,
    pub entries: usize,
    pub conflicts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectInput {
    pub project_id: String,
    pub file_id: Option<String>,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchExportInput {
    pub project_id: String,
    pub file_ids: Vec<String>,
    pub output_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectResult {
    pub project_id: String,
    pub output_path: String,
    pub exported_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ImportedFilePayload {
    pub item: ImportPreviewItem,
    pub raw_bytes: Vec<u8>,
    pub imported_entries: Vec<ImportedEntry>,
}

#[derive(Debug, Clone)]
pub struct ImportedEntry {
    pub key: String,
    pub value: String,
    pub source_value: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportPreviewState {
    pub response: ImportPreviewResponse,
    pub files: Vec<ImportedFilePayload>,
}
