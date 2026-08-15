use crate::lang::catalog;
use crate::lexicon::Action;
use crate::normalize::{compact, fold_umlaut};
use crate::types::{EntityRec, HomeGraph, Settings};
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
    let named = tokens.iter().any(|t| {
        catalog().named_device.contains(t.as_str()) && !is_light_noun(t)
    });
    if named {
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

pub(crate) fn is_infra(entity: &EntityRec) -> bool {
    is_infra_light(entity) || is_infra_switch(entity)
}

fn is_infra_switch(entity: &EntityRec) -> bool {
    if entity.domain != "switch" {
        return false;
    }
    let id = entity.entity_id.to_ascii_lowercase();
    let name = compact(&entity.name);
    id.contains("r2d2_")
        || id.contains("adaptive_lighting")
        || id.contains("adaptiv_")
        || id.contains("cloud_alexa")
        || id.contains("cloud_google")
        || id.contains("adguard")
        || id.contains("bitte_nicht_storen")
        || id.contains("durchsagen")
        || id.contains("kommunikation")
        || id.contains("child_lock")
        || id.contains("wake_sound")
        || name.contains("klimaanlage")
        || name.contains("adaptive")
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

pub(crate) fn is_generic_room_light(entity: &EntityRec, home: &HomeGraph) -> bool {
    if entity.domain != "light" {
        return false;
    }
    let name = compact(&entity.name);
    home.areas.iter().any(|area| {
        generic_name(&name, &compact(&area.name)) || generic_name(&name, &compact(&area.area_id))
    })
}

pub(crate) fn fixture_boost(tokens: &[String], entity: &EntityRec) -> f64 {
    let name = compact(&entity.name);
    tokens.iter().any(|t| {
        catalog().fixture_alias(t).iter().any(|alias| {
            let a = compact(alias);
            a.len() >= 5 && !GENERIC.contains(&a.as_str()) && name.contains(&a)
        })
    })
    .then_some(0.94)
    .unwrap_or(0.0)
}

pub(crate) fn short_name_token(entity: &EntityRec) -> Option<String> {
    entity
        .name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(compact)
        .find(|part| part.len() >= 2 && part.len() <= 3 && !GENERIC.contains(&part.as_str()))
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

pub(crate) fn named_scene_or_script(tokens: &[String], home: &HomeGraph) -> Option<String> {
    let mentioned = tokens.iter().any(|t| catalog().scene_nouns.contains(t.as_str()) || catalog().script_words.contains(t.as_str()));
    if !mentioned && catalog().any(tokens, &catalog().light_nouns) {
        return None;
    }
    let mut hits: Vec<String> = home.entities.iter().filter(|e| matches!(e.domain.as_str(), "scene" | "script")).filter(|e| {
        scene_name_hit(tokens, &e.name, home) || e.aliases.iter().any(|n| scene_name_hit(tokens, n, home))
    }).map(|e| e.entity_id.clone()).collect();
    let named = mentioned || catalog().any(tokens, &catalog().scene_named);
    (hits.len() == 1 && (named || tokens.iter().any(|t| t.len() > 5))).then_some(hits.pop()).flatten()
}

fn scene_token(token: &str) -> String {
    match token { "movie" => "filmabend".into(), "cozy" => "gemuetlich".into(), other => fold_umlaut(other) }
}

fn scene_name_hit(tokens: &[String], name: &str, home: &HomeGraph) -> bool {
    let parts: Vec<String> = fold_umlaut(name).split_whitespace().map(scene_token)
        .filter(|p| p.len() > 3 && !catalog().weak_scene.contains(p.as_str()) && scene_distinctive(p, home)).collect();
    if parts.is_empty() { return false; }
    let mapped: Vec<String> = tokens.iter().map(|t| scene_token(t)).collect();
    parts.iter().all(|p| mapped.iter().any(|t| t == p))
        || (parts.len() == 1 && parts[0].len() > 5 && mapped.iter().any(|t| t == &parts[0]))
}

fn scene_distinctive(part: &str, home: &HomeGraph) -> bool {
    if catalog().light_nouns.contains(part) || GENERIC.contains(&part) { return false; }
    let folded = compact(part);
    !home.areas.iter().any(|a| compact(&a.area_id) == folded || compact(&a.name) == folded)
}

pub(crate) fn room_light_id(home: &HomeGraph, area: &str) -> Option<String> {
    home.entities.iter().find(|e| e.entity_id == format!("light.{area}")).map(|e| e.entity_id.clone())
}

pub(crate) fn query_keeps_entity(tokens: &[String], home: &HomeGraph, resolved: &crate::resolve::Resolved) -> bool {
    if resolved.entities.is_empty() || resolved.areas.len() > 1 { return false; }
    let cat = catalog();
    if cat.any(tokens, &cat.named_device) { return true; }
    if cat.any(tokens, &cat.climate_nouns) { return false; }
    if cat.any(tokens, &cat.media_nouns) || tokens.iter().any(|t| matches!(t.as_str(), "steckdose" | "outlet")) {
        return true;
    }
    if !resolved.areas.is_empty() && cat.any(tokens, &cat.light_nouns)
        && !cat.any(tokens, &cat.ceiling) && !cat.any(tokens, &cat.lamp_fixture) && !cat.any(tokens, &cat.island)
    {
        return false;
    }
    resolved.entities.iter().any(|e| e.domain != "light" || !is_generic_room_light(e, home))
}

pub(crate) fn area_slots(
    action: Action,
    area: &str,
    domain: Option<&str>,
    home: &HomeGraph,
) -> (Option<String>, Option<String>, Option<String>) {
    if matches!(action, Action::On | Action::Off | Action::Toggle | Action::SetLight)
        && domain.is_none_or(|d| d == "light")
    {
        if let Some(id) = room_light_id(home, area) {
            return (Some(id), None, None);
        }
        return (None, Some(area.to_string()), Some("light".into()));
    }
    let id = domain
        .filter(|d| matches!(*d, "climate" | "media_player" | "fan"))
        .and_then(|d| crate::resolve::unique_in_area(home, area, d));
    (id, Some(area.to_string()), domain.map(str::to_string))
}

fn pick_compound_light(home: &HomeGraph, area: &str) -> Option<EntityRec> {
    let room = format!("light.{area}");
    let lights: Vec<&EntityRec> = home
        .entities
        .iter()
        .filter(|e| {
            e.domain == "light"
                && !is_infra_light(e)
                && (e.area.as_deref() == Some(area) || e.entity_id == room)
        })
        .collect();
    if let Some(hit) = lights.iter().find(|e| e.tags.iter().any(|t| t == "preferred")) {
        return Some((*hit).clone());
    }
    if let Some(hit) = lights.iter().find(|e| e.entity_id == room) {
        return Some((*hit).clone());
    }
    let lights: Vec<&EntityRec> = lights
        .into_iter()
        .filter(|e| !is_generic_room_light(e, home))
        .collect();
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
    #[serde(default)]
    pub areas: HashMap<String, String>,
    #[serde(default)]
    pub settings: Option<Settings>,
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
        if let Some(area) = overlay.areas.get(&ent.entity_id) {
            ent.area = if area.is_empty() { None } else { Some(area.clone()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EntityRec;

    #[test]
    fn overlay_sets_and_clears_area() {
        let mut home = HomeGraph {
            entities: vec![EntityRec {
                entity_id: "light.orphan".into(),
                name: "Hue play 2".into(),
                domain: "light".into(),
                area: None,
                aliases: Vec::new(),
                tags: Vec::new(),
            }],
            ..Default::default()
        };
        apply_overlay(
            &mut home,
            &Overlay {
                areas: [("light.orphan".into(), "wohnzimmer".into())].into(),
                ..Default::default()
            },
        );
        assert_eq!(home.entities[0].area.as_deref(), Some("wohnzimmer"));
        apply_overlay(
            &mut home,
            &Overlay {
                areas: [("light.orphan".into(), String::new())].into(),
                ..Default::default()
            },
        );
        assert_eq!(home.entities[0].area, None);
    }
}
