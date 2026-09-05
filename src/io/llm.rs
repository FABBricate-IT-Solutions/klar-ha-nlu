use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::state::AppState;
use crate::llm::{chat, chat_stream, ChatEvent, ChatRequest, LlmEndpoint, LlmError, LlmPublic};
use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio_stream::wrappers::ReceiverStream;

const LLM_BODY_LIMIT: usize = 256 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/llm/endpoint", get(get_endpoint).post(set_endpoint))
        .route("/api/v2/llm/chat", post(llm_chat))
        .route("/api/v2/policies/trainer/chat", post(crate::io::trainer_chat::trainer_chat))
        .layer(DefaultBodyLimit::max(LLM_BODY_LIMIT))
}

#[derive(Debug, Deserialize)]
pub struct EndpointIn {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub configured: Option<bool>,
}

async fn get_endpoint(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<LlmPublic>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(state.llm.lock().await.as_ref().map(LlmEndpoint::public).unwrap_or_else(LlmPublic::empty)))
}

async fn set_endpoint(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<EndpointIn>,
) -> Result<Json<LlmPublic>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if body.configured == Some(false) {
        *state.llm.lock().await = None;
        return Ok(Json(LlmPublic::empty()));
    }
    let mut current = state.llm.lock().await;
    let api_key = match body.api_key.as_deref() {
        Some(key) if !key.is_empty() => key.to_string(),
        _ => current.as_ref().map(|ep| ep.api_key.clone()).unwrap_or_default(),
    };
    let base_url = body.base_url.as_deref().unwrap_or("");
    let model = body.model.as_deref().unwrap_or("");
    let endpoint = LlmEndpoint::from_parts(base_url, &api_key, model).map_err(|_| StatusCode::BAD_REQUEST)?;
    let public = endpoint.public();
    *current = Some(endpoint);
    Ok(Json(public))
}

pub async fn llm_chat(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<ChatRequest>,
) -> Result<axum::response::Response, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let endpoint = state.llm.lock().await.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let stream = body.stream.unwrap_or(true);
    if stream {
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(64);
        tokio::spawn(async move {
            let result = chat_stream(&endpoint, body, |delta| {
                let _ = tx.try_send(Ok(json_event(&ChatEvent::Delta { text: delta.to_string() })));
            })
            .await;
            match result {
                Ok(text) => {
                    let _ = tx.send(Ok(json_event(&ChatEvent::Done { text }))).await;
                }
                Err(err) => {
                    let _ = tx.send(Ok(json_event(&ChatEvent::Error { message: err.to_string() }))).await;
                }
            }
        });
        Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()).into_response())
    } else {
        match chat(&endpoint, body).await {
            Ok(text) => Ok(Json(ChatEvent::Done { text }).into_response()),
            Err(err) => Err(status_for(&err)),
        }
    }
}

pub fn json_event(event: &ChatEvent) -> Event {
    Event::default().json_data(event).unwrap_or_else(|_| Event::default().data("{}"))
}

pub fn status_for(err: &LlmError) -> StatusCode {
    match err {
        LlmError::NotConfigured => StatusCode::SERVICE_UNAVAILABLE,
        LlmError::InvalidEndpoint(_) | LlmError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
        LlmError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        LlmError::Upstream(429) => StatusCode::TOO_MANY_REQUESTS,
        LlmError::Upstream(_) | LlmError::Transport | LlmError::Response => StatusCode::BAD_GATEWAY,
    }
}
