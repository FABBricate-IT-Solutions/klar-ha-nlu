use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, ParseDecision, Settings};

fn home() -> HomeGraph {
    HomeGraph {
        areas: ["kantoor", "keuken", "woonkamer"]
            .into_iter()
            .map(|area| AreaRec { area_id: area.into(), name: area.into(), aliases: vec![], floor_id: None })
            .collect(),
        entities: ["kantoor", "keuken", "woonkamer"]
            .into_iter()
            .flat_map(|area| {
                [
                    EntityRec {
                        entity_id: format!("light.{area}"),
                        name: format!("{area} plafond"),
                        domain: "light".into(),
                        platform: None,
                        area: Some(area.into()),
                        aliases: vec![],
                        tags: vec![],
                    },
                    EntityRec {
                        entity_id: format!("binary_sensor.{area}_raam"),
                        name: format!("{area} raam"),
                        domain: "binary_sensor".into(),
                        platform: None,
                        area: Some(area.into()),
                        aliases: vec![],
                        tags: vec![],
                    },
                ]
            })
            .collect(),
        ..HomeGraph::default()
    }
}

#[test]
fn dutch_power_commands_generalize_across_forms_and_rooms() {
    for area in ["kantoor", "keuken", "woonkamer"] {
        for (form, expected) in [
            ("Doe het licht in het AREA uit", "HassTurnOff"),
            ("Kun je het licht in het AREA even uitdoen?", "HassTurnOff"),
            ("Wil je de lampen in de AREA uitschakelen?", "HassTurnOff"),
            ("Schakel de verlichting in AREA uit", "HassTurnOff"),
            ("Zet de lichten in AREA uit", "HassTurnOff"),
            ("Doe het licht in AREA aan", "HassTurnOn"),
            ("Kun je de lampen in AREA even aandoen?", "HassTurnOn"),
            ("Schakel de verlichting in AREA aan", "HassTurnOn"),
            ("Zet de lichten in AREA aan", "HassTurnOn"),
        ] {
            let text = form.replace("AREA", area);
            let out = parse(&text, &home(), &mut Session::new(), &[], &Settings::pinned("nl"));
            assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
            let intents = out.plan.unwrap().intents();
            assert_eq!(intents.len(), 1, "{text}");
            assert_eq!(intents[0].name, expected, "{text}");
            assert!(
                intents[0].slot("domain") == Some("light") || intents[0].slot("entity_id").is_some_and(|id| id.starts_with("light.")),
                "{text}"
            );
            assert!(
                intents[0].slot("area") == Some(area) || intents[0].slot("entity_id") == Some(format!("light.{area}").as_str()),
                "{text}: {intents:?}"
            );
        }
    }
}

#[test]
fn dutch_window_questions_have_exact_read_only_filters() {
    for (text, state, area) in [
        ("Welke ramen staan er open?", Some("on"), None),
        ("Welk raam is open?", Some("on"), None),
        ("Welke ramen zijn gesloten?", Some("off"), None),
        ("Welke ramen staan dicht?", Some("off"), None),
        ("Welke ramen staan er open in het kantoor?", Some("on"), Some("kantoor")),
        ("Wat is de status van de ramen?", None, None),
    ] {
        let out = parse(text, &home(), &mut Session::new(), &[], &Settings::pinned("nl"));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        let intents = out.plan.unwrap().intents();
        assert_eq!(intents.len(), 1, "{text}");
        let intent = &intents[0];
        assert_eq!(intent.name, "HassGetState", "{text}");
        assert_eq!(intent.slot("domain"), Some("binary_sensor"));
        assert_eq!(intent.slot("device_class"), Some("window"));
        assert_eq!(intent.slot("response"), Some("list"));
        assert_eq!(intent.slot("state"), state);
        assert_eq!(intent.slot("area"), area);
    }
}

#[test]
fn dutch_negation_never_emits_a_plan_even_with_session_context() {
    let mut session = Session::new();
    let settings = Settings::pinned("nl");
    let graph = home();
    let initial = parse("Doe het licht in kantoor aan", &graph, &mut session, &[], &settings);
    assert_eq!(initial.decision, ParseDecision::Execute);
    for text in [
        "Doe het licht in het kantoor niet uit",
        "Kun je het licht in het kantoor niet uitdoen?",
        "Zet geen lampen aan",
        "Schakel de verlichting nooit uit",
        "Nee, doe het licht uit",
        "Niet uitdoen",
        "Doe het licht in kantoor uit en zet de lampen in keuken niet aan",
        "Welke ramen staan niet open?",
        "Welke ramen staan open in de zolder?",
        "Welke ramen staan open of dicht?",
    ] {
        let out = parse(text, &graph, &mut session, &[], &settings);
        assert!(matches!(out.decision, ParseDecision::Reject { .. }), "{text}: {out:?}");
        assert!(out.plan.is_none(), "{text}");
    }
}

#[test]
fn window_list_speech_filters_and_reports_empty_results_in_dutch() {
    use klar_nlu::speech::render_snapshot;
    use klar_nlu::types::SpeechSnapshot;
    let mut snapshot: SpeechSnapshot = serde_json::from_value(serde_json::json!({
        "language": "nl", "outcome": "success",
        "intent": {"name": "HassGetState", "slots": [
            {"name": "domain", "value": "binary_sensor"}, {"name": "device_class", "value": "window"},
            {"name": "state", "value": "on"}, {"name": "response", "value": "list"}
        ]},
        "entities": [
            {"entity_id": "binary_sensor.office", "name": "Kantoorraam", "domain": "binary_sensor", "device_class": "window", "state": "on"},
            {"entity_id": "binary_sensor.kitchen", "name": "Keukenraam", "domain": "binary_sensor", "device_class": "window", "state": "off"},
            {"entity_id": "binary_sensor.door", "name": "Voordeur", "domain": "binary_sensor", "device_class": "door", "state": "on"}
        ]
    })).unwrap();
    assert_eq!(render_snapshot(&snapshot).speech, "Deze ramen staan open: Kantoorraam.");
    snapshot.entities.clear();
    assert_eq!(render_snapshot(&snapshot).speech, "Er staan geen ramen open.");
}

#[test]
fn negation_overrides_custom_rules_and_cancels_pending_confirmation() {
    use klar_nlu::types::{CustomSentence, Intent, IntentPlan};
    let graph = home();
    let settings = Settings::pinned("nl");
    let mut session = Session::new();
    let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("entity_id", "light.kantoor")], 1.0, &[]);
    assert!(session.set_confirm("test-confirm".into(), plan, "Uitdoen?".into()));
    let phrase = "Nee, doe het licht niet uit";
    let custom = [CustomSentence {
        phrase: phrase.into(),
        intent: "HassTurnOff".into(),
        slots: [("entity_id".into(), "light.kantoor".into())].into_iter().collect(),
    }];
    let out = parse(phrase, &graph, &mut session, &custom, &settings);
    assert!(matches!(out.decision, ParseDecision::Reject { .. }));
    assert!(out.plan.is_none());
    assert!(session.pending_confirm().is_none());
    let out = parse("ja", &graph, &mut session, &[], &settings);
    assert!(out.plan.is_none(), "Cancelled confirmation must not replay: {out:?}");
}

#[test]
fn requested_commands_work_with_the_reference_home_graph() {
    let graph = klar_nlu::home::default_home();
    for text in ["Doe het licht in het kantoor uit", "Kun je het licht in het kantoor even uitdoen?"] {
        let out = parse(text, &graph, &mut Session::new(), &[], &Settings::pinned("nl"));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        let intents = out.plan.unwrap().intents();
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].name, "HassTurnOff");
        assert!(
            intents[0].slot("area") == Some("arbeitszimmer") || intents[0].slot("entity_id").is_some_and(|id| id.contains("arbeitszimmer")),
            "{text}: {intents:?}"
        );
    }
}

#[test]
fn dutch_switch_verbs_ask_which_laundry_appliance() {
    let graph = klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/full_home/nl/home_config.yaml")).expect("dutch home");
    for text in ["doe apparaat wasruimte", "zet apparaat wasruimte"] {
        let out = parse(text, &graph, &mut Session::new(), &[], &Settings::pinned("nl"));
        let ParseDecision::Clarify { options, .. } = out.decision else {
            panic!("{text} should ask which appliance: {out:?}");
        };
        assert!(options.iter().any(|option| option.contains("washing_machine")), "{text}: {options:?}");
        assert!(options.iter().any(|option| option.contains("dryer")), "{text}: {options:?}");
    }
}
