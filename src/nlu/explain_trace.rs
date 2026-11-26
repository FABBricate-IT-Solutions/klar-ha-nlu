//! Decision-trace helpers: consumed name spans, leftover tokens, and the trigger word.

use crate::lang::Catalog;
use crate::parse::action::{detect_actions_bounded, Action};
use crate::parse::infer::prefer_action;
use crate::parse::resolve::{free_tokens, link_named_spans};
use crate::types::{ConsumedSpan, HomeGraph, ParseTrace, MAX_TRACE_SPANS, MAX_TRACE_TOKENS};

use super::binding::MAX_ACTION_HYPOTHESES;

/// Record name spans the linker ate and the tokens that remain for verbs and quantities.
pub(super) fn record_spans(trace: &mut ParseTrace, tokens: &[String], home: &HomeGraph) {
    let linked = link_named_spans(tokens, home, &[]);
    let mut indices: Vec<usize> = linked.consumed.iter().copied().collect();
    indices.sort_unstable();
    let mut spans = Vec::new();
    let mut at = 0;
    while at < indices.len() {
        let start = indices[at];
        let mut end = start + 1;
        at += 1;
        while at < indices.len() && indices[at] == end {
            end += 1;
            at += 1;
        }
        if end > tokens.len() {
            break;
        }
        let text = tokens[start..end].join(" ");
        let kind = linked
            .entities
            .iter()
            .chain(linked.ambiguous.iter())
            .find(|entity| span_matches_entity(tokens, start, end, entity))
            .map(|entity| entity.domain.clone())
            .unwrap_or_else(|| "name".into());
        spans.push(ConsumedSpan { start, end, text, kind });
        if spans.len() >= MAX_TRACE_SPANS {
            break;
        }
    }
    trace.consumed_spans = spans;
    trace.remaining_tokens = free_tokens(tokens, &linked).into_iter().take(MAX_TRACE_TOKENS).collect();
}

/// Spoken token that selected the preferred action for this utterance.
pub(super) fn trigger_word(tokens: &[String], catalog: &Catalog, home: &HomeGraph) -> Option<String> {
    let linked = link_named_spans(tokens, home, &[]);
    let mut detected = detect_actions_bounded(tokens, MAX_ACTION_HYPOTHESES);
    detected.retain(|(index, action)| {
        !matches!(action, Action::TimerStart | Action::TimerAdd | Action::TimerRemove | Action::TimerCancel | Action::TimerPause)
            || !linked.consumed.contains(index)
    });
    let selected = prefer_action(&detected)?;
    if let Some((index, _)) = detected.iter().find(|(_, action)| *action == selected) {
        if let Some(word) = tokens.get(*index).filter(|token| !token.is_empty()) {
            return Some(word.clone());
        }
    }
    colour_or_quantity_trigger(tokens, catalog).or_else(|| scene_name_trigger(tokens, catalog, home))
}

fn colour_or_quantity_trigger(tokens: &[String], catalog: &Catalog) -> Option<String> {
    tokens
        .iter()
        .find_map(|token| catalog.color(token).map(|_| token.clone()))
        .or_else(|| tokens.iter().find(|token| catalog.number(token).is_some() || token.parse::<i32>().is_ok()).cloned())
}

fn scene_name_trigger(tokens: &[String], catalog: &Catalog, home: &HomeGraph) -> Option<String> {
    if catalog.any(tokens, catalog.scene_nouns()) || catalog.any(tokens, catalog.script_words()) {
        return tokens
            .iter()
            .find(|token| catalog.scene_nouns().contains(token.as_str()) || catalog.script_words().contains(token.as_str()))
            .cloned();
    }
    home.entities.iter().filter(|entity| entity.domain == "scene" || entity.domain == "script").find_map(|entity| {
        let labels = std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str));
        for label in labels {
            let parts: Vec<String> = crate::parse::normalize::tokenize(label);
            if parts.is_empty() {
                continue;
            }
            if tokens.windows(parts.len()).any(|window| window == parts.as_slice()) {
                return parts.first().cloned();
            }
        }
        None
    })
}

fn span_matches_entity(tokens: &[String], start: usize, end: usize, entity: &crate::types::EntityRec) -> bool {
    let slice = &tokens[start..end];
    let labels = std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str));
    for label in labels {
        let parts: Vec<String> = crate::parse::normalize::tokenize(label);
        if !parts.is_empty() && parts.as_slice() == slice {
            return true;
        }
    }
    false
}
