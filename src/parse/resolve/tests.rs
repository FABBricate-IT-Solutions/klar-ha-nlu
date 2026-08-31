use super::*;
use crate::lang::bind;
use crate::types::{AreaRec, EntityRec, HomeGraph};

fn lamp(id: &str, name: &str, area: &str) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: "light".into(),
        platform: None,
        area: Some(area.into()),
        aliases: vec![name.to_ascii_lowercase()],
        tags: Vec::new(),
    }
}

#[test]
fn resolve_skips_entities_not_exposed_to_assist() {
    let mut home = HomeGraph {
        areas: vec![AreaRec { area_id: "kuche".into(), name: "Küche".into(), aliases: vec!["kueche".into()], floor_id: None }],
        entities: vec![lamp("light.hidden", "Geheimlampe", "kuche"), lamp("light.kuche_kuche", "Deckenlampe", "kuche")],
        assist: Some(["light.kuche_kuche".into()].into()),
        ..Default::default()
    };
    let hit = resolve(&["deckenlampe".into()], &home, Some("light"));
    assert_eq!(hit.entities.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["light.kuche_kuche"]);
    home.assist = Some(["light.hidden".into()].into());
    let hidden_only = resolve(&["geheimlampe".into()], &home, Some("light"));
    assert_eq!(hidden_only.entities.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["light.hidden"]);
    home.assist = Some(["light.kuche_kuche".into()].into());
    assert_eq!(crate::parse::compound::room_light_id(&home, "kuche"), None);
}

#[test]
fn resolve_skips_nlu_ignored_entities() {
    let mut ignored = lamp("switch.create_calendar_event", "Create Calendar Event", "wohnung");
    ignored.domain = "switch".into();
    ignored.tags = vec!["nlu_ignore".into()];
    let home = HomeGraph {
        areas: vec![AreaRec { area_id: "wohnung".into(), name: "Wohnung".into(), aliases: Vec::new(), floor_id: None }],
        entities: vec![ignored, lamp("light.wohnzimmer", "Wohnzimmer Licht", "wohnung")],
        ..Default::default()
    };
    let hit = resolve(&["calendar".into(), "event".into()], &home, None);
    assert!(!hit.entities.iter().any(|entity| entity.entity_id == "switch.create_calendar_event"), "{hit:?}");
}

#[test]
fn french_plafond_picks_decke_not_room_light() {
    let _bind = bind(&["fr".into()]);
    let home = HomeGraph {
        areas: vec![AreaRec {
            area_id: "schlafzimmer".into(),
            name: "Schlafzimmer".into(),
            aliases: vec!["chambre".into(), "schlafzimmer".into()],
            floor_id: None,
        }],
        entities: vec![
            EntityRec {
                entity_id: "light.schlafzimmer_licht".into(),
                name: "Schlafzimmer Licht".into(),
                domain: "light".into(),
                platform: None,
                area: Some("schlafzimmer".into()),
                aliases: vec!["licht".into()],
                tags: Vec::new(),
            },
            EntityRec {
                entity_id: "light.schlafzimmer_decke".into(),
                name: "Schlafzimmer Decke".into(),
                domain: "light".into(),
                platform: None,
                area: Some("schlafzimmer".into()),
                aliases: vec!["decke".into(), "deckenlampe".into()],
                tags: Vec::new(),
            },
        ],
        ..HomeGraph::default()
    };
    let hit = resolve(&["plafond".into(), "chambre".into()], &home, Some("light"));
    assert_eq!(hit.entities.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["light.schlafzimmer_decke"], "{hit:?}");
}

#[test]
fn french_laundry_device_hints_switch() {
    let _bind = bind(&["fr".into()]);
    assert_eq!(domain_hint(&["allumer".into(), "appareil".into(), "buanderie".into()]), Some("switch"));
}
