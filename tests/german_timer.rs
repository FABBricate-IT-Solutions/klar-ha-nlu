use klar_nlu::home::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, Settings};

fn slots(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let home = default_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &Settings::pinned("de"));
    result.intents.into_iter().map(|i| (i.name, i.slots.into_iter().map(|s| (s.name, s.value)).collect())).collect()
}

#[test]
fn timer_nutzt_minutes() {
    let found = slots("Stell einen Timer auf fünf Minuten");
    assert_eq!(found[0].0, "HassStartTimer", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "minutes" && v == "5"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "duration"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "entity_id"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "domain"), "{found:?}");
}

#[test]
fn timer_abbrechen() {
    let found = slots("Timer abbrechen");
    assert_eq!(found[0].0, "HassCancelTimer", "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "minutes"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "hours"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "seconds"), "{found:?}");
}

#[test]
fn timer_aus_bricht_ab() {
    let found = slots("Timer aus");
    assert_eq!(found[0].0, "HassCancelTimer", "{found:?}");
}

#[test]
fn cancel_the_timer() {
    let home = default_home();
    let mut session = Session::new();
    let result = parse("Cancel the timer", &home, &mut session, &[], &Settings::pinned("en"));
    let found: Vec<_> = result.intents.into_iter().map(|i| i.name).collect();
    assert_eq!(found.first().map(String::as_str), Some("HassCancelTimer"), "{found:?}");
}

#[test]
fn timer_pausieren() {
    let found = slots("Timer pausieren");
    assert_eq!(found[0].0, "HassPauseTimer", "{found:?}");
}

#[test]
fn timer_eine_minute() {
    let found = slots("Stell einen Timer auf eine Minute");
    assert_eq!(found[0].0, "HassStartTimer", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "minutes" && v == "1"), "{found:?}");
}

#[test]
fn timer_haengt_fremden_helper_nicht_an() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "timer.5min_warten".into(),
        name: "5min warten".into(),
        domain: "timer".into(),
        platform: None,
        area: None,
        aliases: vec!["5min".into()],
        tags: Vec::new(),
    });
    let mut session = Session::new();
    let result = parse("Stell einen Timer auf 5 Minuten", &home, &mut session, &[], &Settings::pinned("de"));
    let slots: Vec<_> = result.intents[0].slots.iter().map(|s| (s.name.as_str(), s.value.as_str())).collect();
    assert_eq!(result.intents[0].name, "HassStartTimer", "{slots:?}");
    assert!(slots.iter().any(|(k, v)| *k == "minutes" && *v == "5"), "{slots:?}");
    assert!(!slots.iter().any(|(k, v)| *k == "entity_id" && *v == "timer.5min_warten"), "{slots:?}");
}

#[test]
fn decrease_timer_is_not_increase() {
    for (lang, sentence) in [
        ("en", "decrease the timer by 5 minutes"),
        ("en", "reduce the timer by 5 minutes"),
        ("en", "subtract 5 minutes from the timer"),
        ("en", "make the timer 20 seconds shorter"),
        ("de", "Timer verringern um 5 Minuten"),
        ("de", "Timer um 20 Sekunden kürzer machen"),
    ] {
        let result = parse(sentence, &default_home(), &mut Session::new(), &[], &Settings::pinned(lang));
        assert!(!result.clarify, "{sentence}: {result:?}");
        assert_eq!(result.intents.len(), 1, "{sentence}: {result:?}");
        assert_eq!(result.intents[0].name, "HassDecreaseTimer", "{sentence}: {result:?}");
        assert_ne!(result.intents[0].name, "HassIncreaseTimer", "{sentence}: {result:?}");
        let amount = if sentence.contains("20") { "20" } else { "5" };
        let unit = if sentence.contains("second") || sentence.contains("Sekunden") { "seconds" } else { "minutes" };
        assert_eq!(result.intents[0].slot(unit), Some(amount), "{sentence}: {result:?}");
    }
}

#[test]
fn kuerzer_machen_asks_then_takes_seconds() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings::pinned("de");
    let ask = parse("Timer kürzer machen", &home, &mut session, &[], &settings);
    assert!(ask.clarify, "{ask:?}");
    assert!(ask.intents.is_empty(), "{ask:?}");
    let found = parse("20 Sekunden", &home, &mut session, &[], &settings);
    assert!(!found.clarify, "{found:?}");
    assert_eq!(found.intents.len(), 1, "{found:?}");
    assert_eq!(found.intents[0].name, "HassDecreaseTimer", "{found:?}");
    assert_eq!(found.intents[0].slot("seconds"), Some("20"), "{found:?}");
}

#[test]
fn start_timer_how_long_takes_minutes() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings::pinned("de");
    let ask = parse("Stell einen Timer", &home, &mut session, &[], &settings);
    assert!(ask.clarify, "{ask:?}");
    let found = parse("5 Minuten", &home, &mut session, &[], &settings);
    assert_eq!(found.intents[0].name, "HassStartTimer", "{found:?}");
    assert_eq!(found.intents[0].slot("minutes"), Some("5"), "{found:?}");
}

#[test]
fn increase_timer_stays_increase() {
    let result = parse("increase the timer by 5 minutes", &default_home(), &mut Session::new(), &[], &Settings::pinned("en"));
    assert_eq!(result.intents[0].name, "HassIncreaseTimer", "{result:?}");
}

#[test]
fn cancel_all_timers_cancels_each_named_timer() {
    let mut home = default_home();
    home.entities.extend([
        EntityRec {
            entity_id: "timer.oven".into(),
            name: "Oven".into(),
            domain: "timer".into(),
            platform: None,
            area: None,
            aliases: vec!["oven".into()],
            tags: Vec::new(),
        },
        EntityRec {
            entity_id: "timer.laundry".into(),
            name: "Laundry".into(),
            domain: "timer".into(),
            platform: None,
            area: None,
            aliases: vec!["laundry".into()],
            tags: Vec::new(),
        },
    ]);
    let result = parse("Cancel all timers", &home, &mut Session::new(), &[], &Settings::pinned("en"));
    assert!(!result.clarify, "{result:?}");
    let ids: Vec<_> = result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect();
    assert!(result.intents.iter().all(|intent| intent.name == "HassCancelTimer"), "{result:?}");
    assert!(ids.contains(&"timer.oven") && ids.contains(&"timer.laundry"), "{result:?}");
}

#[test]
fn start_timer_without_duration_clarifies() {
    for (lang, sentence) in [("en", "Start the timer"), ("de", "Stell einen Timer")] {
        let result = parse(sentence, &default_home(), &mut Session::new(), &[], &Settings::pinned(lang));
        assert!(result.clarify, "{sentence}: {result:?}");
        assert!(result.intents.is_empty(), "{sentence}: {result:?}");
    }
}

#[test]
fn resume_named_timer_does_not_ask_duration() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "timer.oven".into(),
        name: "Oven".into(),
        domain: "timer".into(),
        platform: None,
        area: None,
        aliases: vec!["oven".into(), "ofen".into()],
        tags: Vec::new(),
    });
    for (lang, sentence) in [("en", "Resume the oven timer"), ("de", "Setze den Ofen-Timer fort")] {
        let result = parse(sentence, &home, &mut Session::new(), &[], &Settings::pinned(lang));
        assert!(!result.clarify, "{sentence}: {result:?}");
        assert_eq!(result.intents.len(), 1, "{sentence}: {result:?}");
        assert_eq!(result.intents[0].name, "HassStartTimer", "{sentence}: {result:?}");
        assert_eq!(result.intents[0].slot("entity_id"), Some("timer.oven"), "{sentence}: {result:?}");
        assert!(result.intents[0].slot("hours").is_none(), "{sentence}: {result:?}");
        assert!(result.intents[0].slot("minutes").is_none(), "{sentence}: {result:?}");
        assert!(result.intents[0].slot("seconds").is_none(), "{sentence}: {result:?}");
    }
    let restart = parse("Please restart the timer", &home, &mut Session::new(), &[], &Settings::pinned("en"));
    assert!(!restart.clarify, "{restart:?}");
    assert_eq!(restart.intents[0].name, "HassStartTimer", "{restart:?}");
    assert!(restart.intents[0].slot("minutes").is_none(), "{restart:?}");
}
