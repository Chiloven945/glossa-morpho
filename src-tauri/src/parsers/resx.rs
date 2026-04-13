use crate::models::ImportedEntry;
use crate::utils::xml_utils::{collect_text, find_direct_child_text, local_name, parse_xml_root};
use std::path::Path;
use xmltree::XMLNode;

pub fn import(path: &Path, content: &[u8]) -> Result<Vec<ImportedEntry>, String> {
    let root = parse_xml_root(content, &format!("RESX {}", path.display()))?;

    let mut entries = Vec::new();
    for child in &root.children {
        let XMLNode::Element(data) = child else {
            continue;
        };
        if local_name(&data.name) != "data" {
            continue;
        }
        let Some(key) = data.attributes.get("name").cloned() else {
            continue;
        };
        let value = find_direct_child_text(data, "value")
            .unwrap_or_else(|| collect_text(data).trim().to_string());
        let note = find_direct_child_text(data, "comment");
        entries.push(ImportedEntry {
            key,
            value,
            source_value: None,
            note,
        });
    }

    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}
