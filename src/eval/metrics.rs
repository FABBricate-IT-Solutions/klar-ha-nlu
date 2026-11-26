use crate::types::{ParseDecision, ParseOutcome};
use std::collections::{BTreeMap, BTreeSet};

use super::corpus::{EvalItem, GoldIntent, Split};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EvalMetrics {
    pub utterances: usize,
    pub intent_macro_f1: f64,
    pub slot_micro_f1: f64,
    pub intent_slot_pairing: f64,
    pub asr_recovery: f64,
    pub clarify_precision: f64,
    pub clarify_recall: f64,
    /// Share of in-domain items the engine rejected.
    pub reject_fpr: f64,
    /// Share of out-of-domain items that did not end in a reject (chat counts as a miss).
    pub reject_fnr: f64,
    /// Share of out-of-domain items the engine would have acted on or asked about.
    pub reject_unsafe: f64,
    /// Typo-split items recovered to the gold intent; reported and floored, see `MIN_TYPO_RECOVERY`.
    pub typo_recovery: f64,
    /// Exact decision accuracy over every item: the figure comparable to the Assist run.
    pub accuracy: f64,
    /// The same over in-domain items only, without credit for rejecting out-of-domain sentences.
    pub in_domain_accuracy: f64,
}

pub fn score_items(items: &[EvalItem], outcomes: &[ParseOutcome]) -> EvalMetrics {
    assert_eq!(items.len(), outcomes.len());
    let mut per_intent: BTreeMap<String, Counts> = BTreeMap::new();
    let mut slot_tp = 0usize;
    let mut slot_fp = 0usize;
    let mut slot_fn = 0usize;
    let mut pair_ok = 0usize;
    let mut pair_n = 0usize;
    let mut asr_ok = 0usize;
    let mut asr_n = 0usize;
    let mut clarify_tp = 0usize;
    let mut clarify_fp = 0usize;
    let mut clarify_fn = 0usize;
    let mut reject_fp = 0usize;
    let mut reject_fn = 0usize;
    let mut reject_neg = 0usize;
    let mut reject_pos = 0usize;
    let mut reject_unsafe = 0usize;
    let mut typo_ok = 0usize;
    let mut typo_n = 0usize;
    let mut correct_total = 0usize;
    let mut in_domain_n = 0usize;
    let mut in_domain_ok = 0usize;

    for (item, outcome) in items.iter().zip(outcomes) {
        let actual = match &outcome.decision {
            ParseDecision::Execute => outcome.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default(),
            _ => Vec::new(),
        };
        let predicted_names: BTreeSet<String> = actual.iter().map(|intent| intent.name.clone()).collect();
        let gold_names: BTreeSet<String> =
            item.expect_intents.as_ref().map(|intents| intents.iter().map(|intent| intent.name.clone()).collect()).unwrap_or_default();
        // Typo items have their own recovery figure and floor; they do not dilute the core F1 gates.
        let core = item.split != Split::Typo;
        for name in predicted_names.union(&gold_names).filter(|_| core) {
            let counts = per_intent.entry(name.clone()).or_default();
            let pred = predicted_names.contains(name);
            let gold = gold_names.contains(name);
            counts.tp += usize::from(pred && gold);
            counts.fp += usize::from(pred && !gold);
            counts.fn_ += usize::from(!pred && gold);
        }

        if let Some(gold) = item.expect_intents.as_ref().filter(|_| core) {
            pair_n += 1;
            if pairing_ok(gold, &actual) {
                pair_ok += 1;
            }
            score_slots(gold, &actual, &mut slot_tp, &mut slot_fp, &mut slot_fn);
        }

        if item.split == Split::Asr {
            asr_n += 1;
            if item.expect_intents.as_ref().is_some_and(|gold| pairing_ok(gold, &actual)) {
                asr_ok += 1;
            }
        }

        let predicted_clarify = matches!(outcome.decision, ParseDecision::Clarify { .. } | ParseDecision::Confirm { .. });
        // Asking about a mistyped name is the safe outcome, not a spurious clarification.
        clarify_tp += usize::from(core && predicted_clarify && item.expect_clarify);
        clarify_fp += usize::from(core && predicted_clarify && !item.expect_clarify);
        clarify_fn += usize::from(core && !predicted_clarify && item.expect_clarify);

        let predicted_reject = matches!(outcome.decision, ParseDecision::Reject { .. });
        if item.expect_reject {
            reject_pos += 1;
            reject_fn += usize::from(!predicted_reject);
            reject_unsafe += usize::from(matches!(
                outcome.decision,
                ParseDecision::Execute | ParseDecision::Clarify { .. } | ParseDecision::Confirm { .. }
            ));
        } else {
            reject_neg += 1;
            reject_fp += usize::from(predicted_reject);
        }

        let correct = decision_correct(item, outcome, &actual);
        correct_total += usize::from(correct);
        if item.in_domain() {
            in_domain_n += 1;
            in_domain_ok += usize::from(correct);
        }
        if item.split == Split::Typo {
            typo_n += 1;
            typo_ok += usize::from(correct);
        }
    }

    let intent_macro_f1 =
        if per_intent.is_empty() { 1.0 } else { per_intent.values().map(Counts::f1).sum::<f64>() / per_intent.len() as f64 };
    let slot_micro_f1 = f1(slot_tp, slot_fp, slot_fn);
    EvalMetrics {
        utterances: items.len(),
        intent_macro_f1,
        slot_micro_f1,
        intent_slot_pairing: ratio(pair_ok, pair_n),
        asr_recovery: ratio(asr_ok, asr_n),
        clarify_precision: precision(clarify_tp, clarify_fp),
        clarify_recall: recall(clarify_tp, clarify_fn),
        reject_fpr: ratio(reject_fp, reject_neg),
        reject_fnr: ratio(reject_fn, reject_pos),
        reject_unsafe: ratio_zero(reject_unsafe, reject_pos),
        typo_recovery: ratio(typo_ok, typo_n),
        accuracy: ratio(correct_total, items.len()),
        in_domain_accuracy: ratio(in_domain_ok, in_domain_n),
    }
}

#[derive(Default)]
struct Counts {
    tp: usize,
    fp: usize,
    fn_: usize,
}

impl Counts {
    fn f1(&self) -> f64 {
        f1(self.tp, self.fp, self.fn_)
    }
}

fn score_slots(gold: &[GoldIntent], actual: &[crate::types::Intent], slot_tp: &mut usize, slot_fp: &mut usize, slot_fn: &mut usize) {
    let paired = gold.len().max(actual.len());
    for index in 0..paired {
        let wanted = gold.get(index);
        let got = actual.get(index);
        match (wanted, got) {
            (Some(wanted), Some(got)) if wanted.name == got.name => {
                for (key, value) in &wanted.slots {
                    if got.slot(key) == Some(value.as_str()) {
                        *slot_tp += 1;
                    } else {
                        *slot_fn += 1;
                        *slot_fp += usize::from(got.slot(key).is_some());
                    }
                }
            }
            (Some(wanted), Some(got)) => {
                *slot_fn += wanted.slots.len().max(1);
                *slot_fp += got.slots.len().max(1);
            }
            (Some(wanted), None) => *slot_fn += wanted.slots.len().max(1),
            (None, Some(got)) => *slot_fp += got.slots.len().max(1),
            (None, None) => {}
        }
    }
}

fn pairing_ok(gold: &[GoldIntent], actual: &[crate::types::Intent]) -> bool {
    gold.len() == actual.len()
        && gold
            .iter()
            .zip(actual)
            .all(|(wanted, got)| wanted.name == got.name && wanted.slots.iter().all(|(key, value)| got.slot(key) == Some(value.as_str())))
}

fn decision_correct(item: &EvalItem, outcome: &ParseOutcome, actual: &[crate::types::Intent]) -> bool {
    if item.expect_reject {
        return matches!(outcome.decision, ParseDecision::Reject { .. });
    }
    if item.expect_clarify {
        return matches!(outcome.decision, ParseDecision::Clarify { .. } | ParseDecision::Confirm { .. });
    }
    matches!(outcome.decision, ParseDecision::Execute) && item.expect_intents.as_ref().is_some_and(|gold| pairing_ok(gold, actual))
}

fn f1(tp: usize, fp: usize, fn_: usize) -> f64 {
    let prec = precision(tp, fp);
    let rec = recall(tp, fn_);
    if prec + rec == 0.0 {
        0.0
    } else {
        2.0 * prec * rec / (prec + rec)
    }
}

fn precision(tp: usize, fp: usize) -> f64 {
    ratio(tp, tp + fp)
}

fn recall(tp: usize, fn_: usize) -> f64 {
    ratio(tp, tp + fn_)
}

fn ratio(num: usize, den: usize) -> f64 {
    if den == 0 {
        1.0
    } else {
        num as f64 / den as f64
    }
}

fn ratio_zero(num: usize, den: usize) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 / den as f64
    }
}
