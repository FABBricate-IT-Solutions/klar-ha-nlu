use super::*;
use crate::home::test_dir::TestDir;
use crate::home::{default_home, LoadedHome};
use crate::types::Settings;

fn state(token: Option<String>) -> (TestDir, AppState) {
    let scratch = TestDir::new("llm-auth");
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
        token,
    );
    (scratch, state)
}

async fn list_without_token(peer: &str) -> StatusCode {
    let peer = ConnectInfo(peer.parse().unwrap());
    let (_scratch, state) = state(Some("secret".into()));
    list_endpoint_models(State(state), peer, HeaderMap::new(), Json(ModelsIn::default())).await.unwrap_err()
}

#[tokio::test]
async fn supervisor_ingress_lists_models_without_token() {
    assert_eq!(list_without_token("172.30.32.1:9").await, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn loopback_lists_models_without_token() {
    assert_eq!(list_without_token("127.0.0.1:9").await, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lan_lists_models_without_token_is_unauthorized() {
    assert_eq!(list_without_token("10.0.0.8:9").await, StatusCode::UNAUTHORIZED);
}
