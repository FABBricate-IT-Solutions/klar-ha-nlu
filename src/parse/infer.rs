use crate::lang::catalog;
use crate::lang::VerbKind;
use crate::parse::action::{has_cover_noun, has_light_noun, Action};
use crate::parse::fuzzy::{select_unique, Profile};
use crate::parse::normalize::{fold_umlaut, join_tokens};
use crate::session::Session;
use crate::types::{known_intent, CustomSentence, EntityRec, Intent};

fn dock_hint(tokens: &[String]) -> bool {
    tokens.iter().any(|token| {
        matches!(catalog().verb(token), Some(VerbKind::Dock))
            || matches!(token.as_str(), "dock" | "docking" | "station" | "base" | "zurueck")
    })
}

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
    let vacuum = catalog().any(tokens, catalog().vacuum_nouns());
    if question && (vacuum || matches!(action, Action::VacuumDock | Action::VacuumStart)) {
        return Action::GetState;
    }
    if vacuum && dock_hint(tokens) {
        return Action::VacuumDock;
    }
    if vacuum
        && (matches!(action, Action::On)
            || catalog().any(tokens, catalog().start_words())
            || catalog().any(tokens, catalog().vacuum_start()))
    {
        return Action::VacuumStart;
    }
    if matches!(action, Action::On | Action::Off) && tokens.iter().any(|t| catalog().timer_nouns().contains(t.as_str())) {
        return if matches!(action, Action::Off) { Action::TimerCancel } else { Action::TimerStart };
    }
    if matches!(action, Action::FanSpeed | Action::TimerStart | Action::TimerAdd)
        && number.is_none()
        && (matches!(action, Action::FanSpeed) || question || tokens.iter().any(|t| catalog().timer_query().contains(t.as_str())))
    {
        return Action::GetState;
    }
    if matches!(action, Action::SetLight) && number.is_none() && color_word(tokens).is_none() && question {
        return Action::GetState;
    }
    action
}

pub(crate) fn mentions_lamp_fixture(tokens: &[String]) -> bool {
    tokens.iter().any(|token| token == "lamp" || catalog().lamp_fixture().contains(token.as_str()))
}

fn session_domain(session: &Session, tokens: &[String]) -> Option<&'static str> {
    let cat = catalog();
    if has_light_noun(tokens)
        || has_cover_noun(tokens)
        || cat.any(tokens, cat.ceiling())
        || cat.any(tokens, cat.named_device())
        || cat.any(tokens, cat.island())
        || mentions_lamp_fixture(tokens)
        || cat.any(tokens, cat.bedside())
    {
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
            Some("fan") if !cat.any(tokens, cat.kitchen()) => return Action::FanSpeed,
            Some("media_player") => return Action::On,
            _ => {}
        }
    }
    if matches!(action, Action::On | Action::Off | Action::Lock)
        && !has_cover_noun(tokens)
        && (domain == Some("lock") || cat.any(tokens, cat.lock_nouns()))
        && (cat.any(tokens, cat.unlock_follow()) || cat.any(tokens, cat.open_words()))
    {
        return Action::Unlock;
    }
    let cover_follow = matches!(action, Action::On | Action::Off) || (session_follow && matches!(action, Action::GetState));
    if cover_follow && domain == Some("cover") && !light_noun && number.is_none() {
        if cat.any(tokens, cat.cover_open_follow()) {
            return Action::CoverOpen;
        }
        if cat.any(tokens, cat.close_words()) {
            return Action::CoverClose;
        }
    }
    if matches!(action, Action::On)
        && (color_word(tokens).is_some() || number.is_some())
        && (domain == Some("light") || light_noun || cat.any(tokens, cat.ceiling()))
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
pub(crate) fn wants_all_lights(tokens: &[String]) -> bool {
    let cat = catalog();
    if !cat.any(tokens, cat.light_nouns()) && !cat.any(tokens, cat.light_plural()) {
        return false;
    }
    tokens.iter().any(|t| cat.is_all(t))
        || tokens.iter().any(|t| cat.is_except(t))
        || (cat.any(tokens, cat.status_words()) && cat.any(tokens, cat.light_plural()))
}

pub(crate) fn except_tail(tokens: &[String]) -> Option<&[String]> {
    if let Some(at) = tokens.iter().position(|token| catalog().is_except(token)) {
        let tail = &tokens[at + 1..];
        if !tail.is_empty() {
            return Some(tail);
        }
        let head = except_head_focus(&tokens[..at]);
        return (!head.is_empty()).then_some(head);
    }
    if let Some(tail) = tokens.windows(2).enumerate().find_map(|(at, pair)| {
        let phrase = matches!(
            (pair[0].as_str(), pair[1].as_str()),
            ("bis", "auf") | ("but", "not") | ("except", "for") | ("aber", "nicht") | ("nicht", "im") | ("nicht", "in")
        );
        let tail = &tokens[at + 2..];
        (phrase && !tail.is_empty()).then_some(tail)
    }) {
        return Some(tail);
    }
    if tokens.iter().any(|token| catalog().is_all(token)) {
        if let Some(at) = tokens.iter().position(|token| matches!(token.as_str(), "nicht" | "not")) {
            let tail = &tokens[at + 1..];
            return (!tail.is_empty()).then_some(tail);
        }
    }
    None
}

fn except_head_focus(tokens: &[String]) -> &[String] {
    let cat = catalog();
    let start = tokens
        .iter()
        .rposition(|token| cat.verb(token).is_some() || cat.is_all(token) || cat.is_particle(token))
        .map(|index| index + 1)
        .unwrap_or(0);
    &tokens[start..]
}

/// Exception focus without the command verb leaked after the room ("… außer Schlafzimmer ausschalten").
pub(crate) fn except_focus(tokens: &[String]) -> Option<Vec<String>> {
    let tail = except_tail(tokens)?;
    let cat = catalog();
    let focus: Vec<String> =
        tail.iter().filter(|token| cat.verb(token).is_none() && !cat.is_particle(token) && !cat.is_all(token)).cloned().collect();
    (!focus.is_empty()).then_some(focus)
}

pub(crate) fn looks_like_named_device(tokens: &[String]) -> bool {
    catalog().any(tokens, catalog().named_device())
}

pub(crate) fn color_word(tokens: &[String]) -> Option<String> {
    tokens.iter().find_map(|t| catalog().color(t).map(str::to_string))
}

pub(crate) fn looks_like_question(tokens: &[String]) -> bool {
    let cat = catalog();
    tokens.first().is_some_and(|t| cat.is_question_start(t))
        || tokens.iter().any(|t| cat.is_question_word(t))
        || (cat.any(tokens, cat.on_words()) && cat.any(tokens, cat.off_words()))
        || tokens.iter().any(|t| cat.is_or(t))
}

pub(crate) fn looks_like_correction(tokens: &[String]) -> bool {
    let blob = join_tokens(tokens);
    catalog().correction().iter().any(|w| blob.contains(w)) || catalog().correction_phrases().iter().any(|phrase| blob.contains(phrase))
}

pub(crate) fn pick_clarification(tokens: &[String], session: &Session) -> Option<String> {
    let pending = &session.pending_clarify()?.options;
    if tokens.iter().any(|t| catalog().clarify_pick().contains(t.as_str())) {
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

pub(crate) fn fixture_matches(entity: &EntityRec, needle: &str) -> bool {
    let blob = format!("{} {} {}", entity.entity_id, fold_umlaut(&entity.name), entity.aliases.join(" "));
    let hits = fixture_aliases(needle);
    let matched = hits.iter().any(|alias| blob.contains(alias));
    let cat = catalog();
    if needle == "lamp" || cat.lamp_fixture().contains(needle) {
        matched && !blob.contains("decke") && cat.ceiling().iter().all(|word| !blob.contains(word))
    } else {
        matched
    }
}

fn fixture_aliases(token: &str) -> Vec<&str> {
    let cat = catalog();
    let aliases = cat.fixture_alias(token);
    let mut out: Vec<&str> = if aliases.is_empty() { vec![token] } else { aliases.to_vec() };
    if token == "ceiling" || cat.ceiling().contains(token) {
        out.extend(["ceiling", "decke", "deckenlampe"]);
    }
    if token == "island" || cat.island().contains(token) {
        out.extend(["island", "insel"]);
    }
    if token == "lamp" || cat.lamp_fixture().contains(token) {
        out.extend(["lamp", "lampe"]);
    }
    if cat.bedside().contains(token) {
        out.extend(["nacht", "nachttisch", "bedside"]);
    }
    if token == "globe" || token == "kugel" || aliases.iter().any(|alias| *alias == "globe") {
        out.extend(["kugel", "globe"]);
    }
    out
}

pub(crate) fn match_custom(tokens: &[String], text: &str, custom: &[CustomSentence], allowed: &[String]) -> Option<Intent> {
    let blob = join_tokens(tokens);
    let folded = fold_umlaut(text);
    let mut candidates: Vec<(&CustomSentence, String)> = Vec::new();
    for c in custom {
        if !known_intent(&c.intent) && !allowed.iter().any(|name| name == &c.intent) {
            continue;
        }
        let phrase = fold_umlaut(&c.phrase);
        if phrase.chars().count() < 4 {
            continue;
        }
        if blob == phrase || folded == phrase {
            return Some(custom_to_intent(c));
        }
        if !c.slots.is_empty() {
            continue;
        }
        let words: Vec<&str> = phrase.split_whitespace().filter(|w| !w.is_empty()).collect();
        if words.len() >= 2 && words.iter().all(|word| tokens.iter().any(|token| token == word)) {
            return Some(custom_to_intent(c));
        }
        if phrase.chars().count() < 8 {
            continue;
        }
        if !protected_tokens_match(tokens, &words) {
            continue;
        }
        candidates.push((c, phrase));
    }
    let hit =
        select_unique(&blob, candidates.iter().map(|(candidate, phrase)| (candidate.phrase.as_str(), phrase.as_str())), Profile::Phrase)?;
    candidates.iter().find(|(candidate, _)| candidate.phrase == hit.key).map(|(candidate, _)| custom_to_intent(candidate))
}

fn protected_tokens_match(tokens: &[String], phrase: &[&str]) -> bool {
    let protected = |token: &str| catalog().number(token).is_some() || token.parse::<i32>().is_ok() || catalog().color(token).is_some();
    let mut spoken: Vec<&str> = tokens.iter().map(String::as_str).filter(|token| protected(token)).collect();
    let mut configured: Vec<&str> = phrase.iter().copied().filter(|token| protected(token)).collect();
    spoken.sort_unstable();
    spoken.dedup();
    configured.sort_unstable();
    configured.dedup();
    spoken == configured
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
        assert!(match_custom(&tokens, "filmabend", &[row("an", "HassTurnOn")], &[]).is_none());
        assert!(match_custom(&tokens, "filmabend", &[row("filmabend", "NotAnIntent")], &[]).is_none());
    }

    #[test]
    fn custom_matches_exact_known_intent() {
        let tokens = vec!["filmabend".into()];
        let hit = match_custom(&tokens, "filmabend", &[row("filmabend", "HassTurnOn")], &[]).expect("hit");
        assert_eq!(hit.name, "HassTurnOn");
    }

    #[test]
    fn custom_fuzzy_match_requires_unique_whole_phrase() {
        let tokens = vec!["starte".into(), "den".into(), "filmabent".into()];
        let hit =
            match_custom(&tokens, "starte den filmabent", &[row("starte den filmabend", "HassTurnOn")], &[]).expect("unique fuzzy hit");
        assert_eq!(hit.name, "HassTurnOn");

        let ambiguous = [row("starte den filmabend", "HassTurnOn"), row("starte den filmabenz", "HassTurnOff")];
        assert!(match_custom(&tokens, "starte den filmabent", &ambiguous, &[]).is_none());
    }

    #[test]
    fn custom_fuzzy_match_never_changes_protected_payloads() {
        let tokens = vec!["stelle".into(), "heizung".into(), "auf".into(), "22".into()];
        let mut slotted = row("stelle heizung auf 21", "HassClimateSetTemperature");
        slotted.slots.insert("temperature".into(), "21".into());
        assert!(match_custom(&tokens, "stelle heizung auf 22", std::slice::from_ref(&slotted), &[]).is_none());
        let contradictory = vec!["stelle".into(), "heizung".into(), "auf".into(), "21".into(), "statt".into(), "22".into()];
        assert!(match_custom(&contradictory, "stelle heizung auf 21 statt 22", &[slotted], &[]).is_none());
        assert!(match_custom(&tokens, "stelle heizung auf 22", &[row("stelle heizung auf 21", "HassTurnOn")], &[]).is_none());
    }

    #[test]
    fn bind_pairs_action_with_target_domain() {
        assert_eq!(bind_domain(Action::SetLight, &[], None, Some("switch")), Action::On);
        assert_eq!(bind_domain(Action::On, &[], Some(21), Some("climate")), Action::SetTemp);
        assert_eq!(bind_domain(Action::SetLight, &[], Some(40), Some("cover")), Action::CoverSet);
        let licht = vec!["licht".into()];
        assert_eq!(bind_domain(Action::SetLight, &licht, Some(21), Some("climate")), Action::SetLight);
        assert_eq!(bind_domain(Action::On, &licht, Some(50), Some("light")), Action::SetLight);
        let _th = crate::lang::bind(&["th".into()]);
        let open_lock = vec!["เปิด".into(), "กุญแจ".into()];
        assert_eq!(bind_domain(Action::On, &open_lock, None, Some("lock")), Action::Unlock);
        assert_eq!(bind_domain(Action::Lock, &open_lock, None, Some("lock")), Action::Unlock);
    }

    #[test]
    fn vacuum_question_stays_get_state() {
        let session = Session::new();
        let tokens = vec!["ist".into(), "r2d2".into(), "an".into()];
        assert_eq!(infer_action(Action::On, &tokens, None, true, &session, Some("vacuum")), Action::GetState);
        assert_eq!(infer_action(Action::GetState, &tokens, None, true, &session, Some("vacuum")), Action::GetState);
        assert_eq!(infer_action(Action::On, &["r2d2".into(), "an".into()], None, false, &session, Some("vacuum")), Action::VacuumStart);
        assert_eq!(
            infer_action(Action::On, &["dock".into(), "staubsauger".into()], None, false, &session, Some("vacuum")),
            Action::VacuumDock
        );
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
