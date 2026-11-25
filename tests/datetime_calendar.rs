//! Package 2.5: date, weekday, clock and calendar-time slots (#366 / P-16, P-31, P-33).

use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, HomeGraph, ParseDecision, Settings};

/// 2026-11-23 13:05 Europe/Berlin — a Monday.
const FIXED_UNIX: i64 = 1_795_435_500;

fn settings(lang: &str) -> Settings {
    Settings { time_zone: "Europe/Berlin".into(), clock_unix: Some(FIXED_UNIX), ..Settings::pinned(lang) }
}

fn home_with_weather() -> HomeGraph {
    let mut home = HomeGraph::default();
    home.entities.push(EntityRec {
        entity_id: "weather.home".into(),
        name: "Home".into(),
        domain: "weather".into(),
        platform: None,
        area: None,
        aliases: vec![],
        tags: vec![],
    });
    home
}

fn run(lang: &str, text: &str) -> klar_nlu::types::ParseOutcome {
    parse(text, &home_with_weather(), &mut Session::new(), &[], &settings(lang))
}

#[test]
fn p16_date_and_weekday_routes_beat_calendar() {
    let day = run("en", "what day is it");
    assert!(matches!(day.decision, ParseDecision::Chat), "{day:?}");
    assert!(day.clock_answer, "{day:?}");
    assert_eq!(day.speech, "Today is Monday.");
    assert!(day.plan.is_none());

    let week = run("en", "what day of the week is it");
    assert!(week.clock_answer);
    assert_eq!(week.speech, "Today is Monday.");

    let today = run("en", "what's today");
    assert!(today.clock_answer, "{today:?}");
    assert!(today.speech.contains("Monday"), "{}", today.speech);
    assert!(today.speech.contains("November"), "{}", today.speech);

    let date = run("en", "what's the date");
    assert!(date.clock_answer);
    assert!(date.speech.contains("Monday"), "{}", date.speech);

    let calendar = run("en", "what's on my calendar today");
    assert!(matches!(calendar.decision, ParseDecision::Execute), "{calendar:?}");
    assert!(!calendar.clock_answer);
    let intent = &calendar.plan.as_ref().unwrap().intents()[0];
    assert_eq!(intent.name, "KlarGetCalendarEvents");
    assert_eq!(intent.slot("day"), Some("today"));
}

#[test]
fn p16_german_date_and_weekday() {
    let day = run("de", "welcher Tag ist heute");
    assert!(day.clock_answer, "{day:?}");
    assert_eq!(day.speech, "Heute ist Montag.");

    let date = run("de", "welches Datum haben wir");
    assert!(date.clock_answer, "{date:?}");
    assert!(date.speech.contains("Montag"), "{}", date.speech);

    let agenda = run("de", "was steht heute an");
    assert!(matches!(agenda.decision, ParseDecision::Execute), "{agenda:?}");
    assert_eq!(agenda.plan.as_ref().unwrap().intents()[0].name, "KlarGetCalendarEvents");
}

#[test]
fn p16_generated_locales_speak_fixed_weekday() {
    let cases = [
        ("nl", "welke dag is het", "Monday"),
        ("fr", "quel jour est il", "Monday"),
        ("it", "che giorno e", "Monday"),
        ("de-AT", "welcher Tag ist heute", "Montag"),
        ("en-GB", "what day is it", "Monday"),
    ];
    for (lang, text, want) in cases {
        let day = run(lang, text);
        assert!(day.clock_answer, "{lang}: {day:?}");
        assert!(day.speech.contains(want), "{lang}: {}", day.speech);
    }
}

#[test]
fn p31_snow_on_friday_is_weather_with_day_slot() {
    let result = run("en", "is it going to snow on friday");
    assert!(matches!(result.decision, ParseDecision::Execute), "{result:?}");
    let intent = &result.plan.as_ref().unwrap().intents()[0];
    assert_eq!(intent.slot("domain"), Some("weather"));
    assert_eq!(intent.slot("weather_ask"), Some("snow"));
    assert_eq!(intent.slot("weather_day"), Some("fri"));
}

#[test]
fn p31_weather_place_is_not_home_forecast() {
    let result = run("en", "what's the weather like in paris");
    assert!(
        !result.plan.as_ref().is_some_and(|plan| plan.intents().iter().any(|intent| intent.slot("entity_id") == Some("weather.home"))),
        "{result:?}"
    );
}

#[test]
fn p33_reminder_tomorrow_morning_and_noon() {
    let morning = run("en", "remind me tomorrow morning to take out the trash");
    assert!(matches!(morning.decision, ParseDecision::Execute), "{morning:?}");
    let intent = &morning.plan.as_ref().unwrap().intents()[0];
    assert_eq!(intent.name, "SetReminderAbsolute");
    assert_eq!(intent.slot("dayref"), Some("tomorrow"));
    assert_eq!(intent.slot("hour"), Some("9"));
    assert_eq!(intent.slot("task"), Some("take out the trash"));

    let noon = run("en", "remind me at noon to eat");
    assert!(matches!(noon.decision, ParseDecision::Execute), "{noon:?}");
    assert_eq!(noon.plan.as_ref().unwrap().intents()[0].slot("hour"), Some("12"));

    let bare = run("en", "remind me at 5 to call mom");
    assert!(matches!(bare.decision, ParseDecision::Reject { .. }), "{bare:?}");
}

#[test]
fn clock_answer_is_marked_for_assist() {
    let result = run("en", "what time is it");
    assert!(matches!(result.decision, ParseDecision::Chat));
    assert!(result.clock_answer);
    assert_eq!(result.speech, "It is 13:05.");
}

#[test]
fn calendar_friday_carries_weekday_slot() {
    let result = run("en", "what's on my calendar on friday");
    assert!(matches!(result.decision, ParseDecision::Execute), "{result:?}");
    assert_eq!(result.plan.as_ref().unwrap().intents()[0].slot("day"), Some("fri"));
}
