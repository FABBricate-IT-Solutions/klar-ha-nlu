use crate::home::overlay::{load_overlay, save_overlay};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::state::AppState;
use crate::nlu::parse_with_controls;
use crate::types::{sanitize_rules, sanitize_speech_bank, MatchCatalogRow, MatchControl, ParseOutcome, PolicyRule, SpeechBank};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBundle {
    pub policies: Vec<PolicyRule>,
    #[serde(default)]
    pub speech_bank: SpeechBank,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub match_controls: Vec<MatchControl>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateIn {
    pub text: String,
    pub language: Option<String>,
    pub policies: Option<Vec<PolicyRule>>,
    #[serde(default)]
    pub match_controls: Option<Vec<MatchControl>>,
}

#[derive(Debug, Serialize)]
pub struct EvaluateOut {
    pub outcome: ParseOutcome,
    pub compiled_risky: bool,
    pub matched_rule: Option<String>,
    pub hit: Option<String>,
    pub speech_variant: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/policies", get(get_policies).post(set_policies))
        .route("/api/v2/policies/catalog", get(get_catalog))
        .route("/api/v2/policies/evaluate", post(evaluate_policies))
}

async fn get_catalog(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<MatchCatalog>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(MatchCatalog { matches: crate::parse::match_catalog() }))
}

#[derive(Debug, Clone, Serialize)]
struct MatchCatalog {
    matches: Vec<MatchCatalogRow>,
}

fn bundle(policies: Vec<PolicyRule>, speech_bank: SpeechBank, match_controls: Vec<MatchControl>) -> PolicyBundle {
    PolicyBundle { policies, speech_bank, match_controls }
}

async fn get_policies(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<PolicyBundle>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(bundle(state.policies.lock().await.clone(), state.speech_bank.lock().await.clone(), state.match_controls.lock().await.clone())))
}

async fn set_policies(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<PolicyBundle>,
) -> Result<Json<PolicyBundle>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let policies = sanitize_rules(body.policies).map_err(|_| StatusCode::BAD_REQUEST)?;
    let speech_bank = sanitize_speech_bank(body.speech_bank).map_err(|_| StatusCode::BAD_REQUEST)?;
    let match_controls = crate::parse::sanitize_match_controls(body.match_controls).map_err(|_| StatusCode::BAD_REQUEST)?;
    *state.policies.lock().await = policies.clone();
    *state.speech_bank.lock().await = speech_bank.clone();
    *state.match_controls.lock().await = match_controls.clone();
    let mut overlay = load_overlay(&state.data_dir);
    overlay.policies = policies.clone();
    overlay.speech_bank = speech_bank.clone();
    overlay.match_controls = match_controls.clone();
    let _ = save_overlay(&state.data_dir, &overlay);
    Ok(Json(bundle(policies, speech_bank, match_controls)))
}

async fn evaluate_policies(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<EvaluateIn>,
) -> Result<Json<EvaluateOut>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if body.text.chars().count() > crate::io::limits::MAX_PARSE_CHARS {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let policies = match body.policies {
        Some(rules) => sanitize_rules(rules).map_err(|_| StatusCode::BAD_REQUEST)?,
        None => state.policies.lock().await.clone(),
    };
    let match_controls = match body.match_controls {
        Some(rows) => crate::parse::sanitize_match_controls(rows).map_err(|_| StatusCode::BAD_REQUEST)?,
        None => state.match_controls.lock().await.clone(),
    };
    let speech_bank = state.speech_bank.lock().await.clone();
    let mut settings = state.settings.lock().await.clone();
    if let Some(language) = body.language.as_deref().filter(|item| !item.is_empty()) {
        settings.languages = vec![language.to_string()];
    }
    let custom = state.custom.lock().await.clone();
    let home = state.home.snapshot().await;
    let mut session = crate::session::Session::new();
    let outcome = parse_with_controls(&body.text, &home, &mut session, &custom, &settings, &policies, &speech_bank, &match_controls);
    let trace = outcome.policy_trace.clone().unwrap_or_default();
    Ok(Json(EvaluateOut {
        speech_variant: Some(outcome.speech.clone()),
        outcome,
        compiled_risky: trace.compiled_risky,
        matched_rule: trace.matched_rule,
        hit: trace.hit,
        warnings: crate::parse::match_control_warnings(&match_controls),
    }))
}
