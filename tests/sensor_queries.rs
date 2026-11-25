//! P-17 / #329: window questions with extra words stay on window sensors, never a cover.
//! Table-driven sensor asks also cover door, motion, smoke, humidity, temperature, battery.

use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::speech::render_snapshot;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, ParseDecision, Settings, SpeechSnapshot};

fn entity(id: &str, name: &str, area: Option<&str>) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: id.split_once('.').map(|(domain, _)| domain).unwrap_or("").into(),
        platform: None,
        area: area.map(Into::into),
        aliases: vec![],
        tags: vec![],
    }
}

fn area(id: &str, name: &str) -> AreaRec {
    AreaRec { area_id: id.into(), name: name.into(), aliases: vec![], floor_id: None }
}

/// Home shaped like the #329 probe: garage cover plus window contacts (by device_class).
fn p17_home() -> HomeGraph {
    HomeGraph {
        areas: vec![area("garage", "Garage"), area("living_room", "Living Room"), area("bedroom", "Bedroom"), area("kitchen", "Kitchen")],
        entities: vec![
            entity("cover.garage_door", "Garage Door", Some("garage")),
            entity("binary_sensor.living_room_contact", "Living Room Contact", Some("living_room")),
            entity("binary_sensor.bedroom_window", "Bedroom Window", Some("bedroom")),
            entity("binary_sensor.kitchen_sensor", "Kitchen Sensor", Some("kitchen")),
            entity("light.living_room", "Living Room Light", Some("living_room")),
            entity("light.bedroom", "Bedroom Light", Some("bedroom")),
        ],
        device_classes: [
            ("cover.garage_door".into(), "garage".into()),
            ("binary_sensor.living_room_contact".into(), "window".into()),
            ("binary_sensor.bedroom_window".into(), "window".into()),
            ("binary_sensor.kitchen_sensor".into(), "window".into()),
        ]
        .into_iter()
        .collect(),
        ..HomeGraph::default()
    }
}

fn sensor_home() -> HomeGraph {
    HomeGraph {
        areas: vec![area("kitchen", "Kitchen"), area("hall", "Hall")],
        entities: vec![
            entity("cover.garage_door", "Garage Door", None),
            entity("binary_sensor.kitchen_door", "Kitchen Door", Some("kitchen")),
            entity("binary_sensor.hall_motion", "Hall Motion", Some("hall")),
            entity("binary_sensor.kitchen_smoke", "Kitchen Smoke", Some("kitchen")),
            entity("sensor.kitchen_humidity", "Kitchen Humidity", Some("kitchen")),
            entity("sensor.kitchen_temperature", "Kitchen Temperature", Some("kitchen")),
            entity("sensor.door_battery", "Door Battery", Some("hall")),
        ],
        device_classes: [
            ("cover.garage_door".into(), "garage".into()),
            ("binary_sensor.kitchen_door".into(), "door".into()),
            ("binary_sensor.hall_motion".into(), "motion".into()),
            ("binary_sensor.kitchen_smoke".into(), "smoke".into()),
            ("sensor.kitchen_humidity".into(), "humidity".into()),
            ("sensor.kitchen_temperature".into(), "temperature".into()),
            ("sensor.door_battery".into(), "battery".into()),
        ]
        .into_iter()
        .collect(),
        ..HomeGraph::default()
    }
}

fn assert_window_list(text: &str, home: &HomeGraph) {
    let out = parse(text, home, &mut Session::new(), &[], &Settings::pinned("en"));
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    let intents = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    assert_eq!(intents.len(), 1, "{text}: {out:?}");
    let intent = &intents[0];
    assert_eq!(intent.name, "HassGetState", "{text}");
    assert_eq!(intent.slot("domain"), Some("binary_sensor"), "{text}");
    assert_eq!(intent.slot("device_class"), Some("window"), "{text}");
    assert_eq!(intent.slot("response"), Some("list"), "{text}");
    assert!(intent.slot("entity_id").is_none(), "{text}: named a device {intent:?}");
    assert!(intents.iter().all(|step| !step.slot("entity_id").is_some_and(|id| id.starts_with("cover."))), "{text}: named a cover {out:?}");
}

/// The ten window formulations from the #329 finding (extra words that used to hit the garage).
#[test]
fn p17_ten_window_formulations_never_name_a_cover() {
    let home = p17_home();
    for text in [
        "are any windows still open",
        "are any windows open in the house",
        "are any windows open at home",
        "did I leave any windows open",
        "did I leave a window open",
        "are any windows left open",
        "are all windows shut",
        "how many windows are open",
        "is there a window open somewhere",
        "have I left the windows open",
    ] {
        assert_window_list(text, &home);
    }
}

#[test]
fn sensor_asks_use_device_class_not_covers() {
    let home = sensor_home();
    let settings = Settings::pinned("en");
    for (text, class, domain) in [
        ("are any doors open", "door", "binary_sensor"),
        ("is there motion in the hall", "motion", "binary_sensor"),
        ("is there smoke in the kitchen", "smoke", "binary_sensor"),
        ("what is the humidity in the kitchen", "humidity", "sensor"),
        ("what is the temperature sensor in the kitchen", "temperature", "sensor"),
        ("what is the battery level", "battery", "sensor"),
    ] {
        let out = parse(text, &home, &mut Session::new(), &[], &settings);
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        let intents = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
        assert!(
            intents.iter().any(|intent| {
                intent.name == "HassGetState"
                    && intent.slot("device_class") == Some(class)
                    && intent.slot("domain") == Some(domain)
                    && intent.slot("response") == Some("list")
            }),
            "{text}: {out:?}"
        );
        assert!(intents.iter().all(|intent| intent.slot("entity_id") != Some("cover.garage_door")), "{text}: {out:?}");
    }
}

#[test]
fn empty_window_snapshot_means_no_sensors_visible() {
    let none: SpeechSnapshot = serde_json::from_value(serde_json::json!({
        "language": "en", "outcome": "success",
        "intent": {"name": "HassGetState", "slots": [
            {"name": "domain", "value": "binary_sensor"}, {"name": "device_class", "value": "window"},
            {"name": "state", "value": "on"}, {"name": "response", "value": "list"}
        ]},
        "entities": []
    }))
    .unwrap();
    assert_eq!(render_snapshot(&none).speech, "No window sensors were found.");

    let closed: SpeechSnapshot = serde_json::from_value(serde_json::json!({
        "language": "en", "outcome": "success",
        "intent": {"name": "HassGetState", "slots": [
            {"name": "domain", "value": "binary_sensor"}, {"name": "device_class", "value": "window"},
            {"name": "state", "value": "on"}, {"name": "response", "value": "list"}
        ]},
        "entities": [
            {"entity_id": "binary_sensor.kitchen_window", "name": "Kitchen Window",
             "domain": "binary_sensor", "device_class": "window", "state": "off"}
        ]
    }))
    .unwrap();
    assert_eq!(render_snapshot(&closed).speech, "No windows are open.");
}
