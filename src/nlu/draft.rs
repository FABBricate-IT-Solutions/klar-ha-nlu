use crate::parse::chat::{briefing_followup, is_news, is_news_dismiss, is_ood, wants_llm};
use crate::parse::clause_support::last_visible;
use crate::parse::compound::CompoundSplit;
use crate::parse::infer::{looks_like_correction, match_custom, pick_clarification};
use crate::parse::numbers::first_number;
use crate::parse::respond::{speak, speak_correction, speak_need_target, speak_unknown};
use crate::parse::slots::intent_with_entity;
use crate::types::{
    allow_permitted, first_matching_rule, first_seed_match, Intent, IntentCandidate, IntentPlan, ParseDecision, PolicyHit, PolicyTrace,
    PolicyTraceLayer, PolicyTraceMatch, RejectReason,
};

use super::context::ParseContext;
use super::decision::{decide_band, PolicyBand};
use super::ranking::RankingResult;
use super::speech::{apply_rule_speech, confirmation_prompt};
use super::validation::{filter_valid_steps, requires_confirmation, validate_plan, PlanInvalid};

#[derive(Default)]
pub(super) struct SessionCommit {
    pub remember: Vec<Intent>,
    pub clarify: Option<(Vec<String>, Intent)>,
    pub confirm: Option<(String, IntentPlan, String)>,
    pub clear_pending: bool,
    pub briefing: Option<bool>,
    pub mark_wrong: bool,
    pub teach: Option<(String, String)>,
}

pub(super) struct Draft {
    pub decision: ParseDecision,
    pub plan: Option<IntentPlan>,
    pub speech: String,
    pub confidence: f64,
    pub margin: f64,
    pub selected_candidate_id: Option<String>,
    pub output_candidate: Option<IntentCandidate>,
    pub safety_confirmed: bool,
    pub response_briefing: bool,
    pub competing: bool,
    pub commit: SessionCommit,
    pub policy_trace: Option<crate::types::PolicyTrace>,
}

pub(super) fn route_special(context: &ParseContext<'_>, raw: &[String], split: &CompoundSplit) -> Option<Draft> {
    let tokens = &split.tokens;
    if tokens.is_empty() {
        return Some(reject(RejectReason::EmptyInput, speak_unknown()));
    }
    if context.text.chars().any(|character| character.is_control() && !character.is_whitespace()) {
        return Some(reject(RejectReason::InvalidInput, speak_unknown()));
    }
    if let Some(draft) = super::household::route(context, tokens) {
        return Some(draft);
    }
    if let Some(draft) = super::policy_route::route_phrase(context) {
        return Some(draft);
    }
    if is_news(raw, context.home) || is_news(tokens, context.home) {
        return Some(chat(context.catalog.news_intro.to_string(), true, true));
    }
    if context.session.briefing && (is_news_dismiss(tokens) || is_news_dismiss(raw)) {
        let mut draft = reject(RejectReason::Unsupported, context.catalog.news_done.to_string());
        draft.response_briefing = true;
        draft.commit.briefing = Some(false);
        return Some(draft);
    }
    if briefing_followup(tokens, context.home, context.session) || briefing_followup(raw, context.home, context.session) {
        let news = context.session.briefing;
        return Some(chat(String::new(), news, news));
    }
    if wants_llm(raw, context.home) || wants_llm(tokens, context.home) {
        return Some(chat(String::new(), context.session.briefing, context.session.briefing));
    }
    if is_ood(raw, context.home) || is_ood(tokens, context.home) {
        return Some(reject(RejectReason::NoAction, speak_unknown()));
    }
    if looks_like_correction(tokens) && context.session.pending_confirm().is_none() {
        let mut draft = reject(RejectReason::Unsupported, speak_correction());
        draft.commit.mark_wrong = true;
        return Some(draft);
    }
    None
}

pub(super) fn route_pending(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
    if let Some(confirm) = context.session.pending_confirm() {
        if affirmative(tokens, context.catalog) {
            return Some(execute_plan(
                context,
                confirm.plan.clone(),
                "pending_confirm",
                Some(confirm.candidate_id.clone()),
                None,
                true,
                true,
            ));
        }
        if is_news_dismiss(tokens) {
            let mut draft = reject(RejectReason::Unsupported, speak_unknown());
            draft.commit.clear_pending = true;
            return Some(draft);
        }
        return Some(Draft {
            decision: ParseDecision::Confirm { prompt: confirm.prompt.clone(), candidate_id: confirm.candidate_id.clone() },
            plan: Some(confirm.plan.clone()),
            speech: confirm.prompt.clone(),
            confidence: confirm.plan.confidence,
            margin: confirm.plan.margin,
            selected_candidate_id: Some(confirm.candidate_id.clone()),
            output_candidate: None,
            safety_confirmed: false,
            response_briefing: false,
            competing: false,
            commit: SessionCommit::default(),
            policy_trace: None,
        });
    }
    if context.session.pending_clarify().is_some() {
        let picked = pick_clarification(tokens, context.session);
        if let Some(chosen) = picked {
            let template = context.session.pending_clarify()?.template.clone();
            let intent = if context.home.areas.iter().any(|area| area.area_id == chosen) {
                template.with_set("area", &chosen).with_set("domain", "light")
            } else {
                intent_with_entity(template, &chosen)
            };
            return Some(execute(context, vec![intent], "pending_clarify", 1.0, 1.0, true, false));
        }
    } else if affirmative(tokens, context.catalog) {
        if let Some(entity_id) = last_visible(context.session, context.home) {
            let name =
                context.session.last_names().find(|name| name.starts_with("Hass") && *name != "HassGetState").unwrap_or("HassTurnOn");
            return Some(execute(
                context,
                vec![Intent::new(name).with("entity_id", entity_id)],
                "affirmative_replay",
                0.83,
                1.0,
                false,
                false,
            ));
        }
    }
    match_custom(tokens, context.text, context.custom, &context.home.registered_intents)
        .map(|intent| execute(context, vec![intent], "custom_sentence", 1.0, 1.0, false, false))
}

pub(super) fn replay_or_decide(
    context: &ParseContext<'_>,
    tokens: &[String],
    mut intents: Vec<Intent>,
    selected_plan: Option<IntentPlan>,
    ranking: &RankingResult,
) -> Draft {
    if intents.is_empty() {
        if let Some(previous) = last_visible(context.session, context.home) {
            if let Some(number) = first_number(tokens) {
                let (name, slot) = if previous.starts_with("climate.") {
                    ("HassClimateSetTemperature", "temperature")
                } else if previous.starts_with("fan.") {
                    ("HassFanSetSpeed", "percentage")
                } else {
                    ("HassLightSet", "brightness")
                };
                intents.push(Intent::new(name).with("entity_id", previous).with(slot, number.to_string()));
            } else if context.catalog.any(tokens, context.catalog.replay_on_off()) {
                let name = if context.catalog.any(tokens, context.catalog.replay_off()) { "HassTurnOff" } else { "HassTurnOn" };
                intents.push(Intent::new(name).with("entity_id", previous));
            }
        } else if context.catalog.any(tokens, context.catalog.on_words()) || context.catalog.any(tokens, context.catalog.off_words()) {
            let named_target = {
                let resolved = crate::parse::resolve::resolve(tokens, context.home, None);
                !resolved.areas.is_empty() || !resolved.entities.is_empty() || !resolved.floors.is_empty()
            };
            if named_target {
                let mut draft = reject(RejectReason::NoTarget, speak_unknown());
                draft.commit.briefing = Some(false);
                return draft;
            }
            let off = context.catalog.any(tokens, context.catalog.off_words());
            return Draft {
                decision: ParseDecision::Clarify { prompt: speak_need_target(off), options: Vec::new() },
                plan: None,
                speech: speak_need_target(off),
                confidence: ranking.confidence,
                margin: ranking.margin,
                selected_candidate_id: None,
                output_candidate: None,
                safety_confirmed: false,
                response_briefing: false,
                competing: false,
                commit: SessionCommit { briefing: Some(false), ..SessionCommit::default() },
                policy_trace: None,
            };
        }
    }
    if intents.is_empty() {
        let reason = if ranking.candidates.is_empty() { RejectReason::NoAction } else { RejectReason::NoTarget };
        let mut draft = reject(reason, speak_unknown());
        draft.commit.briefing = Some(false);
        draft
    } else if let Some(plan) = selected_plan.filter(|plan| plan.intents() == intents) {
        with_competing(
            execute_plan(
                context,
                plan,
                "ranked_policy",
                ranking.selected.as_ref().map(|candidate| candidate.id.clone()),
                ranking.selected.clone(),
                context.session.pending_clarify().is_some(),
                false,
            ),
            ranking.competing,
        )
    } else {
        with_competing(
            execute(
                context,
                intents,
                "ranked_policy",
                ranking.confidence,
                ranking.margin,
                context.session.pending_clarify().is_some(),
                false,
            ),
            ranking.competing,
        )
    }
}

pub(super) fn safety_decision(mut draft: Draft, context: &ParseContext<'_>) -> Draft {
    if !matches!(draft.decision, ParseDecision::Execute) {
        return draft;
    }
    if draft.safety_confirmed {
        let Some(plan) = draft.plan.as_ref() else {
            return invalid_plan(draft, PlanInvalid::Schema);
        };
        return match validate_plan(plan, context.home) {
            Ok(()) => draft,
            Err(reason) => invalid_plan(draft, reason),
        };
    }
    if let Some(plan) = draft.plan.take() {
        let filtered = filter_valid_steps(&plan, context.home);
        if filtered.steps.is_empty() {
            let reason = validate_plan(&plan, context.home).err().unwrap_or(PlanInvalid::MissingTarget);
            draft.plan = Some(plan);
            return invalid_plan(draft, reason);
        }
        if filtered.steps.len() != plan.steps.len() {
            let intents = filtered.intents();
            draft.speech = speak(&intents, context.settings.personality, false, Some(context.home));
            draft.commit.remember = intents;
        }
        draft.confidence = filtered.confidence;
        draft.margin = filtered.margin.max(draft.margin);
        if let Some(candidate) = draft.output_candidate.as_mut() {
            candidate.plan = filtered.clone();
            candidate.score = candidate.score.min(filtered.confidence);
        }
        draft.plan = Some(filtered);
    }
    let Some(plan) = draft.plan.as_ref() else {
        return invalid_plan(draft, PlanInvalid::Schema);
    };
    if let Err(reason) = validate_plan(plan, context.home) {
        return invalid_plan(draft, reason);
    }
    let compiled_risky = context.settings.confirm_risky_actions && requires_confirmation(plan);
    if let Some(action) = super::policy_route::apply_matched_action(context, plan) {
        return action;
    }
    let matched = first_matching_rule(context.policies, plan);
    let seed_matched = first_seed_match(context.policies, plan);
    let policy_trace = overlay_trace(&draft, matched, seed_matched, compiled_risky, "reject");
    if matches!(matched.map(|(_, hit)| hit), Some(PolicyHit::Block)) {
        let mut rejected = reject(RejectReason::Unsafe, speak_unknown());
        rejected.confidence = draft.confidence;
        rejected.margin = draft.margin;
        rejected.plan = draft.plan.clone();
        apply_rule_speech(&mut rejected, context, matched.map(|(rule, _)| rule));
        rejected.plan = None;
        rejected.policy_trace = Some(policy_trace);
        return rejected;
    }
    let risky = match matched.map(|(_, hit)| hit) {
        Some(PolicyHit::Confirm) if context.settings.confirm_risky_actions => true,
        Some(PolicyHit::Allow) if allow_permitted(plan) => false,
        _ => compiled_risky,
    };
    let mut decided = match decide_band(draft.confidence, draft.margin, risky, draft.safety_confirmed, draft.competing) {
        PolicyBand::Execute => draft,
        PolicyBand::Confirm => {
            let candidate_id = draft.selected_candidate_id.clone().unwrap_or_else(|| "selected-000".into());
            let prompt = confirmation_prompt(context);
            draft.decision = ParseDecision::Confirm { prompt: prompt.clone(), candidate_id: candidate_id.clone() };
            draft.speech = prompt.clone();
            draft.commit.remember.clear();
            draft.commit.clear_pending = false;
            draft.commit.confirm = Some((candidate_id, plan.clone(), prompt));
            draft
        }
        PolicyBand::Clarify => {
            let prompt = speak_need_target(false);
            draft.decision = ParseDecision::Clarify { prompt: prompt.clone(), options: Vec::new() };
            draft.speech = prompt;
            draft.plan = None;
            draft.commit.remember.clear();
            draft.commit.clear_pending = false;
            draft
        }
        PolicyBand::Reject => {
            let reason = if risky { RejectReason::Unsafe } else { RejectReason::NoAction };
            let mut rejected = reject(reason, speak_unknown());
            rejected.confidence = draft.confidence;
            rejected.margin = draft.margin;
            rejected
        }
    };
    apply_rule_speech(&mut decided, context, matched.map(|(rule, _)| rule));
    if let (ParseDecision::Confirm { prompt, .. }, Some((_, _, stored))) = (&decided.decision, decided.commit.confirm.as_mut()) {
        *stored = prompt.clone();
    }
    decided.policy_trace = Some(overlay_trace(&decided, matched, seed_matched, compiled_risky, decided.decision.type_name()));
    decided
}

fn overlay_trace(
    draft: &Draft,
    matched: Option<(&crate::types::PolicyRule, PolicyHit)>,
    seed_matched: Option<(&crate::types::PolicyRule, PolicyHit)>,
    compiled_risky: bool,
    band: &str,
) -> PolicyTrace {
    PolicyTrace {
        matched_rule: matched.map(|(rule, _)| rule.id.clone()).or_else(|| seed_matched.map(|(rule, _)| rule.id.clone())),
        hit: matched.map(|(_, hit)| hit.as_str().into()).or_else(|| seed_matched.map(|(_, hit)| hit.as_str().into())),
        compiled_risky,
        payload: matched.and_then(|(rule, _)| rule.payload.clone()).or_else(|| seed_matched.and_then(|(rule, _)| rule.payload.clone())),
        match_node: draft.output_candidate.as_ref().and_then(PolicyTraceMatch::from_candidate),
        seed: seed_matched.map(|(rule, hit)| PolicyTraceLayer::seed(rule.id.clone(), hit.as_str())),
        house: matched.map(|(rule, hit)| PolicyTraceLayer::house(rule.id.clone(), hit.as_str())),
        band: Some(band.into()),
        ..PolicyTrace::default()
    }
}

pub(super) fn reject(reason: RejectReason, speech: String) -> Draft {
    Draft {
        decision: ParseDecision::Reject { reason },
        plan: None,
        speech,
        confidence: 0.0,
        margin: 0.0,
        selected_candidate_id: None,
        output_candidate: None,
        safety_confirmed: false,
        response_briefing: false,
        competing: false,
        commit: SessionCommit::default(),
        policy_trace: None,
    }
}

fn invalid_plan(mut draft: Draft, reason: PlanInvalid) -> Draft {
    draft.decision = match reason {
        PlanInvalid::MissingTarget => ParseDecision::Reject { reason: RejectReason::NoTarget },
        PlanInvalid::UnsafeTarget => ParseDecision::Reject { reason: RejectReason::Unsafe },
        PlanInvalid::Schema => ParseDecision::Error { code: "invalid_plan".into(), message: "Planner emitted an invalid plan".into() },
    };
    draft.plan = None;
    draft.speech = speak_unknown();
    draft.commit.remember.clear();
    draft.commit.clear_pending = false;
    draft
}

pub(super) fn execute(
    context: &ParseContext<'_>,
    intents: Vec<Intent>,
    policy: &str,
    confidence: f64,
    margin: f64,
    clear_pending: bool,
    safety_confirmed: bool,
) -> Draft {
    let mut plan = IntentPlan::from_intents(intents, confidence, &[]);
    plan.margin = margin;
    execute_plan(context, plan, policy, None, None, clear_pending, safety_confirmed)
}

pub(super) fn from_adapter_plan(context: &ParseContext<'_>, plan: IntentPlan, provider: &str) -> Draft {
    execute_plan(context, plan, "semantic_adapter", Some(format!("adapter-{provider}")), None, false, false)
}

fn execute_plan(
    context: &ParseContext<'_>,
    plan: IntentPlan,
    policy: &str,
    selected_candidate_id: Option<String>,
    selected_candidate: Option<IntentCandidate>,
    clear_pending: bool,
    safety_confirmed: bool,
) -> Draft {
    let candidate_id = selected_candidate_id.unwrap_or_else(|| format!("direct-{}", policy.replace('_', "-")));
    let output_candidate = selected_candidate.or_else(|| {
        Some(IntentCandidate {
            id: candidate_id.clone(),
            plan: plan.clone(),
            score: plan.confidence,
            margin: plan.margin,
            policy: policy.into(),
            precedence: 0,
            evidence: plan.evidence.clone(),
        })
    });
    let intents = plan.intents();
    let speech = speak(&intents, context.settings.personality, false, Some(context.home));
    Draft {
        decision: ParseDecision::Execute,
        confidence: plan.confidence,
        margin: plan.margin,
        plan: Some(plan),
        speech,
        selected_candidate_id: Some(candidate_id),
        output_candidate,
        safety_confirmed,
        response_briefing: false,
        competing: false,
        commit: SessionCommit { remember: intents, clear_pending, briefing: Some(false), ..SessionCommit::default() },
        policy_trace: None,
    }
}

pub(super) fn chat(speech: String, response_briefing: bool, next_briefing: bool) -> Draft {
    Draft {
        decision: ParseDecision::Chat,
        plan: None,
        speech,
        confidence: 1.0,
        margin: 1.0,
        selected_candidate_id: None,
        output_candidate: None,
        safety_confirmed: false,
        response_briefing,
        competing: false,
        commit: SessionCommit { briefing: Some(next_briefing), ..SessionCommit::default() },
        policy_trace: None,
    }
}

fn with_competing(mut draft: Draft, competing: bool) -> Draft {
    draft.competing = competing;
    draft
}

pub(super) fn decide_execute_plan(
    home: &crate::types::HomeGraph,
    settings: &crate::types::Settings,
    plan: IntentPlan,
    confidence: f64,
    margin: f64,
    competing: bool,
    overlay: (&[crate::types::PolicyRule], &crate::types::SpeechBank),
) -> Draft {
    let session = crate::session::Session::new();
    let catalog = crate::lang::catalog_for(&settings.languages);
    let context = ParseContext::new("test", home, &session, &[], settings, catalog).with_policies(overlay.0, overlay.1);
    let mut draft = execute_plan(&context, plan, "test", None, None, false, false);
    draft.confidence = confidence;
    draft.margin = margin;
    draft.competing = competing;
    safety_decision(draft, &context)
}

fn affirmative(tokens: &[String], catalog: &crate::lang::Catalog) -> bool {
    !tokens.is_empty() && tokens.iter().all(|token| catalog.is_affirm(token))
}

#[cfg(test)]
#[path = "draft_tests.rs"]
mod tests;
