use klar_nlu::lexicon::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::Settings;

fn settings(lang: &str) -> Settings {
    Settings {
        languages: vec![lang.into()],
        ..Settings::default()
    }
}

fn run(text: &str, lang: &str) -> (Vec<String>, String) {
    let home = default_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &settings(lang));
    (
        result.intents.iter().map(|i| i.name.clone()).collect(),
        result.speech,
    )
}

#[test]
fn de_office_speech_stays_german() {
    let (names, speech) = run("Mach das Licht im Arbeitszimmer an", "de");
    assert_eq!(names, vec!["HassTurnOn"]);
    assert!(speech.contains("Schalte"), "{speech}");
}

#[test]
fn en_office_light_uses_english_speech() {
    let (names, speech) = run("Turn on the light in the office", "en");
    assert_eq!(names, vec!["HassTurnOn"], "{speech}");
    assert!(speech.contains("Turn on"), "{speech}");
    assert!(!speech.contains("Schalte"), "{speech}");
}

#[test]
fn en_bedroom_temperature_uses_english_speech() {
    let (names, speech) = run("What is the temperature in the bedroom", "en");
    assert_eq!(names, vec!["HassGetState"], "{speech}");
    assert!(speech.to_lowercase().contains("temperature"), "{speech}");
    assert!(!speech.contains("Frage"), "{speech}");
}
