//! Household specials: teach a name, explain, undo, clock, date, weather, routines.

use crate::home::expose::assist_visible;
use crate::lang::Household;
use crate::parse::normalize::fold_umlaut;
use crate::parse::normalize::umlaut_eq;
use crate::session::LastHeard;
use crate::types::{Intent, RejectReason};

use super::context::ParseContext;
use super::draft::{chat, clock_chat, execute, reject, Draft};
use super::phrase::{after_prefix, has_words, is_any_phrase, is_phrase, words, Slack};
use super::risk::risky_step;
pub(crate) use super::undo::invert_intent;

pub(super) fn route(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
    if tokens.is_empty() {
        return None;
    }
    let cat = context.catalog.as_ref();
    let blob = fold_umlaut(context.text.trim()).replace(['\'', '’', '`'], "");
    let said = words(context.text);
    if let Some(draft) = match_routine(context, &said) {
        return Some(draft);
    }
    if is_any_phrase(&said, cat.household_phrases(|pack| pack.household.explain), cat, Slack::Question) {
        return Some(explain(context));
    }
    if is_any_phrase(&said, cat.household_phrases(|pack| pack.household.undo), cat, Slack::Filler) {
        return Some(undo(context));
    }
    if let Some(alias) = taught_name(context, &said) {
        return Some(teach(context, &alias));
    }
    if looks_like_weekday(context, &said, tokens) {
        return Some(weekday(context));
    }
    if looks_like_date(context, &said, tokens) {
        return Some(date(context));
    }
    if looks_like_today(context, &said, tokens) {
        return Some(today(context));
    }
    if looks_like_clock(context, &said, tokens) {
        return Some(clock(context));
    }
    let pack_weather = cat.household_phrases(|pack| pack.household.weather).any(|phrase| has_words(&said, phrase));
    if let Some(class) = super::weather_lex::classify(&blob, pack_weather) {
        if calendar_overrides_weather(context, tokens) {
            return None;
        }
        if !class.ask.skips_climate() && climate_overrides_weather(context, tokens) {
            return None;
        }
        // No forecast entity: a bare "what is the temperature" can still read a thermostat.
        if super::weather_lex::bare_temperature_ask(&blob) && !has_weather(context) {
            return None;
        }
        // A place leftover is not the home forecast (P-31).
        if !super::weather_lex::leftover_is_home_forecast(&blob) {
            return None;
        }
        return Some(weather(context, class));
    }
    None
}

fn has_weather(context: &ParseContext<'_>) -> bool {
    context.home.entities.iter().any(|entity| entity.domain == "weather" && assist_visible(entity, context.home))
}

fn calendar_overrides_weather(context: &ParseContext<'_>, tokens: &[String]) -> bool {
    context.catalog.any(tokens, context.catalog.calendar_nouns())
}

/// Generated packs store `{query} {climate}` as a weather phrase. A room or
/// extra tokens means indoor climate, not the forecast entity.
fn climate_overrides_weather(context: &ParseContext<'_>, tokens: &[String]) -> bool {
    let blob = fold_umlaut(context.text.trim()).replace(['\'', '’', '`'], "");
    if super::weather_lex::outdoor_temp_ask(&blob) || super::weather_lex::bare_temperature_ask(&blob) {
        return false;
    }
    let cat = context.catalog.as_ref();
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

/// "call it reading lamp" names the last device. "call it a day" names nothing: a name
/// does not open with an indefinite article.
fn taught_name(context: &ParseContext<'_>, said: &[String]) -> Option<String> {
    let cat = context.catalog.as_ref();
    let rest = cat.household_phrases(|pack| pack.household.teach).find_map(|prefix| after_prefix(said, prefix, cat))?;
    let named = rest.first().is_some_and(|word| !cat.article_one().contains(word.as_str()));
    named.then(|| rest.join(" "))
}

fn match_routine(context: &ParseContext<'_>, said: &[String]) -> Option<Draft> {
    for row in context.custom {
        let Some(script) = row.slots.get("entity_id") else {
            continue;
        };
        if row.intent != "HassTurnOn" || !script.starts_with("script.") {
            continue;
        }
        if !is_phrase(said, &row.phrase, &context.catalog, Slack::Filler) {
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
    let trigger = heard.trigger_word.as_deref().filter(|word| !word.is_empty()).map(|word| format!(" ({word})")).unwrap_or_default();
    format!("{heard_line}{why}{trigger}{where_}")
}

fn undo(context: &ParseContext<'_>) -> Draft {
    let house = templates(context);
    let inverted: Vec<Intent> = context.session.last_execute.iter().filter_map(invert_intent).collect();
    if inverted.is_empty() {
        return reject(RejectReason::NoAction, house.nothing_undo.into());
    }
    // Undo is a new action: it is validated, meets the user's rules and asks like any other.
    // Reversing something that needed a yes needs a yes again, whichever way it moves.
    let mut draft = execute(context, inverted, "household_undo", 1.0, 1.0, true, false);
    draft.inherits_risk = context.session.last_execute.iter().any(|intent| risky_step(intent, context.home));
    draft
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
    clock_chat(spoken_clock(context))
}

fn weekday(context: &ParseContext<'_>) -> Draft {
    clock_chat(spoken_weekday(context))
}

fn date(context: &ParseContext<'_>) -> Draft {
    clock_chat(spoken_date(context))
}

fn today(context: &ParseContext<'_>) -> Draft {
    clock_chat(spoken_today(context))
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

fn looks_like_clock(context: &ParseContext<'_>, said: &[String], tokens: &[String]) -> bool {
    if context.catalog.pack_any(tokens, |pack| pack.household.clock_skip) {
        return false;
    }
    is_any_phrase(said, context.catalog.household_phrases(|pack| pack.household.clock), &context.catalog, Slack::Question)
}

fn looks_like_weekday(context: &ParseContext<'_>, said: &[String], tokens: &[String]) -> bool {
    if context.catalog.pack_any(tokens, |pack| pack.household.clock_skip) {
        return false;
    }
    is_any_phrase(said, context.catalog.household_phrases(|pack| pack.household.weekday), &context.catalog, Slack::Question)
}

fn looks_like_date(context: &ParseContext<'_>, said: &[String], tokens: &[String]) -> bool {
    if context.catalog.pack_any(tokens, |pack| pack.household.clock_skip) {
        return false;
    }
    is_any_phrase(said, context.catalog.household_phrases(|pack| pack.household.date), &context.catalog, Slack::Question)
}

fn looks_like_today(context: &ParseContext<'_>, said: &[String], tokens: &[String]) -> bool {
    if context.catalog.pack_any(tokens, |pack| pack.household.clock_skip) {
        return false;
    }
    is_any_phrase(said, context.catalog.household_phrases(|pack| pack.household.today), &context.catalog, Slack::Question)
}

fn spoken_clock(context: &ParseContext<'_>) -> String {
    let house = templates(context);
    match super::clock_time::stamp_for(context) {
        Some(stamp) => house.clock_ok.replace("{time}", &stamp.hhmm),
        None => house.clock_missing.to_string(),
    }
}

fn spoken_weekday(context: &ParseContext<'_>) -> String {
    let house = templates(context);
    match super::clock_time::stamp_for(context) {
        Some(stamp) => house.weekday_ok.replace("{weekday}", &stamp.weekday),
        None => house.clock_missing.to_string(),
    }
}

fn spoken_date(context: &ParseContext<'_>) -> String {
    let house = templates(context);
    match super::clock_time::stamp_for(context) {
        Some(stamp) => house.date_ok.replace("{weekday}", &stamp.weekday).replace("{date}", &stamp.date),
        None => house.clock_missing.to_string(),
    }
}

fn spoken_today(context: &ParseContext<'_>) -> String {
    let house = templates(context);
    match super::clock_time::stamp_for(context) {
        Some(stamp) => house.today_ok.replace("{weekday}", &stamp.weekday).replace("{date}", &stamp.date),
        None => house.clock_missing.to_string(),
    }
}

const FALLBACK: Household = Household {
    teach: &[],
    explain: &[],
    undo: &[],
    clock: &[],
    date: &[],
    weekday: &[],
    today: &[],
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
    date_ok: "{weekday}, {date}",
    weekday_ok: "{weekday}",
    today_ok: "{weekday}, {date}",
    no_weather: "",
};

fn templates(context: &ParseContext<'_>) -> &'static Household {
    context.catalog.household().unwrap_or(&FALLBACK)
}

fn valid_alias(alias: &str) -> bool {
    let chars = alias.chars().count();
    (2..=40).contains(&chars) && !alias.chars().any(|ch| ch.is_control())
}

#[cfg(test)]
#[path = "household_tests.rs"]
mod tests;
