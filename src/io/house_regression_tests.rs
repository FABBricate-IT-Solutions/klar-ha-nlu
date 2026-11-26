use super::*;
use crate::home::sample::default_home;
use crate::home::test_dir::TestDir;
use crate::home::LoadedHome;
use crate::io::state::AppState;
use crate::io::web;
use crate::io::wrong_case::{redact_case, IntendedExpect, WrongCase, WrongCaseStore};
use crate::types::{PolicyEffect, PolicyMatch, PolicyRule, Settings};
use axum::body::{to_bytes, Body};
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use serde_json::json;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use tower::ServiceExt;

fn kitchen_on() -> IntendedExpect {
    IntendedExpect {
        intent: "HassTurnOn".into(),
        slots: BTreeMap::from([("entity_id".into(), "light.kuche_kuche".into())]),
        reject: false,
        clarify: false,
    }
}

fn case_kitchen(id: &str, text: &str) -> WrongCase {
    let home = default_home();
    let (intended, snippet) = redact_case(&kitchen_on(), &home);
    WrongCase {
        id: id.into(),
        ts_ms: 1,
        conversation_id: "c".into(),
        turn_ts_ms: 1,
        text: text.into(),
        language: Some("de".into()),
        intended,
        home: snippet,
    }
}

#[test]
fn dry_run_reports_saved_cases() {
    let settings = Settings { languages: vec!["de".into()], ..Settings::default() };
    let bank = SpeechBank::default();
    let case = case_kitchen("a", "Küche Licht an");
    let report =
        dry_run_cases(&[case], &RunConfig { settings: &settings, house: &[], match_controls: &[], custom: &[], speech_bank: &bank });
    assert_eq!(report.total, 1);
    assert_eq!(report.cases[0].text, "Küche Licht an");
    assert!(!report.cases[0].fingerprint.is_empty());
}

#[test]
fn trainer_impact_lists_sentences_when_behaviour_changes() {
    let settings = Settings { languages: vec!["de".into()], ..Settings::default() };
    let bank = SpeechBank::default();
    let case = case_kitchen("a", "Küche Licht an");
    let before = RunConfig { settings: &settings, house: &[], match_controls: &[], custom: &[], speech_bank: &bank };
    // Domain block flips light commands away from execute.
    let house = vec![PolicyRule {
        id: "block-lights".into(),
        enabled: true,
        label: "block".into(),
        when: PolicyMatch { domain: Some("light".into()), ..PolicyMatch::default() },
        effect: PolicyEffect::Block,
        prefer: None,
        payload: None,
    }];
    let after = RunConfig { settings: &settings, house: &house, match_controls: &[], custom: &[], speech_bank: &bank };
    let impact = trainer_impact(std::slice::from_ref(&case), &before, &after);
    assert_eq!(impact.total, 1);
    assert_eq!(impact.changed, 1, "{impact:?}");
    assert_eq!(impact.cases[0].text, "Küche Licht an");
    assert_ne!(impact.cases[0].before, impact.cases[0].after);
}

#[test]
fn update_dialog_lists_affected_sentences_after_engine_bump() {
    let scratch = TestDir::new("regress-update");
    let settings = Settings { languages: vec!["de".into()], ..Settings::default() };
    let bank = SpeechBank::default();
    let case = case_kitchen("a", "Küche Licht an");
    let cfg = RunConfig { settings: &settings, house: &[], match_controls: &[], custom: &[], speech_bank: &bank };
    let current = run_fingerprint(&case, &cfg);
    // Pretend the previous engine answered differently for this sentence.
    write_baseline_for_test(scratch.path(), "0.0.0-old", BTreeMap::from([("a".into(), "reject".into())]));
    let dialog = update_dialog(scratch.path(), &[case], &cfg);
    assert!(dialog.is_update);
    assert_eq!(dialog.changed, 1);
    assert_eq!(dialog.cases[0].text, "Küche Licht an");
    assert_eq!(dialog.cases[0].before, "reject");
    assert_eq!(dialog.cases[0].after, current);
    assert_eq!(dialog.baseline_version.as_deref(), Some("0.0.0-old"));
}

#[test]
fn update_dialog_seeds_quietly_when_no_baseline() {
    let scratch = TestDir::new("regress-seed");
    let settings = Settings { languages: vec!["de".into()], ..Settings::default() };
    let bank = SpeechBank::default();
    let case = case_kitchen("a", "Küche Licht an");
    let dialog = update_dialog(
        scratch.path(),
        &[case],
        &RunConfig { settings: &settings, house: &[], match_controls: &[], custom: &[], speech_bank: &bank },
    );
    assert!(!dialog.is_update);
    assert_eq!(dialog.changed, 0);
    assert!(read_baseline(scratch.path()).is_some());
}

#[tokio::test]
async fn update_dialog_http_surfaces_changed_sentences() {
    let scratch = TestDir::new("regress-http");
    let store = WrongCaseStore::open(scratch.path());
    let case = case_kitchen("case-1", "Küche Licht an");
    let settings = Settings { languages: vec!["de".into()], ..Settings::default() };
    let bank = SpeechBank::default();
    let fp = run_fingerprint(&case, &RunConfig { settings: &settings, house: &[], match_controls: &[], custom: &[], speech_bank: &bank });
    store.append(case);
    write_baseline_for_test(
        scratch.path(),
        "0.0.0-old",
        BTreeMap::from([("case-1".into(), "execute|HassTurnOff[entity_id=light.e01]".into())]),
    );

    let state = AppState::new(
        LoadedHome {
            graph: default_home(),
            settings,
            custom: Vec::new(),
            language: Default::default(),
            policies: Vec::new(),
            speech_bank: Default::default(),
            match_controls: Vec::new(),
        },
        scratch.path().to_path_buf(),
        None,
    );

    let mut request = Request::builder().method("GET").uri("/api/v2/wrong-cases/update-dialog").body(Body::empty()).unwrap();
    request.extensions_mut().insert(ConnectInfo("127.0.0.1:9".parse::<SocketAddr>().unwrap()));
    let response = web::router(state.clone()).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert_eq!(body["is_update"], true);
    assert_eq!(body["changed"], 1);
    assert_eq!(body["cases"][0]["text"], "Küche Licht an");
    assert_eq!(body["cases"][0]["after"], fp);

    let mut ack = Request::builder().method("POST").uri("/api/v2/wrong-cases/update-dialog").body(Body::empty()).unwrap();
    ack.extensions_mut().insert(ConnectInfo("127.0.0.1:9".parse::<SocketAddr>().unwrap()));
    let ack_response = web::router(state).oneshot(ack).await.unwrap();
    assert_eq!(ack_response.status(), StatusCode::OK);
    let ack_body: serde_json::Value = serde_json::from_slice(&to_bytes(ack_response.into_body(), 64 * 1024).await.unwrap()).unwrap();
    assert_eq!(ack_body["changed"], 0);
    assert_eq!(ack_body["is_update"], false);
}

#[tokio::test]
async fn validate_proposal_includes_regression_sentences() {
    let scratch = TestDir::new("regress-validate");
    let store = WrongCaseStore::open(scratch.path());
    store.append(case_kitchen("v1", "Küche Licht an"));

    let state = AppState::new(
        LoadedHome {
            graph: default_home(),
            settings: Settings { languages: vec!["de".into()], ..Settings::default() },
            custom: Vec::new(),
            language: Default::default(),
            policies: Vec::new(),
            speech_bank: Default::default(),
            match_controls: Vec::new(),
        },
        scratch.path().to_path_buf(),
        None,
    );

    let body = json!({
        "layer": "house",
        "language": "de",
        "policies": [{
            "id": "block-lights",
            "enabled": true,
            "label": "block",
            "when": { "domain": "light" },
            "effect": "block"
        }]
    });
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/v2/policies/propose/validate")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    request.extensions_mut().insert(ConnectInfo("127.0.0.1:9".parse::<SocketAddr>().unwrap()));
    let response = web::router(state).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert_eq!(parsed["ok"], true, "{parsed}");
    assert_eq!(parsed["regression"]["changed"], 1, "{parsed}");
    assert_eq!(parsed["regression"]["cases"][0]["text"], "Küche Licht an");
}
