//! Package 4.5: speech frames live in packs (L-20 / L-24).
//! Snapshot coverage per locale + non-Latin replies without engine ASCII words.

use crate::lang::LangId;
use crate::speech::render_snapshot;
use crate::types::{SpeechEntity, SpeechIntent, SpeechSlot, SpeechSnapshot, UnitSystem};
use std::collections::BTreeMap;

fn snap(language: &str, name: &str, slots: &[(&str, &str)], entities: Vec<SpeechEntity>) -> SpeechSnapshot {
    SpeechSnapshot {
        schema_version: "1".into(),
        language: language.into(),
        personality: "default".into(),
        unit_system: UnitSystem::Metric,
        now: "2026-11-26T06:00:00+01:00".into(),
        intent: SpeechIntent {
            name: name.into(),
            slots: slots.iter().map(|(n, v)| SpeechSlot { name: (*n).into(), value: (*v).into() }).collect(),
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

fn say(language: &str, intent: &str, slots: &[(&str, &str)], entities: Vec<SpeechEntity>) -> String {
    render_snapshot(&snap(language, intent, slots, entities)).speech
}

/// ASCII letter-runs of length ≥2 after named entities/areas are stripped.
fn ascii_words_outside_names(speech: &str, names: &[&str]) -> Vec<String> {
    let mut rest = speech.to_string();
    for name in names {
        if !name.is_empty() {
            rest = rest.replace(name, " ");
        }
    }
    let mut words = Vec::new();
    let mut cur = String::new();
    for ch in rest.chars() {
        if ch.is_ascii_alphabetic() {
            cur.push(ch);
        } else if !cur.is_empty() {
            if cur.len() >= 2 {
                words.push(cur.clone());
            }
            cur.clear();
        }
    }
    if cur.len() >= 2 {
        words.push(cur);
    }
    words
}

#[test]
fn l20_fr_query_does_not_use_english_is() {
    let speech =
        say("fr", "HassGetState", &[("entity_id", "light.living")], vec![entity("light.living", "Lumière du salon", "light", "on")]);
    assert!(!speech.to_ascii_lowercase().contains(" is "), "got {speech:?}");
    assert!(!speech.contains("ist "), "got {speech:?}");
    assert!(speech.contains("Lumière du salon") || speech.contains("allum"), "got {speech:?}");
}

#[test]
fn l20_fr_climate_uses_pack_frame_not_english_is_at() {
    let speech = say(
        "fr",
        "HassClimateSetTemperature",
        &[("entity_id", "climate.living"), ("name", "Chauffage"), ("temperature", "21")],
        vec![entity("climate.living", "Chauffage", "climate", "heat")],
    );
    assert!(!speech.to_ascii_lowercase().contains(" is at "), "got {speech:?}");
    assert!(!speech.contains("Chauffage Chauffage"), "doubled noun: {speech:?}");
}

#[test]
fn l20_cover_uses_cover_frame_not_on_off() {
    let fr = say(
        "fr",
        "HassTurnOn",
        &[("entity_id", "cover.bedroom"), ("name", "Volets de la chambre"), ("domain", "cover")],
        vec![entity("cover.bedroom", "Volets de la chambre", "cover", "open")],
    );
    assert!(!fr.contains("allume"), "cover should not use turn_on: {fr:?}");
    let de = say(
        "de",
        "HassTurnOff",
        &[("entity_id", "cover.bedroom"), ("name", "Rollladen"), ("domain", "cover")],
        vec![entity("cover.bedroom", "Rollladen", "cover", "closed")],
    );
    assert!(!de.contains("ausgeschaltet") && !de.contains("ist aus"), "cover close: {de:?}");
    assert!(de.contains("geschlossen") || de.contains("Rollladen"), "{de:?}");
}

#[test]
fn l20_device_names_keep_source_casing() {
    let speech = say(
        "fr",
        "HassTurnOn",
        &[("entity_id", "light.living"), ("name", "Lumière du salon")],
        vec![entity("light.living", "Lumière du salon", "light", "on")],
    );
    assert!(!speech.contains("Lumière Du Salon"), "title-cased: {speech:?}");
    assert!(speech.contains("Lumière du salon"), "got {speech:?}");
}

#[test]
fn l20_fr_media_pause_not_english() {
    let speech = say(
        "fr",
        "HassMediaPause",
        &[("entity_id", "media_player.tv"), ("name", "Télévision")],
        vec![entity("media_player.tv", "Télévision", "media_player", "paused")],
    );
    assert!(!speech.to_ascii_lowercase().contains(" is paused"), "got {speech:?}");
}

#[test]
fn l24_non_latin_replies_have_no_engine_ascii_words() {
    // Friendly names stay Latin-free; any leftover ASCII word is an engine/pack frame leak.
    let cases: &[(&str, &str, &str, &str)] = &[
        ("ja", "リビングの照明", "HassTurnOn", "on"),
        ("ja", "リビングの照明", "HassGetState", "on"),
        ("zh-CN", "客厅灯", "HassTurnOn", "on"),
        ("ar", "ضوء الصالة", "HassTurnOn", "on"),
        ("ar", "ضوء الصالة", "HassGetState", "off"),
        ("he", "אור הסלון", "HassTurnOn", "on"),
        ("fa", "چراغ پذیرایی", "HassTurnOff", "off"),
        ("hi", "लिविंग लाइट", "HassTurnOn", "on"),
        ("ko", "거실 조명", "HassTurnOn", "on"),
        ("th", "ไฟห้องนั่งเล่น", "HassTurnOn", "on"),
        ("uk", "світло вітальні", "HassTurnOn", "on"),
        ("el", "φως σαλονιού", "HassGetState", "on"),
    ];
    for &(lang, name, intent, state) in cases {
        let speech =
            say(lang, intent, &[("entity_id", "light.living"), ("name", name)], vec![entity("light.living", name, "light", state)]);
        let leaks = ascii_words_outside_names(&speech, &[name]);
        assert!(leaks.is_empty(), "{lang} {intent}: ascii leaks {leaks:?} in {speech:?}");
    }
}

#[test]
fn speech_frame_snapshots_per_locale() {
    // One TurnOn line per compiled locale — packs own the frame.
    for id in LangId::all() {
        let meta = id.meta().expect("meta");
        let code = meta.code;
        let speech = id.pack().speech;
        // Pack light noun already names a light, so spoken_device will not append light_suffix.
        let name = speech.light_noun;
        let line =
            say(code, "HassTurnOn", &[("entity_id", "light.living"), ("name", name)], vec![entity("light.living", name, "light", "on")]);
        assert!(!line.is_empty(), "{code} empty speech");
        let filled = speech.turn_on.replace("{target}", name);
        let expected = {
            let mut chars = filled.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => filled,
            }
        };
        assert_eq!(line, expected, "{code} snapshot");
        if meta.script != "Latn" {
            let leaks = ascii_words_outside_names(&line, &[name]);
            assert!(leaks.is_empty(), "{code} non-Latin snapshot leaks {leaks:?}: {line:?}");
        }
    }
}

#[test]
fn empty_place_comes_from_pack() {
    let ja = say("ja", "HassGetState", &[("area", "wohnzimmer"), ("domain", "light")], vec![]);
    assert_eq!(ja, LangId::from_code("ja").unwrap().pack().speech.empty_place);
    let en = say("en", "HassGetState", &[("area", "living"), ("domain", "light")], vec![]);
    assert_eq!(en, "No devices.");
}
