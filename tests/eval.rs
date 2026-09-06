use klar_nlu::eval::{
    evaluate_dir, family_home, gate_scorecard, heldout_dir, run_scorecard, write_scorecard, MIN_ASR_RECOVERY, MIN_CLARIFY_PRECISION,
    MIN_HELD_OUT, MIN_INTENT_MACRO_F1, MIN_PAIRING, MIN_SLOT_MICRO_F1,
};
use std::path::PathBuf;

fn assert_quality(language: &str) {
    let home = family_home(language).expect("family home");
    let (items, metrics) = evaluate_dir(&heldout_dir(language), &home, language).expect("corpus");
    assert!(items.len() >= MIN_HELD_OUT, "{language} held-out {} < {MIN_HELD_OUT}", items.len());
    assert!(items.iter().any(|item| item.split == klar_nlu::eval::Split::Asr), "{language} missing ASR split");
    assert!(items.iter().any(|item| item.split == klar_nlu::eval::Split::Ood), "{language} missing OOD split");
    assert!(items.iter().any(|item| item.split == klar_nlu::eval::Split::Clarify), "{language} missing clarify split");
    assert!(items.iter().any(|item| item.split == klar_nlu::eval::Split::Multi), "{language} missing multi split");
    assert!(items.iter().any(|item| item.split == klar_nlu::eval::Split::Adversarial), "{language} missing adversarial split");
    if metrics.intent_macro_f1 + 1e-9 < MIN_INTENT_MACRO_F1
        || metrics.slot_micro_f1 + 1e-9 < MIN_SLOT_MICRO_F1
        || metrics.intent_slot_pairing + 1e-9 < MIN_PAIRING
    {
        let settings = klar_nlu::types::Settings { languages: vec![language.into()], ..klar_nlu::types::Settings::default() };
        let (_, outcomes) = klar_nlu::eval::evaluate_corpus(&items, &home, &settings);
        let misses: Vec<String> = items
            .iter()
            .zip(&outcomes)
            .filter(|(item, outcome)| {
                let actual = match &outcome.decision {
                    klar_nlu::types::ParseDecision::Execute => outcome.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default(),
                    _ => Vec::new(),
                };
                let reject_ok = item.expect_reject && matches!(outcome.decision, klar_nlu::types::ParseDecision::Reject { .. });
                let clarify_ok = item.expect_clarify
                    && matches!(
                        outcome.decision,
                        klar_nlu::types::ParseDecision::Clarify { .. } | klar_nlu::types::ParseDecision::Confirm { .. }
                    );
                let exec_ok = matches!(outcome.decision, klar_nlu::types::ParseDecision::Execute)
                    && item.expect_intents.as_ref().is_some_and(|gold| {
                        gold.len() == actual.len()
                            && gold.iter().zip(&actual).all(|(wanted, got)| {
                                wanted.name == got.name && wanted.slots.iter().all(|(key, value)| got.slot(key) == Some(value.as_str()))
                            })
                    });
                !(reject_ok || clarify_ok || exec_ok)
            })
            .map(|(item, outcome)| {
                format!("{:?} {:?} {:?}", item.turns, outcome.decision, outcome.plan.as_ref().map(|plan| plan.intents()))
            })
            .collect();
        let dump = format!("/tmp/klar_eval_misses_{language}.txt");
        std::fs::write(&dump, misses.join("\n")).ok();
        eprintln!("{language} misses {} written to {dump}", misses.len());
    }
    assert!(metrics.intent_macro_f1 + 1e-9 >= MIN_INTENT_MACRO_F1, "{language} intent F1 {}", metrics.intent_macro_f1);
    assert!(metrics.slot_micro_f1 + 1e-9 >= MIN_SLOT_MICRO_F1, "{language} slot F1 {}", metrics.slot_micro_f1);
    assert!(metrics.intent_slot_pairing + 1e-9 >= MIN_PAIRING, "{language} pairing {}", metrics.intent_slot_pairing);
    assert!(metrics.asr_recovery + 1e-9 >= MIN_ASR_RECOVERY, "{language} ASR {}", metrics.asr_recovery);
    assert!(metrics.clarify_precision + 1e-9 >= MIN_CLARIFY_PRECISION, "{language} clarify P {}", metrics.clarify_precision);
}

#[test]
fn held_out_english_meets_scorecard_gates() {
    assert_quality("en");
}

#[test]
fn held_out_german_meets_scorecard_gates() {
    assert_quality("de");
}

#[test]
fn scorecard_json_includes_bench_and_assist_same_graph() {
    let card = run_scorecard(None).expect("scorecard");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/eval_scorecard.json");
    write_scorecard(&path, &card).expect("write");
    let parsed: klar_nlu::eval::Scorecard = serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("json");
    assert_eq!(parsed.schema_version, "m7.1");
    assert_eq!(parsed.languages.len(), 2);
    assert!(parsed.bench.samples >= 8);
    assert!(parsed.comparison.len() >= 2, "{:?}", parsed.comparison);
    assert!(parsed.comparison.iter().all(|row| row.ok == row.cases && row.cases > 0), "{:?}", parsed.comparison);
    gate_scorecard(&parsed).expect("release gate");
}

#[test]
fn metrics_are_not_vacuous_on_mismatched_gold() {
    use klar_nlu::eval::{score_items, EvalItem, GoldIntent, Split};
    use klar_nlu::types::{Intent, IntentPlan, ParseDecision, ParseOutcome};
    let item = EvalItem {
        name: "mismatch".into(),
        split: Split::Control,
        language: "en".into(),
        turns: vec!["x".into()],
        expect_intents: Some(vec![GoldIntent { name: "HassTurnOn".into(), slots: [("entity_id".into(), "light.a".into())].into() }]),
        expect_reject: false,
        expect_clarify: false,
    };
    let outcome = ParseOutcome {
        schema_version: "2.0".into(),
        text: "x".into(),
        conversation_id: "t".into(),
        decision: ParseDecision::Execute,
        speech: String::new(),
        confidence: 0.9,
        margin: 1.0,
        selected_candidate_id: None,
        candidates: Vec::new(),
        plan: Some(IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("entity_id", "light.b")], 0.9, &[])),
        evidence: Vec::new(),
        trace: Default::default(),
        briefing: false,
        retrieval: None,
        policy_trace: None,
        quiet_ack_eligible: false,
    };
    let metrics = score_items(&[item], &[outcome]);
    assert!(metrics.intent_macro_f1 < 0.5, "{}", metrics.intent_macro_f1);
    assert!(metrics.intent_slot_pairing < 0.5, "{}", metrics.intent_slot_pairing);
    assert!(metrics.slot_micro_f1 < 0.5, "slot F1 must not be vacuous: {}", metrics.slot_micro_f1);
}
