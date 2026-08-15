//! Live-Wohnung: Graph aus Home Assistant (Stand 2026-08-15).
//! Parse-only — schaltet nichts.

use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Settings};

fn home() -> HomeGraph {
    serde_json::from_str(include_str!("fixtures/wohnung_live.json")).expect("wohnung_live.json")
}

fn slots(text: &str) -> (String, Vec<(String, String)>, bool) {
    let home = home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &Settings::default());
    let intent = result.intents.first();
    (
        intent.map(|i| i.name.clone()).unwrap_or_default(),
        intent
            .map(|i| i.slots.iter().map(|s| (s.name.clone(), s.value.clone())).collect())
            .unwrap_or_default(),
        result.clarify,
    )
}

fn slot<'a>(found: &'a [(String, String)], name: &str) -> Option<&'a str> {
    found.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
}

fn assert_target(text: &str, intent: &str, allowed: &[&str], forbidden: &[&str]) {
    let (name, found, clarify) = slots(text);
    assert!(!clarify, "{text}: unerwartet nachgefragt {found:?}");
    assert_eq!(name, intent, "{text}: {found:?}");
    let entity = slot(&found, "entity_id");
    let area = slot(&found, "area");
    let hit = entity.or(area).unwrap_or("");
    assert!(
        allowed.iter().any(|id| entity == Some(*id) || area == Some(*id) || hit == *id),
        "{text}: Ziel {found:?}, erlaubt {allowed:?}"
    );
    for bad in forbidden {
        assert!(
            entity != Some(*bad),
            "{text}: verbotenes Gerät {bad} in {found:?}"
        );
    }
}

#[test]
fn schlafzimmerlicht_ist_die_kugel() {
    let forbid = &[
        "light.schlafzimmer_licht",
        "light.schlafzimmer_ambilight",
        "light.schlafzimmer_schlafzimmer_deckenlicht_schlafzimmer_deckenlampe",
    ];
    let kugel = &["light.schlafzimmer", "light.hue_color_lamp_2"];
    assert_target("Schlafzimmerlicht auf 50%", "HassLightSet", kugel, forbid);
    let (_, found, _) = slots("Schlafzimmerlicht auf 50%");
    assert_eq!(slot(&found, "brightness"), Some("50"));
    assert_target("schalte das schlafzimmerlicht an", "HassTurnOn", kugel, forbid);
    assert_target(
        "schalte im schlafzimmer das schlafzimmerlicht an",
        "HassTurnOn",
        kugel,
        forbid,
    );
    assert_target("Kugel an", "HassTurnOn", kugel, forbid);
    assert_target("Kugel auf 40%", "HassLightSet", kugel, forbid);
}

#[test]
fn schlafzimmerlicht_ohne_hue_area_id() {
    let mut home = home();
    for ent in &mut home.entities {
        if ent.entity_id == "light.schlafzimmer" || ent.entity_id == "light.hue_color_lamp_2" {
            ent.area = None;
        }
    }
    let mut session = Session::new();
    let result = parse("Schlafzimmerlicht auf 50%", &home, &mut session, &[], &Settings::default());
    let intent = result.intents.first().expect("intent");
    let entity = intent.slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str());
    assert_eq!(entity, Some("light.schlafzimmer"), "{:?}", intent.slots);
    assert_ne!(entity, Some("light.schlafzimmer_ambilight"));
}

#[test]
fn raumlicht_ohne_diebstahl() {
    assert_target(
        "Küchenlicht an",
        "HassTurnOn",
        &["light.kuche_kuche", "kuche"],
        &["light.schlafzimmer_licht"],
    );
    assert_target(
        "Wohnzimmerlicht an",
        "HassTurnOn",
        &["wohnzimmer", "light.wohnzimmer"],
        &[
            "light.satellite1_db12c8_led_ring",
            "light.home_assistant_voice_0a8d98_led_ring",
            "light.u7_pro_led",
        ],
    );
    assert_target(
        "Mach das Licht im Wohnzimmer an",
        "HassTurnOn",
        &["wohnzimmer", "light.wohnzimmer"],
        &[
            "light.satellite1_db12c8_led_ring",
            "light.home_assistant_voice_0a8d98_led_ring",
        ],
    );
    assert_target(
        "Arbeitszimmerlicht aus",
        "HassTurnOff",
        &["light.arbeitszimmer", "arbeitszimmer"],
        &["switch.pc_steckdose"],
    );
    assert_target(
        "Licht im Flur an",
        "HassTurnOn",
        &["flur"],
        &["light.u7_pro_led"],
    );
}

#[test]
fn licht_im_schlafzimmer_bleibt_raum() {
    let (name, found, clarify) = slots("Licht im Schlafzimmer an");
    assert_eq!(name, "HassTurnOn", "{found:?}");
    if clarify {
        return;
    }
    let entity = slot(&found, "entity_id");
    assert_ne!(entity, Some("light.schlafzimmer_licht"), "{found:?}");
    assert!(
        slot(&found, "area") == Some("schlafzimmer")
            || entity == Some("light.schlafzimmer")
            || entity == Some("light.hue_color_lamp_2"),
        "{found:?}"
    );
}

#[test]
fn klima_sauger_timer_liste() {
    let (name, found, clarify) = slots("Wie warm ist es im Schlafzimmer");
    assert!(!clarify, "{found:?}");
    assert!(
        name == "HassClimateGetTemperature" || name == "HassGetState",
        "{name} {found:?}"
    );
    assert!(
        slot(&found, "area") == Some("schlafzimmer")
            || slot(&found, "entity_id").is_some_and(|id| id.starts_with("climate.")),
        "{found:?}"
    );

    let (name, found, _) = slots("Heizung im Schlafzimmer");
    assert!(
        name == "HassGetState" || name.contains("Climate"),
        "{name} {found:?}"
    );
    assert_ne!(slot(&found, "entity_id"), Some("climate.schlafzimmer_ac"), "{found:?}");

    let (name, found, clarify) = slots("Wo ist R2D2");
    assert!(!clarify, "{found:?}");
    assert_eq!(name, "HassGetState", "{found:?}");
    assert_eq!(slot(&found, "entity_id"), Some("vacuum.r2d2"), "{found:?}");

    let (name, found, _) = slots("Timer eine Minute");
    assert_eq!(name, "HassStartTimer", "{found:?}");
    assert!(slot(&found, "minutes") == Some("1") || slot(&found, "seconds") == Some("60"), "{found:?}");

    let (name, found, _) = slots("Timer abbrechen");
    assert_eq!(name, "HassCancelTimer", "{found:?}");

    let (name, found, _) = slots("Setz Milch auf die Einkaufsliste");
    assert_eq!(name, "HassListAddItem", "{found:?}");
}

#[test]
fn tv_luefter_nicht_medienraum() {
    assert_target(
        "Schlafzimmer TV an",
        "HassTurnOn",
        &["switch.schlafzimmer_tv"],
        &["media_player.schlafzimmer_2", "light.schlafzimmer_licht"],
    );
    assert_target(
        "Lüfter an",
        "HassTurnOn",
        &["fan.arc_casual"],
        &["switch.pc_steckdose"],
    );
}
