use super::*;
use crate::home::default_home;

#[test]
fn german_rooms_get_english_aliases() {
    let office = merge_area_aliases("arbeitszimmer", "Arbeitszimmer", vec![]);
    assert!(office.iter().any(|a| a == "office"), "{office:?}");
    assert!(office.iter().any(|a| a == "study"), "{office:?}");
    let living = merge_area_aliases("wohnzimmer", "Wohnzimmer", vec![]);
    assert!(living.iter().any(|a| a == "living"), "{living:?}");
    let bed = merge_area_aliases("schlafzimmer", "Schlafzimmer", vec![]);
    assert!(bed.iter().any(|a| a == "bedroom"), "{bed:?}");
}

#[test]
fn hue_ids_inherit_room_from_entity_id() {
    let areas = default_home().areas;
    assert_eq!(infer_area("light.schlafzimmer", &areas).as_deref(), Some("schlafzimmer"));
    assert_eq!(infer_area("light.schlafzimmer_ambilight", &areas).as_deref(), Some("schlafzimmer"));
    assert_eq!(infer_area("light.hue_color_lamp_2", &areas), None);
}

fn raw(id: &str) -> RawEntity {
    RawEntity {
        entity_id: id.into(),
        name: None,
        original_name: None,
        has_entity_name: false,
        area_id: None,
        platform: None,
        aliases: vec![],
        aliases_v2: None,
        compat_name: None,
        labels: vec![],
        disabled_by: None,
        device_id: None,
    }
}

fn device(name: &str) -> DeviceInfo {
    DeviceInfo { name: Some(name.into()), area_id: None, via_device_id: None }
}

fn shown(entity: &RawEntity, device: Option<&DeviceInfo>) -> String {
    entity_display_name(entity, device, false, &[])
}

#[test]
fn display_name_uses_device_when_entity_names_are_empty() {
    let mut entity = raw("climate.better_thermostat_wohnzimmer");
    entity.has_entity_name = true;
    assert_eq!(shown(&entity, Some(&device("Better Thermostat Wohnzimmer"))), "Better Thermostat Wohnzimmer");
}

#[test]
fn display_name_does_not_repeat_device_and_original() {
    let mut entity = raw("vacuum.r2d2");
    entity.has_entity_name = true;
    entity.original_name = Some(" R2D2".into());
    assert_eq!(shown(&entity, Some(&device("R2D2"))), "R2D2");
}

#[test]
fn display_name_joins_device_and_distinct_original() {
    let mut entity = raw("switch.better_thermostat_wohnzimmer_child_lock");
    entity.has_entity_name = true;
    entity.original_name = Some("Child Lock".into());
    assert_eq!(shown(&entity, Some(&device("Better Thermostat Wohnzimmer"))), "Better Thermostat Wohnzimmer Child Lock");
}

#[test]
fn display_name_prefers_user_name() {
    let mut entity = raw("light.wohnzimmer");
    entity.name = Some("Decke".into());
    entity.has_entity_name = true;
    assert_eq!(shown(&entity, Some(&device("Wohnzimmer Licht"))), "Decke");
}

#[test]
fn display_name_never_keeps_entity_id() {
    let mut entity = raw("light.wohnzimmer");
    entity.name = Some("light.wohnzimmer".into());
    entity.aliases = vec!["Wohnzimmer Licht".into()];
    assert_eq!(shown(&entity, None), "Wohnzimmer Licht");
    assert_eq!(shown(&raw("climate.better_thermostat_wohnzimmer"), None), "better thermostat wohnzimmer");
}

#[test]
fn keep_domain_includes_weather() {
    assert!(keep_domain("weather.openweathermap"));
    assert!(keep_domain("light.wohnzimmer"));
    assert!(!keep_domain("sensor.openweathermap_temperatur"));
}

#[test]
fn loads_floors_from_ha_storage() {
    let dir = std::env::temp_dir().join(format!("klar-floor-reg-{}", std::process::id()));
    let storage = dir.join(".storage");
    let _ = std::fs::create_dir_all(&storage);
    std::fs::write(
        storage.join("core.entity_registry"),
        r#"{"data":{"entities":[{"entity_id":"light.hallway","name":"Hall","area_id":"hallway"}]}}"#,
    )
    .unwrap();
    std::fs::write(storage.join("core.area_registry"), r#"{"data":{"areas":[{"id":"hallway","name":"Hallway","floor_id":"upper"}]}}"#)
        .unwrap();
    std::fs::write(storage.join("core.floor_registry"), r#"{"data":{"floors":[{"floor_id":"upper","name":"Upper Floor","level":1}]}}"#)
        .unwrap();
    let home = load_home(&dir, default_home());
    assert_eq!(home.floors.iter().map(|floor| floor.floor_id.as_str()).collect::<Vec<_>>(), ["upper"]);
    assert_eq!(home.areas.iter().find(|area| area.area_id == "hallway").and_then(|area| area.floor_id.as_deref()), Some("upper"));
    assert!(home.floors[0].aliases.iter().any(|alias| alias == "upstairs"));
}

#[test]
fn ha_2026_11_storage_composes_the_device_name_and_keeps_the_old_one() {
    let dir = std::env::temp_dir().join(format!("klar-ha-202611-{}", std::process::id()));
    let storage = dir.join(".storage");
    let _ = std::fs::create_dir_all(&storage);
    std::fs::write(
            storage.join("core.device_registry"),
            r#"{"data":{"devices":[{"id":"strip","name":"Power strip","area_id":"kitchen"},{"id":"outlet","name":"Outlet 1","via_device_id":"strip"}]}}"#,
        )
        .unwrap();
    std::fs::write(
            storage.join("core.entity_registry"),
            r#"{"minor_version":24,"data":{"entities":[{"entity_id":"switch.outlet_1","name":"Switch","original_name":"Switch","has_entity_name":true,"device_id":"outlet","aliases":["stale"],"aliases_v2":[null,"coffee machine"],"compat_name":"Coffee machine"}]}}"#,
        )
        .unwrap();
    let home = load_home(&dir, default_home());
    let entity = home.entities.iter().find(|item| item.entity_id == "switch.outlet_1").expect("entity");
    assert_eq!(entity.name, "Power strip Outlet 1 Switch");
    assert_eq!(entity.area.as_deref(), Some("kitchen"));
    assert!(entity.aliases.iter().any(|alias| alias == "coffee machine"), "{:?}", entity.aliases);
    assert!(entity.aliases.iter().any(|alias| alias == "Coffee machine"), "{:?}", entity.aliases);
    assert!(!entity.aliases.iter().any(|alias| alias == "stale"), "{:?}", entity.aliases);
    let _ = std::fs::remove_dir_all(&dir);
}
