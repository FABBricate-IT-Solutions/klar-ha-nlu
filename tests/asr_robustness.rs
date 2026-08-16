use klar_nlu::home::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, HomeGraph, Intent, ParseResult, Settings};

fn run(text: &str, language: &str) -> ParseResult {
    let home = default_home();
    run_in_home(text, language, &home)
}

fn run_in_home(text: &str, language: &str, home: &HomeGraph) -> ParseResult {
    let mut session = Session::new();
    let settings = Settings { languages: vec![language.into()], ..Settings::default() };
    parse(text, home, &mut session, &[], &settings)
}

fn has_target(intents: &[Intent], target: &str) -> bool {
    intents.iter().any(|intent| {
        intent.slot("entity_id").is_some_and(|id| id == target || id.contains(target))
            || intent.slot("area").is_some_and(|area| area == target)
    })
}

#[test]
fn german_recovers_room_syllables_and_glued_names() {
    for (text, target) in [
        ("Licht im Wohnzimer an", "wohnzimmer"),
        ("Licht im Wohnzim an", "wohnzimmer"),
        ("Schlafzimer Lichter aus", "schlafzimmer"),
        ("Wohnzimerlicht an", "wohnzimmer"),
    ] {
        let result = run(text, "de");
        assert!(has_target(&result.intents, target), "{text}: {:?} {}", result.intents, result.speech);
    }
}

#[test]
fn german_recovers_distinctive_device_and_scene_names() {
    let lamp = run("Decknlampe an", "de");
    assert!(has_target(&lamp.intents, "light.schlafzimmer_decke"), "{:?} {}", lamp.intents, lamp.speech);

    let scene = run("Filmabent an", "de");
    assert!(has_target(&scene.intents, "scene.filmabend"), "{:?} {}", scene.intents, scene.speech);
}

#[test]
fn long_action_and_domain_words_need_exact_anchors() {
    let action = run("Aktivire Licht im Wohnzimmer", "de");
    assert_eq!(action.intents.first().map(|intent| intent.name.as_str()), Some("HassTurnOn"), "{:?}", action.intents);
    assert!(has_target(&action.intents, "wohnzimmer"), "{:?}", action.intents);

    let domain = run("Stelle Heizng im Wohnzimmer auf 21 Grad", "de");
    assert_eq!(
        domain.intents.first().map(|intent| intent.name.as_str()),
        Some("HassClimateSetTemperature"),
        "{:?} {}",
        domain.intents,
        domain.speech
    );
}

#[test]
fn english_recovers_long_action_and_room_typo() {
    let result = run("activte the living room light", "en");
    assert_eq!(result.intents.first().map(|intent| intent.name.as_str()), Some("HassTurnOn"), "{:?}", result.intents);
    assert!(has_target(&result.intents, "wohnzimmer"), "{:?}", result.intents);

    let room = run("turn on the bedrom lights", "en");
    assert!(has_target(&room.intents, "schlafzimmer"), "{:?} {}", room.intents, room.speech);
}

#[test]
fn short_and_unknown_tokens_never_bind_targets() {
    let short = run("Kugl an", "de");
    assert!(!has_target(&short.intents, "light.schlafzimmer_kugel"), "{:?}", short.intents);

    let unknown = run("asdfghjkl an", "de");
    assert!(unknown.intents.is_empty(), "{:?} {}", unknown.intents, unknown.speech);
}

#[test]
fn two_fuzzy_repairs_do_not_execute_a_directional_action() {
    for text in ["Aktivire Licht im Wohnzim", "Aktivire Wohnzimer und Esszimmer Licht"] {
        let result = run(text, "de");
        assert!(
            result.intents.iter().all(|intent| !matches!(intent.name.as_str(), "HassTurnOn" | "HassTurnOff")),
            "{text}: {:?} {}",
            result.intents,
            result.speech
        );
    }
}

#[test]
fn short_opposites_numbers_and_colors_stay_exact() {
    let on = run("Licht im Wohnzimmer an", "de");
    assert_eq!(on.intents.first().map(|intent| intent.name.as_str()), Some("HassTurnOn"));
    let off = run("Licht im Wohnzimmer aus", "de");
    assert_eq!(off.intents.first().map(|intent| intent.name.as_str()), Some("HassTurnOff"));

    let color = run("Mach das Licht im Wohnzimmer schwarx", "de");
    assert!(
        color.intents.iter().all(|intent| !matches!(intent.name.as_str(), "HassTurnOn" | "HassTurnOff" | "HassLightSet")),
        "{:?}",
        color.intents
    );

    let timer = run("Timer dreiundzwanzgi Minuten", "de");
    assert!(
        timer.intents.iter().all(|intent| {
            intent.name != "HassStartTimer"
                && intent.slot("seconds").is_none()
                && intent.slot("minutes").is_none()
                && intent.slot("hours").is_none()
        }),
        "{:?}",
        timer.intents
    );
}

#[test]
fn equal_fuzzy_target_names_are_ambiguous() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "light.arbeitszimmer_decke".into(),
        name: "Deckenlampe".into(),
        domain: "light".into(),
        platform: None,
        area: Some("arbeitszimmer".into()),
        aliases: vec!["deckenlampe".into()],
        tags: Vec::new(),
    });
    let result = run_in_home("Decknlampe an", "de", &home);
    assert!(result.clarify, "{:?} {}", result.intents, result.speech);
    assert!(result.intents.is_empty(), "{:?}", result.intents);
}

#[test]
fn protected_value_guard_keeps_exact_home_names() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "switch.schwarx".into(),
        name: "Schwarx".into(),
        domain: "switch".into(),
        platform: None,
        area: Some("arbeitszimmer".into()),
        aliases: vec!["schwarx".into()],
        tags: Vec::new(),
    });
    let result = run_in_home("Schwarx an", "de", &home);
    assert!(has_target(&result.intents, "switch.schwarx"), "{:?} {}", result.intents, result.speech);
}

#[test]
fn protected_value_guard_leaves_list_payload_untouched() {
    let result = run("add oranges to the shopping list", "en");
    assert!(
        result
            .intents
            .iter()
            .any(|intent| { intent.name.contains("ListAdd") && intent.slot("item").is_some_and(|item| item.contains("oranges")) }),
        "{:?} {}",
        result.intents,
        result.speech
    );
}

#[test]
fn fuzzy_media_action_does_not_turn_payload_into_home_target() {
    for text in ["listenn to music by Kitchan", "freezd music in bedrom"] {
        let result = run(text, "en");
        assert!(
            result.intents.iter().all(|intent| {
                !matches!(
                    intent.name.as_str(),
                    "HassTurnOn" | "HassTurnOff" | "HassLightSet" | "HassMediaPause" | "HassMediaNext" | "HassMediaPlayerMute"
                ) && intent.slot("search_query").is_none_or(|query| text.contains("Kitchan") && query.to_lowercase().contains("kitchan"))
            }),
            "{text}: {:?} {}",
            result.intents,
            result.speech
        );
    }
}
