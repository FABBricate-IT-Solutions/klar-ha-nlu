//! Household specials: teach a name, explain, undo, clock, weather, routines.

use crate::home::expose::assist_visible;
use crate::lang::Household;
use crate::parse::normalize::fold_umlaut;
use crate::parse::normalize::umlaut_eq;
use crate::session::LastHeard;
use crate::types::{Intent, RejectReason};

use super::context::ParseContext;
use super::draft::{chat, execute, reject, Draft};

pub(super) fn route(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
    if tokens.is_empty() {
        return None;
    }
    let blob = fold_umlaut(context.text.trim()).replace(['\'', '’', '`'], "");
    if let Some(draft) = match_routine(context, &blob) {
        return Some(draft);
    }
    if context.catalog.household_hit(&blob, |pack| pack.household.explain) {
        return Some(explain(context));
    }
    if context.catalog.household_hit(&blob, |pack| pack.household.undo) {
        return Some(undo(context));
    }
    if let Some(alias) = context.catalog.household_prefix(&blob, |pack| pack.household.teach) {
        return Some(teach(context, &alias));
    }
    if looks_like_clock(context, &blob, tokens) {
        return Some(clock(context));
    }
    let pack_weather = context.catalog.household_hit(&blob, |pack| pack.household.weather);
    if let Some(class) = super::weather_lex::classify(&blob, pack_weather) {
        if calendar_overrides_weather(context, tokens) {
            return None;
        }
        if !class.ask.skips_climate() && climate_overrides_weather(context, tokens) {
            return None;
        }
        return Some(weather(context, class));
    }
    None
}

fn calendar_overrides_weather(context: &ParseContext<'_>, tokens: &[String]) -> bool {
    context.catalog.any(tokens, context.catalog.calendar_nouns())
}

/// Generated packs store `{query} {climate}` as a weather phrase. A room or
/// extra tokens means indoor climate, not the forecast entity.
fn climate_overrides_weather(context: &ParseContext<'_>, tokens: &[String]) -> bool {
    let blob = fold_umlaut(context.text.trim()).replace(['\'', '’', '`'], "");
    if super::weather_lex::outdoor_temp_ask(&blob) {
        return false;
    }
    let cat = context.catalog;
    if !cat.any(tokens, cat.climate_nouns()) {
        return false;
    }
    let has_area = context.home.areas.iter().any(|area| {
        tokens.iter().any(|token| {
            fold_umlaut(&area.area_id) == *token
                || fold_umlaut(&area.name) == *token
                || umlaut_eq(&fold_umlaut(&area.area_id), token)
                || umlaut_eq(&fold_umlaut(&area.name), token)
                || area.aliases.iter().any(|alias| fold_umlaut(alias) == *token || umlaut_eq(&fold_umlaut(alias), token))
        })
    });
    has_area || tokens.len() > 2
}

fn match_routine(context: &ParseContext<'_>, blob: &str) -> Option<Draft> {
    for row in context.custom {
        let script = row.slots.get("entity_id")?;
        if row.intent != "HassTurnOn" || !script.starts_with("script.") {
            continue;
        }
        if !phrase_hit(blob, &row.phrase) {
            continue;
        }
        return Some(execute(
            context,
            vec![Intent::new("HassTurnOn").with("entity_id", script).with("domain", "script")],
            "household_routine",
            1.0,
            1.0,
            false,
            false,
        ));
    }
    None
}

fn explain(context: &ParseContext<'_>) -> Draft {
    let house = templates(context);
    let Some(heard) = context.session.last_heard.as_ref() else {
        return chat(house.heard_nothing.into(), false, false);
    };
    chat(format_explain(heard, house), false, false)
}

fn format_explain(heard: &LastHeard, house: &Household) -> String {
    let area = heard.area.as_deref().filter(|area| !area.is_empty());
    let where_ = area.map(|area| house.in_area.replace("{area}", area)).unwrap_or_default();
    let heard_line = house.heard.replace("{text}", &heard.text);
    let why = match heard.decision.as_str() {
        "execute" => house.executed.replace("{names}", &heard.names.join(", ")),
        "confirm" => house.asked_risky.to_string(),
        "clarify" => house.unclear_device.to_string(),
        "reject" => house.stopped.replace("{reason}", heard.reason.as_deref().unwrap_or(house.no_match)),
        "chat" => house.was_chat.to_string(),
        _ => house.decision.replace("{decision}", &heard.decision),
    };
    format!("{heard_line}{why}{where_}")
}

fn undo(context: &ParseContext<'_>) -> Draft {
    let house = templates(context);
    let inverted: Vec<Intent> = context.session.last_execute.iter().filter_map(invert_intent).collect();
    if inverted.is_empty() {
        return reject(RejectReason::NoAction, house.nothing_undo.into());
    }
    execute(context, inverted, "household_undo", 1.0, 1.0, true, true)
}

fn teach(context: &ParseContext<'_>, alias: &str) -> Draft {
    let house = templates(context);
    let Some(entity_id) = context.session.last.iter().find_map(|turn| turn.entity.clone()) else {
        return reject(RejectReason::NoTarget, house.teach_which.into());
    };
    if !valid_alias(alias) {
        return reject(RejectReason::InvalidInput, house.teach_invalid.into());
    }
    let mut draft = chat(house.teach_ok.replace("{alias}", alias), false, false);
    draft.commit.teach = Some((entity_id, alias.to_string()));
    draft
}

fn clock(context: &ParseContext<'_>) -> Draft {
    let house = templates(context);
    chat(spoken_clock(house), false, false)
}

fn weather(context: &ParseContext<'_>, class: super::weather_lex::WeatherClass) -> Draft {
    let house = templates(context);
    let entity = context.home.entities.iter().find(|entity| entity.domain == "weather" && assist_visible(entity, context.home));
    match entity {
        Some(entity) => {
            let mut intent = Intent::new("HassGetState")
                .with("entity_id", &entity.entity_id)
                .with("domain", "weather")
                .with("weather_ask", class.ask.as_str());
            if let Some(day) = class.day.as_str() {
                intent = intent.with("weather_day", day);
            }
            if let Some(part) = class.part {
                intent = intent.with("weather_part", part.as_str());
            }
            execute(context, vec![intent], "household_weather", 1.0, 1.0, false, false)
        }
        None => chat(house.no_weather.into(), false, false),
    }
}

pub(crate) fn invert_intent(intent: &Intent) -> Option<Intent> {
    let name = match intent.name.as_str() {
        "HassTurnOn" => "HassTurnOff",
        "HassTurnOff" => "HassTurnOn",
        "HassToggle" => "HassToggle",
        "HassMediaPause" => "HassMediaUnpause",
        "HassMediaUnpause" => "HassMediaPause",
        "HassMediaPlayerMute" => "HassMediaPlayerUnmute",
        "HassMediaPlayerUnmute" => "HassMediaPlayerMute",
        "HassVacuumStart" => "HassVacuumReturnToBase",
        "HassStartTimer" => "HassCancelTimer",
        "HassOpenCover" => "HassCloseCover",
        "HassCloseCover" => "HassOpenCover",
        "HassLock" => "HassUnlock",
        "HassUnlock" => "HassLock",
        _ => return None,
    };
    Some(Intent { name: name.into(), slots: intent.slots.clone() })
}

fn looks_like_clock(context: &ParseContext<'_>, blob: &str, tokens: &[String]) -> bool {
    if context.catalog.pack_any(tokens, |pack| pack.household.clock_skip) {
        return false;
    }
    context.catalog.household_hit(blob, |pack| pack.household.clock)
}

fn spoken_clock(house: &Household) -> String {
    match utc_hhmm() {
        Some(time) => house.clock_ok.replace("{time}", &time),
        None => house.clock_missing.to_string(),
    }
}

fn utc_hhmm() -> Option<String> {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let minutes = (secs / 60) % (24 * 60);
    Some(format!("{:02}:{:02}", minutes / 60, minutes % 60))
}

const FALLBACK: Household = Household {
    teach: &[],
    explain: &[],
    undo: &[],
    clock: &[],
    weather: &[],
    clock_skip: &[],
    heard_nothing: "",
    heard: "{text}",
    executed: "{names}",
    asked_risky: "",
    unclear_device: "",
    stopped: "{reason}",
    no_match: "",
    was_chat: "",
    decision: "{decision}",
    in_area: "{area}",
    nothing_undo: "",
    teach_which: "",
    teach_invalid: "",
    teach_ok: "{alias}",
    clock_ok: "{time}",
    clock_missing: "",
    no_weather: "",
};

fn templates(context: &ParseContext<'_>) -> &'static Household {
    context.catalog.household().unwrap_or(&FALLBACK)
}

fn phrase_hit(blob: &str, phrase: &str) -> bool {
    let candidate = fold_umlaut(phrase.trim());
    !candidate.is_empty() && (blob == candidate || blob.contains(&candidate))
}

fn valid_alias(alias: &str) -> bool {
    let chars = alias.chars().count();
    (2..=40).contains(&chars) && !alias.chars().any(|ch| ch.is_control())
}

#[cfg(test)]
#[path = "household_tests.rs"]
mod tests;
