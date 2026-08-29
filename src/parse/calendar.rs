use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::infer::looks_like_question;
use crate::parse::normalize::{compact, is_word_char};
use crate::parse::slots::ClauseOut;
use crate::types::{EntityRec, HomeGraph, Intent};

pub(crate) fn mentions_calendar(tokens: &[String]) -> bool {
    let cat = catalog();
    any_lexeme(tokens, cat.calendar_nouns()) || any_exact(tokens, cat.calendar_query()) || agenda_query(tokens)
}

pub(crate) fn calendar_clause(
    tokens: &[String],
    _home: &HomeGraph,
    _action: Action,
    number: Option<i32>,
    _domain: Option<&str>,
) -> Option<ClauseOut> {
    let cat = catalog();
    let has_noun = any_lexeme(tokens, cat.calendar_nouns());
    let has_query = any_exact(tokens, cat.calendar_query());
    let has_create = any_stem(tokens, cat.calendar_create());
    let has_delete = any_stem(tokens, cat.calendar_delete());
    let has_move = any_stem(tokens, cat.calendar_move());
    let agenda = agenda_query(tokens);
    if !has_noun && !has_query && !agenda {
        return None;
    }
    if other_domain_noun(tokens) && !has_noun {
        return None;
    }
    if cat.any(tokens, cat.list_nouns()) && !has_noun {
        return None;
    }
    let question = looks_like_question(tokens);
    let when = when_slots(tokens, number);
    let summary = title_from_tokens(tokens);
    let follow = has_noun || has_query || summary.is_some() || has_anaphora(tokens);
    if has_delete && follow && !has_create {
        return Some(delete_outcome(summary.as_deref()));
    }
    if has_move && (follow || when.has_date) && !has_create {
        return Some(move_outcome(summary.as_deref(), &when));
    }
    if !has_noun && !has_query && !agenda {
        return None;
    }
    let create = !question && has_create || (!question && has_noun && when.has_date && summary.is_some() && !has_query && !agenda);
    if create {
        return Some(create_outcome(summary.as_deref(), &when));
    }
    Some(ClauseOut::Intents(vec![list_intent(&when)]))
}

fn any_lexeme(tokens: &[String], set: &std::collections::HashSet<&str>) -> bool {
    tokens.iter().any(|token| set.iter().any(|word| lexeme_hit(token, word))) || phrase_hit(tokens, set)
}

fn any_exact(tokens: &[String], set: &std::collections::HashSet<&str>) -> bool {
    tokens.iter().any(|token| set.contains(token.as_str())) || phrase_hit(tokens, set)
}

fn any_stem(tokens: &[String], set: &std::collections::HashSet<&str>) -> bool {
    tokens.iter().any(|token| set.iter().any(|word| stem_hit(token, word))) || phrase_hit(tokens, set)
}

fn stem_hit(token: &str, word: &str) -> bool {
    if token == word {
        return true;
    }
    if word.len() < 4 || token.len() < word.len() {
        return false;
    }
    token.starts_with(word)
        && matches!(token.get(word.len()..).unwrap_or(""), "" | "n" | "en" | "e" | "s" | "t" | "te" | "ten" | "er" | "est")
}

fn phrase_hit(tokens: &[String], set: &std::collections::HashSet<&str>) -> bool {
    set.iter().any(|phrase| {
        let parts: Vec<&str> = phrase.split(|c: char| !is_word_char(c)).filter(|part| !part.is_empty()).collect();
        parts.len() > 1 && tokens.windows(parts.len()).any(|window| window.iter().zip(parts.iter()).all(|(token, part)| token == part))
    })
}

fn lexeme_hit(token: &str, word: &str) -> bool {
    if token == word {
        return true;
    }
    if token.len() < 4 || word.len() < 4 {
        return false;
    }
    if token.starts_with(word) || word.starts_with(token) || token.ends_with(word) {
        return true;
    }
    token.chars().zip(word.chars()).take_while(|(left, right)| left == right).count() >= 5
}

fn other_domain_noun(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, cat.light_nouns())
        || cat.any(tokens, cat.climate_nouns())
        || cat.any(tokens, cat.cover_nouns())
        || cat.any(tokens, cat.fan_nouns())
        || cat.any(tokens, cat.lock_nouns())
        || cat.any(tokens, cat.vacuum_nouns())
        || cat.any(tokens, cat.media_nouns())
        || cat.any(tokens, cat.timer_nouns())
        || cat.any(tokens, cat.scene_nouns())
}

fn agenda_query(tokens: &[String]) -> bool {
    let cat = catalog();
    let day = cat.any(tokens, cat.calendar_today()) || has_tomorrow(tokens);
    let ask = looks_like_question(tokens)
        || any_exact(tokens, cat.calendar_query())
        || tokens.iter().any(|token| matches!(token.as_str(), "habe" | "haben" | "have" | "got" | "steht"));
    day && ask && !other_domain_noun(tokens)
}

fn has_tomorrow(tokens: &[String]) -> bool {
    let cat = catalog();
    tokens.iter().enumerate().any(|(index, token)| {
        if greeting_morning_word(token) || !cat.calendar_tomorrow().contains(token.as_str()) {
            return false;
        }
        !greeting_morning_pair(index.checked_sub(1).and_then(|prev| tokens.get(prev)).map(String::as_str), token)
    })
}

fn greeting_morning_word(token: &str) -> bool {
    matches!(token, "gutenmorgen" | "goedenmorgen")
}

fn greeting_morning_pair(prev: Option<&str>, token: &str) -> bool {
    matches!((prev.unwrap_or(""), token), ("guten", "morgen") | ("goeden", "morgen") | ("good", "morning"))
}

fn list_intent(when: &WhenSlots) -> Intent {
    let mut intent = Intent::new("KlarGetCalendarEvents").with("domain", "calendar");
    if let Some(day) = when.day {
        intent = intent.with("day", day);
    }
    if let Some(days) = when.in_days {
        intent = intent.with("in_days", days.to_string());
    }
    intent
}

fn delete_outcome(summary: Option<&str>) -> ClauseOut {
    let mut intent = Intent::new("KlarDeleteCalendarEvent").with("domain", "calendar");
    if let Some(summary) = summary {
        intent = intent.with("summary", summary);
    }
    ClauseOut::Intents(vec![intent])
}

fn move_outcome(summary: Option<&str>, when: &WhenSlots) -> ClauseOut {
    let mut intent = Intent::new("KlarMoveCalendarEvent").with("domain", "calendar");
    if let Some(summary) = summary {
        intent = intent.with("summary", summary);
    }
    if let Some(day) = when.day {
        intent = intent.with("day", day);
    }
    if let Some(hour) = when.hour {
        intent = intent.with("hour", hour.to_string());
    }
    if let Some(days) = when.in_days {
        intent = intent.with("in_days", days.to_string());
    }
    if !when.has_date {
        return ClauseOut::Clarify(Vec::new(), intent.with("need", "when"));
    }
    ClauseOut::Intents(vec![intent])
}

fn has_anaphora(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "it" | "that" | "this" | "den" | "ihn" | "das" | "es" | "le" | "la" | "lo"))
}

fn create_outcome(summary: Option<&str>, when: &WhenSlots) -> ClauseOut {
    let mut intent = Intent::new("KlarCreateCalendarEvent").with("domain", "calendar");
    if let Some(summary) = summary {
        intent = intent.with("summary", summary);
    }
    if let Some(day) = when.day {
        intent = intent.with("day", day);
    }
    if let Some(hour) = when.hour {
        intent = intent.with("hour", hour.to_string());
    }
    if let Some(days) = when.in_days {
        intent = intent.with("in_days", days.to_string());
    }
    if summary.is_none() {
        return ClauseOut::Clarify(Vec::new(), intent.with("need", "title"));
    }
    if !when.has_date {
        return ClauseOut::Clarify(Vec::new(), intent.with("need", "when"));
    }
    ClauseOut::Intents(vec![intent])
}

struct WhenSlots {
    day: Option<&'static str>,
    hour: Option<i32>,
    in_days: Option<i32>,
    has_date: bool,
}

fn when_slots(tokens: &[String], number: Option<i32>) -> WhenSlots {
    let cat = catalog();
    let today = cat.any(tokens, cat.calendar_today());
    let tomorrow = has_tomorrow(tokens);
    let weekday = tokens.iter().any(|token| cat.calendar_when().contains(token.as_str()) && !clock_particle(token));
    let in_days = in_days_slot(tokens, number);
    let hour = clock_hour(tokens).or_else(|| number.filter(|value| (0..=23).contains(value)));
    let has_date = today || tomorrow || weekday || hour.is_some() || in_days.is_some();
    let day = if today {
        Some("today")
    } else if tomorrow {
        Some("tomorrow")
    } else {
        None
    };
    WhenSlots { day, hour, in_days, has_date }
}

fn clock_hour(tokens: &[String]) -> Option<i32> {
    tokens.iter().enumerate().find_map(|(index, token)| {
        let value = token.parse::<i32>().ok().filter(|hour| (0..=23).contains(hour))?;
        clock_number_at(tokens, index).then_some(value)
    })
}

fn clock_number_at(tokens: &[String], index: usize) -> bool {
    let prev = index.checked_sub(1).and_then(|prev| tokens.get(prev)).map(String::as_str).unwrap_or("");
    let next = tokens.get(index + 1).map(String::as_str).unwrap_or("");
    clock_particle(next) || matches!(prev, "um" | "at" | "à" | "a")
}

fn clock_particle(token: &str) -> bool {
    let cat = catalog();
    token.len() <= 3 || cat.hours().contains(token) || cat.minutes().contains(token) || cat.seconds().contains(token)
}

fn in_days_slot(tokens: &[String], number: Option<i32>) -> Option<i32> {
    let number = number.filter(|value| (1..=14).contains(value))?;
    let has_in = tokens.iter().any(|token| matches!(token.as_str(), "in" | "dans" | "en" | "za" | "om"));
    let day_unit = tokens.iter().any(|token| token.contains("tag") || token.contains("day") || token.contains("jour"));
    (has_in && day_unit).then_some(number)
}

fn title_from_tokens(tokens: &[String]) -> Option<String> {
    title_after_event_noun(tokens).or_else(|| join_title(&title_leftover_scan(tokens)))
}

fn title_after_event_noun(tokens: &[String]) -> Option<String> {
    let start = tokens.iter().position(|token| event_noun(token))?;
    join_title(
        &tokens[start + 1..]
            .iter()
            .take_while(|token| !title_boundary(token.as_str()))
            .map(String::as_str)
            .filter(|value| keep_title_token(value))
            .collect::<Vec<_>>(),
    )
}

fn event_noun(token: &str) -> bool {
    let cat = catalog();
    cat.calendar_nouns().contains(token) || ["termin", "event", "appointment", "rendezvous"].iter().any(|word| lexeme_hit(token, word))
}

fn title_boundary(token: &str) -> bool {
    let cat = catalog();
    cat.calendar_today().contains(token)
        || cat.calendar_tomorrow().contains(token)
        || greeting_morning_word(token)
        || (cat.calendar_when().contains(token) && !clock_particle(token))
        || cat.hours().contains(token)
        || matches!(token, "um" | "at" | "à")
}

fn keep_title_token(value: &str) -> bool {
    let cat = catalog();
    if title_function_word(value) {
        return false;
    }
    !cat.calendar_nouns().contains(value)
        && !cat.calendar_query().contains(value)
        && !cat.calendar_create().iter().any(|word| stem_hit(value, word) || lexeme_hit(value, word))
        && !cat.calendar_delete().iter().any(|word| stem_hit(value, word) || lexeme_hit(value, word))
        && !cat.calendar_move().iter().any(|word| stem_hit(value, word) || lexeme_hit(value, word))
        && !cat.list_skip().contains(value)
        && !cat.on_words().contains(value)
        && !cat.off_words().contains(value)
}

fn title_function_word(value: &str) -> bool {
    matches!(
        value,
        "it" | "that"
            | "this"
            | "the"
            | "a"
            | "an"
            | "to"
            | "my"
            | "your"
            | "our"
            | "their"
            | "for"
            | "of"
            | "on"
            | "and"
            | "or"
            | "into"
            | "from"
            | "with"
            | "den"
            | "dem"
            | "der"
            | "die"
            | "das"
            | "ihn"
            | "es"
            | "ein"
            | "eine"
            | "einen"
            | "einem"
            | "einer"
            | "eines"
            | "mein"
            | "meine"
            | "meinen"
            | "in"
            | "ins"
            | "im"
            | "um"
            | "at"
            | "und"
            | "zum"
            | "zur"
            | "auf"
            | "aus"
            | "bei"
            | "mit"
            | "nach"
            | "von"
            | "vor"
            | "le"
            | "la"
            | "lo"
    )
}

fn title_leftover_scan(tokens: &[String]) -> Vec<&str> {
    tokens
        .iter()
        .enumerate()
        .filter(|(index, value)| keep_title_token(value) && !title_boundary(value) && !clock_number_at(tokens, *index))
        .map(|(_, value)| value.as_str())
        .collect()
}

fn join_title(words: &[&str]) -> Option<String> {
    if words.is_empty() {
        None
    } else if words.len() > 1 && words.iter().all(|word| word.chars().all(|c| c.is_ascii_alphanumeric())) {
        Some(words.join("-"))
    } else {
        Some(words.join(" "))
    }
}

pub(crate) fn is_calendar_control(entity: &EntityRec) -> bool {
    if entity.domain == "calendar" {
        return false;
    }
    let blob = compact(&format!("{} {}", entity.name, entity.aliases.join(" ")));
    let calendar = ["calendar", "kalender", "termin", "agenda", "calendrier"].iter().any(|word| blob.contains(word));
    let create =
        ["create", "anlegen", "add", "schedule", "erstellen", "delete", "remove", "cancel", "loesch", "lösch", "move", "verschieb"]
            .iter()
            .any(|word| blob.contains(word));
    calendar && create
}

pub(crate) fn speak_calendar_need(intent: &Intent) -> String {
    let pack = catalog().speech();
    match intent.slot("need") {
        Some("when") => pack.calendar_need_when.to_string(),
        Some("which") => pack.calendar_which.to_string(),
        _ => pack.calendar_need_title.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::mentions_calendar;
    use crate::parse::normalize::{strip_fillers, tokenize};

    #[test]
    fn guten_morgen_is_not_a_calendar_cue() {
        let _bind = crate::lang::bind(&["de".into()]);
        let tokens = strip_fillers(&tokenize("Ist Guten Morgen an"));
        assert!(!mentions_calendar(&tokens), "{tokens:?}");
    }

    #[test]
    fn french_rendezvous_is_a_calendar_cue() {
        let _bind = crate::lang::bind(&["fr".into()]);
        let tokens = strip_fillers(&tokenize("quels sont mes rendez-vous"));
        assert!(mentions_calendar(&tokens), "{tokens:?}");
    }

    #[test]
    fn hyphenated_retest_title_keeps_klar_and_number() {
        let _bind = crate::lang::bind(&["de".into()]);
        let tokens = crate::parse::normalize::tokenize("Trage den Termin Klar-Retest-62 morgen um 15 Uhr in den Kalender ein");
        let title = super::title_from_tokens(&tokens).unwrap_or_default();
        assert!(title.contains("klar-retest-62"), "{title:?} tokens={tokens:?}");
        assert!(!title.contains("15"), "{title:?}");
    }

    #[test]
    fn untitled_add_to_calendar_has_no_leftover_title() {
        let _bind = crate::lang::bind(&["en".into()]);
        let tokens = crate::parse::normalize::tokenize("add to my calendar tomorrow at 3");
        assert_eq!(super::title_from_tokens(&tokens), None, "{tokens:?}");
    }

    #[test]
    fn french_list_smoke_executes() {
        let home = crate::home::default_home();
        let settings = crate::types::Settings { languages: vec!["fr".into()], ..crate::types::Settings::default() };
        let mut session = crate::session::Session::default();
        let parsed = crate::nlu::parse("quels sont mes rendez-vous", &home, &mut session, &[], &settings);
        let names: Vec<_> = parsed.plan.as_ref().map(|plan| plan.intents().into_iter().map(|item| item.name).collect()).unwrap_or_default();
        assert!(
            matches!(parsed.decision, crate::types::ParseDecision::Execute) && names.iter().any(|name| name == "KlarGetCalendarEvents"),
            "decision={:?} names={names:?} speech={} trace={:#?} candidates={:#?}",
            parsed.decision,
            parsed.speech,
            parsed.trace,
            parsed.candidates
        );
    }
}
