use crate::parse::action::Action;
use crate::parse::calendar::calendar_clause;
use crate::parse::policy::{candidate, ClauseCandidate, PolicyId};
use crate::parse::slots::{laundry_switch_clause, timer_clause, ClauseOut};
use crate::types::HomeGraph;

type EarlyFn = fn(&[String], &HomeGraph, Action, Option<i32>, Option<&str>) -> Option<ClauseOut>;

const EARLY: &[(PolicyId, EarlyFn)] =
    &[(PolicyId::LaundrySwitch, laundry_switch_clause), (PolicyId::Timer, timer_clause), (PolicyId::Calendar, calendar_clause)];

pub(crate) fn early_special_clauses(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    early: Action,
    number: Option<i32>,
    domain: Option<&str>,
) -> Vec<ClauseCandidate> {
    let mut candidates = Vec::new();
    for (policy, evaluate) in EARLY {
        let input = if *policy == PolicyId::Calendar { raw } else { tokens };
        if let Some(outcome) = evaluate(input, home, early, number, domain) {
            candidates.push(candidate(*policy, early, outcome));
        }
    }
    candidates
}
