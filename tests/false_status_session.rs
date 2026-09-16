use klar_nlu::home::default_home;
use klar_nlu::nlu;
use klar_nlu::session::Session;
use klar_nlu::types::{Intent, ParseDecision, Settings};

fn plan_names(outcome: &klar_nlu::types::ParseOutcome) -> Vec<String> {
    outcome.plan.as_ref().map(|plan| plan.steps.iter().map(|step| step.intent.name.clone()).collect()).unwrap_or_default()
}

#[test]
fn mid_sentence_ist_and_ein_do_not_replay_last_light() {
    let home = default_home();
    let mut session = Session::new();
    session.preferred_area = Some("wohnzimmer".into());
    session.remember(&Intent::new("HassTurnOff").with("entity_id", "light.wohnzimmer"));

    let text = "Habe ich jetzt nicht mehr ihre Arbeit da hinten gemacht, wieder? Okay, da ist extra das Tat und ein bisschen verschwunden, dass sie da hinschweissen könnte.";
    let outcome = nlu::parse(text, &home, &mut session, &[], &Settings::pinned("de"));
    assert!(
        !matches!(outcome.decision, ParseDecision::Execute),
        "expected reject/chat, got decision={:?} names={:?}",
        outcome.decision,
        plan_names(&outcome)
    );
    assert!(plan_names(&outcome).is_empty(), "expected no plan, got {:?}", plan_names(&outcome));
}

#[test]
fn explicit_status_still_replays_last_light() {
    let home = default_home();
    let mut session = Session::new();
    session.remember(&Intent::new("HassTurnOn").with("entity_id", "light.wohnzimmer"));
    let outcome = nlu::parse("wie ist der Status", &home, &mut session, &[], &Settings::pinned("de"));
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{:#?}", outcome);
    assert!(plan_names(&outcome).iter().any(|name| name == "HassGetState"), "{:#?}", outcome);
}

#[test]
fn ist_das_licht_an_still_works() {
    let home = default_home();
    let mut session = Session::new();
    let outcome = nlu::parse("Ist das Licht im Wohnzimmer an", &home, &mut session, &[], &Settings::pinned("de"));
    assert!(plan_names(&outcome).iter().any(|name| name == "HassGetState"), "{:#?}", outcome);
}

#[test]
fn trailing_ein_still_turns_light_on() {
    let home = default_home();
    let mut session = Session::new();
    let outcome = nlu::parse("Wohnzimmerlicht ein", &home, &mut session, &[], &Settings::pinned("de"));
    assert!(plan_names(&outcome).iter().any(|name| name == "HassTurnOn"), "{:#?}", outcome);
}

#[test]
fn schalte_ein_still_works() {
    let home = default_home();
    let mut session = Session::new();
    let outcome = nlu::parse("schalte das Licht im Wohnzimmer ein", &home, &mut session, &[], &Settings::pinned("de"));
    assert!(plan_names(&outcome).iter().any(|name| name == "HassTurnOn"), "{:#?}", outcome);
}
