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

pub fn domain_for(action: Action, tokens: &[String]) -> Option<&'static str> {
    forced_domain(action).or_else(|| crate::parse::resolve::domain_hint(tokens)).or_else(|| implied_domain(action))
}

fn forced_domain(action: Action) -> Option<&'static str> {
    match action {
        Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause => Some("timer"),
        Action::SetTemp => Some("climate"),
        Action::CoverOpen | Action::CoverClose | Action::CoverSet => Some("cover"),
        Action::FanSpeed => Some("fan"),
        Action::Lock | Action::Unlock => Some("lock"),
        _ => None,
    }
}

fn implied_domain(action: Action) -> Option<&'static str> {
    match action {
        Action::SetLight => Some("light"),
        Action::SetTemp => Some("climate"),
        Action::CoverOpen | Action::CoverClose | Action::CoverSet => Some("cover"),
        Action::Lock | Action::Unlock => Some("lock"),
        Action::FanSpeed => Some("fan"),
        Action::VacuumStart | Action::VacuumDock => Some("vacuum"),
        Action::MediaPause | Action::MediaPlay | Action::MediaNext | Action::MediaMute => Some("media_player"),
        Action::Scene => Some("scene"),
        Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause => Some("timer"),
        Action::ListAdd | Action::ListComplete => Some("todo"),
        _ => None,
    }
}

pub fn detect_actions(tokens: &[String]) -> Vec<(usize, Action)> {
    let cat = catalog();
    let fuzzy = if tokens.iter().any(|token| cat.verb(token).is_some()) || !has_structural_anchor(tokens) {
        None
    } else {
        let hits: Vec<(usize, VerbKind)> =
            tokens.iter().enumerate().filter_map(|(index, token)| cat.fuzzy_verb(token).map(|kind| (index, kind))).collect();
        (hits.len() == 1).then(|| hits[0])
    };
    let mut found = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        let verb = cat.verb(t).or_else(|| fuzzy.and_then(|(index, kind)| (index == i).then_some(kind)));
        let action = match verb {
            Some(VerbKind::On) => Some(Action::On),
            Some(VerbKind::OnParticle) => on_action(tokens, i),
            Some(VerbKind::Open) => open_action(tokens),
            Some(VerbKind::OpenDoor) => open_door_action(tokens),
            Some(VerbKind::Off) => off_action(tokens),
            Some(VerbKind::Lower) => {
                if crate::parse::numbers::first_number(tokens).is_some() {
                    Some(set_by_number(tokens))
                } else {
                    Some(Action::CoverClose)
                }
            }
            Some(VerbKind::Roll) => {
                if catalog().any(tokens, &catalog().roll_close) {
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
            Some(VerbKind::Brightness) => {
                Some(if crate::parse::numbers::first_number(tokens).is_some() { Action::SetLight } else { Action::GetState })
            }
            Some(VerbKind::Speed) => {
                Some(if crate::parse::numbers::first_number(tokens).is_some() { Action::FanSpeed } else { Action::GetState })
            }
            Some(VerbKind::Climate) | Some(VerbKind::Temperature) => {
                Some(if crate::parse::numbers::first_number(tokens).is_some() { Action::SetTemp } else { Action::GetState })
            }
            Some(VerbKind::Query) => Some(Action::GetState),
            Some(VerbKind::Switch) => Some(if cat.any(tokens, &cat.off_words) { Action::Off } else { Action::On }),
            Some(VerbKind::Pause) => Some(if cat.any(tokens, &cat.timer_nouns) { Action::TimerPause } else { Action::MediaPause }),
            Some(VerbKind::Playback) => playback_action(tokens),
            Some(VerbKind::Play) => Some(if cat.any(tokens, &cat.timer_nouns) { Action::TimerStart } else { Action::MediaPlay }),
            Some(VerbKind::Next) => Some(Action::MediaNext),
            Some(VerbKind::Mute) => Some(Action::MediaMute),
            Some(VerbKind::FanNoun) => {
                if has_power_word(tokens) && crate::parse::numbers::first_number(tokens).is_none() {
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
            Some(VerbKind::Position) => {
                Some(if crate::parse::numbers::first_number(tokens).is_some() { Action::CoverSet } else { Action::GetState })
            }
            Some(VerbKind::Flick) => flick_action(tokens),
            Some(VerbKind::ClarifyWrong) => Some(Action::ClarifyWrong),
            Some(VerbKind::Timer) => Some(timer_kind(tokens)),
            Some(VerbKind::List) => {
                Some(if catalog().any(tokens, &catalog().list_complete) { Action::ListComplete } else { Action::ListAdd })
            }
            Some(VerbKind::Add) => add_action(tokens),
            Some(VerbKind::ListComplete) => Some(Action::ListComplete),
            Some(VerbKind::Make) => {
                Some(if crate::parse::numbers::first_number(tokens).is_some() { set_by_number(tokens) } else { Action::On })
            }
            Some(VerbKind::Color) => Some(Action::SetLight),
            Some(VerbKind::Percent) => {
                Some(if crate::parse::numbers::first_number(tokens).is_some() { set_by_number(tokens) } else { Action::GetState })
            }
            None => None,
        };
        if let Some(a) = action {
            found.push((i, a));
        }
    }
    found
}

fn has_structural_anchor(tokens: &[String]) -> bool {
    let cat = catalog();
    crate::parse::numbers::first_number(tokens).is_some()
        || cat.any(tokens, &cat.light_nouns)
        || cat.any(tokens, &cat.cover_nouns)
        || cat.any(tokens, &cat.fan_nouns)
        || cat.any(tokens, &cat.climate_nouns)
        || cat.any(tokens, &cat.media_nouns)
        || cat.any(tokens, &cat.lock_nouns)
        || cat.any(tokens, &cat.timer_nouns)
        || cat.any(tokens, &cat.list_nouns)
        || cat.any(tokens, &cat.vacuum_nouns)
        || cat.any(tokens, &cat.scene_nouns)
        || cat.any(tokens, &cat.script_words)
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
    let cat = catalog();
    if cat.any(tokens, &cat.door_nouns) || cat.any(tokens, &cat.lock_nouns) {
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
    if cat.any(tokens, &cat.close_words) {
        None
    } else if crate::parse::numbers::first_number(tokens).is_some() {
        Some(set_by_number(tokens))
    } else if has_cover_noun(tokens) || tokens.iter().any(|x| x == "roll") {
        Some(Action::CoverOpen)
    } else {
        Some(Action::On)
    }
}

fn down_action(tokens: &[String]) -> Option<Action> {
    let cat = catalog();
    if cat.any(tokens, &cat.list_down) || cat.any(tokens, &cat.timer_add) {
        None
    } else if crate::parse::numbers::first_number(tokens).is_some() {
        Some(set_by_number(tokens))
    } else if has_cover_noun(tokens) {
        Some(Action::CoverClose)
    } else {
        Some(Action::Off)
    }
}

fn auf_action(tokens: &[String]) -> Option<Action> {
    if crate::parse::numbers::first_number(tokens).is_none() {
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
    let cat = catalog();
    if cat.any(tokens, &cat.timer_cancel) {
        Action::TimerCancel
    } else if cat.any(tokens, &cat.timer_pause) {
        Action::TimerPause
    } else if cat.any(tokens, &cat.timer_add) {
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
    if catalog().any(tokens, &catalog().playback_resume) {
        Some(Action::MediaPlay)
    } else {
        Some(Action::MediaPause)
    }
}

fn vacuum_noun_action(tokens: &[String]) -> Option<Action> {
    if is_query_token(tokens) {
        Some(Action::GetState)
    } else if catalog().any(tokens, &catalog().vacuum_start) || catalog().any(tokens, &catalog().start_words) {
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
    let cat = catalog();
    let trailing_on = tokens.last().is_some_and(|x| cat.on_words.contains(x.as_str()));
    let unlock = tokens.iter().any(|x| cat.unlock_follow.contains(x.as_str()) || x == "unlock")
        || (tokens.iter().any(|x| matches!(x.as_str(), "flick" | "flip")) && cat.any(tokens, &cat.door_nouns) && !trailing_on);
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
    } else if cat.any(tokens, &cat.open_words) {
        Some(if has_cover_noun(tokens) { Action::CoverOpen } else { Action::On })
    } else if cat.any(tokens, &cat.close_words) {
        Some(if has_cover_noun(tokens) { Action::CoverClose } else { Action::Off })
    } else if crate::parse::numbers::first_number(tokens).is_some() && (has_cover_noun(tokens) || is_garage_cover(tokens)) {
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
        || (has_media_noun(tokens) && tokens.iter().any(|x| matches!(x.as_str(), "stop" | "pause" | "playback" | "stopped")))
    {
        return None;
    }
    Some(Action::On)
}
