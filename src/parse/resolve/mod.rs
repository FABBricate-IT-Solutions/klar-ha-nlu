use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::home::roles::{looks_like_tv, matches_domain, tv_asked};
use crate::lang::catalog;
use crate::parse::action::{has_light_noun, is_garage_cover, is_query_token};
use crate::parse::fuzzy::{evidence, Profile};
use crate::parse::normalize::{compact, fold_umlaut, inflected_eq, is_time_unit, umlaut_eq};
use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph};
pub(crate) use report::{resolve_scored, ResolveEvidence, ResolveReport};
use score::{fuzzy_tokens, overlap, score_entity, sort_hits};
pub(crate) use score::{has_fuzzy_target_token, known_target_token};

mod prefer;
mod report;
mod scope;
mod score;
pub(crate) use scope::{light_rooms_for_clarify, mentions_home, query_grounded};

#[derive(Debug, Clone)]
pub struct Resolved {
    pub areas: Vec<String>,
    pub floors: Vec<String>,
    pub entities: Vec<EntityRec>,
    pub ambiguous: Vec<EntityRec>,
}

pub fn resolve(tokens: &[String], home: &HomeGraph, domain: Option<&str>) -> Resolved {
    let mut areas = match_areas(tokens, &home.areas);
    let floors = match_floors(tokens, &home.floors);
    if !floors.is_empty() && areas.len() == 1 {
        let on_floor: Vec<String> = areas
            .iter()
            .filter(|area_id| {
                home.areas
                    .iter()
                    .any(|area| area.area_id.as_str() == area_id.as_str() && area.floor_id.as_ref().is_some_and(|id| floors.contains(id)))
            })
            .cloned()
            .collect();
        if !on_floor.is_empty() {
            areas = on_floor;
        }
    }
    let floor_areas: Vec<String> =
        floors.iter().flat_map(|floor_id| home.areas_on_floor(floor_id).map(|area| area.area_id.clone())).collect();
    let scope = if areas.is_empty() { floor_areas.as_slice() } else { areas.as_slice() };
    let fuzzy_tokens = fuzzy_tokens(tokens, home);
    let mut candidates: Vec<(f64, EntityRec)> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| !is_infra(e))
        .filter(|e| !crate::home::policy::is_nlu_ignored(e))
        .filter(|e| !crate::parse::calendar::is_calendar_control(e))
        .filter(|e| domain.is_none_or(|d| matches_domain(e, d, catalog())))
        .filter_map(|e| score_entity(tokens, &fuzzy_tokens, e, home).map(|s| (s, e.clone())))
        .collect();
    if domain == Some("climate") {
        if let Some(kind) = crate::home::roles::wanted_climate_kind(tokens, catalog()) {
            candidates.retain(|(_, entity)| crate::home::roles::climate_kind(entity, catalog()) == Some(kind));
        }
    }
    if !scope.is_empty() {
        let in_area: Vec<(f64, EntityRec)> =
            candidates.iter().filter(|(_, e)| e.area.as_ref().is_some_and(|a| scope.contains(a))).cloned().collect();
        let named: Vec<(f64, EntityRec)> =
            candidates.iter().filter(|(s, e)| *s >= 0.96 && !e.area.as_ref().is_some_and(|a| scope.contains(a))).cloned().collect();
        if !in_area.is_empty() {
            candidates = in_area;
            candidates.extend(named);
        }
    }
    prefer::prefer_entry_lock(tokens, home, &mut candidates);
    prefer::prefer_tv(tokens, home, &mut candidates);
    if crate::parse::compound::named_scene_or_script(tokens, home).is_none()
        && !catalog().any(tokens, catalog().scene_nouns())
        && !catalog().any(tokens, catalog().script_words())
    {
        candidates.retain(|(_, entity)| entity.domain != "scene" && entity.domain != "script");
    }
    sort_hits(&mut candidates, tokens, home);
    if let Some(locks) = prefer::mentioned_locks(tokens, &candidates) {
        return Resolved { areas, floors, entities: locks, ambiguous: Vec::new() };
    }

    if let Some(named) = crate::parse::resolve_named::collect_named_devices(tokens, home) {
        let scoped: Vec<EntityRec> = if scope.is_empty() {
            named
        } else {
            named.into_iter().filter(|entity| entity.area.as_ref().is_some_and(|area| scope.contains(area))).collect()
        };
        if scoped.len() > 1 {
            return Resolved { areas, floors, entities: scoped, ambiguous: Vec::new() };
        }
    }

    let mut entities = Vec::new();
    let mut ambiguous = Vec::new();
    if let Some((best, rec)) = candidates.first() {
        let best_overlap = overlap(tokens, rec, home);
        let peers: Vec<EntityRec> = candidates
            .iter()
            .filter(|(s, e)| (*s - best).abs() < 0.08 && e.entity_id != rec.entity_id && overlap(tokens, e, home) >= best_overlap)
            .map(|(_, e)| e.clone())
            .collect();
        if *best >= 0.86 && peers.is_empty() {
            let generic_light = rec.domain == "light"
                && catalog().any(tokens, catalog().light_nouns())
                && !catalog().any(tokens, catalog().named_device())
                && !catalog().any(tokens, catalog().ceiling())
                && !catalog().any(tokens, catalog().island())
                && !catalog().any(tokens, catalog().bedside())
                && !catalog().any(tokens, catalog().lamp_fixture());
            let crowded = rec.area.as_deref().is_some_and(|area| {
                home.entities
                    .iter()
                    .filter(|entity| {
                        assist_visible(entity, home)
                            && entity.domain == "light"
                            && !is_infra(entity)
                            && entity.area.as_deref() == Some(area)
                    })
                    .count()
                    > 1
            });
            if distinctive_light_name(tokens, rec, home) || !(generic_light && crowded) {
                entities.push(rec.clone());
            }
        } else if *best >= 0.86 && !peers.is_empty() {
            ambiguous.push(rec.clone());
            ambiguous.extend(peers);
        }
    }

    if domain.is_none_or(|d| d == "light") && catalog().any(tokens, catalog().ceiling()) {
        let needle = tokens.iter().find(|t| catalog().ceiling().contains(t.as_str())).map(String::as_str).unwrap_or("ceiling");
        let fixtures: Vec<EntityRec> = home
            .entities
            .iter()
            .filter(|e| assist_visible(e, home))
            .filter(|e| {
                e.domain == "light"
                    && fixture_matches(e, needle)
                    && (scope.is_empty() || e.area.as_ref().is_some_and(|a| scope.contains(a)))
            })
            .cloned()
            .collect();
        if fixtures.len() == 1 {
            return Resolved { areas, floors, entities: fixtures, ambiguous: Vec::new() };
        }
    }

    if domain.is_none_or(|d| d == "light") && !catalog().any(tokens, catalog().timer_nouns()) {
        if let Some(picked) = pick_fixture(tokens, home, scope) {
            return Resolved { areas, floors, entities: picked, ambiguous: Vec::new() };
        }
    }
    if !areas.is_empty()
        && catalog().any(tokens, catalog().lamp_fixture())
        && !catalog().any(tokens, catalog().light_plural())
        && !catalog().any(tokens, catalog().room_level())
    {
        let lights: Vec<EntityRec> = home
            .entities
            .iter()
            .filter(|e| assist_visible(e, home))
            .filter(|e| e.domain == "light" && e.area.as_ref().is_some_and(|a| scope.contains(a)))
            .cloned()
            .collect();
        if lights.len() > 1 {
            return Resolved { areas, floors, entities: Vec::new(), ambiguous: lights };
        }
    }

    if entities.is_empty() && ambiguous.is_empty() {
        if let Some(d) = domain {
            let in_domain: Vec<EntityRec> = home
                .entities
                .iter()
                .filter(|e| assist_visible(e, home))
                .filter(|e| matches_domain(e, d, catalog()) && !is_infra(e))
                .filter(|e| scope.is_empty() || e.area.as_ref().is_some_and(|a| scope.contains(a)))
                .filter(|e| d != "media_player" || !tv_asked(tokens) || looks_like_tv(e))
                .cloned()
                .collect();
            if in_domain.len() == 1 {
                entities.extend(in_domain);
            }
        }
    }

    Resolved { areas, floors, entities, ambiguous }
}

pub(crate) fn entity_has_name_evidence(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> bool {
    assist_visible(entity, home) && !is_infra(entity) && entity_name_is_mentioned(tokens, entity, home)
}

pub(crate) fn entity_name_is_mentioned(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> bool {
    score::entity_name_evidence(tokens, entity, home)
}

fn distinctive_light_name(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> bool {
    let cat = catalog();
    let area = entity.area.as_deref().and_then(|id| home.areas.iter().find(|area| area.area_id == id));
    let room = area.map(|area| {
        std::iter::once(area.area_id.as_str())
            .chain(std::iter::once(area.name.as_str()))
            .chain(area.aliases.iter().map(String::as_str))
            .map(compact)
            .collect::<Vec<_>>()
    });
    let room = room.unwrap_or_default();
    let blob = compact(&format!("{} {}", entity.name, entity.aliases.join(" ")));
    tokens.iter().any(|token| {
        let folded = compact(token);
        folded.len() >= 4
            && !cat.light_nouns().contains(token.as_str())
            && !cat.light_singular().contains(token.as_str())
            && !room.iter().any(|name| name == &folded || name.contains(&folded) || folded.contains(name.as_str()))
            && (blob.contains(&folded)
                || entity.entity_id.contains(&folded)
                || evidence(token, &blob, Profile::Target).is_some()
                || entity
                    .name
                    .split([' ', '_'])
                    .chain(entity.entity_id.split(['.', '_']))
                    .any(|part| evidence(token, part, Profile::Target).is_some()))
    })
}

fn pick_fixture(tokens: &[String], home: &HomeGraph, areas: &[String]) -> Option<Vec<EntityRec>> {
    let cat = catalog();
    let room_level = cat.any(tokens, cat.light_plural()) || cat.any(tokens, cat.room_level());
    let needle = if cat.any(tokens, cat.island()) {
        Some("island")
    } else if cat.any(tokens, cat.pendant()) {
        Some("pendant")
    } else if cat.any(tokens, cat.bedside()) {
        if cat.any(tokens, cat.right()) {
            Some("right")
        } else if cat.any(tokens, cat.left()) {
            Some("left")
        } else {
            Some("bedside")
        }
    } else if tokens.iter().any(|token| token == "floor" || cat.fixture_alias("floor").contains(&token.as_str())) {
        Some("floor")
    } else if !room_level && cat.any(tokens, cat.lamp_fixture()) {
        Some("lamp")
    } else if let Some(word) = tokens.iter().find(|t| cat.ceiling().contains(t.as_str())) {
        Some(word.as_str())
    } else {
        None
    }?;
    let hits: Vec<EntityRec> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| e.domain == "light")
        .filter(|e| areas.is_empty() || e.area.as_ref().is_some_and(|a| areas.contains(a)))
        .filter(|e| fixture_matches(e, needle))
        .cloned()
        .collect();
    (hits.len() == 1).then_some(hits)
}

pub(crate) fn fixture_matches(entity: &EntityRec, needle: &str) -> bool {
    let blob = format!("{} {} {}", entity.entity_id, fold_umlaut(&entity.name), entity.aliases.join(" "));
    let aliases = catalog().fixture_alias(needle);
    let hits: Vec<&str> = if aliases.is_empty() { vec![needle] } else { aliases.to_vec() };
    let matched = hits.iter().any(|alias| blob.contains(alias));
    if needle == "lamp" {
        matched && !catalog().ceiling().iter().any(|word| blob.contains(word))
    } else {
        matched
    }
}

fn match_areas(tokens: &[String], areas: &[AreaRec]) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = Vec::new();
    for area in areas {
        let names: Vec<String> = std::iter::once(fold_umlaut(&area.name))
            .chain(std::iter::once(area.area_id.clone()))
            .chain(area.aliases.iter().map(|a| fold_umlaut(a)))
            .collect();
        let mut best = 0usize;
        for n in &names {
            if token_hit(tokens, n) {
                let parts = n.split([' ', '_']).filter(|p| !p.is_empty()).count().max(1);
                best = best.max(parts);
            }
        }
        if best > 0 {
            scored.push((best, area.area_id.clone()));
        }
    }
    let max = scored.iter().map(|(s, _)| *s).max().unwrap_or(0);
    let strong: Vec<String> = scored.iter().filter(|(s, _)| *s == max).map(|(_, id)| id.clone()).collect();
    let mut ids: Vec<String> = scored
        .into_iter()
        .filter(|(s, id)| *s == max || !strong.iter().any(|other| other.split('_').next() == id.split('_').next()))
        .map(|(_, id)| id)
        .collect();
    if ids.is_empty() {
        if let Some(fuzzy) = crate::parse::resolve_named::fuzzy_areas(tokens, areas) {
            return fuzzy;
        }
    }
    if ids.is_empty() && crate::home::policy::mentions_generic_bedroom(tokens) {
        if let Some(bedroom) = crate::home::policy::primary_bedroom(&HomeGraph { areas: areas.to_vec(), ..Default::default() }) {
            return vec![bedroom];
        }
    }
    if ids.len() > 1 {
        ids.retain(|id| !areas.iter().any(|area| area.area_id == *id && crate::home::policy::is_whole_home(area)));
    }
    ids
}

fn match_floors(tokens: &[String], floors: &[FloorRec]) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = Vec::new();
    for floor in floors {
        let names: Vec<String> = std::iter::once(fold_umlaut(&floor.name))
            .chain(std::iter::once(floor.floor_id.clone()))
            .chain(floor.aliases.iter().map(|alias| fold_umlaut(alias)))
            .collect();
        let mut best = 0usize;
        for name in &names {
            if token_hit(tokens, name) {
                best = best.max(name.split([' ', '_']).filter(|part| !part.is_empty()).count().max(1));
            }
        }
        if best > 0 {
            scored.push((best, floor.floor_id.clone()));
        }
    }
    let max = scored.iter().map(|(score, _)| *score).max().unwrap_or(0);
    scored.into_iter().filter(|(score, _)| *score == max && max > 0).map(|(_, id)| id).collect()
}
pub(crate) fn token_hit(tokens: &[String], label: &str) -> bool {
    if label.is_empty() {
        return false;
    }
    let compact_label = compact(label);
    if tokens.iter().any(|token| token_eq(token, label)) {
        return true;
    }
    if !compact_label.is_empty()
        && tokens.iter().any(|token| {
            let folded = compact(token);
            !folded.is_empty() && (folded == compact_label || umlaut_eq(&folded, &compact_label))
        })
    {
        return true;
    }
    if label.contains(' ') || label.contains('_') || label.contains('-') {
        let parts: Vec<&str> = label.split([' ', '_', '-']).filter(|part| !part.is_empty()).collect();
        return !parts.is_empty() && parts.iter().all(|part| tokens.iter().any(|token| token_eq(token, part)));
    }
    false
}

pub(crate) fn token_eq(token: &str, label: &str) -> bool {
    if token == label || umlaut_eq(token, label) {
        return true;
    }
    if number_word(token).as_deref() == Some(label) || number_word(label).as_deref() == Some(token) {
        return true;
    }
    if inflected_eq(token, label) {
        return true;
    }
    catalog().synonyms(token).any(|alias| alias == label) || catalog().synonyms(label).any(|alias| alias == token)
}

fn number_word(token: &str) -> Option<String> {
    if let Some(n) = catalog().number(token) {
        return Some(n.to_string());
    }
    token.parse::<i32>().ok().and_then(|n| catalog().number_word(n).map(str::to_string))
}

pub fn domain_hint(tokens: &[String]) -> Option<&'static str> {
    let cat = catalog();
    if cat.any(tokens, cat.timer_nouns()) {
        return Some("timer");
    }
    // Generic "device/appliance" in the laundry room is washer-vs-dryer, not lights.
    if cat.any(tokens, cat.laundry_area())
        && cat.any(tokens, cat.extra_device_nouns())
        && !has_light_noun(tokens)
        && !cat.any(tokens, cat.laundry_machines())
        && !cat.any(tokens, cat.skip_light())
    {
        return Some("switch");
    }
    for t in tokens {
        if *t == "hue" {
            return Some("light");
        }
        if cat.door_nouns().contains(t.as_str()) {
            if cat.any(tokens, cat.sensor_words()) {
                return Some("binary_sensor");
            }
            if tokens.iter().any(|x| cat.lock_verbs().contains(x.as_str())) {
                return Some("lock");
            }
            if cat.any(tokens, cat.entry_words()) {
                return Some("lock");
            }
            if is_garage_cover(tokens) {
                return Some("cover");
            }
            return Some("lock");
        }
        if cat.window_words().contains(t.as_str()) {
            if cat.any(tokens, cat.sensor_words()) {
                return Some("binary_sensor");
            }
            return Some("cover");
        }
        let Some(domain) = cat.domain_map.get(t.as_str()).copied() else {
            continue;
        };
        if domain == "switch" {
            let skip_laundry = cat.laundry_area().contains(t.as_str())
                && (has_light_noun(tokens)
                    || crate::parse::numbers::first_number(tokens).is_some()
                    || is_query_token(tokens)
                    || tokens.iter().any(|x| catalog().color(x).is_some()));
            let skip_bare_machine = cat.bare_switch().contains(t.as_str()) && !cat.any(tokens, cat.laundry_hint());
            if skip_laundry || skip_bare_machine {
                continue;
            }
        }
        return Some(domain);
    }
    cat.fuzzy_domain(tokens)
}

pub(crate) fn pick_timers(tokens: &[String], home: &HomeGraph) -> Vec<String> {
    let ids: Vec<String> =
        home.entities.iter().filter(|e| assist_visible(e, home) && e.domain == "timer").map(|e| e.entity_id.clone()).collect();
    let want = |n: &str| ids.iter().filter(|id| id.contains(n)).cloned().collect::<Vec<_>>();
    if tokens.iter().any(|t| catalog().is_all(t)) && !tokens.iter().any(|t| is_time_unit(t)) {
        return ids.into_iter().filter(|id| !id.contains("abstract")).collect();
    }
    if catalog().any(tokens, catalog().oven()) {
        return want("oven");
    }
    if catalog().any(tokens, catalog().laundry_timer())
        || crate::home::policy::timer_hint(home, crate::parse::numbers::first_number(tokens)) == Some("laundry")
    {
        return want("laundry");
    }
    ids.iter().find(|id| id.contains("abstract")).cloned().into_iter().collect()
}

pub(crate) fn unique_in_area(home: &HomeGraph, area: &str, domain: &str, tokens: &[String]) -> Option<String> {
    let kind = (domain == "climate").then(|| crate::home::roles::wanted_climate_kind(tokens, catalog())).flatten();
    let hits: Vec<&str> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| matches_domain(e, domain, catalog()) && !is_infra(e) && e.area.as_deref() == Some(area))
        .filter(|e| kind.is_none_or(|want| crate::home::roles::climate_kind(e, catalog()) == Some(want)))
        .map(|e| e.entity_id.as_str())
        .collect();
    (hits.len() == 1).then(|| hits[0].to_string())
}

pub(crate) fn climates_of_kind(home: &HomeGraph, tokens: &[String]) -> Vec<String> {
    let kind = crate::home::roles::wanted_climate_kind(tokens, catalog());
    home.entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| matches_domain(e, "climate", catalog()) && !is_infra(e))
        .filter(|e| kind.is_none_or(|want| crate::home::roles::climate_kind(e, catalog()) == Some(want)))
        .map(|e| e.entity_id.clone())
        .collect()
}

#[cfg(test)]
mod tests;
