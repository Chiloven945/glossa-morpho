use crate::models::ImportedEntry;
use std::path::Path;

pub fn import(path: &Path, content: &[u8]) -> Result<Vec<ImportedEntry>, String> {
    let raw = String::from_utf8(content.to_vec()).map_err(|error| {
        format!(
            "failed to decode properties file {} as UTF-8: {error}",
            path.display()
        )
    })?;

    let mut entries = Vec::new();
    let mut current = String::new();

    for raw_line in raw.lines() {
        let line = raw_line.trim_end();
        if line.ends_with('\\') {
            current.push_str(line.trim_end_matches('\\'));
            continue;
        }
        current.push_str(line);

        let line = current.trim().to_string();
        current.clear();

        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }

        let (key, value) = if let Some((key, value)) = line.split_once('=') {
            (key.trim(), value.trim())
        } else if let Some((key, value)) = line.split_once(':') {
            (key.trim(), value.trim())
        } else {
            let mut parts = line.splitn(2, char::is_whitespace);
            let key = parts.next().unwrap_or_default().trim();
            let value = parts.next().unwrap_or_default().trim();
            (key, value)
        };

        if !key.is_empty() {
            entries.push(ImportedEntry {
                key: key.to_string(),
                value: decode_escapes(value),
                source_value: None,
                note: None,
            });
        }
    }

    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

fn decode_escapes(value: &str) -> String {
    value
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\\", "\\")
}
