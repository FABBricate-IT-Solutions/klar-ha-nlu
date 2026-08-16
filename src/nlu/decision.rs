use crate::types::{EntityRec, Evidence, Intent, IntentCandidate, IntentPlan, MAX_CLARIFY_OPTIONS};
use std::collections::BTreeSet;

/// Minimum confidence for an executable plan. Below this, the engine clarifies or rejects.
pub const EXECUTE_MIN_CONFIDENCE: f64 = 0.80;
/// Solo plans get margin 1.0; competing complete plans must beat this gap to execute.
pub const EXECUTE_MIN_MARGIN: f64 = 0.05;
/// Risky actions may be offered for confirmation only at or above this score.
pub const CONFIRM_MIN_CONFIDENCE: f64 = 0.62;
/// Below this, the engine rejects instead of guessing.
pub const CLARIFY_MIN_CONFIDENCE: f64 = 0.70;
/// Two complete same-intent plans closer than this must clarify, not pick first.
pub const COMPETING_PLAN_MARGIN: f64 = 0.05;

const INFERRED_CAP: f64 = 0.86;
const FALLBACK_CAP: f64 = 0.68;
const SESSION_FLOOR: f64 = 0.80;
const SESSION_CAP: f64 = 0.80;
const FUZZY_CAP: f64 = 0.88;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PolicyBand {
    Execute,
    Confirm,
    Clarify,
    Reject,
}

pub(super) fn decide_band(confidence: f64, margin: f64, risky: bool, confirmed: bool, competing: bool) -> PolicyBand {
    if competing && margin < COMPETING_PLAN_MARGIN && confidence >= CLARIFY_MIN_CONFIDENCE {
        return PolicyBand::Clarify;
    }
    if risky && !confirmed {
        return if confidence >= CONFIRM_MIN_CONFIDENCE { PolicyBand::Confirm } else { PolicyBand::Reject };
    }
    if confidence < CLARIFY_MIN_CONFIDENCE {
        return PolicyBand::Reject;
    }
    if confidence >= EXECUTE_MIN_CONFIDENCE && (!competing || margin >= EXECUTE_MIN_MARGIN) {
        PolicyBand::Execute
    } else {
        PolicyBand::Clarify
    }
}

pub(super) fn competing_plans_need_clarify(margin: f64, same_intent_names: bool, distinct_targets: bool, target_margin: f64) -> bool {
    same_intent_names && distinct_targets && margin < COMPETING_PLAN_MARGIN && target_margin < COMPETING_PLAN_MARGIN
}

pub(super) fn complete_plans_compete(selected_policy: &str, runner_policy: &str, selected: &IntentPlan, runner: &IntentPlan) -> bool {
    !session_replay_is_not_a_competitor(selected_policy, runner_policy)
        && same_intent_names(selected, runner)
        && distinct_plan_targets(selected, runner)
}

pub(super) fn ranking_plans_compete(selected: &IntentCandidate, runner: &IntentCandidate) -> bool {
    complete_plans_compete(&selected.policy, &runner.policy, &selected.plan, &runner.plan)
        && best_plan_target_score(&selected.plan) - best_plan_target_score(&runner.plan) < COMPETING_PLAN_MARGIN
}

pub(super) fn same_intent_names(left: &IntentPlan, right: &IntentPlan) -> bool {
    left.steps.iter().map(|step| &step.intent.name).eq(right.steps.iter().map(|step| &step.intent.name))
}

pub(super) fn distinct_plan_targets(left: &IntentPlan, right: &IntentPlan) -> bool {
    distinct_slot_targets(left, right, "entity_id")
        || distinct_slot_targets(left, right, "area")
        || distinct_slot_targets(left, right, "floor")
}

fn distinct_slot_targets(left: &IntentPlan, right: &IntentPlan, slot: &str) -> bool {
    let left: Vec<_> = left.steps.iter().filter_map(|step| step.intent.slot(slot)).collect();
    let right: Vec<_> = right.steps.iter().filter_map(|step| step.intent.slot(slot)).collect();
    !left.is_empty() && left.len() == right.len() && left != right
}

pub(super) fn best_plan_target_score(plan: &IntentPlan) -> f64 {
    plan.steps.iter().map(|step| best_target_score(&step.evidence)).fold(0.0, f64::max)
}

pub(super) fn best_target_score(evidence: &[Evidence]) -> f64 {
    evidence.iter().filter(|item| item.kind.starts_with("target_")).map(|item| item.score).fold(0.0, f64::max)
}

pub(super) fn plan_targets(plan: &IntentPlan) -> Vec<String> {
    plan.steps
        .iter()
        .flat_map(|step| [step.intent.slot("entity_id"), step.intent.slot("area"), step.intent.slot("floor")])
        .flatten()
        .map(str::to_string)
        .collect()
}

pub(super) fn capped_options(mut options: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    options.retain(|option| seen.insert(option.clone()));
    options.truncate(MAX_CLARIFY_OPTIONS);
    options
}

pub(super) fn penalize_incomplete_area_coverage(score: f64, intents: &[Intent], areas: &[String], _entities: &[EntityRec]) -> f64 {
    if areas.len() < 2 || intents.len() >= areas.len() {
        score
    } else {
        (score - 0.08).max(0.0)
    }
}

pub(super) fn session_replay_is_not_a_competitor(selected_policy: &str, runner_up_policy: &str) -> bool {
    is_session_policy(selected_policy) != is_session_policy(runner_up_policy)
}

fn is_session_policy(policy: &str) -> bool {
    policy.split('+').any(|part| part.starts_with("session_"))
}

pub(super) fn calibrate_step_confidence(raw: f64, policy: &str, action: &Evidence, evidence: &[Evidence]) -> f64 {
    let session = is_session_evidence(policy, evidence);
    let cap = evidence_confidence_cap(policy, action, evidence);
    let mut score = raw.min(cap);
    if session {
        score = score.max(SESSION_FLOOR).min(cap);
    }
    score.clamp(0.0, 1.0)
}

fn is_strong_policy(policy: &str) -> bool {
    matches!(
        policy,
        "media"
            | "timer"
            | "list"
            | "laundry_switch"
            | "named_scene"
            | "all_lights"
            | "area_command"
            | "floor_command"
            | "preferred_area_command"
            | "follow_named"
            | "query_area"
            | "multi_area"
    )
}

fn is_session_evidence(policy: &str, evidence: &[Evidence]) -> bool {
    policy.starts_with("session_") || evidence.iter().any(|item| item.source.contains("session"))
}

fn evidence_confidence_cap(policy: &str, action: &Evidence, evidence: &[Evidence]) -> f64 {
    let session = is_session_evidence(policy, evidence);
    if (policy.contains("leftover") || policy == "fallback_cover") && !session {
        return FALLBACK_CAP;
    }
    if session {
        return SESSION_CAP;
    }
    if !is_strong_policy(policy) && (action.source == "context_inference" || evidence.iter().any(|item| item.source == "context_inference"))
    {
        return INFERRED_CAP;
    }
    if action.source.contains("fuzzy") || evidence.iter().any(|item| item.source.contains("fuzzy")) {
        return FUZZY_CAP;
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Evidence;

    fn action(source: &str, exact: bool, score: f64) -> Evidence {
        Evidence { kind: "action".into(), source: source.into(), value: "on".into(), score, exact }
    }

    fn target(exact: bool) -> Evidence {
        Evidence {
            kind: "target_entity".into(),
            source: if exact { "resolver_exact" } else { "resolver_fuzzy" }.into(),
            value: "light.x".into(),
            score: if exact { 1.0 } else { 0.87 },
            exact,
        }
    }

    #[test]
    fn threshold_matrix_changes_the_band() {
        assert_eq!(decide_band(0.93, 1.0, false, false, false), PolicyBand::Execute);
        assert_eq!(decide_band(0.88, 1.0, false, false, false), PolicyBand::Execute);
        assert_eq!(decide_band(0.75, 1.0, false, false, false), PolicyBand::Clarify);
        assert_eq!(decide_band(0.69, 1.0, false, false, false), PolicyBand::Reject);
        assert_eq!(decide_band(0.93, 0.02, false, false, true), PolicyBand::Clarify);
        assert_eq!(decide_band(0.93, 0.08, false, false, true), PolicyBand::Execute);
        assert_eq!(decide_band(0.90, 1.0, true, false, false), PolicyBand::Confirm);
        assert_eq!(decide_band(0.90, 1.0, true, true, false), PolicyBand::Execute);
        assert_eq!(decide_band(0.65, 1.0, true, false, false), PolicyBand::Confirm);
        assert_eq!(decide_band(0.61, 1.0, true, false, false), PolicyBand::Reject);
        assert_eq!(decide_band(0.60, 1.0, true, false, false), PolicyBand::Reject);
        assert_eq!(decide_band(0.79, 1.0, false, false, false), PolicyBand::Clarify);
        assert_eq!(decide_band(0.80, 0.04, false, false, false), PolicyBand::Execute);
        assert_eq!(decide_band(0.80, 0.04, false, false, true), PolicyBand::Clarify);
        assert_eq!(decide_band(0.80, 0.05, false, false, true), PolicyBand::Execute);
    }

    #[test]
    fn competing_margin_is_not_cosmetic() {
        assert!(competing_plans_need_clarify(0.02, true, true, 0.01));
        assert!(!competing_plans_need_clarify(0.05, true, true, 0.01));
        assert!(!competing_plans_need_clarify(0.001, true, false, 0.01));
        assert!(!competing_plans_need_clarify(0.001, false, true, 0.01));
        assert!(!competing_plans_need_clarify(0.02, true, true, 0.06));
        let living = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("area", "wohnzimmer").with("domain", "light")], 0.9, &[]);
        let kitchen = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("area", "kuche").with("domain", "light")], 0.88, &[]);
        let floors = IntentPlan::from_intents(
            vec![
                Intent::new("HassTurnOn").with("floor", "upper").with("domain", "light"),
                Intent::new("HassTurnOn").with("floor", "ground").with("domain", "light"),
            ],
            0.9,
            &[],
        );
        let other_floors = IntentPlan::from_intents(
            vec![
                Intent::new("HassTurnOn").with("floor", "upper").with("domain", "light"),
                Intent::new("HassTurnOn").with("floor", "lower").with("domain", "light"),
            ],
            0.88,
            &[],
        );
        assert!(complete_plans_compete("area_command", "area_command", &living, &kitchen));
        assert!(complete_plans_compete("floor_command", "floor_command", &floors, &other_floors));
        assert!(complete_plans_compete("multi_area", "multi_area", &floors, &other_floors));
        let richer = IntentPlan::from_intents(
            vec![
                Intent::new("HassTurnOff").with("entity_id", "light.powder"),
                Intent::new("HassTurnOn").with("entity_id", "scene.morning"),
            ],
            0.9,
            &[],
        );
        let area_and_scene = IntentPlan::from_intents(
            vec![
                Intent::new("HassTurnOff").with("area", "powder_room").with("domain", "light"),
                Intent::new("HassTurnOn").with("entity_id", "scene.morning"),
            ],
            0.88,
            &[],
        );
        assert!(!complete_plans_compete("grounded_entities+named_scene", "grounded_areas+named_scene", &richer, &area_and_scene));
        assert!(!complete_plans_compete("grounded_entities", "session_entities", &living, &kitchen));
        let close = IntentCandidate {
            id: "a".into(),
            plan: living.clone(),
            score: 0.90,
            margin: 0.02,
            policy: "area_command".into(),
            precedence: 0,
            evidence: Vec::new(),
        };
        let runner = IntentCandidate {
            id: "b".into(),
            plan: kitchen.clone(),
            score: 0.88,
            margin: 1.0,
            policy: "area_command".into(),
            precedence: 0,
            evidence: Vec::new(),
        };
        assert!(ranking_plans_compete(&close, &runner));
        let mut far_plan = kitchen.clone();
        far_plan.steps[0].evidence.push(Evidence {
            kind: "target_area".into(),
            source: "resolver_exact".into(),
            value: "kuche".into(),
            score: 0.80,
            exact: false,
        });
        let mut selected_plan = living.clone();
        selected_plan.steps[0].evidence.push(Evidence {
            kind: "target_area".into(),
            source: "resolver_exact".into(),
            value: "wohnzimmer".into(),
            score: 1.0,
            exact: true,
        });
        let far = IntentCandidate { plan: selected_plan, ..close };
        let weak = IntentCandidate { plan: far_plan, ..runner };
        assert!(!ranking_plans_compete(&far, &weak));
        assert!(session_replay_is_not_a_competitor("grounded_entities", "session_entities"));
        assert!(session_replay_is_not_a_competitor("session_areas", "area_command"));
        assert!(!session_replay_is_not_a_competitor("grounded_entities", "follow_named"));
        assert!(!session_replay_is_not_a_competitor("session_entities", "session_areas"));
    }

    #[test]
    fn evidence_caps_rank_exact_above_fuzzy_session_and_inferred() {
        let exact = calibrate_step_confidence(0.93, "area_command", &action("lexicon_exact", true, 1.0), &[target(true)]);
        let fuzzy = calibrate_step_confidence(0.91, "area_command", &action("lexicon_exact", true, 1.0), &[target(false)]);
        let session = calibrate_step_confidence(0.90, "session_entities", &action("lexicon_exact", true, 1.0), &[target(true)]);
        let inferred = calibrate_step_confidence(0.90, "grounded_entities", &action("context_inference", false, 0.65), &[target(true)]);
        let leftover = calibrate_step_confidence(0.90, "leftover_command", &action("lexicon_exact", true, 1.0), &[target(true)]);
        assert!(exact > fuzzy, "exact={exact} fuzzy={fuzzy}");
        assert!(fuzzy > inferred, "fuzzy={fuzzy} inferred={inferred}");
        assert!(inferred > session, "inferred={inferred} session={session}");
        assert!(session > leftover, "session={session} leftover={leftover}");
        assert!(exact >= EXECUTE_MIN_CONFIDENCE);
        assert!(inferred >= EXECUTE_MIN_CONFIDENCE);
        assert!(session >= EXECUTE_MIN_CONFIDENCE);
        assert!(leftover < CLARIFY_MIN_CONFIDENCE);
        assert!(inferred < 1.0);
        assert!(inferred < exact);
    }

    #[test]
    fn incomplete_multi_area_plans_lose_score() {
        let kitchen = Intent::new("HassTurnOn").with("entity_id", "light.kuche");
        let both = [Intent::new("HassTurnOn").with("area", "wohnzimmer"), Intent::new("HassTurnOn").with("area", "kuche")];
        let areas = ["wohnzimmer".into(), "kuche".into()];
        let incomplete = penalize_incomplete_area_coverage(0.90, std::slice::from_ref(&kitchen), &areas, &[]);
        let complete = penalize_incomplete_area_coverage(0.88, &both, &areas, &[]);
        assert!(complete > incomplete, "complete={complete} incomplete={incomplete}");
        assert_eq!(penalize_incomplete_area_coverage(0.90, &both, &["kuche".into()], &[]), 0.90);
    }
}
