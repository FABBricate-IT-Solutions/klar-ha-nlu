//! Read-only questions about window contacts. Cover commands stay on the cover domain.
use super::context::ParseContext;
use super::draft::{execute, reject, Draft};
use crate::lang::Catalog;
use crate::parse::infer::looks_like_question;
use crate::parse::resolve::resolve;
use crate::parse::respond::speak_unknown;
use crate::types::{HomeGraph, Intent, IntentPlan, RejectReason};

pub(super) fn route(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
    if !is_status_question(context.catalog, tokens) {
        return None;
    }
    let open = context.catalog.any(tokens, context.catalog.open_words());
    let closed = context.catalog.any(tokens, context.catalog.close_words());
    if open && closed {
        return Some(reject(RejectReason::Unsupported, speak_unknown()));
    }
    let resolved = resolve(tokens, context.home, Some("binary_sensor"));
    if !tokens.iter().all(|word| allowed(word, context.catalog, &resolved.areas, context.home)) {
        return Some(reject(RejectReason::NoTarget, speak_unknown()));
    }
    if resolved.areas.len() > 1 || !resolved.floors.is_empty() {
        return Some(reject(RejectReason::Unsupported, speak_unknown()));
    }
    let mut intent = Intent::new("HassGetState").with("domain", "binary_sensor").with("device_class", "window").with("response", "list");
    if open || closed {
        intent = intent.with("state", if open { "on" } else { "off" });
    }
    if let Some(area) = resolved.areas.first() {
        intent = intent.with("area", area);
    }
    if super::validation::validate_plan(&IntentPlan::from_intents(vec![intent.clone()], 1.0, &[]), context.home).is_err() {
        return Some(reject(RejectReason::NoTarget, speak_unknown()));
    }
    Some(execute(context, vec![intent], "window_state_query", 1.0, 1.0, false, false))
}

fn is_status_question(cat: &Catalog, tokens: &[String]) -> bool {
    let Some(window_at) = tokens.iter().position(|word| glazing(word) && cat.window_words().contains(word.as_str())) else {
        return false;
    };
    if cat.any(tokens, cat.door_nouns()) || cat.any(tokens, cat.garage_words()) {
        return false;
    }
    let command_before = tokens[..window_at].iter().any(|word| {
        cat.open_words().contains(word.as_str()) || cat.close_words().contains(word.as_str()) || matches!(word.as_str(), "turn" | "shut")
    });
    let question = looks_like_question(tokens) || tokens.iter().any(|word| matches!(word.as_str(), "which" | "any" | "there"));
    let state_after =
        tokens[window_at + 1..].iter().any(|word| cat.open_words().contains(word.as_str()) || cat.close_words().contains(word.as_str()));
    if command_before && !question {
        return false;
    }
    question || state_after
}

fn glazing(word: &str) -> bool {
    matches!(word, "window" | "windows" | "fenster")
}

fn allowed(word: &str, cat: &Catalog, areas: &[String], home: &HomeGraph) -> bool {
    (glazing(word) && cat.window_words().contains(word))
        || cat.fillers().contains(word)
        || cat.is_question_word(word)
        || cat.is_question_start(word)
        || cat.open_words().contains(word)
        || cat.close_words().contains(word)
        || cat.all_words().contains(word)
        || matches!(word, "any" | "there" | "which" | "do" | "does" | "did" | "i" | "have" | "has" | "got" | "we" | "our" | "opened")
        || areas
            .iter()
            .any(|area| area_label(home, area).iter().any(|label| crate::parse::normalize::tokenize(label).contains(&word.to_string())))
}

fn area_label(home: &HomeGraph, area: &str) -> Vec<String> {
    home.areas
        .iter()
        .find(|record| record.area_id == area)
        .map(|record| {
            std::iter::once(record.name.clone())
                .chain(std::iter::once(record.area_id.clone()))
                .chain(record.aliases.iter().cloned())
                .collect()
        })
        .unwrap_or_default()
}
