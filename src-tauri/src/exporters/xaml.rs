use crate::models::{EntryDetail, EntrySummary, ResourceFileNode};
use crate::utils::xml_utils::{
    load_raw_or_default, local_name, parse_xml_root, write_xml_document,
};
use std::collections::HashMap;
use std::path::Path;
use xmltree::{Element, XMLNode};

pub fn export(
    file: &ResourceFileNode,
    entries: &[EntrySummary],
    _details: &HashMap<String, EntryDetail>,
    workspace_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let raw = load_raw_or_default(
        file,
        workspace_dir,
        b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ResourceDictionary xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\" xmlns:x=\"http://schemas.microsoft.com/winfx/2006/xaml\" xmlns:sys=\"clr-namespace:System;assembly=mscorlib\"/>\n",
    );
    let mut root = parse_xml_root(&raw, "XAML raw file")?;
    ensure_xaml_namespaces(&mut root);

    for entry in entries {
        let value = if entry.target_value.is_empty() {
            entry.source_value.clone()
        } else {
            entry.target_value.clone()
        };
        if let Some(element) = find_keyed_element_mut(&mut root, &entry.key) {
            set_element_text(element, &value);
        } else {
            append_string_element(&mut root, &entry.key, &value);
        }
    }

    write_xml_document(&root, "XAML")
}

fn ensure_xaml_namespaces(root: &mut Element) {
    root.attributes
        .entry("xmlns:x".into())
        .or_insert_with(|| "http://schemas.microsoft.com/winfx/2006/xaml".into());
    root.attributes
        .entry("xmlns:sys".into())
        .or_insert_with(|| "clr-namespace:System;assembly=mscorlib".into());
}

fn find_keyed_element_mut<'a>(element: &'a mut Element, key: &str) -> Option<&'a mut Element> {
    let is_match = element
        .attributes
        .iter()
        .any(|(name, value)| local_name(name) == "Key" && value == key);
    if is_match {
        return Some(element);
    }

    for child in &mut element.children {
        if let XMLNode::Element(item) = child {
            if let Some(found) = find_keyed_element_mut(item, key) {
                return Some(found);
            }
        }
    }
    None
}

fn set_element_text(element: &mut Element, value: &str) {
    element
        .children
        .retain(|node| !matches!(node, XMLNode::Text(_) | XMLNode::CData(_)));
    element.children.insert(0, XMLNode::Text(value.to_string()));
}

fn append_string_element(root: &mut Element, key: &str, value: &str) {
    let mut element = Element::new("sys:String");
    element.attributes.insert("x:Key".into(), key.to_string());
    element.children.push(XMLNode::Text(value.to_string()));
    root.children.push(XMLNode::Element(element));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntryDetail, EntrySummary, ResourceFileNode};
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn preserves_xaml_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let raw_rel = "raw/locales/en-US/Strings.xaml";
        let raw_path = dir.path().join(raw_rel);
        std::fs::create_dir_all(raw_path.parent().unwrap()).unwrap();
        std::fs::write(
            &raw_path,
            include_bytes!("../../../tests/fixtures/xaml/Strings.xaml"),
        )
        .unwrap();
        let file = ResourceFileNode {
            id: "f1".into(),
            name: "Strings.xaml".into(),
            logical_path: "Strings.xaml".into(),
            format: "xaml".into(),
            locale: "en-US".into(),
            based_on_locale: None,
            raw_relative_path: Some(raw_rel.into()),
        };
        let entry = EntrySummary {
            id: "e1".into(),
            file_id: "f1".into(),
            key: "AppTitle".into(),
            source_value: "Starter".into(),
            target_value: "Localized".into(),
            status: "translated".into(),
            note_count: 0,
            candidate_count: 0,
            updated_at: String::new(),
        };
        let out = export(
            &file,
            &[entry],
            &HashMap::<String, EntryDetail>::new(),
            Some(Path::new(dir.path())),
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("Localized"));
        assert!(text.contains("AppTitle"));
        assert!(text.contains("xmlns:sys"));
    }
}
