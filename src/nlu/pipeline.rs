use crate::parse::calendar::speak_calendar_need;
use crate::parse::compound::expand_compounds;
use crate::parse::normalize::{strip_fillers, tokenize};
use crate::parse::respond::{speak_clarify, speak_unknown};
use crate::parse::split::split_clauses;
use crate::session::Session;
use crate::types::{
    Evidence, Intent, IntentCandidate, IntentPlan, ParseDecision, ParseOutcome, ParseTrace, PolicyTraceDiscarded, PolicyTraceMatch,
    RejectReason, StageTrace,
};
use std::collections::BTreeSet;
use std::time::Instant;

use super::binding::{build_analyses, MAX_CLAUSES};
use super::context::ParseContext;
use super::draft::{reject, replay_or_decide, route_pending, route_special, safety_decision, Draft, SessionCommit};
use super::ranking::rank_candidates;
use super::retrieval;
use super::semantic;

const MAX_NORMALIZED_TOKENS: usize = 256;

pub(super) struct PipelineResult {
    pub outcome: ParseOutcome,
    commit: SessionCommit,
}

pub(super) fn run(context: ParseContext<'_>) -> PipelineResult {
    let _bound = crate::lang::bind_catalog(context.catalog);
    let mut trace = ParseTrace::default();
    let started = Instant::now();
    let raw_tokens = tokenize(context.text);
    let split = expand_compounds(&strip_fillers(&raw_tokens), context.home);
    trace.tokens = split.tokens.iter().take(MAX_NORMALIZED_TOKENS).cloned().collect();
    trace.normalized = trace.tokens.join(" ");
    record_stage(&mut trace, "normalize", started, format!("{} normalized tokens", split.tokens.len()));
    if split.tokens.len() > MAX_NORMALIZED_TOKENS {
        let draft = reject(RejectReason::InvalidInput, speak_unknown());
        return finish(&context, draft, Vec::new(), Vec::new(), trace);
    }

    let started = Instant::now();
    let clauses = split_clauses(&split.tokens, context.home);
    record_stage(&mut trace, "features", started, format!("{} clauses", clauses.len()));

    if let Some(draft) = route_special(&context, &raw_tokens, &split) {
        record_stage(&mut trace, "safety_decision", Instant::now(), decision_name(&draft.decision).into());
        let candidates = draft.output_candidate.clone().into_iter().collect();
        return finish(&context, draft, candidates, Vec::new(), trace);
    }
    if let Some(draft) = route_pending(&context, &split.tokens) {
        let started = Instant::now();
        let draft = safety_decision(draft, &context);
        record_stage(&mut trace, "safety_decision", started, decision_name(&draft.decision).into());
        let candidates = draft.output_candidate.clone().into_iter().collect();
        return finish(&context, draft, candidates, Vec::new(), trace);
    }

    if clauses.len() > MAX_CLAUSES {
        let draft = reject(RejectReason::InvalidInput, speak_unknown());
        return finish(&context, draft, Vec::new(), Vec::new(), trace);
    }
    let analyses = match build_analyses(&context, clauses, &raw_tokens, &split, &mut trace) {
        Ok(analyses) => analyses,
        Err(_) => {
            let draft = reject(RejectReason::InvalidInput, speak_unknown());
            return finish(&context, draft, Vec::new(), Vec::new(), trace);
        }
    };

    let started = Instant::now();
    let mut ranking = rank_candidates(&analyses, context.home, context.policies, &mut trace);
    apply_resolved_lock_pair(&mut ranking, &context, &split.tokens);
    record_stage(&mut trace, "ranking", started, format!("{} ranked candidates", ranking.candidates.len()));
    let clarify = ranking.clarification.clone();
    let mut intents = ranking.selected.as_ref().map_or_else(Vec::new, |candidate| candidate.plan.intents());
    dedup_intents(&mut intents);
    let selected_plan = ranking.selected.as_ref().map(|candidate| candidate.plan.clone());
    let draft = if let Some((options, template)) = clarify {
        let prompt =
            if template.name.contains("Calendar") { speak_calendar_need(&template) } else { speak_clarify(&options, Some(context.home)) };
        Draft {
            decision: ParseDecision::Clarify { prompt: prompt.clone(), options: options.clone() },
            plan: None,
            speech: prompt,
            confidence: ranking.confidence,
            margin: ranking.margin,
            selected_candidate_id: None,
            output_candidate: ranking.selected.clone(),
            safety_confirmed: false,
            response_briefing: false,
            competing: ranking.competing,
            commit: SessionCommit { clarify: Some((options, template)), briefing: Some(false), ..SessionCommit::default() },
            policy_trace: None,
        }
    } else {
        replay_or_decide(&context, &split.tokens, intents, selected_plan, &ranking)
    };
    let started = Instant::now();
    let mut draft = safety_decision(draft, &context);
    record_stage(&mut trace, "safety_decision", started, decision_name(&draft.decision).into());
    if let Some(adapted) = semantic::consider(&context, &split.tokens, &draft) {
        let started = Instant::now();
        draft = safety_decision(adapted, &context);
        record_stage(&mut trace, "semantic_adapter", started, decision_name(&draft.decision).into());
    }
    let mut candidates = ranking.candidates;
    if let Some(updated) = draft.output_candidate.clone() {
        if let Some(existing) = candidates.iter_mut().find(|candidate| candidate.id == updated.id) {
            *existing = updated;
        } else {
            candidates.push(updated);
        }
    }
    let mut evidence = ranking.evidence;
    if let Some(plan) = draft.plan.as_ref() {
        for item in &plan.evidence {
            if item.kind == "semantic_adapter" && !evidence.iter().any(|existing| existing == item) {
                evidence.push(item.clone());
            }
        }
    }
    finish(&context, draft, candidates, evidence, trace)
}

impl PipelineResult {
    pub(super) fn commit(self, session: &mut Session) -> ParseOutcome {
        if self.commit.clear_pending {
            session.clear_pending();
        }
        if let Some((options, template)) = self.commit.clarify {
            session.set_clarify(options, template);
        }
        if let Some((candidate_id, plan, prompt)) = self.commit.confirm {
            session.set_confirm(candidate_id, plan, prompt);
        }
        if !self.commit.remember.is_empty() {
            session.begin_remember_batch();
        }
        for intent in &self.commit.remember {
            session.remember(intent);
        }
        if self.commit.remember.iter().any(|intent| super::household::invert_intent(intent).is_some()) {
            session.last_execute = self.commit.remember.clone();
        }
        if let Some(briefing) = self.commit.briefing {
            session.briefing = briefing;
        }
        if self.commit.mark_wrong {
            session.mark_wrong();
        }
        if let Some(teach) = self.commit.teach {
            session.pending_teach = Some(teach);
        }
        session.note_heard(&self.outcome);
        self.outcome
    }
}

fn finish(
    context: &ParseContext<'_>,
    mut draft: Draft,
    mut candidates: Vec<IntentCandidate>,
    evidence: Vec<Evidence>,
    mut trace: ParseTrace,
) -> PipelineResult {
    let mut evidence = evidence;
    if let Some(area) = context.session.preferred_area.as_deref().filter(|area| !area.is_empty()) {
        if !evidence.iter().any(|item| item.kind == "preferred_area") {
            evidence.push(Evidence {
                kind: "preferred_area".into(),
                source: "satellite".into(),
                value: area.to_string(),
                score: 1.0,
                exact: true,
            });
        }
    }
    let started = Instant::now();
    let executable_plan = matches!(draft.decision, ParseDecision::Execute).then(|| draft.plan.clone()).flatten();
    let execute = matches!(draft.decision, ParseDecision::Execute);
    attach_policy_trace(&mut draft, &trace);
    if !execute {
        candidates.clear();
    }
    let selected_candidate_id = execute.then(|| draft.selected_candidate_id.clone()).flatten();
    record_stage(
        &mut trace,
        "planning",
        started,
        format!("{} executable plan steps", executable_plan.as_ref().map_or(0, |value| value.steps.len())),
    );
    let mut outcome = ParseOutcome {
        schema_version: ParseOutcome::schema_version(),
        text: context.text.to_string(),
        conversation_id: context.session.id.clone(),
        decision: draft.decision,
        speech: draft.speech,
        confidence: draft.confidence,
        margin: draft.margin,
        selected_candidate_id,
        candidates,
        plan: executable_plan,
        evidence,
        trace,
        briefing: draft.response_briefing,
        retrieval: None,
        policy_trace: draft.policy_trace.clone(),
    };
    let values: Vec<String> = outcome.evidence.iter().map(|item| item.value.clone()).collect();
    outcome.retrieval = retrieval::build(context, &outcome.decision, &values);
    if let Some(pack) = &mut outcome.retrieval {
        pack.tokens = outcome.trace.tokens.clone();
    }
    if matches!(outcome.decision, ParseDecision::Execute) {
        let selected_matches = outcome.selected_candidate_id.as_ref().is_some_and(|selected_id| {
            !selected_id.is_empty()
                && selected_id.chars().count() <= 128
                && outcome
                    .plan
                    .as_ref()
                    .is_some_and(|plan| outcome.candidates.iter().any(|candidate| candidate.id == *selected_id && candidate.plan == *plan))
        });
        if !selected_matches {
            outcome.decision =
                ParseDecision::Error { code: "invalid_selection".into(), message: "Planner selection was not representable".into() };
            outcome.plan = None;
            outcome.selected_candidate_id = None;
            outcome.candidates.clear();
            draft.commit.remember.clear();
            draft.commit.clear_pending = false;
        }
    }
    outcome.enforce_output_caps();
    PipelineResult { outcome, commit: draft.commit }
}

fn attach_policy_trace(draft: &mut Draft, parse_trace: &ParseTrace) {
    let mut policy = draft.policy_trace.take().unwrap_or_default();
    if policy.match_node.is_none() {
        policy.match_node = draft.output_candidate.as_ref().and_then(PolicyTraceMatch::from_candidate);
    }
    if policy.band.is_none() {
        policy.band = Some(draft.decision.type_name().into());
    }
    if policy.discarded.is_empty() {
        policy.discarded = parse_trace.discarded.iter().map(PolicyTraceDiscarded::from_alternative).collect();
    }
    draft.policy_trace = Some(policy);
}

fn apply_resolved_lock_pair(ranking: &mut super::ranking::RankingResult, context: &ParseContext<'_>, tokens: &[String]) {
    let locks = crate::parse::resolve::resolve(tokens, context.home, Some("lock"))
        .entities
        .into_iter()
        .filter(|entity| entity.domain == "lock")
        .map(|entity| entity.entity_id)
        .collect::<Vec<_>>();
    if locks.len() < 2 {
        return;
    }
    if ranking.selected.as_ref().is_some_and(|candidate| {
        candidate.plan.steps.iter().any(|step| matches!(step.intent.name.as_str(), "HassGetState" | "HassClimateGetTemperature"))
    }) {
        return;
    }
    let lock_only = ranking.selected.as_ref().is_some_and(|candidate| {
        candidate.plan.steps.iter().all(|step| {
            step.intent.slot("entity_id").is_some_and(|id| id.starts_with("lock.")) || step.intent.slot("domain") == Some("lock")
        })
    });
    let clarifying_locks =
        ranking.clarification.as_ref().is_some_and(|(options, _)| options.iter().filter(|id| id.starts_with("lock.")).count() >= 2);
    if !lock_only && !clarifying_locks {
        return;
    }
    let name = ranking
        .selected
        .as_ref()
        .and_then(|candidate| candidate.plan.steps.first())
        .map(|step| step.intent.name.as_str())
        .filter(|name| *name == "HassTurnOff")
        .unwrap_or("HassTurnOn");
    let intents: Vec<Intent> = locks.iter().map(|id| Intent::new(name).with("entity_id", id).with("domain", "lock")).collect();
    let plan = IntentPlan::from_intents(intents, ranking.confidence.max(0.92), &[]);
    ranking.clarification = None;
    ranking.competing = false;
    ranking.margin = 1.0;
    ranking.confidence = ranking.confidence.max(0.92);
    if let Some(selected) = ranking.selected.as_mut() {
        selected.plan = plan;
        selected.margin = 1.0;
        selected.score = selected.score.max(0.92);
    } else {
        ranking.selected = Some(IntentCandidate {
            id: "plan-000".into(),
            plan,
            score: 0.92,
            margin: 1.0,
            policy: "grounded_entities".into(),
            precedence: 0,
            evidence: Vec::new(),
        });
    }
}

fn dedup_intents(intents: &mut Vec<Intent>) {
    let mut seen = BTreeSet::new();
    intents.retain(|intent| {
        let entity_domain = intent.slot("entity_id").and_then(|entity_id| entity_id.split_once('.').map(|(domain, _)| domain));
        let mut slots = intent
            .slots
            .iter()
            .filter(|slot| slot.name != "domain" || entity_domain != Some(slot.value.as_str()))
            .map(|slot| format!("{}={}", slot.name, slot.value))
            .collect::<Vec<_>>();
        slots.sort();
        seen.insert(format!("{}|{}", intent.name, slots.join("|")))
    });
}

fn record_stage(trace: &mut ParseTrace, stage: &str, started: Instant, detail: String) {
    trace.stages.push(StageTrace { stage: stage.into(), duration_us: started.elapsed().as_micros() as u64, detail });
}

fn decision_name(decision: &ParseDecision) -> &'static str {
    decision.type_name()
}
