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
fn retrieval_on_chat_when_rag_enabled() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings { nlu_rag: true, ..Settings::default() };
    let outcome = parse_with_policies("danke", &home, &mut session, &[], &settings, &[], &SpeechBank::default());
    if matches!(outcome.decision, ParseDecision::Chat) {
        assert!(outcome.retrieval.is_some());
    }
}
