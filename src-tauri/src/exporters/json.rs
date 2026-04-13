use crate::models::EntrySummary;

pub fn export(entries: &[EntrySummary]) -> Result<Vec<u8>, String> {
    let value = super::build_nested_map(entries);
    serde_json::to_vec_pretty(&value)
        .map_err(|error| format!("failed to serialize JSON export: {error}"))
}
