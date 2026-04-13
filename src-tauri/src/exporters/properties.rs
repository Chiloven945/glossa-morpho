use crate::models::EntrySummary;

pub fn export(entries: &[EntrySummary]) -> Result<Vec<u8>, String> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.key.cmp(&b.key));
    let content = sorted
        .into_iter()
        .map(|entry| {
            let value = if entry.target_value.is_empty() {
                entry.source_value
            } else {
                entry.target_value
            };
            format!("{}={}", entry.key, escape_value(&value))
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(content.into_bytes())
}

fn escape_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
}
