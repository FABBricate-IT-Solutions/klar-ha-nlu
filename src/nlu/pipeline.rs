use crate::parse::calendar::speak_calendar_need;
use crate::parse::compound::expand_compounds;
use crate::parse::normalize::{strip_fillers_for_areas, tokenize};
use crate::parse::numbers::fold_spoken_numbers;
use crate::parse::respond::{speak_clarify, speak_unknown};
use crate::parse::split::split_clauses;
use crate::session::Session;
use crate::types::{
    Evidence, Intent, IntentCandidate, IntentPlan, ParseDecision, ParseOutcome, ParseTrace, PolicyTraceDiscarded, PolicyTraceMatch,
    RejectReason, StageTrace,
};
use std::collections::BTreeSet;
use std::time::Instant;

use super::answer::MAX_ANSWER_WORDS;
use super::binding::{build_analyses, MAX_CLAUSES};
use super::context::ParseContext;
use super::draft::{reject, replay_or_decide, route_custom, route_special, safety_decision, Draft, SessionCommit};
use super::ranking::rank_candidates;
use super::retrieval;
use super::semantic;

const MAX_NORMALIZED_TOKENS: usize = 256;
/// A reply to an open question is short; longer text is a command of its own.
const MAX_REPLY_WORDS: usize = 12;

pub(super) struct PipelineResult {
    pub outcome: ParseOutcome,
    commit: SessionCommit,
}

pub(super) fn run(context: ParseContext<'_>) -> PipelineResult {
    let _bound = crate::lang::bind_catalog(context.catalog);
    if context.session.pending().is_none() {
        return decide(&context);
    }
    // 1. An answer to the open question. Read on the words as spoken, before any other route.
    let said = super::phrase::words(context.text);
    let heard = fold_spoken_numbers(&tokenize(context.text));
    let tokens = expand_compounds(&strip_fillers_for_areas(&heard, &context.home.areas), context.home).tokens;
    let readable =
        said.len() <= MAX_ANSWER_WORDS.max(MAX_REPLY_WORDS) && !context.text.chars().any(|c| c.is_control() && !c.is_whitespace());
    if let Some(draft) = readable.then(|| super::pending::answer(&context, &said, &tokens)).flatten() {
        let mut trace = ParseTrace::default();
        let started = Instant::now();
        let draft = safety_decision(draft, &context);
        record_stage(&mut trace, "safety_decision", started, decision_name(&draft.decision).into());
        let candidates = draft.output_candidate.clone().into_iter().collect();
        return finish(&context, draft, candidates, Vec::new(), trace);
    }
    // 2. A complete new command. It is read as if nothing had been asked and replaces the question.
    let unasked = context.session.as_new_command();
    let mut fresh = decide(&context.with_session(&unasked));
    if replaces_question(&fresh, false) {
        fresh.commit.clear_pending = true;
        return fresh;
    }
    // 3. A reply that only makes sense with the question in mind ("turn on the office one").
    if let Some(draft) = super::pending::named_option(&context, &tokens) {
        let draft = safety_decision(draft, &context);
        let candidates = draft.output_candidate.clone().into_iter().collect();
        return finish(&context, draft, candidates, Vec::new(), fresh.outcome.trace);
    }
    if let Some(clarify) = context.session.pending_clarify() {
        let mut reply = decide(&context);
        let steps = match &reply.outcome.decision {
            ParseDecision::Execute => reply.outcome.plan.as_ref().map(IntentPlan::intents),
            ParseDecision::Confirm { .. } => reply.commit.confirm.as_ref().map(|(_, plan, _)| plan.intents()),
            ParseDecision::Clarify { .. } | ParseDecision::Reject { .. } | ParseDecision::Chat | ParseDecision::Error { .. } => None,
        };
        if steps.is_some_and(|steps| super::pending::stays_inside(&steps, clarify, context.home)) {
            reply.commit.clear_pending = true;
            return reply;
        }
    }
    // 4. A command that needs a question of its own: the new question replaces the old one.
    if replaces_question(&fresh, true) {
        fresh.commit.clear_pending = true;
        return fresh;
    }
    // 5. None of these: the question stands and is asked again.
    finish(&context, super::pending::ask_again(&context), Vec::new(), Vec::new(), fresh.outcome.trace)
}

/// Whether the new utterance is a command of its own, so the open question gives way.
/// A command that only leads to another question counts when `asking` is set.
fn replaces_question(result: &PipelineResult, asking: bool) -> bool {
    match &result.outcome.decision {
        ParseDecision::Execute | ParseDecision::Confirm { .. } | ParseDecision::Chat => true,
        ParseDecision::Clarify { .. } => asking && result.commit.clarify.is_some(),
        ParseDecision::Reject { .. } | ParseDecision::Error { .. } => result.commit.clear_pending || result.commit.mark_wrong,
    }
}

/// One utterance with no question open: routes, ranking and the safety decision.
fn decide(context: &ParseContext<'_>) -> PipelineResult {
    let mut trace = ParseTrace::default();
    let started = Instant::now();
    let raw_tokens = fold_spoken_numbers(&tokenize(context.text));
    let split = expand_compounds(&strip_fillers_for_areas(&raw_tokens, &context.home.areas), context.home);
    trace.tokens = split.tokens.iter().take(MAX_NORMALIZED_TOKENS).cloned().collect();
    trace.normalized = trace.tokens.join(" ");
    record_stage(&mut trace, "normalize", started, format!("{} normalized tokens", split.tokens.len()));
    if split.tokens.len() > MAX_NORMALIZED_TOKENS {
        let draft = reject(RejectReason::InvalidInput, speak_unknown());
        return finish(context, draft, Vec::new(), Vec::new(), trace);
    }

    let started = Instant::now();
    let clauses = split_clauses(&split.tokens, context.home);
    record_stage(&mut trace, "features", started, format!("{} clauses", clauses.len()));

    // Every route ends in the same safety decision: undo, routines and the other specials
    // get no way around validation, the user's rules or the confirmation.
    // A sentence that takes something out of its command ("… except the oven") is read
    // without it, or changes nothing. Only a sentence the user taught is taken as taught.
    let excepted = super::except::read(context, &raw_tokens);
    let heed = |draft: Draft| match &excepted {
        Some(excepted) => excepted.heed(context, draft),
        None => draft,
    };
    let special = || {
        super::dutch::route(context, &raw_tokens, &split.tokens)
            .or_else(|| super::windows::route(context, &split.tokens))
            .or_else(|| route_special(context, &raw_tokens, &split))
    };
    let taught = || route_custom(context, &split.tokens);
    let routed = match &excepted {
        Some(excepted) => taught().or_else(|| excepted.route(context)).or_else(|| special().map(heed)),
        None => special().or_else(taught),
    };
    if let Some(draft) = routed {
        let started = Instant::now();
        let draft = safety_decision(draft, context);
        record_stage(&mut trace, "safety_decision", started, decision_name(&draft.decision).into());
        let candidates = draft.output_candidate.clone().into_iter().collect();
        return finish(context, draft, candidates, Vec::new(), trace);
    }

    if clauses.len() > MAX_CLAUSES {
        let draft = reject(RejectReason::InvalidInput, speak_unknown());
        return finish(context, draft, Vec::new(), Vec::new(), trace);
    }
    let commands = clauses.clone();
    let analyses = match build_analyses(context, clauses, &raw_tokens, &split, &mut trace) {
        Ok(analyses) => analyses,
        Err(_) => {
            let draft = reject(RejectReason::InvalidInput, speak_unknown());
            return finish(context, draft, Vec::new(), Vec::new(), trace);
        }
    };

    let started = Instant::now();
    let mut ranking = rank_candidates(&analyses, context.home, context.policies, &mut trace);
    apply_resolved_lock_pair(&mut ranking, context, &split.tokens);
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
            inherits_risk: false,
            response_briefing: false,
            competing: ranking.competing,
            commit: SessionCommit { clarify: Some((options, template)), briefing: Some(false), ..SessionCommit::default() },
            policy_trace: None,
        }
    } else {
        replay_or_decide(context, &split.tokens, intents, selected_plan, &ranking)
    };
    let started = Instant::now();
    let mut draft = safety_decision(heed(draft), context);
    record_stage(&mut trace, "safety_decision", started, decision_name(&draft.decision).into());
    // Nothing is switched that the sentence did not ask for. An exception was read word by
    // word already, and its command went through here on its own.
    if excepted.is_none() {
        draft = super::heard::guard(context, &split.tokens, &commands, draft);
    }
    draft = super::unmet::guard(context, draft);
    if let Some(adapted) = semantic::consider(context, &split.tokens, &draft) {
        let started = Instant::now();
        draft = safety_decision(heed(adapted), context);
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
    finish(context, draft, candidates, evidence, trace)
}

/// What `text` alone would do as a command of its own: its plan steps, read without the
/// mood guard, without an open question and without touching the session. The guard uses
/// it to tell which clause a step comes from.
pub(super) struct Probe {
    pub intents: Vec<Intent>,
    pub decision: ParseDecision,
    pub confidence: f64,
    pub margin: f64,
}

pub(super) fn probe(context: &ParseContext<'_>, text: &str) -> Probe {
    let mut nested = ParseContext::new(text, context.home, context.session, context.custom, context.settings, context.catalog)
        .with_policies(context.policies, context.speech_bank)
        .with_match_controls(context.match_controls);
    nested.probe = true;
    let result = decide(&nested);
    let intents = match &result.outcome.decision {
        ParseDecision::Execute => result.outcome.plan.as_ref().map_or_else(Vec::new, IntentPlan::intents),
        ParseDecision::Confirm { .. } => result.commit.confirm.as_ref().map_or_else(Vec::new, |(_, plan, _)| plan.intents()),
        _ => Vec::new(),
    };
    Probe { intents, decision: result.outcome.decision, confidence: result.outcome.confidence, margin: result.outcome.margin }
}

impl PipelineResult {
    pub(super) fn commit(self, session: &mut Session) -> ParseOutcome {
        if self.commit.clear_pending {
            session.clear_pending();
        }
        if let Some((options, template)) = self.commit.clarify {
            session.set_clarify_prompt(options, template, self.outcome.speech.clone());
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
            if !briefing {
                session.news_topic = None;
            }
        }
        if self.commit.replace_news_topic {
            session.news_topic = self.commit.news_topic.clone().filter(|topic| !topic.is_empty());
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

fn outcome_topic(draft: &Draft, session: &Session) -> Option<String> {
    if draft.commit.briefing == Some(false) {
        return None;
    }
    if draft.commit.replace_news_topic {
        return draft.commit.news_topic.clone().filter(|topic| !topic.is_empty());
    }
    if draft.response_briefing {
        session.news_topic.clone()
    } else {
        None
    }
}

fn finish(
    context: &ParseContext<'_>,
    draft: Draft,
    mut candidates: Vec<IntentCandidate>,
    evidence: Vec<Evidence>,
    mut trace: ParseTrace,
) -> PipelineResult {
    let mut evidence = evidence;
    let (mut draft, guarded) = super::mood::guard(context, draft);
    if guarded {
        candidates = draft.output_candidate.clone().into_iter().collect();
        record_stage(&mut trace, "mood_guard", Instant::now(), decision_name(&draft.decision).into());
    }
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
    let news_topic = outcome_topic(&draft, context.session);
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
        news_topic,
        retrieval: None,
        policy_trace: draft.policy_trace.clone(),
        quiet_ack_eligible: false,
        refine_band: None,
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
    outcome.quiet_ack_eligible =
        matches!(outcome.decision, ParseDecision::Execute) && outcome.plan.as_ref().is_some_and(IntentPlan::quiet_ack_eligible);
    outcome.refine_band = ParseOutcome::classify_refine_band(&outcome.decision, outcome.plan.as_ref());
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
    if ranking.competing && !paired_lock_command(tokens) {
        return;
    }
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
    // Named locks ("haustuer und garagentor") may leave ranking empty; still install the pair.
    let install_pair = lock_only || clarifying_locks || (ranking.selected.is_none() && paired_lock_command(tokens));
    if !install_pair {
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

fn paired_lock_command(tokens: &[String]) -> bool {
    let cat = crate::lang::catalog();
    tokens.iter().any(|token| cat.is_conj(token))
        && (cat.any(tokens, cat.lock_nouns()) || cat.any(tokens, cat.door_nouns()) || cat.any(tokens, cat.lock_verbs()))
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
