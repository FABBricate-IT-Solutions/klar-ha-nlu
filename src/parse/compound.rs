use crate::home::classify::is_generic_room_light;
use crate::home::expose::assist_visible;
use crate::home::policy::is_infra_light;
use crate::home::policy::is_whole_home;
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::fuzzy::{evidence, select_unique, Profile};
use crate::parse::normalize::{compact, fold_marks, fold_umlaut, inflected_eq, umlaut_eq};
use crate::types::{EntityRec, HomeGraph};
use std::collections::HashSet;

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
    CompoundSplit { tokens: expand_room_shorts(&out, home), light_areas }
}

/// "Wohn und Esszimmer lichte aus" — short room token next to "und".
fn expand_room_shorts(tokens: &[String], home: &HomeGraph) -> Vec<String> {
    let light_cmd = tokens.iter().any(|token| is_light_noun(token) || catalog().color(token).is_some());
    let exact_action = tokens.iter().any(|token| catalog().verb(token).is_some());
    if !light_cmd || !exact_action || !tokens.iter().any(|token| catalog().is_conj(token)) {
        return tokens.to_vec();
    }
    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            let next_conj = tokens.get(index + 1).is_some_and(|next| catalog().is_conj(next));
            let prev_conj = index > 0 && catalog().is_conj(&tokens[index - 1]);
            if next_conj || prev_conj {
                expand_room_short(token, home).unwrap_or_else(|| token.clone())
            } else {
                token.clone()
            }
        })
        .collect()
}

fn expand_room_short(token: &str, home: &HomeGraph) -> Option<String> {
    if token.len() < 4 || area_already_named(token, home) {
        return None;
    }
    let hits: Vec<String> = home
        .areas
        .iter()
        .filter(|area| !is_whole_home(area))
        .filter(|area| {
            let id = compact(&area.area_id);
            let name = compact(&area.name);
            (id.starts_with(token) && id != token) || (name.starts_with(token) && name != token)
        })
        .map(|area| area.area_id.clone())
        .collect();
    if hits.len() == 1 {
        return hits.into_iter().next();
    }
    if !hits.is_empty() || token.len() < 6 {
        return None;
    }
    let labels: Vec<(String, String)> = home
        .areas
        .iter()
        .filter(|area| !is_whole_home(area))
        .flat_map(|area| {
            std::iter::once(compact(&area.area_id))
                .chain(std::iter::once(compact(&area.name)))
                .chain(area.aliases.iter().map(|alias| compact(alias)))
                .map(|label| (area.area_id.clone(), label))
        })
        .collect();
    select_unique(token, labels.iter().map(|(id, label)| (id.as_str(), label.as_str())), Profile::Target).map(|hit| hit.key.to_string())
}

fn area_already_named(token: &str, home: &HomeGraph) -> bool {
    home.areas.iter().any(|area| {
        let id = compact(&area.area_id);
        let name = compact(&area.name);
        label_hits(token, &id) || label_hits(token, &name) || area.aliases.iter().any(|alias| label_hits(token, &compact(alias)))
    })
}

fn label_hits(token: &str, label: &str) -> bool {
    label == token || inflected_eq(token, label)
}

pub fn apply_compound_light(home: &HomeGraph, tokens: &[String], light_areas: &[String], resolved: &mut crate::parse::resolve::Resolved) {
    let Some(area) = light_areas.iter().find(|a| resolved.areas.contains(a) || tokens.iter().any(|t| t == *a)).cloned() else {
        return;
    };
    if !resolved.areas.contains(&area) {
        resolved.areas.push(area.clone());
    }
    let named = tokens.iter().any(|t| catalog().named_device().contains(t.as_str()) && !is_light_noun(t));
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
        let slug = compact(&fold_marks(&area.name));
        if slug.len() >= 4 && slug != compact(&area.area_id) {
            prefixes.push((slug, area.area_id.clone()));
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
    split_exact_area_device(token, prefixes).or_else(|| split_fuzzy_area_device(token, prefixes))
}

fn split_exact_area_device(token: &str, prefixes: &[(String, String)]) -> Option<(String, String)> {
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

fn split_fuzzy_area_device(token: &str, prefixes: &[(String, String)]) -> Option<(String, String)> {
    if token.chars().count() < 10 {
        return None;
    }
    for noun in compound_heads() {
        let Some(stem) = token.strip_suffix(noun) else {
            continue;
        };
        if stem.chars().count() < 5 {
            continue;
        }
        let hit = select_unique(stem, prefixes.iter().map(|(prefix, area)| (area.as_str(), prefix.as_str())), Profile::Target)?;
        return Some((hit.key.to_string(), noun.to_string()));
    }
    None
}

fn compound_heads() -> Vec<&'static str> {
    let cat = catalog();
    let mut nouns: Vec<&'static str> = cat
        .light_nouns()
        .iter()
        .chain(cat.light_singular().iter())
        .chain(cat.climate_nouns().iter())
        .chain(cat.fan_nouns().iter())
        .chain(cat.cover_nouns().iter())
        .copied()
        .filter(|noun| noun.len() >= 4)
        .collect();
    nouns.sort_unstable_by_key(|noun| std::cmp::Reverse(noun.len()));
    nouns.dedup();
    nouns
}

fn strip_fuge(rest: &str) -> &str {
    for link in catalog().morphology.effective_linking() {
        if rest.len() < link.min_rest_len || !rest.starts_with(link.morpheme) {
            continue;
        }
        let stripped = &rest[link.morpheme.len()..];
        if stripped.is_empty() {
            continue;
        }
        if !link.require_noun || is_device_noun(stripped) {
            return stripped;
        }
    }
    rest
}

fn is_device_noun(token: &str) -> bool {
    let cat = catalog();
    cat.light_nouns().contains(token)
        || cat.light_singular().contains(token)
        || cat.climate_nouns().contains(token)
        || cat.fan_nouns().contains(token)
        || cat.cover_nouns().contains(token)
        || cat.media_nouns().contains(token)
        || cat.named_device().contains(token)
        || catalog().extra_device_nouns().contains(token)
}

fn is_light_noun(token: &str) -> bool {
    let cat = catalog();
    cat.light_nouns().contains(token) || cat.light_singular().contains(token) || token == "beleuchtung"
}

pub(crate) fn named_scene_or_script(tokens: &[String], home: &HomeGraph) -> Option<String> {
    let mentioned = tokens.iter().any(|t| catalog().scene_nouns().contains(t.as_str()) || catalog().script_words().contains(t.as_str()));
    if !mentioned && catalog().any(tokens, catalog().light_nouns()) {
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
    let named = mentioned || catalog().any(tokens, catalog().scene_named());
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
        .filter(|p| p.len() > 3 && !catalog().weak_scene().contains(p.as_str()) && scene_distinctive(p, home))
        .collect();
    if parts.is_empty() {
        return false;
    }
    let mapped: Vec<String> = tokens.iter().map(|t| scene_token(t)).collect();
    let allow_fuzzy = tokens.iter().any(|token| catalog().verb(token).is_some());
    let mut repairs = 0;
    parts.iter().all(|part| {
        if mapped.iter().any(|token| token == part) {
            return true;
        }
        if !allow_fuzzy {
            return false;
        }
        let fuzzy = mapped
            .iter()
            .filter_map(|token| evidence(token, part, Profile::Target))
            .max_by(|left, right| left.score.partial_cmp(&right.score).unwrap_or(std::cmp::Ordering::Equal));
        if fuzzy.is_some() {
            repairs += 1;
        }
        fuzzy.is_some() && repairs <= 1
    })
}

fn scene_distinctive(part: &str, home: &HomeGraph) -> bool {
    if catalog().light_nouns().contains(part) || catalog().generic().contains(&part) {
        return false;
    }
    let folded = compact(part);
    !home.areas.iter().any(|a| compact(&a.area_id) == folded || compact(&a.name) == folded)
}

/// Room-level light bind. `light.{area}` is a room group when named as the room
/// or a generic light; a named fixture on that id is `OccupiedId`.
pub(crate) enum LightAim {
    RoomGroup(String),
    OccupiedId,
    Unique(String),
    AreaLights,
    Clarify,
}

pub(crate) fn room_light_id(home: &HomeGraph, area: &str) -> Option<String> {
    home.entities.iter().find(|e| assist_visible(e, home) && e.entity_id == format!("light.{area}")).map(|e| e.entity_id.clone())
}

pub(crate) fn room_light_standin(home: &HomeGraph, area: &str) -> Option<String> {
    let id = room_light_id(home, area)?;
    let entity = home.entities.iter().find(|entity| entity.entity_id == id)?;
    let name = compact(&entity.name);
    let room = compact(area);
    (name == room || name == format!("{room}licht") || name == format!("{room}light") || is_generic_room_light(entity, home, catalog()))
        .then_some(id)
}

pub(crate) fn light_aim(home: &HomeGraph, area: &str, tokens: &[String]) -> LightAim {
    if let Some(id) = room_light_standin(home, area) {
        return LightAim::RoomGroup(id);
    }
    if room_light_id(home, area).is_some() {
        return LightAim::OccupiedId;
    }
    if let Some(id) = crate::home::roles::unique_role_in_area(home, area, "light", catalog()) {
        return LightAim::Unique(id);
    }
    let cat = catalog();
    let singular = cat.any(tokens, cat.light_singular()) && !cat.any(tokens, cat.light_plural()) && !cat.any(tokens, cat.illuminate());
    if singular && area_light_count(home, area) > 1 {
        return LightAim::Clarify;
    }
    LightAim::AreaLights
}

fn area_light_count(home: &HomeGraph, area: &str) -> usize {
    home.entities
        .iter()
        .filter(|entity| entity.domain == "light" && !is_infra_light(entity) && entity.area.as_deref() == Some(area))
        .count()
}

pub(crate) fn wants_light_clarify(tokens: &[String], home: &HomeGraph, areas: &[String]) -> bool {
    areas.iter().any(|area| matches!(light_aim(home, area, tokens), LightAim::Clarify))
}

pub(crate) fn query_keeps_entity(
    tokens: &[String],
    home: &HomeGraph,
    resolved: &crate::parse::resolve::Resolved,
    light_areas: &[String],
) -> bool {
    if resolved.entities.is_empty() || resolved.areas.len() > 1 {
        return false;
    }
    if !light_areas.is_empty() && resolved.entities.len() == 1 {
        return true;
    }
    let cat = catalog();
    if cat.any(tokens, cat.named_device()) {
        return true;
    }
    if cat.any(tokens, cat.climate_nouns()) {
        return false;
    }
    if cat.any(tokens, cat.media_nouns()) || cat.any(tokens, cat.outlet_words()) {
        return true;
    }
    if room_status_only(tokens, home, resolved) {
        return false;
    }
    if !resolved.areas.is_empty()
        && cat.any(tokens, cat.light_nouns())
        && !cat.any(tokens, cat.ceiling())
        && !cat.any(tokens, cat.lamp_fixture())
        && !cat.any(tokens, cat.island())
    {
        return false;
    }
    resolved.entities.iter().any(|e| e.domain != "light" || !is_generic_room_light(e, home, catalog()))
}

fn room_status_only(tokens: &[String], home: &HomeGraph, resolved: &crate::parse::resolve::Resolved) -> bool {
    if resolved.areas.is_empty() || !catalog().any(tokens, catalog().status_words()) {
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
            || catalog().status_words().contains(token.as_str())
            || matches!(token.as_str(), "of" | "the")
            || rooms.contains(token)
            || rooms.iter().any(|word| umlaut_eq(token, word))
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
        words.insert(compact(&fold_marks(&area.name)));
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
        return match light_aim(home, area, tokens) {
            LightAim::RoomGroup(id) | LightAim::Unique(id) => (Some(id), None, None),
            LightAim::OccupiedId | LightAim::AreaLights | LightAim::Clarify => (None, Some(area.to_string()), Some("light".into())),
        };
    }
    let id = domain
        .filter(|d| matches!(*d, "climate" | "fan" | "media_player" | "lock"))
        .and_then(|d| crate::parse::resolve::unique_in_area(home, area, d, tokens));
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
    if let Some(id) = room_light_standin(home, area) {
        if let Some(hit) = lights.iter().find(|e| e.entity_id == id) {
            return Some((*hit).clone());
        }
    }
    let named: Vec<&EntityRec> = lights
        .iter()
        .copied()
        .filter(|e| !is_generic_room_light(e, home, catalog()))
        .filter(|e| {
            let blob = compact(&format!("{} {}", e.name, e.aliases.join(" ")));
            catalog().named_device().iter().any(|n| blob.contains(n))
        })
        .collect();
    if let Some(hit) = crate::home::policy::preferred_named(&named, catalog()) {
        return Some(hit.clone());
    }
    let specific: Vec<&EntityRec> = lights.iter().copied().filter(|e| !is_generic_room_light(e, home, catalog())).collect();
    if specific.len() == 1 {
        return Some(specific[0].clone());
    }
    (lights.len() == 1).then(|| lights[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;

    #[test]
    fn fuzzy_split_transposes_room_before_licht() {
        let home = default_home();
        let split = expand_compounds(&["wonhzimmerlicht".into()], &home);
        assert!(split.tokens.iter().any(|token| token == "wohnzimmer"), "{:?}", split.tokens);
        assert!(split.tokens.iter().any(|token| token == "licht"), "{:?}", split.tokens);
        assert_eq!(split.light_areas, ["wohnzimmer"]);
    }

    #[test]
    fn fuzzy_split_rejects_unrelated_licht() {
        let home = default_home();
        let split = expand_compounds(&["fensterbanklicht".into()], &home);
        assert_eq!(split.tokens, ["fensterbanklicht"]);
        assert!(split.light_areas.is_empty());
    }
}
