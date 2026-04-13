use crate::models::{EntryDetail, EntrySummary, ResourceFileNode};
use crate::utils::xml_utils::{
    element_text, load_raw_or_default, local_name, remove_direct_children,
    set_or_create_direct_child_text, write_xml_document,
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
    let raw = load_raw_or_default(
        file,
        workspace_dir,
        b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<root>\n</root>\n",
    );
    let mut root = crate::utils::xml_utils::parse_xml_root(&raw, "RESX raw file")?;

    for entry in entries {
        let value = effective_value(entry);
        let note = details
            .get(&entry.id)
            .map(|detail| detail.note.clone())
            .unwrap_or_default();

        if let Some(data) = find_resx_data_mut(&mut root, &entry.key) {
            set_or_create_direct_child_text(data, "value", &value);
            if note.trim().is_empty() {
                remove_direct_children(data, "comment");
            } else {
                set_or_create_direct_child_text(data, "comment", &note);
            }
        } else {
            let mut data = Element::new("data");
            data.attributes.insert("name".into(), entry.key.clone());
            data.attributes
                .insert("xml:space".into(), "preserve".into());
            data.children
                .push(XMLNode::Element(element_text("value", &value)));
            if !note.trim().is_empty() {
                data.children
                    .push(XMLNode::Element(element_text("comment", &note)));
            }
            root.children.push(XMLNode::Element(data));
        }
    }

    write_xml_document(&root, "RESX")
}

fn effective_value(entry: &EntrySummary) -> String {
    if entry.target_value.is_empty() {
        entry.source_value.clone()
    } else {
        entry.target_value.clone()
    }
}

fn find_resx_data_mut<'a>(root: &'a mut Element, key: &str) -> Option<&'a mut Element> {
    root.children.iter_mut().find_map(|child| match child {
        XMLNode::Element(data)
            if local_name(&data.name) == "data"
                && data.attributes.get("name").map(|value| value.as_str()) == Some(key) =>
        {
            Some(data)
        }
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EntryDetail, EntrySummary, ResourceFileNode};
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn preserves_resx_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let raw_rel = "raw/locales/en-US/strings.resx";
        let raw_path = dir.path().join(raw_rel);
        std::fs::create_dir_all(raw_path.parent().unwrap()).unwrap();
        std::fs::write(
            &raw_path,
            include_bytes!("../../../tests/fixtures/resx/strings.resx"),
        )
        .unwrap();
        let file = ResourceFileNode {
            id: "f1".into(),
            name: "strings.resx".into(),
            logical_path: "strings.resx".into(),
            format: "resx".into(),
            locale: "en-US".into(),
            based_on_locale: None,
            raw_relative_path: Some(raw_rel.into()),
        };
        let entry = EntrySummary {
            id: "e1".into(),
            file_id: "f1".into(),
            key: "WelcomeTitle".into(),
            source_value: "Welcome".into(),
            target_value: "Hello".into(),
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
                file_path: "strings.resx".into(),
                source_locale: "en-US".into(),
                target_locale: "en-US".into(),
                note: "friendly".into(),
                issues: vec![],
                candidates: vec![],
                history: vec![],
            },
        );
        let out = export(&file, &[entry], &details, Some(Path::new(dir.path()))).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("<value>Hello</value>"));
        assert!(text.contains("<comment>friendly</comment>"));
    }
}
