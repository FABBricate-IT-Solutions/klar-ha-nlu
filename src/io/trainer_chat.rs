//! Trainer chat streams through the engine LLM client, then runs tools with consent.

use crate::io::auth::writes_allowed;
use crate::io::llm::json_event;
use crate::io::state::AppState;
use crate::io::trainer::{context_stub, load_context, TrainerQuery};
use crate::io::trainer_apply::{dispatch, preview_write, write_summary};
use crate::io::trainer_consent::{ConsentDecision, PendingWrite, TrainerConsentHub};
use crate::llm::{
    chat_stream_turn, history_messages, is_write_tool, openai_tools, parse_text_tools, system_prompt, ChatEvent, ChatMessage, ChatRequest,
    CompletionTurn, LlmEndpoint, TrainerTurn,
};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

const MAX_ROUNDS: usize = 8;

#[derive(Debug, Deserialize)]
pub struct TrainerChatIn {
    pub message: String,
    pub layer: Option<String>,
    pub language: Option<String>,
    #[serde(default)]
    pub history: Vec<TrainerTurn>,
}

#[derive(Debug, Deserialize)]
pub struct TrainerConsentIn {
    pub call_id: Option<String>,
    pub decision: ConsentDecision,
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
    let settings = state.settings.lock().await.clone();
    let stub = context_stub(&ctx, &settings.languages);
    let mut messages = vec![ChatMessage::new("system", system_prompt(&stub))];
    messages.extend(history_messages(&body.history).map_err(|_| StatusCode::BAD_REQUEST)?);
    messages.push(ChatMessage::new("user", body.message));
    let session = TrainerConsentHub::session_key(&state.token, peer);
    let (tx, rx) = mpsc::unbounded_channel::<Result<axum::response::sse::Event, Infallible>>();
    tokio::spawn(async move {
        run_loop(state, endpoint, session, messages, tx).await;
    });
    Ok(Sse::new(UnboundedReceiverStream::new(rx)).keep_alive(KeepAlive::default()).into_response())
}

pub async fn trainer_consent(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<TrainerConsentIn>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let key = TrainerConsentHub::session_key(&state.token, peer);
    let call_id = body.call_id.unwrap_or_default();
    state.trainer_consent.decide(&key, &call_id, body.decision).await.map_err(|_| StatusCode::NOT_FOUND)?;
    let (yolo, allowed) = state.trainer_consent.snapshot(&key).await;
    Ok(Json(serde_json::json!({ "ok": true, "yolo": yolo, "allowed": allowed })))
}

async fn run_loop(
    state: AppState,
    endpoint: LlmEndpoint,
    session: String,
    mut messages: Vec<ChatMessage>,
    tx: UnboundedSender<Result<axum::response::sse::Event, Infallible>>,
) {
    let (yolo, allowed) = state.trainer_consent.snapshot(&session).await;
    let _ = tx.send(Ok(json_event(&ChatEvent::Session { yolo, allowed })));
    for _ in 0..MAX_ROUNDS {
        let request = ChatRequest {
            messages: messages.clone(),
            stream: Some(true),
            temperature: Some(0.2),
            max_tokens: Some(2048),
            tools: Some(openai_tools()),
            tool_choice: None,
        };
        let turn = match chat_stream_turn(&endpoint, request, |delta| {
            let _ = tx.send(Ok(json_event(&ChatEvent::Delta { text: delta.to_string() })));
        })
        .await
        {
            Ok(turn) => turn,
            Err(err) => {
                let _ = tx.send(Ok(json_event(&ChatEvent::Error { message: err.to_string() })));
                return;
            }
        };
        let (prose, calls) = merge_calls(turn);
        if calls.is_empty() {
            let _ = tx.send(Ok(json_event(&ChatEvent::Done { text: prose })));
            return;
        }
        messages.push(ChatMessage::assistant_tools(prose, calls.clone()));
        for call in calls {
            let result = handle_call(&state, &session, &tx, &call).await;
            messages.push(ChatMessage::tool(&call.id, result));
        }
    }
    let _ = tx.send(Ok(json_event(&ChatEvent::Done { text: String::new() })));
}

fn merge_calls(turn: CompletionTurn) -> (String, Vec<crate::llm::ToolCall>) {
    let (prose, text_calls) = parse_text_tools(&turn.text);
    let mut calls = turn.tool_calls;
    if calls.is_empty() {
        calls = text_calls;
    }
    (prose, calls)
}

async fn handle_call(
    state: &AppState,
    session: &str,
    tx: &UnboundedSender<Result<axum::response::sse::Event, Infallible>>,
    call: &crate::llm::ToolCall,
) -> String {
    let args = serde_json::from_str(&call.function.arguments).unwrap_or(serde_json::json!({}));
    if !is_write_tool(&call.function.name) {
        return match dispatch(state, &call.function.name, &args).await {
            Ok(value) => value.to_string(),
            Err(err) => serde_json::json!({"error": err}).to_string(),
        };
    }
    let preview = match preview_write(state, &call.function.name, &args).await {
        Ok(value) => value,
        Err(err) => return serde_json::json!({"error": err}).to_string(),
    };
    if preview.get("ok") != Some(&serde_json::json!(true)) {
        return serde_json::json!({"error": "validate failed", "validate": preview}).to_string();
    }
    if !state.trainer_consent.allows(session, &call.function.name).await {
        let pending = PendingWrite {
            name: call.function.name.clone(),
            args: args.clone(),
            summary: write_summary(&call.function.name, &args),
            preview: preview.clone(),
        };
        let _ = tx.send(Ok(json_event(&ChatEvent::Consent {
            call_id: call.id.clone(),
            tool: call.function.name.clone(),
            summary: pending.summary.clone(),
            validate: preview,
        })));
        let decision = state.trainer_consent.wait(session, call.id.clone(), pending).await;
        match decision {
            ConsentDecision::Deny | ConsentDecision::AskAgain => return serde_json::json!({"error": "denied"}).to_string(),
            ConsentDecision::AllowOnce | ConsentDecision::Allow | ConsentDecision::Yolo => {}
        }
        let (yolo, allowed) = state.trainer_consent.snapshot(session).await;
        let _ = tx.send(Ok(json_event(&ChatEvent::Session { yolo, allowed })));
    }
    match dispatch(state, &call.function.name, &args).await {
        Ok(value) => value.to_string(),
        Err(err) => serde_json::json!({"error": err}).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ToolCall;

    #[test]
    fn text_fallback_fills_empty_tool_calls() {
        let turn = CompletionTurn { text: "TRAINER_TOOL: list_gaps {}".into(), tool_calls: Vec::new() };
        let (prose, calls) = merge_calls(turn);
        assert!(prose.is_empty());
        assert_eq!(calls[0].function.name, "list_gaps");
        let native = merge_calls(CompletionTurn {
            text: "ok".into(),
            tool_calls: vec![ToolCall::function("c1", "get_entity", r#"{"entity_id":"light.x"}"#)],
        });
        assert_eq!(native.0, "ok");
        assert_eq!(native.1[0].id, "c1");
    }
}
