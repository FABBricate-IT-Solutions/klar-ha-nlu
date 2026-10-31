//! Replies that act on several devices name every one and use the plural verb.
//! Guards the "Close the bedroom blinds and curtains" → "Closed Bedroom Blinds" bug.

use super::*;
use crate::types::{SpeechEntity, SpeechIntent, SpeechSlot, SpeechSnapshot, UnitSystem};
use std::collections::BTreeMap;

fn ent(id: &str, name: &str) -> SpeechEntity {
    SpeechEntity {
        entity_id: id.into(),
        name: name.into(),
        domain: id.split_once('.').map(|(domain, _)| domain).unwrap_or("").into(),
        state: "unknown".into(),
        area: None,
        area_name: None,
        device_class: None,
        attributes: BTreeMap::new(),
    }
}

fn snap_with(language: &str, intent: &str, slots: &[(&str, &str)], entities: Vec<SpeechEntity>, outcome: &str) -> SpeechSnapshot {
    SpeechSnapshot {
        schema_version: "1".into(),
        language: language.into(),
        personality: "default".into(),
        unit_system: UnitSystem::Metric,
        now: "2026-10-29T11:00:00+00:00".into(),
        intent: SpeechIntent {
            name: intent.into(),
            slots: slots.iter().map(|(name, value)| SpeechSlot { name: (*name).into(), value: (*value).into() }).collect(),
        },
        outcome: outcome.into(),
        entities,
        calendar_events: vec![],
        media_queue: vec![],
        forecast: vec![],
        hourly: vec![],
    }
}

fn say(language: &str, intent: &str, slots: &[(&str, &str)], entities: Vec<SpeechEntity>) -> String {
    render_snapshot(&snap_with(language, intent, slots, entities, "success")).speech
}

fn bedroom_covers() -> Vec<SpeechEntity> {
    vec![ent("cover.bedroom_blinds", "Bedroom Blinds"), ent("cover.bedroom_curtains", "Bedroom Curtains")]
}

fn bedroom_lights() -> Vec<SpeechEntity> {
    vec![ent("light.bedroom_light", "Bedroom Light"), ent("light.bedroom_lamp", "Bedroom Lamp")]
}

const COVER_AREA: &[(&str, &str)] = &[("area", "bedroom"), ("domain", "cover")];
const LIGHT_AREA: &[(&str, &str)] = &[("area", "bedroom"), ("domain", "light")];

// ---------- English: covers ----------

#[test]
fn en_close_blinds_and_curtains_names_both() {
    assert_eq!(say("en", "HassTurnOff", COVER_AREA, bedroom_covers()), "Closed Bedroom Blinds and Bedroom Curtains.");
}

#[test]
fn en_open_blinds_and_curtains_names_both() {
    assert_eq!(say("en", "HassTurnOn", COVER_AREA, bedroom_covers()), "Opened Bedroom Blinds and Bedroom Curtains.");
}

#[test]
fn en_order_follows_the_snapshot() {
    let mut covers = bedroom_covers();
    covers.reverse();
    assert_eq!(say("en", "HassTurnOff", COVER_AREA, covers), "Closed Bedroom Curtains and Bedroom Blinds.");
}

#[test]
fn en_single_cover_is_unchanged() {
    let one = vec![ent("cover.bedroom_blinds", "Bedroom Blinds")];
    assert_eq!(say("en", "HassTurnOff", &[("entity_id", "cover.bedroom_blinds"), ("domain", "cover")], one), "Closed Bedroom Blinds.");
}

// ---------- English: lights ----------

#[test]
fn en_two_lights_on_and_off_use_are() {
    assert_eq!(say("en", "HassTurnOn", LIGHT_AREA, bedroom_lights()), "Bedroom Light and Bedroom Lamp are on.");
    assert_eq!(say("en", "HassTurnOff", LIGHT_AREA, bedroom_lights()), "Bedroom Light and Bedroom Lamp are off.");
}

#[test]
fn en_single_light_still_uses_is() {
    let one = vec![ent("light.bedroom_light", "Bedroom Light")];
    assert_eq!(say("en", "HassTurnOn", &[("entity_id", "light.bedroom_light"), ("domain", "light")], one), "Bedroom Light is on.");
}

#[test]
fn en_two_lights_brightness_uses_are() {
    let out = say("en", "HassLightSet", &[("area", "bedroom"), ("domain", "light"), ("brightness", "50")], bedroom_lights());
    assert_eq!(out, "Bedroom Light and Bedroom Lamp are at 50 percent.");
}

#[test]
fn en_two_lights_toggle_uses_are() {
    assert_eq!(say("en", "HassToggle", LIGHT_AREA, bedroom_lights()), "Bedroom Light and Bedroom Lamp are switched.");
}

#[test]
fn en_gb_two_lights_name_both() {
    let out = say("en-GB", "HassTurnOn", LIGHT_AREA, bedroom_lights());
    assert!(out.contains("Bedroom Light and Bedroom Lamp"), "got {out:?}");
    assert!(!out.contains(" is "), "singular verb with two targets: {out:?}");
}

// ---------- English: locks ----------

#[test]
fn en_two_locks_name_both() {
    let locks = vec![ent("lock.front_door", "Front Door"), ent("lock.back_door", "Back Door")];
    assert_eq!(say("en", "HassTurnOn", &[("domain", "lock")], locks.clone()), "Locked Front Door and Back Door.");
    assert_eq!(say("en", "HassTurnOff", &[("domain", "lock")], locks), "Unlocked Front Door and Back Door.");
}

// ---------- English: list shape ----------

#[test]
fn en_three_and_four_targets_use_commas_then_and() {
    let mut covers = bedroom_covers();
    covers.push(ent("cover.bedroom_shutter", "Bedroom Shutter"));
    assert_eq!(say("en", "HassTurnOff", COVER_AREA, covers.clone()), "Closed Bedroom Blinds, Bedroom Curtains and Bedroom Shutter.");
    covers.push(ent("cover.bedroom_awning", "Bedroom Awning"));
    assert_eq!(
        say("en", "HassTurnOff", COVER_AREA, covers),
        "Closed Bedroom Blinds, Bedroom Curtains, Bedroom Shutter and Bedroom Awning."
    );
}

#[test]
fn en_five_or_more_covers_are_all_named() {
    // No room phrase exists for covers, so every name is read out rather than just the first.
    let covers: Vec<_> = (1..=5).map(|n| ent(&format!("cover.bedroom_{n}"), &format!("Bedroom Blind {n}"))).collect();
    assert_eq!(
        say("en", "HassTurnOff", COVER_AREA, covers),
        "Closed Bedroom Blind 1, Bedroom Blind 2, Bedroom Blind 3, Bedroom Blind 4 and Bedroom Blind 5."
    );
}

#[test]
fn en_five_or_more_lights_are_spoken_as_the_room() {
    let lights: Vec<_> = ["Living Room Lamp", "Living Room Main Light", "Left Wall Light", "Right Wall Light", "Left Floor Light"]
        .iter()
        .enumerate()
        .map(|(n, name)| ent(&format!("light.living_{n}"), name))
        .collect();
    let slots = [("area", "Living Room"), ("domain", "light")];
    assert_eq!(say("en", "HassTurnOff", &slots, lights), "the light in the Living Room is off.");
}

#[test]
fn en_duplicate_names_ignore_case_and_are_spoken_once() {
    let covers = vec![ent("cover.bedroom_blinds", "Bedroom Blinds"), ent("cover.bedroom_blinds_2", "bedroom blinds")];
    assert_eq!(say("en", "HassTurnOff", COVER_AREA, covers), "Closed Bedroom Blinds.");
}

#[test]
fn en_unnamed_entities_are_skipped() {
    let covers = vec![ent("cover.bedroom_blinds", "Bedroom Blinds"), ent("cover.bedroom_curtains", "  ")];
    assert_eq!(say("en", "HassTurnOff", COVER_AREA, covers), "Closed Bedroom Blinds.");
}

#[test]
fn en_entity_id_shaped_names_are_skipped() {
    let covers = vec![ent("cover.bedroom_blinds", "Bedroom Blinds"), ent("cover.bedroom_curtains", "cover.bedroom_curtains")];
    assert_eq!(say("en", "HassTurnOff", COVER_AREA, covers), "Closed Bedroom Blinds.");
}

#[test]
fn en_infra_entities_are_not_read_out() {
    let mut lights = bedroom_lights();
    lights.push(ent("light.bedroom_satellite_led_ring", "Bedroom Satellite LED Ring"));
    assert_eq!(say("en", "HassTurnOn", LIGHT_AREA, lights), "Bedroom Light and Bedroom Lamp are on.");
}

#[test]
fn en_explicit_name_slot_beats_the_entity_list() {
    let slots = [("name", "Bedroom Blinds"), ("entity_id", "cover.bedroom_blinds"), ("domain", "cover")];
    assert_eq!(say("en", "HassTurnOff", &slots, bedroom_covers()), "Closed Bedroom Blinds.");
}

#[test]
fn en_error_outcome_never_claims_success() {
    let out = render_snapshot(&snap_with("en", "HassTurnOff", COVER_AREA, bedroom_covers(), "error")).speech;
    assert!(!out.contains("Closed"), "error must not report success: {out:?}");
}

#[test]
fn en_pluralize_never_touches_is_inside_a_name() {
    // A name containing "Is" must not become "Are".
    let lights = vec![ent("light.this_is_lamp", "This Is Lamp"), ent("light.bedroom_lamp", "Bedroom Lamp")];
    let out = say("en", "HassTurnOn", LIGHT_AREA, lights);
    assert_eq!(out, "This Is Lamp and Bedroom Lamp are on.");
}

// ---------- German ----------

fn schlafzimmer_covers() -> Vec<SpeechEntity> {
    vec![ent("cover.schlafzimmer_rollo", "Rollo"), ent("cover.schlafzimmer_vorhang", "Vorhang")]
}

const DE_COVER_AREA: &[(&str, &str)] = &[("area", "schlafzimmer"), ("domain", "cover")];
const DE_LIGHT_AREA: &[(&str, &str)] = &[("area", "schlafzimmer"), ("domain", "light")];

#[test]
fn de_close_rollo_und_vorhang_uses_sind() {
    assert_eq!(say("de", "HassTurnOff", DE_COVER_AREA, schlafzimmer_covers()), "Rollo und Vorhang sind geschlossen.");
}

#[test]
fn de_open_rollo_und_vorhang_uses_sind() {
    assert_eq!(say("de", "HassTurnOn", DE_COVER_AREA, schlafzimmer_covers()), "Rollo und Vorhang sind geöffnet.");
}

#[test]
fn de_single_cover_still_uses_ist() {
    let one = vec![ent("cover.schlafzimmer_rollo", "Rollo")];
    assert_eq!(say("de", "HassTurnOff", &[("entity_id", "cover.schlafzimmer_rollo"), ("domain", "cover")], one), "Rollo ist geschlossen.");
}

#[test]
fn de_two_lights_on_and_off_use_sind() {
    let lights = vec![ent("light.schlafzimmer_decke", "Deckenlicht"), ent("light.schlafzimmer_nachttisch", "Nachttischlampe")];
    assert_eq!(say("de", "HassTurnOn", DE_LIGHT_AREA, lights.clone()), "Deckenlicht und Nachttischlampe sind an.");
    assert_eq!(say("de", "HassTurnOff", DE_LIGHT_AREA, lights), "Deckenlicht und Nachttischlampe sind aus.");
}

#[test]
fn de_three_targets_use_commas_then_und() {
    let mut covers = schlafzimmer_covers();
    covers.push(ent("cover.schlafzimmer_jalousie", "Jalousie"));
    assert_eq!(say("de", "HassTurnOff", DE_COVER_AREA, covers), "Rollo, Vorhang und Jalousie sind geschlossen.");
}

#[test]
fn de_two_locks_use_sind() {
    let locks = vec![ent("lock.haustuer", "Haustür"), ent("lock.hintertuer", "Hintertür")];
    assert_eq!(say("de", "HassTurnOn", &[("domain", "lock")], locks), "Haustür und Hintertür sind abgeschlossen.");
}

#[test]
fn de_umlaut_duplicates_are_spoken_once() {
    let locks = vec![ent("lock.haustuer", "Haustür"), ent("lock.haustuer_2", "Haustür")];
    assert_eq!(say("de", "HassTurnOn", &[("domain", "lock")], locks), "Haustür ist abgeschlossen.");
}

#[test]
fn de_duplicates_ignore_non_ascii_case() {
    let locks = vec![ent("lock.haustuer", "Haustür"), ent("lock.haustuer_2", "HAUSTÜR")];
    assert_eq!(say("de", "HassTurnOn", &[("domain", "lock")], locks), "Haustür ist abgeschlossen.");
}

// ---------- Other languages are left alone ----------

#[test]
fn other_languages_get_names_joined_but_no_verb_rewrite() {
    let out = say("fr", "HassTurnOff", COVER_AREA, bedroom_covers());
    assert!(out.contains("Bedroom Blinds") && out.contains("Bedroom Curtains"), "got {out:?}");
}

// ---------- Plural only when the reply really lists several names ----------

#[test]
fn generic_first_light_keeps_the_room_phrase_singular() {
    // A first light called just "Light" makes the reply use the room phrase; it must not read "are".
    let lights = vec![ent("light.bedroom", "Light"), ent("light.bedroom_lamp", "Bedroom Lamp")];
    let out = say("en", "HassTurnOn", LIGHT_AREA, lights);
    assert!(out.contains(" is on") && !out.contains(" are "), "got {out:?}");
    let lichter = vec![ent("light.schlafzimmer", "Licht"), ent("light.nachttisch", "Nachttischlampe")];
    let out = say("de", "HassTurnOn", &[("area", "schlafzimmer"), ("domain", "light")], lichter);
    assert!(out.contains(" ist an") && !out.contains(" sind "), "got {out:?}");
}

#[test]
fn regional_german_packs_use_their_own_plural() {
    let lights = || vec![ent("light.tisch", "Tischlampe"), ent("light.regal", "Regallampe")];
    for (language, singular, plural) in [("de-AT", " is ", " san "), ("de_AT", " is ", " san "), ("de-CH", " isch ", " sind ")] {
        let out = say(language, "HassTurnOff", LIGHT_AREA, lights());
        assert!(out.contains(plural) && !out.contains(singular), "{language}: got {out:?}");
        let one = say(language, "HassTurnOff", LIGHT_AREA, vec![ent("light.tisch", "Tischlampe")]);
        assert!(one.contains(singular), "{language} single device keeps the singular: got {one:?}");
    }
}

#[test]
fn dutch_uses_the_plural_verb() {
    let lights = vec![ent("light.bureau", "Bureaulamp"), ent("light.kast", "Kastlamp")];
    let out = say("nl", "HassTurnOff", LIGHT_AREA, lights);
    assert!(out.contains(" staan uit") && !out.contains(" staat "), "got {out:?}");
}
