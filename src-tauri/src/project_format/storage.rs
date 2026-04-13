use crate::models::{
    BatchExportInput, CandidateItem, EntryDetail, EntrySummary, ExportProjectInput,
    ExportProjectResult, HistoryEvent, LocaleDependencyNode, ProjectStats, ProjectWorkspace,
    ResourceFileNode, ValidationIssue,
};
use crate::state::build_treemap;
use chrono::Utc;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use uuid::Uuid;
use walkdir::WalkDir;
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

use super::manifest::{
    normalize_archive_format, normalize_new_project_path, read_manifest, resolve_project_path,
    write_manifest, ProjectManifest, ARCHIVE_FORMAT_DEFLATE, SQLITE_RELATIVE_PATH,
};
use super::migration::apply_schema;

pub struct SaveProjectSummary {
    pub saved_at: String,
}

pub fn create_project_file_path(raw_path: Option<&str>, name: &str) -> Result<PathBuf, String> {
    let archive_path = normalize_new_project_path(raw_path, name)?;
    if archive_path.exists() && archive_path.is_dir() {
        return Err(format!(
            "project path points to a directory, but .gmproj archive file is expected: {}",
            archive_path.display()
        ));
    }

    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create parent directory {}: {error}",
                parent.display()
            )
        })?;
    }

    Ok(archive_path)
}

pub fn scaffold_workspace(
    name: &str,
    path: &str,
    locale_graph: Vec<LocaleDependencyNode>,
    primary_locale: String,
    working_locale: String,
    archive_format: String,
    key_segmentation_profiles: Vec<String>,
) -> ProjectWorkspace {
    let project_id = Uuid::new_v4().to_string();
    let workspace_dir = allocate_workspace_dir(&project_id)
        .to_string_lossy()
        .to_string();

    ProjectWorkspace {
        id: project_id,
        name: name.to_string(),
        path: path.to_string(),
        workspace_dir: Some(workspace_dir),
        locale_graph,
        primary_locale,
        working_locale,
        archive_format: normalize_archive_format(&archive_format),
        key_segmentation_profiles: if key_segmentation_profiles.is_empty() {
            vec!["dot".into(), "camel".into()]
        } else {
            key_segmentation_profiles
        },
        default_view: "list".into(),
        default_sort: "updatedDesc".into(),
        dirty: false,
        files: vec![],
        entries: vec![],
        details: HashMap::new(),
        treemap: vec![],
        stats: ProjectStats {
            total: 0,
            translated: 0,
            missing: 0,
            reviewed: 0,
        },
    }
}

pub fn load_workspace(project_path: &str) -> Result<ProjectWorkspace, String> {
    let resolved = resolve_project_path(project_path)?;
    let workspace_dir = if resolved.is_dir() {
        resolved.clone()
    } else {
        let temp_dir = allocate_workspace_dir(&Uuid::new_v4().to_string());
        extract_archive_to_workspace(&resolved, &temp_dir)?;
        temp_dir
    };

    let manifest = read_manifest(&workspace_dir)?;
    let connection = open_connection(&workspace_dir)?;

    let locale_graph = load_locale_graph(&connection)?;
    let files = load_files(&connection)?;
    let (entries, details) = load_entries(&connection)?;
    let stats = build_stats(&entries);
    let treemap = build_treemap(&entries);

    Ok(ProjectWorkspace {
        id: manifest.project_id,
        name: manifest.name,
        path: project_path.to_string(),
        workspace_dir: Some(workspace_dir.to_string_lossy().to_string()),
        locale_graph: if locale_graph.is_empty() {
            manifest.locale_graph
        } else {
            locale_graph
        },
        primary_locale: manifest.primary_locale,
        working_locale: manifest.working_locale,
        archive_format: normalize_archive_format(&manifest.archive_format),
        key_segmentation_profiles: manifest.key_segmentation_profiles,
        default_view: manifest.default_view,
        default_sort: manifest.default_sort,
        dirty: false,
        files,
        entries,
        details,
        treemap,
        stats,
    })
}

pub fn save_workspace(project: &ProjectWorkspace) -> Result<SaveProjectSummary, String> {
    let workspace_dir = ensure_workspace_dir(project)?;
    prepare_workspace_dirs(&workspace_dir)?;

    let saved_at = Utc::now().to_rfc3339();
    let mut manifest = read_manifest(&workspace_dir).unwrap_or_else(|_| {
        ProjectManifest::new(
            project.id.clone(),
            project.name.clone(),
            project.primary_locale.clone(),
            project.working_locale.clone(),
            project.locale_graph.clone(),
            project.archive_format.clone(),
            project.key_segmentation_profiles.clone(),
        )
    });
    manifest.project_id = project.id.clone();
    manifest.name = project.name.clone();
    manifest.primary_locale = project.primary_locale.clone();
    manifest.working_locale = project.working_locale.clone();
    manifest.locale_graph = project.locale_graph.clone();
    manifest.key_segmentation_profiles = project.key_segmentation_profiles.clone();
    manifest.default_view = project.default_view.clone();
    manifest.default_sort = project.default_sort.clone();
    manifest.updated_at = saved_at.clone();
    manifest.sqlite_path = SQLITE_RELATIVE_PATH.into();
    manifest.archive_format = normalize_archive_format(&project.archive_format);

    let mut connection = open_connection(&workspace_dir)?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start save transaction: {error}"))?;

    apply_schema(&transaction)?;
    clear_workspace_tables(&transaction)?;
    persist_project_meta(&transaction, project, &saved_at)?;
    persist_locale_graph(&transaction, &project.locale_graph)?;
    persist_files(&transaction, &project.files)?;
    persist_entries(&transaction, project)?;

    transaction
        .commit()
        .map_err(|error| format!("failed to commit project save: {error}"))?;

    write_manifest(&workspace_dir, &manifest)?;

    let archive_path = PathBuf::from(&project.path);
    if !archive_path.as_os_str().is_empty() {
        archive_workspace(&workspace_dir, &archive_path, &project.archive_format)?;
    }

    Ok(SaveProjectSummary { saved_at })
}

pub fn write_raw_file(
    workspace_dir: &Path,
    locale: &str,
    file_name: &str,
    raw_bytes: &[u8],
) -> Result<String, String> {
    let file_path = workspace_dir
        .join("raw")
        .join("locales")
        .join(locale)
        .join(file_name);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create raw locale directory {}: {error}",
                parent.display()
            )
        })?;
    }

    fs::write(&file_path, raw_bytes).map_err(|error| {
        format!(
            "failed to write raw locale file {}: {error}",
            file_path.display()
        )
    })?;

    let relative = file_path
        .strip_prefix(workspace_dir)
        .unwrap_or(&file_path)
        .to_string_lossy()
        .replace('\\', "/");
    Ok(relative)
}

fn render_export_content(
    project: &ProjectWorkspace,
    file: &ResourceFileNode,
) -> Result<Vec<u8>, String> {
    let entries = project
        .entries
        .iter()
        .filter(|entry| entry.file_id == file.id)
        .cloned()
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Err(format!(
            "resource file '{}' has no entries to export",
            file.name
        ));
    }

    let workspace_dir = project.workspace_dir.as_ref().map(PathBuf::from);
    match file.format.as_str() {
        "json" => crate::exporters::json::export(&entries),
        "yaml" => crate::exporters::yaml::export(&entries),
        "properties" => crate::exporters::properties::export(&entries),
        "resx" => crate::exporters::resx::export(
            file,
            &entries,
            &project.details,
            workspace_dir.as_deref(),
        ),
        "xaml" => crate::exporters::xaml::export(
            file,
            &entries,
            &project.details,
            workspace_dir.as_deref(),
        ),
        "xliff" => crate::exporters::xliff::export(
            file,
            &entries,
            &project.details,
            workspace_dir.as_deref(),
        ),
        other => Err(format!("export not implemented for format '{other}'")),
    }
}

pub fn export_project_files(
    project: &ProjectWorkspace,
    input: ExportProjectInput,
) -> Result<ExportProjectResult, String> {
    let file = if let Some(file_id) = input.file_id.as_deref() {
        project
            .files
            .iter()
            .find(|item| item.id == file_id)
            .cloned()
            .ok_or_else(|| format!("resource file not found: {file_id}"))?
    } else {
        project
            .files
            .first()
            .cloned()
            .ok_or_else(|| "project has no exportable files".to_string())?
    };

    let output_path = input
        .output_path
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&file.logical_path));

    if let Some(parent) = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create export parent directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let content = render_export_content(project, &file)?;
    fs::write(&output_path, content).map_err(|error| {
        format!(
            "failed to write export file {}: {error}",
            output_path.display()
        )
    })?;

    Ok(ExportProjectResult {
        project_id: project.id.clone(),
        output_path: output_path.to_string_lossy().to_string(),
        exported_files: vec![output_path.to_string_lossy().to_string()],
    })
}

pub fn export_project_batch_files(
    project: &ProjectWorkspace,
    input: BatchExportInput,
) -> Result<ExportProjectResult, String> {
    let output_directory = input
        .output_directory
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "output directory is required for batch export".to_string())?;

    fs::create_dir_all(&output_directory).map_err(|error| {
        format!(
            "failed to create batch export directory {}: {error}",
            output_directory.display()
        )
    })?;

    let file_ids = if input.file_ids.is_empty() {
        project
            .files
            .iter()
            .map(|file| file.id.clone())
            .collect::<Vec<_>>()
    } else {
        input.file_ids
    };

    let mut exported_files = Vec::new();
    for file_id in file_ids {
        let file = project
            .files
            .iter()
            .find(|item| item.id == file_id)
            .cloned()
            .ok_or_else(|| format!("resource file not found: {file_id}"))?;
        let output_path = output_directory.join(&file.logical_path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create export directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        let content = render_export_content(project, &file)?;
        fs::write(&output_path, content).map_err(|error| {
            format!(
                "failed to write export file {}: {error}",
                output_path.display()
            )
        })?;
        exported_files.push(output_path.to_string_lossy().to_string());
    }

    Ok(ExportProjectResult {
        project_id: project.id.clone(),
        output_path: output_directory.to_string_lossy().to_string(),
        exported_files,
    })
}

fn ensure_workspace_dir(project: &ProjectWorkspace) -> Result<PathBuf, String> {
    if let Some(workspace_dir) = &project.workspace_dir {
        let path = PathBuf::from(workspace_dir);
        if !path.exists() {
            prepare_workspace_dirs(&path)?;
        }
        return Ok(path);
    }

    let path = allocate_workspace_dir(&project.id);
    prepare_workspace_dirs(&path)?;
    Ok(path)
}

fn workspace_root() -> PathBuf {
    let base_dir = dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .or_else(dirs::config_dir)
        .or_else(dirs::home_dir)
        .unwrap_or(std::env::temp_dir());
    base_dir.join("glossa-morpho").join("workspaces")
}

fn allocate_workspace_dir(project_id: &str) -> PathBuf {
    workspace_root().join(project_id)
}

fn prepare_workspace_dirs(workspace_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(workspace_dir).map_err(|error| {
        format!(
            "failed to create workspace directory {}: {error}",
            workspace_dir.display()
        )
    })?;
    for relative in ["data", "raw/locales", "exports", "snapshots", "cache"] {
        let dir = workspace_dir.join(relative);
        fs::create_dir_all(&dir).map_err(|error| {
            format!(
                "failed to create workspace directory {}: {error}",
                dir.display()
            )
        })?;
    }
    Ok(())
}

fn archive_workspace(
    workspace_dir: &Path,
    archive_path: &Path,
    archive_format: &str,
) -> Result<(), String> {
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create archive parent directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let file = fs::File::create(archive_path).map_err(|error| {
        format!(
            "failed to create project archive {}: {error}",
            archive_path.display()
        )
    })?;

    match normalize_archive_format(archive_format).as_str() {
        ARCHIVE_FORMAT_DEFLATE => {
            let encoder = ZlibEncoder::new(file, Compression::default());
            let encoder = write_tar_archive(workspace_dir, encoder)?;
            encoder.finish().map_err(|error| {
                format!(
                    "failed to finalize deflate archive {}: {error}",
                    archive_path.display()
                )
            })?;
        }
        _ => {
            let encoder = XzEncoder::new(file, 6);
            let encoder = write_tar_archive(workspace_dir, encoder)?;
            encoder.finish().map_err(|error| {
                format!(
                    "failed to finalize LZMA2 archive {}: {error}",
                    archive_path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn write_tar_archive<W: Write>(workspace_dir: &Path, writer: W) -> Result<W, String> {
    let mut builder = Builder::new(writer);
    for entry in WalkDir::new(workspace_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == workspace_dir {
            continue;
        }
        let relative = path.strip_prefix(workspace_dir).unwrap_or(path);
        if entry.file_type().is_dir() {
            builder.append_dir(relative, path).map_err(|error| {
                format!("failed to archive directory {}: {error}", path.display())
            })?;
        } else if entry.file_type().is_file() {
            builder
                .append_path_with_name(path, relative)
                .map_err(|error| format!("failed to archive file {}: {error}", path.display()))?;
        }
    }
    builder
        .into_inner()
        .map_err(|error| format!("failed to finalize tar builder: {error}"))
}

fn extract_archive_to_workspace(archive_path: &Path, workspace_dir: &Path) -> Result<(), String> {
    if workspace_dir.exists() {
        fs::remove_dir_all(workspace_dir).map_err(|error| {
            format!(
                "failed to reset workspace directory {}: {error}",
                workspace_dir.display()
            )
        })?;
    }
    fs::create_dir_all(workspace_dir).map_err(|error| {
        format!(
            "failed to create workspace directory {}: {error}",
            workspace_dir.display()
        )
    })?;

    let raw = fs::read(archive_path).map_err(|error| {
        format!(
            "failed to read project archive {}: {error}",
            archive_path.display()
        )
    })?;

    if raw.starts_with(&[0xFD, b'7', b'z', b'X', b'Z', 0x00]) {
        let decoder = XzDecoder::new(std::io::Cursor::new(raw));
        let mut archive = Archive::new(decoder);
        archive.unpack(workspace_dir).map_err(|error| {
            format!(
                "failed to unpack LZMA2 project archive {}: {error}",
                archive_path.display()
            )
        })?;
    } else {
        let decoder = ZlibDecoder::new(std::io::Cursor::new(raw));
        let mut archive = Archive::new(decoder);
        archive.unpack(workspace_dir).map_err(|error| {
            format!(
                "failed to unpack deflate project archive {}: {error}",
                archive_path.display()
            )
        })?;
    }

    Ok(())
}

fn open_connection(workspace_dir: &Path) -> Result<Connection, String> {
    let sqlite_path = workspace_dir.join(SQLITE_RELATIVE_PATH);
    if let Some(parent) = sqlite_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create data directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let connection = Connection::open(&sqlite_path).map_err(|error| {
        format!(
            "failed to open SQLite database {}: {error}",
            sqlite_path.display()
        )
    })?;
    apply_schema(&connection)?;
    Ok(connection)
}

fn clear_workspace_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            r#"
            DELETE FROM project_meta;
            DELETE FROM locale_graph;
            DELETE FROM resource_files;
            DELETE FROM entries;
            DELETE FROM entry_issues;
            DELETE FROM entry_candidates;
            DELETE FROM entry_history;
            "#,
        )
        .map_err(|error| format!("failed to clear existing workspace rows: {error}"))
}

fn persist_project_meta(
    connection: &Connection,
    project: &ProjectWorkspace,
    saved_at: &str,
) -> Result<(), String> {
    let meta_pairs = [
        ("project_id", project.id.as_str()),
        ("name", project.name.as_str()),
        ("path", project.path.as_str()),
        ("primary_locale", project.primary_locale.as_str()),
        ("working_locale", project.working_locale.as_str()),
        ("archive_format", project.archive_format.as_str()),
        ("saved_at", saved_at),
    ];

    for (key, value) in meta_pairs {
        connection
            .execute(
                "INSERT INTO project_meta(key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|error| format!("failed to persist project metadata '{key}': {error}"))?;
    }

    Ok(())
}

fn persist_locale_graph(
    connection: &Connection,
    locale_graph: &[LocaleDependencyNode],
) -> Result<(), String> {
    for node in locale_graph {
        connection
            .execute(
                r#"
                INSERT INTO locale_graph(code, label, parent_code)
                VALUES (?1, ?2, ?3)
                "#,
                params![node.code, node.label, node.parent_code],
            )
            .map_err(|error| {
                format!(
                    "failed to persist locale graph node '{}': {error}",
                    node.code
                )
            })?;
    }
    Ok(())
}

fn persist_files(connection: &Connection, files: &[ResourceFileNode]) -> Result<(), String> {
    for (index, file) in files.iter().enumerate() {
        connection
            .execute(
                r#"
                INSERT INTO resource_files(id, sort_order, name, logical_path, format, locale, based_on_locale, raw_relative_path)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    file.id,
                    index as i64,
                    file.name,
                    file.logical_path,
                    file.format,
                    file.locale,
                    file.based_on_locale,
                    file.raw_relative_path,
                ],
            )
            .map_err(|error| format!("failed to persist resource file '{}': {error}", file.id))?;
    }
    Ok(())
}

fn persist_entries(connection: &Connection, project: &ProjectWorkspace) -> Result<(), String> {
    for (index, summary) in project.entries.iter().enumerate() {
        let fallback_detail = EntryDetail {
            summary: summary.clone(),
            file_path: String::new(),
            source_locale: project.primary_locale.clone(),
            target_locale: project.working_locale.clone(),
            note: String::new(),
            issues: vec![],
            candidates: vec![],
            history: vec![],
        };
        let detail = project.details.get(&summary.id).unwrap_or(&fallback_detail);

        connection
            .execute(
                r#"
                INSERT INTO entries(
                  id, sort_order, file_id, key_name, source_value, target_value, status,
                  updated_at, file_path, source_locale, target_locale, note
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    summary.id,
                    index as i64,
                    summary.file_id,
                    summary.key,
                    summary.source_value,
                    summary.target_value,
                    summary.status,
                    summary.updated_at,
                    detail.file_path,
                    detail.source_locale,
                    detail.target_locale,
                    detail.note,
                ],
            )
            .map_err(|error| format!("failed to persist entry '{}': {error}", summary.id))?;

        for issue in &detail.issues {
            connection
                .execute(
                    r#"
                    INSERT INTO entry_issues(id, entry_id, level, message)
                    VALUES (?1, ?2, ?3, ?4)
                    "#,
                    params![issue.id, summary.id, issue.level, issue.message],
                )
                .map_err(|error| {
                    format!("failed to persist validation issue '{}': {error}", issue.id)
                })?;
        }

        for candidate in &detail.candidates {
            connection
                .execute(
                    r#"
                    INSERT INTO entry_candidates(id, entry_id, source, value, score)
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![
                        candidate.id,
                        summary.id,
                        candidate.source,
                        candidate.value,
                        candidate.score
                    ],
                )
                .map_err(|error| {
                    format!("failed to persist candidate '{}': {error}", candidate.id)
                })?;
        }

        for history in &detail.history {
            connection
                .execute(
                    r#"
                    INSERT INTO entry_history(id, entry_id, action, before_value, after_value, operator, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    "#,
                    params![
                        history.id,
                        summary.id,
                        history.action,
                        history.before_value,
                        history.after_value,
                        history.operator,
                        history.created_at,
                    ],
                )
                .map_err(|error| format!("failed to persist history event '{}': {error}", history.id))?;
        }
    }

    Ok(())
}

fn load_locale_graph(connection: &Connection) -> Result<Vec<LocaleDependencyNode>, String> {
    let mut statement = connection
        .prepare("SELECT code, label, parent_code FROM locale_graph ORDER BY rowid ASC")
        .map_err(|error| format!("failed to prepare locale graph query: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok(LocaleDependencyNode {
                code: row.get(0)?,
                label: row.get(1)?,
                parent_code: row.get(2)?,
            })
        })
        .map_err(|error| format!("failed to load locale graph: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to map locale graph rows: {error}"))
}

fn load_files(connection: &Connection) -> Result<Vec<ResourceFileNode>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, logical_path, format, locale, based_on_locale, raw_relative_path FROM resource_files ORDER BY sort_order ASC, rowid ASC",
        )
        .map_err(|error| format!("failed to prepare resource file query: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok(ResourceFileNode {
                id: row.get(0)?,
                name: row.get(1)?,
                logical_path: row.get(2)?,
                format: row.get(3)?,
                locale: row.get(4)?,
                based_on_locale: row.get(5)?,
                raw_relative_path: row.get(6)?,
            })
        })
        .map_err(|error| format!("failed to load resource files: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to map resource file rows: {error}"))
}

fn load_entries(
    connection: &Connection,
) -> Result<(Vec<EntrySummary>, HashMap<String, EntryDetail>), String> {
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, file_id, key_name, source_value, target_value, status,
                   updated_at, file_path, source_locale, target_locale, note
            FROM entries
            ORDER BY sort_order ASC, rowid ASC
            "#,
        )
        .map_err(|error| format!("failed to prepare entry query: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            let entry_id: String = row.get(0)?;
            let file_id: String = row.get(1)?;
            let key: String = row.get(2)?;
            let source_value: String = row.get(3)?;
            let target_value: String = row.get(4)?;
            let status: String = row.get(5)?;
            let updated_at: String = row.get(6)?;
            let file_path: String = row.get(7)?;
            let source_locale: String = row.get(8)?;
            let target_locale: String = row.get(9)?;
            let note: String = row.get(10)?;
            Ok((
                entry_id,
                file_id,
                key,
                source_value,
                target_value,
                status,
                updated_at,
                file_path,
                source_locale,
                target_locale,
                note,
            ))
        })
        .map_err(|error| format!("failed to load entries: {error}"))?;

    let mut entries = Vec::new();
    let mut details = HashMap::new();

    for row in rows {
        let (
            entry_id,
            file_id,
            key,
            source_value,
            target_value,
            status,
            updated_at,
            file_path,
            source_locale,
            target_locale,
            note,
        ) = row.map_err(|error| format!("failed to map entry row: {error}"))?;

        let issues = load_issues(connection, &entry_id)?;
        let candidates = load_candidates(connection, &entry_id)?;
        let history = load_history(connection, &entry_id)?;

        let summary = EntrySummary {
            id: entry_id.clone(),
            file_id,
            key,
            source_value,
            target_value,
            status,
            note_count: usize::from(!note.is_empty()),
            candidate_count: candidates.len(),
            updated_at,
        };

        let detail = EntryDetail {
            summary: summary.clone(),
            file_path,
            source_locale,
            target_locale,
            note,
            issues,
            candidates,
            history,
        };

        entries.push(summary);
        details.insert(entry_id, detail);
    }

    Ok((entries, details))
}

fn load_issues(connection: &Connection, entry_id: &str) -> Result<Vec<ValidationIssue>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, level, message FROM entry_issues WHERE entry_id = ?1 ORDER BY rowid ASC",
        )
        .map_err(|error| format!("failed to prepare issue query: {error}"))?;

    let rows = statement
        .query_map(params![entry_id], |row| {
            Ok(ValidationIssue {
                id: row.get(0)?,
                level: row.get(1)?,
                message: row.get(2)?,
            })
        })
        .map_err(|error| format!("failed to load entry issues for '{entry_id}': {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to map entry issues for '{entry_id}': {error}"))
}

fn load_candidates(connection: &Connection, entry_id: &str) -> Result<Vec<CandidateItem>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, source, value, score FROM entry_candidates WHERE entry_id = ?1 ORDER BY score DESC, rowid ASC",
        )
        .map_err(|error| format!("failed to prepare candidate query: {error}"))?;

    let rows = statement
        .query_map(params![entry_id], |row| {
            Ok(CandidateItem {
                id: row.get(0)?,
                source: row.get(1)?,
                value: row.get(2)?,
                score: row.get(3)?,
            })
        })
        .map_err(|error| format!("failed to load entry candidates for '{entry_id}': {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to map entry candidates for '{entry_id}': {error}"))
}

fn load_history(connection: &Connection, entry_id: &str) -> Result<Vec<HistoryEvent>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, action, before_value, after_value, operator, created_at FROM entry_history WHERE entry_id = ?1 ORDER BY created_at DESC, rowid DESC",
        )
        .map_err(|error| format!("failed to prepare history query: {error}"))?;

    let rows = statement
        .query_map(params![entry_id], |row| {
            Ok(HistoryEvent {
                id: row.get(0)?,
                action: row.get(1)?,
                before_value: row.get(2)?,
                after_value: row.get(3)?,
                operator: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|error| format!("failed to load history for '{entry_id}': {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to map history rows for '{entry_id}': {error}"))
}

fn build_stats(entries: &[EntrySummary]) -> ProjectStats {
    let translated = entries
        .iter()
        .filter(|entry| !entry.target_value.is_empty())
        .count();
    let reviewed = entries
        .iter()
        .filter(|entry| entry.status == "reviewed" || entry.status == "approved")
        .count();

    ProjectStats {
        total: entries.len(),
        translated,
        missing: entries.len().saturating_sub(translated),
        reviewed,
    }
}
