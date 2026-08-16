use crate::parse::chat::{briefing_followup, is_news, is_news_dismiss, is_ood, wants_llm};
use crate::parse::clause_support::last_visible;
use crate::parse::compound::CompoundSplit;
use crate::parse::infer::{looks_like_correction, match_custom, pick_clarification};
use crate::parse::numbers::first_number;
use crate::parse::respond::{speak, speak_correction, speak_need_target, speak_unknown};
use crate::parse::slots::intent_with_entity;
use crate::types::{Intent, IntentCandidate, IntentPlan, ParseDecision, RejectReason};

use super::context::ParseContext;
use super::decision::{decide_band, PolicyBand};
use super::legacy;
use super::ranking::RankingResult;
use super::validation::{filter_valid_steps, requires_confirmation, validate_plan, PlanInvalid};

#[derive(Default)]
pub(super) struct SessionCommit {
    pub remember: Vec<Intent>,
    pub clarify: Option<(Vec<String>, Intent)>,
    pub confirm: Option<(String, IntentPlan, String)>,
    pub clear_pending: bool,
    pub briefing: Option<bool>,
    pub mark_wrong: bool,
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
}

pub(super) fn route_special(context: &ParseContext<'_>, raw: &[String], split: &CompoundSplit) -> Option<Draft> {
    let tokens = &split.tokens;
    if tokens.is_empty() {
        return Some(reject(RejectReason::EmptyInput, speak_unknown()));
    }
    if context.text.chars().any(|character| character.is_control() && !character.is_whitespace()) {
        return Some(reject(RejectReason::InvalidInput, speak_unknown()));
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
        return Some(chat(String::new(), true, true));
    }
    if wants_llm(raw, context.home) || wants_llm(tokens, context.home) {
        return Some(chat(String::new(), context.session.briefing, context.session.briefing));
    }
    if is_ood(raw, context.home) || is_ood(tokens, context.home) {
        return Some(reject(RejectReason::NoAction, speak_unknown()));
    }
    if looks_like_correction(tokens) {
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
        });
    }
    if context.session.pending_clarify().is_some() {
        let picked = pick_clarification(tokens, context.session);
        if let Some(chosen) = picked {
            let template = context.session.pending_clarify()?.template.clone();
            let intent = if context.home.areas.iter().any(|area| area.area_id == chosen) {
                template.with("area", &chosen).with("domain", "light")
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
    match_custom(tokens, context.text, context.custom)
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
            if let Some(number) = legacy::with_catalog(context.catalog, || first_number(tokens)) {
                let (name, slot) = if previous.starts_with("climate.") {
                    ("HassClimateSetTemperature", "temperature")
                } else if previous.starts_with("fan.") {
                    ("HassFanSetSpeed", "percentage")
                } else {
                    ("HassLightSet", "brightness")
                };
                intents.push(Intent::new(name).with("entity_id", previous).with(slot, number.to_string()));
            } else if context.catalog.any(tokens, &context.catalog.replay_on_off) {
                let name = if context.catalog.any(tokens, &context.catalog.replay_off) { "HassTurnOff" } else { "HassTurnOn" };
                intents.push(Intent::new(name).with("entity_id", previous));
            }
        } else if context.catalog.any(tokens, &context.catalog.on_words) || context.catalog.any(tokens, &context.catalog.off_words) {
            let off = context.catalog.any(tokens, &context.catalog.off_words);
            return Draft {
                decision: ParseDecision::Clarify {
                    prompt: legacy::with_catalog(context.catalog, || speak_need_target(off)),
                    options: Vec::new(),
                },
                plan: None,
                speech: legacy::with_catalog(context.catalog, || speak_need_target(off)),
                confidence: ranking.confidence,
                margin: ranking.margin,
                selected_candidate_id: None,
                output_candidate: None,
                safety_confirmed: false,
                response_briefing: false,
                competing: false,
                commit: SessionCommit { briefing: Some(false), ..SessionCommit::default() },
            };
        }
    }
    if intents.is_empty() {
        let reason = if ranking.candidates.is_empty() { RejectReason::NoAction } else { RejectReason::NoTarget };
        let mut draft = reject(reason, legacy::with_catalog(context.catalog, speak_unknown));
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
            return invalid_plan(draft, PlanInvalid::Schema, context.catalog);
        };
        return match validate_plan(plan, context.home) {
            Ok(()) => draft,
            Err(reason) => invalid_plan(draft, reason, context.catalog),
        };
    }
    if let Some(plan) = draft.plan.take() {
        let filtered = filter_valid_steps(&plan, context.home);
        if filtered.steps.is_empty() {
            let reason = validate_plan(&plan, context.home).err().unwrap_or(PlanInvalid::MissingTarget);
            draft.plan = Some(plan);
            return invalid_plan(draft, reason, context.catalog);
        }
        if filtered.steps.len() != plan.steps.len() {
            let intents = filtered.intents();
            draft.speech =
                legacy::with_catalog(context.catalog, || speak(&intents, context.settings.personality, false, Some(context.home)));
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
        return invalid_plan(draft, PlanInvalid::Schema, context.catalog);
    };
    if let Err(reason) = validate_plan(plan, context.home) {
        return invalid_plan(draft, reason, context.catalog);
    }
    let risky = context.settings.confirm_risky_actions && requires_confirmation(plan);
    match decide_band(draft.confidence, draft.margin, risky, draft.safety_confirmed, draft.competing) {
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
            let prompt = legacy::with_catalog(context.catalog, || speak_need_target(false));
            draft.decision = ParseDecision::Clarify { prompt: prompt.clone(), options: Vec::new() };
            draft.speech = prompt;
            draft.plan = None;
            draft.commit.remember.clear();
            draft.commit.clear_pending = false;
            draft
        }
        PolicyBand::Reject => {
            let reason = if risky { RejectReason::Unsafe } else { RejectReason::NoAction };
            let mut rejected = reject(reason, legacy::with_catalog(context.catalog, speak_unknown));
            rejected.confidence = draft.confidence;
            rejected.margin = draft.margin;
            rejected
        }
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
    }
}

fn invalid_plan(mut draft: Draft, reason: PlanInvalid, catalog: &'static crate::lang::Catalog) -> Draft {
    draft.decision = match reason {
        PlanInvalid::MissingTarget => ParseDecision::Reject { reason: RejectReason::NoTarget },
        PlanInvalid::UnsafeTarget => ParseDecision::Reject { reason: RejectReason::Unsafe },
        PlanInvalid::Schema => ParseDecision::Error { code: "invalid_plan".into(), message: "Planner emitted an invalid plan".into() },
    };
    draft.plan = None;
    draft.speech = legacy::with_catalog(catalog, speak_unknown);
    draft.commit.remember.clear();
    draft.commit.clear_pending = false;
    draft
}

fn confirmation_prompt(context: &ParseContext<'_>) -> String {
    match context.catalog.langs.first().copied().unwrap_or(crate::lang::LangId::De) {
        crate::lang::LangId::De => "Soll ich das wirklich ausführen?".into(),
        crate::lang::LangId::En => "Should I really do that?".into(),
    }
}

fn execute(
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
    let speech = legacy::with_catalog(context.catalog, || speak(&intents, context.settings.personality, false, Some(context.home)));
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
    }
}

fn chat(speech: String, response_briefing: bool, next_briefing: bool) -> Draft {
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
) -> Draft {
    let session = crate::session::Session::new();
    let catalog = crate::lang::catalog_for(&settings.languages);
    let context = ParseContext::new("test", home, &session, &[], settings, catalog);
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
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::lang::catalog_for;
    use crate::session::Session;
    use crate::types::{Intent, IntentPlan, Settings};

    fn decide(confidence: f64, margin: f64, competing: bool, risky_lock: bool) -> Draft {
        let home = default_home();
        let session = Session::new();
        let settings = Settings::default();
        let catalog = catalog_for(&["de".into()]);
        let context = ParseContext::new("test", &home, &session, &[], &settings, catalog);
        let entity = if risky_lock { "lock.wohnungstuer" } else { "light.wohnzimmer" };
        let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", entity)], confidence, &[]);
        let mut draft = execute_plan(&context, plan, "test", None, None, false, false);
        draft.confidence = confidence;
        draft.margin = margin;
        draft.competing = competing;
        safety_decision(draft, &context)
    }

    #[test]
    fn competing_low_margin_clarifies_instead_of_execute() {
        let decided = decide(0.92, 0.02, true, false);
        assert!(matches!(decided.decision, ParseDecision::Clarify { .. }), "{:#?}", decided.decision);
        assert!(decided.plan.is_none());
    }

    #[test]
    fn confidence_between_clarify_and_execute_does_not_emit_a_plan() {
        let decided = decide(0.75, 1.0, false, false);
        assert!(matches!(decided.decision, ParseDecision::Clarify { .. }), "{:#?}", decided.decision);
        assert!(decided.plan.is_none());
    }

    #[test]
    fn risky_plan_at_confirm_band_confirms() {
        let decided = decide(0.65, 1.0, false, true);
        assert!(matches!(decided.decision, ParseDecision::Confirm { .. }), "{:#?}", decided.decision);
        assert!(decided.commit.confirm.is_some());
    }
}
