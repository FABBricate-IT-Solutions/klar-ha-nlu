//! P-27: domain verbs, capitalization, step/position templates, no empty values (DE/EN).

use crate::speech::render_snapshot;
use crate::types::{SpeechEntity, SpeechIntent, SpeechSlot, SpeechSnapshot, UnitSystem};
use std::collections::BTreeMap;

fn snap(language: &str, name: &str, slots: &[(&str, &str)], entities: Vec<SpeechEntity>) -> SpeechSnapshot {
    SpeechSnapshot {
        schema_version: "1".into(),
        language: language.into(),
        personality: "default".into(),
        unit_system: UnitSystem::Metric,
        now: "2026-10-24T19:49:00+01:00".into(),
        intent: SpeechIntent {
            name: name.into(),
            slots: slots.iter().map(|(name, value)| SpeechSlot { name: (*name).into(), value: (*value).into() }).collect(),
        },
        outcome: "success".into(),
        entities,
        calendar_events: vec![],
        media_queue: vec![],
        forecast: vec![],
        hourly: vec![],
    }
}

fn entity(id: &str, name: &str, domain: &str) -> SpeechEntity {
    SpeechEntity {
        entity_id: id.into(),
        name: name.into(),
        domain: domain.into(),
        state: "unknown".into(),
        area: None,
        area_name: None,
        device_class: None,
        attributes: BTreeMap::new(),
    }
}

fn say(language: &str, intent: &str, slots: &[(&str, &str)], entities: Vec<SpeechEntity>) -> String {
    render_snapshot(&snap(language, intent, slots, entities)).speech
}

#[test]
fn p27_en_cover_lock_position_step_and_capitalization() {
    assert_eq!(
        say(
            "en",
            "HassTurnOn",
            &[("entity_id", "cover.office_blinds"), ("domain", "cover")],
            vec![entity("cover.office_blinds", "Office Blinds", "cover")]
        ),
        "Opened Office Blinds."
    );
    assert_eq!(
        say(
            "en",
            "HassTurnOn",
            &[("entity_id", "lock.front_door"), ("domain", "lock")],
            vec![entity("lock.front_door", "Front Door", "lock")]
        ),
        "Locked Front Door."
    );
    assert_eq!(
        say(
            "en",
            "HassSetPosition",
            &[("entity_id", "cover.office_blinds"), ("position", "50")],
            vec![entity("cover.office_blinds", "Office Blinds", "cover")],
        ),
        "Office Blinds is at 50 percent."
    );
    assert_eq!(
        say(
            "en",
            "HassLightSet",
            &[("entity_id", "light.desk_lamp"), ("brightness_step", "up")],
            vec![entity("light.desk_lamp", "Desk Lamp", "light")],
        ),
        "Desk Lamp is brighter."
    );
    assert_eq!(
        say(
            "en",
            "HassLightSet",
            &[("entity_id", "light.desk_lamp"), ("brightness_step", "down")],
            vec![entity("light.desk_lamp", "Desk Lamp", "light")],
        ),
        "Desk Lamp is dimmer."
    );
    let area = say("en", "HassTurnOn", &[("area", "kitchen"), ("domain", "light")], vec![]);
    assert_eq!(area, "The light in the kitchen is on.");
    assert!(!area.contains("is at  percent"), "{area}");
    assert!(!area.contains("Done:"), "{area}");
}

#[test]
fn p27_de_cover_lock_position_step() {
    assert_eq!(
        say("de", "HassTurnOn", &[("entity_id", "cover.rollo"), ("domain", "cover")], vec![entity("cover.rollo", "Rollo", "cover")]),
        "Rollo ist geöffnet."
    );
    assert_eq!(
        say("de", "HassTurnOn", &[("entity_id", "lock.haustuer"), ("domain", "lock")], vec![entity("lock.haustuer", "Haustür", "lock")]),
        "Haustür ist abgeschlossen."
    );
    assert_eq!(
        say("de", "HassSetPosition", &[("entity_id", "cover.rollo"), ("position", "30")], vec![entity("cover.rollo", "Rollo", "cover")]),
        "Rollo auf 30 Prozent."
    );
    assert_eq!(
        say(
            "de",
            "HassLightSet",
            &[("entity_id", "light.lampe"), ("brightness_step", "up")],
            vec![entity("light.lampe", "Lampe", "light")]
        ),
        "Lampe ist heller."
    );
    assert_eq!(
        say(
            "de",
            "HassLightSet",
            &[("entity_id", "light.lampe"), ("brightness_step", "down")],
            vec![entity("light.lampe", "Lampe", "light")],
        ),
        "Lampe ist dunkler."
    );
}

#[test]
fn p27_empty_position_or_brightness_does_not_leave_a_hole() {
    let missing_pos = say(
        "en",
        "HassSetPosition",
        &[("entity_id", "cover.office_blinds")],
        vec![entity("cover.office_blinds", "Office Blinds", "cover")],
    );
    assert!(!missing_pos.contains("is at  percent"), "{missing_pos}");
    assert!(!missing_pos.contains("Done: HassSetPosition"), "{missing_pos}");
    let missing_bright =
        say("en", "HassLightSet", &[("entity_id", "light.desk_lamp")], vec![entity("light.desk_lamp", "Desk Lamp", "light")]);
    assert!(!missing_bright.contains("is at  percent"), "{missing_bright}");
}
