use klar_nlu::home::default_home;
use klar_nlu::nlu::{parse, parse_with_controls};
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, MatchControl, ParseOutcome, Settings, SpeechBank};

fn ids(outcome: &ParseOutcome) -> Vec<String> {
    outcome.candidates.iter().map(|candidate| candidate.policy.clone()).collect()
}

fn media_home() -> klar_nlu::types::HomeGraph {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "media_player.wohnzimmer_box".into(),
        name: "Wohnzimmer Soundbar".into(),
        domain: "media_player".into(),
        platform: Some("music_assistant".into()),
        area: Some("wohnzimmer".into()),
        aliases: vec!["soundbar".into(), "musik".into()],
        tags: vec!["Musik".into()],
    });
    home
}

#[test]
fn empty_match_controls_match_library_parse() {
    let home = default_home();
    let settings = Settings::pinned("de");
    let empty_bank = SpeechBank::default();
    for text in ["Licht im Wohnzimmer an", "Wohnungstür abschließen"] {
        let library = parse(text, &home, &mut Session::new(), &[], &settings);
        let overlaid = parse_with_controls(text, &home, &mut Session::new(), &[], &settings, &[], &empty_bank, &[]);
        assert_eq!(ids(&library), ids(&overlaid), "{text}");
        assert_eq!(library.decision, overlaid.decision, "{text}");
    }
}

#[test]
fn empty_controls_stay_stable_on_generated_locales() {
    let home = default_home();
    let empty_bank = SpeechBank::default();
    for (language, text) in
        [("en", "turn on the living room light"), ("fr", "allume la lumiere du salon"), ("ja", "リビングの電気をつけて")]
    {
        let settings = Settings { languages: vec![language.into()], ..Settings::default() };
        let library = parse(text, &home, &mut Session::new(), &[], &settings);
        let overlaid = parse_with_controls(text, &home, &mut Session::new(), &[], &settings, &[], &empty_bank, &[]);
        assert_eq!(ids(&library), ids(&overlaid), "{language} {text}");
    }
}

#[test]
fn disabled_media_drops_media_candidates() {
    let home = media_home();
    let settings = Settings::pinned("de");
    let on = parse("Pausiere die Musik", &home, &mut Session::new(), &[], &settings);
    assert!(on.candidates.iter().any(|candidate| candidate.policy == "media"), "{on:#?}");
    let off = parse_with_controls(
        "Pausiere die Musik",
        &home,
        &mut Session::new(),
        &[],
        &settings,
        &[],
        &SpeechBank::default(),
        &[MatchControl { id: "media".into(), enabled: false, precedence: None }],
    );
    assert!(off.candidates.iter().all(|candidate| candidate.policy != "media"), "{off:#?}");
}

#[test]
fn raised_timer_precedence_is_applied() {
    let home = default_home();
    let settings = Settings::pinned("de");
    let controls = [MatchControl { id: "timer".into(), enabled: true, precedence: Some(0) }];
    let outcome =
        parse_with_controls("Timer 5 Minuten", &home, &mut Session::new(), &[], &settings, &[], &SpeechBank::default(), &controls);
    let timer = outcome.candidates.iter().find(|candidate| candidate.policy == "timer");
    if let Some(timer) = timer {
        assert_eq!(timer.precedence, 0);
    }
}
