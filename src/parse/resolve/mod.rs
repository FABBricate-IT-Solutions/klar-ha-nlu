use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::home::roles::{is_light_like, matches_domain};
use crate::lang::catalog;
use crate::parse::action::{has_light_noun, is_garage_cover, is_query_token};
use crate::parse::normalize::{compact, fold_umlaut, inflected_eq, is_time_unit};
use crate::types::{AreaRec, EntityRec, HomeGraph};
use score::{overlap, score_entity, sort_hits};

mod score;

#[derive(Debug, Clone)]
pub struct Resolved {
    pub areas: Vec<String>,
    pub entities: Vec<EntityRec>,
    pub ambiguous: Vec<EntityRec>,
}

pub fn resolve(tokens: &[String], home: &HomeGraph, domain: Option<&str>) -> Resolved {
    let areas = match_areas(tokens, &home.areas);
    let mut candidates: Vec<(f64, EntityRec)> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| !is_infra(e))
        .filter(|e| domain.is_none_or(|d| matches_domain(e, d)))
        .filter_map(|e| score_entity(tokens, e, home).map(|s| (s, e.clone())))
        .collect();
    if domain == Some("climate") {
        if let Some(kind) = crate::home::roles::wanted_climate_kind(tokens) {
            candidates.retain(|(_, entity)| crate::home::roles::climate_kind(entity) == Some(kind));
        }
    }
    if !areas.is_empty() {
        let in_area: Vec<(f64, EntityRec)> =
            candidates.iter().filter(|(_, e)| e.area.as_ref().is_some_and(|a| areas.contains(a))).cloned().collect();
        let named: Vec<(f64, EntityRec)> =
            candidates.iter().filter(|(s, e)| *s >= 0.96 && !e.area.as_ref().is_some_and(|a| areas.contains(a))).cloned().collect();
        if !in_area.is_empty() {
            candidates = in_area;
            candidates.extend(named);
        }
    }
    sort_hits(&mut candidates, tokens, home);

    if let Some(named) = crate::parse::resolve_named::collect_named_devices(tokens, home) {
        if named.len() > 1 {
            return Resolved { areas, entities: named, ambiguous: Vec::new() };
        }
    }

    let mut entities = Vec::new();
    let mut ambiguous = Vec::new();
    if let Some((best, rec)) = candidates.first() {
        let best_overlap = overlap(tokens, rec, home);
        let peers: Vec<EntityRec> = candidates
            .iter()
            .filter(|(s, e)| {
                (*s - best).abs() < 0.08 && e.entity_id != rec.entity_id && e.name != rec.name && overlap(tokens, e, home) >= best_overlap
            })
            .map(|(_, e)| e.clone())
            .collect();
        if *best >= 0.86 && peers.is_empty() {
            entities.push(rec.clone());
        } else if *best >= 0.86 && !peers.is_empty() {
            ambiguous.push(rec.clone());
            ambiguous.extend(peers);
        }
    }

    if domain.is_none_or(|d| d == "light") && catalog().any(tokens, &catalog().ceiling) {
        let fixtures: Vec<EntityRec> = home
            .entities
            .iter()
            .filter(|e| assist_visible(e, home))
            .filter(|e| {
                e.domain == "light"
                    && catalog().ceiling.iter().any(|needle| {
                        e.name.to_lowercase().contains(needle)
                            || e.entity_id.contains(needle)
                            || e.aliases.iter().any(|a| a.contains(needle))
                    })
                    && (areas.is_empty() || e.area.as_ref().is_some_and(|a| areas.contains(a)))
            })
            .cloned()
            .collect();
        if fixtures.len() == 1 {
            return Resolved { areas, entities: fixtures, ambiguous: Vec::new() };
        }
    }

    if domain.is_none_or(|d| d == "light") && !catalog().any(tokens, &catalog().timer_nouns) {
        if let Some(picked) = pick_fixture(tokens, home, &areas) {
            return Resolved { areas, entities: picked, ambiguous: Vec::new() };
        }
    }
    if !areas.is_empty()
        && catalog().any(tokens, &catalog().lamp_fixture)
        && !catalog().any(tokens, &catalog().light_plural)
        && !catalog().any(tokens, &catalog().room_level)
    {
        let lights: Vec<EntityRec> = home
            .entities
            .iter()
            .filter(|e| assist_visible(e, home))
            .filter(|e| e.domain == "light" && e.area.as_ref().is_some_and(|a| areas.contains(a)))
            .cloned()
            .collect();
        if lights.len() > 1 {
            return Resolved { areas, entities: Vec::new(), ambiguous: lights };
        }
    }

    if entities.is_empty() && ambiguous.is_empty() {
        if let Some(d) = domain {
            let in_domain: Vec<EntityRec> = home
                .entities
                .iter()
                .filter(|e| assist_visible(e, home))
                .filter(|e| matches_domain(e, d) && !is_infra(e))
                .filter(|e| areas.is_empty() || e.area.as_ref().is_some_and(|a| areas.contains(a)))
                .cloned()
                .collect();
            if in_domain.len() == 1 {
                entities.extend(in_domain);
            }
        }
    }

    Resolved { areas, entities, ambiguous }
}

fn pick_fixture(tokens: &[String], home: &HomeGraph, areas: &[String]) -> Option<Vec<EntityRec>> {
    let cat = catalog();
    let room_level = cat.any(tokens, &cat.light_plural) || cat.any(tokens, &cat.room_level);
    let needle = if cat.any(tokens, &cat.island) {
        Some("island")
    } else if cat.any(tokens, &cat.pendant) {
        Some("pendant")
    } else if cat.any(tokens, &cat.bedside) {
        if cat.any(tokens, &cat.right) {
            Some("right")
        } else if cat.any(tokens, &cat.left) {
            Some("left")
        } else {
            Some("bedside")
        }
    } else if !room_level && cat.any(tokens, &cat.lamp_fixture) {
        Some("lamp")
    } else if cat.any(tokens, &cat.ceiling) {
        Some("ceiling")
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
        matched && !catalog().ceiling.iter().any(|word| blob.contains(word))
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
pub(crate) fn token_hit(tokens: &[String], label: &str) -> bool {
    if label.is_empty() {
        return false;
    }
    if label.contains(' ') || label.contains('_') {
        let glued = compact(label);
        if glued.len() > 6 && tokens.contains(&glued) {
            return true;
        }
        let parts: Vec<&str> = label.split([' ', '_']).filter(|p| !p.is_empty()).collect();
        return parts.iter().all(|p| tokens.iter().any(|t| token_eq(t, p)));
    }
    tokens.iter().any(|t| token_eq(t, label))
}

pub(crate) fn token_eq(token: &str, label: &str) -> bool {
    if token == label {
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
    if cat.any(tokens, &cat.timer_nouns) {
        return Some("timer");
    }
    for t in tokens {
        if *t == "hue" {
            return Some("light");
        }
        if cat.door_nouns.contains(t.as_str()) {
            if cat.any(tokens, &cat.sensor_words) {
                return Some("binary_sensor");
            }
            if tokens.iter().any(|x| cat.lock_verbs.contains(x.as_str())) {
                return Some("lock");
            }
            if cat.any(tokens, &cat.entry_words) {
                return Some("lock");
            }
            if is_garage_cover(tokens) {
                return Some("cover");
            }
            return Some("lock");
        }
        if cat.window_words.contains(t.as_str()) {
            if cat.any(tokens, &cat.sensor_words) {
                return Some("binary_sensor");
            }
            return Some("cover");
        }
        let Some(domain) = cat.domain_map.get(t.as_str()).copied() else {
            continue;
        };
        if domain == "switch" {
            let skip_laundry = cat.laundry_area.contains(t.as_str())
                && (has_light_noun(tokens)
                    || crate::parse::numbers::first_number(tokens).is_some()
                    || is_query_token(tokens)
                    || tokens.iter().any(|x| catalog().color(x).is_some()));
            let skip_bare_machine = cat.bare_switch.contains(t.as_str()) && !cat.any(tokens, &cat.laundry_hint);
            if skip_laundry || skip_bare_machine {
                continue;
            }
        }
        return Some(domain);
    }
    None
}

pub(crate) fn pick_timers(tokens: &[String], home: &HomeGraph) -> Vec<String> {
    let ids: Vec<String> =
        home.entities.iter().filter(|e| assist_visible(e, home) && e.domain == "timer").map(|e| e.entity_id.clone()).collect();
    let want = |n: &str| ids.iter().filter(|id| id.contains(n)).cloned().collect::<Vec<_>>();
    if tokens.iter().any(|t| catalog().is_all(t)) && !tokens.iter().any(|t| is_time_unit(t)) {
        return ids.into_iter().filter(|id| !id.contains("abstract")).collect();
    }
    if catalog().any(tokens, &catalog().oven) {
        return want("oven");
    }
    if catalog().any(tokens, &catalog().laundry_timer)
        || crate::home::policy::timer_hint(home, crate::parse::numbers::first_number(tokens)) == Some("laundry")
    {
        return want("laundry");
    }
    ids.iter().find(|id| id.contains("abstract")).cloned().into_iter().collect()
}

pub(crate) fn unique_in_area(home: &HomeGraph, area: &str, domain: &str, tokens: &[String]) -> Option<String> {
    let kind = (domain == "climate").then(|| crate::home::roles::wanted_climate_kind(tokens)).flatten();
    let hits: Vec<&str> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| matches_domain(e, domain) && !is_infra(e) && e.area.as_deref() == Some(area))
        .filter(|e| kind.is_none_or(|want| crate::home::roles::climate_kind(e) == Some(want)))
        .map(|e| e.entity_id.as_str())
        .collect();
    (hits.len() == 1).then(|| hits[0].to_string())
}

pub(crate) fn climates_of_kind(home: &HomeGraph, tokens: &[String]) -> Vec<String> {
    let kind = crate::home::roles::wanted_climate_kind(tokens);
    home.entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| matches_domain(e, "climate") && !is_infra(e))
        .filter(|e| kind.is_none_or(|want| crate::home::roles::climate_kind(e) == Some(want)))
        .map(|e| e.entity_id.clone())
        .collect()
}

pub(crate) fn query_grounded(tokens: &[String], home: &HomeGraph, has_target: bool) -> bool {
    has_target || mentions_home(tokens, home)
}

pub(crate) fn mentions_home(tokens: &[String], home: &HomeGraph) -> bool {
    let cat = catalog();
    if cat.any(tokens, &cat.temp_query)
        || cat.any(tokens, &cat.light_nouns)
        || cat.any(tokens, &cat.climate_nouns)
        || cat.any(tokens, &cat.cover_nouns)
        || cat.any(tokens, &cat.fan_nouns)
        || cat.any(tokens, &cat.lock_nouns)
        || cat.any(tokens, &cat.vacuum_nouns)
        || cat.any(tokens, &cat.media_nouns)
        || cat.any(tokens, &cat.timer_nouns)
        || cat.any(tokens, &cat.list_nouns)
        || cat.any(tokens, &cat.scene_nouns)
        || cat.any(tokens, &cat.named_device)
        || cat.any(tokens, &cat.on_words)
        || cat.any(tokens, &cat.off_words)
        || cat.any(tokens, &cat.laundry_machines)
        || cat.any(tokens, &cat.status_words)
    {
        return true;
    }
    if home.entities.iter().any(crate::home::roles::is_music_player)
        && (tokens.windows(2).any(|w| matches!((w[0].as_str(), w[1].as_str()), ("was", "laeuft") | ("was", "spielt")))
            || tokens.windows(3).any(|w| matches!((w[0].as_str(), w[1].as_str(), w[2].as_str()), ("what", "s", "playing")))
            || tokens.iter().any(|t| matches!(t.as_str(), "queue" | "warteschlange")))
    {
        return true;
    }
    if home.areas.iter().any(|area| {
        std::iter::once(compact(&area.area_id))
            .chain(std::iter::once(compact(&area.name)))
            .chain(area.aliases.iter().map(|alias| compact(alias)))
            .any(|name| !name.is_empty() && tokens.iter().any(|token| token == &name))
    }) {
        return true;
    }
    home.entities.iter().filter(|entity| assist_visible(entity, home)).any(|entity| {
        let name = fold_umlaut(&entity.name);
        tokens.iter().any(|token| {
            token.len() > 3
                && !cat.is_question_start(token)
                && !cat.is_question_word(token)
                && (name.split([' ', '_']).any(|part| part == token) || entity.aliases.iter().any(|alias| alias == token))
        })
    })
}

pub(crate) fn light_rooms_for_clarify(home: &HomeGraph) -> Vec<String> {
    home.areas
        .iter()
        .filter(|area| !crate::home::policy::is_whole_home(area))
        .filter(|area| {
            home.entities.iter().any(|entity| {
                assist_visible(entity, home) && is_light_like(entity) && entity.area.as_deref() == Some(area.area_id.as_str())
            })
        })
        .map(|area| area.area_id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AreaRec, EntityRec, HomeGraph};

    fn lamp(id: &str, name: &str, area: &str) -> EntityRec {
        EntityRec {
            entity_id: id.into(),
            name: name.into(),
            domain: "light".into(),
            platform: None,
            area: Some(area.into()),
            aliases: vec![name.to_ascii_lowercase()],
            tags: Vec::new(),
        }
    }

    #[test]
    fn resolve_skips_entities_not_exposed_to_assist() {
        let mut home = HomeGraph {
            areas: vec![AreaRec { area_id: "kuche".into(), name: "Küche".into(), aliases: vec!["kueche".into()] }],
            entities: vec![lamp("light.hidden", "Geheimlampe", "kuche"), lamp("light.kuche_kuche", "Deckenlampe", "kuche")],
            assist: Some(["light.kuche_kuche".into()].into()),
            ..Default::default()
        };
        let hit = resolve(&["deckenlampe".into()], &home, Some("light"));
        assert_eq!(hit.entities.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["light.kuche_kuche"]);
        home.assist = Some(["light.hidden".into()].into());
        let hidden_only = resolve(&["geheimlampe".into()], &home, Some("light"));
        assert_eq!(hidden_only.entities.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["light.hidden"]);
        home.assist = Some(["light.kuche_kuche".into()].into());
        assert_eq!(crate::parse::compound::room_light_id(&home, "kuche"), None);
    }
}
