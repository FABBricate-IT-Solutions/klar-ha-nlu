use klar_nlu::home::default_home;
use klar_nlu::nlu;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{Intent, IntentPlan, ParseDecision, ParseOutcome, Settings, PARSE_SCHEMA_VERSION};

fn stable_json(mut outcome: ParseOutcome) -> String {
    for stage in &mut outcome.trace.stages {
        stage.duration_us = 0;
    }
    serde_json::to_string(&outcome).expect("serialize outcome")
}

fn assert_no_executable_shape(value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for forbidden in ["plan", "intent", "slots", "selected_candidate_id"] {
                assert!(!fields.contains_key(forbidden), "non-execute payload leaked {forbidden}: {value}");
            }
            for child in fields.values() {
                assert_no_executable_shape(child);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                assert_no_executable_shape(child);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) | serde_json::Value::String(_) => {}
    }
}

#[test]
fn v2_json_contract_has_versioned_shape_and_deterministic_ordering() {
    let home = default_home();
    let mut first_session = Session::new();
    first_session.id = "contract-session".into();
    let mut second_session = first_session.clone();
    let settings = Settings::default();
    let first = nlu::parse("Mach das Licht im Wohnzimmer an", &home, &mut first_session, &[], &settings);
    let second = nlu::parse("Mach das Licht im Wohnzimmer an", &home, &mut second_session, &[], &settings);

    assert_eq!(stable_json(first.clone()), stable_json(second));
    let json = serde_json::to_value(&first).expect("JSON value");
    assert_eq!(json["schema_version"], PARSE_SCHEMA_VERSION);
    assert_eq!(json["decision"]["type"], "execute");
    assert_eq!(json["plan"]["steps"][0]["index"], 0);
    assert_eq!(json["plan"]["steps"][0]["intent"]["name"], "HassTurnOn");
    assert!(json["candidates"][0]["policy"].is_string());
    assert!(json["candidates"][0]["score"].is_number());
    assert!(json["candidates"][0]["margin"].is_number());
    assert!(json["evidence"].is_array());
    assert_eq!(
        first.trace.stages.iter().map(|stage| stage.stage.as_str()).collect::<Vec<_>>(),
        ["normalize", "features", "action_candidates", "target_resolution", "binding", "ranking", "safety_decision", "planning"]
    );
    assert!(!first.trace.tokens.is_empty());
    assert!(!first.trace.normalized.is_empty());
}

#[test]
fn v2_propagates_fuzzy_target_evidence_and_confidence() {
    let home = default_home();
    let mut session = Session::new();
    let outcome = nlu::parse("Licht im Wohnzimer an", &home, &mut session, &[], &Settings::default());
    let mut exact_session = Session::new();
    let exact = nlu::parse("Licht im Wohnzimmer an", &home, &mut exact_session, &[], &Settings::default());
    assert!(matches!(outcome.decision, ParseDecision::Execute));
    assert!(outcome.confidence > 0.0);
    assert!(outcome.confidence < exact.confidence, "fuzzy={} exact={}", outcome.confidence, exact.confidence);
    assert!(outcome.evidence.iter().any(|evidence| evidence.kind.starts_with("target_") && !evidence.exact));
    assert!(outcome.candidates.iter().all(|candidate| (0.0..=1.0).contains(&candidate.score)));
}

#[test]
fn excessive_clause_complexity_rejects_before_candidate_generation() {
    let home = default_home();
    let mut session = Session::new();
    let text = (0..20)
        .map(|index| if index % 2 == 0 { "Licht im Wohnzimmer an" } else { "Licht im Esszimmer aus" })
        .collect::<Vec<_>>()
        .join(" und ");
    let outcome = nlu::parse(&text, &home, &mut session, &[], &Settings::default());
    assert!(matches!(outcome.decision, ParseDecision::Reject { .. }), "{outcome:#?}");
    assert!(outcome.candidates.is_empty());
    assert!(session.last.is_empty());
}

#[test]
fn selected_candidate_matches_plan_and_aggregates_confidence() {
    let home = default_home();
    let mut session = Session::new();
    let outcome =
        nlu::parse("Mach das Licht im Wohnzimmer an und das Licht im Esszimmer aus", &home, &mut session, &[], &Settings::default());
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    let plan = outcome.plan.as_ref().expect("execute plan");
    let selected = outcome
        .candidates
        .iter()
        .find(|candidate| Some(candidate.id.as_str()) == outcome.selected_candidate_id.as_deref())
        .expect("selected candidate");
    assert_eq!(&selected.plan, plan);
    assert_eq!(plan.confidence, plan.steps.iter().map(|step| step.confidence).reduce(f64::min).unwrap());
    assert_eq!(plan.margin, selected.margin);
    assert!(outcome.candidates.iter().all(|candidate| candidate.plan.steps.len() >= 2));
    for step in &plan.steps {
        let target = step.intent.slot("entity_id").or_else(|| step.intent.slot("area"));
        assert!(step.evidence.iter().filter(|item| item.kind.starts_with("target_")).all(|item| Some(item.value.as_str()) == target));
    }
}

#[test]
fn whole_plan_ranking_does_not_replay_a_previous_target() {
    let home = klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/family_home_en/home_config.yaml")).expect("home");
    let mut session = Session::new();
    let outcome =
        nlu::parse("Turn off the dishwasher and then start the Movie Night scene.", &home, &mut session, &[], &Settings::default());
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    let intents = outcome.plan.as_ref().expect("execute plan").steps.iter().map(|step| &step.intent).collect::<Vec<_>>();
    assert_eq!(intents.len(), 2, "{outcome:#?}");
    assert_eq!(intents[0].name, "HassTurnOff");
    assert_eq!(intents[0].slot("entity_id"), Some("switch.dishwasher"));
    assert_eq!(intents[1].name, "HassTurnOn");
    assert_eq!(intents[1].slot("entity_id"), Some("scene.movie_night"));
}

#[test]
fn relative_volume_parser_emits_only_valid_directions() {
    let home = klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/family_home_en/home_config.yaml")).expect("home");
    let settings = Settings { languages: vec!["en".into()], ..Settings::default() };
    for (text, expected) in [("Family Room TV louder", "up"), ("Family Room TV quieter", "down")] {
        let mut session = Session::new();
        let outcome = nlu::parse(text, &home, &mut session, &[], &settings);
        assert!(matches!(outcome.decision, ParseDecision::Execute), "{text}: {outcome:#?}");
        let step = outcome.plan.as_ref().and_then(|plan| plan.steps.first()).expect("relative volume step");
        assert_eq!(step.intent.name, "HassSetVolumeRelative");
        assert_eq!(step.intent.slot("volume_step"), Some(expected));
    }
}

#[test]
fn parallel_language_catalogs_do_not_cross_contaminate() {
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let mut handles = Vec::new();
    for (language, text, expected) in [("de", "Mach das Licht im Arbeitszimmer an", "ist an"), ("en", "Turn on the office light", "is on")]
    {
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            let home = default_home();
            let settings = Settings { languages: vec![language.into()], ..Settings::default() };
            barrier.wait();
            for _ in 0..50 {
                let mut session = Session::new();
                let outcome = nlu::parse(text, &home, &mut session, &[], &settings);
                assert!(matches!(outcome.decision, ParseDecision::Execute), "{language}: {outcome:#?}");
                assert!(outcome.speech.contains(expected), "{language}: {}", outcome.speech);
            }
        }));
    }
    for handle in handles {
        handle.join().expect("language worker");
    }
}

#[test]
fn dual_run_handles_scored_disambiguation_and_queries() {
    let home = klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/family_home_en/home_config.yaml")).expect("home");
    for text in [
        "Are the laundry switches on or off",
        "Flick on the laundry switch",
        "Is the front door sensor open or closed?",
        "What's playing on the Family Room TV right now",
        "What's the volume level of the Living Room TV?",
    ] {
        let mut session = Session::new();
        let (_, parity_error) = klar_nlu::parse::parse_checked(text, &home, &mut session, &[], &Settings::default());
        assert!(parity_error.is_none(), "{text}: {}", parity_error.unwrap_or_default());
    }
}

#[test]
fn clarify_state_mutates_memory_only_after_execution() {
    let home = default_home();
    let settings = Settings::default();
    let mut session = Session::new();
    let clarification = nlu::parse("Mach das Licht an", &home, &mut session, &[], &settings);
    assert!(matches!(clarification.decision, ParseDecision::Clarify { .. }));
    assert!(session.pending_clarify().is_some());
    assert!(session.last.is_empty());

    let executed = nlu::parse("das erste", &home, &mut session, &[], &settings);
    assert!(matches!(executed.decision, ParseDecision::Execute));
    assert!(session.pending_clarify().is_none());
    assert!(!session.last.is_empty());
}

#[test]
fn confirm_state_exposes_plan_and_commits_only_after_affirmation() {
    let home = default_home();
    let settings = Settings::default();
    let mut session = Session::new();
    let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", "lock.wohnungstuer")], 0.9, &[]);
    session.set_confirm("confirm-lock".into(), plan, "Really lock it?".into());

    let pending = nlu::parse("maybe", &home, &mut session, &[], &settings);
    assert!(matches!(pending.decision, ParseDecision::Confirm { .. }));
    assert!(pending.plan.is_none());
    assert_no_executable_shape(&serde_json::to_value(&pending).expect("serialize confirmation"));
    assert!(session.last.is_empty());

    let executed = nlu::parse("ja", &home, &mut session, &[], &settings);
    assert!(matches!(executed.decision, ParseDecision::Execute));
    assert!(session.pending_confirm().is_none());
    assert_eq!(session.last_entities().next(), Some("lock.wohnungstuer"));
}

#[test]
fn risky_lock_is_automatically_confirmed_without_executable_plan() {
    let home = default_home();
    let settings = Settings::default();
    let mut session = Session::new();
    let confirmation = nlu::parse("Wohnungstür abschließen", &home, &mut session, &[], &settings);
    assert!(matches!(confirmation.decision, ParseDecision::Confirm { .. }), "{confirmation:#?}");
    assert!(confirmation.plan.is_none());
    assert_no_executable_shape(&serde_json::to_value(&confirmation).expect("serialize confirmation"));
    assert!(session.pending_confirm().is_some());
    assert!(session.last.is_empty());

    let executed = nlu::parse("ja", &home, &mut session, &[], &settings);
    assert!(matches!(executed.decision, ParseDecision::Execute), "{executed:#?}");
    assert_eq!(
        executed.plan.as_ref().and_then(|plan| plan.steps.first()).and_then(|step| step.intent.slot("entity_id")),
        Some("lock.wohnungstuer")
    );
}

#[test]
fn golden_dual_run_including_follow_up() {
    let home = default_home();
    let settings = Settings::default();
    let mut session = Session::new();
    for text in ["Licht im Wohnzimmer an", "mach sie aus", "Wie ist der Status vom Schlafzimmerlicht"] {
        let result = parse(text, &home, &mut session, &[], &settings);
        assert_eq!(result.intents.len(), 1, "{text}: {result:?}");
    }
}

#[test]
fn golden_dual_run_clarification_follow_up() {
    let home = klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/familienhaus_de/home_config.yaml")).expect("home");
    let settings = Settings::default();
    let mut session = Session::new();
    let first = parse("Mach Licht Schlafzimmer an", &home, &mut session, &[], &settings);
    assert!(first.clarify);
    let second = parse("die Lampe", &home, &mut session, &[], &settings);
    assert_eq!(second.intents.first().and_then(|intent| intent.slot("entity_id")), Some("light.master_bedside_left"));
}
