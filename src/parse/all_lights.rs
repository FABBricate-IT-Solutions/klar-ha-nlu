//! "All the lights" as one step per room, for homes without a whole-home light group.
use crate::home::expose::assist_visible;
use crate::home::policy::{is_infra, is_nlu_ignored};
use crate::home::roles::is_light_like;
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::clause_area::area_slots;
use crate::parse::slots::{fill_intent, ClauseOut};
use crate::types::{HomeGraph, Intent};

pub(crate) fn room_light_steps<'a>(
    rooms: impl IntoIterator<Item = &'a str>,
    tokens: &[String],
    home: &HomeGraph,
    action: Action,
    number: Option<i32>,
) -> Vec<Intent> {
    rooms
        .into_iter()
        .filter_map(|area| {
            let (id, slot, dom) = area_slots(action, area, Some("light"), home, tokens);
            let intent = fill_intent(action, tokens, number, id.as_deref(), slot.as_deref(), dom.as_deref());
            (intent.name != "Unknown").then_some(intent)
        })
        .collect()
}

/// Every room with a light, plus each light that has no room. Klar never sends a step
/// without a target, so "all lights" can't be a single home-wide step here.
pub(crate) fn every_light(tokens: &[String], home: &HomeGraph, action: Action, number: Option<i32>) -> Option<ClauseOut> {
    let lights: Vec<_> = home
        .entities
        .iter()
        .filter(|e| is_light_like(e, catalog()) && assist_visible(e, home) && !is_infra(e) && !is_nlu_ignored(e))
        .collect();
    let mut rooms: Vec<&str> = lights.iter().filter_map(|e| e.area.as_deref()).collect();
    rooms.sort_unstable();
    rooms.dedup();
    let mut steps = room_light_steps(rooms, tokens, home, action, number);
    steps.extend(
        lights.iter().filter(|e| e.area.is_none()).map(|e| fill_intent(action, tokens, number, Some(&e.entity_id), None, Some("light"))),
    );
    (!steps.is_empty()).then_some(ClauseOut::Intents(steps))
}
