//! `GET /api/v2/health` — version, home age, overlay, LLM, Wyoming (R-19).
//!
//! Public (no token): add-on watchdog, container healthcheck, and the integration
//! version check all probe this without write credentials.

use crate::home::overlay_store::OverlayRead;
use crate::io::state::AppState;
use crate::llm::LlmPublic;
use crate::ops::{ops, OpsSnapshot};
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v2/health", get(health))
}

#[derive(Debug, Serialize)]
struct Health {
    ok: bool,
    version: &'static str,
    sha: &'static str,
    home: HomeHealth,
    overlay: OverlayHealth,
    llm: LlmPublic,
    wyoming: WyomingHealth,
    /// Latency, sessions, overlay writes, catalog cache, Wyoming (3.10).
    ops: OpsSnapshot,
}

#[derive(Debug, Serialize)]
struct HomeHealth {
    /// `live` after a Home Assistant push, else `registry` or `empty`.
    source: &'static str,
    /// Seconds since the last successful `POST /api/v2/home`. Absent until the first push.
    #[serde(skip_serializing_if = "Option::is_none")]
    age_secs: Option<u64>,
    entities: usize,
}

#[derive(Debug, Serialize)]
struct OverlayHealth {
    /// `missing` | `ok` | `partial` | `unreadable`
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct WyomingHealth {
    listening: bool,
}

fn build_sha() -> &'static str {
    option_env!("GITHUB_SHA").or(option_env!("KLAR_GIT_SHA")).unwrap_or("dev")
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    let graph = state.home.snapshot().await;
    let live = state.live_sync.load(Ordering::Relaxed);
    let synced_ms = state.home_synced_ms.load(Ordering::Relaxed);
    let age_secs = if synced_ms == 0 { None } else { Some(now_ms().saturating_sub(synced_ms) / 1000) };
    let source = if live {
        "live"
    } else if graph.entities.is_empty() {
        "empty"
    } else {
        "registry"
    };
    let overlay = match state.overlay.read() {
        OverlayRead::Missing => "missing",
        OverlayRead::Loaded { dropped, .. } if dropped.is_empty() => "ok",
        OverlayRead::Loaded { .. } => "partial",
        OverlayRead::Unreadable(_) => "unreadable",
    };
    let llm = state.llm.lock().await.as_ref().map(|endpoint| endpoint.public()).unwrap_or_else(LlmPublic::empty);
    {
        let sessions = state.sessions.lock().await;
        ops().sync_sessions(sessions.created(), sessions.len() as u64);
    }
    Json(Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
        sha: build_sha(),
        home: HomeHealth { source, age_secs, entities: graph.entities.len() },
        overlay: OverlayHealth { status: overlay },
        llm,
        wyoming: WyomingHealth { listening: state.wyoming_up.load(Ordering::Relaxed) },
        ops: ops().snapshot(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::test_dir::TestDir;
    use crate::home::LoadedHome;
    use crate::types::{HomeGraph, Settings};
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tower::ServiceExt;

    fn state() -> (AppState, TestDir) {
        let scratch = TestDir::new("health");
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
            Some("secret".into()),
        );
        (state, scratch)
    }

    #[tokio::test]
    async fn health_is_public_and_reports_build_identity() {
        let (state, _scratch) = state();
        state.wyoming_up.store(true, Ordering::Relaxed);
        let response =
            crate::io::web::router(state).oneshot(Request::get("/api/v2/health").body(axum::body::Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap()).unwrap();
        assert_eq!(body["ok"], true);
        assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
        assert!(body["sha"].as_str().is_some_and(|sha| !sha.is_empty()));
        assert_eq!(body["home"]["source"], "empty");
        assert!(body["home"]["age_secs"].is_null());
        assert_eq!(body["overlay"]["status"], "missing");
        assert_eq!(body["llm"]["configured"], false);
        assert_eq!(body["wyoming"]["listening"], true);
        assert!(body["ops"]["requests"].as_u64().is_some());
        assert!(body["ops"]["overlay_writes"].as_u64().is_some());
        assert!(body["ops"]["wyoming_connects"].as_u64().is_some());
    }

    #[tokio::test]
    async fn health_reports_live_home_age_after_sync_stamp() {
        let (state, _scratch) = state();
        state.live_sync.store(true, Ordering::Relaxed);
        state.home_synced_ms.store(now_ms().saturating_sub(5_000), Ordering::Relaxed);
        let response = health(State(state)).await;
        assert_eq!(response.0.home.source, "live");
        assert!(response.0.home.age_secs.is_some_and(|age| age >= 4));
    }
}
