use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::catalog;
use crate::parse::action::{has_light_noun, Action};
use crate::parse::compound::named_scene_or_script;
use crate::parse::infer::{color_word, light_level};
use crate::parse::media::media_transport_form;
use crate::parse::resolve::resolve;
use crate::parse::slots::ClauseOut;
use crate::session::Session;
use crate::types::{HomeGraph, Intent, Mode, Settings};

pub(crate) fn resolve_targets(
    tokens: &[String],
    home: &HomeGraph,
    settings: &Settings,
    domain: Option<&str>,
    action: Action,
) -> crate::parse::resolve::Resolved {
    if settings.mode == Mode::ContextOnly {
        let resolved = resolve(tokens, home, None);
        return crate::parse::resolve::Resolved {
            areas: resolved.areas,
            floors: resolved.floors,
            entities: Vec::new(),
            ambiguous: Vec::new(),
            ask_areas: Vec::new(),
        };
    }
    let first = resolve(tokens, home, domain);
    if domain.is_none() && matches!(action, Action::On | Action::Off | Action::Toggle) {
        let named_other = first.entities.iter().any(|entity| entity.domain != "light");
        let skip_lights = named_other
            || catalog().any(tokens, catalog().skip_light())
            || (catalog().any(tokens, catalog().laundry_area()) && !catalog().any(tokens, catalog().light_nouns()));
        if !skip_lights {
            let lights = resolve(tokens, home, Some("light"));
            if lights.ambiguous.is_empty() && (!lights.entities.is_empty() || !lights.areas.is_empty()) {
                return lights;
            }
        }
    }
    first
}

/// "set the hallway radiator to 21": no word says what kind of device is meant, so the
/// number alone made it a light command. A device the home names in the sentence decides
/// instead, so a thermostat is not passed over for the lights of its room.
pub(crate) fn named_instead_of_light(
    action: Action,
    tokens: &[String],
    number: Option<i32>,
    home: &HomeGraph,
    settings: &Settings,
) -> Option<&'static str> {
    if action != Action::SetLight
        || number.is_none()
        || settings.mode == Mode::ContextOnly
        || has_light_noun(tokens)
        || color_word(tokens).is_some()
        || light_level(tokens).is_some()
    {
        return None;
    }
    let named = resolve(tokens, home, None).entities;
    if named.is_empty() {
        return None;
    }
    ["climate", "cover", "fan"].into_iter().find(|domain| named.iter().all(|entity| entity.domain == *domain))
}

pub(crate) fn preferred_area_domain(domain: Option<&str>, action: Action, tokens: &[String]) -> Option<&'static str> {
    let cat = catalog();
    if (domain == Some("light") || has_light_noun(tokens))
        && matches!(action, Action::On | Action::Off | Action::Toggle | Action::SetLight | Action::GetState)
    {
        return Some("light");
    }
    if (domain == Some("fan") || cat.any(tokens, cat.fan_nouns()))
        && matches!(action, Action::On | Action::Off | Action::Toggle | Action::FanSpeed | Action::GetState)
    {
        return Some("fan");
    }
    if (domain == Some("cover") || crate::parse::action::has_cover_noun(tokens))
        && matches!(
            action,
            Action::CoverOpen | Action::CoverClose | Action::CoverSet | Action::On | Action::Off | Action::Toggle | Action::GetState
        )
    {
        return Some("cover");
    }
    if (domain == Some("climate") || crate::parse::action::has_climate_noun(tokens))
        && matches!(action, Action::SetTemp | Action::On | Action::Off | Action::Toggle | Action::GetState)
    {
        return Some("climate");
    }
    if (domain == Some("media_player") || cat.any(tokens, cat.media_nouns()))
        && matches!(
            action,
            Action::MediaPause | Action::MediaPlay | Action::MediaNext | Action::MediaMute | Action::On | Action::Off | Action::GetState
        )
    {
        return Some("media_player");
    }
    if crate::home::roles::tv_asked(tokens) {
        return Some("media_player");
    }
    // Bare on/off at a satellite with no device noun still means the room lights.
    // Do not bind GetState here: mid-sentence "ist" must not become a light status ask.
    if domain.is_none() && matches!(action, Action::On | Action::Off | Action::Toggle) {
        return Some("light");
    }
    None
}

pub(crate) fn named_scene_policy(tokens: &[String], home: &HomeGraph, question: bool, action: Action) -> Option<ClauseOut> {
    let asked = catalog().any(tokens, catalog().question_words()) || tokens.first().is_some_and(|token| catalog().is_question_start(token));
    if (question && asked)
        || media_transport_form(tokens, action)
        || !matches!(action, Action::On | Action::Off | Action::Scene | Action::GetState)
    {
        return None;
    }
    // A fully named device ("Garage Door") beats a script that only shares a word ("Open Garage").
    let linked = crate::parse::resolve::link_named_spans(tokens, home, &[]);
    if linked.entities.iter().any(|entity| entity.domain != "scene" && entity.domain != "script") {
        return None;
    }
    let id = named_scene_or_script(tokens, home)?;
    Some(ClauseOut::Intents(vec![Intent::new("HassTurnOn")
        .with("entity_id", &id)
        .with("domain", if id.starts_with("script.") { "script" } else { "scene" })]))
}

pub(crate) fn last_visible<'a>(session: &'a Session, home: &'a HomeGraph) -> Option<&'a str> {
    last_matching(session, home, None).into_iter().next()
}

pub(crate) fn last_matching<'a>(session: &'a Session, home: &'a HomeGraph, domain: Option<&str>) -> Vec<&'a str> {
    session
        .last_entities()
        .filter(|id| home.entities.iter().any(|entity| entity.entity_id == *id && assist_visible(entity, home) && !is_infra(entity)))
        .filter(|id| domain.is_none_or(|wanted| id.starts_with(&format!("{wanted}."))))
        .collect()
}
