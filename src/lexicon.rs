pub use crate::types::default_home;

use crate::lang::{catalog, VerbKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    On,
    Off,
    Toggle,
    SetLight,
    SetTemp,
    GetState,
    MediaPause,
    MediaPlay,
    MediaNext,
    MediaMute,
    FanSpeed,
    VacuumStart,
    VacuumDock,
    Scene,
    CoverOpen,
    CoverClose,
    CoverSet,
    Lock,
    Unlock,
    TimerStart,
    TimerAdd,
    TimerCancel,
    TimerPause,
    ListAdd,
    ListComplete,
    ClarifyWrong,
}

pub fn detect_actions(tokens: &[String]) -> Vec<(usize, Action)> {
    let cat = catalog();
    let mut found = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        let action = match cat.verb(t) {
            Some(VerbKind::On) => Some(Action::On),
            Some(VerbKind::OnParticle) => on_action(tokens, i),
            Some(VerbKind::Open) => open_action(tokens),
            Some(VerbKind::OpenDoor) => open_door_action(tokens),
            Some(VerbKind::Off) => off_action(tokens),
            Some(VerbKind::Lower) => {
                if crate::numbers::first_number(tokens).is_some() {
                    Some(set_by_number(tokens))
                } else {
                    Some(Action::CoverClose)
                }
            }
            Some(VerbKind::Roll) => {
                if tokens
                    .iter()
                    .any(|x| matches!(x.as_str(), "down" | "zu" | "close"))
                {
                    Some(Action::CoverClose)
                } else {
                    Some(Action::CoverOpen)
                }
            }
            Some(VerbKind::Up) => up_action(tokens),
            Some(VerbKind::Down) => down_action(tokens),
            Some(VerbKind::Auf) => auf_action(tokens),
            Some(VerbKind::Stop) => stop_action(tokens),
            Some(VerbKind::Toggle) => Some(Action::Toggle),
            Some(VerbKind::Dim) => Some(Action::SetLight),
            Some(VerbKind::Brightness) => Some(if crate::numbers::first_number(tokens).is_some() {
                Action::SetLight
            } else {
                Action::GetState
            }),
            Some(VerbKind::Speed) => Some(if crate::numbers::first_number(tokens).is_some() {
                Action::FanSpeed
            } else {
                Action::GetState
            }),
            Some(VerbKind::Climate) | Some(VerbKind::Temperature) => {
                Some(if crate::numbers::first_number(tokens).is_some() {
                    Action::SetTemp
                } else {
                    Action::GetState
                })
            }
            Some(VerbKind::Query) => Some(Action::GetState),
            Some(VerbKind::Switch) => Some(if cat.any(tokens, &cat.off_words) {
                Action::Off
            } else {
                Action::On
            }),
            Some(VerbKind::Pause) => Some(if cat.any(tokens, &cat.timer_nouns) {
                Action::TimerPause
            } else {
                Action::MediaPause
            }),
            Some(VerbKind::Playback) => playback_action(tokens),
            Some(VerbKind::Play) => Some(if cat.any(tokens, &cat.timer_nouns) {
                Action::TimerStart
            } else {
                Action::MediaPlay
            }),
            Some(VerbKind::Next) => Some(Action::MediaNext),
            Some(VerbKind::Mute) => Some(Action::MediaMute),
            Some(VerbKind::FanNoun) => {
                if has_power_word(tokens) && crate::numbers::first_number(tokens).is_none() {
                    None
                } else {
                    Some(Action::FanSpeed)
                }
            }
            Some(VerbKind::Vacuum) => Some(Action::VacuumStart),
            Some(VerbKind::Dock) => Some(Action::VacuumDock),
            Some(VerbKind::Scene) => Some(Action::Scene),
            Some(VerbKind::VacuumNoun) => vacuum_noun_action(tokens),
            Some(VerbKind::Close) => close_action(tokens),
            Some(VerbKind::Lock) => Some(Action::Lock),
            Some(VerbKind::LockNoun) => lock_noun_action(tokens),
            Some(VerbKind::CloseLock) => close_lock_action(tokens),
            Some(VerbKind::Unlock) => Some(Action::Unlock),
            Some(VerbKind::Set) => Some(set_by_number(tokens)),
            Some(VerbKind::Position) => Some(if crate::numbers::first_number(tokens).is_some() {
                Action::CoverSet
            } else {
                Action::GetState
            }),
            Some(VerbKind::Flick) => flick_action(tokens),
            Some(VerbKind::ClarifyWrong) => Some(Action::ClarifyWrong),
            Some(VerbKind::Timer) => Some(timer_kind(tokens)),
            Some(VerbKind::List) => Some(if tokens.iter().any(|x| {
                matches!(x.as_str(), "haken" | "erledigt" | "complete" | "check")
            }) {
                Action::ListComplete
            } else {
                Action::ListAdd
            }),
            Some(VerbKind::Add) => add_action(tokens),
            Some(VerbKind::ListComplete) => Some(Action::ListComplete),
            Some(VerbKind::Make) => Some(if crate::numbers::first_number(tokens).is_some() {
                set_by_number(tokens)
            } else {
                Action::On
            }),
            Some(VerbKind::Color) => Some(Action::SetLight),
            Some(VerbKind::Percent) => Some(if crate::numbers::first_number(tokens).is_some() {
                set_by_number(tokens)
            } else {
                Action::GetState
            }),
            None => None,
        };
        if let Some(a) = action {
            found.push((i, a));
        }
    }
    found
}

pub(crate) fn is_query_token(tokens: &[String]) -> bool {
    tokens.iter().any(|x| catalog().is_query_hint(x))
}

fn set_by_number(tokens: &[String]) -> Action {
    if has_climate_noun(tokens) {
        Action::SetTemp
    } else if has_fan_noun(tokens) {
        Action::FanSpeed
    } else if has_light_noun(tokens) {
        Action::SetLight
    } else if has_cover_noun(tokens) || is_garage_cover(tokens) {
        Action::CoverSet
    } else {
        Action::SetLight
    }
}

pub(crate) fn is_garage_cover(tokens: &[String]) -> bool {
    let cat = catalog();
    let garage = cat.any(tokens, &cat.garage_words);
    let cover_door = cat.any(tokens, &cat.garage_cover);
    let lock_door = cat.any(tokens, &cat.garage_lock_block);
    garage && cover_door && !lock_door
}

fn has_media_noun(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().media_nouns)
}

fn has_cover_noun(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().cover_nouns)
}

fn has_fan_noun(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().fan_nouns)
}

pub(crate) fn has_light_noun(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().light_nouns)
}

fn has_climate_noun(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().climate_nouns)
}

fn has_power_word(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().power_words)
}

fn has_lock_or_door(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, &cat.lock_nouns) || cat.any(tokens, &cat.door_nouns)
}

fn open_action(tokens: &[String]) -> Option<Action> {
    if has_cover_noun(tokens) {
        Some(Action::CoverOpen)
    } else if catalog().any(tokens, &catalog().lock_nouns) {
        Some(Action::Unlock)
    } else {
        Some(Action::On)
    }
}

fn open_door_action(tokens: &[String]) -> Option<Action> {
    if tokens.iter().any(|x| {
        matches!(
            x.as_str(),
            "tuer" | "haustuer" | "garagentuer" | "door" | "lock" | "schloss"
        )
    }) {
        Some(Action::Unlock)
    } else if has_cover_noun(tokens) {
        Some(Action::CoverOpen)
    } else {
        Some(Action::On)
    }
}

fn off_action(tokens: &[String]) -> Option<Action> {
    if catalog().any(tokens, &catalog().lock_nouns) {
        Some(Action::Unlock)
    } else {
        Some(Action::Off)
    }
}

fn up_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, &cat.close_words) || tokens.iter().any(|x| matches!(x.as_str(), "shut")) {
        None
    } else if crate::numbers::first_number(tokens).is_some() {
        Some(set_by_number(tokens))
    } else if has_cover_noun(tokens) || tokens.iter().any(|x| x == "roll") {
        Some(Action::CoverOpen)
    } else {
        Some(Action::On)
    }
}

fn down_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, &cat.list_down) || tokens.iter().any(|x| x == "wipe" || x == "add") {
        None
    } else if crate::numbers::first_number(tokens).is_some() {
        Some(set_by_number(tokens))
    } else if has_cover_noun(tokens) {
        Some(Action::CoverClose)
    } else {
        Some(Action::Off)
    }
}

fn auf_action(tokens: &[String]) -> Option<Action> {
    if crate::numbers::first_number(tokens).is_none() {
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

fn timer_kind(tokens: &[String]) -> Action {
    if tokens.iter().any(|x| {
        matches!(
            x.as_str(),
            "abbrechen"
                | "abbreche"
                | "abbruch"
                | "cancel"
                | "stop"
                | "stopp"
                | "stoppe"
                | "stoppen"
                | "loesche"
                | "loeschen"
                | "delete"
                | "clear"
                | "aus"
                | "off"
        )
    }) {
        Action::TimerCancel
    } else if tokens.iter().any(|x| {
        matches!(
            x.as_str(),
            "pause" | "pausieren" | "anhalten" | "halt" | "freeze"
        )
    }) {
        Action::TimerPause
    } else if tokens
        .iter()
        .any(|x| matches!(x.as_str(), "add" | "plus" | "mehr" | "increase" | "extend"))
    {
        Action::TimerAdd
    } else {
        Action::TimerStart
    }
}

fn stop_action(tokens: &[String]) -> Option<Action> {
    if catalog().any(tokens, &catalog().timer_nouns) {
        Some(Action::TimerCancel)
    } else if has_media_noun(tokens) {
        Some(Action::MediaPause)
    } else {
        Some(Action::Off)
    }
}

fn playback_action(tokens: &[String]) -> Option<Action> {
    if tokens
        .iter()
        .any(|x| matches!(x.as_str(), "resume" | "unpause" | "play" | "weiter" | "fortsetzen"))
    {
        Some(Action::MediaPlay)
    } else {
        Some(Action::MediaPause)
    }
}

fn vacuum_noun_action(tokens: &[String]) -> Option<Action> {
    if is_query_token(tokens) {
        Some(Action::GetState)
    } else if tokens
        .iter()
        .any(|x| {
            matches!(x.as_str(), "an" | "on" | "start" | "starten" | "starte")
                || catalog().start_words.contains(x.as_str())
        })
    {
        Some(Action::VacuumStart)
    } else {
        None
    }
}

fn close_action(tokens: &[String]) -> Option<Action> {
    if has_cover_noun(tokens) || is_garage_cover(tokens) {
        Some(Action::CoverClose)
    } else if has_lock_or_door(tokens) {
        Some(Action::Lock)
    } else {
        Some(Action::Off)
    }
}

fn lock_noun_action(tokens: &[String]) -> Option<Action> {
    let trailing_on = tokens
        .last()
        .is_some_and(|x| matches!(x.as_str(), "on" | "an"));
    let unlock = tokens.iter().any(|x| matches!(x.as_str(), "unlock" | "open"))
        || (tokens.iter().any(|x| matches!(x.as_str(), "flick" | "flip"))
            && tokens.iter().any(|x| x.as_str() == "door")
            && !trailing_on);
    Some(if unlock { Action::Unlock } else { Action::Lock })
}

fn close_lock_action(tokens: &[String]) -> Option<Action> {
    if has_lock_or_door(tokens) || tokens.iter().any(|x| x == "ab") {
        Some(Action::Lock)
    } else if has_cover_noun(tokens) {
        Some(Action::CoverClose)
    } else {
        Some(Action::Off)
    }
}

fn flick_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, &cat.off_words) {
        Some(Action::Off)
    } else if cat.any(tokens, &cat.on_words) {
        Some(Action::On)
    } else if tokens.iter().any(|x| matches!(x.as_str(), "open" | "auf")) {
        Some(if has_cover_noun(tokens) {
            Action::CoverOpen
        } else {
            Action::On
        })
    } else if tokens
        .iter()
        .any(|x| matches!(x.as_str(), "close" | "zu" | "closed"))
    {
        Some(if has_cover_noun(tokens) {
            Action::CoverClose
        } else {
            Action::Off
        })
    } else if crate::numbers::first_number(tokens).is_some() && has_cover_noun(tokens) {
        Some(Action::CoverSet)
    } else {
        None
    }
}

fn add_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, &cat.list_nouns) {
        Some(Action::ListAdd)
    } else if cat.any(tokens, &cat.timer_nouns) {
        Some(Action::TimerAdd)
    } else {
        None
    }
}

fn on_action(tokens: &[String], i: usize) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, &cat.chores) || cat.any(tokens, &cat.list_nouns) {
        return None;
    }
    let after_conj = tokens[i + 1..].iter().any(|x| cat.is_conj(x));
    if after_conj || cat.any(tokens, &cat.scene_nouns) || cat.any(tokens, &cat.script_words) {
        return Some(Action::On);
    }
    if is_query_token(tokens)
        || (has_media_noun(tokens)
            && tokens
                .iter()
                .any(|x| matches!(x.as_str(), "stop" | "pause" | "playback" | "stopped")))
    {
        return None;
    }
    Some(Action::On)
}
