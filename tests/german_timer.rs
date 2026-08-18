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
