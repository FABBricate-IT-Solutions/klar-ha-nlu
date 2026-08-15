use crate::lang::catalog;
use crate::lexicon::{has_light_noun, Action};
use crate::normalize::{fold_umlaut, join_tokens};
use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, Intent};
use strsim::normalized_levenshtein;

fn last_domain(session: &Session, prefix: &str) -> bool {
    session.last_domains.iter().any(|d| d == prefix) || session.last_entities.iter().any(|e| e.starts_with(&format!("{prefix}.")))
}

pub(crate) fn refine_action(action: Action, tokens: &[String], number: Option<i32>, question: bool, session: &Session) -> Action {
    if question && matches!(action, Action::VacuumDock | Action::VacuumStart) {
        return Action::GetState;
    }
    if catalog().any(tokens, &catalog().vacuum_nouns)
        && (matches!(action, Action::On)
            || catalog().any(tokens, &catalog().start_words)
            || tokens.iter().any(|t| matches!(t.as_str(), "an" | "on" | "start")))
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
    if matches!(action, Action::SetLight) && !has_light_noun(tokens) {
        if last_domain(session, "climate") {
            return Action::SetTemp;
        }
        if last_domain(session, "cover") {
            return Action::CoverSet;
        }
        if last_domain(session, "fan") && !catalog().any(tokens, &catalog().kitchen) {
            return Action::FanSpeed;
        }
    }
    if matches!(action, Action::On | Action::Off) && last_domain(session, "lock") && catalog().any(tokens, &catalog().unlock_follow) {
        return Action::Unlock;
    }
    if last_domain(session, "cover") && !has_light_noun(tokens) && number.is_none() {
        if catalog().any(tokens, &catalog().cover_open_follow) {
            return Action::CoverOpen;
        }
        if catalog().any(tokens, &catalog().close_words) {
            return Action::CoverClose;
        }
    }
    if matches!(action, Action::SetLight) && last_domain(session, "media_player") && !has_light_noun(tokens) {
        return Action::On;
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
        .filter(|e| e.domain == "light" && !crate::compound::is_infra_light(e) && e.area.as_ref().is_some_and(|a| areas.contains(a)))
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
        || blob.contains("stimmt nicht")
        || (tokens.iter().any(|t| t == "nein") && tokens.iter().any(|t| t == "falsch" || t == "nicht"))
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
        let phrase = fold_umlaut(&c.phrase);
        if folded.contains(&phrase) || blob == phrase {
            return Some(custom_to_intent(c));
        }
        let score = normalized_levenshtein(&blob, &phrase);
        if score > 0.88 && best.as_ref().is_none_or(|(s, _)| score > *s) {
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
