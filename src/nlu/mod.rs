mod binding;
mod context;
mod decision;
mod draft;
mod evidence;
mod legacy;
mod pipeline;
mod ranking;
pub mod semantic;
mod validation;

use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, IntentPlan, ParseDecision, ParseOutcome, ParseResult, Settings};

pub use context::ParseContext;

/// Production `safety_decision` on an execute draft. Used by calibration tests.
pub fn safety_decide(
    home: &HomeGraph,
    settings: &Settings,
    plan: IntentPlan,
    confidence: f64,
    margin: f64,
    competing: bool,
) -> (ParseDecision, Option<IntentPlan>) {
    let draft = draft::decide_execute_plan(home, settings, plan, confidence, margin, competing);
    (draft.decision, draft.plan)
}

pub fn parse(text: &str, home: &HomeGraph, session: &mut Session, custom: &[CustomSentence], settings: &Settings) -> ParseOutcome {
    let catalog = crate::lang::catalog_for(&settings.languages);
    if catalog.pack_intents.is_empty() {
        let context = ParseContext::new(text, home, session, custom, settings, catalog);
        return pipeline::run(context).commit(session);
    }
    let mut sentences = custom.to_vec();
    sentences.extend(catalog.pack_intents.iter().cloned());
    let context = ParseContext::new(text, home, session, &sentences, settings, catalog);
    pipeline::run(context).commit(session)
}

pub fn parse_compatible(
    text: &str,
    home: &HomeGraph,
    session: &mut Session,
    custom: &[CustomSentence],
    settings: &Settings,
) -> ParseResult {
    legacy_result(parse(text, home, session, custom, settings))
}

pub fn legacy_result(outcome: ParseOutcome) -> ParseResult {
    let intents = match &outcome.decision {
        ParseDecision::Execute => outcome.plan.as_ref().map_or_else(Vec::new, |plan| plan.intents()),
        ParseDecision::Clarify { .. }
        | ParseDecision::Confirm { .. }
        | ParseDecision::Reject { .. }
        | ParseDecision::Chat
        | ParseDecision::Error { .. } => Vec::new(),
    };
    let clarify = match &outcome.decision {
        ParseDecision::Clarify { .. } | ParseDecision::Confirm { .. } => true,
        ParseDecision::Execute | ParseDecision::Reject { .. } | ParseDecision::Chat | ParseDecision::Error { .. } => false,
    };
    let chat = match &outcome.decision {
        ParseDecision::Chat => true,
        ParseDecision::Execute
        | ParseDecision::Clarify { .. }
        | ParseDecision::Confirm { .. }
        | ParseDecision::Reject { .. }
        | ParseDecision::Error { .. } => false,
    };
    ParseResult {
        text: outcome.text,
        intents,
        speech: outcome.speech,
        clarify,
        conversation_id: outcome.conversation_id,
        chat,
        briefing: outcome.briefing,
    }
}
