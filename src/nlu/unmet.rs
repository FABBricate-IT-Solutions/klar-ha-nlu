//! A command is not a question.
//!
//! "Set the fan to high" asks for a change. Where no step can make that change, the
//! sentence is refused. Answering with the state of the fan would treat an order as a
//! question and leave the user thinking it was carried out.
//!
//! Package 2.7 also refuses vacuum pause/stop (not dock) and alarm arm/disarm — HA has
//! no Assist intent Klar wires yet, so GetState must not stand in for them.

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
    if draft.safety_confirmed {
        return draft;
    }
    let tokens = tokenize(context.text);
    if unsupported_spoken_command(&tokens, context.catalog, context.home) {
        return refuse(draft);
    }
    if !matches!(draft.decision, ParseDecision::Execute) {
        return draft;
    }
    let reads = draft.plan.as_ref().is_some_and(|plan| plan.intents().iter().any(|step| step.name == "HassGetState"));
    let cat = context.catalog;
    if !reads || question_mark(context.text) || question_ending(context.text, cat) {
        return draft;
    }
    // Every clause orders something, so no clause asked for the state that would be read.
    let clauses = split_clauses(&tokens, context.home);
    let orders = |tokens: &Vec<String>| commands(tokens, cat) && !is_query_token(tokens) && !Clause::new(tokens).read(cat).asked;
    if clauses.is_empty() || !clauses.iter().all(orders) {
        return draft;
    }
    refuse(draft)
}

fn refuse(draft: Draft) -> Draft {
    let mut refused = reject(RejectReason::Unsupported, speak_unknown());
    refused.commit.briefing = Some(false);
    refused.commit.clear_pending = draft.commit.clear_pending;
    refused
}

/// Vacuum pause/stop (P-30) and alarm arm/disarm (P-36): clear Unsupported, never GetState.
fn unsupported_spoken_command(tokens: &[String], cat: &Catalog, home: &crate::types::HomeGraph) -> bool {
    if is_query_token(tokens) || question_mark(&tokens.join(" ")) {
        return false;
    }
    if alarm_arm_words(tokens) {
        return true;
    }
    vacuum_pause_or_stop(tokens, cat, home)
}

fn alarm_arm_words(tokens: &[String]) -> bool {
    // "armed" alone is a state word in questions; the query check above keeps those.
    tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "arm" | "arms" | "disarm" | "disarms" | "scharf" | "unscharf" | "scharfschalten" | "entschaerfen" | "entschaerfe"
        )
    })
}

fn vacuum_pause_or_stop(tokens: &[String], cat: &Catalog, home: &crate::types::HomeGraph) -> bool {
    if crate::parse::infer::dock_hint(tokens) {
        return false;
    }
    if !tokens.iter().any(|token| matches!(cat.verb(token), Some(VerbKind::Pause | VerbKind::Stop))) {
        return false;
    }
    let vacuum_word = cat.any(tokens, cat.vacuum_nouns())
        || tokens.iter().any(|token| matches!(cat.verb(token), Some(VerbKind::Vacuum | VerbKind::VacuumNoun)));
    if vacuum_word {
        return true;
    }
    // "pause tv wohnzimmer" must stay media: compound vacuum names expand to bare area
    // aliases ("wohnzimmer"), which must not steal a media-cued pause/stop.
    if media_pause_cued(tokens, cat) {
        return false;
    }
    names_vacuum(tokens, home)
}

fn media_pause_cued(tokens: &[String], cat: &Catalog) -> bool {
    cat.any(tokens, cat.media_nouns()) || crate::home::roles::tv_asked(tokens)
}

fn names_vacuum(tokens: &[String], home: &crate::types::HomeGraph) -> bool {
    use crate::parse::normalize::compact;
    // Home-config loading splits "wohnzimmer staubsauger" into alias "wohnzimmer".
    // Matching that alone would refuse every pause scoped to the living room.
    let area_labels: std::collections::HashSet<String> = home
        .areas
        .iter()
        .flat_map(|area| std::iter::once(area.name.as_str()).chain(area.aliases.iter().map(String::as_str)))
        .map(compact)
        .filter(|label| !label.is_empty())
        .collect();
    home.entities.iter().any(|entity| {
        entity.domain == "vacuum"
            && std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str)).any(|label| {
                let folded = compact(label);
                !folded.is_empty() && !area_labels.contains(&folded) && tokens.iter().any(|token| compact(token) == folded)
            })
    })
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
            | VerbKind::Pause
    )
}
