//! `GET /api/v2/setup/token` — Supervisor/loopback claim of the auto-minted write token (6.6).
//!
//! Fresh App installs mint `/data/klar.token` when the option is empty. The integration
//! claims that token over the Supervisor network so the first Assist command needs no paste.

use crate::io::auth::{reads_allowed, trusted_peer};
use crate::io::state::AppState;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v2/setup/token", get(claim_token))
}

fn build_sha() -> &'static str {
    option_env!("GITHUB_SHA").or(option_env!("KLAR_GIT_SHA")).unwrap_or("dev")
}

#[derive(Debug, Serialize)]
struct SetupToken {
    token: String,
    version: &'static str,
    sha: &'static str,
}

async fn claim_token(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<SetupToken>, StatusCode> {
    // Trusted peers may read without the write token; LAN still needs it (same as other reads).
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // Refuse a claim from a peer that is neither loopback nor Supervisor even when a
    // matching LAN token was sent — the write token is for writes, not for redistributing
    // itself off-box.
    if !trusted_peer(peer.ip()) {
        return Err(StatusCode::FORBIDDEN);
    }
    let Some(token) = state.token.as_ref().filter(|value| !value.is_empty()) else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(SetupToken { token: token.clone(), version: env!("CARGO_PKG_VERSION"), sha: build_sha() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::test_dir::TestDir;
    use crate::home::LoadedHome;
    use crate::types::{HomeGraph, Settings};
    use axum::body::to_bytes;
    use axum::extract::ConnectInfo;
    use axum::http::Request;
    use serde_json::Value;
    use tower::ServiceExt;

    fn state(token: Option<&str>) -> (AppState, TestDir) {
        let scratch = TestDir::new("setup");
        let dir = scratch.path().to_path_buf();
        let state = AppState::new(
            LoadedHome {
                graph: HomeGraph::default(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            token.map(str::to_string),
        );
        (state, scratch)
    }

    async fn call(state: AppState, peer: &str, headers: &[(&str, &str)]) -> axum::response::Response {
        let mut builder = Request::builder().method("GET").uri("/api/v2/setup/token");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let mut request = builder.body(axum::body::Body::empty()).unwrap();
        request.extensions_mut().insert(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
        crate::io::web::router(state).oneshot(request).await.unwrap()
    }

    #[tokio::test]
    async fn supervisor_claims_auto_token_without_manual_paste() {
        let (state, _scratch) = state(Some("auto-secret"));
        let response = call(state, "172.30.32.2:9", &[]).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap()).unwrap();
        assert_eq!(body["token"], "auto-secret");
        assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
        assert!(body["sha"].as_str().is_some_and(|sha| !sha.is_empty()));
    }

    #[tokio::test]
    async fn lan_peer_cannot_claim_even_with_matching_token() {
        let (state, _scratch) = state(Some("auto-secret"));
        let response = call(state, "10.0.0.8:9", &[("x-klar-token", "auto-secret")]).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn missing_token_is_not_found() {
        let (state, _scratch) = state(None);
        let response = call(state, "127.0.0.1:9", &[]).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
