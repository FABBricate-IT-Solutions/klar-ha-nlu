//! 6.1 acceptance: ≥99% of held-out general (OOD) questions reject in a way that
//! reaches the HA fallback chain (default agent, then LLM). Reasons that stay
//! with Klar are empty_input, invalid_input, and unsafe — matching
//! `continues_after_reject` in the integration.

use klar_nlu::eval::{evaluate_corpus, family_home, heldout_dir, load_corpus, Split};
use klar_nlu::types::{ParseDecision, RejectReason, Settings};

fn continues_after_reject(decision: &ParseDecision) -> bool {
    match decision {
        ParseDecision::Reject { reason } => !matches!(reason, RejectReason::EmptyInput | RejectReason::InvalidInput | RejectReason::Unsafe),
        ParseDecision::Chat => true,
        _ => false,
    }
}

fn assert_ood_fallback_rate(language: &str) {
    let home = family_home(language).expect("family home");
    let items = load_corpus(&heldout_dir(language), language).expect("held-out corpus");
    let ood: Vec<_> = items.into_iter().filter(|item| item.split == Split::Ood).collect();
    assert!(ood.len() >= 100, "{language} OOD {} < 100", ood.len());
    let settings = Settings { languages: vec![language.into()], ..Settings::default() };
    let (_, outcomes) = evaluate_corpus(&ood, &home, &settings);
    let total = outcomes.len();
    let routed = outcomes.iter().filter(|outcome| continues_after_reject(&outcome.decision)).count();
    let rate = routed as f64 / total as f64;
    assert!(rate + 1e-12 >= 0.99, "{language} held-out OOD fallback routing {:.2}% < 99% ({routed}/{total})", rate * 100.0);
}

#[test]
fn held_out_english_ood_reaches_fallback() {
    assert_ood_fallback_rate("en");
}

#[test]
fn held_out_german_ood_reaches_fallback() {
    assert_ood_fallback_rate("de");
}
