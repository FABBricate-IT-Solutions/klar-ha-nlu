use crate::home::classify::is_tv_switch;
use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::Catalog;
use crate::parse::normalize::compact;
use crate::types::{EntityRec, HomeGraph};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClimateKind {
    Heat,
    Cool,
}

pub fn role_of_tag(tag: &str, cat: &Catalog) -> Option<&'static str> {
    let folded = compact(tag);
    if cat.role_light.contains(folded.as_str()) {
        Some("light")
    } else if cat.role_climate.contains(folded.as_str()) {
        Some("climate")
    } else if cat.role_media.contains(folded.as_str()) {
        Some("media_player")
    } else if cat.role_fan.contains(folded.as_str()) {
        Some("fan")
    } else {
        None
    }
}

pub fn wanted_climate_kind(tokens: &[String], cat: &Catalog) -> Option<ClimateKind> {
    let cool = cat.any(tokens, &cat.climate_cool);
    let heat = cat.any(tokens, &cat.climate_heat);
    match (cool, heat) {
        (true, false) => Some(ClimateKind::Cool),
        (false, true) => Some(ClimateKind::Heat),
        _ => None,
    }
}

pub fn climate_kind(entity: &EntityRec, cat: &Catalog) -> Option<ClimateKind> {
    if entity.domain != "climate" && !has_role(entity, "climate", cat) {
        return None;
    }
    if cool_named(entity, cat) {
        return Some(ClimateKind::Cool);
    }
    if heat_named(entity, cat) {
        return Some(ClimateKind::Heat);
    }
    let tags: Vec<String> = entity.tags.iter().map(|tag| compact(tag)).collect();
    if tags.iter().any(|tag| cat.climate_cool.contains(tag.as_str())) {
        return Some(ClimateKind::Cool);
    }
    if tags.iter().any(|tag| cat.climate_heat.contains(tag.as_str())) {
        return Some(ClimateKind::Heat);
    }
    Some(ClimateKind::Heat)
}

fn climate_blob(entity: &EntityRec) -> String {
    compact(&format!("{} {} {}", entity.entity_id, entity.name, entity.aliases.join(" ")))
}

fn cool_named(entity: &EntityRec, cat: &Catalog) -> bool {
    let blob = climate_blob(entity);
    cat.climate_cool.iter().any(|word| blob.contains(word)) || blob.ends_with("ac")
}

fn heat_named(entity: &EntityRec, cat: &Catalog) -> bool {
    let blob = climate_blob(entity);
    cat.climate_heat.iter().any(|word| blob.contains(word))
}

pub fn is_role_tag(tag: &str, cat: &Catalog) -> bool {
    role_of_tag(tag, cat).is_some()
}

pub fn has_role(entity: &EntityRec, domain: &str, cat: &Catalog) -> bool {
    entity.tags.iter().any(|tag| role_of_tag(tag, cat) == Some(domain))
}

pub fn is_light_like(entity: &EntityRec, cat: &Catalog) -> bool {
    entity.domain == "light" || has_role(entity, "light", cat)
}

pub fn is_music_assistant_player(entity: &EntityRec) -> bool {
    entity.domain == "media_player" && entity.platform.as_deref() == Some("music_assistant")
}

pub fn is_music_player(entity: &EntityRec) -> bool {
    is_music_assistant_player(entity)
        || (entity.domain == "media_player"
            && entity.tags.iter().any(|tag| matches!(compact(tag).as_str(), "musik" | "music" | "medien" | "media")))
}

pub fn matches_domain(entity: &EntityRec, domain: &str, cat: &Catalog) -> bool {
    entity.domain == domain || has_role(entity, domain, cat) || is_tv_switch(domain, entity, cat)
}

pub fn unique_role_in_area(home: &HomeGraph, area: &str, domain: &str, cat: &Catalog) -> Option<String> {
    let hits: Vec<&str> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| e.area.as_deref() == Some(area) && has_role(e, domain, cat) && e.domain != domain && !is_infra(e))
        .map(|e| e.entity_id.as_str())
        .collect();
    (hits.len() == 1).then(|| hits[0].to_string())
}

pub fn role_siblings<'a>(home: &'a HomeGraph, area: &str, domain: &str, cat: &Catalog) -> Vec<&'a EntityRec> {
    home.entities
        .iter()
        .filter(|e| {
            assist_visible(e, home) && e.area.as_deref() == Some(area) && has_role(e, domain, cat) && e.domain != domain && !is_infra(e)
        })
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
        let cat = crate::lang::catalog();
        assert_eq!(role_of_tag("Licht", cat), Some("light"));
        assert_eq!(role_of_tag("lüfter", cat), Some("fan"));
        assert_eq!(role_of_tag("Heizung", cat), Some("climate"));
        assert_eq!(role_of_tag("Klima", cat), Some("climate"));
        assert_eq!(role_of_tag("TV", cat), Some("media_player"));
        let ac = EntityRec {
            entity_id: "climate.schlafzimmer_ac".into(),
            name: "Schlafzimmer AC".into(),
            domain: "climate".into(),
            platform: None,
            area: Some("schlafzimmer".into()),
            aliases: vec!["Klimaanlage".into()],
            tags: vec!["Klima".into()],
        };
        let heat = EntityRec {
            entity_id: "climate.better_thermostat_schlafzimmer".into(),
            name: "Better Thermostat Schlafzimmer".into(),
            domain: "climate".into(),
            platform: None,
            area: Some("schlafzimmer".into()),
            aliases: vec!["Heizung Schlafzimmer".into()],
            tags: vec!["Klima".into()],
        };
        assert_eq!(climate_kind(&ac, cat), Some(ClimateKind::Cool));
        assert_eq!(climate_kind(&heat, cat), Some(ClimateKind::Heat));
        assert_eq!(wanted_climate_kind(&["klimaanlage".into(), "20".into()], cat), Some(ClimateKind::Cool));
        assert_eq!(wanted_climate_kind(&["heizung".into(), "20".into()], cat), Some(ClimateKind::Heat));
        assert_eq!(role_of_tag("wichtig", cat), None);
        assert_eq!(role_of_tag("og", cat), None);
    }

    #[test]
    fn label_ids_resolve_to_names() {
        let names = HashMap::from([("lbl_1".into(), "Licht".into())]);
        let tags = expand_entity_tags(vec!["lbl_1".into()], &names);
        assert!(tags.iter().any(|t| t == "Licht"), "{tags:?}");
        assert!(has_role(
            &EntityRec {
                entity_id: "switch.x".into(),
                name: "X".into(),
                domain: "switch".into(),
                platform: None,
                area: None,
                aliases: Vec::new(),
                tags,
            },
            "light",
            crate::lang::catalog()
        ));
    }
}
