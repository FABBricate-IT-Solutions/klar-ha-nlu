use crate::lang::catalog;
use crate::lexicon::{has_light_noun, is_garage_cover, is_query_token};
use crate::compound::{is_infra, is_tv_switch, short_name_token, usable_labels, GENERIC};
use crate::normalize::{compact, fold_umlaut};
use crate::session::Session;
use crate::types::{AreaRec, EntityRec, HomeGraph};
use strsim::normalized_levenshtein;

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
        .filter(|e| !is_infra(e))
        .filter(|e| domain.is_none_or(|d| e.domain == d || is_tv_switch(d, e)))
        .filter_map(|e| score_entity(tokens, e, home).map(|s| (s, e.clone())))
        .collect();
    if !areas.is_empty() {
        let in_area: Vec<(f64, EntityRec)> = candidates
            .iter()
            .filter(|(_, e)| e.area.as_ref().is_some_and(|a| areas.contains(a)))
            .cloned()
            .collect();
        let named: Vec<(f64, EntityRec)> = candidates
            .iter()
            .filter(|(s, e)| {
                *s >= 0.96 && !e.area.as_ref().is_some_and(|a| areas.contains(a))
            })
            .cloned()
            .collect();
        if !in_area.is_empty() {
            candidates = in_area;
            candidates.extend(named);
        }
    }
    sort_hits(&mut candidates, tokens, home);

    let mut entities = Vec::new();
    let mut ambiguous = Vec::new();
    if let Some((best, rec)) = candidates.first() {
        let best_overlap = overlap(tokens, rec, home);
        let peers: Vec<EntityRec> = candidates
            .iter()
            .filter(|(s, e)| {
                (*s - best).abs() < 0.08
                    && e.entity_id != rec.entity_id
                    && e.name != rec.name
                    && overlap(tokens, e, home) >= best_overlap
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

    if domain.is_none_or(|d| d == "light")
        && tokens.iter().any(|t| matches!(t.as_str(), "decke" | "deckenlampe"))
    {
        let fixtures: Vec<EntityRec> = home
            .entities
            .iter()
            .filter(|e| {
                e.domain == "light"
                    && (e.name.to_lowercase().contains("decke")
                        || e.entity_id.contains("decke")
                        || e.aliases.iter().any(|a| a.contains("decke")))
                    && (areas.is_empty() || e.area.as_ref().is_some_and(|a| areas.contains(a)))
            })
            .cloned()
            .collect();
        if fixtures.len() == 1 {
            return Resolved {
                areas,
                entities: fixtures,
                ambiguous: Vec::new(),
            };
        }
    }

    if domain.is_none_or(|d| d == "light")
        && !catalog().any(tokens, &catalog().timer_nouns)
    {
        if let Some(picked) = pick_fixture(tokens, home, &areas) {
            return Resolved {
                areas,
                entities: picked,
                ambiguous: Vec::new(),
            };
        }
    }
    if !areas.is_empty()
        && catalog().any(tokens, &catalog().lamp_fixture)
        && !catalog().any(tokens, &catalog().light_plural)
        && !tokens.iter().any(|t| matches!(t.as_str(), "licht" | "light"))
    {
        let lights: Vec<EntityRec> = home
            .entities
            .iter()
            .filter(|e| e.domain == "light" && e.area.as_ref().is_some_and(|a| areas.contains(a)))
            .cloned()
            .collect();
        if lights.len() > 1 {
            return Resolved {
                areas,
                entities: Vec::new(),
                ambiguous: lights,
            };
        }
    }

    if entities.is_empty() && ambiguous.is_empty() {
        if let Some(d) = domain {
            let in_domain: Vec<EntityRec> = home
                .entities
                .iter()
                .filter(|e| e.domain == d && !is_infra(e))
                .filter(|e| areas.is_empty() || e.area.as_ref().is_some_and(|a| areas.contains(a)))
                .cloned()
                .collect();
            if in_domain.len() == 1 {
                entities.extend(in_domain);
            }
        }
    }

    Resolved {
        areas,
        entities,
        ambiguous,
    }
}

fn pick_fixture(tokens: &[String], home: &HomeGraph, areas: &[String]) -> Option<Vec<EntityRec>> {
    let cat = catalog();
    let room_level = cat.any(tokens, &cat.light_plural)
        || tokens.iter().any(|t| matches!(t.as_str(), "licht" | "light" | "alle"));
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
        .filter(|e| e.domain == "light")
        .filter(|e| areas.is_empty() || e.area.as_ref().is_some_and(|a| areas.contains(a)))
        .filter(|e| fixture_matches(e, needle))
        .cloned()
        .collect();
    (hits.len() == 1).then_some(hits)
}

fn fixture_matches(entity: &EntityRec, needle: &str) -> bool {
    let blob = format!(
        "{} {} {}",
        entity.entity_id,
        fold_umlaut(&entity.name),
        entity.aliases.join(" ")
    );
    match needle {
        "island" => blob.contains("island") || blob.contains("insel"),
        "pendant" => blob.contains("pendant") || blob.contains("pendel"),
        "right" => blob.contains("right") || blob.contains("rechts"),
        "left" => blob.contains("left") || blob.contains("links"),
        "bedside" => blob.contains("bedside") || blob.contains("nachttisch"),
        "lamp" => {
            (blob.contains("lamp") || blob.contains("lampe"))
                && !blob.contains("ceiling")
                && !blob.contains("decke")
        }
        "ceiling" => blob.contains("ceiling") || blob.contains("decke"),
        _ => false,
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
                let parts = n
                    .split(|c: char| c == ' ' || c == '_')
                    .filter(|p| !p.is_empty())
                    .count()
                    .max(1);
                best = best.max(parts);
            }
        }
        if best > 0 {
            scored.push((best, area.area_id.clone()));
        }
    }
    let max = scored.iter().map(|(s, _)| *s).max().unwrap_or(0);
    let strong: Vec<String> = scored
        .iter()
        .filter(|(s, _)| *s == max)
        .map(|(_, id)| id.clone())
        .collect();
    let mut ids: Vec<String> = scored
        .into_iter()
        .filter(|(s, id)| {
            *s == max
                || !strong.iter().any(|other| other.split('_').next() == id.split('_').next())
        })
        .map(|(_, id)| id)
        .collect();
    if ids.is_empty() && tokens.iter().any(|t| t == "bedroom" || t == "bedrooms")
        && !tokens.iter().any(|t| matches!(t.as_str(), "2" | "3" | "4" | "two" | "three"))
        && areas.iter().any(|a| a.area_id == "master_bedroom")
    {
        return vec!["master_bedroom".into()];
    }
    if ids.len() > 1 {
        ids.retain(|id| id != "wohnung");
    }
    ids
}
fn token_hit(tokens: &[String], label: &str) -> bool {
    if label.is_empty() {
        return false;
    }
    if label.contains(' ') || label.contains('_') {
        let glued = compact(label);
        if glued.len() > 6 && tokens.iter().any(|t| *t == glued) {
            return true;
        }
        let parts: Vec<&str> = label
            .split(|c: char| c == ' ' || c == '_')
            .filter(|p| !p.is_empty())
            .collect();
        return parts.iter().all(|p| tokens.iter().any(|t| token_eq(t, p)));
    }
    tokens.iter().any(|t| token_eq(t, label))
}

fn token_eq(token: &str, label: &str) -> bool {
    if token == label {
        return true;
    }
    if number_word(token) == Some(label) || number_word(label) == Some(token) {
        return true;
    }
    matches!(
        (token, label),
        ("left" | "links", "left" | "links")
            | ("right" | "rechts", "right" | "rechts") | ("globe" | "kugel", "globe" | "kugel")
    )
}

fn number_word(token: &str) -> Option<&'static str> {
    match token {
        "one" | "eins" | "eine" => Some("1"),
        "two" | "zwei" => Some("2"),
        "three" | "drei" => Some("3"),
        "four" | "vier" => Some("4"),
        "five" | "fuenf" => Some("5"),
        "six" | "sechs" => Some("6"),
        "seven" | "sieben" => Some("7"),
        "eight" | "acht" => Some("8"),
        "1" => Some("one"),
        "2" => Some("two"),
        "3" => Some("three"),
        "4" => Some("four"),
        "5" => Some("five"),
        "6" => Some("six"),
        "7" => Some("seven"),
        "8" => Some("eight"),
        _ => None,
    }
}

fn sort_hits(hits: &mut [(f64, EntityRec)], tokens: &[String], home: &HomeGraph) {
    hits.sort_by(|a, b| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then_with(|| {
            overlap(tokens, &b.1, home).cmp(&overlap(tokens, &a.1, home))
        })
    });
}

fn overlap(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> usize {
    usable_labels(entity, home).into_iter().map(|label| {
        fold_umlaut(&label).split(|c: char| c == ' ' || c == '_').filter(|p| !p.is_empty() && tokens.iter().any(|t| token_eq(t, p))).count()
    }).max().unwrap_or(0)
}

fn score_entity(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> Option<f64> {
    let labels: Vec<String> = usable_labels(entity, home)
        .into_iter()
        .map(|label| fold_umlaut(&label))
        .collect();
    let mut best = 0.0_f64;
    for label in labels {
        if label.is_empty() {
            continue;
        }
        if token_hit(tokens, &label) && !GENERIC.contains(&label.as_str()) {
            best = best.max(if label.contains(' ') || label.contains('_') {
                1.0
            } else {
                0.94
            });
            continue;
        }
        let parts: Vec<&str> = label
            .split(|c: char| c == ' ' || c == '_')
            .filter(|p| !p.is_empty() && !GENERIC.contains(p))
            .collect();
        if parts.len() > 1 && parts.iter().all(|p| tokens.iter().any(|t| token_eq(t, p))) {
            best = best.max(0.96);
            continue;
        }
        for part in parts {
            if tokens.iter().any(|t| token_eq(t, part) && part.len() > 3) {
                best = best.max(0.9);
            }
            for t in tokens {
                if GENERIC.contains(&t.as_str()) {
                    continue;
                }
                let s = normalized_levenshtein(t, part);
                if s > 0.88 && part.len() > 3 {
                    best = best.max(s * 0.9);
                }
            }
        }
    }
    if best < 0.86 {
        if let Some(short) = short_name_token(entity) {
            if tokens.iter().any(|t| t == &short) { best = 0.92; }
        }
        best = best.max(crate::compound::fixture_boost(tokens, entity));
    }
    best = best.max(crate::compound::outlet_boost(tokens, entity));
    (best >= 0.86).then_some(best)
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
        if matches!(t.as_str(), "tuer" | "door") {
            if tokens.iter().any(|x| x == "sensor") {
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
            if tokens.iter().any(|x| {
                matches!(x.as_str(), "open" | "close" | "oeffne" | "oeffnen" | "auf" | "zu")
            }) {
                return Some("lock");
            }
            return Some("binary_sensor");
        }
        if matches!(t.as_str(), "fenster" | "window" | "windows") {
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
                    || crate::numbers::first_number(tokens).is_some()
                    || is_query_token(tokens)
                    || tokens.iter().any(|x| catalog().color(x).is_some()));
            let skip_bare_machine = matches!(t.as_str(), "machine" | "appliance" | "geraet")
                && !tokens.iter().any(|x| {
                    matches!(x.as_str(), "laundry" | "washing" | "washer" | "waesche")
                });
            if skip_laundry || skip_bare_machine {
                continue;
            }
        }
        return Some(domain);
    }
    None
}

pub(crate) fn pick_timers(tokens: &[String], home: &HomeGraph) -> Vec<String> {
    let ids: Vec<String> = home.entities.iter().filter(|e| e.domain == "timer").map(|e| e.entity_id.clone()).collect();
    let want = |n: &str| ids.iter().filter(|id| id.contains(n)).cloned().collect::<Vec<_>>();
    if tokens.iter().any(|t| catalog().is_all(t)) {
        return ids.into_iter().filter(|id| !id.contains("abstract")).collect();
    }
    if catalog().any(tokens, &catalog().oven) { return want("oven"); }
    if catalog().any(tokens, &catalog().laundry_timer)
        || crate::numbers::first_number(tokens) == Some(90)
    {
        return want("laundry");
    }
    ids.iter().find(|id| id.contains("abstract")).cloned().into_iter().collect()
}

pub(crate) fn unique_in_area(home: &HomeGraph, area: &str, domain: &str) -> Option<String> {
    let hits: Vec<&str> = home.entities.iter()
        .filter(|e| e.domain == domain && !is_infra(e) && e.area.as_deref() == Some(area))
        .map(|e| e.entity_id.as_str()).collect();
    (hits.len() == 1).then(|| hits[0].to_string())
}

pub(crate) fn query_grounded(
    tokens: &[String],
    home: &HomeGraph,
    has_target: bool,
    session: &Session,
) -> bool {
    if has_target || !session.last_entities.is_empty() || !session.last_areas.is_empty() {
        return true;
    }
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
    {
        return true;
    }
    home.entities.iter().any(|entity| {
        let name = fold_umlaut(&entity.name);
        tokens.iter().any(|token| {
            token.len() > 3
                && !cat.is_question_start(token)
                && !cat.is_question_word(token)
                && (name.split(|c: char| c == ' ' || c == '_').any(|part| part == token)
                    || entity.aliases.iter().any(|alias| alias == token))
        })
    })
}

pub(crate) fn light_rooms_for_clarify(home: &HomeGraph) -> Vec<String> {
    home.areas
        .iter()
        .filter(|area| area.area_id != "wohnung")
        .filter(|area| {
            home.entities.iter().any(|entity| {
                entity.domain == "light" && entity.area.as_deref() == Some(area.area_id.as_str())
            })
        })
        .map(|area| area.area_id.clone())
        .collect()
}
