//! Spoken reminders stay off the light and timer paths.
//!
//! A phrase Klar can slot is `SetReminderRelative` or `SetReminderAbsolute`,
//! which Home Assistant's reminder intent scripts already understand. Anything
//! else that is clearly a reminder is out of domain so the integration can hand
//! the original sentence to the default Assist agent.

use crate::lang::catalog;
use crate::parse::respond::speak_unknown;
use crate::types::{Intent, RejectReason};

use super::context::ParseContext;
use super::draft::{execute, reject, Draft};

pub(super) fn route(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
    if !reminder_shaped(tokens) {
        return None;
    }
    let Some(intent) = reminder_intent(tokens) else {
        return Some(reject(RejectReason::OutOfDomain, speak_unknown()));
    };
    let task = intent.slot("task").unwrap_or("reminder").to_string();
    let mut draft = execute(context, vec![intent], "reminder", 1.0, 1.0, true, true);
    draft.speech = ack(tokens, &task);
    Some(draft)
}

fn ack(_tokens: &[String], task: &str) -> String {
    catalog().speech().reminder.replace("{task}", task)
}

fn reminder_shaped(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "remind" | "reminder" | "reminders") || token.starts_with("erinner"))
}

fn reminder_intent(tokens: &[String]) -> Option<Intent> {
    if !tokens.iter().any(|token| token == "remind" || token == "reminder") {
        return None;
    }
    // "in 20 minutes at 10 am" is two times. Leave it for the default agent
    // instead of filing the delay and swallowing the clock time into the task.
    if relative_marker(tokens) && absolute_marker(tokens) {
        return None;
    }
    if relative_marker(tokens) {
        return relative_intent(tokens);
    }
    if absolute_marker(tokens) {
        return absolute_intent(tokens);
    }
    None
}

fn relative_marker(tokens: &[String]) -> bool {
    tokens.windows(4).any(|window| window[0] == "in" && window[1] == "half" && window[2] == "an" && window[3] == "hour")
        || tokens.windows(3).any(|window| {
            window[0] == "in"
                && (matches!(window[1].as_str(), "a" | "an")
                    || window[1].parse::<u32>().ok().is_some_and(|count| (1..=600).contains(&count)))
                && unit_name(&window[2]).is_some()
        })
}

fn absolute_marker(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "noon" | "midnight" | "tonight"))
        || (has_day_word(tokens) && has_part_of_day(tokens))
        || tokens.windows(2).any(|window| {
            matches!(window[0].as_str(), "at" | "for")
                && (window[1].parse::<u32>().ok().is_some_and(|hour| (0..=23).contains(&hour))
                    || matches!(window[1].as_str(), "noon" | "midnight"))
        })
}

fn has_day_word(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "today" | "tomorrow"))
}

fn has_part_of_day(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "morning" | "afternoon" | "evening" | "night"))
}

fn relative_intent(tokens: &[String]) -> Option<Intent> {
    let in_at = tokens.iter().position(|token| token == "in")?;
    let (count, unit, time_end) = relative_when(tokens, in_at)?;
    let task = task_text(tokens, in_at, time_end)?;
    Some(Intent::new("SetReminderRelative").with("count", count).with("unit", unit).with("task", task))
}

fn relative_when(tokens: &[String], in_at: usize) -> Option<(String, &'static str, usize)> {
    if tokens.get(in_at + 1).map(String::as_str) == Some("half")
        && tokens.get(in_at + 2).map(String::as_str) == Some("an")
        && tokens.get(in_at + 3).map(String::as_str) == Some("hour")
    {
        return Some(("30".into(), "minutes", in_at + 4));
    }
    let count_token = tokens.get(in_at + 1)?;
    let count = if matches!(count_token.as_str(), "a" | "an") {
        "1".to_string()
    } else {
        let count = count_token.parse::<u32>().ok()?;
        if !(1..=600).contains(&count) {
            return None;
        }
        count.to_string()
    };
    let unit = unit_name(tokens.get(in_at + 2)?)?;
    Some((count, unit, in_at + 3))
}

fn absolute_intent(tokens: &[String]) -> Option<Intent> {
    if let Some(intent) = named_clock_intent(tokens) {
        return Some(intent);
    }
    if let Some(intent) = day_part_intent(tokens) {
        return Some(intent);
    }
    let at = tokens.iter().position(|token| token == "at" || token == "for")?;
    let hour: u32 = tokens.get(at + 1)?.parse().ok()?;
    if !(0..=23).contains(&hour) {
        return None;
    }
    let mut idx = at + 2;
    let mut minute = 0u32;
    if let Some(next) = tokens.get(idx) {
        if let Ok(parsed) = next.parse::<u32>() {
            if parsed <= 59 {
                minute = parsed;
                idx += 1;
            }
        }
    }
    let meridiem = meridiem_at(tokens, &mut idx);
    // "at 5" without am/pm is ambiguous for 1–12; leave it for the default agent (P-33).
    if (1..=12).contains(&hour) && meridiem.is_none() {
        return None;
    }
    let task = task_text(tokens, at, idx)?;
    let mut intent = Intent::new("SetReminderAbsolute").with("hour", hour.to_string()).with("task", task);
    if minute != 0 {
        intent = intent.with("minute", minute.to_string());
    }
    if let Some(meridiem) = meridiem {
        intent = intent.with("meridiem", meridiem);
    }
    if let Some(day) = day_slot(tokens) {
        intent = intent.with("dayref", day);
    }
    Some(intent)
}

fn named_clock_intent(tokens: &[String]) -> Option<Intent> {
    let (hour, meridiem, at) = if let Some(at) = tokens.iter().position(|token| token == "noon") {
        (12u32, Some("pm"), at)
    } else if let Some(at) = tokens.iter().position(|token| token == "midnight") {
        (0u32, Some("am"), at)
    } else if let Some(at) = tokens.iter().position(|token| token == "tonight") {
        (20u32, Some("pm"), at)
    } else {
        return None;
    };
    let time_end = at + 1;
    let time_at = if tokens.get(at.wrapping_sub(1)).map(String::as_str) == Some("at") { at - 1 } else { at };
    let task = task_text(tokens, time_at, time_end)?;
    let mut intent = Intent::new("SetReminderAbsolute").with("hour", hour.to_string()).with("task", task);
    if let Some(meridiem) = meridiem {
        intent = intent.with("meridiem", meridiem);
    }
    if let Some(day) = day_slot(tokens) {
        intent = intent.with("dayref", day);
    }
    Some(intent)
}

fn day_part_intent(tokens: &[String]) -> Option<Intent> {
    let day = day_slot(tokens)?;
    let (hour, meridiem) = if tokens.iter().any(|token| token == "morning") {
        (9u32, "am")
    } else if tokens.iter().any(|token| token == "afternoon") {
        (15u32, "pm")
    } else if tokens.iter().any(|token| token == "evening") {
        (18u32, "pm")
    } else if tokens.iter().any(|token| token == "night") {
        (21u32, "pm")
    } else {
        return None;
    };
    let part_at = tokens.iter().position(|token| matches!(token.as_str(), "morning" | "afternoon" | "evening" | "night"))?;
    let day_at = tokens.iter().position(|token| matches!(token.as_str(), "today" | "tomorrow"))?;
    let time_at = day_at.min(part_at);
    let time_end = day_at.max(part_at) + 1;
    let task = task_text(tokens, time_at, time_end)?;
    Some(
        Intent::new("SetReminderAbsolute").with("hour", hour.to_string()).with("meridiem", meridiem).with("dayref", day).with("task", task),
    )
}

fn day_slot(tokens: &[String]) -> Option<&'static str> {
    if tokens.iter().any(|token| token == "tomorrow") {
        Some("tomorrow")
    } else if tokens.iter().any(|token| token == "today") {
        Some("today")
    } else {
        None
    }
}

fn meridiem_at(tokens: &[String], idx: &mut usize) -> Option<&'static str> {
    let next = tokens.get(*idx)?.as_str();
    let meridiem = if matches!(next, "am" | "pm") {
        *idx += 1;
        next
    } else if matches!(next, "a" | "p") && tokens.get(*idx + 1).map(String::as_str) == Some("m") {
        *idx += 2;
        if next == "a" {
            "am"
        } else {
            "pm"
        }
    } else {
        return None;
    };
    Some(if meridiem == "am" { "am" } else { "pm" })
}

fn task_text(tokens: &[String], time_at: usize, time_end: usize) -> Option<String> {
    let after = tokens.get(time_end..).unwrap_or(&[]);
    let before = tokens.get(..time_at).unwrap_or(&[]);
    let task = if let Some(rest) = strip_connector(after) {
        rest
    } else if let Some(to_at) = before.iter().rposition(|token| token == "to" || token == "that") {
        before[to_at + 1..].join(" ")
    } else {
        return None;
    };
    let task = task.trim();
    if task.is_empty() || task.chars().count() > 240 {
        None
    } else {
        Some(task.to_string())
    }
}

fn strip_connector(tokens: &[String]) -> Option<String> {
    let rest = match tokens.first().map(String::as_str) {
        Some("to" | "that") => &tokens[1..],
        Some(_) => return None,
        None => return None,
    };
    Some(rest.join(" "))
}

fn unit_name(token: &str) -> Option<&'static str> {
    match token {
        "minute" | "minutes" | "min" | "mins" => Some("minutes"),
        "hour" | "hours" | "hr" | "hrs" => Some("hours"),
        "day" | "days" => Some("days"),
        "week" | "weeks" => Some("weeks"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::reminder_intent;

    fn slots(text: &str) -> Vec<(String, String)> {
        let tokens: Vec<String> = text.split_whitespace().map(str::to_string).collect();
        let intent = reminder_intent(&tokens).unwrap_or_else(|| panic!("no reminder intent for {text}"));
        intent.slots.into_iter().map(|slot| (slot.name, slot.value)).collect()
    }

    #[test]
    fn relative_reminder_keeps_the_task_after_to() {
        let got = slots("remind me in 20 minutes to feed the cats");
        assert_eq!(got, vec![("count".into(), "20".into()), ("unit".into(), "minutes".into()), ("task".into(), "feed the cats".into()),]);
    }

    #[test]
    fn task_before_the_delay_still_parses() {
        let got = slots("remind me to feed the cats in 20 minutes");
        assert!(got.iter().any(|(name, value)| name == "task" && value == "feed the cats"));
        assert!(got.iter().any(|(name, value)| name == "count" && value == "20"));
    }

    #[test]
    fn absolute_hour_keeps_meridiem() {
        let tokens: Vec<String> = "remind me at 2 pm to do laundry".split_whitespace().map(str::to_string).collect();
        let intent = reminder_intent(&tokens).expect("absolute");
        assert_eq!(intent.name, "SetReminderAbsolute");
        assert_eq!(intent.slot("hour"), Some("2"));
        assert_eq!(intent.slot("meridiem"), Some("pm"));
        assert_eq!(intent.slot("task"), Some("do laundry"));
    }

    #[test]
    fn both_relative_and_absolute_stay_unparsed() {
        let tokens: Vec<String> = "remind me in 20 minutes at 10 am to feed the cats".split_whitespace().map(str::to_string).collect();
        assert!(reminder_intent(&tokens).is_none());
    }

    #[test]
    fn tomorrow_morning_slots_day_and_hour() {
        let intent =
            reminder_intent(&"remind me tomorrow morning to take out the trash".split_whitespace().map(str::to_string).collect::<Vec<_>>())
                .expect("tomorrow morning");
        assert_eq!(intent.slot("dayref"), Some("tomorrow"));
        assert_eq!(intent.slot("hour"), Some("9"));
        assert_eq!(intent.slot("meridiem"), Some("am"));
        assert_eq!(intent.slot("task"), Some("take out the trash"));
    }

    #[test]
    fn at_noon_is_twelve() {
        let intent = reminder_intent(&"remind me at noon to eat".split_whitespace().map(str::to_string).collect::<Vec<_>>()).expect("noon");
        assert_eq!(intent.slot("hour"), Some("12"));
        assert_eq!(intent.slot("meridiem"), Some("pm"));
        assert_eq!(intent.slot("task"), Some("eat"));
    }

    #[test]
    fn at_five_without_meridiem_stays_unparsed() {
        let tokens: Vec<String> = "remind me at 5 to call mom".split_whitespace().map(str::to_string).collect();
        assert!(reminder_intent(&tokens).is_none());
    }
}
