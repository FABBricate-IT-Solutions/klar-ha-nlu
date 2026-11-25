//! Longest device labels first: find name spans in the sentence, consume those tokens, and
//! leave verbs, numbers and colours for the rest of the parse.
//!
//! A full name beats a domain hint and "room plus noun". When several devices share a label,
//! a spoken floor keeps only the ones on that floor.

use super::{token_eq, token_hit};
use crate::home::expose::assist_visible;
use crate::home::policy::{is_infra, is_nlu_ignored};
use crate::lang::catalog;
use crate::parse::normalize::{compact, fold_label, umlaut_eq};
use crate::types::{EntityRec, HomeGraph};
use std::collections::BTreeSet;

/// Where a device name was said: first token index and how many tokens it took.
type Span = (usize, usize);

#[derive(Debug, Clone)]
struct Hit {
    entity: EntityRec,
    parts: usize,
    chars: usize,
    span: Span,
}

/// Devices whose whole label sits in the sentence, plus the token indices that label used.
#[derive(Debug, Clone, Default)]
pub(crate) struct LinkedNames {
    pub entities: Vec<EntityRec>,
    pub ambiguous: Vec<EntityRec>,
    pub consumed: BTreeSet<usize>,
}

/// Longest non-overlapping name spans. One clear name becomes `entities`; several devices
/// under the same span become `ambiguous` unless a floor narrows them to one.
pub(crate) fn link_named_spans(tokens: &[String], home: &HomeGraph, floors: &[String]) -> LinkedNames {
    let mut hits: Vec<Hit> = Vec::new();
    for entity in home.entities.iter().filter(|e| {
        assist_visible(e, home)
            && !is_infra(e)
            && !is_nlu_ignored(e)
            // Scenes and scripts have their own matcher; a shared word must not link them here.
            && e.domain != "scene"
            && e.domain != "script"
    }) {
        for label in usable_labels(entity) {
            if !linkable(&label, home) {
                continue;
            }
            let parts = word_count(&label);
            let chars = compact(&label).chars().count();
            for span in label_spans(tokens, &label) {
                hits.push(Hit { entity: entity.clone(), parts, chars, span });
            }
        }
    }
    hits.sort_by(|left, right| {
        right
            .parts
            .cmp(&left.parts)
            .then_with(|| right.chars.cmp(&left.chars))
            .then_with(|| left.span.0.cmp(&right.span.0))
            .then_with(|| left.entity.entity_id.cmp(&right.entity.entity_id))
    });

    let mut taken: Vec<Span> = Vec::new();
    let mut by_span: Vec<(Span, usize, usize)> = Vec::new();
    for hit in hits {
        if taken.iter().any(|span| overlaps(hit.span, *span)) {
            continue;
        }
        taken.push(hit.span);
        by_span.push((hit.span, hit.parts, hit.chars));
    }

    let by_span: Vec<(Span, Vec<EntityRec>)> =
        by_span.into_iter().map(|(span, parts, chars)| (span, entities_at_span(tokens, home, span, parts, chars))).collect();

    let mut linked = LinkedNames::default();
    for (span, mut group) in by_span {
        if !floors.is_empty() {
            let on_floor: Vec<EntityRec> = group.iter().filter(|entity| entity_on_floors(entity, home, floors)).cloned().collect();
            if !on_floor.is_empty() {
                group = on_floor;
            }
        }
        for index in span.0..span.0 + span.1 {
            linked.consumed.insert(index);
        }
        match group.len() {
            0 => {}
            1 => linked.entities.push(group.remove(0)),
            _ => linked.ambiguous.extend(group),
        }
    }
    linked.entities = dedupe(linked.entities);
    linked.ambiguous = dedupe(linked.ambiguous);
    // A device already chosen uniquely is not also ambiguous.
    linked.ambiguous.retain(|entity| !linked.entities.iter().any(|have| have.entity_id == entity.entity_id));
    linked
}

/// Tokens that are not part of a linked device name (verbs, numbers, colours stay here).
pub(crate) fn free_tokens(tokens: &[String], linked: &LinkedNames) -> Vec<String> {
    tokens.iter().enumerate().filter(|(index, _)| !linked.consumed.contains(index)).map(|(_, token)| token.clone()).collect()
}

/// A timer noun that is only there as part of a device name does not make a timer intent.
pub(crate) fn timer_noun_free(tokens: &[String], linked: &LinkedNames) -> bool {
    tokens.iter().enumerate().any(|(index, token)| !linked.consumed.contains(&index) && catalog().timer_nouns().contains(token.as_str()))
}

fn entities_at_span(tokens: &[String], home: &HomeGraph, span: Span, parts: usize, chars: usize) -> Vec<EntityRec> {
    let mut out = Vec::new();
    for entity in home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home) && !is_infra(e) && !is_nlu_ignored(e) && e.domain != "scene" && e.domain != "script")
    {
        for label in usable_labels(entity) {
            if !linkable(&label, home) || word_count(&label) != parts || compact(&label).chars().count() != chars {
                continue;
            }
            if label_spans(tokens, &label).into_iter().any(|found| found == span)
                && !out.iter().any(|have: &EntityRec| have.entity_id == entity.entity_id)
            {
                out.push(entity.clone());
            }
        }
    }
    out
}

fn entity_on_floors(entity: &EntityRec, home: &HomeGraph, floors: &[String]) -> bool {
    entity.area.as_ref().is_some_and(|area_id| {
        home.areas.iter().any(|area| area.area_id == *area_id && area.floor_id.as_ref().is_some_and(|floor| floors.contains(floor)))
    })
}

fn usable_labels(entity: &EntityRec) -> Vec<String> {
    std::iter::once(entity.name.clone()).chain(entity.aliases.iter().cloned()).collect()
}

fn linkable(label: &str, home: &HomeGraph) -> bool {
    let parts: Vec<String> = fold_label(label).split([' ', '_', '-']).filter(|part| !part.is_empty()).map(str::to_string).collect();
    if parts.is_empty() {
        return false;
    }
    if parts.len() >= 2 {
        return true;
    }
    let word = parts[0].as_str();
    let cat = catalog();
    // A lone floor or room name on a device (alias "obergeschoss" on the upstairs heater)
    // is not an entity link — that word scopes the floor or room.
    if names_floor_or_area(word, home) {
        return false;
    }
    !cat.generic().contains(word)
        && !cat.light_nouns().contains(word)
        && !cat.light_singular().contains(word)
        && !cat.lamp_fixture().contains(word)
        && word != "lamp"
        && word != "light"
        && !cat.is_filler(word)
        && !cat.is_particle(word)
        && cat.verb(word).is_none()
}

fn names_floor_or_area(word: &str, home: &HomeGraph) -> bool {
    home.floors.iter().any(|floor| {
        token_eq(word, &floor.floor_id)
            || token_eq(word, &fold_label(&floor.name))
            || floor.aliases.iter().any(|alias| token_eq(word, &fold_label(alias)))
    }) || home.areas.iter().any(|area| {
        token_eq(word, &area.area_id)
            || token_eq(word, &fold_label(&area.name))
            || area.aliases.iter().any(|alias| token_eq(word, &fold_label(alias)))
    })
}

fn word_count(label: &str) -> usize {
    fold_label(label).split([' ', '_', '-']).filter(|part| !part.is_empty()).count().max(1)
}

fn label_spans(tokens: &[String], label: &str) -> Vec<Span> {
    let folded_label = fold_label(label);
    let parts: Vec<&str> = folded_label.split([' ', '_', '-']).filter(|part| !part.is_empty()).collect();
    let squeezed = compact(label);
    if parts.is_empty() || squeezed.is_empty() {
        return Vec::new();
    }
    let whole = |token: &String| {
        let folded = compact(token);
        token_eq(token, label) || (!folded.is_empty() && (folded == squeezed || umlaut_eq(&folded, &squeezed)))
    };
    let mut found: Vec<Span> = tokens.iter().enumerate().filter(|(_, token)| whole(token)).map(|(at, _)| (at, 1)).collect();
    match parts.len() {
        0 => {}
        1 => {
            // Single distinctive word already covered by `whole`; also accept the bare part.
            found.extend(tokens.iter().enumerate().filter(|(_, token)| token_eq(token, parts[0])).map(|(at, _)| (at, 1)));
            found.sort_unstable();
            found.dedup();
        }
        _ => {
            let run = |window: &[String]| window.iter().zip(&parts).all(|(token, part)| token_eq(token, part));
            found.extend(tokens.windows(parts.len()).enumerate().filter(|(_, window)| run(window)).map(|(at, _)| (at, parts.len())));
        }
    }
    // Prefer the multi-word span when both a squeezed form and a run match the same start.
    found.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
    found.dedup_by(|left, right| left.0 == right.0);
    found
}

fn overlaps(left: Span, right: Span) -> bool {
    let (a0, a1) = (left.0, left.0 + left.1);
    let (b0, b1) = (right.0, right.0 + right.1);
    a0 < b1 && b0 < a1
}

fn dedupe(entities: Vec<EntityRec>) -> Vec<EntityRec> {
    let mut out = Vec::new();
    for entity in entities {
        if !out.iter().any(|have: &EntityRec| have.entity_id == entity.entity_id) {
            out.push(entity);
        }
    }
    out
}

/// Whether every word of the entity's primary name was said (contiguous or loose).
pub(crate) fn fully_named(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> bool {
    usable_labels(entity)
        .into_iter()
        .any(|label| linkable(&label, home) && (token_hit(tokens, &fold_label(&label)) || !label_spans(tokens, &label).is_empty()))
}

/// First token of an exception marker ("except", "außer", "but not", …), if any.
fn except_at(tokens: &[String]) -> Option<usize> {
    let cat = catalog();
    if let Some(at) = tokens.iter().position(|token| cat.is_except(token)) {
        return Some(at);
    }
    tokens.windows(2).enumerate().find_map(|(at, pair)| {
        matches!(
            (pair[0].as_str(), pair[1].as_str()),
            ("bis", "auf") | ("but", "not") | ("except", "for") | ("aber", "nicht") | ("nicht", "im") | ("nicht", "in")
        )
        .then_some(at)
    })
}

/// Apply name-first linking: either an exclusive device list, a room to keep for a list of
/// targets, or nothing when the spoken room does not hold the named device.
///
/// Names after an exception word are exclusions, not the exclusive target — so
/// "everything in the kitchen except the oven" keeps the room-wide path.
pub(crate) fn take_linked(
    tokens: &[String],
    home: &HomeGraph,
    floors: &[String],
    areas: &[String],
    domain: Option<&str>,
) -> Option<super::Resolved> {
    if crate::parse::infer::wants_all_lights(tokens) {
        return None;
    }
    // Link only the command head: exception-tail names must not steal the target.
    let head = except_at(tokens).map(|at| &tokens[..at]).unwrap_or(tokens);
    let linked = link_named_spans(head, home, floors);
    if linked.entities.is_empty() && linked.ambiguous.is_empty() {
        return None;
    }
    // Free tokens for "more targets" still come from the whole sentence so cover/light
    // nouns after "and" stay visible; exception-tail device names are not in `linked`.
    let full = link_named_spans(tokens, home, floors);
    let free = free_tokens(tokens, &full);
    let filtered_entities = filter_domain(linked.entities.clone(), domain);
    let filtered_ambiguous = filter_domain(linked.ambiguous.clone(), domain);
    let (mut entities, mut ambiguous) = if filtered_entities.is_empty() && filtered_ambiguous.is_empty() {
        (linked.entities, linked.ambiguous)
    } else {
        (filtered_entities, filtered_ambiguous)
    };
    if !areas.is_empty() {
        let in_room = |entity: &EntityRec| entity.area.as_ref().is_some_and(|area| areas.contains(area));
        let scoped: Vec<EntityRec> = entities.iter().filter(|entity| in_room(entity)).cloned().collect();
        let scoped_ambiguous: Vec<EntityRec> = ambiguous.iter().filter(|entity| in_room(entity)).cloned().collect();
        if !scoped.is_empty() || !scoped_ambiguous.is_empty() {
            entities = scoped;
            ambiguous = scoped_ambiguous;
        } else if entities.iter().chain(ambiguous.iter()).any(|entity| entity.area.is_some()) {
            return Some(super::Resolved {
                areas: Vec::new(),
                floors: floors.to_vec(),
                entities: Vec::new(),
                ambiguous: Vec::new(),
                ask_areas: Vec::new(),
            });
        }
    }
    if entities.is_empty() && ambiguous.is_empty() {
        return None;
    }
    let cat = catalog();
    let more_targets = cat.any(&free, cat.cover_nouns())
        || cat.any(&free, cat.light_nouns())
        || cat.any(&free, cat.light_plural())
        || cat.any(&free, cat.lamp_fixture())
        || cat.any(&free, cat.fan_nouns())
        || cat.any(&free, cat.climate_nouns())
        || cat.any(&free, cat.lock_nouns())
        || cat.any(&free, cat.switch_plural());
    if more_targets {
        // Keep the room, drop the exclusive entity so "blinds and curtains" stays room-wide.
        let mut area_ids: Vec<String> = entities.iter().chain(ambiguous.iter()).filter_map(|entity| entity.area.clone()).collect();
        area_ids.extend(areas.iter().cloned());
        area_ids.sort();
        area_ids.dedup();
        return Some(super::Resolved {
            areas: area_ids,
            floors: floors.to_vec(),
            entities: Vec::new(),
            ambiguous: Vec::new(),
            ask_areas: Vec::new(),
        });
    }
    // Spoken rooms only — never invent areas from the linked device's home room.
    // Exception handling treats a non-empty resolve().areas as "this token names a room";
    // stuffing entity.area here made "oven" protect the whole kitchen.
    Some(super::Resolved { areas: areas.to_vec(), floors: floors.to_vec(), entities, ambiguous, ask_areas: Vec::new() })
}

fn filter_domain(entities: Vec<EntityRec>, domain: Option<&str>) -> Vec<EntityRec> {
    match domain {
        Some(wanted) => entities.into_iter().filter(|entity| crate::home::roles::matches_domain(entity, wanted, catalog())).collect(),
        None => entities,
    }
}
