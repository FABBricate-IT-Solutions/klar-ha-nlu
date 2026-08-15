use crate::lang::catalog;
use crate::normalize::compact;
use crate::types::{EntityRec, HomeGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub(crate) const GENERIC: &[&str] = &[
    "licht", "lichter", "lampe", "lampen", "leuchte", "heizung", "thermostat",
    "steckdose", "schalter", "szene", "sensor", "light", "lights", "lamp", "lamps",
    "heater", "heating", "switch", "scene", "ceiling", "decke", "blinds", "rollo",
    "curtain", "curtains", "fan", "luefter", "bedroom", "bedrooms", "kinderzimmer",
    "bathroom", "bath", "door", "tuer", "window", "windows", "fenster", "lock",
    "locks", "schloss", "timer", "kitchen", "living", "dining", "garage", "hallway",
    "laundry", "entryway", "family", "master", "powder", "wohnzimmer", "schlafzimmer",
    "kuche", "kueche", "badezimmer", "flur", "esszimmer",
];

pub struct CompoundSplit {
    pub tokens: Vec<String>,
    pub light_areas: Vec<String>,
}

pub fn expand_compounds(tokens: &[String], home: &HomeGraph) -> CompoundSplit {
    let prefixes = area_prefixes(home);
    let mut out = Vec::new();
    let mut light_areas = Vec::new();
    for token in tokens {
        if let Some((area, noun)) = split_area_device(token, &prefixes) {
            if is_light_noun(&noun) && !light_areas.contains(&area) {
                light_areas.push(area.clone());
            }
            if !out.iter().any(|t| t == &area) {
                out.push(area);
            }
            if !noun.is_empty() {
                out.push(noun);
            }
        } else {
            out.push(token.clone());
        }
    }
    CompoundSplit {
        tokens: out,
        light_areas,
    }
}

pub fn apply_compound_light(
    home: &HomeGraph,
    tokens: &[String],
    light_areas: &[String],
    resolved: &mut crate::resolve::Resolved,
) {
    let Some(area) = light_areas
        .iter()
        .find(|a| resolved.areas.contains(a) || tokens.iter().any(|t| t == *a))
        .cloned()
    else {
        return;
    };
    if !resolved.areas.contains(&area) {
        resolved.areas.push(area.clone());
    }
    let generic = resolved.entities.is_empty()
        || resolved.entities.iter().any(|e| is_generic_room_light(e, home));
    if !generic {
        return;
    }
    if let Some(picked) = pick_compound_light(home, &area) {
        resolved.entities = vec![picked];
        resolved.ambiguous.clear();
    }
}

fn area_prefixes(home: &HomeGraph) -> Vec<(String, String)> {
    let mut prefixes = Vec::new();
    for area in &home.areas {
        prefixes.push((compact(&area.area_id), area.area_id.clone()));
        let name = compact(&area.name);
        if name.len() >= 4 {
            prefixes.push((name, area.area_id.clone()));
        }
        for alias in &area.aliases {
            let folded = compact(alias);
            if folded.len() >= 6 {
                prefixes.push((folded, area.area_id.clone()));
            }
        }
    }
    prefixes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    prefixes.dedup_by(|a, b| a.0 == b.0);
    prefixes
}

fn split_area_device(token: &str, prefixes: &[(String, String)]) -> Option<(String, String)> {
    for (prefix, area) in prefixes {
        if token.len() <= prefix.len() || !token.starts_with(prefix.as_str()) {
            continue;
        }
        let noun = strip_fuge(&token[prefix.len()..]);
        if is_device_noun(noun) {
            return Some((area.clone(), noun.to_string()));
        }
    }
    None
}

fn strip_fuge(rest: &str) -> &str {
    if rest.starts_with("en") && rest.len() > 4 {
        &rest[2..]
    } else if rest.starts_with('n') && rest.len() > 3 && is_device_noun(&rest[1..]) {
        &rest[1..]
    } else if rest.starts_with('s') && rest.len() > 3 && is_device_noun(&rest[1..]) {
        &rest[1..]
    } else {
        rest
    }
}

fn is_device_noun(token: &str) -> bool {
    let cat = catalog();
    cat.light_nouns.contains(token)
        || cat.light_singular.contains(token)
        || cat.climate_nouns.contains(token)
        || cat.fan_nouns.contains(token)
        || cat.cover_nouns.contains(token)
        || cat.media_nouns.contains(token)
        || cat.named_device.contains(token)
        || matches!(token, "steckdose" | "beleuchtung" | "lampe" | "leuchte")
}

fn is_light_noun(token: &str) -> bool {
    let cat = catalog();
    cat.light_nouns.contains(token) || cat.light_singular.contains(token) || token == "beleuchtung"
}

pub(crate) fn is_tv_switch(domain: &str, entity: &EntityRec) -> bool {
    domain == "media_player"
        && entity.domain == "switch"
        && {
            let blob = format!(
                "{} {}",
                entity.entity_id,
                compact(&format!("{} {}", entity.name, entity.aliases.join(" ")))
            );
            blob.contains("tv") || blob.contains("fernseher")
        }
}

pub(crate) fn is_infra_light(entity: &EntityRec) -> bool {
    if entity.domain != "light" {
        return false;
    }
    let id = entity.entity_id.to_ascii_lowercase();
    let name = compact(&entity.name);
    id.contains("led_ring")
        || id.contains("voice_led")
        || id.contains("u7_pro")
        || name.contains("ledring")
        || name.contains("u7pro")
}

pub(crate) fn has_room_light(home: &HomeGraph, area: &str) -> bool {
    home.entities.iter().any(|e| {
        e.domain == "light"
            && (e.entity_id == format!("light.{area}")
                || (is_generic_room_light(e, home) && e.area.as_deref() == Some(area)))
    })
}

pub(crate) fn is_generic_room_light(entity: &EntityRec, home: &HomeGraph) -> bool {
    if entity.domain != "light" {
        return false;
    }
    let name = compact(&entity.name);
    home.areas.iter().any(|area| {
        generic_name(&name, &compact(&area.name)) || generic_name(&name, &compact(&area.area_id))
    })
}

pub(crate) fn usable_labels(entity: &EntityRec, home: &HomeGraph) -> Vec<String> {
    let generic = is_generic_room_light(entity, home);
    std::iter::once(entity.name.clone())
        .chain(entity.aliases.iter().cloned())
        .chain(entity.tags.iter().cloned())
        .filter(|label| !generic || !stolen_label(label, entity, home))
        .collect()
}

fn stolen_label(label: &str, entity: &EntityRec, home: &HomeGraph) -> bool {
    let folded = compact(label);
    if folded.is_empty() {
        return true;
    }
    if catalog().named_device.iter().any(|n| compact(n) == folded) {
        return true;
    }
    if home
        .areas
        .iter()
        .any(|area| compact(&area.name) == folded || compact(&area.area_id) == folded)
    {
        return true;
    }
    if home.entities.iter().any(|other| {
        other.entity_id != entity.entity_id && compact(&other.name) == folded
    }) {
        return true;
    }
    let parts: Vec<String> = label
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(compact)
        .filter(|p| !p.is_empty())
        .collect();
    !parts.is_empty()
        && parts.iter().all(|p| GENERIC.contains(&p.as_str()))
        && sibling_lights(home, entity) > 0
}

fn sibling_lights(home: &HomeGraph, entity: &EntityRec) -> usize {
    home.entities
        .iter()
        .filter(|other| {
            other.entity_id != entity.entity_id
                && other.domain == "light"
                && !is_infra_light(other)
                && other.area == entity.area
        })
        .count()
}

fn generic_name(name: &str, room: &str) -> bool {
    !room.is_empty()
        && (name == format!("{room}licht")
            || name == format!("{room}light")
            || name == format!("{room}lampe"))
}

fn pick_compound_light(home: &HomeGraph, area: &str) -> Option<EntityRec> {
    let lights: Vec<&EntityRec> = home
        .entities
        .iter()
        .filter(|e| e.domain == "light" && e.area.as_deref() == Some(area))
        .filter(|e| !is_generic_room_light(e, home))
        .collect();
    if let Some(hit) = lights.iter().find(|e| e.tags.iter().any(|t| t == "preferred")) {
        return Some((*hit).clone());
    }
    if let Some(hit) = lights
        .iter()
        .find(|e| e.entity_id == format!("light.{area}"))
    {
        return Some((*hit).clone());
    }
    let named: Vec<&EntityRec> = lights
        .iter()
        .copied()
        .filter(|e| {
            let blob = compact(&format!("{} {}", e.name, e.aliases.join(" ")));
            catalog().named_device.iter().any(|n| blob.contains(n))
        })
        .collect();
    if let Some(hit) = named.iter().find(|e| compact(&e.name) == "kugel") {
        return Some((*hit).clone());
    }
    if named.len() == 1 {
        return Some(named[0].clone());
    }
    (lights.len() == 1).then(|| lights[0].clone())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Overlay {
    #[serde(default)]
    pub aliases: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub preferred: Vec<String>,
}

pub fn overlay_path(dir: &Path) -> std::path::PathBuf {
    dir.join("klar_nlu.json")
}

pub fn load_overlay(dir: &Path) -> Overlay {
    let raw = std::fs::read_to_string(overlay_path(dir)).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_overlay(dir: &Path, overlay: &Overlay) -> std::io::Result<()> {
    std::fs::write(overlay_path(dir), serde_json::to_vec_pretty(overlay).unwrap_or_default())
}

pub fn apply_overlay(home: &mut HomeGraph, overlay: &Overlay) {
    for ent in &mut home.entities {
        if let Some(extra) = overlay.aliases.get(&ent.entity_id) {
            for alias in extra {
                if !ent.aliases.iter().any(|a| a == alias) {
                    ent.aliases.push(alias.clone());
                }
            }
        }
        if overlay.preferred.iter().any(|id| id == &ent.entity_id)
            && !ent.tags.iter().any(|t| t == "preferred")
        {
            ent.tags.push("preferred".into());
        }
    }
}
