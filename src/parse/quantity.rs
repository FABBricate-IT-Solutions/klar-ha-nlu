//! Typed quantity lexing: signs, decimals, fractions, clock times, compound durations,
//! and relative temperature steps. Slot fillers read these instead of silently clipping.

use crate::lang::{catalog, VerbKind};
use crate::parse::normalize::is_time_unit;
use crate::parse::numbers::first_number;
use crate::types::Intent;

/// Spoken magnitude that may carry a fraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub start: usize,
    pub end: usize,
}

/// Hours / minutes / seconds for a timer, or `None` when the words hold no duration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationSlots {
    pub hours: Option<i32>,
    pub minutes: Option<i32>,
    pub seconds: Option<i32>,
}

impl DurationSlots {
    pub fn is_empty(&self) -> bool {
        self.hours.is_none() && self.minutes.is_none() && self.seconds.is_none()
    }

    pub fn total_seconds(&self) -> i64 {
        i64::from(self.hours.unwrap_or(0)) * 3600 + i64::from(self.minutes.unwrap_or(0)) * 60 + i64::from(self.seconds.unwrap_or(0))
    }
}

/// Relative climate step in degrees (positive = warmer).
pub fn temperature_step(tokens: &[String]) -> Option<f64> {
    let dir = relative_temp_direction(tokens)?;
    let magnitude = first_quantity(tokens).map(|q| q.value).or_else(|| first_number(tokens).map(f64::from)).unwrap_or(1.0);
    if magnitude <= 0.0 {
        return None;
    }
    Some(magnitude * dir)
}

/// Warmer / cooler / too cold / too warm markers.
pub fn relative_temp_direction(tokens: &[String]) -> Option<f64> {
    if too_cold(tokens) || warmer(tokens) {
        return Some(1.0);
    }
    if too_warm(tokens) || cooler(tokens) {
        return Some(-1.0);
    }
    None
}

pub fn is_relative_temp(tokens: &[String]) -> bool {
    relative_temp_direction(tokens).is_some()
}

fn warmer(tokens: &[String]) -> bool {
    tokens.iter().any(|t| matches!(t.as_str(), "warmer" | "waermer" | "wärmer"))
}

fn cooler(tokens: &[String]) -> bool {
    tokens.iter().any(|t| matches!(t.as_str(), "cooler" | "colder" | "kaelter" | "kälter"))
}

fn too_cold(tokens: &[String]) -> bool {
    has_phrase(tokens, &["too", "cold"])
        || has_phrase(tokens, &["zu", "kalt"])
        || has_phrase(tokens, &["zu", "kaelt"])
        || tokens.windows(2).any(|w| w[0] == "zu" && w[1].starts_with("kalt"))
}

fn too_warm(tokens: &[String]) -> bool {
    has_phrase(tokens, &["too", "warm"])
        || has_phrase(tokens, &["too", "hot"])
        || has_phrase(tokens, &["zu", "warm"])
        || has_phrase(tokens, &["zu", "heiss"])
}

fn has_phrase(tokens: &[String], phrase: &[&str]) -> bool {
    tokens.windows(phrase.len()).any(|window| window.iter().map(String::as_str).eq(phrase.iter().copied()))
}

/// First signed / decimal quantity in the token list.
pub fn first_quantity(tokens: &[String]) -> Option<Quantity> {
    extract_quantities(tokens).into_iter().next()
}

pub fn extract_quantities(tokens: &[String]) -> Vec<Quantity> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if let Some((value, width)) = quantity_at(tokens, i) {
            out.push(Quantity { value, start: i, end: i + width });
            i += width;
            continue;
        }
        i += 1;
    }
    out
}

fn quantity_at(tokens: &[String], at: usize) -> Option<(f64, usize)> {
    let token = tokens.get(at)?;
    if let Some(value) = parse_numeric_token(token) {
        return Some((value, 1));
    }
    // "minus 5" / "plus 2"
    if matches!(token.as_str(), "minus" | "plus") {
        let sign = if token == "minus" { -1.0 } else { 1.0 };
        let next = tokens.get(at + 1)?;
        let value = parse_numeric_token(next).or_else(|| crate::parse::numbers::word_number(next).map(f64::from))?;
        return Some((sign * value, 2));
    }
    if let Some(frac) = fraction_word(token) {
        return Some((frac, 1));
    }
    // "one and a half" / "an hour and a half" handled in duration; here "half" alone.
    if token == "half" || token == "halb" {
        return Some((0.5, 1));
    }
    None
}

pub(crate) fn parse_numeric_token(token: &str) -> Option<f64> {
    if token.contains(':') {
        return None;
    }
    let normalized = token.replace(',', ".");
    normalized.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn fraction_word(token: &str) -> Option<f64> {
    match token {
        "half" | "halb" | "halbe" | "halbes" | "halben" => Some(0.5),
        "quarter" | "viertel" => Some(0.25),
        "eineinhalb" | "anderthalb" | "einhalb" => Some(1.5),
        "dreiviertel" => Some(0.75),
        _ => None,
    }
}

/// Format a quantity for an intent slot without trailing `.0`.
pub fn format_quantity(value: f64) -> String {
    if !value.is_finite() {
        return "0".into();
    }
    if (value - value.round()).abs() < 1e-9 {
        return format!("{}", value.round() as i64);
    }
    let text = format!("{value:.4}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Absolute temperature spoken for a set-point, or `None` when the words are relative.
pub fn absolute_temperature(tokens: &[String]) -> Option<f64> {
    if is_relative_temp(tokens) {
        return None;
    }
    first_quantity(tokens).map(|q| q.value).or_else(|| first_number(tokens).map(f64::from))
}

/// Parse a timer duration from the words. Rejects a zero-length duration.
pub fn extract_duration(tokens: &[String]) -> Option<DurationSlots> {
    if let Some(slots) = clock_duration(tokens) {
        return validate_duration(slots);
    }
    if let Some(slots) = fraction_duration(tokens) {
        return validate_duration(slots);
    }
    if let Some(slots) = compound_duration(tokens) {
        return validate_duration(slots);
    }
    None
}

fn validate_duration(slots: DurationSlots) -> Option<DurationSlots> {
    if slots.is_empty() || slots.total_seconds() <= 0 {
        None
    } else {
        Some(slots)
    }
}

/// `1:30` → 1 hour 30 minutes when it sits next to timer words or alone as a duration.
fn clock_duration(tokens: &[String]) -> Option<DurationSlots> {
    for token in tokens {
        let Some((left, right)) = token.split_once(':') else {
            continue;
        };
        let hours: i32 = left.parse().ok()?;
        let minutes: i32 = right.parse().ok()?;
        if !(0..=23).contains(&hours) || !(0..60).contains(&minutes) {
            continue;
        }
        if hours == 0 && minutes == 0 {
            continue;
        }
        return Some(DurationSlots { hours: (hours > 0).then_some(hours), minutes: (minutes > 0).then_some(minutes), seconds: None });
    }
    None
}

fn fraction_duration(tokens: &[String]) -> Option<DurationSlots> {
    // "half an hour" / "a quarter of an hour" / "eineinhalb Stunden"
    // Articles may already be stripped as fillers ("hour and half").
    if has_phrase(tokens, &["half", "an", "hour"])
        || has_phrase(tokens, &["half", "hour"])
        || has_phrase(tokens, &["halbe", "stunde"])
        || has_phrase(tokens, &["halb", "stunde"])
    {
        return Some(DurationSlots { hours: None, minutes: Some(30), seconds: None });
    }
    if has_phrase(tokens, &["quarter", "of", "an", "hour"])
        || has_phrase(tokens, &["quarter", "of", "hour"])
        || has_phrase(tokens, &["quarter", "an", "hour"])
        || has_phrase(tokens, &["quarter", "hour"])
        || has_phrase(tokens, &["a", "quarter", "hour"])
        || has_phrase(tokens, &["viertel", "stunde"])
        || has_phrase(tokens, &["viertelstunde"])
    {
        return Some(DurationSlots { hours: None, minutes: Some(15), seconds: None });
    }
    // "an hour and a half" / "one and a half hours" / "eineinhalb Stunden"
    if has_phrase(tokens, &["hour", "and", "a", "half"])
        || has_phrase(tokens, &["hours", "and", "a", "half"])
        || has_phrase(tokens, &["hour", "and", "half"])
        || has_phrase(tokens, &["hours", "and", "half"])
        || has_phrase(tokens, &["and", "a", "half", "hours"])
        || has_phrase(tokens, &["and", "a", "half", "hour"])
        || has_phrase(tokens, &["and", "half", "hours"])
        || has_phrase(tokens, &["and", "half", "hour"])
        || (tokens.iter().any(|t| matches!(t.as_str(), "eineinhalb" | "anderthalb"))
            && tokens.iter().any(|t| is_hour_word(t) || is_time_unit(t)))
    {
        return Some(DurationSlots { hours: Some(1), minutes: Some(30), seconds: None });
    }
    if let Some(q) = first_quantity(tokens).filter(|q| (q.value - 1.5).abs() < 1e-9) {
        if tokens[q.end..].iter().any(|t| is_hour_word(t)) {
            return Some(DurationSlots { hours: Some(1), minutes: Some(30), seconds: None });
        }
    }
    None
}

fn is_hour_word(token: &str) -> bool {
    matches!(token, "hour" | "hours" | "hrs" | "stunde" | "stunden") || catalog().hours().contains(token)
}

fn is_minute_word(token: &str) -> bool {
    matches!(token, "minute" | "minutes" | "min" | "mins" | "minuten") || catalog().minutes().contains(token)
}

fn is_second_word(token: &str) -> bool {
    matches!(token, "second" | "seconds" | "sec" | "secs" | "sekunde" | "sekunden") || catalog().seconds().contains(token)
}

/// "2 Stunden 30 Minuten" / "two hours and thirty minutes".
/// Each unit takes the composed number that stands just before it ("one hundred twenty seconds").
fn compound_duration(tokens: &[String]) -> Option<DurationSlots> {
    let mut hours = 0i32;
    let mut minutes = 0i32;
    let mut seconds = 0i32;
    let mut saw_unit = false;
    let mut cursor = 0;
    for unit_at in 0..tokens.len() {
        let unit = &tokens[unit_at];
        let kind = if is_hour_word(unit) {
            "hours"
        } else if is_minute_word(unit) {
            "minutes"
        } else if is_second_word(unit) {
            "seconds"
        } else {
            continue;
        };
        let span = &tokens[cursor..unit_at];
        let value = first_quantity(span).map(|q| q.value).or_else(|| first_number(span).map(f64::from)).filter(|v| *v > 0.0);
        let Some(value) = value else {
            cursor = unit_at + 1;
            continue;
        };
        match kind {
            "hours" if value.fract().abs() > 1e-9 => {
                let total_minutes = (value * 60.0).round() as i32;
                hours += total_minutes / 60;
                minutes += total_minutes % 60;
            }
            "hours" => hours += value.round() as i32,
            "minutes" => minutes += value.round() as i32,
            "seconds" => seconds += value.round() as i32,
            _ => {}
        }
        saw_unit = true;
        cursor = unit_at + 1;
    }
    if !saw_unit {
        // Bare number next to a timer noun → minutes (existing default).
        let n = first_number(tokens).filter(|n| *n > 0)?;
        if catalog().any(tokens, catalog().timer_nouns()) || tokens.iter().any(|t| matches!(t.as_str(), "timer" | "timers")) {
            return Some(DurationSlots { hours: None, minutes: Some(n), seconds: None });
        }
        return None;
    }
    Some(DurationSlots {
        hours: (hours > 0).then_some(hours),
        minutes: (minutes > 0).then_some(minutes),
        seconds: (seconds > 0).then_some(seconds),
    })
}

/// Apply duration slots onto a timer intent.
pub fn apply_duration(mut intent: crate::types::Intent, tokens: &[String]) -> crate::types::Intent {
    let Some(duration) = extract_duration(tokens) else {
        return intent;
    };
    intent.slots.retain(|slot| !matches!(slot.name.as_str(), "hours" | "minutes" | "seconds"));
    if let Some(h) = duration.hours {
        intent = intent.with("hours", h.to_string());
    }
    if let Some(m) = duration.minutes {
        intent = intent.with("minutes", m.to_string());
    }
    if let Some(s) = duration.seconds {
        intent = intent.with("seconds", s.to_string());
    }
    intent
}

/// True when both sides of a conjunction carry their own numeric value.
pub fn both_sides_valued(tokens: &[String], conj_at: usize) -> bool {
    // "an hour and a half" / "two hours and thirty minutes" are one duration, not two clauses.
    if duration_conj(tokens, conj_at) {
        return false;
    }
    let left = &tokens[..conj_at];
    let right = &tokens[conj_at + 1..];
    side_has_value(left) && side_has_value(right)
}

fn duration_conj(tokens: &[String], conj_at: usize) -> bool {
    let left = tokens.get(conj_at.wrapping_sub(1)).map(String::as_str);
    let right_tokens = &tokens[conj_at + 1..];
    // "… hour(s) and (a) half"
    if matches!(left, Some("hour" | "hours" | "stunde" | "stunden"))
        && right_tokens.iter().any(|t| matches!(t.as_str(), "half" | "halb" | "halbe"))
    {
        return true;
    }
    // "two hours and thirty minutes"
    if matches!(left, Some("hour" | "hours" | "stunde" | "stunden")) && right_tokens.iter().any(|t| is_minute_word(t) || is_second_word(t))
    {
        return true;
    }
    // "one and a half hours"
    if right_tokens.iter().any(|t| matches!(t.as_str(), "half" | "halb" | "halbe"))
        && right_tokens.iter().any(|t| is_hour_word(t) || is_minute_word(t))
    {
        return true;
    }
    false
}

fn side_has_value(tokens: &[String]) -> bool {
    first_quantity(tokens).is_some()
        || first_number(tokens).is_some()
        || tokens.iter().any(|t| fraction_word(t).is_some() || crate::parse::numbers::word_number(t).is_some())
}

pub(crate) fn timer_unit(tokens: &[String]) -> &'static str {
    if catalog().any(tokens, catalog().hours()) || tokens.iter().any(|token| matches!(token.as_str(), "hour" | "hours" | "hrs")) {
        "hours"
    } else if catalog().any(tokens, catalog().seconds())
        || tokens.iter().any(|token| matches!(token.as_str(), "second" | "seconds" | "sec" | "secs"))
    {
        "seconds"
    } else {
        "minutes"
    }
}

pub(crate) fn start_timer_missing_duration(intent: &Intent) -> bool {
    intent.name == "HassStartTimer" && timer_missing_duration(intent) && !named_timer_target(intent)
}

fn named_timer_target(intent: &Intent) -> bool {
    intent.slot("timer_name").is_some() || intent.slot("entity_id").is_some_and(|id| id.starts_with("timer.") && !id.contains("abstract"))
}

fn timer_missing_duration(intent: &Intent) -> bool {
    intent.slot("hours").is_none() && intent.slot("minutes").is_none() && intent.slot("seconds").is_none()
}

pub(crate) fn needs_timer_duration_prompt(intent: &Intent, tokens: &[String]) -> bool {
    match intent.name.as_str() {
        "HassStartTimer" => start_timer_missing_duration(intent) && !timer_keeps_existing_duration(tokens),
        "HassIncreaseTimer" | "HassDecreaseTimer" => timer_missing_duration(intent),
        _ => false,
    }
}

pub(crate) fn fill_pending_timer_duration(template: &Intent, tokens: &[String]) -> Option<Intent> {
    if !matches!(template.name.as_str(), "HassStartTimer" | "HassIncreaseTimer" | "HassDecreaseTimer") {
        return None;
    }
    if let Some(filled) =
        crate::parse::quantity::extract_duration(tokens).map(|_| crate::parse::quantity::apply_duration(template.clone(), tokens))
    {
        return Some(filled);
    }
    let number = crate::parse::numbers::first_number(tokens).filter(|n| *n > 0)?;
    if !duration_reply(tokens) {
        return None;
    }
    Some(template.clone().with_set(timer_unit(tokens), number.to_string()))
}

fn duration_reply(tokens: &[String]) -> bool {
    let cat = &catalog();
    if cat.any(tokens, cat.hours())
        || cat.any(tokens, cat.minutes())
        || cat.any(tokens, cat.seconds())
        || tokens.iter().any(|token| {
            matches!(
                token.as_str(),
                "hour" | "hours" | "hrs" | "minute" | "minutes" | "min" | "mins" | "second" | "seconds" | "sec" | "secs"
            )
        })
    {
        return true;
    }
    tokens.len() <= 3
        && tokens.iter().all(|token| {
            token.parse::<i32>().is_ok()
                || matches!(token.as_str(), "um" | "fuer" | "for" | "by" | "eine" | "einen" | "ein" | "a" | "an")
                || cat.number(token).is_some()
        })
}

fn timer_keeps_existing_duration(tokens: &[String]) -> bool {
    let cat = &catalog();
    cat.any(tokens, cat.playback_resume())
        || cat.any(tokens, cat.timer_pause())
        || cat.any(tokens, cat.timer_cancel())
        || cat.any(tokens, cat.timer_add())
        || tokens.iter().any(|token| {
            matches!(cat.verb(token), Some(VerbKind::Play | VerbKind::Pause))
                || matches!(token.as_str(), "hold" | "halt" | "unpause" | "resume" | "restart")
        })
}

#[cfg(test)]
#[path = "quantity_tests.rs"]
mod tests;
