use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::infer::looks_like_question;
use crate::parse::normalize::compact;
use crate::parse::slots::ClauseOut;
use crate::types::{EntityRec, HomeGraph, Intent};

pub(crate) fn mentions_calendar(tokens: &[String]) -> bool {
    let cat = catalog();
    any_lexeme(tokens, cat.calendar_nouns()) || any_exact(tokens, cat.calendar_query())
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
    if !has_noun && !has_query {
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
    let summary = title_leftover(tokens);
    let follow = has_noun || has_query || summary.is_some() || has_anaphora(tokens);
    if has_delete && follow && !has_create {
        return Some(delete_outcome(summary.as_deref()));
    }
    if has_move && (follow || when.has_date) && !has_create {
        return Some(move_outcome(summary.as_deref(), &when));
    }
    if !has_noun && !has_query {
        return None;
    }
    let create = !question && has_create || (!question && has_noun && when.has_date && summary.is_some() && !has_query);
    if create {
        return Some(create_outcome(summary.as_deref(), &when));
    }
    Some(ClauseOut::Intents(vec![list_intent()]))
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
        let parts: Vec<&str> = phrase.split(|c: char| !c.is_alphanumeric()).filter(|part| !part.is_empty()).collect();
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

fn list_intent() -> Intent {
    Intent::new("KlarGetCalendarEvents").with("domain", "calendar")
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
    let tomorrow = cat.any(tokens, cat.calendar_tomorrow());
    let weekday = tokens.iter().any(|token| cat.calendar_when().contains(token.as_str()) && !clock_particle(token));
    let in_days = in_days_slot(tokens, number);
    let hour = number.filter(|value| (0..=23).contains(value));
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

fn title_leftover(tokens: &[String]) -> Option<String> {
    let cat = catalog();
    let words: Vec<&str> = tokens
        .iter()
        .filter(|token| {
            let value = token.as_str();
            !cat.calendar_nouns().iter().any(|word| lexeme_hit(value, word))
                && !cat.calendar_query().iter().any(|word| lexeme_hit(value, word))
                && !cat.calendar_create().iter().any(|word| lexeme_hit(value, word))
                && !cat.calendar_delete().iter().any(|word| lexeme_hit(value, word))
                && !cat.calendar_move().iter().any(|word| lexeme_hit(value, word))
                && !cat.calendar_today().contains(value)
                && !cat.calendar_tomorrow().contains(value)
                && !cat.calendar_when().contains(value)
                && !cat.list_skip().contains(value)
                && !cat.on_words().contains(value)
                && !cat.off_words().contains(value)
                && !cat.hours().contains(value)
                && !cat.minutes().contains(value)
                && !cat.seconds().contains(value)
                && !cat.fillers().contains(value)
                && !matches!(value, "it" | "that" | "this" | "den" | "ihn" | "das" | "es" | "le" | "la" | "lo")
                && value.parse::<i32>().is_err()
        })
        .map(String::as_str)
        .collect();
    if words.is_empty() {
        None
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
    fn french_rendezvous_is_a_calendar_cue() {
        let _bind = crate::lang::bind(&["fr".into()]);
        let tokens = strip_fillers(&tokenize("quels sont mes rendez-vous"));
        assert!(mentions_calendar(&tokens), "{tokens:?}");
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
