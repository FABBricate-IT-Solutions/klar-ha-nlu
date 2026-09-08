use crate::lang::{catalog, VerbKind};

use super::Action;

pub(super) fn timer_kind(tokens: &[String]) -> Action {
    let cat = catalog();
    if cat.any(tokens, cat.timer_pause()) {
        Action::TimerPause
    } else if cat.any(tokens, cat.playback_resume()) {
        Action::TimerStart
    } else if cat.any(tokens, cat.timer_add()) {
        Action::TimerAdd
    } else if timer_decrease(tokens) {
        Action::TimerRemove
    } else if cat.any(tokens, cat.timer_cancel()) || tokens.iter().any(|token| matches!(token.as_str(), "remove" | "removed")) {
        Action::TimerCancel
    } else {
        Action::TimerStart
    }
}

pub(super) fn timer_decrease(tokens: &[String]) -> bool {
    let cat = catalog();
    if cat.any(tokens, cat.timer_add()) {
        return false;
    }
    let numbered = crate::parse::numbers::first_number(tokens).is_some();
    tokens.iter().any(|token| match token.as_str() {
        "minus" => true,
        "remove" | "removed" => numbered,
        _ => {
            cat.timer_remove().contains(token.as_str())
                || matches!(cat.verb(token), Some(VerbKind::Lower))
                || (numbered && matches!(cat.verb(token), Some(VerbKind::Down | VerbKind::Dim)))
        }
    })
}
