//! Preview, explain, and rollback for user language rules.

use crate::home::overlay::{load_overlay, save_overlay};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::lang_explain::{explain_outcome, ExplainOut};
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::state::AppState;
use crate::lang::{
    bind_preview_user, install_user_overlay, pin_language, push_revision, select_revision, validate_custom, validate_language,
    LanguageOverlay, LanguageRevision,
};
use crate::nlu::parse;
use crate::session::Session;
use crate::types::{CustomSentence, ParseOutcome, Settings};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/languages", get(list_languages))
        .route("/api/v2/intents", get(list_intents))
        .route("/api/lang/overlay", get(get_overlay).post(set_overlay))
        .route("/api/lang/preview", post(preview))
        .route("/api/lang/explain", post(explain))
        .route("/api/lang/rollback", post(rollback))
}

async fn list_intents() -> Json<Vec<&'static str>> {
    Json(crate::types::KNOWN_INTENTS.to_vec())
}

async fn list_languages() -> Json<Vec<LanguageOut>> {
    Json(
        crate::lang::languages()
            .iter()
            .map(|meta| LanguageOut {
                code: meta.code.to_string(),
                native_name: meta.native_name.to_string(),
                script: meta.script.to_string(),
                variants: meta.variants.iter().map(|item| (*item).to_string()).collect(),
            })
            .collect(),
    )
}

#[derive(Serialize)]
struct LanguageOut {
    code: String,
    native_name: String,
    script: String,
    variants: Vec<String>,
}

#[derive(Serialize)]
struct OverlayOut {
    custom: Vec<CustomSentence>,
    language: LanguageOverlay,
    history: Vec<HistoryRow>,
}

#[derive(Serialize)]
struct HistoryRow {
    hash: String,
    label: String,
    saved_at: String,
}

#[derive(Deserialize)]
struct OverlayIn {
    #[serde(default)]
    custom: Vec<CustomSentence>,
    #[serde(default)]
    language: Option<LanguageOverlay>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Deserialize)]
struct PreviewIn {
    text: String,
    language: Option<String>,
    #[serde(default)]
    custom: Option<Vec<CustomSentence>>,
    #[serde(default)]
    language_overlay: Option<LanguageOverlay>,
}

#[derive(Deserialize)]
struct RollbackIn {
    hash: Option<String>,
}

fn write_gate(peer: SocketAddr, headers: &HeaderMap, token: &Option<String>) -> Result<(), StatusCode> {
    if writes_allowed(Some(peer), headers, token) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn read_gate(peer: SocketAddr, headers: &HeaderMap, token: &Option<String>) -> Result<(), StatusCode> {
    if reads_allowed(Some(peer), headers, token) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn get_overlay(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<OverlayOut>, StatusCode> {
    read_gate(peer, &headers, &state.token)?;
    let overlay = load_overlay(&state.data_dir);
    Ok(Json(overlay_out(&overlay.custom, &overlay.language, &overlay.language_history)))
}

async fn set_overlay(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<OverlayIn>,
) -> Result<Json<OverlayOut>, StatusCode> {
    write_gate(peer, &headers, &state.token)?;
    let language = match body.language {
        Some(language) => language,
        None => load_overlay(&state.data_dir).language,
    };
    persist_rules(&state, body.custom, language, body.label.unwrap_or_else(|| "save".into())).await
}

async fn preview(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<PreviewIn>,
) -> Result<Json<ParseOutcome>, StatusCode> {
    read_gate(peer, &headers, &state.token)?;
    if body.text.chars().count() > MAX_PARSE_CHARS {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let settings = pin_settings(state.settings.lock().await.clone(), body.language.as_deref())?;
    let custom = match &body.custom {
        Some(rows) => {
            if !validate_custom(rows).is_empty() {
                return Err(StatusCode::UNPROCESSABLE_ENTITY);
            }
            rows.clone()
        }
        None => state.custom.lock().await.clone(),
    };
    if let Some(language) = &body.language_overlay {
        if !validate_language(language).is_empty() {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }
    let home = state.home.snapshot().await;
    let mut session = Session::default();
    let _preview = body.language_overlay.clone().map(|overlay| bind_preview_user(Some(overlay)));
    let outcome = parse(&body.text, &home, &mut session, &custom, &settings);
    Ok(Json(outcome))
}

async fn explain(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<PreviewIn>,
) -> Result<Json<ExplainOut>, StatusCode> {
    let language = body.language.clone().unwrap_or_default();
    let Json(outcome) = preview(State(state.clone()), ConnectInfo(peer), headers, Json(body)).await?;
    let mut out = explain_outcome(&language, &outcome);
    out.language = language;
    Ok(Json(out))
}

async fn rollback(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<RollbackIn>,
) -> Result<Json<OverlayOut>, StatusCode> {
    write_gate(peer, &headers, &state.token)?;
    let overlay = load_overlay(&state.data_dir);
    let revision = select_revision(&overlay.language_history, body.hash.as_deref()).ok_or(StatusCode::NOT_FOUND)?;
    persist_rules(&state, revision.custom, revision.language, format!("rollback {}", revision.hash)).await
}

pub(crate) async fn persist_language_overlay(state: &AppState, language: LanguageOverlay, label: &str) -> Result<(), StatusCode> {
    let custom = state.custom.lock().await.clone();
    persist_rules(state, custom, language, label.to_string()).await.map(|_| ())
}

pub(crate) async fn persist_custom(state: &AppState, custom: Vec<CustomSentence>, label: &str) -> Result<(), StatusCode> {
    let language = load_overlay(&state.data_dir).language;
    persist_rules(state, custom, language, label.to_string()).await.map(|_| ())
}

async fn persist_rules(
    state: &AppState,
    custom: Vec<CustomSentence>,
    language: LanguageOverlay,
    label: String,
) -> Result<Json<OverlayOut>, StatusCode> {
    if !validate_custom(&custom).is_empty() || !validate_language(&language).is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let mut overlay = load_overlay(&state.data_dir);
    push_revision(&mut overlay.language_history, overlay.custom.clone(), overlay.language.clone(), label);
    overlay.custom = custom.clone();
    overlay.language = language.clone();
    save_overlay(&state.data_dir, &overlay).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    *state.custom.lock().await = custom.clone();
    install_user_overlay(if language.sets.is_empty() { None } else { Some(language.clone()) });
    Ok(Json(overlay_out(&custom, &language, &overlay.language_history)))
}

fn overlay_out(custom: &[CustomSentence], language: &LanguageOverlay, history: &[LanguageRevision]) -> OverlayOut {
    OverlayOut {
        custom: custom.to_vec(),
        language: language.clone(),
        history: history
            .iter()
            .rev()
            .map(|row| HistoryRow { hash: row.hash.clone(), label: row.label.clone(), saved_at: row.saved_at.clone() })
            .collect(),
    }
}

fn pin_settings(mut settings: Settings, language: Option<&str>) -> Result<Settings, StatusCode> {
    let Some(raw) = language.filter(|value| !value.is_empty()) else {
        return Ok(settings);
    };
    match pin_language(raw) {
        Ok(tag) => {
            settings.languages = vec![tag];
            Ok(settings)
        }
        Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }
}

#[cfg(test)]
#[path = "lang_api_tests.rs"]
mod tests;
