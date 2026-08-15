use klar_nlu::lexicon::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, Settings};

fn run(text: &str) -> (Vec<String>, bool) {
    let home = default_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &Settings::default());
    (
        result.intents.iter().map(|i| i.name.clone()).collect(),
        result.clarify,
    )
}

fn slots(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let home = default_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &Settings::default());
    result
        .intents
        .into_iter()
        .map(|i| {
            (
                i.name,
                i.slots.into_iter().map(|s| (s.name, s.value)).collect(),
            )
        })
        .collect()
}

#[test]
fn wohnzimmer_licht_an() {
    let (names, _) = run("Mach das Licht im Wohnzimmer an");
    assert_eq!(names, vec!["HassTurnOn"]);
}

#[test]
fn zwei_raeume_und_heizung() {
    let found = slots(
        "Mach das Licht im Wohnzimmer und in der Küche an und stell die Heizung auf 23",
    );
    let names: Vec<_> = found.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names.iter().filter(|n| **n == "HassTurnOn").count(),
        2,
        "{found:?}"
    );
    assert!(names.contains(&"HassClimateSetTemperature"));
    let areas: Vec<_> = found
        .iter()
        .filter(|(n, _)| n == "HassTurnOn")
        .flat_map(|(_, slots)| slots.iter().filter(|(k, _)| k == "area").map(|(_, v)| v.as_str()))
        .collect();
    assert!(areas.contains(&"wohnzimmer") || found.iter().any(|(_, s)| {
        s.iter().any(|(k, v)| k == "entity_id" && v.contains("wohnzimmer"))
    }));
}

#[test]
fn temperatur_wohnung() {
    let found = slots("Wie warm ist es in der Wohnung");
    assert_eq!(found[0].0, "HassGetState");
    assert!(found[0]
        .1
        .iter()
        .any(|(k, v)| k == "area" && v == "wohnung"));
}

#[test]
fn alle_lichter_aus() {
    let found = slots("Alle Lichter aus");
    assert_eq!(found[0].0, "HassTurnOff");
}

#[test]
fn heizung_zahlwort() {
    let found = slots("Heizung Wohnzimmer auf dreiundzwanzig Grad");
    assert_eq!(found[0].0, "HassClimateSetTemperature");
    assert!(found[0]
        .1
        .iter()
        .any(|(k, v)| k == "temperature" && v == "23"));
}

#[test]
fn staubsauger() {
    let (names, _) = run("R2D2 soll bitte saugen");
    assert_eq!(names, vec!["HassVacuumStart"]);
}

#[test]
fn licht_nicht_mehrdeutig() {
    let (names, clarify) = run("Mach das Licht im Wohnzimmer an");
    assert!(!clarify);
    assert_eq!(names, vec!["HassTurnOn"]);
}

#[test]
fn licht_ohne_raum_fragt_nach() {
    let (names, clarify) = run("Mach das Licht an");
    assert!(clarify, "{names:?}");
    assert!(names.is_empty(), "{names:?}");
}

#[test]
fn sauger_status_dock_kein_return() {
    let found = slots("Ist R2D2 am Dock");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "HassGetState");
}

#[test]
fn timer_nutzt_minutes() {
    let found = slots("Stell einen Timer auf fünf Minuten");
    assert_eq!(found[0].0, "HassStartTimer", "{found:?}");
    assert!(
        found[0].1.iter().any(|(k, v)| k == "minutes" && v == "5"),
        "{found:?}"
    );
    assert!(!found[0].1.iter().any(|(k, _)| k == "duration"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "entity_id"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "domain"), "{found:?}");
}

#[test]
fn timer_eine_minute() {
    let found = slots("Stell einen Timer auf eine Minute");
    assert_eq!(found[0].0, "HassStartTimer", "{found:?}");
    assert!(
        found[0].1.iter().any(|(k, v)| k == "minutes" && v == "1"),
        "{found:?}"
    );
}

#[test]
fn timer_haengt_fremden_helper_nicht_an() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "timer.5min_warten".into(),
        name: "5min warten".into(),
        domain: "timer".into(),
        area: None,
        aliases: vec!["5min".into()],
        tags: Vec::new(),
    });
    let mut session = Session::new();
    let result = parse(
        "Stell einen Timer auf 5 Minuten",
        &home,
        &mut session,
        &[],
        &Settings::default(),
    );
    let slots: Vec<_> = result.intents[0]
        .slots
        .iter()
        .map(|s| (s.name.as_str(), s.value.as_str()))
        .collect();
    assert_eq!(result.intents[0].name, "HassStartTimer", "{slots:?}");
    assert!(slots.iter().any(|(k, v)| *k == "minutes" && *v == "5"), "{slots:?}");
    assert!(!slots.iter().any(|(k, v)| *k == "entity_id" && *v == "timer.5min_warten"), "{slots:?}");
}

#[test]
fn einkaufsliste_nutzt_todo_ohne_shopping_list_name() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "todo.einkaufsliste".into(),
        name: "Einkaufsliste".into(),
        domain: "todo".into(),
        area: None,
        aliases: vec!["einkaufsliste".into(), "einkauf".into()],
        tags: Vec::new(),
    });
    let mut session = Session::new();
    let result = parse(
        "Setze Milch auf die Einkaufsliste",
        &home,
        &mut session,
        &[],
        &Settings::default(),
    );
    let slots: Vec<_> = result.intents[0]
        .slots
        .iter()
        .map(|s| (s.name.as_str(), s.value.as_str()))
        .collect();
    assert_eq!(result.intents[0].name, "HassListAddItem", "{slots:?}");
    assert!(slots.iter().any(|(k, v)| *k == "entity_id" && *v == "todo.einkaufsliste"), "{slots:?}");
    assert!(!slots.iter().any(|(k, v)| *k == "name" && *v == "shopping_list"), "{slots:?}");
}

#[test]
fn einkaufsliste_heisst_list_add() {
    let found = slots("Setze Milch auf die Einkaufsliste");
    assert_eq!(found[0].0, "HassListAddItem", "{found:?}");
    assert!(
        found[0].1.iter().any(|(k, v)| k == "item" && v.contains("milch")),
        "{found:?}"
    );
    assert!(
        found[0].1.iter().any(|(k, v)| k == "name" && v == "shopping_list"),
        "{found:?}"
    );
}

#[test]
fn smalltalk_hat_keinen_home_intent() {
    for text in [
        "Was ist die Hauptstadt von Frankreich",
        "Erzähl einen Katzenwitz",
        "asdfghjkl qwerty",
    ] {
        let (names, clarify) = run(text);
        assert!(!clarify, "{text}: {names:?}");
        assert!(names.is_empty(), "{text}: {names:?}");
    }
}

#[test]
fn follow_up_aus() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings::default();
    parse("Licht im Wohnzimmer an", &home, &mut session, &[], &settings);
    let second = parse("mach sie aus", &home, &mut session, &[], &settings);
    assert_eq!(second.intents[0].name, "HassTurnOff");
}

