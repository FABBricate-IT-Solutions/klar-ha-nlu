use super::*;

fn snapshot() -> HomeSnapshot {
    HomeSnapshot {
        schema_version: HOME_SCHEMA_VERSION.into(),
        entities: vec![SnapshotEntity {
            entity_id: "light.living".into(),
            name: Some("Living".into()),
            original_name: None,
            has_entity_name: false,
            area_id: Some("living".into()),
            device_id: None,
            platform: Some("hue".into()),
            aliases: vec!["decke".into()],
            labels: vec!["Licht".into()],
            disabled: false,
            device_class: None,
        }],
        devices: Vec::new(),
        areas: vec![SnapshotArea {
            id: "living".into(),
            name: "Wohnzimmer".into(),
            aliases: vec!["wohnzimmer".into()],
            floor_id: Some("upper".into()),
        }],
        floors: vec![SnapshotFloor { floor_id: "upper".into(), name: "Upper Floor".into(), aliases: Vec::new(), level: Some(1) }],
        labels: vec![SnapshotLabel { label_id: "lbl_1".into(), name: "Licht".into() }],
        assist: Some(vec!["light.living".into()]),
        registered_intents: Vec::new(),
    }
}

#[test]
fn rejects_unknown_snapshot_fields() {
    let err = serde_json::from_value::<HomeSnapshot>(serde_json::json!({
        "schema_version": HOME_SCHEMA_VERSION,
        "entities": [],
        "extra": true
    }))
    .expect_err("unknown field");
    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn ingest_keeps_floors_and_assist() {
    let home = ingest(snapshot()).expect("snapshot");
    assert_eq!(home.floors.len(), 1);
    assert_eq!(home.floors[0].floor_id, "upper");
    assert!(home.floors[0].aliases.iter().any(|alias| alias == "upstairs"));
    assert_eq!(home.areas[0].floor_id.as_deref(), Some("upper"));
    assert_eq!(home.entities[0].entity_id, "light.living");
    assert_eq!(home.assist.as_ref().map(HashSet::len), Some(1));
}

#[test]
fn ingest_keeps_the_device_class() {
    let mut raw = snapshot();
    raw.entities[0].device_class = Some(" Garage ".into());
    assert_eq!(ingest(raw).expect("snapshot").device_class("light.living"), Some("garage"));
}

#[test]
fn rejects_bad_schema_and_caps_filtered_entities() {
    let mut bad = snapshot();
    bad.schema_version = "9".into();
    assert_eq!(ingest(bad).unwrap_err(), SnapshotError::Schema);
    let mut huge = snapshot();
    // Sensors would once trip the raw 4096 cap before keep_domain filtering (R-10).
    huge.entities = (0..MAX_SNAPSHOT_ENTITIES + 500)
        .map(|index| SnapshotEntity {
            entity_id: format!("sensor.meter_{index}"),
            name: Some("Meter".into()),
            original_name: None,
            has_entity_name: false,
            area_id: None,
            device_id: None,
            platform: None,
            aliases: Vec::new(),
            labels: Vec::new(),
            disabled: false,
            device_class: None,
        })
        .chain(std::iter::once(SnapshotEntity {
            entity_id: "light.kept".into(),
            name: Some("Kept".into()),
            original_name: None,
            has_entity_name: false,
            area_id: None,
            device_id: None,
            platform: None,
            aliases: vec!["x".repeat(MAX_ALIAS_CHARS + 1)],
            labels: Vec::new(),
            disabled: false,
            device_class: None,
        }))
        .collect();
    let home = ingest(huge).expect("filter then cap");
    assert_eq!(home.entities.len(), 1);
    assert_eq!(home.entities[0].aliases[0].chars().count(), MAX_ALIAS_CHARS);
    let mut over = snapshot();
    over.entities = (0..=MAX_SNAPSHOT_ENTITIES)
        .map(|index| SnapshotEntity {
            entity_id: format!("light.lamp{index}"),
            name: Some("Lamp".into()),
            original_name: None,
            has_entity_name: false,
            area_id: None,
            device_id: None,
            platform: None,
            aliases: Vec::new(),
            labels: Vec::new(),
            disabled: false,
            device_class: None,
        })
        .collect();
    assert_eq!(ingest(over).expect("cap").entities.len(), MAX_SNAPSHOT_ENTITIES);
}

#[test]
fn rejects_malformed_ids_without_panic() {
    let mut bad = snapshot();
    bad.entities[0].entity_id = "../etc/passwd".into();
    assert_eq!(ingest(bad).unwrap_err(), SnapshotError::Malformed("invalid entity_id"));
    let mut missing = snapshot();
    missing.floors[0].floor_id.clear();
    assert_eq!(ingest(missing).unwrap_err(), SnapshotError::Malformed("invalid id"));
}

#[test]
fn drops_unknown_floor_links_and_disabled_entities() {
    let mut raw = snapshot();
    raw.areas[0].floor_id = Some("ghost".into());
    raw.entities.push(SnapshotEntity {
        entity_id: "light.hidden".into(),
        name: Some("Hidden".into()),
        original_name: None,
        has_entity_name: false,
        area_id: Some("living".into()),
        device_id: None,
        platform: None,
        aliases: Vec::new(),
        labels: Vec::new(),
        disabled: true,
        device_class: None,
    });
    let home = ingest(raw).expect("snapshot");
    assert!(home.areas[0].floor_id.is_none());
    assert_eq!(home.entities.len(), 1);
}

#[test]
fn rejects_duplicate_ids_and_empty_or_missing_assist_frees_nothing() {
    let mut dup = snapshot();
    dup.entities.push(dup.entities[0].clone());
    assert_eq!(ingest(dup).unwrap_err(), SnapshotError::Malformed("duplicate entity_id"));

    let mut empty = snapshot();
    empty.entities.clear();
    assert_eq!(ingest(empty).unwrap_err(), SnapshotError::Malformed("empty home snapshot"));

    let mut no_assist = snapshot();
    no_assist.assist = None;
    let home = ingest(no_assist).expect("missing assist becomes explicit set");
    // Omitted assist → every kept entity is exposed, but the set is Some (not None), so later
    // additions are not silently freed.
    assert_eq!(home.assist.as_ref().map(HashSet::len), Some(1));
    assert!(crate::home::expose::assist_visible(&home.entities[0], &home));
    assert!(home.assist.is_some());

    let mut empty_assist = snapshot();
    empty_assist.assist = Some(Vec::new());
    let locked = ingest(empty_assist).expect("explicit empty assist");
    assert_eq!(locked.assist.as_ref().map(HashSet::len), Some(0));
    assert!(!crate::home::expose::assist_visible(&locked.entities[0], &locked));
}

#[test]
fn ingest_keeps_weather_and_assist_only_domains() {
    let mut raw = snapshot();
    raw.entities.push(SnapshotEntity {
        entity_id: "weather.openweathermap".into(),
        name: Some("OpenWeatherMap".into()),
        original_name: None,
        has_entity_name: false,
        area_id: None,
        device_id: None,
        platform: Some("openweathermap".into()),
        aliases: vec!["Wetter".into()],
        labels: Vec::new(),
        disabled: false,
        device_class: None,
    });
    raw.entities.push(SnapshotEntity {
        entity_id: "sensor.openweathermap_temperatur".into(),
        name: Some("Temperatur".into()),
        original_name: None,
        has_entity_name: false,
        area_id: None,
        device_id: None,
        platform: Some("openweathermap".into()),
        aliases: Vec::new(),
        labels: Vec::new(),
        disabled: false,
        device_class: None,
    });
    raw.entities.push(SnapshotEntity {
        entity_id: "sensor.hidden_temp".into(),
        name: Some("Hidden".into()),
        original_name: None,
        has_entity_name: false,
        area_id: None,
        device_id: None,
        platform: None,
        aliases: Vec::new(),
        labels: Vec::new(),
        disabled: false,
        device_class: None,
    });
    raw.assist = Some(vec!["light.living".into(), "weather.openweathermap".into(), "sensor.openweathermap_temperatur".into()]);
    let home = ingest(raw).expect("snapshot");
    let ids: HashSet<_> = home.entities.iter().map(|entity| entity.entity_id.as_str()).collect();
    assert!(ids.contains("weather.openweathermap"));
    assert!(ids.contains("sensor.openweathermap_temperatur"));
    assert!(!ids.contains("sensor.hidden_temp"));
}
