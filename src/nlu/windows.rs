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
    let resolved = resolve(tokens, context.home, Some("binary_sensor"));
    // "ein Fenster": an article that is also a command word, right before the window.
    let article = |at: usize| {
        context.catalog.article_one().contains(tokens[at].as_str())
            && tokens.get(at + 1).is_some_and(|next| glazing(next) && context.catalog.window_words().contains(next.as_str()))
    };
    if !tokens.iter().enumerate().all(|(at, word)| article(at) || allowed(word, context.catalog, &resolved.areas, context.home)) {
        return None;
    }
    if resolved.areas.len() > 1 || !resolved.floors.is_empty() {
        return None;
    }
    let mut intent = Intent::new("HassGetState").with("domain", "binary_sensor").with("device_class", "window").with("response", "list");
    if open || closed {
        intent = intent.with("state", if open { "on" } else { "off" });
    }
    if let Some(area) = resolved.areas.first() {
        intent = intent.with("area", area);
        // Blinds/curtains but no sensor named like a window: a motion or door sensor
        // in the room must not swallow the question; let the cover parser answer.
        if !has_window_sensor(context.catalog, context.home, area) && has_window_cover(context.catalog, context.home, area) {
            return None;
        }
    }
    // No window contact here (e.g. the "windows" are blinds): let the cover parser answer.
    if super::validation::validate_plan(&IntentPlan::from_intents(vec![intent.clone()], 1.0, &[]), context.home).is_err() {
        return None;
    }
    // "open or closed?" on real window contacts has no single state filter yet.
    if open && closed {
        return Some(reject(RejectReason::Unsupported, speak_unknown()));
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

fn has_window_sensor(cat: &Catalog, home: &HomeGraph, area: &str) -> bool {
    home.entities.iter().any(|entity| {
        entity.domain == "binary_sensor" && entity.area.as_deref() == Some(area) && {
            let blob = format!("{} {}", entity.entity_id, entity.name).to_lowercase();
            cat.window_words().iter().any(|word| glazing(word) && blob.contains(*word))
        }
    })
}

fn has_window_cover(cat: &Catalog, home: &HomeGraph, area: &str) -> bool {
    home.entities.iter().any(|entity| entity.domain == "cover" && entity.area.as_deref() == Some(area) && !is_garage_door(cat, entity))
}

fn is_garage_door(cat: &Catalog, entity: &crate::types::EntityRec) -> bool {
    let blob = format!("{} {}", entity.entity_id, entity.name).to_lowercase();
    cat.garage_words().iter().any(|word| word.len() >= 4 && blob.contains(word)) || blob.contains("garagentor")
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
        || matches!(
            word,
            "any"
                | "there"
                | "which"
                | "do"
                | "does"
                | "did"
                | "i"
                | "have"
                | "has"
                | "got"
                | "we"
                | "our"
                | "opened"
                | "tell"
                | "check"
                | "see"
                | "know"
                | "whether"
                | "about"
                | "show"
        )
        || areas
            .iter()
            .any(|area| area_label(home, area).iter().any(|label| crate::parse::normalize::tokenize(label).contains(&word.to_string())))
        || report_word(word, cat)
}

/// A word of question or report grammar: "did", "left open", "still", "gelassen", "noch".
/// It names no room and no device, so a window question may carry it.
fn report_word(word: &str, cat: &Catalog) -> bool {
    cat.moods().any(|mood| {
        [mood.question_starts, mood.subjects, mood.state_verbs, mood.participles, mood.ask_frames, mood.state_padding]
            .iter()
            .any(|list| list.iter().any(|entry| entry.split(' ').any(|part| part == word)))
    })
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
