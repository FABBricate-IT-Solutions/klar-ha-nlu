use crate::home::paths::{read_to_string_confined, remove_confined, write_atomic_confined};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::state::AppState;
use crate::llm::{
    assist, chat, chat_stream, refine, AssistRequest, ChatEvent, ChatRequest, LlmEndpoint, LlmError, LlmPublic, RefineRequest,
};
use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use tokio_stream::wrappers::ReceiverStream;

const LLM_FILE: &str = "llm_endpoint.json";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct StoredEndpoint {
    base_url: String,
    api_key: String,
    model: String,
}

pub fn load_endpoint(dir: &Path) -> Option<LlmEndpoint> {
    if let Some(from_env) = LlmEndpoint::from_env() {
        return Some(from_env);
    }
    from_file(dir)
}

fn from_file(dir: &Path) -> Option<LlmEndpoint> {
    let raw = read_to_string_confined(dir, LLM_FILE).ok()?;
    let stored: StoredEndpoint = serde_json::from_str(&raw).ok()?;
    LlmEndpoint::from_parts(&stored.base_url, &stored.api_key, &stored.model).ok()
}

fn save_endpoint(dir: &Path, endpoint: &LlmEndpoint) -> std::io::Result<()> {
    let stored = StoredEndpoint { base_url: endpoint.base_url.clone(), api_key: endpoint.api_key.clone(), model: endpoint.model.clone() };
    write_atomic_confined(dir, LLM_FILE, &serde_json::to_vec_pretty(&stored).unwrap_or_default())
}

fn clear_endpoint(dir: &Path) -> std::io::Result<()> {
    match remove_confined(dir, LLM_FILE) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

const LLM_BODY_LIMIT: usize = 256 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/llm/endpoint", get(get_endpoint).post(set_endpoint))
        .route("/api/v2/llm/chat", post(llm_chat))
        .route("/api/v2/llm/refine", post(llm_refine))
        .route("/api/v2/llm/assist", post(llm_assist))
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
        let _ = clear_endpoint(&state.data_dir);
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
    save_endpoint(&state.data_dir, &endpoint).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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

pub async fn llm_refine(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<RefineRequest>,
) -> Result<axum::response::Response, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let stream = body.stream.unwrap_or(false);
    let endpoint = state.llm.lock().await.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    if stream {
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(8);
        tokio::spawn(async move {
            match refine(&endpoint, body).await {
                Ok(out) => {
                    let _ = tx.send(Ok(json_data(&out))).await;
                }
                Err(err) => {
                    let _ = tx.send(Ok(json_event(&ChatEvent::Error { message: err.to_string() }))).await;
                }
            }
        });
        Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()).into_response())
    } else {
        match refine(&endpoint, body).await {
            Ok(out) => Ok(Json(out).into_response()),
            Err(err) => Err(status_for(&err)),
        }
    }
}

pub async fn llm_assist(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<AssistRequest>,
) -> Result<axum::response::Response, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let stream = body.stream.unwrap_or(true);
    let endpoint = state.llm.lock().await.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    if stream {
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(16);
        tokio::spawn(async move {
            match assist(&endpoint, body).await {
                Ok(out) => {
                    for event in out.events {
                        let _ = tx.send(Ok(json_event(&event))).await;
                    }
                }
                Err(err) => {
                    let _ = tx.send(Ok(json_event(&ChatEvent::Error { message: err.to_string() }))).await;
                }
            }
        });
        Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()).into_response())
    } else {
        match assist(&endpoint, body).await {
            Ok(out) => {
                if let Some(tool) = out.tool {
                    Ok(Json(tool.event()).into_response())
                } else {
                    Ok(Json(ChatEvent::Done { text: out.text }).into_response())
                }
            }
            Err(err) => Err(status_for(&err)),
        }
    }
}

pub fn json_event(event: &ChatEvent) -> Event {
    json_data(event)
}

fn json_data(event: &impl serde::Serialize) -> Event {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("klar-llm-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn persists_endpoint_and_keeps_key_off_public() {
        let dir = temp_dir("save");
        let endpoint = LlmEndpoint::from_parts("http://127.0.0.1:11434/v1", "sk-secret", "llama3").unwrap();
        save_endpoint(&dir, &endpoint).unwrap();
        let loaded = from_file(&dir).unwrap();
        assert_eq!(loaded.model, "llama3");
        assert_eq!(loaded.api_key, "sk-secret");
        assert_eq!(loaded.base_url, "http://127.0.0.1:11434/v1");
        let public = loaded.public();
        let json = serde_json::to_string(&public).unwrap();
        assert!(!json.contains("sk-secret"));
        assert!(json.contains("llama3"));
        clear_endpoint(&dir).unwrap();
        assert!(from_file(&dir).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clear_missing_file_is_ok() {
        let dir = temp_dir("missing");
        clear_endpoint(&dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
