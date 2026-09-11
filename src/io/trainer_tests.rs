use super::*;
use crate::home::default_home;
use crate::lang::{LanguageOverlay, SetDelta};
use std::collections::HashMap;

fn settings() -> Settings {
    Settings::pinned("de")
}

#[test]
fn unknown_match_id_is_rejected() {
    let home = default_home();
    let controls = vec![MatchControl { id: "media_new_matcher".into(), enabled: true, precedence: None }];
    let out =
        validate(&home, &settings(), "de", "match", Vec::new(), controls, LanguageOverlay::default(), &SpeechBank::default(), &[], &[]);
    assert!(!out.ok);
    assert!(out.errors.iter().any(|row| row.path == "match_controls"));
}

#[test]
fn bound_locale_particle_add_is_rejected() {
    let home = default_home();
    let mut sets = HashMap::new();
    sets.insert("nouns.light_nouns".into(), SetDelta { add: vec!["an".into()], remove: Vec::new() });
    let de = validate(
        &home,
        &settings(),
        "de",
        "language",
        Vec::new(),
        Vec::new(),
        LanguageOverlay { sets: sets.clone() },
        &SpeechBank::default(),
        &[],
        &[],
    );
    assert!(!de.ok, "{de:?}");
    let mut ja_sets = HashMap::new();
    ja_sets.insert("nouns.light_nouns".into(), SetDelta { add: vec!["つけて".into()], remove: Vec::new() });
    let ja = validate(
        &home,
        &Settings::pinned("ja"),
        "ja",
        "language",
        Vec::new(),
        Vec::new(),
        LanguageOverlay { sets: ja_sets },
        &SpeechBank::default(),
        &[],
        &[],
    );
    assert!(!ja.ok, "{ja:?}");
}

#[test]
fn missing_entity_is_not_grounded() {
    let home = default_home();
    let rules = vec![PolicyRule {
        id: "ghost".into(),
        enabled: true,
        label: "x".into(),
        when: crate::types::PolicyMatch { entity_id: Some("light.missing".into()), ..crate::types::PolicyMatch::default() },
        effect: PolicyEffect::Block,
        prefer: None,
        payload: None,
    }];
    let out = validate(&home, &settings(), "de", "house", rules, Vec::new(), LanguageOverlay::default(), &SpeechBank::default(), &[], &[]);
    assert!(!out.ok);
    assert!(out.errors.iter().any(|row| row.path.contains("entity_id")));
}

#[test]
fn custom_sentences_run_in_dry_run_and_validate() {
    let home = default_home();
    let custom =
        vec![crate::types::CustomSentence { phrase: "licht wohnzimmer an".into(), intent: "HassTurnOn".into(), slots: Default::default() }];
    let ok = validate(
        &home,
        &settings(),
        "de",
        "language",
        Vec::new(),
        Vec::new(),
        LanguageOverlay::default(),
        &SpeechBank::default(),
        &[],
        &custom,
    );
    assert!(ok.ok, "{ok:?}");
    assert!(ok.dry_run.iter().any(|row| row.text == "licht wohnzimmer an"));
    let bad = vec![crate::types::CustomSentence { phrase: "ok".into(), intent: "HassTurnOn".into(), slots: Default::default() }];
    let out = validate(
        &home,
        &settings(),
        "de",
        "language",
        Vec::new(),
        Vec::new(),
        LanguageOverlay::default(),
        &SpeechBank::default(),
        &[],
        &bad,
    );
    assert!(!out.ok);
}

#[test]
fn context_stub_is_compact() {
    let ctx = TrainerContext {
        language: "de".into(),
        layer: "all".into(),
        prompt_version: "2".into(),
        graph: GraphOut { areas: Vec::new(), floors: Vec::new(), entities: Vec::new() },
        gaps: vec!["light.a".into(), "light.b".into()],
        matches: Vec::new(),
        seeds: Vec::new(),
        overlays: OverlaysOut { policies: Vec::new(), match_controls: Vec::new(), language: LanguageOverlay::default() },
        schema: schema_out(),
    };
    let stub = context_stub(&ctx, &["de".into(), "en".into()]);
    assert!(stub.contains("\"gap_count\":2"));
    assert!(stub.contains("\"languages\":[\"de\",\"en\"]"));
    assert!(stub.contains("\"reply_language\":\"de\""));
    assert!(!stub.contains("light.a"));
    assert!(!stub.contains("graph"));
}
