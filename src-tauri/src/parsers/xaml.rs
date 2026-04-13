use crate::models::ImportedEntry;
use crate::utils::xml_utils::{collect_text, local_name, parse_xml_root};
use std::path::Path;
use xmltree::{Element, XMLNode};

pub fn import(path: &Path, content: &[u8]) -> Result<Vec<ImportedEntry>, String> {
    let root = parse_xml_root(content, &format!("XAML {}", path.display()))?;
    let mut entries = Vec::new();
    visit_keyed_elements(&root, &mut entries);
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

fn visit_keyed_elements(element: &Element, entries: &mut Vec<ImportedEntry>) {
    if let Some(key) = key_attribute(element) {
        entries.push(ImportedEntry {
            key,
            value: collect_text(element).trim().to_string(),
            source_value: None,
            note: None,
        });
    }

    for child in &element.children {
        if let XMLNode::Element(item) = child {
            visit_keyed_elements(item, entries);
        }
    }
}

fn key_attribute(element: &Element) -> Option<String> {
    element
        .attributes
        .iter()
        .find(|(key, _)| local_name(key) == "Key")
        .map(|(_, value)| value.clone())
}
