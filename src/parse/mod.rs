pub mod action;
pub(crate) mod chat;
pub(crate) mod clause;
pub(crate) mod clause_support;
pub mod compound;
pub(crate) mod fuzzy;
pub(crate) mod infer;
pub(crate) mod media;
pub mod normalize;
pub mod numbers;
pub(crate) mod policy;
pub mod resolve;
pub(crate) mod resolve_named;
pub mod respond;
pub(crate) mod slots;
pub mod split;

#[cfg(debug_assertions)]
use crate::lang::catalog;
#[cfg(debug_assertions)]
use crate::parse::chat::{briefing_followup, is_news, is_news_dismiss, is_ood, looks_like_home, wants_llm};
#[cfg(debug_assertions)]
use crate::parse::clause::parse_clause;
#[cfg(debug_assertions)]
use crate::parse::clause_support::last_visible;
#[cfg(debug_assertions)]
use crate::parse::compound::{expand_compounds, CompoundSplit};
#[cfg(debug_assertions)]
use crate::parse::infer::{looks_like_correction, match_custom, pick_clarification};
#[cfg(debug_assertions)]
use crate::parse::normalize::{strip_fillers, tokenize};
#[cfg(debug_assertions)]
use crate::parse::numbers::first_number;
#[cfg(debug_assertions)]
use crate::parse::respond::{speak, speak_clarify, speak_correction, speak_need_target, speak_unknown};
#[cfg(debug_assertions)]
use crate::parse::slots::{intent_with_entity, ClauseOut};
#[cfg(debug_assertions)]
use crate::parse::split::split_clauses;
use crate::session::Session;
#[cfg(debug_assertions)]
use crate::types::Intent;
use crate::types::{CustomSentence, HomeGraph, ParseResult, Settings};

pub fn parse(text: &str, home: &HomeGraph, session: &mut Session, custom: &[CustomSentence], settings: &Settings) -> ParseResult {
    #[cfg(debug_assertions)]
    {
        let (current, _) = parse_checked(text, home, session, custom, settings);
        current
    }
    #[cfg(not(debug_assertions))]
    {
        let mut compatibility = settings.clone();
        compatibility.confirm_risky_actions = false;
        crate::nlu::parse_compatible(text, home, session, custom, &compatibility)
    }
}

#[cfg(debug_assertions)]
#[doc(hidden)]
pub fn parse_checked(
    text: &str,
    home: &HomeGraph,
    session: &mut Session,
    custom: &[CustomSentence],
    settings: &Settings,
) -> (ParseResult, Option<String>) {
    let mut compatibility = settings.clone();
    compatibility.confirm_risky_actions = false;
    let mut legacy_session = session.clone();
    let pending_template = legacy_session.pending_clarify().map(|pending| pending.template.clone());
    if let Some(template) = &pending_template {
        legacy_session.remember(template);
    }
    let legacy = parse_v1(text, home, &mut legacy_session, custom, settings);
    if let Some(template) = &pending_template {
        legacy_session.last.retain(|turn| {
            turn.name != template.name
                || turn.entity.as_deref() != template.slot("entity_id")
                || turn.area.as_deref() != template.slot("area")
        });
    }
    let current = crate::nlu::parse_compatible(text, home, session, custom, &compatibility);
    if !legacy.clarify && !legacy.intents.is_empty() && session.pending_clarify().is_none() && session.pending_confirm().is_none() {
        // V1 can retain a consumed clarification marker when its parser,
        // rather than `pick_clarification`, resolves the follow-up.
        legacy_session.clear_pending();
    }
    let parity_error = v1_v2_parity_error(text, &legacy, &current, &legacy_session, session);
    (current, parity_error)
}

#[cfg(debug_assertions)]
fn parse_v1(text: &str, home: &HomeGraph, session: &mut Session, custom: &[CustomSentence], settings: &Settings) -> ParseResult {
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

#[cfg(debug_assertions)]
fn v1_v2_parity_error(
    text: &str,
    legacy: &ParseResult,
    current: &ParseResult,
    legacy_session: &Session,
    current_session: &Session,
) -> Option<String> {
    let result_matches = legacy.text == current.text
        && legacy.intents == current.intents
        && legacy.speech == current.speech
        && legacy.clarify == current.clarify
        && legacy.chat == current.chat
        && legacy.briefing == current.briefing;
    let legacy_state = legacy_session.parity_snapshot();
    let current_state = current_session.parity_snapshot();
    (!result_matches || legacy_state != current_state).then(|| {
        format!(
            "V1/V2 parity mismatch for {text:?}: legacy={legacy:?} current={current:?} \
             legacy_state={legacy_state:?} current_state={current_state:?}"
        )
    })
}

#[cfg(debug_assertions)]
fn preprocess(text: &str, home: &HomeGraph) -> (Vec<String>, CompoundSplit) {
    let raw_tokens = tokenize(text);
    let split = expand_compounds(&strip_fillers(&raw_tokens), home);
    (raw_tokens, split)
}

#[cfg(debug_assertions)]
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
    if is_ood(raw_tokens, home) || is_ood(tokens, home) {
        return Some(done(text, session, Vec::new(), speak_unknown(), false, false));
    }
    if looks_like_correction(tokens) {
        session.mark_wrong();
        return Some(done(text, session, Vec::new(), speak_correction(), false, false));
    }
    None
}

#[cfg(debug_assertions)]
fn session_followups(
    text: &str,
    session: &mut Session,
    home: &HomeGraph,
    tokens: &[String],
    custom: &[CustomSentence],
    settings: &Settings,
) -> Option<ParseResult> {
    if session.pending_clarify().is_none() && tokens.iter().all(|t| catalog().is_affirm(t)) && !tokens.is_empty() {
        if let Some(id) = last_visible(session, home) {
            let name = session.last_names().find(|n| n.starts_with("Hass") && *n != "HassGetState").unwrap_or("HassTurnOn").to_string();
            let intent = Intent::new(name).with("entity_id", id);
            session.remember(&intent);
            let speech = speak(std::slice::from_ref(&intent), settings.personality, false, Some(home));
            return Some(done(text, session, vec![intent], speech, false, false));
        }
    }
    if session.pending_clarify().is_some() {
        if let Some(chosen) = pick_clarification(tokens, session) {
            let template = session
                .pending_clarify()
                .map(|state| state.template.clone())
                .unwrap_or_else(|| Intent::new("HassTurnOn").with("entity_id", chosen.clone()));
            let intent = if home.areas.iter().any(|area| area.area_id == chosen) {
                template.with("area", &chosen).with("domain", "light")
            } else {
                intent_with_entity(template, &chosen)
            };
            session.clear_pending();
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

#[cfg(debug_assertions)]
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
                session.remember(&template);
                session.set_clarify(names.clone(), template);
                clarify_names = names;
            }
        }
    }
    (intents, clarify_names)
}

#[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
fn done(text: &str, session: &Session, intents: Vec<Intent>, speech: String, clarify: bool, chat: bool) -> ParseResult {
    ParseResult { text: text.to_string(), intents, speech, clarify, conversation_id: session.id.clone(), chat, briefing: session.briefing }
}
