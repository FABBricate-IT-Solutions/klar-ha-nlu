use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, ParseDecision, Settings};

fn home() -> HomeGraph {
    HomeGraph {
        areas: vec![AreaRec { area_id: "kitchen".into(), name: "Kitchen".into(), aliases: vec![], floor_id: None }],
        entities: vec![
            EntityRec {
                entity_id: "cover.garage_door".into(),
                name: "Garage Door".into(),
                domain: "cover".into(),
                platform: None,
                area: None,
                aliases: vec![],
                tags: vec![],
            },
            EntityRec {
                entity_id: "binary_sensor.kitchen_window".into(),
                name: "Kitchen Window".into(),
                domain: "binary_sensor".into(),
                platform: None,
                area: Some("kitchen".into()),
                aliases: vec![],
                tags: vec![],
            },
        ],
        ..HomeGraph::default()
    }
}

fn window_query(text: &str) -> klar_nlu::types::Intent {
    let out = parse(text, &home(), &mut Session::new(), &[], &Settings::pinned("en"));
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    let intents = out.plan.unwrap().intents();
    assert_eq!(intents.len(), 1, "{text}");
    let intent = intents[0].clone();
    assert_eq!(intent.name, "HassGetState", "{text}");
    assert_eq!(intent.slot("domain"), Some("binary_sensor"), "{text}");
    assert_eq!(intent.slot("device_class"), Some("window"), "{text}");
    assert_eq!(intent.slot("response"), Some("list"), "{text}");
    assert!(intent.slot("entity_id").is_none(), "{text}");
    intent
}

#[test]
fn open_window_questions_use_window_sensors_not_the_garage_cover() {
    for text in ["are any windows open", "are there any windows open", "which windows are open", "is the window open"] {
        let intent = window_query(text);
        assert_eq!(intent.slot("state"), Some("on"), "{text}");
        assert!(intent.slot("area").is_none(), "{text}");
    }
    let kitchen = window_query("are the kitchen windows open");
    assert_eq!(kitchen.slot("area"), Some("kitchen"));
    let closed = window_query("are the windows closed");
    assert_eq!(closed.slot("state"), Some("off"));
}

#[test]
fn window_commands_and_garage_questions_stay_covers() {
    let graph = home();
    let settings = Settings::pinned("en");
    for text in ["open the window", "close the windows", "open the garage door", "is the garage door open"] {
        let out = parse(text, &graph, &mut Session::new(), &[], &settings);
        let intents = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
        assert!(intents.iter().all(|intent| intent.slot("device_class") != Some("window")), "{text}: {out:?}");
    }
}

#[test]
fn english_window_list_names_open_sensors_and_ignores_covers() {
    use klar_nlu::speech::render_snapshot;
    use klar_nlu::types::SpeechSnapshot;
    let snapshot: SpeechSnapshot = serde_json::from_value(serde_json::json!({
        "language": "en", "outcome": "success",
        "intent": {"name": "HassGetState", "slots": [
            {"name": "domain", "value": "binary_sensor"}, {"name": "device_class", "value": "window"},
            {"name": "state", "value": "on"}, {"name": "response", "value": "list"}
        ]},
        "entities": [
            {"entity_id": "binary_sensor.kitchen_window", "name": "Kitchen Window", "domain": "binary_sensor", "device_class": "window", "state": "on"},
            {"entity_id": "binary_sensor.bedroom_window", "name": "Bedroom Window", "domain": "binary_sensor", "device_class": "window", "state": "off"},
            {"entity_id": "cover.garage_door", "name": "Garage Door", "domain": "cover", "state": "open"}
        ]
    }))
    .unwrap();
    assert_eq!(render_snapshot(&snapshot).speech, "These windows are open: Kitchen Window.");
}
