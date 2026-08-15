use crate::expose::assist_visible;
use crate::lang::catalog;
use crate::lexicon::Action;
use crate::normalize::{compact, fold_umlaut};
use crate::types::{EntityRec, HomeGraph};
use std::collections::HashSet;

pub use crate::overlay::{apply_overlay, load_overlay, overlay_path, save_overlay, Overlay};

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
    CompoundSplit { tokens: out, light_areas }
}

pub fn apply_compound_light(home: &HomeGraph, tokens: &[String], light_areas: &[String], resolved: &mut crate::resolve::Resolved) {
    let Some(area) = light_areas.iter().find(|a| resolved.areas.contains(a) || tokens.iter().any(|t| t == *a)).cloned() else {
        return;
    };
    if !resolved.areas.contains(&area) {
        resolved.areas.push(area.clone());
    }
    let named = tokens.iter().any(|t| catalog().named_device.contains(t.as_str()) && !is_light_noun(t));
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
    prefixes.sort_by_key(|(prefix, _)| std::cmp::Reverse(prefix.len()));
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
    } else if rest.len() > 3 && matches!(rest.as_bytes().first(), Some(b'n' | b's')) && is_device_noun(&rest[1..]) {
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
        || catalog().extra_device_nouns.contains(token)
}

fn is_light_noun(token: &str) -> bool {
    let cat = catalog();
    cat.light_nouns.contains(token) || cat.light_singular.contains(token) || token == "beleuchtung"
}

pub use crate::home_policy::{is_infra, is_infra_light};

pub(crate) fn is_tv_switch(domain: &str, entity: &EntityRec) -> bool {
    domain == "media_player" && entity.domain == "switch" && {
        let blob = format!("{} {}", entity.entity_id, compact(&format!("{} {}", entity.name, entity.aliases.join(" "))));
        catalog().tv_words.iter().any(|word| blob.contains(word))
    }
}

pub(crate) fn is_generic_room_light(entity: &EntityRec, home: &HomeGraph) -> bool {
    if !crate::roles::is_light_like(entity) {
        return false;
    }
    let name = compact(&entity.name);
    if matches!(name.as_str(), "licht" | "light" | "lampe" | "lamp" | "leuchte") {
        return true;
    }
    home.areas.iter().any(|area| generic_name(&name, &compact(&area.name)) || generic_name(&name, &compact(&area.area_id)))
}

pub(crate) fn fixture_boost(tokens: &[String], entity: &EntityRec) -> f64 {
    let name = compact(&entity.name);
    if tokens.iter().any(|t| {
        catalog().fixture_alias(t).iter().any(|alias| {
            let a = compact(alias);
            a.len() >= 5 && !catalog().generic.contains(&a.as_str()) && name.contains(&a)
        })
    }) {
        0.94
    } else {
        0.0
    }
}

pub(crate) fn outlet_boost(tokens: &[String], entity: &EntityRec) -> f64 {
    if !catalog().any(tokens, &catalog().outlet_words) {
        return 0.0;
    }
    let id = entity.entity_id.to_ascii_lowercase();
    let name = compact(&entity.name);
    if catalog().outlet_words.iter().any(|word| id.contains(word) || name.contains(word)) {
        0.97
    } else {
        0.0
    }
}

pub(crate) fn short_name_token(entity: &EntityRec) -> Option<String> {
    let cat = catalog();
    entity.name.split(|c: char| !c.is_ascii_alphanumeric()).map(compact).find(|part| {
        part.len() >= 2
            && part.len() <= 3
            && !catalog().generic.contains(&part.as_str())
            && !cat.is_particle(part)
            && !cat.is_filler(part)
            && !matches!(part.as_str(), "von" | "vom" | "of" | "und" | "and")
            && !cat.on_words.contains(part.as_str())
            && !cat.off_words.contains(part.as_str())
    })
}

pub(crate) fn usable_labels(entity: &EntityRec, home: &HomeGraph) -> Vec<String> {
    let generic = is_generic_room_light(entity, home);
    std::iter::once(entity.name.clone())
        .chain(entity.aliases.iter().cloned())
        .chain(entity.tags.iter().filter(|tag| !crate::roles::is_role_tag(tag)).cloned())
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
    if home.areas.iter().any(|area| compact(&area.name) == folded || compact(&area.area_id) == folded) {
        return true;
    }
    if home.entities.iter().any(|other| other.entity_id != entity.entity_id && compact(&other.name) == folded) {
        return true;
    }
    let parts: Vec<String> = label.split(|c: char| !c.is_ascii_alphanumeric()).map(compact).filter(|p| !p.is_empty()).collect();
    !parts.is_empty() && parts.iter().all(|p| catalog().generic.contains(&p.as_str())) && sibling_lights(home, entity) > 0
}

fn sibling_lights(home: &HomeGraph, entity: &EntityRec) -> usize {
    home.entities
        .iter()
        .filter(|other| {
            other.entity_id != entity.entity_id && other.domain == "light" && !is_infra_light(other) && other.area == entity.area
        })
        .count()
}

fn generic_name(name: &str, room: &str) -> bool {
    if room.is_empty() {
        return false;
    }
    let light = name.ends_with("licht") || name.ends_with("light") || name.ends_with("lampe");
    light && (name == format!("{room}licht") || name == format!("{room}light") || name == format!("{room}lampe") || name.starts_with(room))
}

pub(crate) fn named_scene_or_script(tokens: &[String], home: &HomeGraph) -> Option<String> {
    let mentioned = tokens.iter().any(|t| catalog().scene_nouns.contains(t.as_str()) || catalog().script_words.contains(t.as_str()));
    if !mentioned && catalog().any(tokens, &catalog().light_nouns) {
        return None;
    }
    let mut hits: Vec<String> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| matches!(e.domain.as_str(), "scene" | "script"))
        .filter(|e| scene_name_hit(tokens, &e.name, home) || e.aliases.iter().any(|n| scene_name_hit(tokens, n, home)))
        .map(|e| e.entity_id.clone())
        .collect();
    let named = mentioned || catalog().any(tokens, &catalog().scene_named);
    (hits.len() == 1 && (named || tokens.iter().any(|t| t.len() > 5))).then_some(hits.pop()).flatten()
}

fn scene_token(token: &str) -> String {
    let mapped = catalog().scene_token(token);
    if mapped == token {
        fold_umlaut(token)
    } else {
        mapped
    }
}

fn scene_name_hit(tokens: &[String], name: &str, home: &HomeGraph) -> bool {
    let parts: Vec<String> = fold_umlaut(name)
        .split_whitespace()
        .map(scene_token)
        .filter(|p| p.len() > 3 && !catalog().weak_scene.contains(p.as_str()) && scene_distinctive(p, home))
        .collect();
    if parts.is_empty() {
        return false;
    }
    let mapped: Vec<String> = tokens.iter().map(|t| scene_token(t)).collect();
    parts.iter().all(|p| mapped.iter().any(|t| t == p)) || (parts.len() == 1 && parts[0].len() > 5 && mapped.iter().any(|t| t == &parts[0]))
}

fn scene_distinctive(part: &str, home: &HomeGraph) -> bool {
    if catalog().light_nouns.contains(part) || catalog().generic.contains(&part) {
        return false;
    }
    let folded = compact(part);
    !home.areas.iter().any(|a| compact(&a.area_id) == folded || compact(&a.name) == folded)
}

pub(crate) fn room_light_id(home: &HomeGraph, area: &str) -> Option<String> {
    home.entities.iter().find(|e| assist_visible(e, home) && e.entity_id == format!("light.{area}")).map(|e| e.entity_id.clone())
}

pub(crate) fn query_keeps_entity(tokens: &[String], home: &HomeGraph, resolved: &crate::resolve::Resolved, light_areas: &[String]) -> bool {
    if resolved.entities.is_empty() || resolved.areas.len() > 1 {
        return false;
    }
    if !light_areas.is_empty() && resolved.entities.len() == 1 {
        return true;
    }
    let cat = catalog();
    if cat.any(tokens, &cat.named_device) {
        return true;
    }
    if cat.any(tokens, &cat.climate_nouns) {
        return false;
    }
    if cat.any(tokens, &cat.media_nouns) || cat.any(tokens, &cat.outlet_words) {
        return true;
    }
    if room_status_only(tokens, home, resolved) {
        return false;
    }
    if !resolved.areas.is_empty()
        && cat.any(tokens, &cat.light_nouns)
        && !cat.any(tokens, &cat.ceiling)
        && !cat.any(tokens, &cat.lamp_fixture)
        && !cat.any(tokens, &cat.island)
    {
        return false;
    }
    resolved.entities.iter().any(|e| e.domain != "light" || !is_generic_room_light(e, home))
}

fn room_status_only(tokens: &[String], home: &HomeGraph, resolved: &crate::resolve::Resolved) -> bool {
    if resolved.areas.is_empty() || !catalog().any(tokens, &catalog().status_words) {
        return false;
    }
    let cat = catalog();
    let rooms = area_words(home, &resolved.areas);
    tokens.iter().all(|token| {
        cat.is_filler(token)
            || cat.is_particle(token)
            || cat.is_query_hint(token)
            || cat.is_question_word(token)
            || cat.is_question_start(token)
            || catalog().status_words.contains(token.as_str())
            || matches!(token.as_str(), "of" | "the")
            || rooms.contains(token)
    })
}

fn area_words(home: &HomeGraph, areas: &[String]) -> HashSet<String> {
    let mut words = HashSet::new();
    for id in areas {
        let Some(area) = home.areas.iter().find(|area| area.area_id == *id) else {
            continue;
        };
        words.insert(compact(&area.area_id));
        words.insert(compact(&area.name));
        words.extend(area.aliases.iter().map(|alias| compact(alias)));
    }
    words
}

pub(crate) fn area_slots(
    action: Action,
    area: &str,
    domain: Option<&str>,
    home: &HomeGraph,
    tokens: &[String],
) -> (Option<String>, Option<String>, Option<String>) {
    if matches!(action, Action::On | Action::Off | Action::Toggle | Action::SetLight) && domain.is_none_or(|d| d == "light") {
        if let Some(id) = room_light_id(home, area) {
            return (Some(id), None, None);
        }
        if let Some(id) = crate::roles::unique_role_in_area(home, area, "light") {
            return (Some(id), None, None);
        }
        return (None, Some(area.to_string()), Some("light".into()));
    }
    let id = domain
        .filter(|d| matches!(*d, "climate" | "media_player" | "fan"))
        .and_then(|d| crate::resolve::unique_in_area(home, area, d, tokens));
    (id, Some(area.to_string()), domain.map(str::to_string))
}

fn pick_compound_light(home: &HomeGraph, area: &str) -> Option<EntityRec> {
    let room = format!("light.{area}");
    let lights: Vec<&EntityRec> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| e.domain == "light" && !is_infra_light(e) && (e.area.as_deref() == Some(area) || e.entity_id == room))
        .collect();
    if let Some(hit) = lights.iter().find(|e| e.tags.iter().any(|t| t == "preferred")) {
        return Some((*hit).clone());
    }
    if let Some(hit) = lights.iter().find(|e| e.entity_id == room) {
        return Some((*hit).clone());
    }
    let lights: Vec<&EntityRec> = lights.into_iter().filter(|e| !is_generic_room_light(e, home)).collect();
    let named: Vec<&EntityRec> = lights
        .iter()
        .copied()
        .filter(|e| {
            let blob = compact(&format!("{} {}", e.name, e.aliases.join(" ")));
            catalog().named_device.iter().any(|n| blob.contains(n))
        })
        .collect();
    if let Some(hit) = crate::home_policy::preferred_named(&named) {
        return Some(hit.clone());
    }
    (lights.len() == 1).then(|| lights[0].clone())
}
