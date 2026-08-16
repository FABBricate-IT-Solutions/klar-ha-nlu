pub mod action;
pub(crate) mod chat;
pub(crate) mod clause;
pub mod compound;
pub(crate) mod fuzzy;
pub(crate) mod infer;
pub(crate) mod media;
pub mod normalize;
pub mod numbers;
pub mod resolve;
pub(crate) mod resolve_named;
pub mod respond;
pub(crate) mod slots;
pub mod split;

use crate::lang::catalog;
use crate::parse::chat::{briefing_followup, is_news, is_news_dismiss, looks_like_home, wants_llm};
use crate::parse::clause::{last_visible, parse_clause};
use crate::parse::compound::{expand_compounds, CompoundSplit};
use crate::parse::infer::{looks_like_correction, match_custom, pick_clarification};
use crate::parse::normalize::{strip_fillers, tokenize};
use crate::parse::numbers::first_number;
use crate::parse::respond::{speak, speak_clarify, speak_correction, speak_need_target, speak_unknown};
use crate::parse::slots::{intent_with_entity, ClauseOut};
use crate::parse::split::split_clauses;
use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, Intent, ParseResult, Settings};

pub fn parse(text: &str, home: &HomeGraph, session: &mut Session, custom: &[CustomSentence], settings: &Settings) -> ParseResult {
    let _guard = crate::lang::bind(&settings.languages);
    let (raw_tokens, split) = preprocess(text, home);
    let tokens = &split.tokens;

    if let Some(result) = route_non_home(text, session, home, &raw_tokens, tokens) {
        return result;
    }
    if let Some(result) = session_followups(text, session, home, tokens, custom, settings) {
        return result;
    }

    let (mut intents, clarify_names) = parse_clauses(&raw_tokens, &split, home, session, settings);
    dedup_intents(&mut intents);
    if !clarify_names.is_empty() {
        return done(text, session, Vec::new(), speak_clarify(&clarify_names, Some(home)), true, false);
    }
    if let Some(result) = fill_replay_or_need_target(text, session, home, tokens, &mut intents) {
        return result;
    }
    let speech = if intents.is_empty() { speak_unknown() } else { speak(&intents, settings.personality, false, Some(home)) };
    done(text, session, intents, speech, false, false)
}

fn preprocess(text: &str, home: &HomeGraph) -> (Vec<String>, CompoundSplit) {
    let raw_tokens = tokenize(text);
    let split = expand_compounds(&strip_fillers(&raw_tokens), home);
    (raw_tokens, split)
}

fn route_non_home(text: &str, session: &mut Session, home: &HomeGraph, raw_tokens: &[String], tokens: &[String]) -> Option<ParseResult> {
    if is_news(raw_tokens, home) || is_news(tokens, home) {
        session.briefing = true;
        return Some(done(text, session, Vec::new(), catalog().news_intro.to_string(), false, true));
    }
    if session.briefing && (is_news_dismiss(tokens) || is_news_dismiss(raw_tokens)) {
        session.briefing = false;
        return Some(ParseResult {
            text: text.to_string(),
            intents: Vec::new(),
            speech: catalog().news_done.to_string(),
            clarify: false,
            conversation_id: session.id.clone(),
            chat: false,
            briefing: true,
        });
    }
    if briefing_followup(tokens, home, session) || briefing_followup(raw_tokens, home, session) {
        return Some(done(text, session, Vec::new(), String::new(), false, true));
    }
    if looks_like_home(tokens, home) || looks_like_home(raw_tokens, home) {
        session.briefing = false;
    }
    if wants_llm(raw_tokens, home) || wants_llm(tokens, home) {
        return Some(done(text, session, Vec::new(), String::new(), false, true));
    }
    if looks_like_correction(tokens) {
        session.mark_wrong();
        return Some(done(text, session, Vec::new(), speak_correction(), false, false));
    }
    None
}

fn session_followups(
    text: &str,
    session: &mut Session,
    home: &HomeGraph,
    tokens: &[String],
    custom: &[CustomSentence],
    settings: &Settings,
) -> Option<ParseResult> {
    if session.pending_clarify.is_none() && tokens.iter().all(|t| catalog().is_affirm(t)) && !tokens.is_empty() {
        if let Some(id) = last_visible(session, home) {
            let name = session.last_names().find(|n| n.starts_with("Hass") && *n != "HassGetState").unwrap_or("HassTurnOn").to_string();
            let intent = Intent::new(name).with("entity_id", id);
            session.remember(&intent);
            let speech = speak(std::slice::from_ref(&intent), settings.personality, false, Some(home));
            return Some(done(text, session, vec![intent], speech, false, false));
        }
    }
    if session.pending_clarify.is_some() {
        if let Some(chosen) = pick_clarification(tokens, session) {
            let template =
                session.last_intent_template.clone().unwrap_or_else(|| Intent::new("HassTurnOn").with("entity_id", chosen.clone()));
            let intent = if home.areas.iter().any(|area| area.area_id == chosen) {
                template.with("area", &chosen).with("domain", "light")
            } else {
                intent_with_entity(template, &chosen)
            };
            session.clear_clarify();
            session.remember(&intent);
            let speech = speak(std::slice::from_ref(&intent), settings.personality, false, Some(home));
            return Some(done(text, session, vec![intent], speech, false, false));
        }
    }
    if let Some(hit) = match_custom(tokens, text, custom) {
        session.remember(&hit);
        let speech = speak(std::slice::from_ref(&hit), settings.personality, false, Some(home));
        return Some(done(text, session, vec![hit], speech, false, false));
    }
    None
}

fn parse_clauses(
    raw_tokens: &[String],
    split: &CompoundSplit,
    home: &HomeGraph,
    session: &mut Session,
    settings: &Settings,
) -> (Vec<Intent>, Vec<String>) {
    let clauses = split_clauses(&split.tokens, home);
    let mut intents = Vec::new();
    let mut clarify_names = Vec::new();
    for clause in clauses {
        match parse_clause(&clause, raw_tokens, home, session, settings, &split.light_areas) {
            ClauseOut::Intents(mut list) => {
                for intent in &list {
                    session.remember(intent);
                }
                intents.append(&mut list);
            }
            ClauseOut::Clarify(names, template) => {
                session.pending_clarify = Some(names.clone());
                session.remember(&template);
                session.last_intent_template = Some(template);
                clarify_names = names;
            }
        }
    }
    (intents, clarify_names)
}

fn dedup_intents(intents: &mut Vec<Intent>) {
    let mut seen: Vec<String> = Vec::new();
    intents.retain(|intent| {
        let mut slots: Vec<String> = intent.slots.iter().map(|slot| format!("{}={}", slot.name, slot.value)).collect();
        slots.sort();
        let key = format!("{}|{}", intent.name, slots.join("|"));
        if seen.iter().any(|item| item == &key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}

fn fill_replay_or_need_target(
    text: &str,
    session: &Session,
    home: &HomeGraph,
    tokens: &[String],
    intents: &mut Vec<Intent>,
) -> Option<ParseResult> {
    if intents.is_empty() {
        if let Some(prev) = last_visible(session, home) {
            if let Some(n) = first_number(tokens) {
                let (name, slot) = if prev.starts_with("climate.") {
                    ("HassClimateSetTemperature", "temperature")
                } else if prev.starts_with("fan.") {
                    ("HassFanSetSpeed", "percentage")
                } else {
                    ("HassLightSet", "brightness")
                };
                intents.push(Intent::new(name).with("entity_id", prev).with(slot, n.to_string()));
            } else if catalog().any(tokens, &catalog().replay_on_off) {
                let name = if catalog().any(tokens, &catalog().replay_off) { "HassTurnOff" } else { "HassTurnOn" };
                intents.push(Intent::new(name).with("entity_id", prev));
            }
        } else if catalog().any(tokens, &catalog().on_words) || catalog().any(tokens, &catalog().off_words) {
            return Some(done(text, session, Vec::new(), speak_need_target(catalog().any(tokens, &catalog().off_words)), true, false));
        }
    }
    None
}

fn done(text: &str, session: &Session, intents: Vec<Intent>, speech: String, clarify: bool, chat: bool) -> ParseResult {
    ParseResult { text: text.to_string(), intents, speech, clarify, conversation_id: session.id.clone(), chat, briefing: session.briefing }
}
