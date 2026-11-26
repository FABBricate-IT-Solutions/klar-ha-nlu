//! HTTP for rules/settings change history (package 6.7).

use crate::home::change_history::{
    history_rows_policies, history_rows_settings, push_policies_revision, push_settings_revision, select_policies, select_settings,
    HistoryRow, MAX_CHANGE_HISTORY,
};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::http_error::HttpError;
use crate::io::policies::PolicyBundle;
use crate::io::state::AppState;
use crate::types::{sanitize_rules, sanitize_speech_bank, MatchControl, PolicyRule, Settings, SpeechBank};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct RollbackIn {
    pub hash: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/policies/history", get(list_policies_history))
        .route("/api/v2/policies/rollback", post(rollback_policies))
        .route("/api/v2/settings/history", get(list_settings_history))
        .route("/api/v2/settings/rollback", post(rollback_settings))
}

async fn list_policies_history(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Vec<HistoryRow>>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let overlay = state.overlay.load();
    Ok(Json(history_rows_policies(&overlay.policies_history)))
}

async fn list_settings_history(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Vec<HistoryRow>>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let overlay = state.overlay.load();
    Ok(Json(history_rows_settings(&overlay.settings_history)))
}

async fn rollback_policies(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<RollbackIn>,
) -> Result<Json<PolicyBundle>, HttpError> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED.into());
    }
    let overlay = state.overlay.load();
    let revision = select_policies(&overlay.policies_history, body.hash.as_deref()).ok_or(StatusCode::NOT_FOUND)?;
    let policies = sanitize_rules(revision.policies).map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("policies"))?;
    let speech_bank =
        sanitize_speech_bank(revision.speech_bank).map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("speech_bank"))?;
    let match_controls = crate::parse::sanitize_match_controls(revision.match_controls)
        .map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("match_controls"))?;
    save_policies(&state, policies.clone(), speech_bank.clone(), match_controls.clone(), format!("rollback {}", revision.hash))?;
    *state.policies.lock().await = policies.clone();
    *state.speech_bank.lock().await = speech_bank.clone();
    *state.match_controls.lock().await = match_controls.clone();
    Ok(Json(PolicyBundle { policies, speech_bank, match_controls }))
}

async fn rollback_settings(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<RollbackIn>,
) -> Result<Json<Settings>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let overlay = state.overlay.load();
    let revision = select_settings(&overlay.settings_history, body.hash.as_deref()).ok_or(StatusCode::NOT_FOUND)?;
    let current = state.settings.lock().await.clone();
    let mut next = crate::lang::check_settings(revision.settings, &current).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    next.time_zone = crate::io::trainer_settings::checked_time_zone(&next.time_zone).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    save_settings(&state, next.clone(), format!("rollback {}", revision.hash))?;
    *state.settings.lock().await = next.clone();
    Ok(Json(next))
}

/// Persist a new policy bundle and keep the previous one in `policies_history`.
pub(crate) fn save_policies(
    state: &AppState,
    policies: Vec<PolicyRule>,
    speech_bank: SpeechBank,
    match_controls: Vec<MatchControl>,
    label: String,
) -> Result<(), HttpError> {
    state
        .overlay
        .update(|overlay| {
            push_policies_revision(
                &mut overlay.policies_history,
                overlay.policies.clone(),
                overlay.speech_bank.clone(),
                overlay.match_controls.clone(),
                label.clone(),
            );
            overlay.policies = policies;
            overlay.speech_bank = speech_bank;
            overlay.match_controls = match_controls;
        })
        .map_err(HttpError::save_failed)?;
    Ok(())
}

/// Persist settings and keep the previous snapshot in `settings_history`.
pub(crate) fn save_settings(state: &AppState, settings: Settings, label: String) -> Result<(), StatusCode> {
    state
        .overlay
        .update(|overlay| {
            if let Some(previous) = overlay.settings.clone() {
                push_settings_revision(&mut overlay.settings_history, previous, label.clone());
            }
            overlay.settings = Some(settings);
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

pub(crate) fn history_ok_policies(overlay: &crate::home::overlay::Overlay) -> bool {
    overlay.policies_history.len() <= MAX_CHANGE_HISTORY
        && overlay.policies_history.iter().all(|row| {
            sanitize_rules(row.policies.clone()).is_ok()
                && sanitize_speech_bank(row.speech_bank.clone()).is_ok()
                && crate::parse::sanitize_match_controls(row.match_controls.clone()).is_ok()
        })
}

pub(crate) fn history_ok_settings(overlay: &crate::home::overlay::Overlay) -> bool {
    overlay.settings_history.len() <= MAX_CHANGE_HISTORY
}
