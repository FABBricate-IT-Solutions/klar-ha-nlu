use klar_nlu::home::default_home;
use klar_nlu::nlu::{parse, safety_decide_policies};
use klar_nlu::session::Session;
use klar_nlu::types::{
    Intent, IntentPlan, ParseDecision, PolicyEffect, PolicyMatch, PolicyRule, Settings, SpeechBank, SEED_BLOCK_AREA_LOCK,
    SEED_CONFIRM_COVER_CLOSE, SEED_CONFIRM_LOCK,
};

fn lock_plan() -> IntentPlan {
    IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("entity_id", "lock.wohnungstuer").with("domain", "lock")], 0.95, &[])
}

fn cover_plan() -> IntentPlan {
    IntentPlan::from_intents(
        vec![Intent::new("HassTurnOff").with("entity_id", "cover.wohnzimmer_rollo").with("domain", "cover")],
        0.95,
        &[],
    )
}

#[test]
fn lock_and_cover_seeds_are_visible_on_every_pinned_locale() {
    let home = default_home();
    for lang in ["de", "en", "ja", "ar", "pt-BR", "de-CH", "zh-CN"] {
        let settings = Settings::pinned(lang);
        let (lock_decision, _) = safety_decide_policies(&home, &settings, lock_plan(), 0.95, 1.0, false, (&[], &SpeechBank::default()));
        assert!(matches!(lock_decision, ParseDecision::Confirm { .. }), "{lang} lock {lock_decision:#?}");
        let (cover_decision, _) = safety_decide_policies(&home, &settings, cover_plan(), 0.95, 1.0, false, (&[], &SpeechBank::default()));
        assert!(matches!(cover_decision, ParseDecision::Confirm { .. }), "{lang} cover {cover_decision:#?}");
    }
}

#[test]
fn german_lock_parse_sets_seed_and_keeps_confirm() {
    let home = default_home();
    let outcome = parse("Wohnungstür abschließen", &home, &mut Session::new(), &[], &Settings::pinned("de"));
    assert!(matches!(outcome.decision, ParseDecision::Confirm { .. }), "{outcome:#?}");
    let trace = outcome.policy_trace.as_ref().expect("policy_trace");
    assert_eq!(trace.seed.as_ref().map(|layer| layer.id.as_str()), Some(SEED_CONFIRM_LOCK));
    assert!(trace.compiled_risky);
    assert_eq!(trace.band.as_deref(), Some("confirm"));
}

#[test]
fn english_cover_close_sets_cover_seed() {
    let home = default_home();
    let (decision, _) =
        safety_decide_policies(&home, &Settings::pinned("en"), cover_plan(), 0.95, 1.0, false, (&[], &SpeechBank::default()));
    assert!(matches!(decision, ParseDecision::Confirm { .. }), "{decision:#?}");
}

#[test]
fn house_override_hides_seed_without_changing_lock_band() {
    let home = default_home();
    let house = vec![PolicyRule {
        id: SEED_CONFIRM_LOCK.into(),
        enabled: false,
        label: "off".into(),
        when: PolicyMatch { domain: Some("lock".into()), ..PolicyMatch::default() },
        effect: PolicyEffect::Confirm,
        prefer: None,
        payload: None,
    }];
    let outcome = klar_nlu::nlu::parse_with_policies(
        "Wohnungstür abschließen",
        &home,
        &mut Session::new(),
        &[],
        &Settings::pinned("de"),
        &house,
        &SpeechBank::default(),
    );
    assert!(matches!(outcome.decision, ParseDecision::Confirm { .. }), "{outcome:#?}");
    let trace = outcome.policy_trace.as_ref().expect("policy_trace");
    assert!(trace.seed.is_none(), "{trace:#?}");
    assert!(trace.compiled_risky);
}

#[test]
fn area_lock_seed_is_block_but_floor_still_confirms() {
    let home = default_home();
    let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("area", "wohnzimmer").with("domain", "lock")], 0.9, &[]);
    let (decision, _) = safety_decide_policies(&home, &Settings::pinned("de"), plan, 0.9, 1.0, false, (&[], &SpeechBank::default()));
    assert!(matches!(decision, ParseDecision::Confirm { .. } | ParseDecision::Reject { .. }), "{decision:#?}");
    let _ = SEED_BLOCK_AREA_LOCK;
    let _ = SEED_CONFIRM_COVER_CLOSE;
}
