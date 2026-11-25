use super::{link_named_spans, take_linked, timer_noun_free};
use crate::types::{AreaRec, EntityRec, HomeGraph};

fn home() -> HomeGraph {
    HomeGraph {
        areas: vec![AreaRec { area_id: "bedroom".into(), name: "Bedroom".into(), aliases: vec![], floor_id: None }],
        entities: vec![EntityRec {
            entity_id: "light.timer_lamp".into(),
            name: "Timer Lamp".into(),
            domain: "light".into(),
            platform: None,
            area: Some("bedroom".into()),
            aliases: vec![],
            tags: vec![],
        }],
        assist: Some(["light.timer_lamp".into()].into_iter().collect()),
        ..HomeGraph::default()
    }
}

#[test]
fn timer_lamp_span_consumes_timer_word() {
    let _lang = crate::lang::bind(&["en".into()]);
    let tokens: Vec<String> = ["turn", "on", "the", "timer", "lamp"].into_iter().map(str::to_string).collect();
    let linked = link_named_spans(&tokens, &home(), &[]);
    assert_eq!(linked.entities.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["light.timer_lamp"]);
    assert!(!timer_noun_free(&tokens, &linked));
}

#[test]
fn obergeschoss_does_not_link_the_upstairs_heater() {
    let _lang = crate::lang::bind(&["de".into()]);
    let home = crate::home::load_home_config(std::path::Path::new("tests/datasets/familienhaus_de/home_config.yaml")).expect("home");
    let tokens: Vec<String> = ["licht", "im", "obergeschoss", "an"].into_iter().map(str::to_string).collect();
    let linked = link_named_spans(&tokens, &home, &[]);
    assert!(
        !linked.entities.iter().any(|e| e.entity_id == "climate.upper_thermostat"),
        "floor alias on a heater must not link: {:?}",
        linked.entities.iter().map(|e| &e.entity_id).collect::<Vec<_>>()
    );
    let resolved = crate::parse::resolve::resolve(&tokens, &home, Some("light"));
    assert!(resolved.floors.iter().any(|f| f == "upper"), "{resolved:?}");
    assert!(resolved.entities.iter().all(|e| e.domain == "light") || !resolved.areas.is_empty() || !resolved.floors.is_empty());
}

#[test]
fn blinds_and_curtains_keep_the_room() {
    let _lang = crate::lang::bind(&["en".into()]);
    let home = HomeGraph {
        floors: vec![],
        areas: vec![AreaRec { area_id: "bedroom".into(), name: "Bedroom".into(), aliases: vec![], floor_id: None }],
        entities: vec![
            EntityRec {
                entity_id: "cover.bedroom_blinds".into(),
                name: "Bedroom Blinds".into(),
                domain: "cover".into(),
                platform: None,
                area: Some("bedroom".into()),
                aliases: vec![],
                tags: vec![],
            },
            EntityRec {
                entity_id: "cover.bedroom_curtains".into(),
                name: "Bedroom Curtains".into(),
                domain: "cover".into(),
                platform: None,
                area: Some("bedroom".into()),
                aliases: vec![],
                tags: vec![],
            },
        ],
        assist: Some(["cover.bedroom_blinds".into(), "cover.bedroom_curtains".into()].into_iter().collect()),
        ..HomeGraph::default()
    };
    let tokens: Vec<String> = ["close", "the", "bedroom", "blinds", "and", "curtains"].into_iter().map(str::to_string).collect();
    let taken = take_linked(&tokens, &home, &[], &["bedroom".into()], Some("cover")).expect("linked");
    assert!(taken.entities.is_empty(), "must not pin only blinds: {:?}", taken.entities);
    assert_eq!(taken.areas, ["bedroom"]);
}

#[test]
fn everything_except_oven_is_not_an_exclusive_oven_link() {
    let _lang = crate::lang::bind(&["en".into()]);
    let home = crate::types::HomeGraph {
        areas: vec![crate::types::AreaRec { area_id: "kitchen".into(), name: "Kitchen".into(), aliases: vec![], floor_id: None }],
        entities: vec![
            crate::types::EntityRec {
                entity_id: "light.kitchen_ceiling".into(),
                name: "Ceiling Light".into(),
                domain: "light".into(),
                platform: None,
                area: Some("kitchen".into()),
                aliases: vec![],
                tags: vec![],
            },
            crate::types::EntityRec {
                entity_id: "switch.oven".into(),
                name: "Oven".into(),
                domain: "switch".into(),
                platform: None,
                area: Some("kitchen".into()),
                aliases: vec![],
                tags: vec![],
            },
        ],
        assist: Some(["light.kitchen_ceiling".into(), "switch.oven".into()].into_iter().collect()),
        ..crate::types::HomeGraph::default()
    };
    let tokens: Vec<String> =
        ["turn", "on", "everything", "in", "the", "kitchen", "except", "the", "oven"].into_iter().map(str::to_string).collect();
    // "Oven" sits after except — must not become the exclusive target.
    assert!(take_linked(&tokens, &home, &[], &["kitchen".into()], None).is_none());
}

#[test]
fn wohn_und_esszimmer_label_is_not_exclusive_when_lights_asked() {
    let _lang = crate::lang::bind(&["de".into()]);
    let home = crate::home::default_home();
    let tokens = crate::parse::normalize::tokenize("Wohn und Esszimmer lichte aus");
    let resolved = crate::parse::resolve::resolve(&tokens, &home, Some("light"));
    assert!(resolved.areas.iter().any(|a| a == "wohnzimmer"), "{resolved:?}");
    assert!(resolved.areas.iter().any(|a| a == "esszimmer"), "{resolved:?}");
    assert!(resolved.entities.is_empty(), "must not pin one room light: {resolved:?}");
}

#[test]
fn group_status_links_wohn_und_esszimmer_device() {
    let _lang = crate::lang::bind(&["de".into()]);
    let live: crate::types::HomeGraph = serde_json::from_str(include_str!("../../../tests/fixtures/wohnung_live.json")).expect("live");
    let tokens = crate::parse::normalize::tokenize("Wie ist der Status von Wohn und Esszimmer");
    let resolved = crate::parse::resolve::resolve(&tokens, &live, None);
    assert!(resolved.entities.iter().any(|e| e.entity_id == "light.wohn_und_esszimmer"), "{resolved:?}");
}

#[test]
fn garagentor_stays_whole_in_compound_expand() {
    let _lang = crate::lang::bind(&["de".into()]);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datasets/full_home/de/home_config.yaml");
    let home = crate::home::load_home_config(&path).expect("de home");
    let tokens = crate::parse::normalize::tokenize("schliess haustuer und garagentor");
    let split = crate::parse::compound::expand_compounds(&tokens, &home);
    assert!(split.tokens.iter().any(|t| t == "garagentor"), "{:?}", split.tokens);
    assert!(!split.tokens.iter().any(|t| t == "tor"), "must not split garagentor: {:?}", split.tokens);
    let resolved = crate::parse::resolve::resolve(&split.tokens, &home, Some("lock"));
    assert!(resolved.entities.iter().any(|e| e.entity_id == "lock.front_door"), "{resolved:?}");
    assert!(resolved.entities.iter().any(|e| e.entity_id == "lock.garage_entry"), "{resolved:?}");
}

#[test]
fn room_scopes_shared_ceiling_word_to_one_fixture() {
    let _lang = crate::lang::bind(&["af".into()]);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datasets/full_home/af/home_config.yaml");
    let home = crate::home::load_home_config(&path).expect("af home");
    let tokens = crate::parse::normalize::tokenize("skakel plafon sitkamer");
    let resolved = crate::parse::resolve::resolve(&tokens, &home, Some("light"));
    assert_eq!(resolved.areas, ["living"], "{resolved:?}");
    assert!(resolved.ambiguous.is_empty(), "{resolved:?}");
    assert_eq!(resolved.entities.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["light.living_ceiling"], "{resolved:?}");
}

#[test]
fn group_status_survives_when_und_is_not_a_local_conjunction() {
    // Pidgin parity keeps the German group label under packs where "und" is not a conj.
    let live: crate::types::HomeGraph = serde_json::from_str(include_str!("../../../tests/fixtures/wohnung_live.json")).expect("live");
    for lang in ["de", "af", "fr", "en"] {
        let _bind = crate::lang::bind(&[lang.into()]);
        let tokens = crate::parse::normalize::tokenize("status wohn und esszimmer");
        let resolved = crate::parse::resolve::resolve(&tokens, &live, None);
        assert!(resolved.entities.iter().any(|e| e.entity_id == "light.wohn_und_esszimmer"), "{lang}: {resolved:?}");
        assert!(resolved.areas.is_empty(), "{lang}: multi_area must not steal status: {resolved:?}");
    }
}
