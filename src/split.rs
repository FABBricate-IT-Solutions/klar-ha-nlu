use crate::lang::catalog;
use crate::lexicon::{detect_actions, Action};
use crate::normalize::fold_umlaut;
use crate::types::HomeGraph;

/// Split a token stream into clauses. "Wohnzimmer und Küche" stays one clause
/// unless a new action verb appears after the conjunction.
pub fn split_clauses(tokens: &[String], home: &HomeGraph) -> Vec<Vec<String>> {
    if phrase_spans_conj(tokens, home) {
        return vec![tokens.to_vec()];
    }
    if let Some(at) = split_two_targets(tokens) {
        let left = tokens[..at].to_vec();
        let mut right = tokens[at + 1..].to_vec();
        if detect_actions(&right).is_empty() {
            for (i, _) in detect_actions(tokens) {
                if i < at {
                    right.insert(0, tokens[i].clone());
                }
            }
        }
        if !left.is_empty() && !right.is_empty() {
            return vec![left, right];
        }
    }
    let actions = detect_actions(tokens);
    if actions.len() <= 1 {
        return vec![tokens.to_vec()];
    }

    let mut cuts = Vec::new();
    for window in actions.windows(2) {
        let (i0, a0) = window[0];
        let (i1, a1) = window[1];
        if i1 <= i0 {
            continue;
        }
        let between = &tokens[i0 + 1..i1];
        let has_conj = between.iter().any(|t| is_conj(t));
        // Same verb twice ("mach … an") is one clause. A new verb, or
        // the same verb after "und", starts another.
        if a0 == a1 {
            continue;
        }
        if is_same_command(a0, a1) && !has_conj {
            continue;
        }
        if a0 == a1 && is_particle(&tokens[i1]) {
            continue;
        }
        if has_conj || (is_new_action_span(a1) && a1 != a0) {
            if let Some(cut) = between.iter().rposition(|t| is_conj(t)) {
                cuts.push(i0 + 1 + cut + 1);
            } else {
                cuts.push(i1);
            }
        }
    }

    if cuts.is_empty() {
        return vec![tokens.to_vec()];
    }

    let mut clauses = Vec::new();
    let mut start = 0;
    for cut in cuts {
        if cut > start {
            clauses.push(tokens[start..cut].to_vec());
        }
        start = cut;
    }
    if start < tokens.len() {
        clauses.push(tokens[start..].to_vec());
    }
    clauses.retain(|c| !c.is_empty());
    if clauses.is_empty() {
        vec![tokens.to_vec()]
    } else {
        clauses
    }
}

fn phrase_spans_conj(tokens: &[String], home: &HomeGraph) -> bool {
    let Some(at) = tokens.iter().position(|t| is_conj(t)) else {
        return false;
    };
    home.entities.iter().any(|ent| {
        name_covers_conj(&ent.name, tokens, at)
            || ent.aliases.iter().any(|alias| name_covers_conj(alias, tokens, at))
            || name_covers_conj(ent.entity_id.rsplit('.').next().unwrap_or(&ent.entity_id), tokens, at)
    })
}

fn name_covers_conj(name: &str, tokens: &[String], at: usize) -> bool {
    let parts: Vec<String> = fold_umlaut(name)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();
    if parts.len() < 3 || !parts.iter().any(|p| is_conj(p)) {
        return false;
    }
    tokens.windows(parts.len()).enumerate().any(|(start, window)| {
        (start..start + parts.len()).contains(&at)
            && window.iter().zip(&parts).all(|(token, part)| token == part)
    })
}

fn split_two_targets(tokens: &[String]) -> Option<usize> {
    let at = tokens.iter().position(|t| is_conj(t))?;
    let left = &tokens[..at];
    let right = &tokens[at + 1..];
    let right_head = right.split(|t| is_conj(t)).next().unwrap_or(right);
    if device_side(left) && device_side(right_head) && !right.iter().skip(right_head.len()).any(|t| is_conj(t))
    {
        Some(at)
    } else {
        None
    }
}

fn device_side(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().device_side)
}

fn is_conj(t: &str) -> bool {
    catalog().is_conj(t)
}

fn is_new_action_span(action: Action) -> bool {
    !matches!(action, Action::GetState)
}

fn is_particle(token: &str) -> bool {
    catalog().is_particle(token)
}

fn is_same_command(a: Action, b: Action) -> bool {
    matches!(
        (a, b),
        (Action::On | Action::Off, Action::FanSpeed)
            | (Action::FanSpeed, Action::On | Action::Off)
            | (Action::On | Action::Off, Action::SetLight)
            | (Action::SetLight, Action::On | Action::Off)
            | (Action::CoverOpen | Action::CoverClose, Action::CoverSet)
            | (Action::CoverSet, Action::CoverOpen | Action::CoverClose)
            | (Action::Lock | Action::Unlock, Action::On | Action::Off)
            | (Action::On | Action::Off, Action::Lock | Action::Unlock)
            | (Action::Lock, Action::Unlock)
            | (Action::Unlock, Action::Lock)
            | (Action::SetTemp, Action::SetLight)
            | (Action::SetLight, Action::SetTemp)
            | (Action::GetState, Action::FanSpeed | Action::SetLight | Action::SetTemp)
            | (Action::FanSpeed | Action::SetLight | Action::SetTemp, Action::GetState)
            | (
                Action::GetState,
                Action::On
                    | Action::Off
                    | Action::Lock
                    | Action::Unlock
                    | Action::CoverOpen
                    | Action::CoverClose
                    | Action::CoverSet
            )
            | (
                Action::On
                    | Action::Off
                    | Action::Lock
                    | Action::Unlock
                    | Action::CoverOpen
                    | Action::CoverClose
                    | Action::CoverSet,
                Action::GetState
            )
            | (Action::CoverOpen, Action::CoverClose)
            | (Action::CoverClose, Action::CoverOpen)
            | (Action::On | Action::Off | Action::SetLight | Action::MediaPause | Action::MediaPlay, Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause)
            | (Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause, Action::On | Action::Off | Action::SetLight | Action::MediaPause | Action::MediaPlay)
            | (Action::GetState, Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause)
            | (Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause, Action::GetState)
            | (Action::GetState, Action::VacuumDock | Action::VacuumStart)
            | (Action::VacuumDock | Action::VacuumStart, Action::GetState)
            | (Action::VacuumStart | Action::VacuumDock, Action::On | Action::Off)
            | (Action::On | Action::Off, Action::VacuumStart | Action::VacuumDock)
            | (Action::ListAdd | Action::ListComplete, Action::On | Action::Off | Action::SetLight)
            | (Action::On | Action::Off | Action::SetLight, Action::ListAdd | Action::ListComplete)
    )
}

pub(crate) fn wants_group_clarify(raw: &[String]) -> bool {
    catalog().wants_group_clarify(raw)
}

pub(crate) fn follow_fixture(
    tokens: &[String],
    home: &crate::types::HomeGraph,
    areas: &[String],
) -> Option<String> {
    if areas.is_empty() {
        return None;
    }
    let lights: Vec<&str> = home
        .entities
        .iter()
        .filter(|e| e.domain == "light" && e.area.as_ref().is_some_and(|a| areas.contains(a)))
        .map(|e| e.entity_id.as_str())
        .collect();
    let find = |n: &str| lights.iter().find(|id| id.contains(n)).map(|s| (*s).to_string());
    let cat = catalog();
    if cat.any(tokens, &cat.island) {
        return find("island").or_else(|| find("insel"));
    }
    if cat.any(tokens, &cat.ceiling) {
        return find("ceiling").or_else(|| find("decke"));
    }
    if cat.any(tokens, &cat.lamp_fixture) {
        return find("bedside").or_else(|| find("lamp")).or_else(|| find("ceiling"));
    }
    None
}

pub(crate) fn implied_domain(action: Action) -> Option<&'static str> {
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
