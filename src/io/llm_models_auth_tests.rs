use super::*;
use crate::home::{default_home, LoadedHome};
use crate::types::Settings;

fn state(token: Option<String>) -> AppState {
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
        std::env::temp_dir().join(format!("klar-llm-auth-{}", std::process::id())),
        token,
    )
}

async fn list_without_token(peer: &str) -> StatusCode {
    let peer = ConnectInfo(peer.parse().unwrap());
    list_endpoint_models(State(state(Some("secret".into()))), peer, HeaderMap::new(), Json(ModelsIn::default())).await.unwrap_err()
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
