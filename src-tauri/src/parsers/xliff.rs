use crate::models::ImportedEntry;
use crate::utils::xml_utils::{find_first_descendant_text, local_name, parse_xml_root};
use std::path::Path;
use xmltree::{Element, XMLNode};

pub fn import(path: &Path, content: &[u8]) -> Result<Vec<ImportedEntry>, String> {
    let root = parse_xml_root(content, &format!("XLIFF {}", path.display()))?;

    let mut entries = Vec::new();
    visit_units(&root, &mut entries);
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

fn visit_units(element: &Element, entries: &mut Vec<ImportedEntry>) {
    let element_name = local_name(&element.name);
    if element_name == "trans-unit" || element_name == "unit" {
        if let Some(key) = element
            .attributes
            .get("id")
            .or_else(|| element.attributes.get("resname"))
            .cloned()
        {
            let source_value = find_first_descendant_text(element, "source");
            let target_value = find_first_descendant_text(element, "target")
                .or_else(|| source_value.clone())
                .unwrap_or_default();
            let note = find_first_descendant_text(element, "note");
            entries.push(ImportedEntry {
                key,
                value: target_value,
                source_value,
                note,
            });
        }
    }

    for child in &element.children {
        if let XMLNode::Element(item) = child {
            visit_units(item, entries);
        }
    }
}
