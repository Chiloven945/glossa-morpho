use crate::models::EntrySummary;

pub fn export(entries: &[EntrySummary]) -> Result<Vec<u8>, String> {
    let value = super::build_nested_map(entries);
    serde_yaml::to_string(&value)
        .map(|text| text.into_bytes())
        .map_err(|error| format!("failed to serialize YAML export: {error}"))
}
