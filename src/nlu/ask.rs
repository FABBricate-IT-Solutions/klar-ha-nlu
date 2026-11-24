//! The confirmation question says what will happen and to which device.
//!
//! "Should I really unlock Front Door?" instead of "Should I really do that?". A language
//! without action wording names the devices and then asks its general question.

use crate::parse::respond::spoken_where;
use crate::types::{HomeGraph, Intent, IntentPlan};

use super::context::ParseContext;

/// More names than this are spoken as a count.
const MAX_NAMED: usize = 3;

pub(super) fn confirmation_prompt(context: &ParseContext<'_>, plan: &IntentPlan) -> String {
    let speech = context.catalog.speech();
    let general = speech.confirm.to_string();
    let mut groups: Vec<(&'static str, Option<String>, Vec<String>)> = Vec::new();
    for intent in plan.intents() {
        // A question to the house changes nothing and needs no naming.
        if matches!(intent.name.as_str(), "HassGetState" | "HassClimateGetTemperature") {
            continue;
        }
        let (key, value) = action_key(&intent);
        let names = step_targets(&intent, context.home);
        match groups.iter_mut().find(|(known, number, _)| *known == key && *number == value) {
            Some((_, _, listed)) => listed.extend(names),
            None => groups.push((key, value, names)),
        }
    }
    for (_, _, names) in &mut groups {
        *names = distinct(std::mem::take(names));
    }
    let Some(wording) = context.catalog.ask_wording() else {
        let names: Vec<String> = groups.into_iter().flat_map(|(_, _, names)| names).collect();
        return if names.is_empty() || names.len() > MAX_NAMED { general } else { format!("{}. {general}", join(&names, speech.and_join)) };
    };
    let clauses: Vec<String> = groups
        .iter()
        .filter(|(_, _, names)| !names.is_empty())
        .filter_map(|(key, value, names)| {
            let template = wording
                .actions
                .iter()
                .find(|(known, _)| known == key)
                .or_else(|| wording.actions.iter().find(|(known, _)| *known == "other"))?
                .1;
            let target =
                if names.len() > MAX_NAMED { wording.many.replace("{n}", &names.len().to_string()) } else { join(names, speech.and_join) };
            Some(template.replace("{target}", &target).replace("{n}", value.as_deref().unwrap_or("")))
        })
        .collect();
    if clauses.is_empty() {
        return general;
    }
    wording.ask.replace("{actions}", &join(&clauses, speech.and_join))
}

fn domain_of(intent: &Intent) -> &str {
    intent.slot("domain").or_else(|| intent.slot("entity_id").and_then(|id| id.split_once('.').map(|(domain, _)| domain))).unwrap_or("")
}

/// Which wording fits the step, and the number it sets, if any.
fn action_key(intent: &Intent) -> (&'static str, Option<String>) {
    let number = |slot: &str| intent.slot(slot).map(str::to_string);
    match (intent.name.as_str(), domain_of(intent)) {
        ("HassTurnOn", "lock") => ("lock", None),
        ("HassTurnOff", "lock") => ("unlock", None),
        ("HassTurnOn", "cover") => ("open", None),
        ("HassTurnOff", "cover") => ("close", None),
        ("HassTurnOn", "script" | "scene") => ("start", None),
        ("HassTurnOn", _) => ("on", None),
        ("HassTurnOff", _) => ("off", None),
        ("HassSetPosition", _) if intent.slot("position").is_some() => ("percent", number("position")),
        ("HassLightSet", _) if intent.slot("brightness").is_some() => ("percent", number("brightness")),
        ("HassFanSetSpeed", _) if intent.slot("percentage").is_some() => ("percent", number("percentage")),
        ("HassClimateSetTemperature", _) if intent.slot("temperature").is_some() => ("degrees", number("temperature")),
        _ => ("other", None),
    }
}

/// The devices a step reaches, by their spoken names. A room-wide step lists the devices
/// of its kind in that room, as Home Assistant will act on them.
fn step_targets(intent: &Intent, home: &HomeGraph) -> Vec<String> {
    let whole = vec![spoken_where(intent, Some(home))];
    if intent.slot("entity_id").is_some() {
        return whole;
    }
    let domain = domain_of(intent);
    let areas: Vec<&str> = match (intent.slot("area"), intent.slot("floor")) {
        (Some(area), _) => vec![area],
        (None, Some(floor)) => {
            home.areas.iter().filter(|area| area.floor_id.as_deref() == Some(floor)).map(|area| area.area_id.as_str()).collect()
        }
        (None, None) => return whole,
    };
    let reached: Vec<String> = home
        .entities
        .iter()
        .filter(|entity| entity.domain == domain && entity.area.as_deref().is_some_and(|area| areas.contains(&area)))
        .filter(|entity| home.assist.as_ref().is_none_or(|exposed| exposed.contains(&entity.entity_id)))
        .map(|entity| spoken_where(&Intent::new(&intent.name).with("entity_id", &entity.entity_id), Some(home)))
        .collect();
    if reached.is_empty() {
        whole
    } else {
        reached
    }
}

fn distinct(names: Vec<String>) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for name in names {
        if !name.is_empty() && !kept.contains(&name) {
            kept.push(name);
        }
    }
    kept
}

fn join(names: &[String], and: &str) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [head @ .., last] => format!("{}{and}{last}", head.join(", ")),
    }
}
