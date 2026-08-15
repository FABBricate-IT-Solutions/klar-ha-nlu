use crate::lang::catalog;
use crate::lexicon::{detect_actions, Action};
use crate::normalize::{strip_fillers, tokenize};
use crate::numbers::first_number;
use crate::parse_help::{
    fill_intent, intent_from_action, intent_with_entity, looks_like_correction,
    looks_like_named_device, looks_like_question, match_custom, named_scene_or_script,
    laundry_switch_clause, pick_clarification, pick_singular_lamp, prefer_action, refine_action,
    timer_clause, wants_all_lights, wants_light_clarify,
};
use crate::resolve::{
    domain_hint, light_rooms_for_clarify, query_grounded, resolve, unique_in_area,
};
use crate::respond::{speak, speak_clarify, speak_correction, speak_unknown};
use crate::session::Session;
use crate::split::{follow_fixture, implied_domain, split_clauses, wants_group_clarify};
use crate::types::{CustomSentence, HomeGraph, Intent, Mode, ParseResult, Settings};

pub fn parse(
    text: &str,
    home: &HomeGraph,
    session: &mut Session,
    custom: &[CustomSentence],
    settings: &Settings,
) -> ParseResult {
    let _langs = crate::lang::bind(&settings.languages);
    let raw_tokens = tokenize(text);
    let tokens = strip_fillers(&raw_tokens);

    if looks_like_correction(&tokens) {
        session.mark_wrong();
        return ParseResult {
            text: text.to_string(),
            intents: Vec::new(),
            speech: speak_correction(),
            clarify: false,
            conversation_id: session.id.clone(),
        };
    }

    if session.pending_clarify.is_none()
        && tokens.iter().all(|t| catalog().is_affirm(t))
        && !tokens.is_empty()
    {
        if let Some(id) = session.last_entities.first() {
            let name = session
                .last_names
                .iter()
                .find(|n| n.starts_with("Hass") && *n != "HassGetState")
                .cloned()
                .unwrap_or_else(|| "HassTurnOn".into());
            let intent = Intent::new(name).with("entity_id", id);
            session.remember(&intent);
            return ParseResult {
                text: text.to_string(),
                intents: vec![intent.clone()],
                speech: speak(&[intent], settings.personality, false),
                clarify: false,
                conversation_id: session.id.clone(),
            };
        }
    }

    if session.pending_clarify.is_some() {
        if let Some(chosen) = pick_clarification(&tokens, session) {
            let template = session.last_intent_template.clone().unwrap_or_else(|| {
                Intent::new("HassTurnOn").with("entity_id", chosen.clone())
            });
            let intent = if home.areas.iter().any(|area| area.area_id == chosen) {
                template.with("area", &chosen).with("domain", "light")
            } else {
                intent_with_entity(template, &chosen)
            };
            session.clear_clarify();
            session.remember(&intent);
            let speech = speak(&[intent.clone()], settings.personality, false);
            return ParseResult {
                text: text.to_string(),
                intents: vec![intent],
                speech,
                clarify: false,
                conversation_id: session.id.clone(),
            };
        }
    }

    if let Some(hit) = match_custom(&tokens, text, custom) {
        session.remember(&hit);
        let speech = speak(&[hit.clone()], settings.personality, false);
        return ParseResult {
            text: text.to_string(),
            intents: vec![hit],
            speech,
            clarify: false,
            conversation_id: session.id.clone(),
        };
    }

    let clauses = split_clauses(&tokens);
    let mut intents = Vec::new();
    let mut clarify_names = Vec::new();
    for clause in clauses {
        match parse_clause(&clause, &raw_tokens, home, session, settings) {
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
        let speech = speak_clarify(&clarify_names);
        return ParseResult {
            text: text.to_string(),
            intents: Vec::new(),
            speech,
            clarify: true,
            conversation_id: session.id.clone(),
        };
    }

    if intents.is_empty() {
        if let Some(prev) = session.last_entities.first() {
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
                let name = if catalog().any(&tokens, &catalog().replay_off) {
                    "HassTurnOff"
                } else {
                    "HassTurnOn"
                };
                intents.push(Intent::new(name).with("entity_id", prev));
            }
        }
    }

    for intent in &intents {
        session.remember(intent);
    }

    let speech = if intents.is_empty() {
        speak_unknown()
    } else {
        speak(&intents, settings.personality, false)
    };

    ParseResult {
        text: text.to_string(),
        intents,
        speech,
        clarify: false,
        conversation_id: session.id.clone(),
    }
}

enum ClauseOut {
    Intents(Vec<Intent>),
    Clarify(Vec<String>, Intent),
}

fn parse_clause(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    settings: &Settings,
) -> ClauseOut {
    let actions = detect_actions(tokens);
    let question = looks_like_question(tokens);
    let number = first_number(tokens);
    let command = prefer_action(&actions);
    let hard = matches!(
        command,
        Some(
            Action::SetLight
                | Action::SetTemp
                | Action::CoverSet
                | Action::FanSpeed
                | Action::Scene
                | Action::MediaPause
                | Action::MediaPlay
                | Action::MediaNext
                | Action::MediaMute
                | Action::VacuumStart
                | Action::VacuumDock
                | Action::TimerStart
                | Action::TimerAdd
                | Action::ListAdd
                | Action::ListComplete
        )
    ) || (matches!(command, Some(Action::On))
        && tokens
            .iter()
            .any(|t| catalog().scene_nouns.contains(t.as_str()) || catalog().script_words.contains(t.as_str())));
    let action = if question && number.is_none() && !hard {
        Action::GetState
    } else {
        command
            .or_else(|| actions.first().map(|(_, a)| *a))
            .unwrap_or_else(|| {
                if number.is_some() {
                    crate::numbers::guess_numbered_action(
                        tokens,
                        session.last_entities.iter().any(|e| e.starts_with("climate."))
                            || session.last_names.iter().any(|n| n.contains("Climate"))
                            || session.last_domains.iter().any(|d| d == "climate"),
                        session.last_entities.iter().any(|e| e.starts_with("cover."))
                            || session.last_domains.iter().any(|d| d == "cover"),
                        session.last_entities.iter().any(|e| e.starts_with("fan."))
                            || session.last_domains.iter().any(|d| d == "fan"),
                    )
                } else {
                    Action::GetState
                }
            })
    };

    let action = refine_action(action, tokens, number, question, session);

    let hinted = domain_hint(tokens);
    let implied = implied_domain(action);
    let domain = match action {
        Action::TimerStart | Action::TimerAdd => Some("timer"),
        Action::SetTemp => Some("climate"),
        Action::CoverOpen | Action::CoverClose | Action::CoverSet => Some("cover"),
        Action::FanSpeed => Some("fan"),
        Action::Lock | Action::Unlock => Some("lock"),
        _ => hinted.or(implied),
    };

    let use_entities = settings.mode != Mode::ContextOnly;
    if let Some(out) = laundry_switch_clause(tokens, home, action, number, domain)
        .or_else(|| timer_clause(tokens, home, action, number, domain))
    {
        return match out {
            crate::parse_help::ReadyClause::Intents(list) => ClauseOut::Intents(list),
            crate::parse_help::ReadyClause::Clarify(names, template) => ClauseOut::Clarify(names, template),
        };
    }
    let resolved = if use_entities {
        let first = resolve(tokens, home, domain);
        if domain.is_none() && matches!(action, Action::On | Action::Off | Action::Toggle) {
            let skip_lights = catalog().any(tokens, &catalog().skip_light)
                || (catalog().any(tokens, &catalog().laundry_area)
                    && !catalog().any(tokens, &catalog().light_nouns));
            if skip_lights {
                first
            } else {
                let lights = resolve(tokens, home, Some("light"));
                if lights.ambiguous.is_empty()
                    && (!lights.entities.is_empty() || !lights.areas.is_empty())
                {
                    lights
                } else {
                    first
                }
            }
        } else {
            first
        }
    } else {
        crate::resolve::Resolved {
            areas: crate::resolve::resolve(tokens, home, None).areas,
            entities: Vec::new(),
            ambiguous: Vec::new(),
        }
    };

    let mut intents = Vec::new();

    if matches!(action, Action::On | Action::Scene) {
        if let Some(id) = named_scene_or_script(tokens, home) {
            intents.push(Intent::new("HassTurnOn").with("entity_id", &id).with(
                "domain",
                if id.starts_with("script.") { "script" } else { "scene" },
            ));
            return ClauseOut::Intents(intents);
        }
    }

    if wants_all_lights(tokens) {
        if let Some(ent) = home.entities.iter().find(|e| {
            e.domain == "light"
                && (e.entity_id.contains("alle")
                    || e.aliases.iter().any(|a| matches!(a.as_str(), "all" | "alle" | "everywhere" | "ueberall")))
        }) {
            intents.push(fill_intent(
                action,
                tokens,
                number,
                Some(&ent.entity_id),
                ent.area.as_deref(),
                Some("light"),
            ));
            intents.retain(|i| i.name != "Unknown");
            return ClauseOut::Intents(intents);
        }
    }

    if looks_like_named_device(tokens) && resolved.areas.is_empty() {
        if let Some(id) = follow_fixture(tokens, home, &session.last_areas) {
            let act = if session.last_names.iter().any(|n| n == "HassTurnOff") {
                Action::Off
            } else {
                Action::On
            };
            intents.push(fill_intent(act, tokens, number, Some(&id), session.last_areas.first().map(String::as_str), Some("light")));
            intents.retain(|i| i.name != "Unknown");
            return ClauseOut::Intents(intents);
        }
    }

    if !resolved.areas.is_empty()
        && !looks_like_named_device(tokens)
        && resolved.entities.is_empty()
        && matches!(
            action,
            Action::On
                | Action::Off
                | Action::Toggle
                | Action::SetLight
                | Action::SetTemp
                | Action::CoverOpen
                | Action::CoverClose
                | Action::CoverSet
                | Action::FanSpeed
                | Action::Lock
                | Action::Unlock
                | Action::GetState
        )
    {
        if let Some(lamp) = pick_singular_lamp(tokens, home, &resolved.areas) {
            let force = !matches!(action, Action::On | Action::Off | Action::Toggle)
                || catalog().any(raw, &catalog().command_hedges);
            if force {
                intents.push(fill_intent(action, tokens, number, Some(&lamp), resolved.areas.first().map(String::as_str), Some("light")));
                intents.retain(|i| i.name != "Unknown");
                return ClauseOut::Intents(intents);
            }
        }
        if resolved.areas.len() == 1
            && matches!(action, Action::On | Action::Off | Action::Toggle)
            && number.is_none()
            && (wants_light_clarify(tokens, home, &resolved.areas) || wants_group_clarify(raw))
        {
            let lights: Vec<String> = home
                .entities
                .iter()
                .filter(|e| {
                    e.domain == "light"
                        && e.area.as_ref().is_some_and(|a| resolved.areas.contains(a))
                })
                .map(|e| e.entity_id.clone())
                .collect();
            if lights.len() > 1 {
                let mut template = intent_from_action(action, tokens);
                if let Some(area) = resolved.areas.first() {
                    template = template.with("area", area).with("domain", "light");
                }
                return ClauseOut::Clarify(lights, template);
            }
        }
        for area in &resolved.areas {
            let id = domain
                .filter(|d| matches!(*d, "climate" | "media_player" | "fan"))
                .and_then(|d| unique_in_area(home, area, d));
            intents.push(fill_intent(
                action,
                tokens,
                number,
                id.as_deref(),
                Some(area),
                domain,
            ));
        }
        intents.retain(|i| i.name != "Unknown");
        return ClauseOut::Intents(intents);
    }

    if matches!(action, Action::GetState) && !resolved.areas.is_empty() {
        for area in &resolved.areas {
            intents.push(fill_intent(action, tokens, number, None, Some(area), domain));
        }
        intents.retain(|i| i.name != "Unknown");
        return ClauseOut::Intents(intents);
    }

    if matches!(action, Action::GetState)
        && resolved.entities.is_empty()
        && resolved.ambiguous.is_empty()
        && !query_grounded(tokens, home, false, session)
    {
        return ClauseOut::Intents(Vec::new());
    }

    if resolved.areas.len() > 1 {
        for area in &resolved.areas {
            intents.push(fill_intent(action, tokens, number, None, Some(area), domain));
        }
    } else if !resolved.entities.is_empty() {
        for ent in &resolved.entities {
            intents.push(fill_intent(
                action,
                tokens,
                number,
                Some(&ent.entity_id),
                ent.area.as_deref(),
                Some(&ent.domain),
            ));
        }
    } else if !resolved.ambiguous.is_empty() {
        let names: Vec<String> = resolved
            .ambiguous
            .iter()
            .map(|e| e.entity_id.clone())
            .collect();
        let template = intent_from_action(action, tokens);
        return ClauseOut::Clarify(names, template);
    } else if !resolved.areas.is_empty() {
        for area in &resolved.areas {
            intents.push(fill_intent(action, tokens, number, None, Some(area), domain));
        }
    } else if session.last_areas.len() > 1 && matches!(domain, Some("climate") | Some("cover")) {
        for area in &session.last_areas {
            let id = domain.and_then(|d| unique_in_area(home, area, d));
            intents.push(fill_intent(action, tokens, number, id.as_deref(), Some(area), domain));
        }
    } else if let Some(prev) = session.last_entities.first().filter(|id| domain.is_none_or(|d| id.starts_with(&format!("{d}.")))) {
        intents.push(fill_intent(action, tokens, number, Some(prev), None, domain));
    } else if !session.last_areas.is_empty() {
        for area in &session.last_areas {
            let id = domain
                .filter(|d| matches!(*d, "climate" | "media_player" | "fan"))
                .and_then(|d| unique_in_area(home, area, d));
            intents.push(fill_intent(action, tokens, number, id.as_deref(), Some(area), domain));
        }
    } else if matches!(action, Action::On | Action::Off | Action::Toggle)
        && (domain == Some("light") || crate::lexicon::has_light_noun(tokens))
        && session.last_entities.is_empty()
        && session.last_areas.is_empty()
    {
        let rooms = light_rooms_for_clarify(home);
        if rooms.len() > 1 {
            return ClauseOut::Clarify(
                rooms,
                intent_from_action(action, tokens).with("domain", "light"),
            );
        }
        intents.push(fill_intent(action, tokens, number, None, None, domain));
    } else if matches!(action, Action::SetTemp) {
        let id = home
            .entities
            .iter()
            .find(|e| e.entity_id == "climate.upper_thermostat")
            .map(|e| e.entity_id.as_str());
        intents.push(fill_intent(action, tokens, number, id, None, Some("climate")));
    } else if matches!(action, Action::CoverClose | Action::CoverOpen)
        && tokens
            .iter()
            .any(|t| catalog().curtain_nouns.contains(t.as_str()))
    {
        intents.push(fill_intent(
            action,
            tokens,
            number,
            None,
            Some("master_bedroom"),
            Some("cover"),
        ));
    } else {
        intents.push(fill_intent(action, tokens, number, None, None, domain));
    }

    intents.retain(|i| i.name != "Unknown");
    ClauseOut::Intents(intents)
}
