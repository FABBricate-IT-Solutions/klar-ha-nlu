//! Reminder phrases must not become a light or a timer.

use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Settings};

fn run(sentence: &str) -> klar_nlu::types::ParseResult {
    parse(sentence, &HomeGraph::default(), &mut Session::new(), &[], &Settings::pinned("en"))
}

#[test]
fn remind_me_in_twenty_minutes_sets_a_relative_reminder() {
    let result = run("remind me in 20 minutes to feed the cats");
    assert_eq!(result.intents.len(), 1, "{result:?}");
    let intent = &result.intents[0];
    assert_eq!(intent.name, "SetReminderRelative");
    assert_eq!(intent.slot("count"), Some("20"));
    assert_eq!(intent.slot("unit"), Some("minutes"));
    assert_eq!(intent.slot("task"), Some("feed the cats"));
    assert_eq!(result.speech, "Reminder: feed the cats.");
    assert!(result.intents.iter().all(|intent| !intent.name.starts_with("HassLight") && intent.name != "HassTurnOn"));
}

#[test]
fn remind_me_tomorrow_morning_is_not_a_device_command() {
    let result = run("remind me tomorrow morning to call the bank");
    assert!(result.intents.is_empty(), "{result:?}");
}
