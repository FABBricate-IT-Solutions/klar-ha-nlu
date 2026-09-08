use super::*;
use crate::home::default_home;
use crate::lang::catalog_for;
use crate::session::Session;
use crate::types::{Intent, IntentPlan, Settings};

fn decide(confidence: f64, margin: f64, competing: bool, risky_lock: bool) -> Draft {
    let home = default_home();
    let session = Session::new();
    let settings = Settings::default();
    let catalog = catalog_for(&["de".into()]);
    let context = ParseContext::new("test", &home, &session, &[], &settings, catalog);
    let entity = if risky_lock { "lock.wohnungstuer" } else { "light.wohnzimmer" };
    let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", entity)], confidence, &[]);
    let mut draft = execute_plan(&context, plan, "test", None, None, false, false);
    draft.confidence = confidence;
    draft.margin = margin;
    draft.competing = competing;
    safety_decision(draft, &context)
}

#[test]
fn competing_low_margin_clarifies_instead_of_execute() {
    let decided = decide(0.92, 0.02, true, false);
    assert!(matches!(decided.decision, ParseDecision::Clarify { .. }), "{:#?}", decided.decision);
    assert!(decided.plan.is_none());
}

#[test]
fn confidence_between_clarify_and_execute_does_not_emit_a_plan() {
    let decided = decide(0.75, 1.0, false, false);
    assert!(matches!(decided.decision, ParseDecision::Clarify { .. }), "{:#?}", decided.decision);
    assert!(decided.plan.is_none());
}

#[test]
fn risky_plan_at_confirm_band_confirms() {
    let decided = decide(0.65, 1.0, false, true);
    assert!(matches!(decided.decision, ParseDecision::Confirm { .. }), "{:#?}", decided.decision);
    assert!(decided.commit.confirm.is_some());
}

#[test]
fn start_timer_without_duration_clarifies() {
    let home = default_home();
    let session = Session::new();
    let settings = Settings::pinned("en");
    let catalog = catalog_for(&["en".into()]);
    let context = ParseContext::new("Start the timer", &home, &session, &[], &settings, catalog);
    let plan = IntentPlan::from_intents(vec![Intent::new("HassStartTimer")], 1.0, &[]);
    let decided = safety_decision(execute_plan(&context, plan, "test", None, None, false, false), &context);
    assert!(matches!(decided.decision, ParseDecision::Clarify { .. }), "{:#?}", decided.decision);
    assert!(decided.plan.is_none());
}

#[test]
fn resume_named_timer_without_duration_executes() {
    let mut home = default_home();
    home.entities.push(crate::types::EntityRec {
        entity_id: "timer.oven".into(),
        name: "Oven".into(),
        domain: "timer".into(),
        platform: None,
        area: None,
        aliases: vec!["oven".into()],
        tags: Vec::new(),
    });
    let session = Session::new();
    let settings = Settings::pinned("en");
    let catalog = catalog_for(&["en".into()]);
    let context = ParseContext::new("Resume the oven timer", &home, &session, &[], &settings, catalog);
    let plan = IntentPlan::from_intents(vec![Intent::new("HassStartTimer").with("entity_id", "timer.oven")], 1.0, &[]);
    let decided = safety_decision(execute_plan(&context, plan, "test", None, None, false, false), &context);
    assert!(matches!(decided.decision, ParseDecision::Execute), "{:#?}", decided.decision);
    let name = decided.plan.as_ref().map(|plan| plan.intents()[0].name.clone());
    assert_eq!(name.as_deref(), Some("HassStartTimer"));
}

#[test]
fn lock_partial_fails_closed() {
    let home = default_home();
    let session = Session::new();
    let settings = Settings::default();
    let catalog = catalog_for(&["de".into()]);
    let context = ParseContext::new("test", &home, &session, &[], &settings, catalog);
    let plan = IntentPlan::from_intents(
        vec![Intent::new("HassTurnOn").with("entity_id", "light.wohnzimmer"), Intent::new("HassTurnOn").with("entity_id", "lock.missing")],
        1.0,
        &[],
    );
    let decided = safety_decision(execute_plan(&context, plan, "test", None, None, false, false), &context);
    assert!(matches!(decided.decision, ParseDecision::Reject { .. }), "{:#?}", decided.decision);
    assert!(decided.plan.is_none());
}
