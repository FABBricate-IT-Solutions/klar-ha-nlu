use klar_nlu::lexicon::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::Settings;

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
fn follow_up_aus() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings::default();
    parse("Licht im Wohnzimmer an", &home, &mut session, &[], &settings);
    let second = parse("mach sie aus", &home, &mut session, &[], &settings);
    assert_eq!(second.intents[0].name, "HassTurnOff");
}

