use crate::io::auth::writes_allowed;
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
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let snap = body.sanitize().map_err(status_for)?;
    Ok(Json(crate::speech::render_snapshot(&snap)))
}

fn status_for(err: SnapshotError) -> StatusCode {
    match err {
        SnapshotError::Schema | SnapshotError::Outcome | SnapshotError::Language | SnapshotError::Now => StatusCode::BAD_REQUEST,
    }
}
