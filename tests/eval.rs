use klar_nlu::eval::{
    evaluate_dir, family_home, gate_scorecard, heldout_dir, run_scorecard, write_scorecard, MAX_REJECT_FNR, MAX_REJECT_FPR,
    MAX_REJECT_UNSAFE, MIN_ASR_RECOVERY, MIN_CLARIFY_PRECISION, MIN_HELD_OUT, MIN_INTENT_MACRO_F1, MIN_IN_DOMAIN, MIN_PAIRING,
    MIN_SLOT_MICRO_F1, MIN_TYPO_RECOVERY,
};
use std::path::PathBuf;

fn assert_quality(language: &str) {
    let home = family_home(language).expect("family home");
    let (items, metrics) = evaluate_dir(&heldout_dir(language), &home, language).expect("corpus");
    assert!(items.len() >= MIN_HELD_OUT, "{language} held-out {} < {MIN_HELD_OUT}", items.len());
    let in_domain = items.iter().filter(|item| item.in_domain()).count();
    assert!(in_domain >= MIN_IN_DOMAIN, "{language} in-domain held-out {in_domain} < {MIN_IN_DOMAIN}");
    assert!(
        in_domain * 100 >= items.len() * 40,
        "{language} held-out is {} % in-domain; out-of-domain rejects must not dominate",
        in_domain * 100 / items.len()
    );
    assert!(items.iter().any(|item| item.split == klar_nlu::eval::Split::Typo), "{language} missing typo split");
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
    assert!(metrics.reject_fnr <= MAX_REJECT_FNR + 1e-9, "{language} reject miss rate {}", metrics.reject_fnr);
    assert!(metrics.reject_fpr <= MAX_REJECT_FPR + 1e-9, "{language} in-domain reject rate {}", metrics.reject_fpr);
    assert!(metrics.reject_unsafe <= MAX_REJECT_UNSAFE + 1e-9, "{language} out-of-domain acted on {}", metrics.reject_unsafe);
    assert!(metrics.typo_recovery + 1e-9 >= MIN_TYPO_RECOVERY, "{language} typo recovery {}", metrics.typo_recovery);
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
    assert_eq!(parsed.schema_version, "m7.2");
    assert_eq!(parsed.languages.len(), 2);
    assert!(parsed.languages.iter().all(|card| card.in_domain >= MIN_IN_DOMAIN), "{:?}", parsed.languages);
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
        news_topic: None,
        retrieval: None,
        policy_trace: None,
        quiet_ack_eligible: false,
        clock_answer: false,
        refine_band: None,
        teach: None,
    };
    let metrics = score_items(&[item], &[outcome]);
    assert!(metrics.intent_macro_f1 < 0.5, "{}", metrics.intent_macro_f1);
    assert!(metrics.intent_slot_pairing < 0.5, "{}", metrics.intent_slot_pairing);
    assert!(metrics.slot_micro_f1 < 0.5, "slot F1 must not be vacuous: {}", metrics.slot_micro_f1);
}

#[test]
fn scorecard_has_no_uncalibrated_confidence_field() {
    // `ParseOutcome.confidence` is a ranking score behind the decision bands, not a probability, and a reject
    // carries 0.0. Scoring it as one gave ECE 0.89. The figure is gone from the scorecard instead of being tuned.
    let card = run_scorecard(None).expect("scorecard");
    let json = serde_json::to_value(&card).expect("json");
    let text = json.to_string();
    for banned in ["ece", "brier", "confidence", "calibration"] {
        assert!(!text.contains(&format!("\"{banned}\"")), "scorecard still exposes {banned}: {text}");
    }
    for language in json["languages"].as_array().expect("languages") {
        for key in ["reject_fpr", "reject_fnr", "reject_unsafe", "typo_recovery", "accuracy"] {
            assert!(language["metrics"][key].is_number(), "metrics.{key} missing");
        }
    }
}

#[test]
fn scorecard_is_stamped_with_the_commit() {
    std::env::set_var("KLAR_COMMIT_SHA", "0123456789abcdef0123456789abcdef01234567");
    let card = run_scorecard(None).expect("scorecard");
    assert_eq!(card.commit, "0123456789abcdef0123456789abcdef01234567");
}

#[test]
fn gate_fails_on_reject_errors_and_thin_in_domain_corpus() {
    let card = run_scorecard(None).expect("scorecard");
    gate_scorecard(&card).expect("baseline passes");

    let mut missed = card.clone();
    missed.languages[0].metrics.reject_fnr = MAX_REJECT_FNR + 0.01;
    assert!(gate_scorecard(&missed).expect_err("reject miss rate is gated").contains("reject miss rate"));

    let mut over = card.clone();
    over.languages[1].metrics.reject_fpr = MAX_REJECT_FPR + 0.01;
    assert!(gate_scorecard(&over).expect_err("in-domain reject rate is gated").contains("in-domain reject rate"));

    let mut acted = card.clone();
    acted.languages[0].metrics.reject_unsafe = 0.01;
    assert!(gate_scorecard(&acted).expect_err("acting outside the home is gated").contains("acted on"));

    let mut thin = card.clone();
    thin.languages[1].in_domain = MIN_IN_DOMAIN - 1;
    assert!(gate_scorecard(&thin).expect_err("thin corpus is gated").contains("in-domain utterances"));

    let mut typos = card;
    typos.languages[0].metrics.typo_recovery = MIN_TYPO_RECOVERY - 0.01;
    assert!(gate_scorecard(&typos).expect_err("typo floor is gated").contains("typo recovery"));
}
