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

fn klima_ohne_alias_trifft_ac() {
    let mut graph = home();
    for ent in &mut graph.entities {
        if ent.entity_id == "climate.schlafzimmer_ac" {
            ent.aliases.clear();
        }
    }
    let mut session = Session::new();
    let result = parse("Klimaanlage auf 20 Grad", &graph, &mut session, &[], &Settings::default());
    let found: Vec<(String, String)> =
        result.intents.first().map(|i| i.slots.iter().map(|s| (s.name.clone(), s.value.clone())).collect()).unwrap_or_default();
    assert!(!result.clarify, "Klimaanlage ohne Alias: nachgefragt {found:?}");
    assert_eq!(result.intents.first().map(|i| i.name.as_str()), Some("HassClimateSetTemperature"), "{found:?}");
    assert_eq!(slot(&found, "entity_id"), Some("climate.schlafzimmer_ac"), "{found:?}");
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
    klima_ohne_alias_trifft_ac();
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

fn mittel() -> HomeGraph {
    klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/wohnung_mittel/home_config.yaml")).expect("mittel")
}

fn parse_mittel(text: &str, session: &mut Session) -> klar_nlu::types::ParseResult {
    parse(text, &mittel(), session, &[], &Settings::default())
}

#[test]
fn status_von_kueche_leuchten_kugel() {
    for text in ["Wie ist der Status von Küche", "Wie ist der Status der Küche"] {
        let kitchen = parse_mittel(text, &mut Session::new());
        assert!(!kitchen.clarify && !kitchen.chat, "{text}: {:?}", kitchen.intents);
        assert_eq!(kitchen.intents[0].name, "HassGetState", "{text}: {:?}", kitchen.intents);
        assert_eq!(kitchen.intents[0].slot("area"), Some("kuche"), "{text}: {:?}", kitchen.intents);
        assert!(kitchen.intents[0].slot("entity_id").is_none(), "{text}: {:?}", kitchen.intents);
    }

    for text in ["Wie ist der Status der Leuchten", "Wie ist der Status von allen Lichtern"] {
        let result = parse_mittel(text, &mut Session::new());
        assert!(!result.clarify && !result.chat, "{text}: {:?}", result.intents);
        assert_eq!(result.intents[0].name, "HassGetState", "{text}: {:?}", result.intents);
        assert_eq!(result.intents[0].slot("entity_id"), Some("light.alle_lichter"), "{text}: {:?}", result.intents);
        assert!(result.intents[0].slot("area").is_none(), "{text}: {:?}", result.intents);
    }

    let kugel = parse_mittel("Wie ist der Status von der Kugel", &mut Session::new());
    assert!(!kugel.clarify && !kugel.chat, "{:?}", kugel.intents);
    assert_eq!(kugel.intents[0].name, "HassGetState", "{:?}", kugel.intents);
    assert_eq!(kugel.intents[0].slot("entity_id"), Some("light.schlafzimmer_kugel"), "{:?}", kugel.intents);
}

fn intent_ids(result: &klar_nlu::types::ParseResult) -> Vec<&str> {
    result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect()
}

#[test]
fn kugel_und_deckenlampe_aus() {
    let result = parse_mittel("Kugel und Deckenlampe aus", &mut Session::new());
    assert!(!result.clarify, "{:?}", result.intents);
    let ids = intent_ids(&result);
    assert!(ids.contains(&"light.schlafzimmer_kugel"), "{ids:?}");
    assert!(ids.contains(&"light.schlafzimmer_decke"), "{ids:?}");
}

#[test]
fn wohnzimer_trifft_wohnzimmer() {
    let result = parse_mittel("Wohnzimer Licht aus", &mut Session::new());
    assert!(!result.clarify, "{:?}", result.intents);
    assert_eq!(result.intents[0].name, "HassTurnOff", "{:?}", result.intents);
    let entity = result.intents[0].slot("entity_id").unwrap_or("");
    let area = result.intents[0].slot("area").unwrap_or("");
    assert!(area == "wohnzimmer" || entity.contains("wohnzimmer"), "{:?}", result.intents);
}

#[test]
fn schlafzimer_licht_aus_ist_raum() {
    let result = parse_mittel("Schlafzimer Lichte aus", &mut Session::new());
    assert!(!result.clarify, "{:?}", result.intents);
    assert_eq!(result.intents[0].name, "HassTurnOff", "{:?}", result.intents);
    let area = result.intents[0].slot("area");
    let entity = result.intents[0].slot("entity_id").unwrap_or("");
    assert!(area == Some("schlafzimmer") || entity.contains("schlafzimmer"), "{:?}", result.intents);
}

#[test]
fn wohnzimmer_dann_kueche_auch() {
    let mut session = Session::new();
    let first = parse_mittel("Licht im Wohnzimmer an", &mut session);
    assert_eq!(first.intents[0].name, "HassTurnOn", "{:?}", first.intents);
    let second = parse_mittel("und die Küche auch", &mut session);
    assert!(!second.clarify, "{:?}", second.intents);
    assert_eq!(second.intents[0].name, "HassTurnOn", "{:?}", second.intents);
    let entity = second.intents[0].slot("entity_id").unwrap_or("");
    let area = second.intents[0].slot("area").unwrap_or("");
    assert!(area == "kuche" || entity.contains("kuche"), "{:?}", second.intents);
    assert_ne!(entity, "light.wohn_und_esszimmer", "{:?}", second.intents);
}

#[test]
fn schlafzimmern_licht_auf_rot() {
    let home = klar_nlu::home::default_home();
    let mut session = Session::new();
    let result = parse("Schlafzimmern Licht auf Rot", &home, &mut session, &[], &Settings::default());
    assert_eq!(result.intents[0].name, "HassLightSet", "{:?}", result.intents);
    assert_eq!(result.intents[0].slot("color"), Some("red"));
    let entity = result.intents[0].slot("entity_id").unwrap_or("");
    let area = result.intents[0].slot("area").unwrap_or("");
    assert!(area == "schlafzimmer" || entity.contains("schlafzimmer"), "{:?}", result.intents);
}

#[test]
fn wohn_und_esszimmer_auf_rot() {
    let home = klar_nlu::home::default_home();
    let mut session = Session::new();
    let result = parse("Wohn und Esszimmer auf Rot", &home, &mut session, &[], &Settings::default());
    assert_eq!(result.intents.len(), 2, "{:?}", result.intents);
    assert!(
        result.intents.iter().all(|intent| intent.name == "HassLightSet" && intent.slot("color") == Some("red")),
        "{:?}",
        result.intents
    );
}
