use crate::lang::{catalog, VerbKind};

use super::Action;

pub(super) fn timer_kind(tokens: &[String]) -> Action {
    let cat = catalog();
    if cat.any(tokens, cat.timer_pause()) || timer_hold(tokens) {
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

pub(super) fn timer_hold(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "hold" | "halt"))
}

pub(super) fn timer_decrease(tokens: &[String]) -> bool {
    !catalog().any(tokens, catalog().timer_add())
        && tokens.iter().any(|token| match token.as_str() {
            "decrease" | "decreased" | "reduce" | "reduced" | "subtract" | "subtracted" | "minus" | "shorten" | "shortened"
            | "verringern" | "verringere" | "reduzieren" | "reduziere" | "weniger" | "kuerzen" | "kuerze" | "abziehen" => true,
            "remove" | "removed" | "cut" | "take" | "knock" | "down" => crate::parse::numbers::first_number(tokens).is_some(),
            _ => matches!(catalog().verb(token), Some(VerbKind::Lower))
                || (matches!(catalog().verb(token), Some(VerbKind::Down)) && crate::parse::numbers::first_number(tokens).is_some()),
        })
}
