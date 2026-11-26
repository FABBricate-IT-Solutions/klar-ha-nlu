use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::change_history::save_policies;
use crate::io::conversations::ConversationTurn;
use crate::io::http_error::HttpError;
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

#[derive(Debug, Deserialize)]
pub struct PreviewTurnsIn {
    pub rule_id: String,
    pub language: Option<String>,
    pub policies: Option<Vec<PolicyRule>>,
    #[serde(default)]
    pub match_controls: Option<Vec<MatchControl>>,
    /// How many recent journal turns to scan (default 50, max 200).
    pub last: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PreviewTurnsOut {
    pub matched: usize,
    pub scanned: usize,
    pub with_text: usize,
    pub samples: Vec<PreviewSample>,
}

#[derive(Debug, Serialize)]
pub struct PreviewSample {
    pub ts_ms: u64,
    pub decision: String,
    pub text: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/policies", get(get_policies).post(set_policies))
        .route("/api/v2/policies/catalog", get(get_catalog))
        .route("/api/v2/policies/evaluate", post(evaluate_policies))
        .route("/api/v2/policies/preview-turns", post(preview_turns))
        .route("/api/v2/policies/trainer-context", get(crate::io::trainer::trainer_context))
        .route("/api/v2/policies/propose/validate", post(crate::io::trainer::validate_proposal))
}

async fn get_catalog(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<MatchCatalog>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(MatchCatalog { matches: crate::parse::match_catalog(), seeds: crate::types::govern_safety_seeds().to_vec() }))
}

#[derive(Debug, Clone, Serialize)]
struct MatchCatalog {
    matches: Vec<MatchCatalogRow>,
    seeds: Vec<PolicyRule>,
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
) -> Result<Json<PolicyBundle>, HttpError> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED.into());
    }
    let policies = sanitize_rules(body.policies).map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("policies"))?;
    let speech_bank =
        sanitize_speech_bank(body.speech_bank).map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("speech_bank"))?;
    let match_controls = crate::parse::sanitize_match_controls(body.match_controls)
        .map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("match_controls"))?;
    save_policies(&state, policies.clone(), speech_bank.clone(), match_controls.clone(), "policies".into())?;
    *state.policies.lock().await = policies.clone();
    *state.speech_bank.lock().await = speech_bank.clone();
    *state.match_controls.lock().await = match_controls.clone();
    Ok(Json(bundle(policies, speech_bank, match_controls)))
}

async fn evaluate_policies(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<EvaluateIn>,
) -> Result<Json<EvaluateOut>, HttpError> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED.into());
    }
    if body.text.chars().count() > crate::io::limits::MAX_PARSE_CHARS {
        return Err(StatusCode::PAYLOAD_TOO_LARGE.into());
    }
    let policies = match body.policies {
        Some(rules) => sanitize_rules(rules).map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("policies"))?,
        None => state.policies.lock().await.clone(),
    };
    let match_controls = match body.match_controls {
        Some(rows) => crate::parse::sanitize_match_controls(rows)
            .map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("match_controls"))?,
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

async fn preview_turns(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<PreviewTurnsIn>,
) -> Result<Json<PreviewTurnsOut>, HttpError> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED.into());
    }
    let rule_id = body.rule_id.trim();
    if rule_id.is_empty() || rule_id.chars().count() > 128 {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "rule_id").with_path("rule_id"));
    }
    let last = body.last.unwrap_or(50).clamp(1, 200) as usize;
    let policies = match body.policies {
        Some(rules) => sanitize_rules(rules).map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("policies"))?,
        None => state.policies.lock().await.clone(),
    };
    if !policies.iter().any(|rule| rule.id == rule_id) {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "unknown rule_id").with_path("rule_id"));
    }
    let match_controls = match body.match_controls {
        Some(rows) => crate::parse::sanitize_match_controls(rows)
            .map_err(|msg| HttpError::new(StatusCode::BAD_REQUEST, msg).with_path("match_controls"))?,
        None => state.match_controls.lock().await.clone(),
    };
    let speech_bank = state.speech_bank.lock().await.clone();
    let mut settings = state.settings.lock().await.clone();
    if let Some(language) = body.language.as_deref().filter(|item| !item.is_empty()) {
        settings.languages = vec![language.to_string()];
    }
    let custom = state.custom.lock().await.clone();
    let home = state.home.snapshot().await;
    let turns = state.journal.list();
    let recent: Vec<ConversationTurn> = if turns.len() > last { turns[turns.len() - last..].to_vec() } else { turns };
    let mut matched = 0usize;
    let mut with_text = 0usize;
    let mut samples = Vec::new();
    for turn in &recent {
        let Some(text) = turn_utterance(turn) else {
            continue;
        };
        with_text += 1;
        let mut session = crate::session::Session::new();
        let outcome = parse_with_controls(&text, &home, &mut session, &custom, &settings, &policies, &speech_bank, &match_controls);
        let hit = outcome.policy_trace.as_ref().and_then(|trace| trace.matched_rule.as_deref()) == Some(rule_id);
        if hit {
            matched += 1;
            if samples.len() < 5 {
                samples.push(PreviewSample { ts_ms: turn.ts_ms, decision: turn.decision.clone(), text });
            }
        }
    }
    Ok(Json(PreviewTurnsOut { matched, scanned: recent.len(), with_text, samples }))
}

fn turn_utterance(turn: &ConversationTurn) -> Option<String> {
    if let Some(text) = turn.text.as_deref().map(str::trim).filter(|item| !item.is_empty()) {
        return Some(text.to_string());
    }
    let joined = turn.tokens.iter().map(String::as_str).collect::<Vec<_>>().join(" ");
    let trimmed = joined.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
