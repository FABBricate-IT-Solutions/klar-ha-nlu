//! The stored LLM key goes to the stored `base_url` and nowhere else, the model list sits
//! behind the write gate, and the key file is owner-only.
mod state_support;

use axum::http::{Method, StatusCode};
use serde_json::{json, Value};
use state_support::fake_llm::FakeLlm;
use state_support::{process, read_json, Engine, TempDir, LLM_FILE};
use std::os::unix::fs::PermissionsExt;

const MADE_UP_KEY: &str = "sk-made-up-for-tests";

fn bearer(key: &str) -> Option<String> {
    Some(format!("Bearer {key}"))
}

async fn engine_with_stored_key(dir: &TempDir, home: &FakeLlm) -> Engine {
    let engine = Engine::start(dir.path());
    let saved = json!({"base_url": home.base_url, "api_key": MADE_UP_KEY, "model": "kleines-modell"});
    assert_eq!(engine.post("/api/v2/llm/endpoint", saved).await, StatusCode::OK);
    engine
}

#[tokio::test]
async fn a_foreign_base_url_gets_no_key_from_the_model_list() {
    let dir = TempDir::new("llm-foreign");
    let (home, foreign) = (FakeLlm::start().await, FakeLlm::start().await);
    let engine = engine_with_stored_key(&dir, &home).await;

    let (status, listed) = engine.post_json("/api/v2/llm/models", json!({"base_url": foreign.base_url})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["models"], json!(["kleines-modell"]));
    assert_eq!(foreign.credentials(), [None], "the stored key was sent to a host it was not entered for");

    // The stored address still gets it, named or not.
    assert_eq!(engine.post_json("/api/v2/llm/models", json!({})).await.0, StatusCode::OK);
    assert_eq!(engine.post_json("/api/v2/llm/models", json!({"base_url": format!("{}/", home.base_url)})).await.0, StatusCode::OK);
    assert_eq!(home.credentials(), [bearer(MADE_UP_KEY), bearer(MADE_UP_KEY)]);

    // A key the caller sends along is the caller's to send.
    let own = json!({"base_url": foreign.base_url, "api_key": "sk-typed-for-this-host"});
    assert_eq!(engine.post_json("/api/v2/llm/models", own).await.0, StatusCode::OK);
    assert_eq!(foreign.credentials(), [None, bearer("sk-typed-for-this-host")]);
}

#[tokio::test]
async fn saving_another_base_url_without_a_key_does_not_move_the_stored_key() {
    let dir = TempDir::new("llm-move");
    let (home, foreign) = (FakeLlm::start().await, FakeLlm::start().await);
    let engine = engine_with_stored_key(&dir, &home).await;

    // Another model on the same address keeps the key.
    let same = json!({"base_url": home.base_url, "model": "anderes-modell"});
    assert_eq!(engine.post("/api/v2/llm/endpoint", same).await, StatusCode::OK);
    assert_eq!(read_json(&dir.file(LLM_FILE))["api_key"], MADE_UP_KEY);

    let moved = json!({"base_url": foreign.base_url, "model": "kleines-modell"});
    assert_eq!(engine.post("/api/v2/llm/endpoint", moved).await, StatusCode::OK);
    let stored = read_json(&dir.file(LLM_FILE));
    assert_eq!(stored["base_url"], foreign.base_url.as_str());
    assert_eq!(stored["api_key"], "", "the key must not follow to another address");

    let chat = json!({"messages": [{"role": "user", "content": "hallo"}], "stream": false});
    let (status, answer) = engine.post_json("/api/v2/llm/chat", chat).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(foreign.credentials(), [None], "the new address must not be sent the old key");
    assert_eq!(home.credentials(), Vec::<Option<String>>::new());
}

#[tokio::test]
async fn the_model_list_is_behind_the_write_gate() {
    let dir = TempDir::new("llm-gate");
    let home = FakeLlm::start().await;
    let engine = Engine::start_with_token(dir.path(), Some("werkstatt-token"));
    let body = serde_json::to_vec(&json!({"base_url": home.base_url})).unwrap();
    let list = |peer: &'static str, token: Option<&'static str>| {
        let (engine, body) = (&engine, body.clone());
        async move { engine.send_as(peer, token, Method::POST, "/api/v2/llm/models", "application/json", body).await.0 }
    };

    // The Supervisor network may read without a token. This call is not a read.
    assert_eq!(list("172.30.32.2:4711", None).await, StatusCode::UNAUTHORIZED);
    assert_eq!(list("192.168.77.20:4711", None).await, StatusCode::UNAUTHORIZED);
    assert_eq!(home.credentials(), Vec::<Option<String>>::new(), "a refused caller must not make the engine call out");

    assert_eq!(list("172.30.32.2:4711", Some("werkstatt-token")).await, StatusCode::OK);
    assert_eq!(list("127.0.0.1:4711", None).await, StatusCode::OK);
    let read = engine.send_as("172.30.32.2:4711", None, Method::GET, "/api/v2/llm/endpoint", "application/json", Vec::new()).await;
    assert_eq!(read.0, StatusCode::OK, "reads stay open to the Supervisor network");
}

fn mode(path: &std::path::Path) -> u32 {
    std::fs::metadata(path).expect("file exists").permissions().mode() & 0o777
}

#[tokio::test]
async fn the_key_file_is_written_owner_only() {
    let dir = TempDir::new("llm-mode");
    let home = FakeLlm::start().await;
    let _engine = engine_with_stored_key(&dir, &home).await;
    assert_eq!(mode(&dir.file(LLM_FILE)), 0o600);
}

#[test]
fn a_key_file_from_an_older_version_is_closed_at_start() {
    let root = TempDir::new("llm-mode-start");
    std::fs::create_dir_all(root.file("data")).unwrap();
    let file = root.file("data").join(LLM_FILE);
    let stored = json!({"base_url": "http://127.0.0.1:9/v1", "api_key": MADE_UP_KEY, "model": "kleines-modell"});
    std::fs::write(&file, stored.to_string()).unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();

    let running = process::start(&root, &[]);
    let (status, body) = running.get("/api/v2/llm/endpoint");
    assert_eq!(status, 200, "{}", running.log());
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["model"], "kleines-modell");
    assert_eq!(mode(&file), 0o600, "{}", running.log());
    assert!(!running.log().contains(MADE_UP_KEY), "the key must not be logged");
}
