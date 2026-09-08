use crate::io::auth::reads_allowed;
use crate::io::state::AppState;
use crate::types::{SnapshotError, SpeechRenderOut, SpeechSnapshot};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use std::net::SocketAddr;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v2/speech/render", post(speech_render))
}

async fn speech_render(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<SpeechSnapshot>,
) -> Result<Json<SpeechRenderOut>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let mut snap = body.sanitize().map_err(status_for)?;
    snap.unit_system = state.settings.lock().await.unit_system;
    Ok(Json(crate::speech::render_snapshot(&snap)))
}

fn status_for(err: SnapshotError) -> StatusCode {
    match err {
        SnapshotError::Schema | SnapshotError::Outcome | SnapshotError::Language | SnapshotError::Now => StatusCode::BAD_REQUEST,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::types::{Settings, SpeechIntent, SpeechSlot, UnitSystem};

    fn snap() -> SpeechSnapshot {
        SpeechSnapshot {
            schema_version: "1".into(),
            language: "de".into(),
            personality: "default".into(),
            unit_system: UnitSystem::Metric,
            now: "2026-09-06T22:26:00+02:00".into(),
            intent: SpeechIntent {
                name: "HassTurnOn".into(),
                slots: vec![
                    SpeechSlot { name: "entity_id".into(), value: "light.kuche_kuche".into() },
                    SpeechSlot { name: "name".into(), value: "Küche Licht".into() },
                ],
            },
            outcome: "success".into(),
            entities: vec![],
            calendar_events: vec![],
            media_queue: vec![],
        }
    }

    fn state() -> AppState {
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
            std::env::temp_dir().join(format!("klar-speech-{}", std::process::id())),
            None,
        )
    }

    #[tokio::test]
    async fn supervisor_core_renders_without_token() {
        let peer = ConnectInfo("172.30.32.1:9".parse().unwrap());
        let out = speech_render(State(state()), peer, HeaderMap::new(), Json(snap())).await.expect("supervisor render").0;
        assert_eq!(out.source, "post_execute");
        assert!(out.speech.contains("Küche Licht"), "{:?}", out.speech);
    }

    #[tokio::test]
    async fn lan_render_without_token_is_unauthorized() {
        let peer = ConnectInfo("10.0.0.8:9".parse().unwrap());
        let err = speech_render(State(state()), peer, HeaderMap::new(), Json(snap())).await.unwrap_err();
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn route_uses_read_gate() {
        let src = include_str!("speech.rs");
        let body = src.split("#[cfg(test)]").next().expect("prod");
        assert!(body.contains("reads_allowed"));
        assert!(!body.contains("writes_allowed"));
    }
}
