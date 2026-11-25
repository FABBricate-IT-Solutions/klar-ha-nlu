use crate::lang::{catalog, VerbKind};

use super::{timer_ops, Action};

/// Remove wins over tick-off, which wins over add: "remove X from the list" must never add "remove X".
/// An explicit "add …" keeps ListAdd even when the item contains "take out" (P-18).
pub(super) fn list_action(tokens: &[String]) -> Action {
    let cat = catalog();
    if tokens.iter().any(|token| matches!(cat.verb(token), Some(VerbKind::Add))) {
        return Action::ListAdd;
    }
    if tokens.iter().any(|token| matches!(cat.verb(token), Some(VerbKind::ListRemove | VerbKind::Stop))) {
        Action::ListRemove
    } else if cat.any(tokens, cat.list_complete()) {
        Action::ListComplete
    } else {
        Action::ListAdd
    }
}

pub(super) fn has_list_noun(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, cat.list_nouns()) || cat.any(tokens, cat.chores())
}

pub(super) fn add_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if has_list_noun(tokens) {
        Some(list_action(tokens))
    } else if cat.any(tokens, cat.timer_nouns()) {
        Some(if timer_ops::timer_decrease(tokens) {
            Action::TimerRemove
        } else if cat.any(tokens, cat.timer_pause()) {
            Action::TimerPause
        } else {
            Action::TimerAdd
        })
    } else if tokens.iter().any(|token| matches!(cat.verb(token), Some(VerbKind::Add))) && bare_list_add(tokens) {
        // "add milk" with no list noun still means the shopping list (P-18).
        Some(Action::ListAdd)
    } else {
        None
    }
}

fn bare_list_add(tokens: &[String]) -> bool {
    let cat = catalog();
    !cat.any(tokens, cat.light_nouns())
        && !cat.any(tokens, cat.cover_nouns())
        && !cat.any(tokens, cat.fan_nouns())
        && !cat.any(tokens, cat.climate_nouns())
        && !cat.any(tokens, cat.media_nouns())
        && !cat.any(tokens, cat.lock_nouns())
        && !cat.any(tokens, cat.vacuum_nouns())
}
