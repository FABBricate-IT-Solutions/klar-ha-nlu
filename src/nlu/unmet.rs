//! A command is not a question.
//!
//! "Set the fan to high" asks for a change. Where no step can make that change, the
//! sentence is refused. Answering with the state of the fan would treat an order as a
//! question and leave the user thinking it was carried out.

use super::context::ParseContext;
use super::draft::{reject, Draft};
use crate::lang::{Catalog, VerbKind};
use crate::parse::action::is_query_token;
use crate::parse::mood::{question_ending, question_mark, Clause};
use crate::parse::normalize::tokenize;
use crate::parse::respond::speak_unknown;
use crate::parse::split::split_clauses;
use crate::types::{ParseDecision, RejectReason};

pub(super) fn guard(context: &ParseContext<'_>, draft: Draft) -> Draft {
    if !matches!(draft.decision, ParseDecision::Execute) || draft.safety_confirmed {
        return draft;
    }
    let reads = draft.plan.as_ref().is_some_and(|plan| plan.intents().iter().any(|step| step.name == "HassGetState"));
    let cat = context.catalog;
    if !reads || question_mark(context.text) || question_ending(context.text, cat) {
        return draft;
    }
    // Every clause orders something, so no clause asked for the state that would be read.
    let clauses = split_clauses(&tokenize(context.text), context.home);
    let orders = |tokens: &Vec<String>| commands(tokens, cat) && !is_query_token(tokens) && !Clause::new(tokens).read(cat).asked;
    if clauses.is_empty() || !clauses.iter().all(orders) {
        return draft;
    }
    let mut refused = reject(RejectReason::Unsupported, speak_unknown());
    refused.commit.briefing = Some(false);
    refused.commit.clear_pending = draft.commit.clear_pending;
    refused
}

/// The clause carries a verb that orders a change. A particle ("on", "zu") also describes
/// a state, so it does not count.
fn commands(tokens: &[String], cat: &Catalog) -> bool {
    tokens.iter().any(|token| !cat.is_particle(token) && cat.verb(token).is_some_and(orders_change))
}

fn orders_change(kind: VerbKind) -> bool {
    matches!(
        kind,
        VerbKind::On
            | VerbKind::Off
            | VerbKind::Open
            | VerbKind::OpenDoor
            | VerbKind::Close
            | VerbKind::Up
            | VerbKind::Down
            | VerbKind::Lower
            | VerbKind::Stop
            | VerbKind::Toggle
            | VerbKind::Dim
            | VerbKind::Switch
            | VerbKind::Lock
            | VerbKind::CloseLock
            | VerbKind::Unlock
            | VerbKind::Set
            | VerbKind::Flick
            | VerbKind::Make
    )
}
