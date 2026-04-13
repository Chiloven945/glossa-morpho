use crate::models::ResourceFileNode;
use std::fs;
use std::path::Path;
use xmltree::{Element, EmitterConfig, XMLNode};

pub fn local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

pub fn parse_xml_root(content: &[u8], label: &str) -> Result<Element, String> {
    Element::parse(content).map_err(|error| format!("failed to parse {label} XML: {error}"))
}

pub fn load_raw_or_default(
    file: &ResourceFileNode,
    workspace_dir: Option<&Path>,
    default_xml: &[u8],
) -> Vec<u8> {
    let Some(relative) = file.raw_relative_path.as_ref() else {
        return default_xml.to_vec();
    };
    let Some(root) = workspace_dir else {
        return default_xml.to_vec();
    };
    fs::read(root.join(relative)).unwrap_or_else(|_| default_xml.to_vec())
}

pub fn collect_text(element: &Element) -> String {
    let mut out = String::new();
    for child in &element.children {
        match child {
            XMLNode::Text(text) | XMLNode::CData(text) => out.push_str(text),
            XMLNode::Element(item) => out.push_str(&collect_text(item)),
            _ => {}
        }
    }
    out
}

pub fn find_direct_child_text(element: &Element, name: &str) -> Option<String> {
    element.children.iter().find_map(|child| match child {
        XMLNode::Element(item) if local_name(&item.name) == name => {
            let text = collect_text(item).trim().to_string();
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    })
}

pub fn find_first_descendant_text(element: &Element, name: &str) -> Option<String> {
    if local_name(&element.name) == name {
        let value = collect_text(element).trim().to_string();
        if !value.is_empty() {
            return Some(value);
        }
    }

    for child in &element.children {
        if let XMLNode::Element(item) = child {
            if let Some(value) = find_first_descendant_text(item, name) {
                return Some(value);
            }
        }
    }
    None
}

pub fn element_text(name: &str, value: &str) -> Element {
    let mut element = Element::new(name);
    element.children.push(XMLNode::Text(value.to_string()));
    element
}

pub fn direct_child_mut<'a>(parent: &'a mut Element, name: &str) -> Option<&'a mut Element> {
    parent.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(item) if local_name(&item.name) == name => Some(item),
        _ => None,
    })
}

pub fn get_or_create_direct_child<'a>(parent: &'a mut Element, name: &str) -> &'a mut Element {
    if let Some(index) = parent
        .children
        .iter()
        .position(|child| matches!(child, XMLNode::Element(item) if local_name(&item.name) == name))
    {
        match parent.children.get_mut(index) {
            Some(XMLNode::Element(item)) => return item,
            _ => unreachable!(),
        }
    }
    parent.children.push(XMLNode::Element(Element::new(name)));
    match parent.children.last_mut() {
        Some(XMLNode::Element(item)) => item,
        _ => unreachable!(),
    }
}

pub fn remove_direct_children(parent: &mut Element, name: &str) {
    parent
        .children
        .retain(|child| !matches!(child, XMLNode::Element(item) if local_name(&item.name) == name));
}

pub fn set_or_create_direct_child_text(parent: &mut Element, name: &str, value: &str) {
    let child = get_or_create_direct_child(parent, name);
    child
        .children
        .retain(|node| !matches!(node, XMLNode::Text(_) | XMLNode::CData(_)));
    child.children.insert(0, XMLNode::Text(value.to_string()));
}

pub fn write_xml_document(root: &Element, label: &str) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    root.write_with_config(
        &mut output,
        EmitterConfig::new()
            .perform_indent(true)
            .write_document_declaration(true),
    )
    .map_err(|error| format!("failed to write {label} XML: {error}"))?;
    Ok(output)
}
