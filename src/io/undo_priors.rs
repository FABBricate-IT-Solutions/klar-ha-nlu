//! HA stamps prior continuous values onto the last execute so undo can restore them.

use crate::io::auth::writes_allowed;
use crate::io::state::AppState;
use crate::session::UndoPrior;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

const MAX_PRIORS: usize = 32;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UndoPriorsIn {
    pub conversation_id: String,
    #[serde(default)]
    pub priors: Vec<PriorIn>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorIn {
    pub entity_id: String,
    pub brightness: Option<u8>,
    pub position: Option<u8>,
    pub temperature: Option<String>,
    pub volume_level: Option<u8>,
    pub percentage: Option<u8>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UndoPriorsOut {
    pub stamped: usize,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v2/session/undo-priors", post(stamp_undo_priors))
}

async fn stamp_undo_priors(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<UndoPriorsIn>,
) -> Result<Json<UndoPriorsOut>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if body.conversation_id.is_empty() || !crate::session::valid_id(&body.conversation_id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if body.priors.len() > MAX_PRIORS {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let priors: Vec<UndoPrior> = body
        .priors
        .into_iter()
        .filter_map(|row| {
            let entity_id = row.entity_id.trim();
            if entity_id.is_empty() || entity_id.chars().count() > 128 || entity_id.chars().any(char::is_control) {
                return None;
            }
            Some(UndoPrior {
                entity_id: entity_id.to_string(),
                brightness: row.brightness.filter(|v| *v <= 100),
                position: row.position.filter(|v| *v <= 100),
                temperature: row.temperature.filter(|t| !t.is_empty() && t.chars().count() <= 32),
                volume_level: row.volume_level.filter(|v| *v <= 100),
                percentage: row.percentage.filter(|v| *v <= 100),
                color: row.color.filter(|c| !c.is_empty() && c.chars().count() <= 64),
            })
        })
        .collect();
    let stamped = priors.len();
    let mut sessions = state.sessions.lock().await;
    if !sessions.stamp_undo_priors(&body.conversation_id, &priors) {
        return Ok(Json(UndoPriorsOut { stamped: 0 }));
    }
    Ok(Json(UndoPriorsOut { stamped }))
}
