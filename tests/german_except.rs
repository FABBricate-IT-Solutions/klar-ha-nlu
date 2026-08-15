mod common;

use common::slots;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::Settings;

#[test]
fn alle_lichter_aus_ausser_kugel() {
    let found = slots("Alle lichter aus ausser der Kugel");
    assert!(found.iter().all(|(name, _)| name == "HassTurnOff"), "{found:?}");
    let ids: Vec<&str> =
        found.iter().filter_map(|(_, slots)| slots.iter().find(|(k, _)| k == "entity_id").map(|(_, v)| v.as_str())).collect();
    assert!(!ids.contains(&"light.alle_lichter"), "{found:?}");
    assert!(!ids.contains(&"light.schlafzimmer_kugel"), "{found:?}");
    let areas: Vec<&str> =
        found.iter().filter_map(|(_, slots)| slots.iter().find(|(k, _)| *k == "area").map(|(_, v)| v.as_str())).collect();
    assert!(ids.contains(&"light.wohnzimmer") || areas.contains(&"wohnzimmer"), "{found:?}");
    assert!(ids.contains(&"light.schlafzimmer_decke") || ids.contains(&"light.schlafzimmer_licht"), "{found:?}");
}

#[test]
fn alle_lichter_ausser_schlafzimmer() {
    let found = slots("Alle Lichter außer Schlafzimmer ausschalten");
    assert!(found.iter().all(|(name, _)| name == "HassTurnOff"), "{found:?}");
    let ids: Vec<&str> =
        found.iter().filter_map(|(_, slots)| slots.iter().find(|(k, _)| k == "entity_id").map(|(_, v)| v.as_str())).collect();
    let areas: Vec<&str> = found.iter().filter_map(|(_, slots)| slots.iter().find(|(k, _)| k == "area").map(|(_, v)| v.as_str())).collect();
    assert!(!ids.contains(&"light.alle_lichter"), "{found:?}");
    assert!(!areas.contains(&"schlafzimmer"), "{found:?}");
    assert!(!ids.iter().any(|id| id.contains("schlafzimmer")), "{found:?}");
    assert!(ids.contains(&"light.wohnzimmer") || areas.contains(&"wohnzimmer"), "{found:?}");
}

#[test]
fn alle_lichter_ausser_insel() {
    let home = klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/family_home_en/home_config.yaml")).expect("home");
    for text in ["Turn off all lights except the island", "All lights off except the island", "Turn off all lights but not the island"] {
        let mut session = Session::new();
        let result = parse(text, &home, &mut session, &[], &Settings::default());
        let ids: Vec<&str> = result.intents.iter().filter_map(|i| i.slot("entity_id")).collect();
        let areas: Vec<&str> = result.intents.iter().filter_map(|i| i.slot("area")).collect();
        assert!(!result.clarify, "{text}: {}", result.speech);
        assert!(!ids.contains(&"light.kitchen_island"), "{text}: island off {ids:?} {areas:?}");
        assert!(!areas.contains(&"kitchen"), "{text}: whole kitchen {ids:?} {areas:?}");
        assert!(ids.contains(&"light.kitchen_ceiling"), "{text}: ceiling missing {ids:?} {areas:?}");
        assert!(areas.contains(&"living"), "{text}: living missing {ids:?} {areas:?}");
    }
}

#[test]
fn alle_lichter_ausser_insel_de() {
    let home = klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/familienhaus_de/home_config.yaml")).expect("home");
    let mut session = Session::new();
    let result = parse("Alle Lichter außer der Insel aus", &home, &mut session, &[], &Settings::default());
    let ids: Vec<&str> = result.intents.iter().filter_map(|i| i.slot("entity_id")).collect();
    let areas: Vec<&str> = result.intents.iter().filter_map(|i| i.slot("area")).collect();
    assert!(!ids.contains(&"light.kitchen_island"), "{ids:?} {areas:?}");
    assert!(!areas.contains(&"kitchen"), "{ids:?} {areas:?}");
    assert!(ids.contains(&"light.kitchen_ceiling"), "{ids:?} {areas:?}");
}
