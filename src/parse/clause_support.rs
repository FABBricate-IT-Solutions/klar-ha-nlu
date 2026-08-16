use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::catalog;
use crate::parse::action::{has_light_noun, Action};
use crate::parse::compound::named_scene_or_script;
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
        };
    }
    let first = resolve(tokens, home, domain);
    if domain.is_none() && matches!(action, Action::On | Action::Off | Action::Toggle) {
        let named_other = first.entities.iter().any(|entity| entity.domain != "light");
        let skip_lights = named_other
            || catalog().any(tokens, &catalog().skip_light)
            || (catalog().any(tokens, &catalog().laundry_area) && !catalog().any(tokens, &catalog().light_nouns));
        if !skip_lights {
            let lights = resolve(tokens, home, Some("light"));
            if lights.ambiguous.is_empty() && (!lights.entities.is_empty() || !lights.areas.is_empty()) {
                return lights;
            }
        }
    }
    first
}

pub(crate) fn preferred_area_domain(domain: Option<&str>, action: Action, tokens: &[String]) -> Option<&'static str> {
    let cat = catalog();
    if (domain == Some("light") || has_light_noun(tokens))
        && matches!(action, Action::On | Action::Off | Action::Toggle | Action::SetLight | Action::GetState)
    {
        return Some("light");
    }
    if domain == Some("fan") && matches!(action, Action::On | Action::Off | Action::Toggle | Action::FanSpeed | Action::GetState) {
        return Some("fan");
    }
    if domain == Some("media_player") && cat.any(tokens, &cat.media_nouns) {
        return Some("media_player");
    }
    None
}

pub(crate) fn named_scene_policy(tokens: &[String], home: &HomeGraph, question: bool, action: Action) -> Option<ClauseOut> {
    if question || media_transport_form(tokens, action) || !matches!(action, Action::On | Action::Scene | Action::GetState) {
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
