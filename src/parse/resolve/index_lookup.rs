//! Inverted-index helpers for resolve (R-09).

use crate::lang::catalog;
use crate::lang::Catalog;
use crate::parse::normalize::compact;
use crate::types::{EntityRec, HomeGraph, LabelIndex};
use std::collections::HashSet;

/// Prefer intersection when every content token hit the index; otherwise union.
/// Digits count so "fixture 1" does not score every `fixture*` row (R-09).
/// Multi-device lines fall back to union when AND would be empty.
pub(super) fn content_candidates(tokens: &[String], index: &LabelIndex) -> Vec<usize> {
    let cat = catalog();
    let content: Vec<&str> = tokens.iter().map(String::as_str).filter(|token| is_content_token(token, cat)).collect();
    if content.is_empty() {
        return Vec::new();
    }
    let mut posting: Option<HashSet<usize>> = None;
    let mut all_hit = true;
    for token in &content {
        let Some(rows) = index.by_token.get(*token) else {
            all_hit = false;
            continue;
        };
        let set: HashSet<usize> = rows.iter().copied().collect();
        posting = Some(match posting {
            None => set,
            Some(prev) => prev.intersection(&set).copied().collect(),
        });
    }
    if all_hit {
        if let Some(set) = posting.filter(|set| !set.is_empty()) {
            return set.into_iter().collect();
        }
    }
    let mut union = HashSet::new();
    for token in &content {
        if let Some(rows) = index.by_token.get(*token) {
            union.extend(rows.iter().copied());
        }
    }
    union.into_iter().collect()
}

/// Catalog fixture words ("skottelgoed" → dishwasher) are not stored on entity labels.
/// Seed those rows so the indexed resolve path still scores them (parity packs).
pub(super) fn fixture_alias_candidates(tokens: &[String], home: &HomeGraph) -> Vec<usize> {
    let cat = catalog();
    let needles: Vec<String> = tokens
        .iter()
        .flat_map(|token| cat.fixture_alias(token).iter().copied())
        .map(compact)
        .filter(|alias| alias.len() >= 5 && !cat.generic().contains(&alias.as_str()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    if needles.is_empty() {
        return Vec::new();
    }
    home.entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| {
            let name = compact(&entity.name);
            needles.iter().any(|alias| name.contains(alias)).then_some(index)
        })
        .collect()
}

pub(super) fn entities_for_indices<'a>(home: &'a HomeGraph, indices: &[usize]) -> Vec<&'a EntityRec> {
    indices.iter().filter_map(|&index| home.entities.get(index)).collect()
}

fn is_content_token(token: &str, cat: &Catalog) -> bool {
    if cat.is_filler(token)
        || cat.is_particle(token)
        || cat.verb(token).is_some()
        || cat.on_words().contains(token)
        || cat.off_words().contains(token)
        || cat.open_words().contains(token)
        || cat.close_words().contains(token)
        || cat.all_words().contains(token)
        || cat.conjunctions().contains(token)
    {
        return false;
    }
    token.chars().count() >= 2 || token.chars().all(|ch| ch.is_ascii_digit())
}
