mod json;
mod properties;
mod resx;
mod xaml;
mod xliff;
mod yaml;

use crate::models::{ImportFileInput, ImportPreviewItem, ImportedEntry, ImportedFilePayload};
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub fn import_files(files: &[ImportFileInput]) -> Result<Vec<ImportedFilePayload>, String> {
    files.iter().map(import_file).collect()
}

fn import_file(input: &ImportFileInput) -> Result<ImportedFilePayload, String> {
    let path = Path::new(input.path.trim());
    if !path.exists() {
        return Err(format!("import file does not exist: {}", path.display()));
    }

    let raw_bytes = fs::read(path)
        .map_err(|error| format!("failed to read import file {}: {error}", path.display()))?;

    let format = detect_format(path)?;
    let imported_entries = match format.as_str() {
        "json" => json::import(path, &raw_bytes)?,
        "yaml" => yaml::import(path, &raw_bytes)?,
        "properties" => properties::import(path, &raw_bytes)?,
        "resx" => resx::import(path, &raw_bytes)?,
        "xaml" => xaml::import(path, &raw_bytes)?,
        "xliff" => xliff::import(path, &raw_bytes)?,
        other => return Err(format!("unsupported import format: {other}")),
    };

    let logical_path = input.logical_path.clone().unwrap_or_else(|| {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("resource")
            .to_string()
    });

    let item = ImportPreviewItem {
        preview_file_id: Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        logical_path: logical_path.clone(),
        name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&logical_path)
            .to_string(),
        format,
        locale: input.locale.clone(),
        based_on_locale: input.based_on_locale.clone(),
        entry_count: imported_entries.len(),
        conflict_count: 0,
    };

    Ok(ImportedFilePayload {
        item,
        raw_bytes,
        imported_entries,
    })
}

pub fn detect_format(path: &Path) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "json" => Ok("json".into()),
        "yaml" | "yml" => Ok("yaml".into()),
        "properties" => Ok("properties".into()),
        "resx" => Ok("resx".into()),
        "xaml" => Ok("xaml".into()),
        "xlf" | "xliff" => Ok("xliff".into()),
        _ => Err(format!(
            "unsupported import format for file {}",
            path.display()
        )),
    }
}

fn flatten_value(prefix: Option<&str>, value: &serde_json::Value, output: &mut Vec<ImportedEntry>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let next_prefix = match prefix {
                    Some(prefix) if !prefix.is_empty() => format!("{prefix}.{key}"),
                    _ => key.clone(),
                };
                flatten_value(Some(&next_prefix), child, output);
            }
        }
        serde_json::Value::Array(_) => {
            let key = prefix.unwrap_or("root").to_string();
            output.push(ImportedEntry {
                key,
                value: serde_json::to_string(value).unwrap_or_default(),
                source_value: None,
                note: None,
            });
        }
        serde_json::Value::Null => {
            let key = prefix.unwrap_or("root").to_string();
            output.push(ImportedEntry {
                key,
                value: String::new(),
                source_value: None,
                note: None,
            });
        }
        serde_json::Value::Bool(boolean) => {
            let key = prefix.unwrap_or("root").to_string();
            output.push(ImportedEntry {
                key,
                value: boolean.to_string(),
                source_value: None,
                note: None,
            });
        }
        serde_json::Value::Number(number) => {
            let key = prefix.unwrap_or("root").to_string();
            output.push(ImportedEntry {
                key,
                value: number.to_string(),
                source_value: None,
                note: None,
            });
        }
        serde_json::Value::String(string) => {
            let key = prefix.unwrap_or("root").to_string();
            output.push(ImportedEntry {
                key,
                value: string.clone(),
                source_value: None,
                note: None,
            });
        }
    }
}

pub(crate) fn flatten_json_value(value: &serde_json::Value) -> Vec<ImportedEntry> {
    let mut output = Vec::new();
    flatten_value(None, value, &mut output);
    output.sort_by(|a, b| a.key.cmp(&b.key));
    output
}
