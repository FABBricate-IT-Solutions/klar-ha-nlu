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
    pub reject_fpr: f64,
    pub reject_fnr: f64,
    pub brier: f64,
    pub ece: f64,
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
    let mut brier_sum = 0.0;
    let mut ece_bins = [(0usize, 0.0, 0usize); 10];

    for (item, outcome) in items.iter().zip(outcomes) {
        let actual = match &outcome.decision {
            ParseDecision::Execute => outcome.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default(),
            _ => Vec::new(),
        };
        let predicted_names: BTreeSet<String> = actual.iter().map(|intent| intent.name.clone()).collect();
        let gold_names: BTreeSet<String> =
            item.expect_intents.as_ref().map(|intents| intents.iter().map(|intent| intent.name.clone()).collect()).unwrap_or_default();
        for name in predicted_names.union(&gold_names) {
            let counts = per_intent.entry(name.clone()).or_default();
            let pred = predicted_names.contains(name);
            let gold = gold_names.contains(name);
            counts.tp += usize::from(pred && gold);
            counts.fp += usize::from(pred && !gold);
            counts.fn_ += usize::from(!pred && gold);
        }

        if let Some(gold) = &item.expect_intents {
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
        clarify_tp += usize::from(predicted_clarify && item.expect_clarify);
        clarify_fp += usize::from(predicted_clarify && !item.expect_clarify);
        clarify_fn += usize::from(!predicted_clarify && item.expect_clarify);

        let predicted_reject = matches!(outcome.decision, ParseDecision::Reject { .. });
        if item.expect_reject {
            reject_pos += 1;
            reject_fn += usize::from(!predicted_reject);
        } else {
            reject_neg += 1;
            reject_fp += usize::from(predicted_reject);
        }

        let correct = decision_correct(item, outcome, &actual);
        let confidence = outcome.confidence.clamp(0.0, 1.0);
        brier_sum += (confidence - f64::from(u8::from(correct))).powi(2);
        let bin = ((confidence * 10.0).floor() as usize).min(9);
        ece_bins[bin].0 += 1;
        ece_bins[bin].1 += confidence;
        ece_bins[bin].2 += usize::from(correct);
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
        brier: if items.is_empty() { 0.0 } else { brier_sum / items.len() as f64 },
        ece: expected_calibration_error(&ece_bins, items.len()),
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

fn expected_calibration_error(bins: &[(usize, f64, usize)], total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    bins.iter()
        .map(|(count, conf_sum, correct)| {
            if *count == 0 {
                0.0
            } else {
                let acc = *correct as f64 / *count as f64;
                let conf = conf_sum / *count as f64;
                (*count as f64 / total as f64) * (acc - conf).abs()
            }
        })
        .sum()
}
