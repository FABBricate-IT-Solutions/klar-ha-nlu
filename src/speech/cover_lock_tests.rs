//! Covers and locks speak their own state words instead of on/off.

use super::*;
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

fn entity(id: &str, name: &str, domain: &str, state: &str) -> SpeechEntity {
    SpeechEntity {
        entity_id: id.into(),
        name: name.into(),
        domain: domain.into(),
        state: state.into(),
        area: None,
        area_name: None,
        device_class: None,
        attributes: BTreeMap::new(),
    }
}

fn say(language: &str, intent: &str, id: &str, name: &str, domain: &str) -> String {
    render_snapshot(&snap(language, intent, &[("entity_id", id), ("domain", domain)], vec![entity(id, name, domain, "unknown")])).speech
}

#[test]
fn cover_open_and_close_use_cover_words() {
    let open = say("en", "HassTurnOn", "cover.bedroom_blinds", "Bedroom Blinds", "cover");
    assert_eq!(open, "Opened Bedroom Blinds.");
    let close = say("en", "HassTurnOff", "cover.bedroom_blinds", "Bedroom Blinds", "cover");
    assert_eq!(close, "Closed Bedroom Blinds.");
}

#[test]
fn lock_and_unlock_use_lock_words() {
    let lock = say("en", "HassTurnOn", "lock.front_door", "Front Door", "lock");
    assert_eq!(lock, "Locked Front Door.");
    let unlock = say("en", "HassTurnOff", "lock.front_door", "Front Door", "lock");
    assert_eq!(unlock, "Unlocked Front Door.");
}

#[test]
fn german_cover_words() {
    let open = say("de", "HassTurnOn", "cover.rollo", "Rollo", "cover");
    assert_eq!(open, "Rollo ist geöffnet.");
    let close = say("de", "HassTurnOff", "cover.rollo", "Rollo", "cover");
    assert_eq!(close, "Rollo ist geschlossen.");
}

#[test]
fn area_wide_covers_use_cover_words() {
    let entities = vec![
        entity("cover.bedroom_blinds", "Bedroom Blinds", "cover", "closed"),
        entity("cover.bedroom_curtains", "Bedroom Curtains", "cover", "open"),
    ];
    let out = render_snapshot(&snap("en", "HassTurnOn", &[("area", "bedroom"), ("domain", "cover")], entities)).speech;
    assert_eq!(out, "Opened Bedroom Blinds.");
}

#[test]
fn lights_still_say_on_and_off() {
    let on = say("en", "HassTurnOn", "light.bedroom_light", "Bedroom Light", "light");
    assert_eq!(on, "Bedroom Light is on.");
    let off = say("en", "HassTurnOff", "light.bedroom_light", "Bedroom Light", "light");
    assert_eq!(off, "Bedroom Light is off.");
}

#[test]
fn generated_locales_speak_the_name_until_worded() {
    let out = say("fr", "HassTurnOn", "cover.volet", "Volet", "cover");
    assert_eq!(out, "Volet.");
}
