use crate::models::{EntryDetail, EntrySummary, ResourceFileNode};
use crate::utils::xml_utils::{
    element_text, get_or_create_direct_child, load_raw_or_default, local_name, parse_xml_root,
    remove_direct_children, set_or_create_direct_child_text, write_xml_document,
};
use std::collections::HashMap;
use std::path::Path;
use xmltree::{Element, XMLNode};

pub fn export(
    file: &ResourceFileNode,
    entries: &[EntrySummary],
    details: &HashMap<String, EntryDetail>,
    workspace_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let default_xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<xliff version=\"1.2\">\n  <file source-language=\"en-US\" target-language=\"{}\">\n    <body/>\n  </file>\n</xliff>\n",
        file.locale
    );
    let raw = load_raw_or_default(file, workspace_dir, default_xml.as_bytes());
    let mut root = parse_xml_root(&raw, "XLIFF raw file")?;

    let use_units = has_descendant_named(&root, "unit");

    for entry in entries {
        let detail = details.get(&entry.id);
        let source = entry.source_value.clone();
        let target = if entry.target_value.is_empty() {
            entry.source_value.clone()
        } else {
            entry.target_value.clone()
        };
        let note = detail.map(|item| item.note.clone()).unwrap_or_default();
        if use_units {
            if let Some(unit) = find_named_unit_mut(&mut root, "unit", &entry.key) {
                update_unit(unit, &entry.key, &source, &target, &note);
            } else {
                append_xliff2_unit(&mut root, &entry.key, &source, &target, &note);
            }
        } else if let Some(unit) = find_named_unit_mut(&mut root, "trans-unit", &entry.key) {
            update_trans_unit(unit, &entry.key, &source, &target, &note);
        } else {
            append_xliff12_unit(&mut root, &entry.key, &source, &target, &note);
        }
    }

    write_xml_document(&root, "XLIFF")
}

fn has_descendant_named(element: &Element, name: &str) -> bool {
    if local_name(&element.name) == name {
        return true;
    }
    element.children.iter().any(|child| match child {
        XMLNode::Element(item) => has_descendant_named(item, name),
        _ => false,
    })
}

fn find_named_unit_mut<'a>(
    element: &'a mut Element,
    element_name: &str,
    key: &str,
) -> Option<&'a mut Element> {
    let is_match = local_name(&element.name) == element_name
        && element
            .attributes
            .get("id")
            .or_else(|| element.attributes.get("resname"))
            .map(|value| value == key)
            .unwrap_or(false);
    if is_match {
        return Some(element);
    }
    for child in &mut element.children {
        if let XMLNode::Element(item) = child {
            if let Some(found) = find_named_unit_mut(item, element_name, key) {
                return Some(found);
            }
        }
    }
    None
}

fn update_trans_unit(unit: &mut Element, key: &str, source: &str, target: &str, note: &str) {
    unit.attributes.insert("id".into(), key.to_string());
    set_or_create_direct_child_text(unit, "source", source);
    set_or_create_direct_child_text(unit, "target", target);
    if note.trim().is_empty() {
        remove_direct_children(unit, "note");
    } else {
        set_or_create_direct_child_text(unit, "note", note);
    }
}

fn update_unit(unit: &mut Element, key: &str, source: &str, target: &str, note: &str) {
    unit.attributes.insert("id".into(), key.to_string());
    let segment = get_or_create_direct_child(unit, "segment");
    set_or_create_direct_child_text(segment, "source", source);
    set_or_create_direct_child_text(segment, "target", target);
    if note.trim().is_empty() {
        remove_direct_children(unit, "note");
    } else {
        set_or_create_direct_child_text(unit, "note", note);
    }
}

fn append_xliff12_unit(root: &mut Element, key: &str, source: &str, target: &str, note: &str) {
    let body = find_or_create_descendant(root, &["file", "body"]);
    let mut unit = Element::new("trans-unit");
    unit.attributes.insert("id".into(), key.to_string());
    unit.children
        .push(XMLNode::Element(element_text("source", source)));
    unit.children
        .push(XMLNode::Element(element_text("target", target)));
    if !note.trim().is_empty() {
        unit.children
            .push(XMLNode::Element(element_text("note", note)));
    }
    body.children.push(XMLNode::Element(unit));
}

fn append_xliff2_unit(root: &mut Element, key: &str, source: &str, target: &str, note: &str) {
    let file = find_or_create_descendant(root, &["file"]);
    let mut unit = Element::new("unit");
    unit.attributes.insert("id".into(), key.to_string());
    let mut segment = Element::new("segment");
    segment
        .children
        .push(XMLNode::Element(element_text("source", source)));
    segment
        .children
        .push(XMLNode::Element(element_text("target", target)));
    unit.children.push(XMLNode::Element(segment));
    if !note.trim().is_empty() {
        unit.children
            .push(XMLNode::Element(element_text("note", note)));
    }
    file.children.push(XMLNode::Element(unit));
}

fn find_or_create_descendant<'a>(root: &'a mut Element, path: &[&str]) -> &'a mut Element {
    let mut current = root;
    for segment in path {
        let next_index = current.children.iter().position(
            |child| matches!(child, XMLNode::Element(item) if local_name(&item.name) == *segment),
        );
        let index = match next_index {
            Some(index) => index,
            None => {
                current
                    .children
                    .push(XMLNode::Element(Element::new(*segment)));
                current.children.len() - 1
            }
        };
        current = match current.children.get_mut(index) {
            Some(XMLNode::Element(item)) => item,
            _ => unreachable!(),
        };
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntryDetail, EntrySummary, ResourceFileNode};
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn preserves_xliff_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let raw_rel = "raw/locales/zh-CN/demo.xlf";
        let raw_path = dir.path().join(raw_rel);
        std::fs::create_dir_all(raw_path.parent().unwrap()).unwrap();
        std::fs::write(
            &raw_path,
            include_bytes!("../../../tests/fixtures/xliff/demo.xlf"),
        )
        .unwrap();
        let file = ResourceFileNode {
            id: "f1".into(),
            name: "demo.xlf".into(),
            logical_path: "demo.xlf".into(),
            format: "xliff".into(),
            locale: "zh-CN".into(),
            based_on_locale: Some("en-US".into()),
            raw_relative_path: Some(raw_rel.into()),
        };
        let entry = EntrySummary {
            id: "e1".into(),
            file_id: "f1".into(),
            key: "welcome".into(),
            source_value: "Welcome back, {name}".into(),
            target_value: "您好，{name}".into(),
            status: "translated".into(),
            note_count: 0,
            candidate_count: 0,
            updated_at: String::new(),
        };
        let mut details = HashMap::new();
        details.insert(
            "e1".into(),
            EntryDetail {
                summary: entry.clone(),
                file_path: "demo.xlf".into(),
                source_locale: "en-US".into(),
                target_locale: "zh-CN".into(),
                note: "friendly".into(),
                issues: vec![],
                candidates: vec![],
                history: vec![],
            },
        );
        let out = export(&file, &[entry], &details, Some(Path::new(dir.path()))).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("<target>您好，{name}</target>"));
        assert!(text.contains("<note>friendly</note>"));
    }
}
