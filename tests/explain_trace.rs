//! Package 6.4: every full-home catalog case exposes the action trigger word in the trace.

use klar_nlu::nlu;
use klar_nlu::session::Session;
use klar_nlu::types::{ParseDecision, Settings};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

mod voice_suite_support;

use voice_suite_support::suite_home;

#[derive(Debug, Deserialize)]
struct CatalogCase {
    name: String,
    #[serde(default)]
    sentences: Vec<String>,
}

fn quick_cases(lang: &str) -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/datasets/full_home/{lang}/quick"));
    let mut out = Vec::new();
    for entry in fs::read_dir(&root).unwrap_or_else(|err| panic!("read {root:?}: {err}")) {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {path:?}: {err}"));
        let cases: Vec<CatalogCase> = serde_yaml::from_str(&raw).unwrap_or_else(|err| panic!("parse {path:?}: {err}"));
        for case in cases {
            for sentence in case.sentences {
                out.push((format!("{}::{}", case.name, sentence), sentence));
            }
        }
    }
    assert!(!out.is_empty(), "no quick catalog cases for {lang}");
    out
}

#[test]
fn full_home_quick_de_trace_shows_trigger_word() {
    assert_catalog_triggers("de");
}

#[test]
fn full_home_quick_en_trace_shows_trigger_word() {
    assert_catalog_triggers("en");
}

fn assert_catalog_triggers(lang: &str) {
    let home = suite_home(&format!("full_home/{lang}"));
    let settings = Settings::pinned(lang);
    let mut missing = Vec::new();
    for (label, sentence) in quick_cases(lang) {
        let mut session = Session::new();
        let outcome = nlu::parse(&sentence, &home, &mut session, &[], &settings);
        if !matches!(outcome.decision, ParseDecision::Execute | ParseDecision::Confirm { .. }) {
            continue;
        }
        let Some(word) = outcome.trace.trigger_word.as_deref().filter(|word| !word.is_empty()) else {
            missing.push(format!("{label}: no trigger_word (decision={})", outcome.decision.type_name()));
            continue;
        };
        let in_tokens = outcome.trace.tokens.iter().any(|token| token == word);
        let in_remaining = outcome.trace.remaining_tokens.iter().any(|token| token == word);
        let in_text = outcome.trace.normalized.contains(word) || sentence.to_lowercase().contains(&word.to_lowercase());
        if !(in_tokens || in_remaining || in_text) {
            missing.push(format!("{label}: trigger_word={word:?} missing from tokens/normalized"));
        }
    }
    assert!(missing.is_empty(), "catalog cases without a speakable trigger word:\n{}", missing.join("\n"));
}

#[test]
fn warum_speaks_trigger_word_from_last_turn() {
    let home = suite_home("full_home/de");
    let settings = Settings::pinned("de");
    let mut session = Session::new();
    let first = nlu::parse("lichter wohnzimmer an", &home, &mut session, &[], &settings);
    assert!(matches!(first.decision, ParseDecision::Execute), "setup execute: {:?}", first.decision);
    assert_eq!(first.trace.trigger_word.as_deref(), Some("an"));
    let explain = nlu::parse("warum", &home, &mut session, &[], &settings);
    assert!(matches!(explain.decision, ParseDecision::Chat), "explain chat: {:?}", explain.decision);
    assert!(explain.speech.contains("an"), "speech should name the trigger: {}", explain.speech);
}

#[test]
fn discarded_clause_reason_is_traced() {
    let home = suite_home("full_home/de");
    let settings = Settings::pinned("de");
    let mut session = Session::new();
    let negated = nlu::parse("schalte das licht nicht an", &home, &mut session, &[], &settings);
    assert!(matches!(negated.decision, ParseDecision::Reject { .. }), "negated reject: {:?}", negated.decision);
    assert!(
        negated.trace.discarded_clauses.iter().any(|row| row.reason == "negated"),
        "expected negated clause reason: {:?}",
        negated.trace.discarded_clauses
    );
    let conditional = nlu::parse("mach das licht an wenn es regnet", &home, &mut session, &[], &settings);
    assert!(matches!(conditional.decision, ParseDecision::Reject { .. }), "conditional reject: {:?}", conditional.decision);
    assert!(
        conditional.trace.discarded_clauses.iter().any(|row| row.reason == "conditional"),
        "expected conditional clause reason: {:?}",
        conditional.trace.discarded_clauses
    );
}

#[test]
fn stage_timings_include_mood_guard() {
    let home = suite_home("full_home/de");
    let settings = Settings::pinned("de");
    let mut session = Session::new();
    let outcome = nlu::parse("lichter wohnzimmer an", &home, &mut session, &[], &settings);
    let names: Vec<&str> = outcome.trace.stages.iter().map(|stage| stage.stage.as_str()).collect();
    for required in ["normalize", "features", "ranking", "safety_decision", "mood_guard", "planning"] {
        assert!(names.contains(&required), "missing stage {required} in {names:?}");
    }
}
