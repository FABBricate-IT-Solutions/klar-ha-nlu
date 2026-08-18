use crate::home::classify::is_generic_room_light;
use crate::home::policy::is_infra_light;
use crate::lang::catalog;
use crate::parse::fuzzy::{evidence, Evidence, Profile};
use crate::parse::normalize::{compact, fold_umlaut};
use crate::types::{EntityRec, HomeGraph};

use super::token_eq;

pub(super) fn sort_hits(hits: &mut [(f64, EntityRec)], tokens: &[String], home: &HomeGraph) {
    hits.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| overlap(tokens, &b.1, home).cmp(&overlap(tokens, &a.1, home)))
            .then_with(|| a.1.entity_id.cmp(&b.1.entity_id))
    });
}

pub(super) fn overlap(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> usize {
    usable_labels(entity, home)
        .into_iter()
        .map(|label| fold_umlaut(&label).split([' ', '_']).filter(|p| !p.is_empty() && tokens.iter().any(|t| token_eq(t, p))).count())
        .max()
        .unwrap_or(0)
}

pub(super) fn fuzzy_tokens<'a>(tokens: &'a [String], home: &HomeGraph) -> Vec<&'a str> {
    if !tokens.iter().any(|token| catalog().verb(token).is_some()) {
        return Vec::new();
    }
    tokens.iter().map(String::as_str).filter(|token| token.chars().count() >= 5 && !known_target_token(token, home)).collect()
}

pub(crate) fn has_fuzzy_target_token(tokens: &[String], home: &HomeGraph) -> bool {
    tokens.iter().map(String::as_str).filter(|token| token.chars().count() >= 5 && !known_target_token(token, home)).any(|token| {
        home.areas.iter().any(|area| {
            fuzzy_label_token(token, &area.area_id)
                || fuzzy_label_token(token, &area.name)
                || area.aliases.iter().any(|alias| fuzzy_label_token(token, alias))
        }) || home.floors.iter().any(|floor| {
            fuzzy_label_token(token, &floor.floor_id)
                || fuzzy_label_token(token, &floor.name)
                || floor.aliases.iter().any(|alias| fuzzy_label_token(token, alias))
        }) || home.entities.iter().any(|entity| {
            fuzzy_label_token(token, &entity.name)
                || entity.aliases.iter().any(|alias| fuzzy_label_token(token, alias))
                || entity.tags.iter().any(|tag| fuzzy_label_token(token, tag))
        })
    })
}

pub(super) fn score_entity(tokens: &[String], fuzzy_tokens: &[&str], entity: &EntityRec, home: &HomeGraph) -> Option<f64> {
    let labels: Vec<String> = usable_labels(entity, home).into_iter().map(|label| fold_umlaut(&label)).collect();
    let mut best = 0.0_f64;
    for label in labels {
        if label.is_empty() {
            continue;
        }
        if super::token_hit(tokens, &label)
            && !catalog().generic().contains(&label.as_str())
            && !catalog().is_conj(&label)
            && !catalog().is_filler(&label)
        {
            best = best.max(if label.contains(' ') || label.contains('_') { 1.0 } else { 0.94 });
            continue;
        }
        let parts: Vec<&str> = label.split([' ', '_']).filter(|p| !p.is_empty() && !catalog().generic().contains(p)).collect();
        if parts.len() > 1 && parts.iter().all(|p| tokens.iter().any(|t| token_eq(t, p))) {
            best = best.max(0.96);
            continue;
        }
        if let Some(hit) = fuzzy_label_window(tokens, fuzzy_tokens, &label) {
            best = best.max(target_confidence(hit));
        }
        for part in parts {
            if tokens.iter().any(|t| token_eq(t, part) && part.len() > 3) {
                best = best.max(0.9);
            }
            if best >= 0.86 {
                continue;
            }
            for t in fuzzy_tokens {
                if part.len() <= 4 {
                    continue;
                }
                if let Some(hit) = evidence(t, part, Profile::Target) {
                    best = best.max(target_confidence(hit));
                }
            }
        }
    }
    if best < 0.86 {
        if let Some(short) = short_name_token(entity) {
            if tokens.iter().any(|t| t == &short) {
                best = 0.92;
            }
        }
        best = best.max(fixture_boost(tokens, entity));
    }
    best = best.max(outlet_boost(tokens, entity));
    (best >= 0.86).then_some(best)
}

pub(super) fn entity_name_evidence(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> bool {
    score_entity(tokens, &fuzzy_tokens(tokens, home), entity, home).is_some()
        || (entity.domain == "todo"
            && usable_labels(entity, home).iter().any(|label| {
                let label = compact(&fold_umlaut(label));
                !label.is_empty()
                    && tokens.iter().any(|token| {
                        let token = compact(token);
                        catalog().list_nouns().iter().any(|suffix| token == format!("{label}{}", compact(suffix)))
                    })
            }))
}

fn fuzzy_label_window(tokens: &[String], fuzzy_tokens: &[&str], label: &str) -> Option<Evidence> {
    let candidate = compact(label);
    if candidate.len() < 6 {
        return None;
    }
    let max_width = tokens.len().min(3);
    (1..=max_width)
        .flat_map(|width| tokens.windows(width))
        .filter(|window| window.iter().filter(|token| fuzzy_tokens.contains(&token.as_str())).count() == 1)
        .filter_map(|window| evidence(&window.join(""), &candidate, Profile::Target))
        .max_by(|left, right| left.score.partial_cmp(&right.score).unwrap_or(std::cmp::Ordering::Equal))
}

pub(crate) fn known_target_token(token: &str, home: &HomeGraph) -> bool {
    let cat = catalog();
    if cat.generic().contains(token)
        || cat.is_filler(token)
        || cat.is_particle(token)
        || cat.verb(token).is_some()
        || cat.domain_map.contains_key(token)
        || cat.number(token).is_some()
        || cat.color(token).is_some()
    {
        return true;
    }
    home.areas.iter().any(|area| {
        label_has_token(token, &area.area_id)
            || label_has_token(token, &area.name)
            || area.aliases.iter().any(|alias| label_has_token(token, alias))
    }) || home.floors.iter().any(|floor| {
        label_has_token(token, &floor.floor_id)
            || label_has_token(token, &floor.name)
            || floor.aliases.iter().any(|alias| label_has_token(token, alias))
    }) || home.entities.iter().any(|entity| {
        label_has_token(token, &entity.name)
            || entity.aliases.iter().any(|alias| label_has_token(token, alias))
            || entity.tags.iter().any(|tag| label_has_token(token, tag))
    })
}

fn label_has_token(token: &str, label: &str) -> bool {
    fold_umlaut(label).split([' ', '_']).filter(|part| !part.is_empty()).any(|part| token_eq(token, part))
}

fn fuzzy_label_token(token: &str, label: &str) -> bool {
    let folded = fold_umlaut(label);
    evidence(token, &compact(&folded), Profile::Target).is_some()
        || folded.split([' ', '_']).filter(|part| !part.is_empty()).any(|part| evidence(token, part, Profile::Target).is_some())
}

fn target_confidence(hit: Evidence) -> f64 {
    0.86 + hit.score * 0.03
}

fn fixture_boost(tokens: &[String], entity: &EntityRec) -> f64 {
    let name = compact(&entity.name);
    if tokens.iter().any(|t| {
        catalog().fixture_alias(t).iter().any(|alias| {
            let a = compact(alias);
            a.len() >= 5 && !catalog().generic().contains(&a.as_str()) && name.contains(&a)
        })
    }) {
        0.94
    } else {
        0.0
    }
}

fn outlet_boost(tokens: &[String], entity: &EntityRec) -> f64 {
    if !catalog().any(tokens, catalog().outlet_words()) {
        return 0.0;
    }
    let id = entity.entity_id.to_ascii_lowercase();
    let name = compact(&entity.name);
    if catalog().outlet_words().iter().any(|word| id.contains(word) || name.contains(word)) {
        0.97
    } else {
        0.0
    }
}

fn short_name_token(entity: &EntityRec) -> Option<String> {
    let cat = catalog();
    entity.name.split(|c: char| !c.is_ascii_alphanumeric()).map(compact).find(|part| {
        part.len() >= 2
            && part.len() <= 3
            && !catalog().generic().contains(&part.as_str())
            && !cat.is_particle(part)
            && !cat.is_filler(part)
            && !matches!(part.as_str(), "von" | "vom" | "of" | "und" | "and")
            && !cat.on_words().contains(part.as_str())
            && !cat.off_words().contains(part.as_str())
    })
}

fn usable_labels(entity: &EntityRec, home: &HomeGraph) -> Vec<String> {
    let generic = is_generic_room_light(entity, home, catalog());
    std::iter::once(entity.name.clone())
        .chain(entity.aliases.iter().cloned())
        .chain(entity.tags.iter().filter(|tag| !crate::home::roles::is_role_tag(tag, catalog())).cloned())
        .filter(|label| !generic || !stolen_label(label, entity, home))
        .collect()
}

fn stolen_label(label: &str, entity: &EntityRec, home: &HomeGraph) -> bool {
    let folded = compact(label);
    if folded.is_empty() {
        return true;
    }
    if catalog().named_device().iter().any(|n| compact(n) == folded) {
        return true;
    }
    if home.areas.iter().any(|area| compact(&area.name) == folded || compact(&area.area_id) == folded) {
        return true;
    }
    if home.entities.iter().any(|other| other.entity_id != entity.entity_id && compact(&other.name) == folded) {
        return true;
    }
    let parts: Vec<String> = label.split(|c: char| !c.is_ascii_alphanumeric()).map(compact).filter(|p| !p.is_empty()).collect();
    !parts.is_empty() && parts.iter().all(|p| catalog().generic().contains(&p.as_str())) && sibling_lights(home, entity) > 0
}

fn sibling_lights(home: &HomeGraph, entity: &EntityRec) -> usize {
    home.entities
        .iter()
        .filter(|other| {
            other.entity_id != entity.entity_id && other.domain == "light" && !is_infra_light(other) && other.area == entity.area
        })
        .count()
}
