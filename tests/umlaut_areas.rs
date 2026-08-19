use klar_nlu::home::default_home;
use klar_nlu::parse::parse;
use klar_nlu::parse::respond::speak;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, Personality, Settings};

fn parse_de(text: &str, home: &HomeGraph) -> (String, Vec<(String, String)>) {
    let mut session = Session::new();
    let result = parse(text, home, &mut session, &[], &Settings::pinned("de"));
    let intent = result.intents.first();
    (
        intent.map(|item| item.name.clone()).unwrap_or_default(),
        intent.map(|item| item.slots.iter().map(|slot| (slot.name.clone(), slot.value.clone())).collect()).unwrap_or_default(),
    )
}

fn slot<'a>(found: &'a [(String, String)], name: &str) -> Option<&'a str> {
    found.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
}

fn umlaut_home() -> HomeGraph {
    let mut home = default_home();
    if let Some(kitchen) = home.entities.iter_mut().find(|entity| entity.entity_id == "light.kuche_kuche") {
        kitchen.name = "Licht".into();
        kitchen.aliases = vec!["Küche Licht".into(), "Küche".into(), "Küchenlicht".into(), "Kuche".into()];
    }
    if let Some(office) = home.areas.iter_mut().find(|area| area.area_id == "arbeitszimmer") {
        office.aliases.retain(|alias| alias != "buero");
    }
    home.areas.push(AreaRec { area_id: "buro".into(), name: "Büro".into(), aliases: vec!["office".into()], floor_id: None });
    home.entities.push(EntityRec {
        entity_id: "light.buro_decke".into(),
        name: "Licht".into(),
        domain: "light".into(),
        platform: None,
        area: Some("buro".into()),
        aliases: vec!["Bürolampe".into()],
        tags: Vec::new(),
    });
    home.areas.push(AreaRec { area_id: "gastezimmer".into(), name: "Gästezimmer".into(), aliases: Vec::new(), floor_id: None });
    home.entities.push(EntityRec {
        entity_id: "light.gastezimmer".into(),
        name: "Gästezimmer Licht".into(),
        domain: "light".into(),
        platform: None,
        area: Some("gastezimmer".into()),
        aliases: vec!["gaestezimmer".into()],
        tags: Vec::new(),
    });
    home
}

fn hits_room(found: &[(String, String)], area: &str, entity: &str) -> bool {
    slot(found, "area") == Some(area) || slot(found, "entity_id") == Some(entity)
}

#[test]
fn kitchen_on_off_and_status_accept_umlaut_and_ascii() {
    let home = umlaut_home();
    for text in ["Licht in der Küche an", "Licht in der Kuche an", "Küche an", "Kuche an"] {
        let (name, found) = parse_de(text, &home);
        assert_eq!(name, "HassTurnOn", "{text}: {found:?}");
        assert!(hits_room(&found, "kuche", "light.kuche_kuche"), "{text}: {found:?}");
    }
    for text in ["Licht in der Küche aus", "Küche aus", "Kuche aus"] {
        let (name, found) = parse_de(text, &home);
        assert_eq!(name, "HassTurnOff", "{text}: {found:?}");
        assert!(hits_room(&found, "kuche", "light.kuche_kuche"), "{text}: {found:?}");
    }
    for text in ["Wie ist der Status der Küche", "Wie ist der Status der Kuche", "Wie ist der Status von der Küche"] {
        let (name, found) = parse_de(text, &home);
        assert_eq!(name, "HassGetState", "{text}: {found:?}");
        assert_eq!(slot(&found, "area"), Some("kuche"), "{text}: {found:?}");
        assert!(slot(&found, "entity_id").is_none(), "{text}: {found:?}");
    }
}

#[test]
fn other_umlaut_rooms_match_ha_slugs() {
    let home = umlaut_home();
    let (name, found) = parse_de("Licht im Büro an", &home);
    assert_eq!(name, "HassTurnOn", "{found:?}");
    assert!(hits_room(&found, "buro", "light.buro_decke"), "{found:?}");
    let (name, found) = parse_de("Licht im Buro an", &home);
    assert_eq!(name, "HassTurnOn", "{found:?}");
    assert!(hits_room(&found, "buro", "light.buro_decke"), "{found:?}");
    let (name, found) = parse_de("Licht im Gästezimmer an", &home);
    assert_eq!(name, "HassTurnOn", "{found:?}");
    assert!(hits_room(&found, "gastezimmer", "light.gastezimmer"), "{found:?}");
}

#[test]
fn kitchen_status_speech_keeps_umlaut() {
    let home = umlaut_home();
    let mut session = Session::new();
    let result = parse("Wie ist der Status der Küche", &home, &mut session, &[], &Settings::pinned("de"));
    let speech = speak(&result.intents, Personality::Default, false, Some(&home));
    assert!(speech.contains("Küche") || result.speech.contains("Küche"), "{speech} / {}", result.speech);
    assert!(!speech.contains("Kuche"), "{speech}");
    assert!(!result.speech.contains("Kuche"), "{}", result.speech);
}
