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
    home.areas.push(crate::types::AreaRec { area_id: "salon".into(), name: "salon".into(), floor_id: None, aliases: vec!["salon".into()] });
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
    assert_eq!(draft.plan.as_ref().unwrap().intents()[0].slot("weather_ask"), Some("now"));
}

#[test]
fn weather_tomorrow_is_still_forecast() {
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
    let context = ctx("Wie wird das Wetter morgen", &home, &session, &custom);
    let draft = route(&context, &["wie".into(), "wird".into(), "das".into(), "wetter".into(), "morgen".into()]).expect("weather");
    assert!(matches!(draft.decision, ParseDecision::Execute));
    assert_eq!(draft.plan.as_ref().unwrap().intents()[0].name, "HassGetState");
    assert_eq!(draft.plan.as_ref().unwrap().intents()[0].slot("entity_id"), Some("weather.home"));
    assert_eq!(draft.plan.as_ref().unwrap().intents()[0].slot("weather_ask"), Some("tomorrow"));
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
    assert_eq!(spoken.matches(':').count(), 1, "{spoken}");
    assert!(!spoken.contains("sekunde"));
}

#[test]
fn calendar_query_is_not_forecast() {
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
    let settings = Box::leak(Box::new(Settings::pinned("en")));
    let catalog = catalog_for(&["en".into()]);
    let context = ParseContext::new("What's on my calendar tomorrow?", &home, &session, &custom, settings, catalog);
    assert!(
        route(&context, &["what".into(), "on".into(), "calendar".into(), "tomorrow".into()]).is_none(),
        "calendar query must not become household weather"
    );
}

#[test]
fn german_calendar_query_is_not_forecast() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "weather.home".into(),
        name: "Zuhause".into(),
        domain: "weather".into(),
        platform: None,
        aliases: Vec::new(),
        tags: Vec::new(),
        area: None,
    });
    let session = Session::new();
    let custom = Vec::new();
    let settings = Box::leak(Box::new(Settings::pinned("de")));
    let catalog = catalog_for(&["de".into()]);
    let context = ParseContext::new("Was steht morgen im Kalender?", &home, &session, &custom, settings, catalog);
    assert!(
        route(&context, &["was".into(), "steht".into(), "morgen".into(), "im".into(), "kalender".into()]).is_none(),
        "German calendar query must not become household weather"
    );
}

#[test]
fn english_weather_keeps_apostrophe() {
    let home = default_home();
    let session = Session::new();
    let custom = Vec::new();
    let settings = Box::leak(Box::new(Settings::pinned("en")));
    let catalog = catalog_for(&["en".into()]);
    let context = ParseContext::new("What's the weather?", &home, &session, &custom, settings, catalog);
    let draft = route(&context, &["what".into(), "s".into(), "the".into(), "weather".into()]).expect("weather");
    assert!(matches!(draft.decision, ParseDecision::Chat | ParseDecision::Execute), "{:?}", draft.decision);
}

#[test]
fn outdoor_temperature_binds_weather_not_climate() {
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
    home.entities.push(EntityRec {
        entity_id: "climate.thermostat".into(),
        name: "Thermostat".into(),
        domain: "climate".into(),
        platform: None,
        area: None,
        aliases: Vec::new(),
        tags: Vec::new(),
    });
    let session = Session::new();
    let custom = Vec::new();
    let settings = Box::leak(Box::new(Settings::pinned("en")));
    let catalog = catalog_for(&["en".into()]);
    let context = ParseContext::new("What's the temperature outside?", &home, &session, &custom, settings, catalog);
    let draft = route(&context, &["what".into(), "s".into(), "the".into(), "temperature".into(), "outside".into()]).expect("outdoor temp");
    assert!(matches!(draft.decision, ParseDecision::Execute), "{:?}", draft.decision);
    let intent = &draft.plan.as_ref().unwrap().intents()[0];
    assert_eq!(intent.name, "HassGetState");
    assert_eq!(intent.slot("entity_id"), Some("weather.home"));
    assert_eq!(intent.slot("domain"), Some("weather"));
}
