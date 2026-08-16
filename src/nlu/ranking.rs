use crate::parse::action::Action;
use crate::parse::slots::ClauseOut;
use crate::types::{
    DiscardedAlternative, Evidence, HomeGraph, Intent, IntentCandidate, IntentPlan, ParseTrace, PlanStep, MAX_CANDIDATES,
    MAX_EVIDENCE_PER_ITEM, MAX_PLAN_STEPS,
};
use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::binding::{Analysis, BindingAnalysis};
use super::decision::{
    best_plan_target_score, calibrate_step_confidence, capped_options, competing_plans_need_clarify, complete_plans_compete,
    penalize_incomplete_area_coverage, plan_targets, ranking_plans_compete,
};
use super::evidence::target_evidence;
use super::validation::{filter_valid_steps, requires_target, validate_plan};

pub(super) const MAX_BINDINGS_PER_CLAUSE: usize = 24;
const MAX_ALTERNATIVES_PER_CLAUSE: usize = 8;
const BEAM_WIDTH: usize = 16;

#[derive(Clone)]
struct Choice {
    clause_index: usize,
    binding_index: usize,
}

#[derive(Clone)]
struct BeamEntry {
    candidate: IntentCandidate,
    choices: Vec<Choice>,
}

pub(super) struct RankingResult {
    pub candidates: Vec<IntentCandidate>,
    pub selected: Option<IntentCandidate>,
    pub clarification: Option<(Vec<String>, Intent)>,
    pub evidence: Vec<Evidence>,
    pub confidence: f64,
    pub margin: f64,
    pub competing: bool,
}

pub(super) fn provisional_selection(analysis: &Analysis) -> Option<usize> {
    ranked_clause(analysis).first().map(|row| row.0)
}

pub(super) fn rank_candidates(analyses: &[Analysis], home: &HomeGraph, trace: &mut ParseTrace) -> RankingResult {
    let clause_alternatives = analyses
        .iter()
        .map(|analysis| ranked_clause(analysis).into_iter().filter(|(_, fragment)| keep_fragment(fragment, home)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut applied_clauses = 0;
    let mut beam = vec![empty_entry()];

    for (analysis, alternatives) in analyses.iter().zip(clause_alternatives) {
        if alternatives.is_empty() {
            continue;
        }
        let mut generated = Vec::with_capacity(BEAM_WIDTH * MAX_ALTERNATIVES_PER_CLAUSE);
        for current in beam.iter().take(BEAM_WIDTH) {
            for (binding_index, fragment) in alternatives.iter().take(MAX_ALTERNATIVES_PER_CLAUSE) {
                if let Some(next) = combine_entry(current, analysis.index, *binding_index, fragment, home) {
                    generated.push(next);
                }
            }
        }
        generated.sort_by(|left, right| compare_candidates(&left.candidate, &right.candidate));
        let mut plans = BTreeSet::new();
        generated.retain(|entry| plans.insert(plan_key(&entry.candidate.plan)));
        generated.truncate(BEAM_WIDTH);
        if generated.is_empty() {
            continue;
        }
        applied_clauses += 1;
        beam = generated;
    }

    for entry in &mut beam {
        if !is_clarify_candidate(&entry.candidate) {
            let filtered = filter_valid_steps(&entry.candidate.plan, home);
            if filtered.steps.len() != entry.candidate.plan.steps.len() {
                entry.candidate.score = entry.candidate.score.min(filtered.confidence);
            }
            entry.candidate.plan = filtered;
        }
    }
    beam.retain(|entry| {
        entry.choices.len() == applied_clauses
            && !entry.candidate.plan.steps.is_empty()
            && (is_clarify_candidate(&entry.candidate) || validate_plan(&entry.candidate.plan, home).is_ok())
    });
    beam.sort_by(|left, right| compare_candidates(&left.candidate, &right.candidate));
    beam.truncate(MAX_CANDIDATES.min(BEAM_WIDTH));
    assign_ids_and_margins(&mut beam);
    record_discarded(trace, &beam);

    let clarification = derive_clarification(analyses, &beam);
    let selected = beam.first().filter(|entry| !entry.candidate.plan.steps.is_empty()).map(|entry| entry.candidate.clone());
    let candidates = if selected.is_some() {
        beam.iter().filter(|entry| !entry.candidate.plan.steps.is_empty()).map(|entry| entry.candidate.clone()).collect()
    } else {
        Vec::new()
    };
    let confidence = selected.as_ref().map_or(0.0, |candidate| candidate.plan.confidence);
    let margin = selected.as_ref().map_or(0.0, |candidate| candidate.margin);
    let evidence = selected.as_ref().map_or_else(Vec::new, |candidate| candidate.evidence.clone());
    let competing = beam
        .get(1)
        .is_some_and(|runner| beam.first().is_some_and(|selected| ranking_plans_compete(&selected.candidate, &runner.candidate)));
    RankingResult { candidates, selected, clarification, evidence, confidence, margin, competing }
}

fn ranked_clause(analysis: &Analysis) -> Vec<(usize, IntentCandidate)> {
    let mut rows = analysis
        .bindings
        .iter()
        .take(MAX_BINDINGS_PER_CLAUSE)
        .enumerate()
        .map(|(binding_index, binding)| (binding_index, build_fragment(analysis.index, binding_index, binding)))
        .collect::<Vec<_>>();
    rows.retain(|row| usable_fragment(&row.1));
    rows.sort_by(|left, right| compare_candidates(&left.1, &right.1));
    let mut plans = BTreeSet::new();
    rows.retain(|row| plans.insert(plan_key(&row.1.plan)));
    rows.truncate(MAX_ALTERNATIVES_PER_CLAUSE);
    rows
}

fn is_clarify_candidate(candidate: &IntentCandidate) -> bool {
    candidate.policy.contains("clarify") || candidate.evidence.iter().any(|item| item.value == "clarify")
}

fn keep_fragment(fragment: &IntentCandidate, home: &HomeGraph) -> bool {
    if !usable_fragment(fragment) {
        return false;
    }
    if is_clarify_candidate(fragment) {
        return true;
    }
    !filter_valid_steps(&fragment.plan, home).steps.is_empty()
}

fn usable_fragment(fragment: &IntentCandidate) -> bool {
    if fragment.plan.steps.is_empty() {
        return false;
    }
    if fragment.policy.contains("clarify") || fragment.evidence.iter().any(|item| item.value == "clarify") {
        return true;
    }
    let grounded = fragment.plan.steps.iter().all(|step| {
        !requires_target(&step.intent.name)
            || step.intent.slot("entity_id").is_some()
            || step.intent.slot("area").is_some()
            || step.intent.slot("floor").is_some()
    });
    if grounded && fragment.score > 0.0 {
        return true;
    }
    fragment
        .plan
        .steps
        .iter()
        .any(|step| step.intent.slots.iter().any(|slot| !matches!(slot.name.as_str(), "entity_id" | "area" | "floor" | "domain")))
}

fn empty_entry() -> BeamEntry {
    BeamEntry {
        candidate: IntentCandidate {
            id: String::new(),
            plan: IntentPlan::from_steps(Vec::new(), 0.0),
            score: 1.0,
            margin: 0.0,
            policy: String::new(),
            precedence: 0,
            evidence: Vec::new(),
        },
        choices: Vec::new(),
    }
}

fn combine_entry(
    current: &BeamEntry,
    clause_index: usize,
    binding_index: usize,
    fragment: &IntentCandidate,
    home: &HomeGraph,
) -> Option<BeamEntry> {
    if fragment.plan.steps.is_empty()
        || (!current.choices.is_empty()
            && (fragment.policy.contains("fallback") || fragment.policy.contains("leftover") || fragment.policy.starts_with("session_")))
    {
        return None;
    }
    let previous_step_count = current.candidate.plan.steps.len();
    let mut steps = current.candidate.plan.steps.clone();
    for step in &fragment.plan.steps {
        let repeats_session_target = fragment.policy.starts_with("session_")
            && steps.iter().any(|existing| {
                [existing.intent.slot("entity_id"), existing.intent.slot("area"), existing.intent.slot("floor")].into_iter().flatten().any(
                    |target| {
                        step.intent.slot("entity_id") == Some(target)
                            || step.intent.slot("area") == Some(target)
                            || step.intent.slot("floor") == Some(target)
                    },
                )
            });
        if !repeats_session_target {
            steps.push(step.clone());
        }
    }
    let mut merged: Vec<PlanStep> = Vec::with_capacity(steps.len());
    for step in steps {
        let key = intent_key(&step.intent);
        if let Some(existing) = merged.iter_mut().find(|existing| intent_key(&existing.intent) == key) {
            if step.intent.slots.len() > existing.intent.slots.len() {
                *existing = step;
            }
        } else {
            merged.push(step);
        }
    }
    let mut steps = if is_clarify_candidate(fragment) || current.candidate.policy.contains("clarify") {
        merged
    } else {
        filter_valid_steps(&IntentPlan::from_steps(merged, 0.0), home).steps
    };
    if steps.len() == previous_step_count || steps.len() > MAX_PLAN_STEPS {
        return None;
    }
    for (index, step) in steps.iter_mut().enumerate() {
        step.index = index;
    }
    let mut evidence = current.candidate.evidence.clone();
    merge_evidence(&mut evidence, &fragment.evidence);
    let mut policy = current.candidate.policy.clone();
    if !policy.is_empty() {
        policy.push('+');
    }
    policy.push_str(&fragment.policy);
    policy.truncate(128);
    let score = current.candidate.score.min(fragment.score);
    let mut choices = current.choices.clone();
    choices.push(Choice { clause_index, binding_index });
    let mut plan = IntentPlan::from_steps(steps, 0.0);
    plan.evidence = evidence.clone();
    Some(BeamEntry {
        candidate: IntentCandidate {
            id: String::new(),
            plan,
            score,
            margin: 0.0,
            policy,
            precedence: current.candidate.precedence.saturating_add(fragment.precedence),
            evidence,
        },
        choices,
    })
}

fn build_fragment(clause_index: usize, binding_index: usize, binding: &BindingAnalysis) -> IntentCandidate {
    let policy = &binding.policy;
    let (intents, binding_value, mut binding_score) = match &policy.outcome {
        ClauseOut::Intents(intents) => (intents.clone(), "bound", binding_score(intents, binding)),
        ClauseOut::Clarify(_, template) => {
            let score = if policy.policy.as_str() == "light_rooms_clarify" { 0.72 } else { 0.90 };
            (vec![template.clone()], "clarify", score)
        }
    };
    if let Some(allowed) = &binding.allowed_targets {
        let violates =
            intents.iter().filter_map(|intent| intent.slot("entity_id")).any(|entity_id| !allowed.iter().any(|target| target == entity_id));
        if violates {
            binding_score *= 0.2;
        }
    }
    let binding_evidence = Evidence {
        kind: "binding".into(),
        source: policy.policy.as_str().into(),
        value: binding_value.into(),
        score: binding_score,
        exact: false,
    };
    let mut steps = Vec::new();
    for (index, intent) in intents.into_iter().take(MAX_PLAN_STEPS).enumerate() {
        let mut evidence = vec![binding.action_evidence.clone(), binding_evidence.clone()];
        add_target_evidence(&mut evidence, &intent, binding);
        evidence.truncate(MAX_EVIDENCE_PER_ITEM);
        let target_score =
            evidence.iter().filter(|item| item.kind.starts_with("target_")).map(|item| item.score).reduce(f64::min).unwrap_or(0.85);
        let resolver_certainty = if binding.targets.ranked.len() > 1 { 0.85 + binding.targets.margin.clamp(0.0, 1.0) * 0.15 } else { 1.0 };
        let raw =
            0.50 * policy.score + 0.20 * binding.action_evidence.score + 0.20 * target_score.min(resolver_certainty) + 0.10 * binding_score;
        let confidence = calibrate_step_confidence(raw, policy.policy.as_str(), &binding.action_evidence, &evidence);
        steps.push(PlanStep { index, intent, confidence, evidence });
    }
    let plan = IntentPlan::from_steps(steps, 0.0);
    let mut score = if !plan.steps.is_empty() && policy.policy.as_str() == "all_lights" {
        (plan.confidence + 0.05).min(1.0)
    } else if plan.steps.is_empty() && policy.policy.as_str() == "media" {
        policy.score
    } else if plan.steps.is_empty() {
        policy.score.min(binding.action_evidence.score)
    } else {
        plan.confidence
    };
    score = penalize_incomplete_area_coverage(score, &plan.intents(), &binding.targets.resolved.areas, &binding.targets.resolved.entities);
    let mut evidence = plan.evidence.clone();
    if evidence.is_empty() {
        evidence.extend([binding.action_evidence.clone(), binding_evidence]);
        evidence.truncate(MAX_EVIDENCE_PER_ITEM);
    }
    IntentCandidate {
        id: format!("fragment-{clause_index:02}-{binding_index:02}"),
        plan,
        score,
        margin: 0.0,
        policy: policy.policy.as_str().into(),
        precedence: policy.precedence,
        evidence,
    }
}

fn binding_score(intents: &[Intent], binding: &BindingAnalysis) -> f64 {
    if intents.is_empty() {
        return 0.55;
    }
    if intents.iter().any(|intent| {
        requires_target(&intent.name)
            && intent.slot("entity_id").is_none()
            && intent.slot("area").is_none()
            && intent.slot("floor").is_none()
    }) {
        return 0.0;
    }
    if intents.iter().any(|intent| intent.slot("entity_id").is_some()) {
        return 1.0;
    }
    if intents.iter().any(|intent| intent.slot("area").is_some() || intent.slot("floor").is_some()) {
        if binding.policy.policy.as_str() == "laundry_switch" && binding.policy.action == Action::GetState {
            return 1.0;
        }
        if binding.policy.policy.as_str() == "multi_area" && binding.policy.action != Action::GetState {
            return 1.0;
        }
        if binding.policy.policy.as_str() == "multi_area" && binding.policy.action == Action::GetState {
            return 0.50;
        }
        return if matches!(
            binding.policy.policy.as_str(),
            "laundry_switch" | "all_lights" | "preferred_area_command" | "area_command" | "floor_command" | "query_area" | "multi_area"
        ) {
            0.90
        } else {
            0.50
        };
    }
    0.85
}

fn add_target_evidence(evidence: &mut Vec<Evidence>, intent: &Intent, binding: &BindingAnalysis) {
    for target in [intent.slot("entity_id"), intent.slot("area"), intent.slot("floor")].into_iter().flatten() {
        if let Some(found) = binding.targets.ranked.iter().find(|row| row.target == target) {
            evidence.push(target_evidence(found));
            continue;
        }
        let (source, score) = if binding.policy.policy.as_str() == "preferred_area_command" {
            ("preferred_area_context", 0.78)
        } else if binding.policy.policy.as_str().starts_with("session_") {
            ("session_context", 0.76)
        } else if matches!(
            binding.policy.policy.as_str(),
            "all_lights" | "area_command" | "floor_command" | "multi_area" | "laundry_switch" | "media"
        ) {
            ("policy_expansion", 0.82)
        } else {
            ("legacy_resolver_binding", 0.74)
        };
        evidence.push(Evidence {
            kind: if target.contains('.') {
                "target_entity"
            } else if intent.slot("floor") == Some(target) {
                "target_floor"
            } else {
                "target_area"
            }
            .into(),
            source: source.into(),
            value: target.into(),
            score,
            exact: false,
        });
    }
}

fn assign_ids_and_margins(beam: &mut [BeamEntry]) {
    for index in 0..beam.len() {
        let margin = beam.get(index + 1).map_or(1.0, |next| (beam[index].candidate.score - next.candidate.score).max(0.0));
        beam[index].candidate.id = format!("plan-{index:03}");
        beam[index].candidate.margin = margin;
        beam[index].candidate.plan.margin = margin;
    }
}

fn derive_clarification(analyses: &[Analysis], beam: &[BeamEntry]) -> Option<(Vec<String>, Intent)> {
    let selected = beam.first()?;
    for choice in &selected.choices {
        let binding = analyses.iter().find(|analysis| analysis.index == choice.clause_index)?.bindings.get(choice.binding_index)?;
        if let ClauseOut::Clarify(options, template) = &binding.policy.outcome {
            return Some((capped_options(options.clone()), template.clone()));
        }
    }
    let runner_up = beam.get(1)?;
    if !complete_plans_compete(&selected.candidate.policy, &runner_up.candidate.policy, &selected.candidate.plan, &runner_up.candidate.plan)
    {
        return None;
    }
    let margin = (selected.candidate.score - runner_up.candidate.score).max(0.0);
    let target_margin = best_plan_target_score(&selected.candidate.plan) - best_plan_target_score(&runner_up.candidate.plan);
    if !competing_plans_need_clarify(margin, true, true, target_margin) {
        return None;
    }
    let mut options = plan_targets(&selected.candidate.plan);
    options.extend(plan_targets(&runner_up.candidate.plan));
    options = capped_options(options);
    let template = selected.candidate.plan.steps.first()?.intent.clone();
    (options.len() > 1).then_some((options, template))
}

fn record_discarded(trace: &mut ParseTrace, beam: &[BeamEntry]) {
    let selected = beam.first().map(|entry| entry.candidate.id.as_str()).unwrap_or("none");
    for entry in beam.iter().skip(1).take(crate::types::MAX_TRACE_DISCARDED) {
        trace.discarded.push(DiscardedAlternative {
            candidate_id: entry.candidate.id.clone(),
            policy: entry.candidate.policy.clone(),
            score: entry.candidate.score,
            reason: format!("complete_plan_ranked_below={selected}"),
        });
    }
}

fn compare_candidates(left: &IntentCandidate, right: &IntentCandidate) -> Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.precedence.cmp(&right.precedence))
        .then_with(|| left.policy.cmp(&right.policy))
        .then_with(|| plan_key(&left.plan).cmp(&plan_key(&right.plan)))
}

fn plan_key(plan: &IntentPlan) -> String {
    plan.steps.iter().map(|step| intent_key(&step.intent)).collect::<Vec<_>>().join(";")
}

fn intent_key(intent: &Intent) -> String {
    let entity_domain = intent.slot("entity_id").and_then(|entity_id| entity_id.split_once('.').map(|(domain, _)| domain));
    let mut slots = intent
        .slots
        .iter()
        .filter(|slot| slot.name != "domain" || entity_domain != Some(slot.value.as_str()))
        .map(|slot| format!("{}={}", slot.name, slot.value))
        .collect::<Vec<_>>();
    slots.sort();
    format!("{}|{}", intent.name, slots.join(","))
}

fn merge_evidence(target: &mut Vec<Evidence>, values: &[Evidence]) {
    for value in values {
        if target.len() >= MAX_EVIDENCE_PER_ITEM {
            break;
        }
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}
