use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, FloorRec, HomeGraph, ParseResult, Settings};

fn area(id: &str, name: &str, floor: &str, aliases: &[&str]) -> AreaRec {
    AreaRec {
        area_id: id.into(),
        name: name.into(),
        aliases: aliases.iter().map(|alias| (*alias).to_string()).collect(),
        floor_id: Some(floor.into()),
    }
}

fn light(id: &str, name: &str, area_id: &str) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: "light".into(),
        platform: None,
        area: Some(area_id.into()),
        aliases: vec![name.to_string()],
        tags: Vec::new(),
    }
}

fn home() -> HomeGraph {
    HomeGraph {
        floors: vec![
            FloorRec {
                floor_id: "dg".into(),
                name: "Dachgeschoss".into(),
                aliases: vec!["dachgeschoss".into(), "dg".into()],
                level: Some(1),
            },
            FloorRec {
                floor_id: "eg".into(),
                name: "Erdgeschoss".into(),
                aliases: vec!["erdgeschoss".into(), "eg".into()],
                level: Some(0),
            },
            FloorRec { floor_id: "kg".into(), name: "Keller".into(), aliases: vec!["keller".into()], level: Some(-1) },
        ],
        areas: vec![
            area("living_room_dg", "Wohnzimmer DG", "dg", &["wohnzimmer", "wohnzimmer dg"]),
            area("living_room_eg", "Wohnzimmer EG", "eg", &["wohnzimmer", "wohnzimmer eg"]),
            area("kitchen_eg", "Küche", "eg", &["kueche", "kuche"]),
            area("office_dg", "Büro DG", "dg", &["buero", "büro", "office dg"]),
            area("office_kg", "Büro Keller", "kg", &["buero keller"]),
        ],
        entities: vec![
            light("light.living_dg", "Wohnzimmer DG", "living_room_dg"),
            light("light.living_eg", "Wohnzimmer EG", "living_room_eg"),
            light("light.kitchen", "Küche", "kitchen_eg"),
        ],
        ..Default::default()
    }
}

fn talk(text: &str, preferred: Option<&str>, session: &mut Session) -> ParseResult {
    session.preferred_area = preferred.map(str::to_string);
    parse(text, &home(), session, &[], &Settings::pinned("de"))
}

fn areas(result: &ParseResult) -> Vec<&str> {
    result.intents.iter().filter_map(|intent| intent.slot("area").or_else(|| intent.slot("entity_id"))).collect()
}

#[test]
fn same_room_on_two_floors_asks_when_the_floor_is_unknown() {
    let mut session = Session::new();
    let result = talk("Licht im Wohnzimmer an", None, &mut session);
    assert!(result.clarify, "{result:?}");
    assert!(result.intents.is_empty(), "{result:?}");
    assert!(result.speech.contains("Wohnzimmer DG"), "{}", result.speech);
    assert!(result.speech.contains("Wohnzimmer EG"), "{}", result.speech);

    let picked = talk("EG", None, &mut session);
    assert!(!picked.clarify, "{picked:?}");
    assert_eq!(areas(&picked), ["living_room_eg"]);
}

#[test]
fn satellite_floor_picks_that_rooms_floor() {
    let mut session = Session::new();
    let result = talk("Licht im Wohnzimmer an", Some("office_dg"), &mut session);
    assert!(!result.clarify, "{result:?}");
    assert_eq!(areas(&result), ["living_room_dg"]);
}

#[test]
fn named_floor_beats_the_satellite_floor() {
    let mut session = Session::new();
    let result = talk("Licht im Wohnzimmer im Dachgeschoss an", Some("office_kg"), &mut session);
    assert!(!result.clarify, "{result:?}");
    assert_eq!(areas(&result), ["living_room_dg"]);
}

#[test]
fn satellite_floor_without_that_room_asks() {
    let mut session = Session::new();
    let result = talk("Licht im Wohnzimmer an", Some("office_kg"), &mut session);
    assert!(result.clarify, "{result:?}");
    assert!(result.intents.is_empty(), "{result:?}");
}

#[test]
fn different_rooms_still_run_together() {
    let mut session = Session::new();
    let result = talk("Licht in der Küche und im Wohnzimmer EG an", None, &mut session);
    assert!(!result.clarify, "{result:?}");
    let mut hit = areas(&result);
    hit.sort_unstable();
    assert!(hit.iter().any(|id| *id == "kitchen_eg" || *id == "light.kitchen"), "{result:?}");
    assert!(hit.iter().any(|id| *id == "living_room_eg" || *id == "light.living_eg"), "{result:?}");
    assert!(!hit.iter().any(|id| id.contains("living_room_dg") || *id == "light.living_dg"), "{result:?}");
}
