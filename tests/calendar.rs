use klar_nlu::home::default_home;
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, ParseDecision, Settings};

fn settings(lang: &str) -> Settings {
    Settings { languages: vec![lang.into()], ..Settings::default() }
}

fn outcome(text: &str, lang: &str) -> (ParseDecision, Vec<String>, Vec<(String, String)>) {
    outcome_home(text, lang, default_home())
}

fn outcome_home(text: &str, lang: &str, home: klar_nlu::types::HomeGraph) -> (ParseDecision, Vec<String>, Vec<(String, String)>) {
    let mut session = Session::default();
    let parsed = parse(text, &home, &mut session, &[], &settings(lang));
    let names = parsed.plan.as_ref().map(|plan| plan.intents().into_iter().map(|item| item.name).collect()).unwrap_or_default();
    let slots = parsed
        .plan
        .as_ref()
        .and_then(|plan| plan.intents().into_iter().next())
        .map(|intent| intent.slots.into_iter().map(|slot| (slot.name, slot.value)).collect())
        .unwrap_or_default();
    (parsed.decision, names, slots)
}

fn trap_home() -> klar_nlu::types::HomeGraph {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "switch.create_calendar_event".into(),
        name: "Create Calendar Event".into(),
        domain: "switch".into(),
        platform: None,
        area: Some("wohnung".into()),
        aliases: vec!["create calendar event".into(), "calendar".into(), "event".into()],
        tags: Vec::new(),
    });
    home
}

#[test]
fn en_calendar_list_does_not_bind_create_button() {
    let (decision, names, slots) = outcome_home("what's on my calendar", "en", trap_home());
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarGetCalendarEvents"), "{names:?}");
    assert!(!names.iter().any(|name| name == "HassGetState" || name == "HassTurnOn"), "{names:?}");
    assert!(!slots.iter().any(|(name, value)| name == "entity_id" && value.contains("create_calendar")), "{slots:?}");
}

#[test]
fn en_calendar_create_has_summary_and_when() {
    let (decision, names, slots) = outcome("add dentist tomorrow at 3 to my calendar", "en");
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarCreateCalendarEvent"), "{names:?}");
    assert!(slots.iter().any(|(name, value)| name == "summary" && value.contains("dentist")), "{slots:?}");
    assert!(slots.iter().any(|(name, value)| name == "day" && value == "tomorrow"), "{slots:?}");
    assert!(slots.iter().any(|(name, value)| name == "hour" && value == "3"), "{slots:?}");
}

#[test]
fn de_calendar_query_without_noun() {
    let (decision, names, _) = outcome("was steht an", "de");
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarGetCalendarEvents"), "{names:?}");
}

#[test]
fn de_calendar_create_native() {
    let (decision, names, slots) = outcome("termin zahnarzt morgen um 15 uhr", "de");
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarCreateCalendarEvent"), "{names:?}");
    assert!(slots.iter().any(|(name, value)| name == "summary" && value.contains("zahnarzt")), "{slots:?}");
}

#[test]
fn create_without_title_clarifies() {
    let (decision, _, _) = outcome("add to my calendar tomorrow at 3", "en");
    assert!(matches!(decision, ParseDecision::Clarify { .. }), "{decision:?}");
}

#[test]
fn create_without_when_clarifies() {
    let (decision, _, _) = outcome("add dentist to my calendar", "en");
    assert!(matches!(decision, ParseDecision::Clarify { .. }), "{decision:?}");
}

#[test]
fn en_calendar_delete_has_summary() {
    let (decision, names, slots) = outcome("delete dentist calendar", "en");
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarDeleteCalendarEvent"), "{names:?}");
    assert!(slots.iter().any(|(name, value)| name == "summary" && value.contains("dentist")), "{slots:?}");
}

#[test]
fn de_calendar_delete_native() {
    let (decision, names, slots) = outcome("loesch zahnarzt kalender", "de");
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarDeleteCalendarEvent"), "{names:?}");
    assert!(slots.iter().any(|(name, value)| name == "summary" && value.contains("zahnarzt")), "{slots:?}");
}

#[test]
fn en_calendar_move_has_when() {
    let (decision, names, slots) = outcome("move dentist tomorrow at 4 calendar", "en");
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarMoveCalendarEvent"), "{names:?}");
    assert!(slots.iter().any(|(name, value)| name == "summary" && value.contains("dentist")), "{slots:?}");
    assert!(slots.iter().any(|(name, value)| name == "day" && value == "tomorrow"), "{slots:?}");
}

#[test]
fn move_without_when_clarifies() {
    let (decision, _, _) = outcome("move dentist calendar", "en");
    assert!(matches!(decision, ParseDecision::Clarify { .. }), "{decision:?}");
}

#[test]
fn delete_calendar_event_switch_is_unbound() {
    let mut home = trap_home();
    home.entities.push(EntityRec {
        entity_id: "switch.delete_calendar_event".into(),
        name: "Delete Calendar Event".into(),
        domain: "switch".into(),
        platform: None,
        area: Some("wohnung".into()),
        aliases: vec!["delete calendar event".into()],
        tags: Vec::new(),
    });
    let (decision, names, slots) = outcome_home("delete dentist calendar", "en", home);
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarDeleteCalendarEvent"), "{names:?}");
    assert!(!slots.iter().any(|(name, value)| name == "entity_id" && value.contains("delete_calendar")), "{slots:?}");
}

#[test]
fn nlu_ignore_keeps_create_button_off_the_plan() {
    let mut home = trap_home();
    if let Some(entity) = home.entities.iter_mut().find(|item| item.entity_id == "switch.create_calendar_event") {
        entity.tags.push("nlu_ignore".into());
    }
    let (decision, names, slots) = outcome_home("what's on my calendar", "en", home);
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarGetCalendarEvents"), "{names:?}");
    assert!(!slots.iter().any(|(name, _value)| name == "entity_id"), "{slots:?}");
}

#[test]
fn delete_anaphora_executes_without_summary() {
    let (decision, names, slots) = outcome("delete that calendar", "en");
    assert!(matches!(decision, ParseDecision::Execute), "{decision:?}");
    assert!(names.iter().any(|name| name == "KlarDeleteCalendarEvent"), "{names:?}");
    assert!(!slots.iter().any(|(name, _)| name == "summary"), "{slots:?}");
}

#[test]
fn bare_delete_is_not_calendar() {
    let (_, names, _) = outcome("delete", "en");
    assert!(!names.iter().any(|name| name.contains("Calendar")), "{names:?}");
}

#[test]
fn family_script_list_smokes() {
    for (lang, text) in [
        ("fr", "quels sont mes rendez-vous"),
        ("es", "que hay en el calendario"),
        ("ja", "nani yotei"),
        ("zh-CN", "rili you shenme"),
        ("ar", "ما في التقويم"),
        ("he", "מה ביומן"),
        ("hi", "calendar me kya hai"),
    ] {
        let (decision, names, _) = outcome(text, lang);
        assert!(
            matches!(decision, ParseDecision::Execute) && names.iter().any(|name| name == "KlarGetCalendarEvents"),
            "{lang} {text} {decision:?} {names:?}"
        );
    }
}

#[test]
fn family_script_delete_smokes() {
    for (lang, text) in [("fr", "supprime dentiste calendrier"), ("ja", "sakujo haisha karendaa"), ("ar", "احذف طبيب تقويم")] {
        let (decision, names, _) = outcome(text, lang);
        assert!(
            matches!(decision, ParseDecision::Execute) && names.iter().any(|name| name == "KlarDeleteCalendarEvent"),
            "{lang} {text} {decision:?} {names:?}"
        );
    }
}
