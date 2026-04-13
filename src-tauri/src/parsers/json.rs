use crate::models::ImportedEntry;
use std::path::Path;

pub fn import(path: &Path, content: &[u8]) -> Result<Vec<ImportedEntry>, String> {
    let value: serde_json::Value = serde_json::from_slice(content)
        .map_err(|error| format!("failed to parse JSON file {}: {error}", path.display()))?;
    Ok(super::flatten_json_value(&value))
}
