use crate::home::paths::{read_to_string_confined, remove_confined, write_atomic_confined};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::state::AppState;
use crate::llm::{
    assist, assist_on, chat, chat_stream, generate_custom_voice, list_models, personality_preview_for, refine, refine_on, AssistRequest,
    ChatEvent, ChatRequest, CustomVoiceRequest, LlmEndpoint, LlmError, LlmPublic, PersonalityPreview, RefineRequest,
};
use axum::extract::{ConnectInfo, DefaultBodyLimit, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub(crate) const LLM_FILE: &str = "llm_endpoint.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StoredEndpoint {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub enable_thinking: bool,
}

pub(crate) fn load_stored_endpoint(dir: &Path) -> Option<StoredEndpoint> {
    let raw = read_to_string_confined(dir, LLM_FILE).ok()?;
    serde_json::from_str(&raw).ok()
}

pub(crate) fn save_stored_endpoint(dir: &Path, stored: &StoredEndpoint) -> std::io::Result<()> {
    write_atomic_confined(dir, LLM_FILE, &serde_json::to_vec_pretty(stored).unwrap_or_default())
}

pub fn load_endpoint(dir: &Path) -> Option<LlmEndpoint> {
    if let Some(from_env) = LlmEndpoint::from_env() {
        return Some(from_env);
    }
    from_file(dir)
}

fn from_file(dir: &Path) -> Option<LlmEndpoint> {
    let stored = load_stored_endpoint(dir)?;
    LlmEndpoint::from_parts(&stored.base_url, &stored.api_key, &stored.model)
        .ok()
        .map(|endpoint| endpoint.with_thinking(stored.enable_thinking))
}

fn save_endpoint(dir: &Path, endpoint: &LlmEndpoint) -> std::io::Result<()> {
    save_stored_endpoint(
        dir,
        &StoredEndpoint {
            base_url: endpoint.base_url.clone(),
            api_key: endpoint.api_key.clone(),
            model: endpoint.model.clone(),
            enable_thinking: endpoint.enable_thinking,
        },
    )
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
        .route("/api/v2/llm/voice", get(get_voice))
        .route("/api/v2/llm/custom-voice", post(make_custom_voice))
        .route("/api/v2/llm/models", post(list_endpoint_models))
        .route("/api/v2/llm/chat", post(llm_chat))
        .route("/api/v2/llm/refine", post(llm_refine))
        .route("/api/v2/llm/assist", post(llm_assist))
        .route("/api/v2/policies/trainer/chat", post(crate::io::trainer_chat::trainer_chat))
        .route("/api/v2/policies/trainer/consent", post(crate::io::trainer_chat::trainer_consent))
        .layer(DefaultBodyLimit::max(LLM_BODY_LIMIT))
}

#[derive(Debug, Deserialize)]
pub struct EndpointIn {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub configured: Option<bool>,
    pub enable_thinking: Option<bool>,
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

#[derive(Debug, Deserialize)]
pub struct VoiceQuery {
    pub personality: Option<String>,
    pub language: Option<String>,
}

async fn get_voice(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<VoiceQuery>,
) -> Result<Json<PersonalityPreview>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let language = query.language.as_deref().unwrap_or("de");
    let personality = query.personality.as_deref().unwrap_or("default");
    if language.chars().count() > 32 || language.chars().any(char::is_control) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if personality.chars().count() > 32 || personality.chars().any(char::is_control) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let custom = if personality.eq_ignore_ascii_case("custom") { state.settings.lock().await.custom_voice.clone() } else { String::new() };
    Ok(Json(personality_preview_for(language, personality, &custom)))
}

async fn make_custom_voice(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<CustomVoiceRequest>,
) -> Result<Json<crate::llm::CustomVoiceOut>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let endpoint = state.llm.lock().await.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    match generate_custom_voice(&endpoint, body).await {
        Ok(out) => Ok(Json(out)),
        Err(err) => Err(status_for(&err)),
    }
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
    let thinking = body.enable_thinking.unwrap_or_else(|| current.as_ref().map(|ep| ep.enable_thinking).unwrap_or(false));
    let endpoint = LlmEndpoint::from_parts(base_url, &api_key, model).map_err(|_| StatusCode::BAD_REQUEST)?.with_thinking(thinking);
    save_endpoint(&state.data_dir, &endpoint).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let public = endpoint.public();
    *current = Some(endpoint);
    Ok(Json(public))
}

#[derive(Debug, Default, Deserialize)]
pub struct ModelsIn {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelsOut {
    pub models: Vec<String>,
}

async fn list_endpoint_models(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<ModelsIn>,
) -> Result<Json<ModelsOut>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let (base_url, api_key) = {
        let current = state.llm.lock().await;
        let api_key = match body.api_key.as_deref() {
            Some(key) if !key.is_empty() => key.to_string(),
            _ => current.as_ref().map(|ep| ep.api_key.clone()).unwrap_or_default(),
        };
        let base_url = match body.base_url.as_deref() {
            Some(url) if !url.trim().is_empty() => url.to_string(),
            _ => current.as_ref().map(|ep| ep.base_url.clone()).unwrap_or_default(),
        };
        (base_url, api_key)
    };
    if base_url.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let endpoint = LlmEndpoint::for_discovery(&base_url, &api_key).map_err(|_| StatusCode::BAD_REQUEST)?;
    match list_models(&endpoint).await {
        Ok(models) => Ok(Json(ModelsOut { models })),
        Err(err) => Err(status_for(&err)),
    }
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
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();
        tokio::spawn(async move {
            let result = chat_stream(&endpoint, body, |delta| {
                let _ = tx.send(Ok(json_event(&ChatEvent::Delta { text: delta.to_string() })));
            })
            .await;
            match result {
                Ok(text) => {
                    let _ = tx.send(Ok(json_event(&ChatEvent::Done { text })));
                }
                Err(err) => {
                    let _ = tx.send(Ok(json_event(&ChatEvent::Error { message: err.to_string() })));
                }
            }
        });
        Ok(Sse::new(UnboundedReceiverStream::new(rx)).keep_alive(KeepAlive::default()).into_response())
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
    let mut body = body;
    if body.custom_voice.is_empty() {
        body.custom_voice = state.settings.lock().await.custom_voice.clone();
    }
    let stream = body.stream.unwrap_or(false);
    let conversation_id = body.conversation_id.clone();
    let endpoint = state.llm.lock().await.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let journal = state.journal.clone();
    if stream {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();
        tokio::spawn(async move {
            match refine_on(&endpoint, body, |event| {
                let _ = tx.send(Ok(json_event(event)));
            })
            .await
            {
                Ok(out) => {
                    if out.accepted {
                        journal.note_spoken(Some(&conversation_id), &out.text, "refine");
                    }
                    let _ = tx.send(Ok(json_data(&out)));
                }
                Err(err) => {
                    let _ = tx.send(Ok(json_event(&ChatEvent::Error { message: err.to_string() })));
                }
            }
        });
        Ok(Sse::new(UnboundedReceiverStream::new(rx)).keep_alive(KeepAlive::default()).into_response())
    } else {
        match refine(&endpoint, body).await {
            Ok(out) => {
                if out.accepted {
                    journal.note_spoken(Some(&conversation_id), &out.text, "refine");
                }
                Ok(Json(out).into_response())
            }
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
    let mut body = body;
    if body.custom_voice.as_ref().is_none_or(|voice| voice.is_empty()) {
        body.custom_voice = Some(state.settings.lock().await.custom_voice.clone());
    }
    let stream = body.stream.unwrap_or(true);
    let conversation_id = body.conversation_id.clone();
    let endpoint = state.llm.lock().await.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let journal = state.journal.clone();
    if stream {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();
        tokio::spawn(async move {
            let result = assist_on(&endpoint, body, |event| {
                let _ = tx.send(Ok(json_event(event)));
            })
            .await;
            match result {
                Ok(out) => {
                    if out.tool.is_none() {
                        journal.note_spoken(Some(&conversation_id), &out.text, "chat");
                    }
                }
                Err(err) => {
                    let _ = tx.send(Ok(json_event(&ChatEvent::Error { message: err.to_string() })));
                }
            }
        });
        Ok(Sse::new(UnboundedReceiverStream::new(rx)).keep_alive(KeepAlive::default()).into_response())
    } else {
        match assist(&endpoint, body).await {
            Ok(out) => {
                if out.tool.is_none() {
                    journal.note_spoken(Some(&conversation_id), &out.text, "chat");
                }
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
        assert!(!loaded.enable_thinking);
        assert!(!public.enable_thinking);
        clear_endpoint(&dir).unwrap();
        assert!(from_file(&dir).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn old_file_defaults_thinking_off_and_roundtrips_on() {
        let dir = temp_dir("legacy");
        std::fs::write(dir.join("llm_endpoint.json"), r#"{"base_url":"http://127.0.0.1:8000/v1","api_key":"k","model":"gemma"}"#).unwrap();
        let loaded = from_file(&dir).unwrap();
        assert!(!loaded.enable_thinking);
        save_endpoint(&dir, &loaded.clone().with_thinking(true)).unwrap();
        let again = from_file(&dir).unwrap();
        assert!(again.enable_thinking);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn assist_sse_forwards_deltas_live() {
        let src = include_str!("llm.rs");
        let start = src.find("pub async fn llm_assist").expect("llm_assist");
        let end = src.find("pub fn json_event").expect("json_event");
        let body = &src[start..end];
        assert!(body.contains("assist_on"));
        assert!(body.contains("unbounded_channel"));
        assert!(!body.contains("blocking_send"));
        assert!(!body.contains("try_send"));
        assert!(!body.contains("for event in out.events"));
    }

    #[test]
    fn custom_voice_route_requires_write_and_endpoint() {
        let src = include_str!("llm.rs");
        let start = src.find("async fn make_custom_voice").expect("make_custom_voice");
        let body = &src[start..start + 900];
        assert!(body.contains("writes_allowed"));
        assert!(body.contains("SERVICE_UNAVAILABLE"));
        assert!(body.contains("generate_custom_voice"));
    }

    #[test]
    fn clear_missing_file_is_ok() {
        let dir = temp_dir("missing");
        clear_endpoint(&dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
