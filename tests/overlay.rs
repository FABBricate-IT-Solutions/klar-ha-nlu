use klar_nlu::home::default_home;
use klar_nlu::lang::{
    bind_preview_user, catalog_for, install_user_overlay, installed_user_overlay, push_revision, reset_runtime_packs, revision_hash,
    select_revision, validate_custom, validate_language, LanguageOverlay, LanguageRevision, SetDelta, MAX_HISTORY, MAX_USER_INTENTS,
};
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{CustomSentence, ParseDecision, Settings};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

fn lock_overlay() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|err| err.into_inner())
}

fn phrase(phrase: &str, intent: &str) -> CustomSentence {
    CustomSentence { phrase: phrase.into(), intent: intent.into(), slots: HashMap::new() }
}

fn revision(hash: &str, label: &str, text: &str) -> LanguageRevision {
    LanguageRevision {
        hash: hash.into(),
        label: label.into(),
        saved_at: "1".into(),
        custom: vec![phrase(text, "HassTurnOn")],
        language: LanguageOverlay::default(),
    }
}

#[test]
fn rejects_unknown_intent_set_path_length_and_cap() {
    assert!(!validate_custom(&[phrase("filmabend", "NotAnIntent")]).is_empty());
    assert!(!validate_custom(&[phrase("abc", "HassTurnOn")]).is_empty());
    assert!(!validate_custom(&[phrase(&"ä".repeat(201), "HassTurnOn")]).is_empty());
    assert!(validate_custom(&[phrase(&"ä".repeat(200), "HassTurnOn")]).is_empty());
    let too_many = (0..=MAX_USER_INTENTS).map(|index| phrase(&format!("regel {index:02} xx"), "HassTurnOn")).collect::<Vec<_>>();
    assert!(!validate_custom(&too_many).is_empty());
    let language = LanguageOverlay { sets: [("nope.words".into(), SetDelta { add: vec!["x".into()], remove: vec![] })].into() };
    assert!(!validate_language(&language).is_empty());
}

#[test]
fn history_caps_and_skips_duplicate_hash() {
    let mut history = Vec::new();
    let language = LanguageOverlay::default();
    for index in 0..12 {
        push_revision(&mut history, vec![phrase(&format!("phrase {index} xx"), "HassTurnOn")], language.clone(), "save".into());
    }
    assert_eq!(history.len(), MAX_HISTORY);
    let last = history.last().unwrap().custom.clone();
    let hash = revision_hash(&last, &language);
    push_revision(&mut history, last, language, "again".into());
    assert_eq!(history.last().unwrap().hash, hash);
    assert_eq!(history.len(), MAX_HISTORY);
}

#[test]
fn select_revision_defaults_to_latest_and_finds_hash() {
    let older = revision("aaa", "one", "erste regel");
    let newer = revision("bbb", "two", "zweite regel");
    let history = vec![older.clone(), newer.clone()];
    assert_eq!(select_revision(&history, None).unwrap().custom[0].phrase, "zweite regel");
    assert_eq!(select_revision(&history, Some("aaa")).unwrap().custom[0].phrase, "erste regel");
    assert_eq!(select_revision(&history[..1], None).unwrap().hash, "aaa");
    assert!(select_revision(&[], None).is_none());
}

#[test]
fn user_set_delta_reaches_catalog_and_can_be_removed() {
    let _guard = lock_overlay();
    reset_runtime_packs();
    install_user_overlay(Some(LanguageOverlay {
        sets: [("nouns.light_nouns".into(), SetDelta { add: vec!["kugelchen".into()], remove: vec![] })].into(),
    }));
    let catalog = catalog_for(&["de".into()]);
    assert!(catalog.light_nouns().contains("kugelchen"));
    install_user_overlay(Some(LanguageOverlay {
        sets: [("nouns.light_nouns".into(), SetDelta { add: vec![], remove: vec!["kugelchen".into()] })].into(),
    }));
    let catalog = catalog_for(&["de".into()]);
    assert!(!catalog.light_nouns().contains("kugelchen"));
    reset_runtime_packs();
}

#[test]
fn preview_bind_does_not_change_installed_overlay() {
    let _guard = lock_overlay();
    reset_runtime_packs();
    let proposed =
        LanguageOverlay { sets: [("nouns.light_nouns".into(), SetDelta { add: vec!["vorschauwort".into()], remove: vec![] })].into() };
    {
        let _guard = bind_preview_user(Some(proposed));
        assert!(catalog_for(&["de".into()]).light_nouns().contains("vorschauwort"));
        assert!(installed_user_overlay().is_none());
    }
    assert!(!catalog_for(&["de".into()]).light_nouns().contains("vorschauwort"));
    reset_runtime_packs();
}

#[test]
fn preview_custom_phrase_does_not_require_save() {
    let home = default_home();
    let settings = Settings { languages: vec!["de".into()], ..Settings::default() };
    let mut session = Session::default();
    let custom = vec![CustomSentence {
        phrase: "filmabend".into(),
        intent: "HassTurnOn".into(),
        slots: [("entity_id".into(), "light.wohnzimmer".into())].into(),
    }];
    let outcome = parse("filmabend", &home, &mut session, &custom, &settings);
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    let empty = parse("filmabend", &home, &mut Session::default(), &[], &settings);
    let intents = empty.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    assert!(!intents.iter().any(|intent| intent.name == "HassTurnOn" && intent.slots.iter().any(|slot| slot.value == "light.wohnzimmer")));
}
