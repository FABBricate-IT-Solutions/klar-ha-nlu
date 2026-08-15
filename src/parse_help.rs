use crate::lang::catalog;
use crate::lexicon::{has_light_noun, Action};
use crate::normalize::{fold_umlaut, join_tokens};
use crate::session::Session;
use crate::types::{known_intent, CustomSentence, HomeGraph, Intent};
use strsim::normalized_levenshtein;

fn last_domain(session: &Session, prefix: &str) -> bool {
    session.last_domains().any(|d| d == prefix) || session.last_entities().any(|e| e.starts_with(&format!("{prefix}.")))
}

/// Token-only tweaks, then bind the verb to a target/session domain.
pub(crate) fn infer_action(
    action: Action,
    tokens: &[String],
    number: Option<i32>,
    question: bool,
    session: &Session,
    target_domain: Option<&str>,
) -> Action {
    let action = refine_tokens(action, tokens, number, question);
    let session_d = session_domain(session, tokens);
    let domain = target_domain.or(session_d);
    bind_domain_with(action, tokens, number, domain, target_domain.is_none() && session_d.is_some())
}

fn refine_tokens(action: Action, tokens: &[String], number: Option<i32>, question: bool) -> Action {
    if question && matches!(action, Action::VacuumDock | Action::VacuumStart) {
        return Action::GetState;
    }
    if catalog().any(tokens, &catalog().vacuum_nouns)
        && (matches!(action, Action::On) || catalog().any(tokens, &catalog().start_words) || catalog().any(tokens, &catalog().vacuum_start))
    {
        return Action::VacuumStart;
    }
    if matches!(action, Action::On | Action::Off) && tokens.iter().any(|t| catalog().timer_nouns.contains(t.as_str())) {
        return if matches!(action, Action::Off) { Action::TimerCancel } else { Action::TimerStart };
    }
    if matches!(action, Action::FanSpeed | Action::TimerStart | Action::TimerAdd)
        && number.is_none()
        && (matches!(action, Action::FanSpeed) || question || tokens.iter().any(|t| catalog().timer_query.contains(t.as_str())))
    {
        return Action::GetState;
    }
    if matches!(action, Action::SetLight) && number.is_none() && color_word(tokens).is_none() && question {
        return Action::GetState;
    }
    action
}

fn session_domain(session: &Session, tokens: &[String]) -> Option<&'static str> {
    if has_light_noun(tokens) {
        return None;
    }
    ["climate", "cover", "fan", "lock", "media_player"].into_iter().find(|prefix| last_domain(session, prefix))
}

/// Bind a verb to the domain of the chosen target. Slot filling stays in `fill_intent`.
pub(crate) fn bind_domain(action: Action, tokens: &[String], number: Option<i32>, domain: Option<&str>) -> Action {
    bind_domain_with(action, tokens, number, domain, false)
}

fn bind_domain_with(action: Action, tokens: &[String], number: Option<i32>, domain: Option<&str>, session_follow: bool) -> Action {
    let cat = catalog();
    let light_noun = has_light_noun(tokens);
    if matches!(action, Action::SetLight | Action::On) && number.is_some() && !light_noun && domain == Some("climate") {
        return Action::SetTemp;
    }
    if matches!(action, Action::SetLight) && !light_noun {
        match domain {
            Some("switch") => return Action::On,
            Some("cover") => return Action::CoverSet,
            Some("fan") if !cat.any(tokens, &cat.kitchen) => return Action::FanSpeed,
            Some("media_player") => return Action::On,
            _ => {}
        }
    }
    if matches!(action, Action::On | Action::Off) && domain == Some("lock") && cat.any(tokens, &cat.unlock_follow) {
        return Action::Unlock;
    }
    let cover_follow = matches!(action, Action::On | Action::Off) || (session_follow && matches!(action, Action::GetState));
    if cover_follow && domain == Some("cover") && !light_noun && number.is_none() {
        if cat.any(tokens, &cat.cover_open_follow) {
            return Action::CoverOpen;
        }
        if cat.any(tokens, &cat.close_words) {
            return Action::CoverClose;
        }
    }
    if matches!(action, Action::On)
        && (color_word(tokens).is_some() || number.is_some())
        && (domain == Some("light") || light_noun || cat.any(tokens, &cat.ceiling))
    {
        return Action::SetLight;
    }
    action
}

pub(crate) fn prefer_action(actions: &[(usize, Action)]) -> Option<Action> {
    const RANK: &[Action] = &[
        Action::TimerAdd,
        Action::TimerCancel,
        Action::TimerPause,
        Action::TimerStart,
        Action::ListComplete,
        Action::ListAdd,
        Action::VacuumDock,
        Action::VacuumStart,
        Action::Unlock,
        Action::Lock,
        Action::CoverSet,
        Action::CoverOpen,
        Action::CoverClose,
        Action::FanSpeed,
        Action::SetTemp,
        Action::SetLight,
        Action::Scene,
        Action::MediaPause,
        Action::MediaPlay,
        Action::MediaNext,
        Action::MediaMute,
        Action::Toggle,
        Action::Off,
        Action::On,
    ];
    for wanted in RANK {
        if let Some((_, a)) = actions.iter().find(|(_, a)| a == wanted) {
            return Some(*a);
        }
    }
    actions.iter().find(|(_, a)| !matches!(a, Action::GetState)).map(|(_, a)| *a)
}
pub(crate) fn wants_light_clarify(tokens: &[String], home: &HomeGraph, areas: &[String]) -> bool {
    let cat = catalog();
    if !cat.any(tokens, &cat.light_singular) || cat.any(tokens, &cat.light_plural) || cat.any(tokens, &cat.illuminate) {
        return false;
    }
    if areas.iter().any(|area| crate::compound::room_light_id(home, area).is_some()) {
        return false;
    }
    home.entities
        .iter()
        .filter(|e| e.domain == "light" && !crate::home_policy::is_infra_light(e) && e.area.as_ref().is_some_and(|a| areas.contains(a)))
        .count()
        > 1
}

pub(crate) fn wants_all_lights(tokens: &[String]) -> bool {
    tokens.iter().any(|t| catalog().is_all(t)) && catalog().any(tokens, &catalog().light_nouns)
}

pub(crate) fn looks_like_named_device(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().named_device)
}

pub(crate) fn color_word(tokens: &[String]) -> Option<String> {
    tokens.iter().find_map(|t| catalog().color(t).map(str::to_string))
}

pub(crate) fn looks_like_question(tokens: &[String]) -> bool {
    let cat = catalog();
    tokens.first().is_some_and(|t| cat.is_question_start(t))
        || tokens.iter().any(|t| cat.is_question_word(t))
        || (cat.any(tokens, &cat.on_words) && cat.any(tokens, &cat.off_words))
        || tokens.iter().any(|t| cat.is_or(t))
}

pub(crate) fn looks_like_correction(tokens: &[String]) -> bool {
    let blob = join_tokens(tokens);
    catalog().correction.iter().any(|w| blob.contains(w))
        || catalog().correction_phrases.iter().any(|phrase| blob.contains(phrase))
        || (tokens.iter().any(|t| t == "nein") && catalog().any(tokens, &catalog().correction))
}

pub(crate) fn pick_clarification(tokens: &[String], session: &Session) -> Option<String> {
    let pending = session.pending_clarify.as_ref()?;
    if tokens.iter().any(|t| catalog().clarify_pick.contains(t.as_str())) {
        return pending.first().cloned();
    }
    let blob = join_tokens(tokens);
    pending
        .iter()
        .find(|id| {
            let tail = id.rsplit('.').next().unwrap_or(id).replace('_', " ");
            let folded = fold_umlaut(&tail);
            blob.contains(&folded)
                || tokens.iter().any(|t| {
                    let aliases = fixture_aliases(t);
                    aliases
                        .iter()
                        .any(|a| (folded.contains(a) && a.len() > 2) || tail.split_whitespace().any(|p| a.contains(p) && p.len() > 2))
                })
        })
        .cloned()
}

fn fixture_aliases(token: &str) -> Vec<&str> {
    let aliases = catalog().fixture_alias(token);
    if aliases.is_empty() {
        vec![token]
    } else {
        aliases.to_vec()
    }
}

pub(crate) fn match_custom(tokens: &[String], text: &str, custom: &[CustomSentence]) -> Option<Intent> {
    let blob = join_tokens(tokens);
    let folded = fold_umlaut(text);
    let mut best: Option<(f64, &CustomSentence)> = None;
    for c in custom {
        if !known_intent(&c.intent) {
            continue;
        }
        let phrase = fold_umlaut(&c.phrase);
        if phrase.chars().count() < 4 {
            continue;
        }
        if blob == phrase || folded == phrase {
            return Some(custom_to_intent(c));
        }
        let words: Vec<&str> = phrase.split_whitespace().filter(|w| !w.is_empty()).collect();
        if words.len() >= 2 && words.iter().all(|word| tokens.iter().any(|token| token == word)) {
            return Some(custom_to_intent(c));
        }
        if phrase.chars().count() < 8 {
            continue;
        }
        let score = normalized_levenshtein(&blob, &phrase);
        if score > 0.92 && best.as_ref().is_none_or(|(s, _)| score > *s) {
            best = Some((score, c));
        }
    }
    best.map(|(_, c)| custom_to_intent(c))
}

fn custom_to_intent(c: &CustomSentence) -> Intent {
    let mut intent = Intent::new(&c.intent);
    for (k, v) in &c.slots {
        intent = intent.with(k, v);
    }
    intent
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn row(phrase: &str, intent: &str) -> CustomSentence {
        CustomSentence { phrase: phrase.into(), intent: intent.into(), slots: HashMap::new() }
    }

    #[test]
    fn custom_rejects_unknown_intent_and_short_phrase() {
        let tokens = vec!["filmabend".into()];
        assert!(match_custom(&tokens, "filmabend", &[row("an", "HassTurnOn")]).is_none());
        assert!(match_custom(&tokens, "filmabend", &[row("filmabend", "NotAnIntent")]).is_none());
    }

    #[test]
    fn custom_matches_exact_known_intent() {
        let tokens = vec!["filmabend".into()];
        let hit = match_custom(&tokens, "filmabend", &[row("filmabend", "HassTurnOn")]).expect("hit");
        assert_eq!(hit.name, "HassTurnOn");
    }

    #[test]
    fn bind_pairs_action_with_target_domain() {
        assert_eq!(bind_domain(Action::SetLight, &[], None, Some("switch")), Action::On);
        assert_eq!(bind_domain(Action::On, &[], Some(21), Some("climate")), Action::SetTemp);
        assert_eq!(bind_domain(Action::SetLight, &[], Some(40), Some("cover")), Action::CoverSet);
        let licht = vec!["licht".into()];
        assert_eq!(bind_domain(Action::SetLight, &licht, Some(21), Some("climate")), Action::SetLight);
        assert_eq!(bind_domain(Action::On, &licht, Some(50), Some("light")), Action::SetLight);
    }

    #[test]
    fn session_cover_follow_opens_without_verb() {
        let mut session = Session::new();
        session.remember(&Intent::new("HassGetState").with("entity_id", "cover.garage_door"));
        let tokens = vec!["mach".into(), "auf".into()];
        assert_eq!(infer_action(Action::GetState, &tokens, None, false, &session, None), Action::CoverOpen);
        assert_eq!(infer_action(Action::GetState, &tokens, None, true, &session, Some("cover")), Action::GetState);
    }
}
