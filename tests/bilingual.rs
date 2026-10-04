use klar_nlu::home::default_home;
use klar_nlu::lang::bind;
use klar_nlu::parse::parse;
use klar_nlu::parse::respond::speak_clarify;
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, HomeGraph, Settings};

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

#[test]
fn empty_languages_last_resort_speech_is_english() {
    let home = default_home();
    let mut session = Session::new();
    let result = parse("zzzznotaword", &home, &mut session, &[], &Settings::default());
    assert!(result.speech.contains("I did not catch that") || result.speech.contains("Do you mean"), "{}", result.speech);
    assert!(!result.speech.contains("Meinst du"), "{}", result.speech);
    assert!(!result.speech.contains("nicht verstanden"), "{}", result.speech);
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
    assert_eq!(names, vec!["HassClimateGetTemperature"], "{speech}");
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

#[test]
fn en_casual_and_special_are_chat() {
    let home = default_home();
    for text in ["Tell a story", "Tell a joke", "How are you"] {
        let mut session = Session::new();
        let result = parse(text, &home, &mut session, &[], &settings("en"));
        assert!(result.intents.is_empty(), "{text}: {:?}", result.intents);
        assert!(result.chat, "{text}: chat fehlt");
    }
    let ood = parse("What is the capital of France", &home, &mut Session::new(), &[], &settings("en"));
    assert!(ood.intents.is_empty(), "{:?}", ood.intents);
    assert!(!ood.chat, "OOD darf nicht chat sein");
    let weather = parse("What's the weather", &home, &mut Session::new(), &[], &settings("en"));
    assert!(weather.chat || !weather.intents.is_empty(), "What's the weather: {}", weather.speech);
    let rain = parse("Will it rain today", &home, &mut Session::new(), &[], &settings("en"));
    assert!(rain.chat || !rain.intents.is_empty(), "Will it rain today: {}", rain.speech);
    assert!(
        weather.intents.is_empty()
            || weather.intents.iter().any(|intent| intent.slot("entity_id").unwrap_or_default().starts_with("weather.")),
        "{:?}",
        weather.intents
    );
    let home_cmd = parse("Turn on the light in the office", &home, &mut Session::new(), &[], &settings("en"));
    assert!(!home_cmd.chat, "{:?}", home_cmd.intents);
}

#[test]
fn en_news_briefing_keeps_yes_on_llm() {
    let home = default_home();
    let mut session = Session::new();
    let light = parse("Turn on the light in the office", &home, &mut session, &[], &settings("en"));
    assert!(!light.intents.is_empty(), "{:?}", light.intents);
    let news = parse("What is the latest news", &home, &mut session, &[], &settings("en"));
    assert!(news.chat && news.briefing, "{}", news.speech);
    assert!(news.speech.contains("news") || news.speech.contains("News"), "{}", news.speech);
    let yes = parse("yes", &home, &mut session, &[], &settings("en"));
    assert!(yes.chat, "yes after news must not replay a device: {:?}", yes.intents);
    assert!(yes.intents.is_empty(), "{:?}", yes.intents);
    let stop = parse("no thanks", &home, &mut session, &[], &settings("en"));
    assert!(!stop.chat, "{}", stop.speech);
    assert!(stop.briefing);
}

fn screenshot_lights() -> Vec<(&'static str, &'static str)> {
    vec![
        ("light.bathroom_closet_a", "Bathroom Closet A Light"),
        ("light.bathroom_closet_b", "Bathroom Closet B Light"),
        ("light.bedroom_ceiling_fan", "Bedroom Ceiling Fan Light"),
        ("light.living_ceiling_fan", "Living Room Ceiling Fan Light"),
        ("light.kitchen_overhead", "Kitchen Overhead Light"),
        ("light.living_bar", "Living Room Bar Light"),
        ("light.living_cat", "Living Room Cat Light"),
    ]
}

fn screenshot_home() -> HomeGraph {
    let mut home = default_home();
    for (id, name) in screenshot_lights() {
        home.entities.push(EntityRec {
            entity_id: id.into(),
            name: name.into(),
            domain: "light".into(),
            platform: None,
            area: Some("wohnzimmer".into()),
            aliases: Vec::new(),
            tags: Vec::new(),
        });
    }
    home
}

#[test]
fn hot_tub_light_clarify_speech_is_english_not_german() {
    let home = screenshot_home();
    let ids: Vec<String> = screenshot_lights().into_iter().map(|(id, _)| id.into()).collect();

    let _empty = bind(&[]);
    let unpinned = speak_clarify(&ids, Some(&home));
    assert!(unpinned.contains("Do you mean"), "{unpinned}");
    assert!(unpinned.contains(" or "), "{unpinned}");
    assert!(unpinned.contains("Bathroom Closet A Light"), "{unpinned}");
    assert!(!unpinned.contains("Meinst du"), "{unpinned}");
    assert!(!unpinned.contains(" oder "), "{unpinned}");
    drop(_empty);

    let _en = bind(&["en".into()]);
    let english = speak_clarify(&ids, Some(&home));
    assert!(english.contains("Do you mean"), "{english}");
    assert!(!english.contains("Meinst du"), "{english}");
    drop(_en);

    let _de = bind(&["de".into()]);
    let german = speak_clarify(&ids, Some(&home));
    assert!(german.contains("Meinst du"), "{german}");
    assert!(german.contains(" oder "), "{german}");

    for langs in [Vec::new(), vec!["en".into()]] {
        let mut session = Session::new();
        let settings = Settings { languages: langs.clone(), ..Settings::default() };
        let result = klar_nlu::nlu::parse("turn on hot tub light", &home, &mut session, &[], &settings);
        assert!(!result.speech.contains("Meinst du"), "{langs:?} {}", result.speech);
        assert!(!result.speech.contains(" oder "), "{langs:?} {}", result.speech);
        assert!(
            matches!(result.decision, klar_nlu::types::ParseDecision::Clarify { .. }),
            "{langs:?} {:?} {}",
            result.decision,
            result.speech
        );
        assert!(result.speech.contains("Do you mean"), "{langs:?} {}", result.speech);
        assert!(result.speech.contains(" or "), "{langs:?} {}", result.speech);
    }
}

#[test]
fn weather_asks_bind_entity_and_day() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "weather.home".into(),
        name: "Home".into(),
        domain: "weather".into(),
        platform: None,
        area: None,
        aliases: Vec::new(),
        tags: Vec::new(),
    });
    let cases = [
        ("Wie ist das Wetter?", "de", "now"),
        ("Wie wird das Wetter?", "de", "today"),
        ("Wie wird das Wetter heute?", "de", "today"),
        ("Wird es heute regnen?", "de", "rain"),
        ("Brauche ich heute einen Schirm?", "de", "umbrella"),
        ("Brauche ich heute einen Regenschirm?", "de", "umbrella"),
        ("Will it rain tomorrow?", "en", "rain"),
        ("Va-t-il pleuvoir ?", "fr", "rain"),
        ("Regent het vandaag?", "nl", "rain"),
        ("Llueve hoy?", "es", "rain"),
        ("雨が降る", "ja", "rain"),
        ("Hoe is het weer?", "nl", "now"),
        ("¿Qué tiempo hace?", "es", "now"),
    ];
    for (text, lang, ask) in cases {
        let result = parse(text, &home, &mut Session::new(), &[], &settings(lang));
        assert_eq!(result.intents.len(), 1, "{text}: {:?} {}", result.intents, result.speech);
        assert_eq!(result.intents[0].name, "HassGetState", "{text}");
        assert_eq!(result.intents[0].slot("entity_id"), Some("weather.home"), "{text}");
        assert_eq!(result.intents[0].slot("weather_ask"), Some(ask), "{text}");
    }
    let monday = parse("Wie wird das Wetter am Montag?", &home, &mut Session::new(), &[], &settings("de"));
    assert_eq!(monday.intents[0].slot("weather_day"), Some("mon"), "{:?}", monday.intents);
    let later = parse("Wird es morgen regnen?", &home, &mut Session::new(), &[], &settings("de"));
    assert_eq!(later.intents[0].slot("weather_ask"), Some("rain"));
    assert_eq!(later.intents[0].slot("weather_day"), Some("tomorrow"));
}
