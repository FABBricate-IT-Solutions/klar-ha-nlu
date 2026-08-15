use crate::compound::{is_infra, is_tv_switch};
use crate::expose::assist_visible;
use crate::normalize::compact;
use crate::types::{EntityRec, HomeGraph};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub fn role_of_tag(tag: &str) -> Option<&'static str> {
    match compact(tag).as_str() {
        "licht" | "light" | "lampe" | "lampen" | "leuchte" | "beleuchtung" | "lighting" => Some("light"),
        "heizung" | "thermostat" | "klima" | "klimaanlage" | "heater" | "climate" | "heat" => Some("climate"),
        "tv" | "fernseher" | "media" => Some("media_player"),
        "luefter" | "ventilator" | "fan" | "geblaese" => Some("fan"),
        _ => None,
    }
}

pub fn is_role_tag(tag: &str) -> bool {
    role_of_tag(tag).is_some()
}

pub fn has_role(entity: &EntityRec, domain: &str) -> bool {
    entity.tags.iter().any(|tag| role_of_tag(tag) == Some(domain))
}

pub fn is_light_like(entity: &EntityRec) -> bool {
    entity.domain == "light" || has_role(entity, "light")
}

pub fn matches_domain(entity: &EntityRec, domain: &str) -> bool {
    entity.domain == domain || has_role(entity, domain) || is_tv_switch(domain, entity)
}

pub fn unique_role_in_area(home: &HomeGraph, area: &str, domain: &str) -> Option<String> {
    let hits: Vec<&str> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| e.area.as_deref() == Some(area) && has_role(e, domain) && e.domain != domain && !is_infra(e))
        .map(|e| e.entity_id.as_str())
        .collect();
    (hits.len() == 1).then(|| hits[0].to_string())
}

pub fn role_siblings<'a>(home: &'a HomeGraph, area: &str, domain: &str) -> Vec<&'a EntityRec> {
    home.entities
        .iter()
        .filter(|e| assist_visible(e, home) && e.area.as_deref() == Some(area) && has_role(e, domain) && e.domain != domain && !is_infra(e))
        .collect()
}

pub fn expand_entity_tags(ids: Vec<String>, names: &HashMap<String, String>) -> Vec<String> {
    let mut out = Vec::new();
    for id in ids {
        if let Some(name) = names.get(&id) {
            if !out.iter().any(|t| t == name) {
                out.push(name.clone());
            }
        }
        if !out.iter().any(|t| t == &id) {
            out.push(id);
        }
    }
    out
}

pub fn load_label_names(path: &Path) -> HashMap<String, String> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    let Ok(parsed) = serde_json::from_str::<LabelStorage>(&raw) else {
        return HashMap::new();
    };
    parsed
        .data
        .labels
        .into_iter()
        .filter_map(|label| {
            let id = label.label_id.or(label.id)?;
            Some((id, label.name))
        })
        .collect()
}

#[derive(Deserialize)]
struct LabelStorage {
    data: LabelData,
}

#[derive(Deserialize)]
struct LabelData {
    #[serde(default)]
    labels: Vec<RawLabel>,
}

#[derive(Deserialize)]
struct RawLabel {
    #[serde(default)]
    label_id: Option<String>,
    #[serde(default)]
    id: Option<String>,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_maps_known_tags() {
        assert_eq!(role_of_tag("Licht"), Some("light"));
        assert_eq!(role_of_tag("lüfter"), Some("fan"));
        assert_eq!(role_of_tag("Heizung"), Some("climate"));
        assert_eq!(role_of_tag("TV"), Some("media_player"));
        assert_eq!(role_of_tag("wichtig"), None);
        assert_eq!(role_of_tag("og"), None);
    }

    #[test]
    fn label_ids_resolve_to_names() {
        let names = HashMap::from([("lbl_1".into(), "Licht".into())]);
        let tags = expand_entity_tags(vec!["lbl_1".into()], &names);
        assert!(tags.iter().any(|t| t == "Licht"), "{tags:?}");
        assert!(has_role(
            &EntityRec { entity_id: "switch.x".into(), name: "X".into(), domain: "switch".into(), area: None, aliases: Vec::new(), tags },
            "light"
        ));
    }
}
