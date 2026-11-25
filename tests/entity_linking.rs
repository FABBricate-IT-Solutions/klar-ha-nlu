//! Package 2.1: names first — entity linking over spans (P-01, P-02, P-07, P-10, P-11, P-12, P-28).
//!
//! Longest device labels win and consume their tokens; verbs, numbers and colours are read
//! from what remains. A full name beats a domain hint and "room plus noun". Floor wording
//! scopes the match. Whole-home light groups only come from an alias or label, never from
//! "alle" inside an entity_id.
mod common;
use common::acts::{intended_changes, reach, show};
use common::collision_home::collision_home;
use common::home::{area, describe, ent, home, Talk};
use klar_nlu::types::{HomeGraph, ParseOutcome};

fn touched(lang: &str, home: &HomeGraph, text: &str) -> (ParseOutcome, Vec<String>) {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(text);
    let devices = intended_changes(&out, &talk.session, home).iter().flat_map(|step| reach(step, home)).collect();
    (out, devices)
}

fn assert_only(lang: &str, home: &HomeGraph, text: &str, only: &[&str]) {
    let (out, devices) = touched(lang, home, text);
    assert_eq!(devices.len(), only.len(), "[{lang}] {text:?} expected exactly {only:?}, got {devices:?}: {}", describe(&out));
    for device in only {
        assert!(devices.iter().any(|id| id == device), "[{lang}] {text:?} must reach {device}, got {devices:?}: {}", describe(&out));
    }
}

fn assert_intent(lang: &str, home: &HomeGraph, text: &str, name: &str) {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(text);
    let intents: Vec<_> = intended_changes(&out, &talk.session, home).iter().map(show).collect();
    assert!(
        intents.iter().any(|i| i.starts_with(name)),
        "[{lang}] {text:?} expected intent starting with {name}, got {intents:?}: {}",
        describe(&out)
    );
}

fn assert_no_value_slot(lang: &str, home: &HomeGraph, text: &str, slots: &[&str]) {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(text);
    let steps = intended_changes(&out, &talk.session, home);
    for step in &steps {
        for slot in slots {
            assert!(step.slot(slot).is_none(), "[{lang}] {text:?} must not set {slot} from a name, got {}: {}", show(step), describe(&out));
        }
    }
}

/// P-01: a more specific fixture name must not also switch every other lamp.
#[test]
fn p01_desk_lamp_is_only_the_desk_lamp() {
    let en = collision_home("en");
    assert_only("en", &en, "turn off the desk lamp", &["light.desk_lamp"]);
    assert_only("en", &en, "turn on the floor lamp", &["light.floor_lamp"]);
    assert_only("en", &en, "turn on the bedside lamp", &["light.bedside_lamp"]);

    let de = collision_home("de");
    assert_only("de", &de, "schalte die Schreibtischlampe aus", &["light.schreibtischlampe"]);
    assert_only("de", &de, "mach die Stehlampe an", &["light.stehlampe"]);
}

/// P-02: a digit that is part of the device name is not a brightness value.
#[test]
fn p02_lamp_number_is_the_name_not_brightness() {
    let en = collision_home("en");
    assert_only("en", &en, "turn on lamp 1", &["light.lamp_1"]);
    assert_intent("en", &en, "turn on lamp 1", "HassTurnOn");
    assert_no_value_slot("en", &en, "turn on lamp 1", &["brightness", "brightness_pct"]);
    assert_only("en", &en, "turn on lamp 2", &["light.lamp_2"]);

    let de = collision_home("de");
    assert_only("de", &de, "schalte Lampe 1 an", &["light.lampe_1"]);
    assert_intent("de", &de, "schalte Lampe 1 an", "HassTurnOn");
    assert_no_value_slot("de", &de, "schalte Lampe 1 an", &["brightness", "brightness_pct"]);
}

/// P-07: "Timer" inside a device name must not steal the utterance into a timer intent.
#[test]
fn p07_timer_in_a_device_name_is_the_device() {
    let en = collision_home("en");
    assert_only("en", &en, "turn on the timer lamp", &["light.timer_lamp"]);
    assert_intent("en", &en, "turn on the timer lamp", "HassTurnOn");
    let (out, _) = touched("en", &en, "turn on the timer lamp");
    let names: Vec<_> = out.plan.as_ref().map(|p| p.intents().iter().map(|i| i.name.clone()).collect()).unwrap_or_default();
    assert!(!names.iter().any(|n| n.contains("Timer")), "timer lamp must not become a timer: {names:?}");

    let de = collision_home("de");
    assert_only("de", &de, "schalte die Timer Lampe an", &["light.timer_lampe"]);
}

/// P-10: an exact switch name beats a domain word that also names another device.
#[test]
fn p10_exact_switch_beats_domain_word() {
    let en = collision_home("en");
    assert_only("en", &en, "turn on the tv plug", &["switch.tv_plug"]);
    assert_intent("en", &en, "turn on the tv plug", "HassTurnOn");

    let de = collision_home("de");
    assert_only("de", &de, "schalte die TV Steckdose ein", &["switch.tv_steckdose"]);
}

/// P-11: a device whose full name is "Office Light" wins over the whole office light domain.
#[test]
fn p11_named_office_light_is_not_the_whole_room() {
    let en = collision_home("en");
    assert_only("en", &en, "turn on the office light", &["light.office_light"]);
    assert_only("en", &en, "turn off the hall light", &["light.hall_light"]);

    let de = collision_home("de");
    assert_only("de", &de, "schalte das Büro Licht an", &["light.buro_licht"]);
    assert_only("de", &de, "mach das Flur Licht aus", &["light.flur_licht"]);
}

/// P-12: "alle" inside an entity_id is not a whole-home light group.
#[test]
fn p12_alle_in_entity_id_is_not_all_lights() {
    let areas = vec![area("gallery", "Galerie"), area("kitchen", "Küche")];
    let entities = vec![
        ent("light.gallery", "Galerie Licht", "gallery"),
        // entity_id contains "alle" the way HA group helpers often do
        ent("light.alle_lichter_galerie", "Galerie Gruppe", "gallery"),
        ent("light.kitchen_ceiling", "Küchenlicht", "kitchen"),
    ];
    let graph = home(areas, entities, &[]);
    let (out, devices) = touched("de", &graph, "schalte alle Lichter aus");
    // Whole-home "all lights" must not collapse onto the gallery group just because its id
    // contains "alle". It fans out to the real lights (and may include the gallery entity as
    // one room light, never as the sole home-wide group).
    assert!(devices.len() >= 2, "alle in entity_id must not be the sole home-wide group: {devices:?} {}", describe(&out));
    assert!(
        devices.iter().any(|id| id == "light.kitchen_ceiling"),
        "all-lights must reach the kitchen light, got {devices:?}: {}",
        describe(&out)
    );
    assert!(devices.iter().any(|id| id == "light.gallery"), "all-lights must reach the gallery light, got {devices:?}: {}", describe(&out));
}

/// P-28: a floor phrase scopes rooms that share a name.
#[test]
fn p28_floor_scopes_shared_room_names() {
    // Two bathrooms on different floors, same room name.
    use klar_nlu::types::{AreaRec, FloorRec};
    let mut graph = HomeGraph {
        floors: vec![
            FloorRec { floor_id: "ground".into(), name: "Ground Floor".into(), aliases: vec!["downstairs".into()], level: Some(0) },
            FloorRec { floor_id: "upper".into(), name: "Upstairs".into(), aliases: vec!["first floor".into()], level: Some(1) },
        ],
        areas: vec![
            AreaRec { area_id: "bathroom_gf".into(), name: "Bathroom".into(), aliases: vec![], floor_id: Some("ground".into()) },
            AreaRec { area_id: "bathroom_up".into(), name: "Bathroom".into(), aliases: vec![], floor_id: Some("upper".into()) },
            AreaRec { area_id: "kitchen".into(), name: "Kitchen".into(), aliases: vec![], floor_id: Some("ground".into()) },
        ],
        entities: vec![
            ent("light.bathroom_gf", "Bathroom Light", "bathroom_gf"),
            ent("light.bathroom_up", "Bathroom Light", "bathroom_up"),
            ent("light.kitchen", "Kitchen Light", "kitchen"),
        ],
        ..HomeGraph::default()
    };
    graph.assist = Some(graph.entities.iter().map(|e| e.entity_id.clone()).collect());

    assert_only("en", &graph, "turn on the bathroom light on the ground floor", &["light.bathroom_gf"]);
    assert_only("en", &graph, "turn on the bathroom light upstairs", &["light.bathroom_up"]);
}
