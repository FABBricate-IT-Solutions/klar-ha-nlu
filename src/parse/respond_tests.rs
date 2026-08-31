use super::*;
use crate::lang::bind;
use crate::types::Intent;

#[test]
fn speech_follows_pinned_pack() {
    let intent = Intent::new("HassTurnOn").with("area", "wohnzimmer");
    let _de = bind(&["de".into()]);
    let de = speak(std::slice::from_ref(&intent), Personality::Default, false, None);
    assert!(de.contains("ist an") || de.contains("Licht"), "{de}");
    drop(_de);
    let _en = bind(&["en".into()]);
    let en = speak(&[intent], Personality::Default, false, None);
    assert!(en.contains("is on") || en.contains("light"), "{en}");
    assert!(!en.contains("ist an"), "{en}");
}

#[test]
fn speech_compounds_room_light() {
    let _de = bind(&["de".into()]);
    let intent = Intent::new("HassTurnOn").with("entity_id", "light.schlafzimmer_licht");
    let de = speak(&[intent], Personality::Default, false, None);
    assert_eq!(de, "Schlafzimmerlicht ist an.");
    assert!(!de.contains("light."));
    let kugel = Intent::new("HassTurnOn").with("entity_id", "light.schlafzimmer");
    assert_eq!(speak(&[kugel], Personality::Default, false, None), "Schlafzimmerlicht ist an.");
    let butler = speak(&[Intent::new("HassTurnOn").with("entity_id", "light.schlafzimmer")], Personality::Butler, false, None);
    let body = "Schlafzimmerlicht ist an.";
    let prefixes = crate::lang::catalog().speech().personality_prefixes("butler");
    assert!(!prefixes.is_empty(), "butler must have spoken variants");
    assert!(prefixes.iter().any(|prefix| butler == format!("{prefix}{body}")), "{butler} not in {prefixes:?}");
}

#[test]
fn clarify_uses_friendly_name() {
    let _de = bind(&["de".into()]);
    let home = crate::home::default_home();
    let speech = speak_clarify(&["light.schlafzimmer_kugel".into()], Some(&home));
    assert!(speech.contains("Kugel"), "{speech}");
    assert!(!speech.contains("schlafzimmer"), "{speech}");
    let raw = speak_clarify(&["light.schlafzimmer".into()], None);
    assert!(raw.contains("Schlafzimmer"), "{raw}");
    assert!(!raw.contains("light."), "{raw}");
}

#[test]
fn vacuum_speech_uses_device_name() {
    let _de = bind(&["de".into()]);
    let mut home = crate::home::default_home();
    if let Some(ent) = home.entities.iter_mut().find(|e| e.entity_id == "vacuum.r2d2") {
        ent.name = "Saugroboter".into();
    }
    let intent = Intent::new("HassVacuumStart").with("entity_id", "vacuum.r2d2");
    let speech = speak(&[intent], Personality::Default, false, Some(&home));
    assert!(speech.contains("Saugroboter"), "{speech}");
    assert!(!speech.contains("R2D2"), "{speech}");
}

#[test]
fn climate_speech_does_not_repeat_heizung() {
    let home = crate::home::default_home();
    let intent =
        Intent::new("HassClimateSetTemperature").with("entity_id", "climate.better_thermostat_wohnzimmer").with("temperature", "21");
    let _de = bind(&["de".into()]);
    let de = speak(std::slice::from_ref(&intent), Personality::Default, false, Some(&home));
    assert_eq!(de, "Heizung Wohnzimmer auf 21 Grad.");
    assert_eq!(de.matches("Heizung").count(), 1, "{de}");
    drop(_de);
    let _en = bind(&["en".into()]);
    let en = speak(&[intent], Personality::Default, false, Some(&home));
    assert_eq!(en, "Heizung Wohnzimmer is at 21 degrees.");
    assert!(!en.contains("Heat Heizung"), "{en}");
}

#[test]
fn light_set_speaks_color_not_percent() {
    let _de = bind(&["de".into()]);
    let intent = Intent::new("HassLightSet").with("area", "schlafzimmer").with("domain", "light").with("color", "red");
    let de = speak(std::slice::from_ref(&intent), Personality::Default, false, None);
    assert!(de.to_lowercase().contains("rot"), "{de}");
    assert!(!de.contains('?'), "{de}");
    assert!(!de.to_lowercase().contains("prozent"), "{de}");
}

#[test]
fn climate_speech_adds_noun_when_target_is_room_only() {
    let _de = bind(&["de".into()]);
    let intent = Intent::new("HassClimateSetTemperature").with("area", "wohnzimmer").with("temperature", "21");
    assert_eq!(speak(&[intent], Personality::Default, false, None), "Heizung Wohnzimmer auf 21 Grad.");
}

#[test]
fn climate_speech_humanizes_entity_id_names() {
    let mut home = crate::home::default_home();
    if let Some(ent) = home.entities.iter_mut().find(|e| e.entity_id == "climate.better_thermostat_wohnzimmer") {
        ent.name = "climate.better_thermostat_wohnzimmer".into();
    }
    let intent =
        Intent::new("HassClimateSetTemperature").with("entity_id", "climate.better_thermostat_wohnzimmer").with("temperature", "21");
    let _de = bind(&["de".into()]);
    let speech = speak(std::slice::from_ref(&intent), Personality::Default, false, Some(&home));
    assert_eq!(speech, "Better Thermostat Wohnzimmer auf 21 Grad.");
    assert!(!speech.contains("climate."), "{speech}");
}

#[test]
fn kitchen_status_speaks_umlaut_not_slug() {
    let _de = bind(&["de".into()]);
    let home = crate::home::default_home();
    let intent = Intent::new("HassGetState").with("area", "kuche");
    let speech = speak(std::slice::from_ref(&intent), Personality::Default, false, Some(&home));
    assert!(speech.contains("Küche"), "{speech}");
    assert!(!speech.contains("Kuche"), "{speech}");
}

#[test]
fn kitchen_followup_names_the_room() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datasets/wohnung_mittel/home_config.yaml");
    let home = crate::home::load_home_config(&path).expect("wohnung home");
    let intent = Intent::new("HassTurnOn").with("entity_id", "light.kuche_kuche");
    let _de = bind(&["de".into()]);
    let de = speak(std::slice::from_ref(&intent), Personality::Default, false, Some(&home));
    assert!(de.to_lowercase().contains("küche"), "{de}");
    drop(_de);
    let _en = bind(&["en".into()]);
    let en = speak(&[intent], Personality::Default, false, Some(&home));
    assert!(en.to_lowercase().contains("kitchen") || en.to_lowercase().contains("küche"), "{en}");
}

#[test]
fn empty_bind_clarify_and_confirm_are_english() {
    let _bind = bind(&[]);
    let speech = speak_clarify(&["Kitchen Light".into(), "Hall Light".into()], None);
    assert!(speech.contains("Do you mean"), "{speech}");
    assert!(speech.contains(" or "), "{speech}");
    assert!(!speech.contains("Meinst du"), "{speech}");
    assert!(!speech.contains(" oder "), "{speech}");
    assert_eq!(crate::lang::catalog().speech().confirm, "Should I really do that?");
}

#[test]
fn clarify_and_confirm_follow_pinned_locale() {
    let names = ["Kitchen Light".into(), "Hall Light".into()];
    let cases = [
        ("de", "Meinst du", " oder ", "Soll ich das wirklich ausführen?"),
        ("en", "Do you mean", " or ", "Should I really do that?"),
        ("fr", "Tu veux dire", " ou ", "Je dois vraiment le faire?"),
        ("ja", "Kitchen Light", "か", "本当に実行しますか？"),
    ];
    for (code, clarify, join, confirm) in cases {
        let _bind = bind(&[code.into()]);
        let speech = speak_clarify(&names, None);
        assert!(speech.contains(clarify), "{code}: {speech}");
        assert!(speech.contains(join), "{code}: {speech}");
        if code != "de" {
            assert!(!speech.contains("Meinst du"), "{code}: {speech}");
        }
        assert_eq!(crate::lang::catalog().speech().confirm, confirm, "{code}");
    }
}
