//! Spoken reminders stay off the light and timer paths.
//!
//! A phrase Klar can slot is `SetReminderRelative` or `SetReminderAbsolute`,
//! which Home Assistant's reminder intent scripts already understand. Anything
//! else that is clearly a reminder is out of domain so the integration can hand
//! the original sentence to the default Assist agent.

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

fn ack(tokens: &[String], task: &str) -> String {
    if tokens.iter().any(|token| token.starts_with("erinner")) {
        format!("Erinnerung: {task}.")
    } else {
        format!("Reminder: {task}.")
    }
}

fn reminder_shaped(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "remind" | "reminder" | "reminders") || token.starts_with("erinner"))
}

fn reminder_intent(tokens: &[String]) -> Option<Intent> {
    if !tokens.iter().any(|token| token == "remind" || token == "reminder") {
        return None;
    }
    let relative = relative_intent(tokens);
    let absolute = absolute_intent(tokens);
    match (relative, absolute) {
        (Some(intent), None) | (None, Some(intent)) => Some(intent),
        _ => None,
    }
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
    let at = tokens.iter().position(|token| token == "at" || token == "for")?;
    let hour: u32 = tokens.get(at + 1)?.parse().ok()?;
    if !(1..=23).contains(&hour) {
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
    let task = task_text(tokens, at, idx)?;
    let mut intent = Intent::new("SetReminderAbsolute").with("hour", hour.to_string()).with("task", task);
    if minute != 0 {
        intent = intent.with("minute", minute.to_string());
    }
    if let Some(meridiem) = meridiem {
        intent = intent.with("meridiem", meridiem);
    }
    Some(intent)
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
}
