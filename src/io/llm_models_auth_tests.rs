use super::*;
use crate::home::test_dir::TestDir;
use crate::home::{default_home, LoadedHome};
use crate::types::Settings;

fn state(dir: &TestDir, token: Option<String>) -> AppState {
    AppState::new(
        LoadedHome {
            graph: default_home(),
            settings: Settings::default(),
            custom: Vec::new(),
            language: Default::default(),
            policies: Vec::new(),
            speech_bank: Default::default(),
            match_controls: Vec::new(),
        },
        dir.path().to_path_buf(),
        token,
    )
}

async fn list_as(peer: &str, sent: Option<&str>) -> StatusCode {
    let peer = ConnectInfo(peer.parse().unwrap());
    let mut headers = HeaderMap::new();
    if let Some(token) = sent {
        headers.insert("x-klar-token", token.parse().unwrap());
    }
    let dir = TestDir::new("llm-auth");
    list_endpoint_models(State(state(&dir, Some("secret".into()))), peer, headers, Json(ModelsIn::default())).await.unwrap_err()
}

// The list makes the engine contact a host the caller names, so it sits behind the write
// gate. Past the gate the empty request is a 400: there is no base URL to ask.

#[tokio::test]
async fn supervisor_ingress_needs_the_token_to_list_models() {
    assert_eq!(list_as("172.30.32.1:9", None).await, StatusCode::UNAUTHORIZED);
    assert_eq!(list_as("172.30.32.1:9", Some("wrong")).await, StatusCode::UNAUTHORIZED);
    assert_eq!(list_as("172.30.32.1:9", Some("secret")).await, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn loopback_lists_models_without_token() {
    assert_eq!(list_as("127.0.0.1:9", None).await, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lan_needs_the_token_to_list_models() {
    assert_eq!(list_as("10.0.0.8:9", None).await, StatusCode::UNAUTHORIZED);
    assert_eq!(list_as("10.0.0.8:9", Some("secret")).await, StatusCode::BAD_REQUEST);
}
