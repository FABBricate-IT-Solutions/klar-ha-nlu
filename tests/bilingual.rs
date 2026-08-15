use klar_nlu::lexicon::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::Settings;

fn settings(lang: &str) -> Settings {
    Settings { languages: vec![lang.into()], ..Settings::default() }
}

fn run(text: &str, lang: &str) -> (Vec<String>, String) {
    let home = default_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &settings(lang));
    (result.intents.iter().map(|i| i.name.clone()).collect(), result.speech)
}

#[test]
fn de_office_speech_stays_german() {
    let (names, speech) = run("Mach das Licht im Arbeitszimmer an", "de");
    assert_eq!(names, vec!["HassTurnOn"]);
    assert!(speech.contains("ist an") || speech.contains("Licht"), "{speech}");
}

#[test]
fn en_office_light_uses_english_speech() {
    let (names, speech) = run("Turn on the light in the office", "en");
    assert_eq!(names, vec!["HassTurnOn"], "{speech}");
    assert!(speech.contains("is on") || speech.contains("light"), "{speech}");
    assert!(!speech.contains("ist an"), "{speech}");
}

fn slots(text: &str, lang: &str) -> Vec<(String, Vec<(String, String)>)> {
    let home = default_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &settings(lang));
    result.intents.into_iter().map(|i| (i.name, i.slots.into_iter().map(|s| (s.name, s.value)).collect())).collect()
}

fn area_of(text: &str, lang: &str) -> Vec<String> {
    slots(text, lang).into_iter().flat_map(|(_, slots)| slots.into_iter().filter(|(k, _)| k == "area").map(|(_, v)| v)).collect()
}

fn target_of(text: &str, lang: &str) -> Vec<String> {
    slots(text, lang)
        .into_iter()
        .flat_map(|(_, slots)| slots.into_iter().filter_map(|(k, v)| matches!(k.as_str(), "entity_id" | "area").then_some(v)))
        .collect()
}

#[test]
fn en_bedroom_temperature_uses_english_speech() {
    let (names, speech) = run("What is the temperature in the bedroom", "en");
    assert_eq!(names, vec!["HassGetState"], "{speech}");
    assert!(speech.to_lowercase().contains("temperature"), "{speech}");
    assert!(!speech.contains("Frage"), "{speech}");
}

#[test]
fn en_office_light_targets_arbeitszimmer() {
    let found = target_of("Turn on the office light", "en");
    assert!(found.iter().any(|v| v == "light.arbeitszimmer" || v == "arbeitszimmer"), "{found:?}");
}

#[test]
fn en_study_light_targets_arbeitszimmer_only() {
    let found = target_of("Turn on the light in the study", "en");
    assert!(found.iter().any(|v| v == "light.arbeitszimmer" || v == "arbeitszimmer"), "{found:?}");
}

#[test]
fn en_bedroom_is_schlafzimmer_not_wohnung() {
    let areas = area_of("Turn on the lights in the bedroom", "en");
    assert_eq!(areas, vec!["schlafzimmer"], "{areas:?}");
}

#[test]
fn en_smalltalk_has_no_home_intent() {
    let (names, speech) = run("What is the capital of France", "en");
    assert!(names.is_empty(), "{names:?} {speech}");
}

#[test]
fn en_smalltalk_after_device_has_no_home_intent() {
    let home = default_home();
    let mut session = Session::new();
    let first = parse("Turn on the light in the office", &home, &mut session, &[], &settings("en"));
    assert_eq!(first.intents[0].name, "HassTurnOn", "{}", first.speech);
    for text in ["Tell me a story", "Tell me a joke", "What is the capital of France"] {
        let result = parse(text, &home, &mut session, &[], &settings("en"));
        assert!(!result.clarify, "{text}: {}", result.speech);
        assert!(result.intents.is_empty(), "{text}: {:?} {}", result.intents, result.speech);
    }
}
