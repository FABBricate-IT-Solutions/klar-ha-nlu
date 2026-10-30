use crate::lang::{catalog, VerbKind};
use crate::parse::action::{detect_actions, Action};
use crate::parse::normalize::fold_umlaut;
use crate::types::HomeGraph;

/// Split a token stream into clauses. "Wohnzimmer und Küche" stays one clause
/// unless a new action verb appears after the conjunction.
pub fn split_clauses(tokens: &[String], home: &HomeGraph) -> Vec<Vec<String>> {
    if let Some(at) = protected_media_followup(tokens, home) {
        let mut clauses = vec![tokens[..at].to_vec()];
        clauses.extend(split_clauses(&tokens[at + 1..], home));
        return clauses;
    }
    if protected_media_status(tokens) || phrase_spans_conj(tokens, home) {
        return vec![tokens.to_vec()];
    }
    if let Some(at) = split_two_targets(tokens) {
        let mut left = tokens[..at].to_vec();
        let mut right = tokens[at + 1..].to_vec();
        if detect_actions(&right).is_empty() {
            for (i, _) in detect_actions(tokens) {
                if i < at {
                    right.insert(0, tokens[i].clone());
                }
            }
        }
        // "Kugel und Decke aus" / "raito ribingu to kitchin keshite":
        // trailing particle or sole off/on verb belongs to both sides.
        if let Some(last) = tokens.last() {
            if is_shared_tail(last) && !left.iter().any(|token| is_shared_tail(token)) {
                left.push(last.clone());
            }
        }
        if !left.is_empty() && !right.is_empty() {
            // A side can hold several commands joined by a comma, which is gone by now:
            // "turn off the TV, turn off the kitchen lights and turn on the hall light".
            let mut clauses = split_clauses(&left, home);
            clauses.extend(split_clauses(&right, home));
            return clauses;
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
        if let Some(start) = verb_final_boundary(tokens, i0, i1) {
            cuts.push(start);
            continue;
        }
        let between = &tokens[i0 + 1..i1];
        let has_conj = between.iter().any(|t| is_conj(t));
        // Same verb twice ("mach … an") is one clause. A new verb, or the same verb
        // restated for another device ("turn off the TV and turn off the kitchen
        // lights", or with a comma instead of "and"), starts another.
        if a0 == a1 {
            if is_conj(&tokens[i1 - 1]) || restated_verb_first(tokens, i0, i1) {
                cuts.push(i1);
            }
            continue;
        }
        if is_same_command(a0, a1) && !has_conj {
            continue;
        }
        if a0 == a1 && is_particle(&tokens[i1]) || trailing_power_particle(tokens, i1, a0, a1, has_conj) {
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

    cuts.extend(opener_cuts(tokens));
    cuts.sort_unstable();
    cuts.dedup();
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

fn protected_media_followup(tokens: &[String], home: &HomeGraph) -> Option<usize> {
    tokens.iter().enumerate().find_map(|(index, token)| {
        if !is_conj(token) || conj_covered_by_name(tokens, home, index) || !protected_media_status(&tokens[..index]) {
            return None;
        }
        let right = &tokens[index + 1..];
        let actions = detect_actions(right);
        let new_command = actions.iter().any(|(_, action)| *action != Action::GetState) || explicit_query_start(right);
        new_command.then_some(index)
    })
}

fn explicit_query_start(tokens: &[String]) -> bool {
    tokens.first().is_some_and(|token| catalog().is_question_start(token) || catalog().is_question_word(token))
}

fn protected_media_status(tokens: &[String]) -> bool {
    let question = tokens.iter().any(|token| matches!(token.as_str(), "was" | "wie" | "what" | "whats"));
    let next = tokens.iter().any(|token| matches!(token.as_str(), "next" | "naechster" | "naechste" | "naechstes"));
    let media = tokens
        .iter()
        .any(|token| matches!(token.as_str(), "queue" | "warteschlange" | "music" | "musik" | "song" | "track" | "lied" | "titel"));
    has_phrase(tokens, &["kommt", "als", "naechstes"]) || has_phrase(tokens, &["wie", "laut"]) || (question && next && media)
}

fn has_phrase(tokens: &[String], phrase: &[&str]) -> bool {
    tokens.windows(phrase.len()).any(|window| window.iter().map(String::as_str).eq(phrase.iter().copied()))
}

fn phrase_spans_conj(tokens: &[String], home: &HomeGraph) -> bool {
    let Some(at) = tokens.iter().position(|t| is_conj(t)) else {
        return false;
    };
    conj_covered_by_name(tokens, home, at)
}

fn conj_covered_by_name(tokens: &[String], home: &HomeGraph, at: usize) -> bool {
    home.entities.iter().any(|ent| {
        name_covers_conj(&ent.name, tokens, at)
            || ent.aliases.iter().any(|alias| name_covers_conj(alias, tokens, at))
            || name_covers_conj(ent.entity_id.rsplit('.').next().unwrap_or(&ent.entity_id), tokens, at)
    }) || home.areas.iter().any(|area| {
        name_covers_conj(&area.name, tokens, at)
            || name_covers_conj(&area.area_id, tokens, at)
            || area.aliases.iter().any(|alias| name_covers_conj(alias, tokens, at))
    })
}

fn name_covers_conj(name: &str, tokens: &[String], at: usize) -> bool {
    let parts: Vec<String> =
        fold_umlaut(name).split(|c: char| !c.is_alphanumeric()).filter(|part| !part.is_empty()).map(str::to_string).collect();
    if parts.len() < 3 || !parts.iter().any(|p| is_conj(p)) {
        return false;
    }
    tokens
        .windows(parts.len())
        .enumerate()
        .any(|(start, window)| (start..start + parts.len()).contains(&at) && window.iter().zip(&parts).all(|(token, part)| token == part))
}

fn split_two_targets(tokens: &[String]) -> Option<usize> {
    let at = tokens.iter().position(|t| is_conj(t))?;
    let left = &tokens[..at];
    let right = &tokens[at + 1..];
    let right_head = right.split(|t| is_conj(t)).next().unwrap_or(right);
    if device_side(left)
        && device_side(right_head)
        && !right.iter().skip(right_head.len()).any(|t| is_conj(t))
        && !both_covers(left, right_head)
    {
        Some(at)
    } else {
        None
    }
}

fn both_covers(left: &[String], right: &[String]) -> bool {
    let cat = catalog();
    let cover = |tokens: &[String]| cat.any(tokens, cat.cover_nouns()) || cat.any(tokens, cat.curtain_nouns());
    cover(left) && cover(right)
}

fn device_side(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, cat.device_side())
        || cat.any(tokens, cat.named_device())
        || cat.any(tokens, cat.ceiling())
        || cat.any(tokens, cat.cover_nouns())
        || cat.any(tokens, cat.fan_nouns())
        || cat.any(tokens, cat.island())
        || crate::parse::infer::mentions_lamp_fixture(tokens)
        || cat.any(tokens, cat.laundry_machines())
        || tokens.iter().any(|token| matches!(token.as_str(), "dryer" | "washer"))
}

fn is_shared_tail(token: &str) -> bool {
    is_particle(token) || matches!(catalog().verb(token), Some(VerbKind::Off | VerbKind::On | VerbKind::OnParticle))
}

/// Verb-final clauses ("Licht an, mach Rollo zu und mach Vorhang zu",
/// "Licht aus und Rollo und Vorhang auf"): a particle closes its clause, a new device
/// follows, and another particle closes that clause. The next clause starts right after
/// the first particle (and after "und" if one follows). Commas are gone by now, so this
/// shape is the only sign of the boundary. Cutting at the last "und" instead would split
/// "Rollo und Vorhang" apart.
fn verb_final_boundary(tokens: &[String], i0: usize, i1: usize) -> Option<usize> {
    if !is_particle(&tokens[i0]) || !is_particle(&tokens[i1]) {
        return None;
    }
    // "Heizung auf 20 und Licht an": a number after the particle makes it a set-point,
    // not the end of a clause.
    if tokens.get(i0 + 1).is_some_and(|t| crate::parse::numbers::first_number(std::slice::from_ref(t)).is_some()) {
        return None;
    }
    if !tokens.get(i1 + 1).is_none_or(|t| is_conj(t)) {
        return None;
    }
    let start = if tokens.get(i0 + 1).is_some_and(|t| is_conj(t)) { i0 + 2 } else { i0 + 1 };
    (start < i1 && names_device(&tokens[start..i1])).then_some(start)
}

/// "off tv livingroom off kitchen lights": the verb comes again with a device on each
/// side of it, so it opens a new command (the comma between them is gone by now).
fn restated_verb_first(tokens: &[String], i0: usize, i1: usize) -> bool {
    // "off … off" restates the verb. In "schalte … aus" the particle closes the clause
    // its verb opened instead.
    if !is_particle(&tokens[i0]) || !is_particle(&tokens[i1]) {
        return false;
    }
    // "… aus schalte …": a verb right after means "aus" ended its clause.
    let after = &tokens[i1 + 1..];
    if after.first().is_some_and(|t| catalog().verb(t).is_some()) {
        return false;
    }
    let next = after.split(|t| is_conj(t)).next().unwrap_or(after);
    names_device(&tokens[i0 + 1..i1]) && names_device(next)
}

/// "schalte Fernseher Wohnzimmer aus schalte Licht Küche aus": an opening verb that comes
/// again after a device starts a new command, wherever the particles sit.
fn opener_cuts(tokens: &[String]) -> Vec<usize> {
    let mut cuts = Vec::new();
    let mut start = 0;
    for p in 1..tokens.len() {
        if !opens_clause(&tokens[p]) || is_conj(&tokens[p - 1]) {
            continue;
        }
        let rest = &tokens[p + 1..];
        let end = rest.iter().position(|t| is_conj(t) || opens_clause(t)).unwrap_or(rest.len());
        if names_device(&tokens[start..p]) && names_device(&rest[..end]) {
            cuts.push(p);
            start = p;
        }
    }
    cuts
}

/// Only switch verbs ("schalte"): colours and other set verbs ("red") also sit in the
/// verb list but never open a clause of their own.
fn opens_clause(token: &str) -> bool {
    matches!(catalog().verb(token), Some(VerbKind::Switch))
}

fn names_device(tokens: &[String]) -> bool {
    device_side(tokens) || catalog().any(tokens, catalog().curtain_nouns())
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

fn trailing_power_particle(tokens: &[String], i1: usize, a0: Action, a1: Action, has_conj: bool) -> bool {
    if i1 + 1 != tokens.len() || !is_particle(&tokens[i1]) {
        return false;
    }
    if matches!((a0, a1), (Action::On, Action::Off) | (Action::Off, Action::On)) {
        return !has_conj;
    }
    matches!(a0, Action::GetState)
        && matches!(a1, Action::On | Action::Off)
        && catalog().any(tokens, catalog().climate_nouns())
        && !tokens.iter().any(|token| catalog().is_question_word(token) || catalog().is_question_start(token))
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
                Action::On | Action::Off | Action::Lock | Action::Unlock | Action::CoverOpen | Action::CoverClose | Action::CoverSet
            )
            | (
                Action::On | Action::Off | Action::Lock | Action::Unlock | Action::CoverOpen | Action::CoverClose | Action::CoverSet,
                Action::GetState
            )
            | (Action::CoverOpen, Action::CoverClose)
            | (Action::CoverClose, Action::CoverOpen)
            | (
                Action::On | Action::Off | Action::SetLight | Action::MediaPause | Action::MediaPlay,
                Action::TimerStart | Action::TimerAdd | Action::TimerRemove | Action::TimerCancel | Action::TimerPause
            )
            | (
                Action::TimerStart | Action::TimerAdd | Action::TimerRemove | Action::TimerCancel | Action::TimerPause,
                Action::On | Action::Off | Action::SetLight | Action::MediaPause | Action::MediaPlay
            )
            | (Action::GetState, Action::TimerStart | Action::TimerAdd | Action::TimerRemove | Action::TimerCancel | Action::TimerPause)
            | (Action::TimerStart | Action::TimerAdd | Action::TimerRemove | Action::TimerCancel | Action::TimerPause, Action::GetState)
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

pub(crate) fn follow_fixture(tokens: &[String], home: &crate::types::HomeGraph, areas: &[String]) -> Option<String> {
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
    if cat.any(tokens, cat.island()) {
        return find("island").or_else(|| find("insel"));
    }
    if cat.any(tokens, cat.ceiling()) {
        return find("ceiling").or_else(|| find("decke"));
    }
    if crate::parse::infer::mentions_lamp_fixture(tokens) {
        return find("bedside").or_else(|| find("lamp")).or_else(|| find("ceiling"));
    }
    None
}

#[cfg(test)]
#[path = "split_tests.rs"]
mod tests;
