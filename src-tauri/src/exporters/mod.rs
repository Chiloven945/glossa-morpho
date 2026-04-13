pub mod json;
pub mod properties;
pub mod resx;
pub mod xaml;
pub mod xliff;
pub mod yaml;

fn nested_insert(root: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: String) {
    let parts = key.split('.').collect::<Vec<_>>();
    let mut current = root;
    for (index, part) in parts.iter().enumerate() {
        let is_last = index == parts.len() - 1;
        if is_last {
            current.insert(
                (*part).to_string(),
                serde_json::Value::String(value.clone()),
            );
            return;
        }

        let entry = current
            .entry((*part).to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if !entry.is_object() {
            *entry = serde_json::Value::Object(serde_json::Map::new());
        }
        current = entry.as_object_mut().expect("object entry");
    }
}

pub fn build_nested_map(entries: &[crate::models::EntrySummary]) -> serde_json::Value {
    let mut root = serde_json::Map::new();
    for entry in entries {
        let value = if entry.target_value.is_empty() {
            entry.source_value.clone()
        } else {
            entry.target_value.clone()
        };
        nested_insert(&mut root, &entry.key, value);
    }
    serde_json::Value::Object(root)
}
