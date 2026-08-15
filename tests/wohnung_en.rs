//! English phrases against the live Wohnung graph. Parse-only.

use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Settings};

fn home() -> HomeGraph {
    serde_json::from_str(include_str!("fixtures/wohnung_live.json")).expect("wohnung_live.json")
}

fn settings() -> Settings {
    Settings { languages: vec!["en".into()], ..Settings::default() }
}

fn parse_one(text: &str) -> (String, Vec<(String, String)>, bool, String) {
    let home = home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &settings());
    let intent = result.intents.first();
    (
        intent.map(|i| i.name.clone()).unwrap_or_default(),
        intent.map(|i| i.slots.iter().map(|s| (s.name.clone(), s.value.clone())).collect()).unwrap_or_default(),
        result.clarify,
        result.speech,
    )
}

fn slot<'a>(found: &'a [(String, String)], name: &str) -> Option<&'a str> {
    found.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
}

fn expect(text: &str, intent: &str, allowed: &[&str], forbidden: &[&str]) {
    let (name, found, clarify, speech) = parse_one(text);
    assert!(!clarify, "{text}: clarify {found:?} {speech}");
    assert_eq!(name, intent, "{text}: {found:?} {speech}");
    assert!(!speech.contains("Schalte") && !speech.contains("Setze") && !speech.contains("Frage"), "{text}: {speech}");
    let entity = slot(&found, "entity_id");
    let area = slot(&found, "area");
    assert!(allowed.iter().any(|id| entity == Some(*id) || area == Some(*id)), "{text}: got {found:?}, allowed {allowed:?}");
    for bad in forbidden {
        assert!(entity != Some(*bad), "{text}: forbidden {bad} in {found:?}");
    }
}

const GROUP: &str = "light.schlafzimmer_licht";
const KUGEL: &[&str] = &["light.schlafzimmer", "light.hue_color_lamp_2"];
const LED: &[&str] = &["light.satellite1_db12c8_led_ring", "light.home_assistant_voice_0a8d98_led_ring", "light.u7_pro_led"];

#[test]
fn bedroom_compounds_and_globe() {
    expect("Set bedroomlight to 50%", "HassLightSet", KUGEL, &[GROUP]);
    let (_, found, _, _) = parse_one("Set bedroomlight to 50%");
    assert_eq!(slot(&found, "brightness"), Some("50"), "{found:?}");
    expect("Turn on the bedroomlight", "HassTurnOn", KUGEL, &[GROUP]);
    expect(
        "Turn on the bedroom light in the bedroom",
        "HassTurnOn",
        &["light.schlafzimmer", "light.hue_color_lamp_2", "schlafzimmer"],
        &[GROUP],
    );
    expect("Turn on the globe", "HassTurnOn", KUGEL, &[GROUP]);
    expect("Turn on Kugel", "HassTurnOn", KUGEL, &[GROUP]);
    expect("Set Kugel to 40%", "HassLightSet", KUGEL, &[GROUP]);
}

#[test]
fn room_lights_english() {
    expect("Turn on the kitchen light", "HassTurnOn", &["light.kuche_kuche", "kuche"], &[GROUP]);
    expect("Turn on the kitchenlight", "HassTurnOn", &["light.kuche_kuche", "kuche"], &[GROUP]);
    expect("Turn on the living room light", "HassTurnOn", &["wohnzimmer", "light.wohnzimmer"], LED);
    expect("Turn off the office light", "HassTurnOff", &["light.arbeitszimmer", "arbeitszimmer"], &["switch.pc_steckdose"]);
    expect("Turn on the dining room light", "HassTurnOn", &["esszimmer", "light.esszimmer"], &["light.hue_color_spot_1"]);
    expect("Turn on the hallway light", "HassTurnOn", &["flur"], &["light.u7_pro_led"]);
    expect("Turn off all lights", "HassTurnOff", &["light.alle_lichter", "wohnung"], &["light.u7_pro_led"]);
}

#[test]
fn climate_vacuum_timer_list_en() {
    expect(
        "Set the bathroom heat to 21",
        "HassClimateSetTemperature",
        &["climate.better_thermostat_badezimmer", "badezimmer"],
        &["climate.schlafzimmer_ac"],
    );
    expect("Set the AC to 22", "HassClimateSetTemperature", &["climate.schlafzimmer_ac"], &["switch.153931629583704_power"]);
    expect("Where is R2D2", "HassGetState", &["vacuum.r2d2"], &["switch.r2d2_fill_light"]);
    expect("Send R2D2 to the station", "HassVacuumReturnToBase", &["vacuum.r2d2"], &["switch.r2d2_fill_light"]);
    expect("Start the vacuum", "HassVacuumStart", &["vacuum.r2d2"], &["switch.r2d2_fill_light"]);
    let (name, found, clarify, speech) = parse_one("Start a 5 minute timer");
    assert!(!clarify && !speech.contains("Schalte"), "{speech}");
    assert_eq!(name, "HassStartTimer", "{found:?}");
    assert_eq!(slot(&found, "minutes"), Some("5"), "{found:?}");
    let (name, found, _, speech) = parse_one("Add milk to the shopping list");
    assert_eq!(name, "HassListAddItem", "{found:?} {speech}");
}

#[test]
fn scenes_tv_pc_fan_en() {
    expect("Movie night", "HassTurnOn", &["scene.wohnzimmer_filmabend"], &["light.wohnzimmer"]);
    expect("Cozy", "HassTurnOn", &["scene.gemutlich"], &["light.wohnzimmer"]);
    expect("Turn on the floor lamp", "HassTurnOn", &["light.hue_color_spot_1"], &["light.esszimmer", GROUP]);
    expect(
        "Turn on the living room TV",
        "HassTurnOn",
        &["media_player.wohnzimmer_tv"],
        &["media_player.lg_dsn9yg_8909", "light.wohnzimmer"],
    );
    expect("Turn on the bedroom TV", "HassTurnOn", &["switch.schlafzimmer_tv"], &["media_player.schlafzimmer_2", GROUP]);
    expect("Turn on the PC", "HassTurnOn", &["switch.pc_steckdose"], &["light.arbeitszimmer"]);
    expect("Set the fan to 40", "HassFanSetSpeed", &["fan.arc_casual"], &["switch.pc_steckdose"]);
}

#[test]
fn dump_english_live() {
    if std::env::var("KLAR_DUMP").is_err() {
        return;
    }
    for text in [
        "Set bedroomlight to 50%",
        "Turn on the bedroom light",
        "Turn on the globe",
        "Turn on Kugel",
        "Turn on the kitchen light",
        "Turn on the kitchenlight",
        "Turn on the living room light",
        "Turn off the office light",
        "Turn on the dining room light",
        "Turn on the floor lamp",
        "Turn on ambilight",
        "Turn on the hallway light",
        "Set the bathroom heat to 21",
        "What is the temperature in the bedroom",
        "Set the AC to 22",
        "Turn off the AC",
        "Movie night",
        "Cozy",
        "Turn on the living room TV",
        "Turn on the bedroom TV",
        "Turn on the PC",
        "Start the vacuum",
        "Dock R2D2",
        "Send R2D2 to the station",
        "Set the fan to 40",
        "Start a 5 minute timer",
        "Add milk to the shopping list",
        "Turn off all lights",
        "Turn on the lights in the living room and the kitchen",
    ] {
        let (name, found, clarify, speech) = parse_one(text);
        println!("{text:?} => {name} {found:?} clarify={clarify} | {speech}");
    }
}
