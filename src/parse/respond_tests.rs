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
fn vacuum_start_does_not_repeat_vacuum() {
    let _en = bind(&["en".into()]);
    let speech = speak(&[Intent::new("HassVacuumStart")], Personality::Default, false, None);
    assert_eq!(speech, "The vacuum has started cleaning.");
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

#[test]
fn list_add_speech_repeats_the_item() {
    let intent = Intent::new("HassListAddItem").with("name", "shopping_list").with("item", "painkillers");
    let _en = bind(&["en".into()]);
    assert_eq!(speak(std::slice::from_ref(&intent), Personality::Default, false, None), "Added painkillers to the list.");
    drop(_en);
    let _de = bind(&["de".into()]);
    let milch = Intent::new("HassShoppingListAddItem").with("item", "milch");
    assert_eq!(speak(&[milch], Personality::Default, false, None), "Milch auf die Liste gesetzt.");
    assert_eq!(speak(&[Intent::new("HassListAddItem")], Personality::Default, false, None), "Auf die Liste gesetzt.");
}

#[test]
fn list_complete_and_remove_speech_repeat_the_item() {
    let _en = bind(&["en".into()]);
    let item = |name: &str| Intent::new(name).with("name", "shopping_list").with("item", "painkillers");
    assert_eq!(speak(&[item("HassListCompleteItem")], Personality::Default, false, None), "Ticked painkillers off the list.");
    assert_eq!(speak(&[item("HassShoppingListCompleteItem")], Personality::Default, false, None), "Ticked painkillers off the list.");
    assert_eq!(speak(&[item("HassListRemoveItem")], Personality::Default, false, None), "Removed painkillers from the list.");
    drop(_en);
    let _de = bind(&["de".into()]);
    let milch = |name: &str| Intent::new(name).with("item", "milch");
    assert_eq!(speak(&[milch("HassListCompleteItem")], Personality::Default, false, None), "Milch abgehakt.");
    assert_eq!(speak(&[milch("HassListRemoveItem")], Personality::Default, false, None), "Milch von der Liste entfernt.");
}

#[test]
fn p27_domain_verbs_capitalization_position_and_step() {
    let _en = bind(&["en".into()]);
    let cover = Intent::new("HassTurnOn").with("entity_id", "cover.office_blinds").with("domain", "cover");
    assert_eq!(speak(std::slice::from_ref(&cover), Personality::Default, false, None), "Opened Office Blinds.");
    let lock = Intent::new("HassTurnOn").with("entity_id", "lock.front_door").with("domain", "lock");
    assert_eq!(speak(&[lock], Personality::Default, false, None), "Locked Front Door.");
    let locks = [
        Intent::new("HassTurnOn").with("entity_id", "lock.front_door").with("domain", "lock"),
        Intent::new("HassTurnOn").with("entity_id", "lock.back_door").with("domain", "lock"),
    ];
    assert_eq!(speak(&locks, Personality::Default, false, None), "Locked Front Door and Back Door.");
    let pos = Intent::new("HassSetPosition").with("entity_id", "cover.office_blinds").with("position", "50");
    assert_eq!(speak(&[pos], Personality::Default, false, None), "Office Blinds is at 50 percent.");
    let step = Intent::new("HassLightSet").with("entity_id", "light.desk_lamp").with("brightness_step", "up");
    let step_speech = speak(&[step], Personality::Default, false, None);
    assert!(step_speech.ends_with("is brighter."), "{step_speech}");
    assert!(!step_speech.contains("is at  percent"), "{step_speech}");
    let area = Intent::new("HassTurnOn").with("area", "kitchen").with("domain", "light");
    assert_eq!(speak(&[area], Personality::Default, false, None), "The light in the Kitchen is on.");
    drop(_en);
    let _de = bind(&["de".into()]);
    let rollo = Intent::new("HassTurnOn").with("entity_id", "cover.rollo").with("domain", "cover");
    assert_eq!(speak(&[rollo], Personality::Default, false, None), "Rollo ist geöffnet.");
    let pos_de = Intent::new("HassSetPosition").with("entity_id", "cover.rollo").with("position", "30");
    assert_eq!(speak(&[pos_de], Personality::Default, false, None), "Rollo auf 30 Prozent.");
}

#[test]
fn p27_clarify_same_name_uses_floor_or_room() {
    use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph};
    let home = HomeGraph {
        floors: vec![
            FloorRec { floor_id: "ground".into(), name: "Ground Floor".into(), aliases: vec![], level: Some(0) },
            FloorRec { floor_id: "upper".into(), name: "Upstairs".into(), aliases: vec![], level: Some(1) },
        ],
        areas: vec![
            AreaRec { area_id: "bath_ground".into(), name: "Bathroom".into(), aliases: vec![], floor_id: Some("ground".into()) },
            AreaRec { area_id: "bath_upper".into(), name: "Bathroom".into(), aliases: vec![], floor_id: Some("upper".into()) },
        ],
        entities: vec![
            EntityRec {
                entity_id: "light.ceiling_kitchen".into(),
                name: "Ceiling Light".into(),
                domain: "light".into(),
                platform: None,
                area: Some("kitchen".into()),
                aliases: vec![],
                tags: vec![],
            },
            EntityRec {
                entity_id: "light.ceiling_living".into(),
                name: "Ceiling Light".into(),
                domain: "light".into(),
                platform: None,
                area: Some("living_room".into()),
                aliases: vec![],
                tags: vec![],
            },
        ],
        ..HomeGraph::default()
    };
    let _en = bind(&["en".into()]);
    let bathrooms = speak_clarify(&["bath_ground".into(), "bath_upper".into()], Some(&home));
    assert!(bathrooms.contains("Bathroom (Ground Floor)"), "{bathrooms}");
    assert!(bathrooms.contains("Bathroom (Upstairs)"), "{bathrooms}");
    assert!(!bathrooms.contains("Bathroom or Bathroom?"), "{bathrooms}");
    let lights = speak_clarify(&["light.ceiling_kitchen".into(), "light.ceiling_living".into()], Some(&home));
    assert!(lights.contains("Ceiling Light (Kitchen)"), "{lights}");
    assert!(lights.contains("Ceiling Light (Living room)"), "{lights}");
}
