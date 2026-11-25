use crate::lang::catalog;

use super::list_ops::has_list_noun;
use super::{
    has_climate_noun, has_cover_noun, has_fan_noun, has_light_noun, has_lock_or_door, has_media_noun, is_garage_cover, is_query_token,
    set_by_number, timer_ops, Action,
};

pub(super) fn open_action(tokens: &[String]) -> Option<Action> {
    if has_cover_noun(tokens) {
        Some(if crate::parse::infer::cover_halfway(tokens) { Action::CoverSet } else { Action::CoverOpen })
    } else if catalog().any(tokens, catalog().lock_nouns()) {
        Some(Action::Unlock)
    } else {
        Some(Action::On)
    }
}

pub(super) fn open_door_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, cat.door_nouns()) || cat.any(tokens, cat.lock_nouns()) {
        Some(Action::Unlock)
    } else if has_cover_noun(tokens) {
        Some(Action::CoverOpen)
    } else {
        Some(Action::On)
    }
}

pub(super) fn off_action(tokens: &[String]) -> Option<Action> {
    if catalog().any(tokens, catalog().lock_nouns()) {
        Some(Action::Unlock)
    } else {
        Some(Action::Off)
    }
}

pub(super) fn up_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, cat.close_words()) {
        None
    } else if crate::parse::numbers::first_number(tokens).is_some() {
        Some(set_by_number(tokens))
    } else if has_cover_noun(tokens) || tokens.iter().any(|x| x == "roll") {
        Some(Action::CoverOpen)
    } else {
        Some(Action::On)
    }
}

pub(super) fn down_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, cat.list_down()) || cat.any(tokens, cat.timer_add()) {
        None
    } else if cat.any(tokens, cat.timer_nouns()) && timer_ops::timer_decrease(tokens) {
        Some(Action::TimerRemove)
    } else if crate::parse::numbers::first_number(tokens).is_some() {
        Some(set_by_number(tokens))
    } else if has_cover_noun(tokens) {
        Some(Action::CoverClose)
    } else {
        Some(Action::Off)
    }
}

pub(super) fn auf_action(tokens: &[String]) -> Option<Action> {
    if crate::parse::numbers::first_number(tokens).is_none() && !crate::parse::infer::cover_halfway(tokens) {
        if has_cover_noun(tokens) {
            Some(Action::CoverOpen)
        } else if has_lock_or_door(tokens) {
            Some(Action::Unlock)
        } else {
            None
        }
    } else if has_climate_noun(tokens) {
        Some(Action::SetTemp)
    } else if has_light_noun(tokens) {
        Some(Action::SetLight)
    } else if has_fan_noun(tokens) {
        Some(Action::FanSpeed)
    } else if has_cover_noun(tokens) {
        Some(Action::CoverSet)
    } else {
        None
    }
}

pub(super) fn stop_action(tokens: &[String]) -> Option<Action> {
    if catalog().any(tokens, catalog().timer_nouns()) {
        Some(Action::TimerCancel)
    } else if has_list_noun(tokens) {
        Some(Action::ListRemove)
    } else if has_cover_noun(tokens) || is_garage_cover(tokens) {
        Some(Action::CoverStop)
    } else if catalog().any(tokens, catalog().vacuum_nouns())
        || tokens
            .iter()
            .any(|token| matches!(catalog().verb(token), Some(crate::lang::VerbKind::Vacuum | crate::lang::VerbKind::VacuumNoun)))
    {
        // Pause/stop a vacuum is not return-to-base (P-30); refuse later as Unsupported.
        None
    } else if has_media_noun(tokens) {
        Some(Action::MediaPause)
    } else {
        Some(Action::Off)
    }
}

pub(super) fn playback_action(tokens: &[String]) -> Option<Action> {
    if catalog().any(tokens, catalog().playback_resume()) {
        Some(Action::MediaPlay)
    } else {
        Some(Action::MediaPause)
    }
}

pub(super) fn vacuum_noun_action(tokens: &[String]) -> Option<Action> {
    if is_query_token(tokens) {
        Some(Action::GetState)
    } else if crate::parse::infer::dock_hint(tokens) {
        Some(Action::VacuumDock)
    } else if catalog().any(tokens, catalog().vacuum_start()) || catalog().any(tokens, catalog().start_words()) {
        Some(Action::VacuumStart)
    } else {
        None
    }
}

pub(super) fn close_action(tokens: &[String]) -> Option<Action> {
    if has_cover_noun(tokens) || is_garage_cover(tokens) {
        Some(Action::CoverClose)
    } else if has_lock_or_door(tokens) {
        Some(Action::Lock)
    } else {
        Some(Action::Off)
    }
}

pub(super) fn lock_noun_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    let trailing_on = tokens.last().is_some_and(|x| cat.on_words().contains(x.as_str()));
    let unlock = tokens.iter().any(|x| cat.unlock_follow().contains(x.as_str()) || x == "unlock")
        || (tokens.iter().any(|x| matches!(x.as_str(), "flick" | "flip")) && cat.any(tokens, cat.door_nouns()) && !trailing_on);
    Some(if unlock { Action::Unlock } else { Action::Lock })
}

pub(super) fn close_lock_action(tokens: &[String]) -> Option<Action> {
    if has_lock_or_door(tokens) || tokens.iter().any(|x| x == "ab") {
        Some(Action::Lock)
    } else if has_cover_noun(tokens) {
        Some(Action::CoverClose)
    } else {
        Some(Action::Off)
    }
}

pub(super) fn flick_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, cat.off_words()) {
        Some(Action::Off)
    } else if cat.any(tokens, cat.on_words()) {
        Some(Action::On)
    } else if cat.any(tokens, cat.open_words()) {
        Some(if has_cover_noun(tokens) || is_garage_cover(tokens) { Action::CoverOpen } else { Action::On })
    } else if cat.any(tokens, cat.close_words()) {
        Some(if has_cover_noun(tokens) || is_garage_cover(tokens) { Action::CoverClose } else { Action::Off })
    } else if crate::parse::numbers::first_number(tokens).is_some() && (has_cover_noun(tokens) || is_garage_cover(tokens)) {
        Some(Action::CoverSet)
    } else {
        None
    }
}

pub(super) fn on_action(tokens: &[String], i: usize) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, cat.chores()) || cat.any(tokens, cat.list_nouns()) {
        return None;
    }
    let after_conj = tokens[i + 1..].iter().any(|x| cat.is_conj(x));
    if after_conj || cat.any(tokens, cat.scene_nouns()) || cat.any(tokens, cat.script_words()) {
        return Some(Action::On);
    }
    if is_query_token(tokens)
        || (has_media_noun(tokens)
            && tokens.iter().any(|x| matches!(x.as_str(), "stop" | "stopp" | "stoppen" | "pause" | "playback" | "stopped")))
    {
        return None;
    }
    Some(Action::On)
}
