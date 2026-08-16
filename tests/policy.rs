use klar_nlu::home::default_home;
use klar_nlu::nlu;
use klar_nlu::session::Session;
use klar_nlu::types::{Intent, IntentPlan, ParseDecision, RejectReason, Settings};
use std::collections::HashSet;

fn parse_de(text: &str, home: &klar_nlu::types::HomeGraph, session: &mut Session) -> klar_nlu::types::ParseOutcome {
    nlu::parse(text, home, session, &[], &Settings::default())
}

#[test]
fn named_followup_beats_session_replay() {
    let home = default_home();
    let mut session = Session::new();
    assert!(matches!(parse_de("Licht im Wohnzimmer an", &home, &mut session).decision, ParseDecision::Execute));
    let follow = parse_de("und die Küche auch", &home, &mut session);
    assert!(matches!(follow.decision, ParseDecision::Execute), "{follow:#?}");
    assert_eq!(
        follow.plan.as_ref().and_then(|plan| plan.steps.first()).and_then(|step| step.intent.slot("entity_id")),
        Some("light.kuche_kuche")
    );
}

fn parse_en(text: &str, home: &klar_nlu::types::HomeGraph, session: &mut Session) -> klar_nlu::types::ParseOutcome {
    nlu::parse(text, home, session, &[], &Settings { languages: vec!["en".into()], ..Settings::default() })
}

fn reject_reason(decision: &ParseDecision) -> Option<&RejectReason> {
    match decision {
        ParseDecision::Reject { reason } => Some(reason),
        _ => None,
    }
}

#[test]
fn exact_lexical_evidence_stays_executable() {
    let home = default_home();
    let outcome = parse_de("Licht im Wohnzimmer an", &home, &mut Session::new());
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    assert!(outcome.confidence >= 0.80, "{}", outcome.confidence);
    assert!(outcome.plan.is_some());
}

#[test]
fn fuzzy_and_session_evidence_score_below_exact() {
    let home = default_home();
    let exact = parse_de("Licht im Wohnzimmer an", &home, &mut Session::new());
    let fuzzy = parse_de("Licht im Wohnzimer an", &home, &mut Session::new());
    assert!(matches!(exact.decision, ParseDecision::Execute));
    assert!(matches!(fuzzy.decision, ParseDecision::Execute), "{fuzzy:#?}");
    assert!(fuzzy.confidence < exact.confidence, "fuzzy={} exact={}", fuzzy.confidence, exact.confidence);
    assert!(fuzzy.evidence.iter().any(|item| item.kind.starts_with("target_") && !item.exact));

    let mut session = Session::new();
    assert!(matches!(parse_de("Licht im Wohnzimmer an", &home, &mut session).decision, ParseDecision::Execute));
    let replay = parse_de("mach sie aus", &home, &mut session);
    assert!(matches!(replay.decision, ParseDecision::Execute), "{replay:#?}");
    assert!(replay.confidence < exact.confidence, "session={} exact={}", replay.confidence, exact.confidence);
    assert!(replay.confidence < 1.0);
}

#[test]
fn inferred_actions_never_claim_perfect_confidence() {
    let home = default_home();
    let mut session = Session::new();
    assert!(matches!(parse_de("Licht im Wohnzimmer an", &home, &mut session).decision, ParseDecision::Execute));
    let outcome = parse_de("und die Küche auch", &home, &mut session);
    assert!(outcome.evidence.iter().any(|item| item.source == "context_inference"), "expected context_inference evidence: {outcome:#?}");
    assert!(outcome.confidence <= 0.86, "{}", outcome.confidence);
    assert!(outcome.confidence < 1.0);
    assert!(outcome.plan.as_ref().is_some_and(|plan| {
        plan.steps.iter().all(|step| step.confidence <= 0.86)
            && plan.steps.iter().any(|step| step.evidence.iter().any(|item| item.source == "context_inference"))
    }));
}

#[test]
fn ood_sentences_reject_in_german_and_english() {
    let home = default_home();
    for (language, text, expected) in [
        ("de", "Wie ist das Wetter", RejectReason::NoAction),
        ("de", "Was ist die Hauptstadt von Frankreich", RejectReason::NoAction),
        ("de", "Was soll ich kochen", RejectReason::NoAction),
        ("de", "bitte mal doch", RejectReason::EmptyInput),
        ("en", "What's the weather", RejectReason::NoAction),
        ("en", "What is the capital of France", RejectReason::NoAction),
        ("en", "please please", RejectReason::EmptyInput),
    ] {
        let outcome =
            if language == "de" { parse_de(text, &home, &mut Session::new()) } else { parse_en(text, &home, &mut Session::new()) };
        assert_eq!(reject_reason(&outcome.decision), Some(&expected), "{language} {text}: {outcome:#?}");
        assert!(outcome.plan.is_none());
        assert!(outcome.candidates.is_empty());
        assert!(!matches!(outcome.decision, ParseDecision::Chat | ParseDecision::Execute));
    }
}

#[test]
fn news_and_opt_in_remain_chat() {
    let home = default_home();
    let news = parse_de("Was sind die aktuellen Nachrichten", &home, &mut Session::new());
    assert!(matches!(news.decision, ParseDecision::Chat), "{news:#?}");
    let joke = parse_de("Erzähl einen Witz", &home, &mut Session::new());
    assert!(matches!(joke.decision, ParseDecision::Chat), "{joke:#?}");
    let mut briefing = Session::new();
    assert!(matches!(parse_de("Was sind die aktuellen Nachrichten", &home, &mut briefing).decision, ParseDecision::Chat));
    let follow = parse_de("Wie ist das Wetter", &home, &mut briefing);
    assert!(matches!(follow.decision, ParseDecision::Chat), "{follow:#?}");
}

#[test]
fn risky_cover_close_and_lock_require_confirm_without_plan() {
    let home = default_home();
    for text in ["Wohnungstür abschließen", "Wohnungstür aufschließen", "Rollo im Wohnzimmer zu"] {
        let outcome = parse_de(text, &home, &mut Session::new());
        assert!(matches!(outcome.decision, ParseDecision::Confirm { .. }), "{text}: {outcome:#?}");
        assert!(outcome.plan.is_none(), "{text} leaked a plan");
        assert!(outcome.candidates.is_empty(), "{text} leaked candidates");
    }
}

#[test]
fn dismiss_clears_pending_confirm_and_executes_nothing() {
    let home = default_home();
    let mut session = Session::new();
    let confirmation = parse_de("Wohnungstür abschließen", &home, &mut session);
    assert!(matches!(confirmation.decision, ParseDecision::Confirm { .. }));
    assert!(session.pending_confirm().is_some());
    let dismissed = parse_de("nein", &home, &mut session);
    assert!(matches!(dismissed.decision, ParseDecision::Reject { .. }), "{dismissed:#?}");
    assert!(dismissed.plan.is_none());
    assert!(session.pending_confirm().is_none());
    assert!(session.last.is_empty());
}

#[test]
fn yes_revalidates_stored_plan_against_current_graph() {
    let home = default_home();
    let mut session = Session::new();
    let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", "lock.wohnungstuer")], 0.9, &[]);
    session.set_confirm("confirm-lock".into(), plan, "Really lock it?".into());

    let mut gone = home.clone();
    gone.assist = Some(HashSet::new());
    let rejected = nlu::parse("ja", &gone, &mut session, &[], &Settings::default());
    assert!(matches!(rejected.decision, ParseDecision::Reject { reason: RejectReason::Unsafe }), "{rejected:#?}");
    assert!(rejected.plan.is_none());
    assert!(session.last.is_empty());

    let mut fresh = Session::new();
    fresh.set_confirm(
        "confirm-lock".into(),
        IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", "lock.wohnungstuer")], 0.9, &[]),
        "Really lock it?".into(),
    );
    let executed = parse_de("ja", &home, &mut fresh);
    assert!(matches!(executed.decision, ParseDecision::Execute), "{executed:#?}");
    assert_eq!(
        executed.plan.as_ref().and_then(|plan| plan.steps.first()).and_then(|step| step.intent.slot("entity_id")),
        Some("lock.wohnungstuer")
    );
}

#[test]
fn multi_intent_drops_invalid_and_keeps_valid() {
    let home = default_home();
    let mut restricted = home.clone();
    restricted.assist =
        Some(home.entities.iter().filter(|entity| entity.entity_id.starts_with("light.")).map(|entity| entity.entity_id.clone()).collect());
    let outcome = parse_de("Licht im Wohnzimmer an und Wohnungstür abschließen", &restricted, &mut Session::new());
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    let steps = &outcome.plan.as_ref().expect("execute plan").steps;
    assert_eq!(steps.len(), 1, "{outcome:#?}");
    assert_eq!(steps[0].intent.name, "HassTurnOn");
    assert_eq!(steps[0].intent.slot("entity_id"), Some("light.wohnzimmer"));
    assert!(steps.iter().all(|step| step.intent.slot("entity_id") != Some("lock.wohnungstuer")));
}

#[test]
fn multi_intent_rejects_when_no_valid_step_remains() {
    let home = default_home();
    let mut restricted = home.clone();
    restricted.assist = Some(HashSet::new());
    let outcome = parse_de("Wohnungstür abschließen und Heizung auf 21", &restricted, &mut Session::new());
    assert!(matches!(outcome.decision, ParseDecision::Reject { .. }), "{outcome:#?}");
    assert!(outcome.plan.is_none());
}

#[test]
fn below_execute_threshold_does_not_emit_a_plan() {
    let mut home = default_home();
    home.entities.push(klar_nlu::types::EntityRec {
        entity_id: "cover.schlafzimmer_vorhang".into(),
        name: "Schlafzimmer Vorhang".into(),
        domain: "cover".into(),
        platform: None,
        area: Some("schlafzimmer".into()),
        aliases: vec!["vorhang".into()],
        tags: Vec::new(),
    });
    let outcome = nlu::parse(
        "Mach die Vorhänge zu",
        &home,
        &mut Session::new(),
        &[],
        &Settings { confirm_risky_actions: false, ..Settings::default() },
    );
    assert!(outcome.confidence < 0.80, "{}", outcome.confidence);
    assert!(!matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    assert!(outcome.plan.is_none());
    if let ParseDecision::Reject { reason } = &outcome.decision {
        assert_ne!(reason, &RejectReason::NoTarget, "{outcome:#?}");
    }
}

#[test]
fn competing_close_complete_plans_clarify() {
    let home = default_home();
    let settings = Settings::default();
    let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", "light.wohnzimmer")], 0.92, &[]);
    let (decision, plan_out) = nlu::safety_decide(&home, &settings, plan, 0.92, 0.02, true);
    assert!(matches!(decision, ParseDecision::Clarify { .. }), "{decision:#?}");
    assert!(plan_out.is_none());

    let mut home = default_home();
    home.entities.push(klar_nlu::types::EntityRec {
        entity_id: "light.nordlicht_a".into(),
        name: "Nordlicht".into(),
        domain: "light".into(),
        platform: None,
        area: Some("wohnzimmer".into()),
        aliases: Vec::new(),
        tags: Vec::new(),
    });
    home.entities.push(klar_nlu::types::EntityRec {
        entity_id: "light.nordlicht_b".into(),
        name: "Nordlicht".into(),
        domain: "light".into(),
        platform: None,
        area: Some("kuche".into()),
        aliases: Vec::new(),
        tags: Vec::new(),
    });
    let outcome = parse_de("Nordlicht an", &home, &mut Session::new());
    assert!(matches!(outcome.decision, ParseDecision::Clarify { .. }), "{outcome:#?}");
    assert!(outcome.plan.is_none());
}

#[test]
fn floor_gate_still_executes() {
    let home = klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/familienhaus_de/home_config.yaml")).expect("home");
    let outcome = nlu::parse(
        "Licht im Obergeschoss an",
        &home,
        &mut Session::new(),
        &[],
        &Settings { languages: vec!["de".into()], ..Settings::default() },
    );
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    assert!(outcome.plan.as_ref().is_some_and(|plan| plan.steps.iter().any(|step| step.intent.slot("floor") == Some("upper"))));
}

fn family_home() -> klar_nlu::types::HomeGraph {
    klar_nlu::home::load_home_config(std::path::Path::new("tests/datasets/family_home_en/home_config.yaml")).expect("family home")
}

#[test]
fn named_thermostat_beats_same_domain_competitor() {
    let home = family_home();
    let outcome = parse_en("Set the Ground Floor Thermostat to 23 degrees", &home, &mut Session::new());
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    assert_eq!(
        outcome.plan.as_ref().and_then(|plan| plan.steps.first()).and_then(|step| step.intent.slot("entity_id")),
        Some("climate.ground_thermostat")
    );
}

#[test]
fn named_ceiling_does_not_expand_to_every_ceiling() {
    let home = family_home();
    let outcome = parse_en("Turn on the Bedroom 2 Ceiling light", &home, &mut Session::new());
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    let entities: Vec<_> =
        outcome.plan.as_ref().map(|plan| plan.steps.iter().filter_map(|step| step.intent.slot("entity_id")).collect()).unwrap_or_default();
    assert!(!entities.contains(&"light.living_ceiling"), "{outcome:#?}");
    assert!(
        entities.contains(&"light.bedroom2_ceiling")
            || outcome.plan.as_ref().is_some_and(|plan| plan.steps.iter().any(|step| step.intent.slot("area") == Some("bedroom_2"))),
        "{outcome:#?}"
    );
}

#[test]
fn area_climate_query_executes_without_confirm() {
    let home = default_home();
    let outcome = parse_de("Wie warm ist es im Schlafzimmer", &home, &mut Session::new());
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    assert!(outcome.plan.as_ref().is_some_and(|plan| {
        plan.steps.iter().any(|step| {
            matches!(step.intent.name.as_str(), "HassClimateGetTemperature" | "HassGetState")
                && (step.intent.slot("area") == Some("schlafzimmer")
                    || step.intent.slot("entity_id").is_some_and(|id| id.starts_with("climate.")))
        })
    }));
}

#[test]
fn garage_close_keeps_the_close_step() {
    let home = family_home();
    let outcome = nlu::parse(
        "Flick the garage door closed",
        &home,
        &mut Session::new(),
        &[],
        &Settings { languages: vec!["en".into()], confirm_risky_actions: false, ..Settings::default() },
    );
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    assert!(
        outcome.plan.as_ref().is_some_and(|plan| {
            plan.steps.iter().any(|step| step.intent.name == "HassTurnOff" && step.intent.slot("entity_id") == Some("cover.garage_door"))
        }),
        "{outcome:#?}"
    );
}
