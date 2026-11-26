use super::*;
use crate::home::sample::default_home;
use crate::home::test_dir::TestDir;
use crate::home::LoadedHome;
use crate::io::conversations::ConversationTurn;
use crate::io::state::AppState;
use crate::io::web;
use crate::types::Settings;
use axum::body::{to_bytes, Body};
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use serde_json::json;
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

#[test]
fn redacts_home_and_slots_together() {
    let home = default_home();
    let (intended, snippet) = redact_case(&kitchen_on(), &home);
    assert_eq!(intended.slots.get("entity_id").map(String::as_str), Some("light.e01"));
    assert!(snippet.entities.iter().any(|entity| entity.entity_id == "light.e01"));
    assert!(!snippet.entities.iter().any(|entity| entity.entity_id.contains("kuche")));
    assert!(snippet.areas.iter().all(|area| area.area_id.starts_with('a')));
}

#[test]
fn dataset_uses_nlu_expect_not_conditions() {
    let home = default_home();
    let (intended, snippet) = redact_case(&kitchen_on(), &home);
    let case = WrongCase {
        id: "1".into(),
        ts_ms: 1,
        conversation_id: "c".into(),
        turn_ts_ms: 1,
        text: "Küche Licht an".into(),
        language: Some("de".into()),
        intended,
        home: snippet,
    };
    let yaml = export_dataset(&[case]);
    assert!(yaml.contains("nlu_expect:"), "{yaml}");
    assert!(yaml.contains("HassTurnOn"), "{yaml}");
    assert!(yaml.contains("light.e01"), "{yaml}");
    assert!(!yaml.contains("type: action"), "{yaml}");
    assert!(!yaml.contains("conditions:"), "{yaml}");
}

#[test]
fn reported_mismatch_is_a_red_gate() {
    let home = default_home();
    // Operator wanted TurnOff; the engine still turns the kitchen light on → red without hand edits.
    let intended = IntendedExpect {
        intent: "HassTurnOff".into(),
        slots: BTreeMap::from([("entity_id".into(), "light.kuche_kuche".into())]),
        reject: false,
        clarify: false,
    };
    let (intended, snippet) = redact_case(&intended, &home);
    let case = WrongCase {
        id: "1".into(),
        ts_ms: 1,
        conversation_id: "c".into(),
        turn_ts_ms: 1,
        text: "Küche Licht an".into(),
        language: Some("de".into()),
        intended,
        home: snippet,
    };
    let yaml = export_one(&case);
    let result = evaluate_exported_yaml(&yaml);
    assert!(result.is_err(), "expected red gate, got {result:?}\n{yaml}");
}

#[test]
fn matching_intent_goes_green() {
    let home = default_home();
    let (intended, snippet) = redact_case(&kitchen_on(), &home);
    let case = WrongCase {
        id: "1".into(),
        ts_ms: 1,
        conversation_id: "c".into(),
        turn_ts_ms: 1,
        text: "Küche Licht an".into(),
        language: Some("de".into()),
        intended,
        home: snippet,
    };
    let yaml = export_one(&case);
    evaluate_exported_yaml(&yaml).unwrap_or_else(|err| panic!("should pass once fixed: {err}\n{yaml}"));
}

#[test]
fn store_roundtrip() {
    let scratch = TestDir::new("wrong-cases");
    let store = WrongCaseStore::open(scratch.path());
    let home = default_home();
    let (intended, snippet) = redact_case(&kitchen_on(), &home);
    store.append(WrongCase {
        id: "a".into(),
        ts_ms: 1,
        conversation_id: "c".into(),
        turn_ts_ms: 2,
        text: "an".into(),
        language: None,
        intended,
        home: snippet,
    });
    assert_eq!(store.load().len(), 1);
    assert!(store.dataset_yaml().contains("nlu_expect"));
}

#[tokio::test]
async fn mark_wrong_http_exports_intended_nlu_expect() {
    let scratch = TestDir::new("wrong-http");
    let state = AppState::new(
        LoadedHome {
            graph: default_home(),
            settings: Settings { support_bundle_raw_text: true, ..Settings::default() },
            custom: Vec::new(),
            language: Default::default(),
            policies: Vec::new(),
            speech_bank: Default::default(),
            match_controls: Vec::new(),
        },
        scratch.path().to_path_buf(),
        None,
    );
    state.journal.append(ConversationTurn {
        conversation_id: "c-wrong".into(),
        ts_ms: 42,
        text: Some("Küche Licht an".into()),
        tokens: vec!["kueche".into(), "licht".into(), "an".into()],
        decision: "execute".into(),
        reject_reason: None,
        speech: String::new(),
        speech_source: None,
        refine_band: None,
        confidence: 0.9,
        briefing: false,
        evidence_kinds: Vec::new(),
        last_names: vec!["HassTurnOn".into()],
        confirm_prompt: None,
        candidate_id: None,
        preferred_area: None,
    });

    let mut request = Request::builder()
        .method("POST")
        .uri("/api/v2/conversations/mark-wrong")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "conversation_id": "c-wrong",
                "ts_ms": 42,
                "intent": "HassTurnOff",
                "slots": {"entity_id": "light.kuche_kuche"}
            }))
            .unwrap(),
        ))
        .unwrap();
    request.extensions_mut().insert(ConnectInfo("127.0.0.1:9".parse::<SocketAddr>().unwrap()));
    let response = web::router(state.clone()).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let yaml = parsed["case_yaml"].as_str().expect("case_yaml");
    assert!(yaml.contains("nlu_expect"), "{yaml}");
    assert!(yaml.contains("HassTurnOff"), "{yaml}");
    assert!(yaml.contains("home:"), "{yaml}");
    assert!(!yaml.contains("light.kuche_kuche"), "{yaml}");
    assert!(evaluate_exported_yaml(yaml).is_err(), "reported error must stay red\n{yaml}");

    let mut dataset_req = Request::builder().method("GET").uri("/api/bundle/dataset").body(Body::empty()).unwrap();
    dataset_req.extensions_mut().insert(ConnectInfo("127.0.0.1:9".parse::<SocketAddr>().unwrap()));
    let dataset = web::router(state).oneshot(dataset_req).await.unwrap();
    assert_eq!(dataset.status(), StatusCode::OK);
    let dataset_body = String::from_utf8(to_bytes(dataset.into_body(), 1024 * 1024).await.unwrap().to_vec()).unwrap();
    assert!(dataset_body.contains("nlu_expect") && dataset_body.contains("HassTurnOff"), "{dataset_body}");
}
