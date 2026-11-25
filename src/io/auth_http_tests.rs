//! Acceptance for package 3.1: cross-origin writes, teach-as-proposal, route table.
use crate::home::test_dir::TestDir;
use crate::home::{default_home, LoadedHome};
use crate::io::auth::{all_routes, route_capability, Capability};
use crate::io::state::AppState;
use crate::io::web;
use crate::types::Settings;
use axum::body::{to_bytes, Body};
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower::ServiceExt;

fn state(token: Option<&str>) -> (AppState, TestDir) {
    let scratch = TestDir::new("auth-http");
    let state = AppState::new(
        LoadedHome {
            graph: default_home(),
            settings: Settings::default(),
            custom: Vec::new(),
            language: Default::default(),
            policies: Vec::new(),
            speech_bank: Default::default(),
            match_controls: Vec::new(),
        },
        scratch.path().to_path_buf(),
        token.map(str::to_string),
    );
    (state, scratch)
}

async fn call(
    state: AppState,
    peer: &str,
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> axum::response::Response {
    let bytes = body.map(|value| serde_json::to_vec(&value).unwrap()).unwrap_or_default();
    let mut builder = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    if !bytes.is_empty() {
        builder = builder.header("content-type", "application/json");
    }
    let mut request = builder.body(Body::from(bytes)).unwrap();
    request.extensions_mut().insert(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
    web::router(state).oneshot(request).await.unwrap()
}

#[tokio::test]
async fn four_cross_origin_writes_are_forbidden() {
    let (state, _dir) = state(None);
    let routes = [
        (Method::POST, "/api/settings", json!({"personality":"neutral","mode":"assist","languages":["de"]})),
        (Method::POST, "/api/custom", json!([])),
        (Method::POST, "/api/entities", json!({"entity_id":"light.wohnzimmer","aliases":["kugel"]})),
        (Method::POST, "/api/v2/settings/restore", json!({})),
    ];
    for (method, path, body) in routes {
        let response = call(
            state.clone(),
            "127.0.0.1:9",
            method.clone(),
            path,
            &[("origin", "https://evil.example"), ("host", "127.0.0.1:10520")],
            Some(body),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{method} {path}");
    }
}

#[tokio::test]
async fn supervisor_without_token_cannot_save_alias() {
    let (state, _dir) = state(Some("secret"));
    let response = call(
        state.clone(),
        "172.30.32.2:9",
        Method::POST,
        "/api/entities",
        &[("host", "klar-nlu.local.hass.io:10520")],
        Some(json!({"entity_id":"light.wohnzimmer","aliases":["kugel"]})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let home = state.home.snapshot().await;
    let lamp = home.entities.iter().find(|entity| entity.entity_id == "light.wohnzimmer").unwrap();
    assert!(!lamp.aliases.iter().any(|alias| alias == "kugel"));
}

#[tokio::test]
async fn parse_teach_proposes_without_saving() {
    let (state, _dir) = state(Some("secret"));
    let turn_on = call(
        state.clone(),
        "172.30.32.2:9",
        Method::POST,
        "/api/v2/parse",
        &[],
        Some(json!({"text":"wohnzimmer licht an","language":"de","conversation_id":"teach-1"})),
    )
    .await;
    assert_eq!(turn_on.status(), StatusCode::OK);
    let teach = call(
        state.clone(),
        "172.30.32.2:9",
        Method::POST,
        "/api/v2/parse",
        &[],
        Some(json!({"text":"nenn das leselampe","language":"de","conversation_id":"teach-1"})),
    )
    .await;
    assert_eq!(teach.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&to_bytes(teach.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert_eq!(body["teach"]["entity_id"], "light.wohnzimmer");
    assert_eq!(body["teach"]["alias"], "leselampe");
    let home = state.home.snapshot().await;
    let lamp = home.entities.iter().find(|entity| entity.entity_id == "light.wohnzimmer").unwrap();
    assert!(!lamp.aliases.iter().any(|alias| alias == "leselampe"), "D-18: read gate must not persist teach");
}

#[tokio::test]
async fn teach_confirm_with_token_saves_alias() {
    let (state, _dir) = state(Some("secret"));
    let response = call(
        state.clone(),
        "172.30.32.2:9",
        Method::POST,
        "/api/entities",
        &[("x-klar-token", "secret"), ("host", "klar-nlu.local.hass.io:10520")],
        Some(json!({"entity_id":"light.wohnzimmer","aliases":["leselampe"]})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let home = state.home.snapshot().await;
    let lamp = home.entities.iter().find(|entity| entity.entity_id == "light.wohnzimmer").unwrap();
    assert!(lamp.aliases.iter().any(|alias| alias == "leselampe"));
}

#[tokio::test]
async fn propose_validate_rejects_oversized_utterance_list() {
    let (state, _dir) = state(Some("secret"));
    let utterances: Vec<String> = (0..40).map(|i| format!("licht an {i}")).collect();
    let response = call(
        state,
        "127.0.0.1:9",
        Method::POST,
        "/api/v2/policies/propose/validate",
        &[("host", "127.0.0.1:10520")],
        Some(json!({"language":"de","utterances":utterances})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[test]
fn route_table_covers_every_capability_row() {
    assert!(all_routes().len() >= 40);
    for (method, path, expected) in all_routes() {
        let concrete = path.replace("{id}", "demo");
        assert_eq!(route_capability(method, &concrete), Some(*expected), "{method} {path}");
    }
    assert_eq!(route_capability(&Method::POST, "/api/v2/parse"), Some(Capability::Read));
    assert_eq!(route_capability(&Method::POST, "/api/entities"), Some(Capability::PersistParse));
}
