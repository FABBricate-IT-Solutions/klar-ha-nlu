use crate::parse::chat::{is_ood, wants_llm};
use crate::parse::clause_support::last_visible;
use crate::parse::compound::CompoundSplit;
use crate::parse::infer::{looks_like_correction, match_custom};
use crate::parse::news::{briefing_followup, is_news_dismiss, news_turn, NewsTurn};
use crate::parse::normalize::tokenize;
use crate::parse::numbers::{first_number, sets_misread_number};
use crate::parse::respond::{speak, speak_correction, speak_need_target, speak_unknown};
use crate::types::{first_seed_match, Intent, IntentCandidate, IntentPlan, ParseDecision, RejectReason};

use super::context::ParseContext;
use super::decision::{decide_band, PolicyBand};
use super::ranking::RankingResult;
use super::risk::{judge, requires_confirmation};
use super::speech::{apply_rule_speech, mark_competing};
use super::validation::{fail_closed_intent, filter_valid_steps, missing_timer_duration, validate_plan, PlanInvalid};

#[derive(Default)]
pub(super) struct SessionCommit {
    pub remember: Vec<Intent>,
    pub clarify: Option<(Vec<String>, Intent)>,
    pub confirm: Option<(String, IntentPlan, String)>,
    pub clear_pending: bool,
    pub briefing: Option<bool>,
    pub news_topic: Option<String>,
    pub replace_news_topic: bool,
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
    /// Reverses an action that itself needed a confirmation, so it asks as well.
    pub inherits_risk: bool,
    pub response_briefing: bool,
    pub competing: bool,
    /// Local clock, weekday or date speech from the household route.
    pub clock_answer: bool,
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
    if let Some(draft) = super::reminder::route(context, raw) {
        return Some(draft);
    }
    if let Some(draft) = super::household::route(context, tokens) {
        return Some(draft);
    }
    if let Some(draft) = super::policy_route::route_phrase(context) {
        return Some(draft);
    }
    match news_turn(raw, context.home, context.session).or_else(|| news_turn(tokens, context.home, context.session)) {
        Some(NewsTurn::Fresh(topic)) => {
            let mut draft = chat(context.catalog.news_intro.to_string(), true, true);
            draft.commit.replace_news_topic = true;
            draft.commit.news_topic = topic;
            return Some(draft);
        }
        Some(NewsTurn::Follow) => return Some(chat(String::new(), true, true)),
        None => {}
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
        return Some(reject(RejectReason::OutOfDomain, speak_unknown()));
    }
    if looks_like_correction(tokens) {
        let mut draft = reject(RejectReason::Unsupported, speak_correction());
        draft.commit.mark_wrong = true;
        return Some(draft);
    }
    None
}

/// A sentence the user taught as a custom rule. A bare "yes" with no open question
/// answers nothing and never replays the last device.
pub(super) fn route_custom(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
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
            if previous.starts_with("climate.") && crate::parse::quantity::is_relative_temp(tokens) {
                if let Some(step) = crate::parse::quantity::temperature_step(tokens) {
                    intents.push(
                        Intent::new("HassClimateSetTemperature")
                            .with("entity_id", previous)
                            .with("temperature_step", crate::parse::quantity::format_quantity(step)),
                    );
                }
            } else if let Some(number) = first_number(tokens) {
                let (name, slot) = if previous.starts_with("climate.") {
                    ("HassClimateSetTemperature", "temperature")
                } else if previous.starts_with("fan.") {
                    ("HassFanSetSpeed", "percentage")
                } else {
                    ("HassLightSet", "brightness")
                };
                let mut intent = Intent::new(name).with("entity_id", previous).with(slot, number.to_string());
                if name == "HassClimateSetTemperature" {
                    crate::units::bind_set_temp(&mut intent, tokens, context.settings.unit_system);
                }
                intents.push(intent);
            } else if context.catalog.any(tokens, context.catalog.replay_on_off()) {
                let name = if context.catalog.any(tokens, context.catalog.replay_off()) { "HassTurnOff" } else { "HassTurnOn" };
                intents.push(Intent::new(name).with("entity_id", previous));
            }
        } else if crate::parse::clause_session::which_lights_query(tokens)
            && !crate::parse::clause_session::session_has_light_areas(context.session, context.home)
        {
            // Ranking sometimes drops the list-status candidate; keep the ask (P-13).
            // Skip when prior light rooms exist — SessionAreas answers those.
            let mut intent = Intent::new("HassGetState").with("domain", "light").with("response", "list");
            if context.catalog.any(tokens, context.catalog.on_words()) {
                intent = intent.with("state", "on");
            } else if context.catalog.any(tokens, context.catalog.off_words()) {
                intent = intent.with("state", "off");
            }
            intents.push(intent);
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
                inherits_risk: false,
                response_briefing: false,
                competing: false,
                clock_answer: false,
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
        mark_competing(
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
        mark_competing(
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
    if draft.plan.as_ref().is_some_and(|plan| missing_timer_duration(plan, context.text)) {
        let prompt = context.catalog.speech().timer_how_long.to_string();
        draft.commit.clarify = draft.plan.as_ref().and_then(|plan| plan.intents().into_iter().next()).map(|intent| (Vec::new(), intent));
        draft.commit.remember.clear();
        draft.decision = ParseDecision::Clarify { prompt: prompt.clone(), options: Vec::new() };
        draft.speech = prompt;
        draft.plan = None;
        return draft;
    }
    // "dix-sept degrés" where only ten was understood: asking again beats setting ten.
    if !draft.safety_confirmed && draft.plan.as_ref().is_some_and(|plan| sets_misread_number(plan, &tokenize(context.text))) {
        return reject(RejectReason::Unsupported, speak_unknown());
    }
    let operator = super::policy_route::operator_script(&draft);
    if draft.safety_confirmed {
        let Some(plan) = draft.plan.as_ref() else {
            return invalid_plan(draft, PlanInvalid::Schema);
        };
        return match validate_plan(plan, context.home) {
            Err(reason) if !operator => invalid_plan(draft, reason),
            _ => draft,
        };
    }
    if let Some(asked) = draft.plan.take_if(|_| !operator) {
        // A sentence the user taught may name a device that is otherwise not a target.
        let taught = draft.output_candidate.as_ref().is_some_and(|candidate| candidate.policy == "custom_sentence");
        let cleared = if taught { None } else { super::reach::without_ignored(&asked, context.home) };
        if cleared.as_ref().is_some_and(|plan| plan.steps.is_empty()) {
            draft.plan = Some(asked);
            return invalid_plan(draft, PlanInvalid::MissingTarget);
        }
        let changed = cleared.is_some();
        let plan = cleared.unwrap_or(asked);
        let filtered = filter_valid_steps(&plan, context.home);
        if filtered.steps.is_empty() {
            let reason = validate_plan(&plan, context.home).err().unwrap_or(PlanInvalid::MissingTarget);
            draft.plan = Some(plan);
            return invalid_plan(draft, reason);
        }
        if changed || filtered.steps.len() != plan.steps.len() {
            if filtered.steps.len() != plan.steps.len() && plan.steps.iter().any(|step| fail_closed_intent(&step.intent)) {
                draft.plan = Some(plan);
                return invalid_plan(draft, PlanInvalid::UnsafeTarget);
            }
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
    if let Some(reason) = validate_plan(plan, context.home).err().filter(|_| !operator) {
        return invalid_plan(draft, reason);
    }
    // A phrase rule that starts a script already names the rule that fired; keep that
    // trace when the script runs, Home Assistant reads the hit from it.
    let phrase_action = draft.policy_trace.clone();
    let confirm_on = context.settings.confirm_risky_actions;
    let compiled_risky = confirm_on && (requires_confirmation(plan, context.home) || draft.inherits_risk);
    if let Some(action) = super::policy_route::apply_matched_action(context, plan) {
        return action;
    }
    let verdict = judge(context.policies, plan, context.home, confirm_on, draft.inherits_risk);
    let (matched, risky) = (verdict.rule, verdict.risky);
    let seed_matched = first_seed_match(context.policies, plan);
    let policy_trace = super::policy_route::overlay_trace(&draft, matched, seed_matched, compiled_risky, "reject");
    if verdict.blocked {
        let mut rejected = reject(RejectReason::Unsafe, speak_unknown());
        rejected.confidence = draft.confidence;
        rejected.margin = draft.margin;
        rejected.plan = draft.plan.clone();
        apply_rule_speech(&mut rejected, context, matched.map(|(rule, _)| rule));
        rejected.plan = None;
        rejected.policy_trace = Some(policy_trace);
        return rejected;
    }
    let mut decided = match decide_band(draft.confidence, draft.margin, risky, draft.safety_confirmed, draft.competing) {
        PolicyBand::Execute => draft,
        PolicyBand::Confirm => {
            let candidate_id = draft.selected_candidate_id.clone().unwrap_or_else(|| "selected-000".into());
            let prompt = super::ask::confirmation_prompt(context, plan);
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
    decided.policy_trace = match phrase_action {
        Some(trace) if matches!(decided.decision, ParseDecision::Execute) => Some(trace),
        _ => Some(super::policy_route::overlay_trace(&decided, matched, seed_matched, compiled_risky, decided.decision.type_name())),
    };
    decided
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
        inherits_risk: false,
        response_briefing: false,
        competing: false,
        clock_answer: false,
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

pub(super) fn execute_plan(
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
        inherits_risk: false,
        response_briefing: false,
        competing: false,
        clock_answer: false,
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
        inherits_risk: false,
        response_briefing,
        competing: false,
        clock_answer: false,
        commit: SessionCommit { briefing: Some(next_briefing), ..SessionCommit::default() },
        policy_trace: None,
    }
}

pub(super) fn clock_chat(speech: String) -> Draft {
    let mut draft = chat(speech, false, false);
    draft.clock_answer = true;
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
    let catalog = crate::lang::bound_catalog(settings);
    let context = ParseContext::new("test", home, &session, &[], settings, catalog).with_policies(overlay.0, overlay.1);
    let mut draft = execute_plan(&context, plan, "test", None, None, false, false);
    draft.confidence = confidence;
    draft.margin = margin;
    draft.competing = competing;
    safety_decision(draft, &context)
}

#[cfg(test)]
#[path = "draft_tests.rs"]
mod tests;
