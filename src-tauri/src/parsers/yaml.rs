use crate::models::ImportedEntry;
use std::path::Path;

pub fn import(path: &Path, content: &[u8]) -> Result<Vec<ImportedEntry>, String> {
    let value: serde_yaml::Value = serde_yaml::from_slice(content)
        .map_err(|error| format!("failed to parse YAML file {}: {error}", path.display()))?;
    let json_value = serde_json::to_value(value)
        .map_err(|error| format!("failed to normalize YAML file {}: {error}", path.display()))?;
    Ok(super::flatten_json_value(&json_value))
}
