//! Household specials: teach a name, explain, undo, clock, weather, routines.

use crate::lang::Household;
use crate::parse::normalize::fold_umlaut;
use crate::session::LastHeard;
use crate::types::{Intent, RejectReason};

use super::context::ParseContext;
use super::draft::{chat, execute, reject, Draft};

pub(super) fn route(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
    if tokens.is_empty() {
        return None;
    }
    let blob = fold_umlaut(context.text.trim());
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
    if context.catalog.household_hit(&blob, |pack| pack.household.weather) && !climate_overrides_weather(context, tokens) {
        return Some(weather(context));
    }
    None
}

/// Generated packs store `{query} {climate}` as a weather phrase. A room or
/// extra tokens means indoor climate, not the forecast entity.
fn climate_overrides_weather(context: &ParseContext<'_>, tokens: &[String]) -> bool {
    let cat = context.catalog;
    if !cat.any(tokens, cat.climate_nouns()) {
        return false;
    }
    let has_area = context.home.areas.iter().any(|area| {
        tokens.iter().any(|token| {
            fold_umlaut(&area.area_id) == *token
                || fold_umlaut(&area.name) == *token
                || area.aliases.iter().any(|alias| fold_umlaut(alias) == *token)
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

fn weather(context: &ParseContext<'_>) -> Draft {
    let house = templates(context);
    let entity = context.home.entities.iter().find(|entity| entity.domain == "weather");
    match entity {
        Some(entity) => execute(
            context,
            vec![Intent::new("HassGetState").with("entity_id", &entity.entity_id).with("domain", "weather")],
            "household_weather",
            1.0,
            1.0,
            false,
            false,
        ),
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
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::lang::catalog_for;
    use crate::nlu::context::ParseContext;
    use crate::session::{LastHeard, Session};
    use crate::types::{CustomSentence, EntityRec, ParseDecision, Settings};
    use std::collections::HashMap;

    fn ctx<'a>(text: &'a str, home: &'a crate::types::HomeGraph, session: &'a Session, custom: &'a [CustomSentence]) -> ParseContext<'a> {
        let settings = Box::leak(Box::new(Settings::pinned("de")));
        let catalog = catalog_for(&["de".into()]);
        ParseContext::new(text, home, session, custom, settings, catalog)
    }

    #[test]
    fn inverts_on_off_and_skips_queries() {
        assert_eq!(invert_intent(&Intent::new("HassTurnOn").with("entity_id", "light.a")).unwrap().name, "HassTurnOff");
        assert_eq!(invert_intent(&Intent::new("HassTurnOff").with("entity_id", "light.a")).unwrap().name, "HassTurnOn");
        assert_eq!(invert_intent(&Intent::new("HassOpenCover").with("entity_id", "cover.a")).unwrap().name, "HassCloseCover");
        assert_eq!(invert_intent(&Intent::new("HassLock").with("entity_id", "lock.a")).unwrap().name, "HassUnlock");
        assert!(invert_intent(&Intent::new("HassGetState").with("entity_id", "lock.a")).is_none());
        assert!(invert_intent(&Intent::new("HassLightSet").with("entity_id", "light.a")).is_none());
        assert!(invert_intent(&Intent::new("HassSetPosition").with("entity_id", "cover.a")).is_none());
    }

    #[test]
    fn teach_needs_last_device() {
        let home = default_home();
        let session = Session::new();
        let custom = Vec::new();
        let context = ctx("nenn das Leselampe", &home, &session, &custom);
        let draft = route(&context, &["nenn".into(), "das".into(), "leselampe".into()]).expect("teach");
        assert!(matches!(draft.decision, ParseDecision::Reject { .. }));
    }

    #[test]
    fn teach_writes_alias_on_last_entity() {
        let home = default_home();
        let mut session = Session::new();
        session.remember(&Intent::new("HassTurnOn").with("entity_id", "light.wohnzimmer"));
        let custom = Vec::new();
        let context = ctx("nenn das Leselampe", &home, &session, &custom);
        let draft = route(&context, &["nenn".into(), "das".into(), "leselampe".into()]).expect("teach");
        assert!(matches!(draft.decision, ParseDecision::Chat));
        assert_eq!(draft.commit.teach.as_ref().map(|(id, alias)| (id.as_str(), alias.as_str())), Some(("light.wohnzimmer", "leselampe")));
    }

    #[test]
    fn explain_uses_last_heard() {
        let home = default_home();
        let mut session = Session::new();
        session.last_heard = Some(LastHeard {
            text: "Licht an".into(),
            decision: "execute".into(),
            speech: "an".into(),
            reason: None,
            area: Some("kueche".into()),
            names: vec!["HassTurnOn".into()],
        });
        let custom = Vec::new();
        let context = ctx("was hast du gehört", &home, &session, &custom);
        let draft = route(&context, &["was".into(), "hast".into(), "du".into(), "gehoert".into()]).expect("explain");
        assert!(draft.speech.contains("Licht an"));
        assert!(draft.speech.contains("kueche"));
    }

    #[test]
    fn weather_without_entity_is_spoken() {
        let home = default_home();
        let session = Session::new();
        let custom = Vec::new();
        let context = ctx("Wie ist das Wetter", &home, &session, &custom);
        let draft = route(&context, &["wie".into(), "ist".into(), "das".into(), "wetter".into()]).expect("weather");
        assert!(matches!(draft.decision, ParseDecision::Chat));
    }

    #[test]
    fn climate_room_query_is_not_forecast() {
        let mut home = default_home();
        home.areas.push(crate::types::AreaRec {
            area_id: "salon".into(),
            name: "salon".into(),
            floor_id: None,
            aliases: vec!["salon".into()],
        });
        home.entities.push(EntityRec {
            entity_id: "weather.home".into(),
            name: "Home".into(),
            domain: "weather".into(),
            platform: None,
            area: None,
            aliases: Vec::new(),
            tags: Vec::new(),
        });
        home.entities.push(EntityRec {
            entity_id: "climate.salon".into(),
            name: "clim salon".into(),
            domain: "climate".into(),
            platform: None,
            area: Some("salon".into()),
            aliases: vec!["clim".into()],
            tags: Vec::new(),
        });
        let session = Session::new();
        let custom = Vec::new();
        let settings = Box::leak(Box::new(Settings::pinned("fr")));
        let catalog = catalog_for(&["fr".into()]);
        let context = ParseContext::new("quel clim salon", &home, &session, &custom, settings, catalog);
        assert!(route(&context, &["quel".into(), "clim".into(), "salon".into()]).is_none());
    }

    #[test]
    fn weather_binds_weather_entity() {
        let mut home = default_home();
        home.entities.push(EntityRec {
            entity_id: "weather.home".into(),
            name: "Home".into(),
            domain: "weather".into(),
            platform: None,
            area: None,
            aliases: Vec::new(),
            tags: Vec::new(),
        });
        let session = Session::new();
        let custom = Vec::new();
        let context = ctx("Wie ist das Wetter", &home, &session, &custom);
        let draft = route(&context, &["wie".into(), "ist".into(), "das".into(), "wetter".into()]).expect("weather");
        assert!(matches!(draft.decision, ParseDecision::Execute));
        assert_eq!(draft.plan.as_ref().unwrap().intents()[0].slot("entity_id"), Some("weather.home"));
    }

    #[test]
    fn routine_beats_greeting() {
        let home = default_home();
        let session = Session::new();
        let custom = vec![CustomSentence {
            phrase: "Gute Nacht".into(),
            intent: "HassTurnOn".into(),
            slots: HashMap::from([("entity_id".into(), "script.good_night".into())]),
        }];
        let context = ctx("Gute Nacht", &home, &session, &custom);
        let draft = route(&context, &["gute".into(), "nacht".into()]).expect("routine");
        assert!(matches!(draft.decision, ParseDecision::Execute));
        assert_eq!(draft.plan.as_ref().unwrap().intents()[0].slot("entity_id"), Some("script.good_night"));
    }

    #[test]
    fn clock_does_not_steal_timer() {
        let home = default_home();
        let session = Session::new();
        let custom = Vec::new();
        let context = ctx("Timer eine Minute", &home, &session, &custom);
        assert!(route(&context, &["timer".into(), "eine".into(), "minute".into()]).is_none());
    }

    #[test]
    fn clock_formats_in_process() {
        let spoken = spoken_clock(&FALLBACK);
        assert!(spoken.contains(':') || spoken.is_empty());
        assert!(!spoken.contains("date"));
    }
}
