//! Dutch negation safety and read-only window-state questions.
use super::context::ParseContext;
use super::draft::{execute, reject, Draft};
use crate::parse::infer::looks_like_question;
use crate::parse::resolve::resolve;
use crate::parse::respond::speak_unknown;
use crate::types::{Intent, IntentPlan, RejectReason};

pub(super) fn route(context: &ParseContext<'_>, raw: &[String], tokens: &[String]) -> Option<Draft> {
    if context.text.chars().any(|character| character.is_control() && !character.is_whitespace()) {
        return None;
    }
    if !context.catalog.langs.iter().any(|lang| lang.code() == "nl") {
        return None;
    }
    // Fail closed before custom rules, pending confirmations, and session replay.
    // Negated queries are also rejected until their scope can be represented.
    if raw.iter().any(|word| matches!(word.as_str(), "niet" | "geen" | "nooit" | "nee" | "zonder")) {
        let mut draft = reject(RejectReason::Unsupported, speak_unknown());
        draft.commit.clear_pending = true;
        return Some(draft);
    }
    if !looks_like_question(tokens) || !context.catalog.any(tokens, context.catalog.window_words()) {
        return None;
    }
    let open = context.catalog.any(tokens, context.catalog.open_words());
    let closed = context.catalog.any(tokens, context.catalog.close_words()) || tokens.iter().any(|word| word == "gesloten");
    if open && closed {
        return Some(reject(RejectReason::Unsupported, speak_unknown()));
    }
    let resolved = resolve(tokens, context.home, Some("binary_sensor"));
    let mut intent = Intent::new("HassGetState").with("domain", "binary_sensor").with("device_class", "window").with("response", "list");
    if open || closed {
        intent = intent.with("state", if open { "on" } else { "off" });
    }
    // Never turn an unknown place into a whole-home request.
    let cat = context.catalog;
    if tokens.iter().any(|word| {
        !cat.window_words().contains(word.as_str())
            && !cat.is_question_word(word)
            && !cat.is_question_start(word)
            && !cat.open_words().contains(word.as_str())
            && !cat.close_words().contains(word.as_str())
            && !matches!(word.as_str(), "gesloten" | "van")
            && !resolved.areas.iter().any(|area| {
                context.home.areas.iter().any(|record| {
                    record.area_id == *area
                        && std::iter::once(&record.name)
                            .chain(std::iter::once(&record.area_id))
                            .chain(record.aliases.iter())
                            .any(|label| crate::parse::normalize::tokenize(label).contains(word))
                })
            })
    }) {
        return Some(reject(RejectReason::NoTarget, speak_unknown()));
    }
    if resolved.areas.len() > 1 || !resolved.floors.is_empty() {
        return Some(reject(RejectReason::Unsupported, speak_unknown()));
    }
    if let Some(area) = resolved.areas.first() {
        intent = intent.with("area", area);
    }
    if super::validation::validate_plan(&IntentPlan::from_intents(vec![intent.clone()], 1.0, &[]), context.home).is_err() {
        return Some(reject(RejectReason::NoTarget, speak_unknown()));
    }
    Some(execute(context, vec![intent], "window_state_query", 1.0, 1.0, false, false))
}
