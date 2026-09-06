use super::*;
use crate::types::{SpeechCalendarEvent, SpeechEntity, SpeechIntent, SpeechSlot, SpeechSnapshot, UnitSystem};
use std::collections::BTreeMap;

fn snap(name: &str, slots: Vec<SpeechSlot>, entities: Vec<SpeechEntity>) -> SpeechSnapshot {
    SpeechSnapshot {
        schema_version: "1".into(),
        language: "de".into(),
        personality: "default".into(),
        unit_system: UnitSystem::Metric,
        now: "2026-09-05T19:22:00+02:00".into(),
        intent: SpeechIntent { name: name.into(), slots },
        outcome: "success".into(),
        entities,
        calendar_events: vec![],
        media_queue: vec![],
    }
}

fn entity(id: &str, name: &str, domain: &str, state: &str, attrs: BTreeMap<String, serde_json::Value>) -> SpeechEntity {
    SpeechEntity {
        entity_id: id.into(),
        name: name.into(),
        domain: domain.into(),
        state: state.into(),
        area: None,
        area_name: None,
        device_class: None,
        attributes: attrs,
    }
}

#[test]
fn turn_on_uses_friendly_name_not_entity_id() {
    let out = render_snapshot(&snap(
        "HassTurnOn",
        vec![
            SpeechSlot { name: "entity_id".into(), value: "light.schlafzimmer".into() },
            SpeechSlot { name: "name".into(), value: "Kugel".into() },
        ],
        vec![],
    ));
    assert_eq!(out.source, "post_execute");
    assert!(out.speech.contains("Kugel"));
    assert!(!out.speech.contains("light."));
}

#[test]
fn kitchen_turn_on_names_the_room() {
    let out = render_snapshot(&snap(
        "HassTurnOn",
        vec![SpeechSlot { name: "entity_id".into(), value: "light.kuche_kuche".into() }],
        vec![entity("light.kuche_kuche", "Licht", "light", "on", BTreeMap::new())],
    ));
    assert!(out.speech.contains("Küche"));
    assert!(!out.speech.to_lowercase().contains("kuche kuche"));
}

#[test]
fn climate_set_speaks_degrees() {
    let out = render_snapshot(&snap(
        "HassClimateSetTemperature",
        vec![
            SpeechSlot { name: "name".into(), value: "Heizung Wohnzimmer".into() },
            SpeechSlot { name: "temperature".into(), value: "21".into() },
        ],
        vec![],
    ));
    assert_eq!(out.speech, "Heizung Wohnzimmer auf 21 Grad.");
    let missing = render_snapshot(&snap(
        "HassClimateSetTemperature",
        vec![SpeechSlot { name: "name".into(), value: "Heizung Wohnzimmer".into() }],
        vec![],
    ));
    assert_eq!(missing.speech, pack_for("de").unknown);
    assert!(!missing.speech.contains('?'));
}

#[test]
fn climate_speech_uses_operator_unit() {
    let mut set = snap(
        "HassClimateSetTemperature",
        vec![
            SpeechSlot { name: "name".into(), value: "Heizung Wohnzimmer".into() },
            SpeechSlot { name: "temperature".into(), value: "21".into() },
        ],
        vec![],
    );
    set.unit_system = UnitSystem::Imperial;
    assert_eq!(render_snapshot(&set).speech, "Heizung Wohnzimmer auf 70 Fahrenheit.");
    let climate = entity(
        "climate.wohnzimmer",
        "Heizung Wohnzimmer",
        "climate",
        "heat",
        BTreeMap::from([("current_temperature".into(), serde_json::json!(21.5)), ("temperature_unit".into(), serde_json::json!("°C"))]),
    );
    let mut query = snap(
        "HassClimateGetTemperature",
        vec![
            SpeechSlot { name: "area".into(), value: "wohnzimmer".into() },
            SpeechSlot { name: "area_name".into(), value: "Wohnzimmer".into() },
        ],
        vec![climate],
    );
    query.unit_system = UnitSystem::Imperial;
    assert_eq!(render_snapshot(&query).speech, "Wohnzimmer 70.7 Fahrenheit.");
}

#[test]
fn warm_white_speaks_color_not_percent() {
    let out = render_snapshot(&snap(
        "HassLightSet",
        vec![
            SpeechSlot { name: "entity_id".into(), value: "light.wohnzimmer".into() },
            SpeechSlot { name: "color".into(), value: "warmwhite".into() },
        ],
        vec![],
    ));
    assert!(out.speech.contains("warmweiß"));
    assert!(!out.speech.contains("Prozent"));
}

#[test]
fn tv_turn_on_does_not_claim_lights() {
    let out = render_snapshot(&snap(
        "HassTurnOn",
        vec![
            SpeechSlot { name: "entity_id".into(), value: "media_player.wohnzimmer_tv".into() },
            SpeechSlot { name: "name".into(), value: "Wohnzimmer TV".into() },
        ],
        vec![entity("media_player.wohnzimmer_tv", "Wohnzimmer TV", "media_player", "on", BTreeMap::new())],
    ));
    assert!(out.speech.contains("TV"));
    assert!(!out.speech.contains("Licht"));
}

#[test]
fn media_now_playing_uses_snapshot_attrs() {
    let mut attrs = BTreeMap::new();
    attrs.insert("media_title".into(), serde_json::json!("Bohemian Rhapsody"));
    attrs.insert("media_artist".into(), serde_json::json!("Queen"));
    let out = render_snapshot(&snap(
        "HassGetState",
        vec![
            SpeechSlot { name: "entity_id".into(), value: "media_player.wohnzimmer_2".into() },
            SpeechSlot { name: "media_status".into(), value: "now_playing".into() },
        ],
        vec![entity("media_player.wohnzimmer_2", "Wohnzimmer Soundbar", "media_player", "playing", attrs)],
    ));
    assert!(out.speech.contains("Bohemian Rhapsody"));
    assert!(out.speech.contains("Queen"));
}

#[test]
fn volume_and_mute_from_attrs() {
    let mut attrs = BTreeMap::new();
    attrs.insert("volume_level".into(), serde_json::json!(0.3));
    attrs.insert("is_volume_muted".into(), serde_json::json!(true));
    let mut snap = snap(
        "HassGetState",
        vec![SpeechSlot { name: "media_status".into(), value: "volume".into() }],
        vec![entity("media_player.wohnzimmer_2", "Living Room", "media_player", "playing", attrs)],
    );
    snap.language = "en".into();
    let out = render_snapshot(&snap);
    assert!(out.speech.contains("30 percent"));
    assert!(out.speech.contains("muted"));
}

#[test]
fn set_volume_and_relative() {
    let set = render_snapshot(&snap(
        "HassSetVolume",
        vec![
            SpeechSlot { name: "name".into(), value: "Wohnzimmer".into() },
            SpeechSlot { name: "volume_level".into(), value: "35".into() },
        ],
        vec![],
    ));
    assert_eq!(set.speech, "Die Lautstärke von Wohnzimmer ist auf 35 Prozent.");
    let rel = render_snapshot(&snap(
        "HassSetVolumeRelative",
        vec![
            SpeechSlot { name: "name".into(), value: "Wohnzimmer".into() },
            SpeechSlot { name: "volume_step".into(), value: "down".into() },
        ],
        vec![],
    ));
    assert!(rel.speech.contains("verringert"));
    assert!(!rel.speech.contains("HassSetVolumeRelative"));
}

#[test]
fn drops_satellite_from_climate_query() {
    let climate = entity(
        "climate.wohnzimmer",
        "Heizung Wohnzimmer",
        "climate",
        "heat",
        BTreeMap::from([("current_temperature".into(), serde_json::json!(21.5))]),
    );
    let sat = entity("sensor.satellite1_db12c8_temperature", "Satellite1 db12c8 Temperature", "sensor", "40.99", BTreeMap::new());
    let out = render_snapshot(&snap(
        "HassClimateGetTemperature",
        vec![
            SpeechSlot { name: "area".into(), value: "wohnzimmer".into() },
            SpeechSlot { name: "area_name".into(), value: "Wohnzimmer".into() },
        ],
        vec![climate, sat],
    ));
    assert!(out.speech.contains("Wohnzimmer"));
    assert!(!out.speech.contains("Satellite"));
    assert!(!out.speech.contains("40"));
}

#[test]
fn media_status_without_player_is_empty() {
    let out = render_snapshot(&snap(
        "HassGetState",
        vec![
            SpeechSlot { name: "area".into(), value: "wohnzimmer".into() },
            SpeechSlot { name: "media_status".into(), value: "now_playing".into() },
        ],
        vec![entity("light.wohnzimmer", "Wohnzimmer Licht", "light", "on", BTreeMap::new())],
    ));
    assert!(out.speech.is_empty());
}

#[test]
fn queue_empty_and_items() {
    let mut empty = snap("MassGetQueue", vec![], vec![entity("media_player.x", "Living Room", "media_player", "idle", BTreeMap::new())]);
    empty.language = "en".into();
    assert_eq!(render_snapshot(&empty).speech, "The queue is empty.");
    let mut playing = snap(
        "MassGetQueue",
        vec![],
        vec![entity(
            "media_player.x",
            "Living Room",
            "media_player",
            "playing",
            BTreeMap::from([("media_title".into(), serde_json::json!("A")), ("media_artist".into(), serde_json::json!("Artist"))]),
        )],
    );
    playing.language = "en".into();
    playing.media_queue = vec![
        crate::types::SpeechQueueItem { title: "A by Artist".into() },
        crate::types::SpeechQueueItem { title: "B".into() },
        crate::types::SpeechQueueItem { title: "C".into() },
    ];
    let spoken = render_snapshot(&playing).speech;
    assert!(spoken.contains("Now playing A by Artist"));
    assert!(spoken.contains("Next is B"));
    assert!(spoken.contains("Then C"));
}

#[test]
fn kitchen_status_and_french_off_word() {
    let item = snap(
        "HassGetState",
        vec![SpeechSlot { name: "area".into(), value: "kuche".into() }, SpeechSlot { name: "area_name".into(), value: "Küche".into() }],
        vec![entity("light.kuche_kuche", "Licht", "light", "off", BTreeMap::new())],
    );
    let de = render_snapshot(&item);
    assert!(de.speech.contains("Küche"));
    assert!(de.speech.contains("Licht aus"));
    assert!(!de.speech.contains("In der Küche"));
    let mut fr = item;
    fr.language = "fr".into();
    assert!(render_snapshot(&fr).speech.contains("Licht éteinte"));
}

#[test]
fn light_set_speaks_color() {
    let out = render_snapshot(&snap(
        "HassLightSet",
        vec![SpeechSlot { name: "area".into(), value: "schlafzimmer".into() }, SpeechSlot { name: "color".into(), value: "red".into() }],
        vec![],
    ));
    assert!(out.speech.contains("rot"));
    assert!(!out.speech.contains("Prozent"));
}

#[test]
fn kitchen_english_uses_pack_room() {
    let mut item = snap(
        "HassTurnOn",
        vec![SpeechSlot { name: "entity_id".into(), value: "light.kuche_kuche".into() }],
        vec![entity("light.kuche_kuche", "Licht", "light", "on", BTreeMap::new())],
    );
    item.language = "en".into();
    let spoken = render_snapshot(&item).speech.to_lowercase();
    assert!(spoken.contains("kitchen"));
}

#[test]
fn calendar_create_fills_summary_and_when() {
    let mut item = snap(
        "KlarCreateCalendarEvent",
        vec![
            SpeechSlot { name: "summary".into(), value: "zahnarzt".into() },
            SpeechSlot { name: "when".into(), value: "morgen um 15 Uhr".into() },
        ],
        vec![],
    );
    assert_eq!(render_snapshot(&item).speech, "zahnarzt morgen um 15 Uhr.");
    item.language = "en".into();
    assert_eq!(render_snapshot(&item).speech, "zahnarzt morgen um 15 Uhr.");
}

#[test]
fn calendar_list_and_empty_use_pack_lines() {
    let empty = snap("KlarGetCalendarEvents", vec![], vec![]);
    assert_eq!(render_snapshot(&empty).speech, pack_for("de").calendar_empty);
    let mut listed = snap("KlarGetCalendarEvents", vec![], vec![]);
    listed.calendar_events = vec![SpeechCalendarEvent { summary: "dentist".into(), start: "tomorrow 3pm".into() }];
    let spoken = render_snapshot(&listed).speech;
    assert!(spoken.contains("dentist"));
    assert!(spoken.contains("tomorrow 3pm"));
    let none = snap("KlarGetCalendarEvents", vec![SpeechSlot { name: "cue".into(), value: "none".into() }], vec![]);
    assert_eq!(render_snapshot(&none).speech, pack_for("de").calendar_none);
}

#[test]
fn floor_status_groups_rooms_and_empty_uses_pack_line() {
    let living = SpeechEntity {
        entity_id: "light.wohnzimmer".into(),
        name: "Wohnzimmer".into(),
        domain: "light".into(),
        state: "on".into(),
        area: Some("wohnzimmer".into()),
        area_name: Some("Wohnzimmer".into()),
        device_class: None,
        attributes: BTreeMap::new(),
    };
    let kitchen = SpeechEntity {
        entity_id: "light.kuche".into(),
        name: "Küche".into(),
        domain: "light".into(),
        state: "off".into(),
        area: Some("kuche".into()),
        area_name: Some("Küche".into()),
        device_class: None,
        attributes: BTreeMap::new(),
    };
    let out =
        render_snapshot(&snap("HassGetState", vec![SpeechSlot { name: "floor".into(), value: "wohnung".into() }], vec![kitchen, living]));
    assert!(out.speech.contains("Küche"));
    assert!(out.speech.contains("Wohnzimmer"));
    assert!(out.speech.contains("aus"));
    assert!(out.speech.contains("an"));
    let empty = render_snapshot(&snap("HassGetState", vec![SpeechSlot { name: "floor".into(), value: "wohnung".into() }], vec![]));
    assert_eq!(empty.speech, "Keine Geräte.");
    let mut french = snap("HassGetState", vec![SpeechSlot { name: "floor".into(), value: "wohnung".into() }], vec![]);
    french.language = "fr".into();
    assert_eq!(render_snapshot(&french).speech, "Aucun appareil.");
}
