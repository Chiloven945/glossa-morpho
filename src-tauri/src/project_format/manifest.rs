use crate::models::LocaleDependencyNode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const PROJECT_SCHEMA_VERSION: i64 = 1;
pub const SQLITE_RELATIVE_PATH: &str = "data/project.sqlite";
pub const MANIFEST_FILE_NAME: &str = "manifest.json";
pub const ARCHIVE_FORMAT_LZMA2: &str = "lzma2";
pub const ARCHIVE_FORMAT_DEFLATE: &str = "deflate";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub schema_version: i64,
    pub project_id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub primary_locale: String,
    pub working_locale: String,
    pub locale_graph: Vec<LocaleDependencyNode>,
    pub key_segmentation_profiles: Vec<String>,
    pub default_view: String,
    pub default_sort: String,
    pub sqlite_path: String,
    pub archive_format: String,
}

impl ProjectManifest {
    pub fn new(
        project_id: String,
        name: String,
        primary_locale: String,
        working_locale: String,
        locale_graph: Vec<LocaleDependencyNode>,
        archive_format: String,
        key_segmentation_profiles: Vec<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            schema_version: PROJECT_SCHEMA_VERSION,
            project_id,
            name,
            created_at: now.clone(),
            updated_at: now,
            primary_locale,
            working_locale,
            locale_graph,
            key_segmentation_profiles,
            default_view: "list".into(),
            default_sort: "updatedDesc".into(),
            sqlite_path: SQLITE_RELATIVE_PATH.into(),
            archive_format: normalize_archive_format(&archive_format),
        }
    }
}

pub fn manifest_path(project_dir: &Path) -> PathBuf {
    project_dir.join(MANIFEST_FILE_NAME)
}

pub fn read_manifest(project_dir: &Path) -> Result<ProjectManifest, String> {
    let path = manifest_path(project_dir);
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read manifest at {}: {error}", path.display()))?;
    let mut manifest: ProjectManifest = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse manifest at {}: {error}", path.display()))?;

    if manifest.schema_version != PROJECT_SCHEMA_VERSION {
        return Err(format!(
            "unsupported project schema version: {} (expected {})",
            manifest.schema_version, PROJECT_SCHEMA_VERSION
        ));
    }

    manifest.archive_format = normalize_archive_format(&manifest.archive_format);
    Ok(manifest)
}

pub fn write_manifest(project_dir: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    let path = manifest_path(project_dir);
    let payload = serde_json::to_string_pretty(manifest)
        .map_err(|error| format!("failed to serialize project manifest: {error}"))?;
    fs::write(&path, payload)
        .map_err(|error| format!("failed to write manifest at {}: {error}", path.display()))
}

pub fn resolve_project_path(raw_path: &str) -> Result<PathBuf, String> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Err("project path cannot be empty".into());
    }
    Ok(PathBuf::from(trimmed))
}

pub fn normalize_new_project_path(
    raw_path: Option<&str>,
    project_name: &str,
) -> Result<PathBuf, String> {
    if let Some(raw_path) = raw_path {
        let trimmed = raw_path.trim();
        if !trimmed.is_empty() {
            let mut path = PathBuf::from(trimmed);
            let display = path.to_string_lossy().to_string();
            if !display.to_ascii_lowercase().ends_with(".gmproj") {
                path = PathBuf::from(format!("{}.gmproj", display.trim_end_matches(['/', '\\'])));
            }
            return Ok(path);
        }
    }

    let base_dir = dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or(std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir()));
    let default_dir = base_dir.join("glossa-morpho-projects");
    let file_name = format!("{}.gmproj", sanitize_project_name(project_name));
    Ok(default_dir.join(file_name))
}

pub fn normalize_archive_format(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        ARCHIVE_FORMAT_DEFLATE => ARCHIVE_FORMAT_DEFLATE.into(),
        _ => ARCHIVE_FORMAT_LZMA2.into(),
    }
}

fn sanitize_project_name(project_name: &str) -> String {
    let sanitized = project_name
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect::<String>();

    let compact = sanitized
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if compact.is_empty() {
        "glossa-project".into()
    } else {
        compact
    }
}
