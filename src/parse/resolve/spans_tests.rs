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
