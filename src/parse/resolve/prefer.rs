use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::catalog;
use crate::parse::normalize::{compact, fold_umlaut};
use crate::types::{EntityRec, HomeGraph};

pub(super) fn prefer_tv(tokens: &[String], home: &HomeGraph, candidates: &mut Vec<(f64, EntityRec)>) {
    if !tv_utterance(tokens) {
        return;
    }
    let mentioned: Vec<String> =
        home.areas.iter().filter(|area| area_mentioned(tokens, &area.area_id, home)).map(|area| area.area_id.clone()).collect();
    if mentioned.is_empty() {
        let aliased: Vec<(f64, EntityRec)> =
            candidates.iter().filter(|(_, entity)| looks_like_tv(entity) && has_tv_alias(entity)).cloned().collect();
        if aliased.len() == 1 {
            *candidates = aliased;
            return;
        }
        for entity in home.entities.iter().filter(|entity| looks_like_tv(entity) && assist_visible(entity, home) && !is_infra(entity)) {
            if !candidates.iter().any(|(_, existing)| existing.entity_id == entity.entity_id) {
                candidates.push((0.95, entity.clone()));
            }
        }
        let tvs: Vec<(f64, EntityRec)> = candidates.iter().filter(|(_, entity)| looks_like_tv(entity)).cloned().collect();
        if tvs.is_empty() {
            return;
        }
        let aliased: Vec<(f64, EntityRec)> = tvs.iter().filter(|(_, entity)| has_tv_alias(entity)).cloned().collect();
        if aliased.len() == 1 {
            *candidates = aliased;
            return;
        }
        let media: Vec<(f64, EntityRec)> = tvs.iter().filter(|(_, entity)| entity.domain == "media_player").cloned().collect();
        if media.len() == 1 {
            *candidates = media;
        }
        return;
    }
    for entity in home.entities.iter().filter(|entity| {
        looks_like_tv(entity)
            && assist_visible(entity, home)
            && !is_infra(entity)
            && entity.area.as_deref().is_some_and(|area| mentioned.iter().any(|id| id == area))
    }) {
        if !candidates.iter().any(|(_, existing)| existing.entity_id == entity.entity_id) {
            candidates.push((0.95, entity.clone()));
        }
    }
    let in_area: Vec<(f64, EntityRec)> = candidates
        .iter()
        .filter(|(_, entity)| looks_like_tv(entity) && entity.area.as_deref().is_some_and(|area| mentioned.iter().any(|id| id == area)))
        .cloned()
        .collect();
    if !in_area.is_empty() {
        let media: Vec<(f64, EntityRec)> = in_area.iter().filter(|(_, entity)| entity.domain == "media_player").cloned().collect();
        *candidates = if media.is_empty() { in_area } else { media };
        return;
    }
    let media: Vec<(f64, EntityRec)> = home
        .entities
        .iter()
        .filter(|entity| {
            entity.domain == "media_player"
                && assist_visible(entity, home)
                && !is_infra(entity)
                && entity.area.as_deref().is_some_and(|area| mentioned.iter().any(|id| id == area))
        })
        .map(|entity| (0.92, entity.clone()))
        .collect();
    *candidates = media;
}

fn area_mentioned(tokens: &[String], area_id: &str, home: &HomeGraph) -> bool {
    home.areas.iter().filter(|area| area.area_id == area_id).any(|area| {
        tokens.iter().any(|token| {
            token == area.area_id.as_str()
                || fold_umlaut(&area.name) == *token
                || area.aliases.iter().any(|alias| fold_umlaut(alias) == *token || compact(alias) == *token)
        })
    })
}

fn looks_like_tv(entity: &EntityRec) -> bool {
    if entity.domain != "media_player" && entity.domain != "switch" {
        return false;
    }
    let hay = format!("{} {} {}", entity.entity_id, entity.name, entity.aliases.join(" "));
    let folded = compact(&fold_umlaut(&hay));
    folded.contains("tv") || folded.contains("fernseher") || folded.contains("television")
}

fn tv_token(token: &str) -> bool {
    let folded = compact(&fold_umlaut(token));
    folded == "tv" || folded == "fernseher" || folded == "television" || catalog().tv_words().iter().any(|word| folded == compact(word))
}

fn tv_utterance(tokens: &[String]) -> bool {
    tokens.iter().any(|token| tv_token(token))
}

fn has_tv_alias(entity: &EntityRec) -> bool {
    entity.aliases.iter().any(|alias| tv_token(alias))
}

pub(super) fn prefer_entry_lock(tokens: &[String], home: &HomeGraph, candidates: &mut Vec<(f64, EntityRec)>) {
    let cat = catalog();
    if crate::parse::action::is_garage_cover(tokens) && !cat.any(tokens, cat.lock_verbs()) {
        return;
    }
    if cat.any(tokens, cat.sensor_words()) {
        return;
    }
    let lockish = cat.any(tokens, cat.lock_verbs()) || cat.any(tokens, cat.lock_nouns()) || cat.any(tokens, cat.door_nouns());
    if !lockish {
        return;
    }
    let has_lock = candidates.iter().any(|(_, entity)| entity.domain == "lock");
    if should_seed_locks(tokens, has_lock) {
        for entity in home.entities.iter().filter(|entity| entity.domain == "lock" && assist_visible(entity, home) && !is_infra(entity)) {
            if candidates.iter().any(|(_, existing)| existing.entity_id == entity.entity_id) {
                continue;
            }
            candidates.push((0.88, entity.clone()));
        }
    }
    let locks: Vec<(f64, EntityRec)> = candidates.iter().filter(|(_, entity)| entity.domain == "lock").cloned().collect();
    if locks.len() == 1 {
        *candidates = locks;
        return;
    }
    if locks.is_empty() {
        return;
    }
    let mentioned: Vec<(f64, EntityRec)> = locks.iter().filter(|(_, entity)| lock_mentioned(tokens, entity)).cloned().collect();
    if mentioned.len() >= 2 {
        *candidates = mentioned;
        return;
    }
    if mentioned.len() == 1 {
        *candidates = mentioned;
        return;
    }
    if session_lock_follow(tokens) {
        candidates.retain(|(_, entity)| entity.domain != "lock");
        return;
    }
    let entry: Vec<(f64, EntityRec)> = locks.iter().filter(|(_, entity)| is_entry_lock(entity)).cloned().collect();
    if entry.len() == 1 {
        *candidates = entry;
    }
}

pub(super) fn lock_mentioned(tokens: &[String], entity: &EntityRec) -> bool {
    let cat = catalog();
    tokens.iter().any(|token| {
        if token.len() <= 2 || cat.lock_nouns().contains(token.as_str()) {
            return false;
        }
        if cat.door_nouns().contains(token.as_str()) {
            return token.len() > 6 && exact_lock_label(entity, token);
        }
        if cat.entry_words().contains(token.as_str()) {
            return is_entry_lock(entity) && !garage_entry_phrase(tokens);
        }
        if (cat.garage_words().contains(token.as_str()) || token == "garage") && entity.area.as_deref() == Some("garage") {
            return true;
        }
        entity.area.as_deref().is_some_and(|area| area == token || fold_umlaut(area) == *token)
            || entity
                .entity_id
                .split_once('.')
                .map(|(_, rest)| rest.split(|c: char| !c.is_alphanumeric()).any(|part| part == token))
                .unwrap_or(false)
            || exact_lock_label(entity, token)
    })
}

fn exact_lock_label(entity: &EntityRec, token: &str) -> bool {
    let name = fold_umlaut(&entity.name);
    if name == *token {
        return true;
    }
    if !name.contains(|c: char| c.is_whitespace() || c == '-') && name.split(|c: char| !c.is_alphanumeric()).any(|part| part == token) {
        return true;
    }
    entity.aliases.iter().any(|alias| fold_umlaut(alias) == *token) || entity.tags.iter().any(|tag| fold_umlaut(tag) == *token)
}

pub(super) fn mentioned_locks(tokens: &[String], candidates: &[(f64, EntityRec)]) -> Option<Vec<EntityRec>> {
    let locks: Vec<EntityRec> = candidates
        .iter()
        .filter(|(_, entity)| entity.domain == "lock" && lock_mentioned(tokens, entity))
        .map(|(_, entity)| entity.clone())
        .collect();
    (locks.len() >= 2).then_some(locks)
}

pub(super) fn two_lock_rooms(tokens: &[String]) -> bool {
    let cat = catalog();
    let has_entry = cat.any(tokens, cat.entry_words());
    let has_garage = tokens.iter().any(|token| token == "garage" || cat.garage_words().contains(token.as_str()));
    has_entry && has_garage && !garage_entry_phrase(tokens)
}

fn garage_entry_phrase(tokens: &[String]) -> bool {
    let cat = catalog();
    tokens.windows(2).any(|window| {
        (window[0] == "garage" || cat.garage_words().contains(window[0].as_str()))
            && (window[1] == "entry" || cat.entry_words().contains(window[1].as_str()))
    })
}

fn should_seed_locks(tokens: &[String], has_lock: bool) -> bool {
    if two_lock_rooms(tokens) {
        return true;
    }
    let cat = catalog();
    if cat.any(tokens, cat.lock_verbs()) && cat.any(tokens, cat.conjunctions()) {
        return true;
    }
    if has_lock {
        return false;
    }
    let door = cat.any(tokens, cat.door_nouns());
    let lock_noun = cat.any(tokens, cat.lock_nouns());
    if !door && !lock_noun {
        return false;
    }
    let grounded = door
        || cat.any(tokens, cat.entry_words())
        || tokens.iter().any(|token| token == "garage" || cat.garage_words().contains(token.as_str()));
    if pronoun_follow(tokens) && !grounded {
        return false;
    }
    true
}

fn pronoun_follow(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "it" | "that" | "this" | "es" | "ihn" | "sie"))
}

fn session_lock_follow(tokens: &[String]) -> bool {
    let cat = catalog();
    !cat.any(tokens, cat.door_nouns())
        && !cat.any(tokens, cat.entry_words())
        && !tokens.iter().any(|token| token == "garage" || cat.garage_words().contains(token.as_str()))
}

fn is_entry_lock(entity: &EntityRec) -> bool {
    let cat = catalog();
    if entity.area.as_deref() == Some("garage") || entity.entity_id.contains("garage") {
        return false;
    }
    entity.entity_id.contains("front")
        || entity.area.as_deref().is_some_and(|area| area == "entryway" || area == "entry")
        || cat
            .entry_words()
            .iter()
            .any(|word| fold_umlaut(&entity.name).contains(word) || entity.aliases.iter().any(|alias| fold_umlaut(alias).contains(word)))
}

#[cfg(test)]
#[path = "prefer_tests.rs"]
mod tests;
