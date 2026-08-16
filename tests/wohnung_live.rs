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
        intent.map(|i| i.slots.iter().map(|s| (s.name.clone(), s.value.clone())).collect()).unwrap_or_default(),
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
    assert!(allowed.iter().any(|id| entity == Some(*id) || area == Some(*id) || hit == *id), "{text}: Ziel {found:?}, erlaubt {allowed:?}");
    for bad in forbidden {
        assert!(entity != Some(*bad), "{text}: verbotenes Gerät {bad} in {found:?}");
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
    assert_target("schalte im schlafzimmer das schlafzimmerlicht an", "HassTurnOn", kugel, forbid);
    assert_target("Kugel an", "HassTurnOn", kugel, forbid);
    assert_target("Kugel auf 40%", "HassLightSet", kugel, forbid);
    assert_target("Wie ist der Status vom Schlafzimmerlicht", "HassGetState", kugel, forbid);
    let (_, status, _) = slots("Wie ist der Status vom Schlafzimmerlicht");
    assert!(slot(&status, "entity_id").is_some(), "{status:?}");
    assert!(slot(&status, "area").is_none(), "{status:?}");
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
    assert_target("Küchenlicht an", "HassTurnOn", &["light.kuche_kuche", "kuche"], &["light.schlafzimmer_licht"]);
    assert_target(
        "Wohnzimmerlicht an",
        "HassTurnOn",
        &["wohnzimmer", "light.wohnzimmer"],
        &["light.satellite1_db12c8_led_ring", "light.home_assistant_voice_0a8d98_led_ring", "light.u7_pro_led"],
    );
    assert_target(
        "schalte das Wohnzimmerlicht an",
        "HassTurnOn",
        &["wohnzimmer", "light.wohnzimmer"],
        &["light.satellite1_db12c8_led_ring", "light.home_assistant_voice_0a8d98_led_ring", "light.u7_pro_led"],
    );
    assert_target(
        "Mach das Licht im Wohnzimmer an",
        "HassTurnOn",
        &["wohnzimmer", "light.wohnzimmer"],
        &["light.satellite1_db12c8_led_ring", "light.home_assistant_voice_0a8d98_led_ring"],
    );
    assert_target("Arbeitszimmerlicht aus", "HassTurnOff", &["light.arbeitszimmer", "arbeitszimmer"], &["switch.pc_steckdose"]);
    assert_target("Licht im Flur an", "HassTurnOn", &["flur"], &["light.u7_pro_led"]);
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
        slot(&found, "area") == Some("schlafzimmer") || entity == Some("light.schlafzimmer") || entity == Some("light.hue_color_lamp_2"),
        "{found:?}"
    );
}

#[test]
fn klima_sauger_timer_liste() {
    let (name, found, clarify) = slots("Wie warm ist es im Schlafzimmer");
    assert!(!clarify, "{found:?}");
    assert!(name == "HassClimateGetTemperature" || name == "HassGetState", "{name} {found:?}");
    assert!(
        slot(&found, "area") == Some("schlafzimmer") || slot(&found, "entity_id").is_some_and(|id| id.starts_with("climate.")),
        "{found:?}"
    );

    let (name, found, _) = slots("Heizung im Schlafzimmer");
    assert!(name == "HassGetState" || name.contains("Climate"), "{name} {found:?}");
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
fn geraet_nicht_raum_oder_szene() {
    assert_target("Schalte das Arbeitszimmer an", "HassTurnOn", &["light.arbeitszimmer"], &["switch.pc_steckdose", "fan.arc_casual"]);
    let (_, found, _) = slots("Schalte das Arbeitszimmer an");
    assert!(slot(&found, "area").is_none(), "{found:?}");

    assert_target(
        "Wie ist der Status von PC Steckdose",
        "HassGetState",
        &["switch.pc_steckdose"],
        &["switch.lars_pc", "switch.schlafzimmer_tv"],
    );
    assert_target("Wie ist der Status von PC Steckdose im Arbeitszimmer", "HassGetState", &["switch.pc_steckdose"], &["switch.lars_pc"]);
    assert_target("Schalte die PC Steckdose an", "HassTurnOn", &["switch.pc_steckdose"], &["switch.lars_pc"]);
    assert_target("Wie ist der Status von Schlafzimmer TV", "HassGetState", &["switch.schlafzimmer_tv"], &["media_player.schlafzimmer_2"]);
    assert_target(
        "Wie ist der Status von Schlafzimmer TV im Schlafzimmer",
        "HassGetState",
        &["switch.schlafzimmer_tv"],
        &["media_player.schlafzimmer_2"],
    );
    assert_target(
        "Wie ist der Status von Kugel im Schlafzimmer",
        "HassGetState",
        &["light.schlafzimmer", "light.hue_color_lamp_2"],
        &["light.schlafzimmer_licht", "switch.schlafzimmer_tv"],
    );
    assert_target("Wie ist der Status von Schlafzimmer", "HassGetState", &["schlafzimmer"], &["light.alle_lichter"]);
    assert_target("wie ist der status von der Küche?", "HassGetState", &["kuche"], &["light.kuche_kuche", "light.alle_lichter"]);
    assert_target("Wie ist der Status von der Küche", "HassGetState", &["kuche"], &["light.kuche_kuche"]);
    assert_target("Wie ist der Status der Küche?", "HassGetState", &["kuche"], &["light.kuche_kuche", "light.alle_lichter"]);
    assert_target("Wie ist der Status der Küche", "HassGetState", &["kuche"], &["light.kuche_kuche", "light.alle_lichter"]);
    assert_target("What's the status of the kitchen?", "HassGetState", &["kuche"], &["light.kuche_kuche", "light.alle_lichter"]);
    let (_, kitchen, _) = slots("Wie ist der Status der Küche?");
    assert_eq!(slot(&kitchen, "area"), Some("kuche"), "{kitchen:?}");
    assert!(slot(&kitchen, "entity_id").is_none(), "{kitchen:?}");

    let home = home();
    let mut session = Session::new();
    let chat = parse("Wie geht es dir?", &home, &mut session, &[], &Settings::default());
    assert!(chat.intents.is_empty(), "{:?}", chat.intents);
    assert!(chat.chat, "Smalltalk muss zum LLM");
    let status = parse("Wie ist der Status der Küche?", &home, &mut session, &[], &Settings::default());
    assert!(!status.chat, "{:?}", status.intents);
    assert_target("Wie ist der Status von Küche", "HassGetState", &["kuche"], &["light.kuche_kuche", "light.alle_lichter"]);
    assert_target(
        "Wie ist der Status von der Kugel",
        "HassGetState",
        &["light.schlafzimmer", "light.hue_color_lamp_2"],
        &["light.schlafzimmer_licht", "light.alle_lichter"],
    );
}

#[test]
fn alle_lichter_im_raum_nicht_ueberall() {
    let forbid = &["light.alle_lichter"];
    assert_target("Schalte alle Lichter im Schlafzimmer ein", "HassTurnOn", &["light.schlafzimmer", "schlafzimmer"], forbid);
    assert_target("Schalte alle Lichter im Arbeitszimmer ein", "HassTurnOn", &["light.arbeitszimmer", "arbeitszimmer"], forbid);
    assert_target("Schalte alle Lichter im Wohnzimmer ein", "HassTurnOn", &["light.wohnzimmer", "wohnzimmer"], forbid);
    assert_target("Schalte alle Lichter im Esszimmer ein", "HassTurnOn", &["light.esszimmer", "esszimmer"], forbid);
    assert_target("Schalte alle Lichter in der Küche ein", "HassTurnOn", &["light.kuche_kuche", "kuche"], forbid);
    assert_target(
        "Schalte alle Lichter in der Wohnung ein",
        "HassTurnOn",
        &["light.alle_lichter"],
        &["light.schlafzimmer", "light.wohnzimmer"],
    );
    assert_target("Schalte alle Lichter ein", "HassTurnOn", &["light.alle_lichter"], &["light.schlafzimmer"]);
}

#[test]
fn gruppen_ohne_bereichsslot() {
    assert_target("Wie ist der Status von Alle Lichter", "HassGetState", &["light.alle_lichter"], &["light.wohn_und_esszimmer"]);
    assert_target("Wie ist der Status der Leuchten", "HassGetState", &["light.alle_lichter"], &["light.wohn_und_esszimmer", "kuche"]);
    assert_target("Wie ist der Status von allen Lichtern", "HassGetState", &["light.alle_lichter"], &["light.wohn_und_esszimmer", "kuche"]);
    let (_, found, _) = slots("Wie ist der Status von Alle Lichter");
    assert!(slot(&found, "area").is_none(), "{found:?}");
    let (_, leuchten, _) = slots("Wie ist der Status der Leuchten");
    assert!(slot(&leuchten, "area").is_none(), "{leuchten:?}");
    assert_eq!(slot(&leuchten, "entity_id"), Some("light.alle_lichter"), "{leuchten:?}");
    assert_target(
        "Wie ist der Status von Alle Lichter in der Wohnung",
        "HassGetState",
        &["light.alle_lichter"],
        &["light.wohn_und_esszimmer"],
    );
    assert_target(
        "Wie ist der Status von Wohn und Esszimmer",
        "HassGetState",
        &["light.wohn_und_esszimmer"],
        &["light.esszimmer", "climate.better_thermostat_esszimmer"],
    );
    assert_target(
        "Wie ist der Status von Wohn und Esszimmer in der Wohnung",
        "HassGetState",
        &["light.wohn_und_esszimmer"],
        &["light.esszimmer"],
    );
    let mut home = home();
    for ent in &mut home.entities {
        if ent.entity_id == "light.wohn_und_esszimmer" {
            ent.name = "wohn_und_esszimmer".into();
            ent.aliases.clear();
        }
    }
    let mut session = Session::new();
    let asked = parse("Ist Wohn und Esszimmer an?", &home, &mut session, &[], &Settings::default());
    assert_eq!(asked.intents.len(), 1, "{:?} {}", asked.intents, asked.speech);
    assert_eq!(asked.intents[0].name, "HassGetState", "{:?}", asked.intents);
    let id = asked.intents[0].slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str());
    assert_eq!(id, Some("light.wohn_und_esszimmer"), "{:?}", asked.intents);
    assert!(!asked.intents.iter().any(|intent| intent.name == "HassTurnOn"), "{:?}", asked.intents);
}

#[test]
fn schalte_es_wieder_aus() {
    let home = home();
    let mut session = Session::new();
    let settings = Settings::default();
    let on = parse("schalte das schlafzimmerlicht ein", &home, &mut session, &[], &settings);
    let on_id = on.intents[0].slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str());
    assert_eq!(on.intents[0].name, "HassTurnOn", "{:?}", on.intents);
    assert!(matches!(on_id, Some("light.schlafzimmer") | Some("light.hue_color_lamp_2")), "{on_id:?}");
    assert!(!on.speech.contains(','), "{}", on.speech);
    assert!(!on.speech.to_lowercase().contains("schlafzimmer, schlafzimmer"), "{}", on.speech);

    let off = parse("schalte es wieder aus", &home, &mut session, &[], &settings);
    assert!(!off.clarify, "{}", off.speech);
    assert_eq!(off.intents[0].name, "HassTurnOff", "{:?} {}", off.intents, off.speech);
    let off_id = off.intents[0].slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str());
    assert_eq!(off_id, on_id, "{:?} {}", off.intents, off.speech);
    assert_ne!(off_id, Some("scene.alles_aus"), "{:?} {}", off.intents, off.speech);
    assert!(!off.speech.contains("alles"), "{}", off.speech);
    assert!(!off.speech.contains("aus aus"), "{}", off.speech);

    let on_again = parse("schalte es wieder ein", &home, &mut session, &[], &settings);
    assert_eq!(on_again.intents[0].name, "HassTurnOn", "{:?} {}", on_again.intents, on_again.speech);
    let again_id = on_again.intents[0].slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str());
    assert_eq!(again_id, on_id, "{:?} {}", on_again.intents, on_again.speech);
}

#[test]
fn esszimmer_licht_ist_keine_szene() {
    assert_target(
        "Schalte Esszimmer Licht an",
        "HassTurnOn",
        &["light.esszimmer"],
        &["scene.esszimmer_abendessen", "scene.wohnzimmer_lesen", "scene.wohnzimmer_hell"],
    );
    assert_target("Filmabend", "HassTurnOn", &["scene.wohnzimmer_filmabend"], &["light.wohnzimmer"]);
}

#[test]
fn tv_luefter_nicht_medienraum() {
    assert_target(
        "Schlafzimmer TV an",
        "HassTurnOn",
        &["switch.schlafzimmer_tv"],
        &["media_player.schlafzimmer_2", "light.schlafzimmer_licht"],
    );
    assert_target("Lüfter an", "HassTurnOn", &["fan.arc_casual"], &["switch.pc_steckdose"]);
}

#[test]
fn r2d2_ist_an_startet_nicht() {
    assert_target("Ist R2D2 an?", "HassGetState", &["vacuum.r2d2"], &["switch.r2d2_fill_light"]);
    assert_target("Ist R2D2 im Wohnzimmer an?", "HassGetState", &["vacuum.r2d2"], &["switch.r2d2_fill_light"]);
    let (name, found, _) = slots("R2D2 an");
    assert_eq!(name, "HassVacuumStart", "{found:?}");
    assert_eq!(slot(&found, "entity_id"), Some("vacuum.r2d2"));
}

#[test]
fn arc_casual_im_raum_bleibt_luefter() {
    assert_target("Schalte ARC Casual an", "HassTurnOn", &["fan.arc_casual"], &["light.arbeitszimmer"]);
    assert_target("Schalte ARC Casual im Arbeitszimmer an", "HassTurnOn", &["fan.arc_casual"], &["light.arbeitszimmer"]);
    assert_target("Wie ist der Status von ARC Casual im Arbeitszimmer", "HassGetState", &["fan.arc_casual"], &["light.arbeitszimmer"]);
}

#[test]
fn schlafzimmer_raum_schaltet_nicht_nur_kugel() {
    let (name, found, clarify) = slots("Schalte Schlafzimmer an");
    assert_eq!(name, "HassTurnOn", "{found:?}");
    assert!(!clarify, "{found:?}");
    assert!(slot(&found, "area") == Some("schlafzimmer") || slot(&found, "entity_id") == Some("light.schlafzimmer_licht"), "{found:?}");
    assert_ne!(slot(&found, "entity_id"), Some("light.schlafzimmer"), "{found:?}");
    let (name, found, clarify) = slots("Licht im Schlafzimmer an");
    assert_eq!(name, "HassTurnOn", "{found:?}");
    if !clarify {
        assert_eq!(slot(&found, "area"), Some("schlafzimmer"), "{found:?}");
        assert_ne!(slot(&found, "entity_id"), Some("light.schlafzimmer_licht"), "{found:?}");
    }
}

#[test]
fn kuechenlicht_ohne_alias_trifft_kueche() {
    let mut home = home();
    for ent in &mut home.entities {
        if ent.entity_id == "light.kuche_kuche" {
            ent.name = "Licht".into();
            ent.aliases.clear();
        }
    }
    let mut session = Session::new();
    for text in ["Küchenlicht an", "Wie ist der Status von Küchenlicht", "Schalte das Küchenlicht in der Küche an"] {
        let result = parse(text, &home, &mut session, &[], &Settings::default());
        let intent = result.intents.first().expect(text);
        let entity = intent.slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str());
        let area = intent.slots.iter().find(|s| s.name == "area").map(|s| s.value.as_str());
        assert!(!result.clarify, "{text}: {entity:?} {area:?}");
        assert_ne!(entity, Some("light.alle_lichter"), "{text}: {:?}", intent.slots);
        assert!(entity == Some("light.kuche_kuche") || area == Some("kuche"), "{text}: {:?}", intent.slots);
    }
}

#[test]
fn wohnzimmer_temperatur_ist_klima() {
    let (name, found, clarify) = slots("Wie ist die Temperatur im Wohnzimmer?");
    assert!(!clarify, "{found:?}");
    assert_eq!(name, "HassClimateGetTemperature", "{name} {found:?}");
    assert_eq!(slot(&found, "area"), Some("wohnzimmer"), "{found:?}");
    assert!(slot(&found, "entity_id").is_none_or(|id| id.starts_with("climate.")), "{found:?}");
}

fn parse_live(text: &str) -> klar_nlu::types::ParseResult {
    parse(text, &home(), &mut Session::new(), &[], &Settings::default())
}

fn intent_targets(result: &klar_nlu::types::ParseResult) -> Vec<(String, Option<String>, Option<String>)> {
    result
        .intents
        .iter()
        .map(|intent| {
            let entity = intent.slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.clone());
            let area = intent.slots.iter().find(|s| s.name == "area").map(|s| s.value.clone());
            (intent.name.clone(), entity, area)
        })
        .collect()
}

#[test]
fn wohn_und_esszimmer_lichte_aus_sind_beide_raeume() {
    let text = "Wohn und Esszimmer lichte aus";
    let result = parse_live(text);
    assert!(!result.clarify, "{text}: {}", result.speech);
    let targets = intent_targets(&result);
    assert_eq!(targets.len(), 2, "{text}: {targets:?}");
    assert!(targets.iter().all(|(name, _, _)| name == "HassTurnOff"), "{text}: {targets:?}");
    let rooms: Vec<&str> = targets.iter().filter_map(|(_, entity, area)| entity.as_deref().or(area.as_deref())).collect();
    assert!(rooms.contains(&"light.wohnzimmer") || rooms.contains(&"wohnzimmer"), "{text}: {targets:?}");
    assert!(rooms.contains(&"light.esszimmer") || rooms.contains(&"esszimmer"), "{text}: {targets:?}");
    assert!(!rooms.contains(&"light.wohn_und_esszimmer") && !rooms.contains(&"light.alle_lichter"), "{text}: {targets:?}");
}

#[test]
fn alle_lichter_aus_ausser_der_kugel() {
    let text = "Alle Lichter aus außer der Kugel";
    let result = parse_live(text);
    assert!(!result.clarify, "{text}: {}", result.speech);
    let targets = intent_targets(&result);
    assert!(targets.iter().all(|(name, _, _)| name == "HassTurnOff"), "{text}: {targets:?}");
    let ids: Vec<&str> = targets.iter().filter_map(|(_, entity, _)| entity.as_deref()).collect();
    let areas: Vec<&str> = targets.iter().filter_map(|(_, _, area)| area.as_deref()).collect();
    assert!(!ids.contains(&"light.alle_lichter"), "{text}: {targets:?}");
    assert!(!ids.contains(&"light.wohn_und_esszimmer"), "{text}: {targets:?}");
    assert!(!ids.contains(&"light.schlafzimmer"), "{text}: {targets:?}");
    assert!(!ids.contains(&"light.hue_color_lamp_2"), "{text}: {targets:?}");
    assert!(ids.contains(&"light.wohnzimmer") || areas.contains(&"wohnzimmer"), "{text}: {targets:?}");
    assert!(ids.contains(&"light.esszimmer") || areas.contains(&"esszimmer"), "{text}: {targets:?}");
    assert!(ids.contains(&"light.schlafzimmer_licht") || ids.contains(&"light.schlafzimmer_ambilight"), "{text}: {targets:?}");
}

#[test]
fn alle_lichter_ausser_schlafzimmer_aus() {
    let text = "Alle Lichter außer Schlafzimmer ausschalten";
    let result = parse_live(text);
    assert!(!result.clarify, "{text}: {}", result.speech);
    let targets = intent_targets(&result);
    assert!(targets.iter().all(|(name, _, _)| name == "HassTurnOff"), "{text}: {targets:?}");
    let ids: Vec<&str> = targets.iter().filter_map(|(_, entity, _)| entity.as_deref()).collect();
    let areas: Vec<&str> = targets.iter().filter_map(|(_, _, area)| area.as_deref()).collect();
    assert!(!ids.contains(&"light.alle_lichter"), "{text}: {targets:?}");
    assert!(!areas.contains(&"schlafzimmer"), "{text}: {targets:?}");
    assert!(!ids.iter().any(|id| id.contains("schlafzimmer") || *id == "light.hue_color_lamp_2"), "{text}: {targets:?}");
    assert!(ids.contains(&"light.wohnzimmer") || areas.contains(&"wohnzimmer"), "{text}: {targets:?}");
    assert!(ids.contains(&"light.esszimmer") || areas.contains(&"esszimmer"), "{text}: {targets:?}");
}

#[test]
fn schlafzimmern_licht_auf_rot() {
    let text = "Schlafzimmern Licht auf Rot";
    let result = parse_live(text);
    assert!(!result.clarify, "{text}: {}", result.speech);
    let targets = intent_targets(&result);
    assert_eq!(targets.len(), 1, "{text}: {targets:?}");
    assert_eq!(targets[0].0, "HassLightSet", "{text}: {targets:?}");
    let entity = targets[0].1.as_deref();
    let area = targets[0].2.as_deref();
    assert!(
        area == Some("schlafzimmer") || entity == Some("light.schlafzimmer") || entity == Some("light.hue_color_lamp_2"),
        "{text}: {targets:?}"
    );
    assert_ne!(entity, Some("light.alle_lichter"), "{text}: {targets:?}");
    let color = result.intents[0].slots.iter().find(|s| s.name == "color").map(|s| s.value.as_str());
    assert_eq!(color, Some("red"), "{text}: {:?}", result.intents);
    assert!(result.speech.to_lowercase().contains("rot"), "{text}: {}", result.speech);
    assert!(!result.speech.contains('?'), "{text}: {}", result.speech);
    assert!(!result.speech.to_lowercase().contains("prozent"), "{text}: {}", result.speech);
}

#[test]
fn licht_befehle_bleiben_treffsicher() {
    let result = parse_live("Schlafzimmern Licht auf rot");
    assert_eq!(result.intents[0].name, "HassLightSet", "{:?}", result.intents);
    assert!(result.intents[0].slots.iter().any(|s| s.name == "color" && s.value == "red"), "{:?}", result.intents);

    let rooms = parse_live("Wohn und Esszimmer auf Rot");
    let targets = intent_targets(&rooms);
    assert!(targets.iter().all(|(name, _, _)| name == "HassLightSet"), "{targets:?}");
    assert_eq!(targets.len(), 2, "{targets:?}");
    let rooms_hit: Vec<&str> = targets.iter().filter_map(|(_, entity, area)| entity.as_deref().or(area.as_deref())).collect();
    assert!(rooms_hit.contains(&"light.wohnzimmer") || rooms_hit.contains(&"wohnzimmer"), "{targets:?}");
    assert!(rooms_hit.contains(&"light.esszimmer") || rooms_hit.contains(&"esszimmer"), "{targets:?}");

    for text in ["Alle Lichter ohne die Kugel aus", "Alle Lichter aus aber nicht die Kugel"] {
        let excepted = parse_live(text);
        let ids: Vec<&str> =
            excepted.intents.iter().filter_map(|i| i.slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str())).collect();
        assert!(!ids.contains(&"light.alle_lichter"), "{text}: {ids:?}");
        assert!(!ids.contains(&"light.schlafzimmer"), "{text}: {ids:?}");
        assert!(!ids.contains(&"light.hue_color_lamp_2"), "{text}: {ids:?}");
    }

    let skip_room = parse_live("Alle Lichter außer Schlafzimmer und Küche aus");
    let skip_ids: Vec<&str> =
        skip_room.intents.iter().filter_map(|i| i.slots.iter().find(|s| s.name == "entity_id").map(|s| s.value.as_str())).collect();
    let skip_areas: Vec<&str> =
        skip_room.intents.iter().filter_map(|i| i.slots.iter().find(|s| s.name == "area").map(|s| s.value.as_str())).collect();
    assert!(!skip_areas.contains(&"schlafzimmer") && !skip_areas.contains(&"kuche"), "{:?}", skip_room.intents);
    assert!(!skip_ids.iter().any(|id| id.contains("schlafzimmer") || *id == "light.kuche_kuche"), "{:?}", skip_room.intents);
    assert!(skip_ids.contains(&"light.wohnzimmer") || skip_areas.contains(&"wohnzimmer"), "{:?}", skip_room.intents);
}
