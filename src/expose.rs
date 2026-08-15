use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const ASSISTANT: &str = "conversation";

#[derive(Debug, Default)]
pub struct ExposeStore {
    legacy: HashMap<String, bool>,
}

#[derive(Debug)]
pub struct ExposeHint {
    pub entity_id: String,
    pub should_expose: Option<bool>,
}

pub fn load_assist(config_dir: &Path) -> Option<HashSet<String>> {
    let registry = config_dir.join(".storage/core.entity_registry");
    let store_path = config_dir.join(".storage/homeassistant.exposed_entities");
    if !registry.exists() && !store_path.exists() {
        return None;
    }
    let store = read_store(&store_path);
    let hints = read_registry_hints(&registry);
    if hints.is_empty() && store.legacy.is_empty() && !store_path.exists() {
        return None;
    }
    Some(resolve(&hints, &store))
}

pub fn resolve(hints: &[ExposeHint], store: &ExposeStore) -> HashSet<String> {
    hints
        .iter()
        .filter(|hint| is_exposed(hint, store))
        .map(|hint| hint.entity_id.clone())
        .collect()
}

fn is_exposed(hint: &ExposeHint, store: &ExposeStore) -> bool {
    hint.should_expose
        .or_else(|| store.legacy.get(&hint.entity_id).copied())
        .unwrap_or(false)
}

fn read_store(path: &Path) -> ExposeStore {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return ExposeStore::default();
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return ExposeStore::default();
    };
    let Some(entities) = v.pointer("/data/exposed_entities").and_then(Value::as_object) else {
        return ExposeStore::default();
    };
    let legacy = entities
        .iter()
        .filter_map(|(id, entity)| {
            flag(entity.pointer(&format!("/assistants/{ASSISTANT}/should_expose"))).map(|f| (id.clone(), f))
        })
        .collect();
    ExposeStore { legacy }
}

fn read_registry_hints(path: &Path) -> Vec<ExposeHint> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return Vec::new();
    };
    let Some(entities) = v.pointer("/data/entities").and_then(Value::as_array) else {
        return Vec::new();
    };
    entities
        .iter()
        .filter(|e| e.get("disabled_by").is_none_or(Value::is_null))
        .filter_map(|e| {
            Some(ExposeHint {
                entity_id: e.get("entity_id")?.as_str()?.to_string(),
                should_expose: flag(e.pointer("/options/conversation/should_expose")),
            })
        })
        .collect()
}

fn flag(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hint(id: &str, expose: Option<bool>) -> ExposeHint {
        ExposeHint {
            entity_id: id.into(),
            should_expose: expose,
        }
    }

    #[test]
    fn only_explicit_assist_flag() {
        let store = ExposeStore::default();
        let ids = resolve(
            &[
                hint("light.kugel", Some(true)),
                hint("light.hue_play_1", Some(false)),
                hint("light.hue_play_2", None),
            ],
            &store,
        );
        assert!(ids.contains("light.kugel"));
        assert!(!ids.contains("light.hue_play_1"));
        assert!(!ids.contains("light.hue_play_2"));
    }

    #[test]
    fn legacy_store_counts_as_explicit() {
        let store = ExposeStore {
            legacy: [("script.musik".into(), true)].into(),
        };
        let ids = resolve(&[hint("script.musik", None)], &store);
        assert!(ids.contains("script.musik"));
    }
}
