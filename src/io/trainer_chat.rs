//! Trainer chat streams through the engine LLM client, then validates JSON.

use crate::io::auth::writes_allowed;
use crate::io::llm::json_event;
use crate::io::state::AppState;
use crate::io::trainer::{load_context, validate, ProposalIn, TrainerQuery};
use crate::llm::{chat_stream, history_messages, json_object, system_prompt, ChatEvent, ChatMessage, ChatRequest, TrainerTurn};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug, Deserialize)]
pub struct TrainerChatIn {
    pub message: String,
    pub layer: Option<String>,
    pub language: Option<String>,
    #[serde(default)]
    pub history: Vec<TrainerTurn>,
}

pub async fn trainer_chat(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<TrainerChatIn>,
) -> Result<axum::response::Response, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if body.message.trim().is_empty() || body.message.chars().count() > 4000 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let endpoint = state.llm.lock().await.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let ctx = load_context(&state, &TrainerQuery { layer: body.layer.clone(), language: body.language.clone() }).await?;
    let context_json = serde_json::to_string(&ctx).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut messages = vec![ChatMessage { role: "system".into(), content: system_prompt(&context_json) }];
    messages.extend(history_messages(&body.history).map_err(|_| StatusCode::BAD_REQUEST)?);
    messages.push(ChatMessage { role: "user".into(), content: body.message });
    let request = ChatRequest { messages, stream: Some(true), temperature: Some(0.2), max_tokens: Some(2048) };
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<axum::response::sse::Event, Infallible>>(64);
    let home = state.home.snapshot().await;
    let settings = state.settings.lock().await.clone();
    let speech_bank = state.speech_bank.lock().await.clone();
    tokio::spawn(async move {
        let result = chat_stream(&endpoint, request, |delta| {
            let _ = tx.try_send(Ok(json_event(&ChatEvent::Delta { text: delta.to_string() })));
        })
        .await;
        match result {
            Ok(text) => {
                if let Some(raw) = json_object(&text) {
                    if let Ok(proposal) = serde_json::from_str::<ProposalIn>(raw) {
                        if let Ok(value) = serde_json::to_value(&proposal) {
                            let _ = tx.send(Ok(json_event(&ChatEvent::Proposal { value }))).await;
                        }
                        let language = proposal.language.clone().unwrap_or_else(|| ctx.language.clone());
                        let layer = proposal.layer.clone().unwrap_or_else(|| ctx.layer.clone());
                        let house = proposal.policies.clone().unwrap_or_else(|| ctx.overlays.policies.clone());
                        let match_controls = proposal.match_controls.clone().unwrap_or_else(|| ctx.overlays.match_controls.clone());
                        let overlay = proposal.language_overlay.clone().unwrap_or_else(|| ctx.overlays.language.clone());
                        let extra = proposal.utterances.clone().unwrap_or_default();
                        let out = validate(&home, &settings, &language, &layer, house, match_controls, overlay, &speech_bank, &extra);
                        if let Ok(value) = serde_json::to_value(&out) {
                            let _ = tx.send(Ok(json_event(&ChatEvent::Validate { value }))).await;
                        }
                    }
                }
                let _ = tx.send(Ok(json_event(&ChatEvent::Done { text }))).await;
            }
            Err(err) => {
                let _ = tx.send(Ok(json_event(&ChatEvent::Error { message: err.to_string() }))).await;
            }
        }
    });
    Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()).into_response())
}
