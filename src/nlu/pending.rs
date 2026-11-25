//! An open question is a small state machine.
//!
//! While a confirmation or an either-or question is open, the next utterance is read in
//! this order: an answer (yes, no, cancel, one of the offered options), a complete new
//! command, which replaces the question, and otherwise the question is asked again. The
//! question expires on its own (`session::PENDING_TTL`) and can be answered once.

use crate::parse::clarify::pick_clarification;
use crate::parse::news::is_news_dismiss;
use crate::parse::respond::{speak_clarify, speak_unknown};
use crate::parse::slots::{fill_pending_timer_duration, intent_with_entity};
use crate::session::{ClarifyState, ConfirmState, PendingInteraction};
use crate::types::{HomeGraph, Intent, ParseDecision, RejectReason};

use crate::parse::action::Action;

use super::answer::{classify, Answer};
use super::binding::action_for_intent;
use super::context::ParseContext;
use super::draft::{execute, execute_plan, reject, Draft, SessionCommit};
use super::evidence::action_hypotheses;

/// The utterance answers the open question. `said` are the words as spoken, `tokens`
/// the normalized ones.
pub(super) fn answer(context: &ParseContext<'_>, said: &[String], tokens: &[String]) -> Option<Draft> {
    let reply = classify(said, &context.catalog);
    match context.session.pending()? {
        PendingInteraction::Confirm(confirm) => match reply {
            Some(Answer::Yes) => Some(confirmed(context, confirm)),
            Some(Answer::No) => Some(dropped(context)),
            None => is_news_dismiss(tokens).then(|| dropped(context)),
        },
        PendingInteraction::Clarify(clarify) => {
            if let Some(intent) = fill_pending_timer_duration(&clarify.template, tokens) {
                return Some(execute(context, vec![intent], "pending_timer_duration", 1.0, 1.0, true, false));
            }
            match reply {
                // "Office or hall?" - "yes" picks neither.
                Some(Answer::Yes) => Some(ask_again(context)),
                Some(Answer::No) => Some(dropped(context)),
                // A sentence with a verb of its own is a command first; see `named_option`.
                None if own_actions(context, tokens).is_empty() => match pick_clarification(tokens, context.session, context.home) {
                    Some(chosen) => Some(chose(context, clarify, &chosen)),
                    None => is_news_dismiss(tokens).then(|| dropped(context)),
                },
                None => None,
            }
        }
    }
}

/// What the utterance itself asks for, read from its verbs.
fn own_actions(context: &ParseContext<'_>, tokens: &[String]) -> Vec<Action> {
    action_hypotheses(tokens, &context.catalog, context.home).into_iter().filter(|row| row.evidence.exact).map(|row| row.action).collect()
}

/// "Turn on the office one" was no command of its own, but it repeats the asked action
/// and names an option. A different verb ("turn off ...") never picks an option.
pub(super) fn named_option(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
    let clarify = context.session.pending_clarify()?;
    let asked = action_for_intent(&clarify.template.name)?;
    if !own_actions(context, tokens).contains(&asked) {
        return None;
    }
    pick_clarification(tokens, context.session, context.home).map(|chosen| chose(context, clarify, &chosen))
}

fn confirmed(context: &ParseContext<'_>, confirm: &ConfirmState) -> Draft {
    execute_plan(context, confirm.plan.clone(), "pending_confirm", Some(confirm.candidate_id.clone()), None, true, true)
}

fn chose(context: &ParseContext<'_>, clarify: &ClarifyState, chosen: &str) -> Draft {
    let template = clarify.template.clone();
    let intent = if context.home.areas.iter().any(|area| area.area_id == chosen) {
        let domain = template.slot("domain").unwrap_or("light").to_string();
        template.with_set("area", chosen).with_set("domain", domain)
    } else {
        intent_with_entity(template, chosen)
    };
    execute(context, vec![intent], "pending_clarify", 1.0, 1.0, true, false)
}

/// "No" or "cancel": the question is closed and nothing runs.
fn dropped(context: &ParseContext<'_>) -> Draft {
    let speech = if context.catalog.news_done.is_empty() { speak_unknown() } else { context.catalog.news_done.to_string() };
    let mut draft = reject(RejectReason::Unsupported, speech);
    draft.commit.clear_pending = true;
    draft
}

/// The same question once more. The session keeps it; its time keeps running.
pub(super) fn ask_again(context: &ParseContext<'_>) -> Draft {
    let (decision, confidence, margin) = match context.session.pending() {
        Some(PendingInteraction::Confirm(confirm)) => (
            ParseDecision::Confirm { prompt: confirm.prompt.clone(), candidate_id: confirm.candidate_id.clone() },
            confirm.plan.confidence,
            confirm.plan.margin,
        ),
        Some(PendingInteraction::Clarify(clarify)) => {
            let prompt =
                if clarify.prompt.is_empty() { speak_clarify(&clarify.options, Some(context.home)) } else { clarify.prompt.clone() };
            (ParseDecision::Clarify { prompt, options: clarify.options.clone() }, 0.0, 0.0)
        }
        None => return reject(RejectReason::NoAction, speak_unknown()),
    };
    let speech = match &decision {
        ParseDecision::Confirm { prompt, .. } | ParseDecision::Clarify { prompt, .. } => prompt.clone(),
        ParseDecision::Execute | ParseDecision::Reject { .. } | ParseDecision::Chat | ParseDecision::Error { .. } => String::new(),
    };
    Draft {
        decision,
        plan: None,
        speech,
        confidence,
        margin,
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

/// A sentence that only makes sense as a reply ("the lamp by the desk") was resolved
/// with the question as context. It counts only if it stays inside the question: the
/// asked action, on one of the offered options.
pub(super) fn stays_inside(steps: &[Intent], clarify: &ClarifyState, home: &HomeGraph) -> bool {
    let offered = |id: &str| clarify.options.iter().any(|option| option == id);
    !steps.is_empty()
        && !clarify.options.is_empty()
        && steps.iter().all(|step| {
            step.name == clarify.template.name
                && (step.slot("entity_id").is_some_and(|id| {
                    offered(id) || home.entities.iter().any(|entity| entity.entity_id == id && entity.area.as_deref().is_some_and(offered))
                }) || step.slot("area").is_some_and(offered))
        })
}
