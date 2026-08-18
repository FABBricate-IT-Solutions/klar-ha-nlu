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
