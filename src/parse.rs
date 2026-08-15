use crate::chat::wants_llm;
use crate::clause::{last_visible, parse_clause};
use crate::compound::expand_compounds;
use crate::lang::catalog;
use crate::normalize::{strip_fillers, tokenize};
use crate::numbers::first_number;
use crate::parse_help::{looks_like_correction, match_custom, pick_clarification};
use crate::parse_slots::{intent_with_entity, ClauseOut};
use crate::respond::{speak, speak_clarify, speak_correction, speak_need_target, speak_unknown};
use crate::session::Session;
use crate::split::split_clauses;
use crate::types::{CustomSentence, HomeGraph, Intent, ParseResult, Settings};

pub fn parse(text: &str, home: &HomeGraph, session: &mut Session, custom: &[CustomSentence], settings: &Settings) -> ParseResult {
    let _langs = crate::lang::bind(&settings.languages);
    let raw_tokens = tokenize(text);
    let split = expand_compounds(&strip_fillers(&raw_tokens), home);
    let tokens = split.tokens;

    if wants_llm(&raw_tokens, home) || wants_llm(&tokens, home) {
        return done(text, session, Vec::new(), String::new(), false, true);
    }
    if looks_like_correction(&tokens) {
        session.mark_wrong();
        return done(text, session, Vec::new(), speak_correction(), false, false);
    }
    if session.pending_clarify.is_none() && tokens.iter().all(|t| catalog().is_affirm(t)) && !tokens.is_empty() {
        if let Some(id) = last_visible(session, home) {
            let name = session
                .last_names
                .iter()
                .find(|n| n.starts_with("Hass") && *n != "HassGetState")
                .cloned()
                .unwrap_or_else(|| "HassTurnOn".into());
            let intent = Intent::new(name).with("entity_id", id);
            session.remember(&intent);
            let speech = speak(std::slice::from_ref(&intent), settings.personality, false, Some(home));
            return done(text, session, vec![intent], speech, false, false);
        }
    }
    if session.pending_clarify.is_some() {
        if let Some(chosen) = pick_clarification(&tokens, session) {
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
            return done(text, session, vec![intent], speech, false, false);
        }
    }
    if let Some(hit) = match_custom(&tokens, text, custom) {
        session.remember(&hit);
        let speech = speak(std::slice::from_ref(&hit), settings.personality, false, Some(home));
        return done(text, session, vec![hit], speech, false, false);
    }

    let clauses = split_clauses(&tokens, home);
    let mut intents = Vec::new();
    let mut clarify_names = Vec::new();
    for clause in clauses {
        match parse_clause(&clause, &raw_tokens, home, session, settings, &split.light_areas) {
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
    if !clarify_names.is_empty() {
        return done(text, session, Vec::new(), speak_clarify(&clarify_names, Some(home)), true, false);
    }
    if intents.is_empty() {
        if let Some(prev) = last_visible(session, home) {
            if let Some(n) = first_number(&tokens) {
                let (name, slot) = if prev.starts_with("climate.") {
                    ("HassClimateSetTemperature", "temperature")
                } else if prev.starts_with("fan.") {
                    ("HassFanSetSpeed", "percentage")
                } else {
                    ("HassLightSet", "brightness")
                };
                intents.push(Intent::new(name).with("entity_id", prev).with(slot, n.to_string()));
            } else if catalog().any(&tokens, &catalog().replay_on_off) {
                let name = if catalog().any(&tokens, &catalog().replay_off) { "HassTurnOff" } else { "HassTurnOn" };
                intents.push(Intent::new(name).with("entity_id", prev));
            }
        } else if catalog().any(&tokens, &catalog().on_words) || catalog().any(&tokens, &catalog().off_words) {
            return done(text, session, Vec::new(), speak_need_target(catalog().any(&tokens, &catalog().off_words)), true, false);
        }
    }
    for intent in &intents {
        session.remember(intent);
    }
    let speech = if intents.is_empty() { speak_unknown() } else { speak(&intents, settings.personality, false, Some(home)) };
    done(text, session, intents, speech, false, false)
}

fn done(text: &str, session: &Session, intents: Vec<Intent>, speech: String, clarify: bool, chat: bool) -> ParseResult {
    ParseResult { text: text.to_string(), intents, speech, clarify, conversation_id: session.id.clone(), chat }
}
