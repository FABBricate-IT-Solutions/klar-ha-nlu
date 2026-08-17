use klar_nlu::home::default_home;
use klar_nlu::nlu::{parse_with_policies, safety_decide_policies};
use klar_nlu::session::Session;
use klar_nlu::types::{Intent, IntentPlan, ParseDecision, PolicyEffect, PolicyMatch, PolicyRule, RejectReason, Settings, SpeechBank};

fn block_ac() -> PolicyRule {
    PolicyRule {
        id: "block-ac".into(),
        enabled: true,
        label: "AC".into(),
        when: PolicyMatch { entity_id: Some("climate.schlafzimmer_ac".into()), ..PolicyMatch::default() },
        effect: PolicyEffect::Block,
        prefer: None,
        payload: None,
    }
}

#[test]
fn user_rule_blocks_named_climate() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings::default();
    let outcome = parse_with_policies("Klimaanlage aus", &home, &mut session, &[], &settings, &[block_ac()], &SpeechBank::default());
    assert!(matches!(outcome.decision, ParseDecision::Reject { reason: RejectReason::Unsafe }), "{:#?}", outcome.decision);
    assert!(outcome.plan.is_none());
    assert!(outcome.candidates.is_empty());
}

#[test]
fn allow_cannot_skip_area_lock() {
    let home = default_home();
    let settings = Settings::default();
    let rules = vec![PolicyRule {
        id: "allow-locks".into(),
        enabled: true,
        label: "locks".into(),
        when: PolicyMatch { domain: Some("lock".into()), ..PolicyMatch::default() },
        effect: PolicyEffect::Allow,
        prefer: None,
        payload: None,
    }];
    let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("area", "wohnzimmer").with("domain", "lock")], 0.9, &[]);
    let (decision, kept) = safety_decide_policies(&home, &settings, plan, 0.9, 1.0, false, (&rules, &SpeechBank::default()));
    assert!(matches!(decision, ParseDecision::Confirm { .. } | ParseDecision::Reject { .. }), "{:#?}", decision);
    if matches!(decision, ParseDecision::Confirm { .. }) {
        assert!(kept.is_some());
    }
}

#[test]
fn retrieval_absent_when_rag_off() {
    let home = default_home();
    let mut session = Session::new();
    let outcome = parse_with_policies("danke", &home, &mut session, &[], &Settings::default(), &[], &SpeechBank::default());
    assert!(matches!(outcome.decision, ParseDecision::Chat | ParseDecision::Reject { .. }), "{:#?}", outcome.decision);
    assert!(outcome.retrieval.is_none());
}

#[test]
fn phrase_reply_skips_intent() {
    let home = default_home();
    let mut session = Session::new();
    let rules = vec![PolicyRule {
        id: "night".into(),
        enabled: true,
        label: "Nacht".into(),
        when: PolicyMatch { phrase: Some("gute nacht".into()), ..PolicyMatch::default() },
        effect: PolicyEffect::Reply,
        prefer: None,
        payload: Some("Schlaf schön.".into()),
    }];
    let outcome = parse_with_policies("Gute Nacht", &home, &mut session, &[], &Settings::default(), &rules, &SpeechBank::default());
    assert!(matches!(outcome.decision, ParseDecision::Chat), "{:#?}", outcome.decision);
    assert_eq!(outcome.speech, "Schlaf schön.");
    assert!(outcome.plan.is_none());
    assert_eq!(outcome.policy_trace.as_ref().and_then(|trace| trace.hit.as_deref()), Some("reply"));
}

#[test]
fn phrase_script_emits_turn_on() {
    let home = default_home();
    let mut session = Session::new();
    let rules = vec![PolicyRule {
        id: "leave".into(),
        enabled: true,
        label: "Gehen".into(),
        when: PolicyMatch { phrase: Some("ich gehe".into()), ..PolicyMatch::default() },
        effect: PolicyEffect::Script,
        prefer: None,
        payload: Some("leaving_home".into()),
    }];
    let outcome = parse_with_policies("Ich gehe", &home, &mut session, &[], &Settings::default(), &rules, &SpeechBank::default());
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{:#?}", outcome.decision);
    let intent = &outcome.plan.as_ref().expect("plan").steps[0].intent;
    assert_eq!(intent.name, "HassTurnOn");
    assert_eq!(intent.slot("entity_id"), Some("script.leaving_home"));
}

#[test]
fn retrieval_on_chat_when_rag_enabled() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings { nlu_rag: true, ..Settings::default() };
    let outcome = parse_with_policies("danke", &home, &mut session, &[], &settings, &[], &SpeechBank::default());
    if matches!(outcome.decision, ParseDecision::Chat) {
        assert!(outcome.retrieval.is_some());
    }
}
