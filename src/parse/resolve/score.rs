use crate::home::classify::is_generic_room_light;
use crate::home::policy::is_infra_light;
use crate::lang::catalog;
use crate::parse::normalize::{compact, fold_umlaut};
use crate::types::{EntityRec, HomeGraph};
use strsim::normalized_levenshtein;

use super::token_eq;

pub(super) fn sort_hits(hits: &mut [(f64, EntityRec)], tokens: &[String], home: &HomeGraph) {
    hits.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| overlap(tokens, &b.1, home).cmp(&overlap(tokens, &a.1, home)))
    });
}

pub(super) fn overlap(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> usize {
    usable_labels(entity, home)
        .into_iter()
        .map(|label| fold_umlaut(&label).split([' ', '_']).filter(|p| !p.is_empty() && tokens.iter().any(|t| token_eq(t, p))).count())
        .max()
        .unwrap_or(0)
}

pub(super) fn score_entity(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> Option<f64> {
    let labels: Vec<String> = usable_labels(entity, home).into_iter().map(|label| fold_umlaut(&label)).collect();
    let mut best = 0.0_f64;
    for label in labels {
        if label.is_empty() {
            continue;
        }
        if super::token_hit(tokens, &label)
            && !catalog().generic.contains(&label.as_str())
            && !catalog().is_conj(&label)
            && !catalog().is_filler(&label)
        {
            best = best.max(if label.contains(' ') || label.contains('_') { 1.0 } else { 0.94 });
            continue;
        }
        let parts: Vec<&str> = label.split([' ', '_']).filter(|p| !p.is_empty() && !catalog().generic.contains(p)).collect();
        if parts.len() > 1 && parts.iter().all(|p| tokens.iter().any(|t| token_eq(t, p))) {
            best = best.max(0.96);
            continue;
        }
        for part in parts {
            if tokens.iter().any(|t| token_eq(t, part) && part.len() > 3) {
                best = best.max(0.9);
            }
            if best >= 0.86 {
                continue;
            }
            for t in tokens {
                if catalog().generic.contains(&t.as_str()) || part.len() <= 4 {
                    continue;
                }
                let s = normalized_levenshtein(t, part);
                if s > 0.88 {
                    best = best.max(s * 0.9);
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

fn fixture_boost(tokens: &[String], entity: &EntityRec) -> f64 {
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

fn outlet_boost(tokens: &[String], entity: &EntityRec) -> f64 {
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

fn short_name_token(entity: &EntityRec) -> Option<String> {
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

fn usable_labels(entity: &EntityRec, home: &HomeGraph) -> Vec<String> {
    let generic = is_generic_room_light(entity, home);
    std::iter::once(entity.name.clone())
        .chain(entity.aliases.iter().cloned())
        .chain(entity.tags.iter().filter(|tag| !crate::home::roles::is_role_tag(tag)).cloned())
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
