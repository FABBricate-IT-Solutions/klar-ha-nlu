//! "All the lights" / "all the blinds" as one step per room (or per unassigned device).
use crate::home::expose::assist_visible;
use crate::home::policy::{is_infra, is_nlu_ignored};
use crate::home::roles::is_light_like;
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::clause_area::area_slots;
use crate::parse::infer::{except_tail, wants_all_covers};
use crate::parse::slots::{fill_intent, ClauseOut};
use crate::types::{EntityRec, HomeGraph, Intent};

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
        .filter(|e| is_light_like(e, &catalog()) && assist_visible(e, home) && !is_infra(e) && !is_nlu_ignored(e))
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

/// "Close all the blinds" / bare "stop the blinds": one step per matching cover (P-21).
pub(crate) fn every_cover(tokens: &[String], home: &HomeGraph, action: Action, number: Option<i32>) -> Option<ClauseOut> {
    let covers: Vec<_> = home
        .entities
        .iter()
        .filter(|e| {
            e.domain == "cover" && assist_visible(e, home) && !is_infra(e) && !is_nlu_ignored(e) && cover_matches_spoken(e, tokens, home)
        })
        .collect();
    let steps: Vec<Intent> = covers
        .iter()
        .map(|e| fill_intent(action, tokens, number, Some(&e.entity_id), None, Some("cover")))
        .filter(|intent| intent.name != "Unknown")
        .collect();
    (!steps.is_empty()).then_some(ClauseOut::Intents(steps))
}

/// "blinds" / "Rollos" must not close a garage door that happens to live in `cover`.
fn cover_matches_spoken(entity: &EntityRec, tokens: &[String], home: &HomeGraph) -> bool {
    const BLIND: &[&str] =
        &["blinds", "blind", "curtain", "curtains", "shade", "shades", "shutter", "shutters", "rollo", "rollos", "rolladen", "jalousie"];
    let spoken_blind = tokens.iter().any(|token| BLIND.contains(&token.as_str()));
    if !spoken_blind {
        return true;
    }
    if home.device_class(&entity.entity_id).is_some_and(|class| matches!(class, "blind" | "curtain" | "shade" | "shutter")) {
        return true;
    }
    let label = format!("{} {}", entity.name, entity.entity_id).to_ascii_lowercase();
    BLIND.iter().any(|word| label.contains(word))
}

pub(crate) fn all_covers_clause(
    tokens: &[String],
    home: &HomeGraph,
    action: Action,
    number: Option<i32>,
    areas: &[String],
) -> Option<ClauseOut> {
    let bare_stop = matches!(action, Action::CoverStop) && areas.is_empty() && !wants_all_covers(tokens);
    if !wants_all_covers(tokens) && !bare_stop {
        return None;
    }
    // Full-sentence cover exceptions go through nlu/except; do not double-claim here.
    if except_tail(tokens).is_some() {
        return None;
    }
    every_cover(tokens, home, action, number)
}
