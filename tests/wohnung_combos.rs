//! Weitere Sätze gegen den Live-Graphen. Parse-only.

use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Settings};

fn home() -> HomeGraph {
    serde_json::from_str(include_str!("fixtures/wohnung_live.json")).expect("wohnung_live.json")
}

fn parse_one(text: &str) -> (String, Vec<(String, String)>, bool) {
    let home = home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &Settings::default());
    let intent = result.intents.first();
    (
        intent.map(|i| i.name.clone()).unwrap_or_default(),
        intent.map(|i| i.slots.iter().map(|s| (s.name.clone(), s.value.clone())).collect()).unwrap_or_default(),
        result.clarify,
    )
}

fn slot<'a>(found: &'a [(String, String)], name: &str) -> Option<&'a str> {
    found.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
}

fn expect(text: &str, intent: &str, allowed: &[&str], forbidden: &[&str]) {
    let (name, found, clarify) = parse_one(text);
    assert!(!clarify, "{text}: nachgefragt {found:?}");
    assert_eq!(name, intent, "{text}: {found:?}");
    let entity = slot(&found, "entity_id");
    let area = slot(&found, "area");
    assert!(allowed.iter().any(|id| entity == Some(*id) || area == Some(*id)), "{text}: Ziel {found:?}, erlaubt {allowed:?}");
    for bad in forbidden {
        assert!(entity != Some(*bad), "{text}: verboten {bad} in {found:?}");
    }
}

#[test]
fn weitere_lichter() {
    expect("Esszimmerlicht an", "HassTurnOn", &["esszimmer", "light.esszimmer"], &["light.hue_color_spot_1", "light.schlafzimmer_licht"]);
    expect("Standleuchte an", "HassTurnOn", &["light.hue_color_spot_1"], &["light.esszimmer"]);
    expect("Ambilight an", "HassTurnOn", &["light.schlafzimmer_ambilight"], &["light.schlafzimmer_licht"]);
    expect("Wohnzimmerlicht auf 30%", "HassLightSet", &["wohnzimmer", "light.wohnzimmer"], &["light.satellite1_db12c8_led_ring"]);
    expect("Alle Lichter aus", "HassTurnOff", &["light.alle_lichter", "wohnung"], &["light.u7_pro_led"]);
}

#[test]
fn heizung_klima_sauger() {
    expect(
        "Heizung im Bad auf 21",
        "HassClimateSetTemperature",
        &["climate.better_thermostat_badezimmer", "badezimmer"],
        &["climate.schlafzimmer_ac"],
    );
    expect(
        "Klimaanlage auf 22",
        "HassClimateSetTemperature",
        &["climate.schlafzimmer_ac"],
        &["switch.153931629583704_power", "climate.better_thermostat_schlafzimmer"],
    );
    expect(
        "Klimaanlage auf 20",
        "HassClimateSetTemperature",
        &["climate.schlafzimmer_ac"],
        &["climate.better_thermostat_schlafzimmer", "climate.heizung_schlafzimmer"],
    );
    expect(
        "Klimaanlage auf 20 Grad",
        "HassClimateSetTemperature",
        &["climate.schlafzimmer_ac"],
        &["climate.better_thermostat_schlafzimmer"],
    );
    expect("Klimaanlage auf 20°", "HassClimateSetTemperature", &["climate.schlafzimmer_ac"], &["climate.better_thermostat_schlafzimmer"]);
    expect("Staubsauger Status", "HassGetState", &["vacuum.r2d2"], &["switch.r2d2_fill_light", "switch.r2d2_child_lock"]);
    expect("R2D2 zur Station", "HassVacuumReturnToBase", &["vacuum.r2d2"], &["switch.r2d2_fill_light"]);
    expect("Staubsauger starten", "HassVacuumStart", &["vacuum.r2d2"], &["switch.r2d2_fill_light"]);
}

#[test]
fn szene_tv_pc_r2d2_schalter() {
    expect("Filmabend", "HassTurnOn", &["scene.wohnzimmer_filmabend"], &["light.wohnzimmer"]);
    expect("Gemütlich", "HassTurnOn", &["scene.gemutlich"], &["light.wohnzimmer"]);
    expect("Wohnzimmer TV an", "HassTurnOn", &["media_player.wohnzimmer_tv"], &["media_player.lg_dsn9yg_8909", "light.wohnzimmer"]);
    expect("PC Steckdose an", "HassTurnOn", &["switch.pc_steckdose"], &["light.arbeitszimmer"]);
    expect("PC an", "HassTurnOn", &["switch.pc_steckdose"], &["light.arbeitszimmer"]);
    let (name, found, clarify) = parse_one("R2D2 an");
    assert!(!clarify, "R2D2 an: {found:?}");
    assert_ne!(slot(&found, "entity_id"), Some("switch.r2d2_fill_light"), "{name} {found:?}");
    assert_ne!(slot(&found, "entity_id"), Some("switch.r2d2_child_lock"), "{name} {found:?}");
}

#[test]
fn zwei_raeume_und_nachsatz() {
    let home = home();
    let mut session = Session::new();
    let settings = Settings::default();
    let first = parse("Mach das Licht im Wohnzimmer und in der Küche an", &home, &mut session, &[], &settings);
    let areas: Vec<_> = first
        .intents
        .iter()
        .filter(|i| i.name == "HassTurnOn")
        .flat_map(|i| i.slots.iter().filter(|s| s.name == "area").map(|s| s.value.as_str()))
        .collect();
    assert!(areas.contains(&"wohnzimmer") && areas.contains(&"kuche"), "{:?}", first.intents);
    let second = parse("Schlafzimmerlicht an", &home, &mut session, &[], &settings);
    let eid = second.intents[0].slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str());
    assert!(eid == Some("light.schlafzimmer") || eid == Some("light.hue_color_lamp_2"), "{:?}", second.intents);
}

#[test]
fn dump_weitere_saetze() {
    if std::env::var("KLAR_DUMP").is_err() {
        return;
    }
    for text in [
        "Esszimmerlicht an",
        "Standleuchte an",
        "Ambilight an",
        "Deckenlampe im Schlafzimmer an",
        "Badezimmerlicht an",
        "Licht im Bad an",
        "Hue Play an",
        "Wohnzimmerlicht auf 30%",
        "Heizung im Bad auf 21",
        "Wie warm ist es im Wohnzimmer",
        "Klimaanlage auf 22",
        "Klima aus",
        "Filmabend",
        "Aktiviere Filmabend",
        "Gemütlich",
        "Wohnzimmer TV an",
        "Fernseher im Wohnzimmer an",
        "PC an",
        "PC Steckdose aus",
        "R2D2 an",
        "Staubsauger starten",
        "Musik an",
        "Radio an",
        "Lüfter auf 40",
        "Timer 5 Minuten",
    ] {
        let (name, found, clarify) = parse_one(text);
        println!("{text:?} => {name} {found:?} clarify={clarify}");
    }
}
