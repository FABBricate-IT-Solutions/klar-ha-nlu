use klar_nlu::home::default_home;
use klar_nlu::nlu::semantic::{with_fixture_proposals, SemanticProposal, ADAPTER_CONFIDENCE_CAP};
use klar_nlu::nlu::{self};
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Intent, IntentPlan, ParseDecision, Settings};
use std::collections::HashSet;

fn adapters_on() -> Settings {
    Settings { semantic_adapters: true, languages: vec!["en".into()], ..Settings::default() }
}

fn adapters_off() -> Settings {
    Settings { languages: vec!["en".into()], ..Settings::default() }
}

fn proposal(name: &str, entity_id: &str, confidence: f64) -> SemanticProposal {
    SemanticProposal {
        provider: "test.fixture".into(),
        plan: IntentPlan::from_intents(vec![Intent::new(name).with("entity_id", entity_id)], confidence, &[]),
        reason: "fixture".into(),
    }
}

fn parse_with(text: &str, home: &HomeGraph, settings: &Settings, proposals: Vec<SemanticProposal>) -> klar_nlu::types::ParseOutcome {
    with_fixture_proposals(proposals, || {
        let mut session = Session::new();
        nlu::parse(text, home, &mut session, &[], settings)
    })
}

fn rejected_utterance() -> &'static str {
    "fnord the zyxlamp please"
}

#[test]
fn adapters_are_off_by_default_and_ignored() {
    let home = default_home();
    let off = parse_with(rejected_utterance(), &home, &adapters_off(), vec![proposal("HassTurnOn", "light.wohnzimmer", 0.99)]);
    assert!(matches!(off.decision, ParseDecision::Reject { .. }), "{off:#?}");
    assert!(off.plan.is_none());
    assert!(!off.trace.stages.iter().any(|stage| stage.stage == "semantic_adapter"));
}

#[test]
fn valid_typed_proposal_is_revalidated_and_can_execute() {
    let home = default_home();
    let outcome = parse_with(rejected_utterance(), &home, &adapters_on(), vec![proposal("HassTurnOn", "light.wohnzimmer", 0.99)]);
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    let plan = outcome.plan.expect("execute plan");
    assert_eq!(plan.intents()[0].name, "HassTurnOn");
    assert_eq!(plan.intents()[0].slot("entity_id"), Some("light.wohnzimmer"));
    assert!(outcome.confidence <= ADAPTER_CONFIDENCE_CAP);
    assert!(outcome.evidence.iter().any(|item| item.kind == "semantic_adapter" && item.source == "test.fixture"));
    assert!(outcome.trace.stages.iter().any(|stage| stage.stage == "semantic_adapter"));
    assert_eq!(outcome.candidates[0].policy, "semantic_adapter");
}

#[test]
fn unknown_intent_and_unexposed_entity_stay_rejected() {
    let home = default_home();
    let unknown = parse_with(rejected_utterance(), &home, &adapters_on(), vec![proposal("NotAnIntent", "light.wohnzimmer", 0.9)]);
    assert!(matches!(unknown.decision, ParseDecision::Reject { .. }), "{unknown:#?}");

    let mut hidden = default_home();
    hidden.assist = Some(HashSet::from(["light.esszimmer".into()]));
    let unexposed = parse_with(rejected_utterance(), &hidden, &adapters_on(), vec![proposal("HassTurnOn", "light.wohnzimmer", 0.9)]);
    assert!(matches!(unexposed.decision, ParseDecision::Reject { .. }), "{unexposed:#?}");
}

#[test]
fn risky_adapter_proposal_confirms_without_leaking_plan() {
    let home = default_home();
    let outcome = parse_with(rejected_utterance(), &home, &adapters_on(), vec![proposal("HassTurnOn", "lock.wohnungstuer", 0.86)]);
    assert!(matches!(outcome.decision, ParseDecision::Confirm { .. }), "{outcome:#?}");
    assert!(outcome.plan.is_none());
    assert!(outcome.candidates.is_empty());
    assert!(outcome.selected_candidate_id.is_none());
}

#[test]
fn adapters_do_not_override_execute_or_rewrite_ood() {
    let home = default_home();
    let hijack = vec![proposal("HassTurnOff", "light.esszimmer", 0.86)];
    let execute = parse_with("Turn on the living room light", &home, &adapters_on(), hijack.clone());
    assert!(matches!(execute.decision, ParseDecision::Execute), "{execute:#?}");
    assert_eq!(execute.plan.as_ref().unwrap().intents()[0].slot("entity_id"), Some("light.wohnzimmer"));
    assert!(!execute.trace.stages.iter().any(|stage| stage.stage == "semantic_adapter"));

    let ood = parse_with("What is the capital of France", &home, &adapters_on(), hijack);
    assert!(matches!(ood.decision, ParseDecision::Reject { .. }), "{ood:#?}");
    assert!(ood.plan.is_none());
    assert!(!ood.trace.stages.iter().any(|stage| stage.stage == "semantic_adapter"));
}
