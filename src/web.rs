use crate::compound::{apply_overlay, load_overlay, save_overlay, Overlay};
use crate::gaps::{assist_visible, leftover};
use crate::parse::parse;
use crate::session::Sessions;
use crate::types::{AreaRec, CustomSentence, EntityRec, HomeGraph, Settings};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub const MAX_PARSE_CHARS: usize = 4096;

#[derive(Clone)]
pub struct AppState {
    pub home: Arc<Mutex<HomeGraph>>,
    pub sessions: Arc<Mutex<Sessions>>,
    pub settings: Arc<Mutex<Settings>>,
    pub custom: Arc<Mutex<Vec<CustomSentence>>>,
    pub data_dir: PathBuf,
    pub token: Option<String>,
}

#[derive(Deserialize)]
pub struct ParseIn {
    pub text: String,
    pub conversation_id: Option<String>,
    /// BCP-47 tag from Assist (`de`, `en-US`, …). Pins the pack for this request.
    pub language: Option<String>,
    /// Home Assistant option. Wins over the addon overlay for this request.
    pub personality: Option<crate::types::Personality>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/parse", post(api_parse))
        .route("/api/settings", get(get_settings).post(set_settings))
        .route("/api/custom", get(get_custom).post(set_custom))
        .route("/api/entities", get(get_entities).post(tag_entity))
        .route("/api/gaps", get(get_gaps))
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

fn request_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-klar-token")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .or_else(|| headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")))
}

pub fn writes_allowed(peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> bool {
    if peer.is_some_and(|addr| addr.ip().is_loopback()) {
        return true;
    }
    let Some(expected) = token.as_deref().filter(|s| !s.is_empty()) else {
        return false;
    };
    request_token(headers) == Some(expected)
}

fn gate(peer: SocketAddr, headers: &HeaderMap, token: &Option<String>) -> Result<(), StatusCode> {
    if writes_allowed(Some(peer), headers, token) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn api_parse(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<ParseIn>,
) -> Result<Json<ParseOut>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    if body.text.chars().count() > MAX_PARSE_CHARS {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let home = state.home.lock().await.clone();
    let settings = settings_for_parse(state.settings.lock().await.clone(), body.language.as_deref(), body.personality);
    let custom = state.custom.lock().await.clone();
    let mut sessions = state.sessions.lock().await;
    let session = sessions.get_or_create(body.conversation_id.as_deref());
    let result = parse(&body.text, &home, session, &custom, &settings);
    Ok(Json(ParseOut { personality: settings.personality, result }))
}

#[derive(serde::Serialize)]
struct ParseOut {
    #[serde(flatten)]
    result: crate::types::ParseResult,
    personality: crate::types::Personality,
}

fn settings_for_parse(mut settings: Settings, language: Option<&str>, personality: Option<crate::types::Personality>) -> Settings {
    if let Some(personality) = personality {
        settings.personality = personality;
    }
    let Some(raw) = language.filter(|s| !s.is_empty()) else {
        return settings;
    };
    let code = raw.split(['-', '_']).next().unwrap_or(raw).to_ascii_lowercase();
    if code == "de" || code == "en" {
        settings.languages = vec![code];
    }
    settings
}

async fn get_settings(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Settings>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    Ok(Json(state.settings.lock().await.clone()))
}

async fn set_settings(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<Settings>,
) -> Result<Json<Settings>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    *state.settings.lock().await = body.clone();
    let mut overlay = load_overlay(&state.data_dir);
    overlay.settings = Some(body.clone());
    let _ = save_overlay(&state.data_dir, &overlay);
    Ok(Json(body))
}

async fn get_custom(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Vec<CustomSentence>>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    Ok(Json(state.custom.lock().await.clone()))
}

async fn set_custom(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<Vec<CustomSentence>>,
) -> Result<Json<Vec<CustomSentence>>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    if body.len() > 64 || body.iter().any(|row| row.phrase.len() > 200 || row.intent.len() > 64) {
        return Err(StatusCode::BAD_REQUEST);
    }
    *state.custom.lock().await = body.clone();
    let mut overlay = load_overlay(&state.data_dir);
    overlay.custom = body.clone();
    let _ = save_overlay(&state.data_dir, &overlay);
    Ok(Json(body))
}

async fn get_entities(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Vec<EntityRec>>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    let home = state.home.lock().await;
    Ok(Json(home.entities.iter().filter(|e| assist_visible(e, &home)).cloned().collect()))
}

#[derive(serde::Serialize)]
struct GapsOut {
    leftover: Vec<EntityRec>,
    rooms: Vec<AreaRec>,
    overlay: Overlay,
}

async fn get_gaps(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<GapsOut>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    let home = state.home.lock().await.clone();
    Ok(Json(GapsOut { leftover: leftover(&home), rooms: home.areas, overlay: load_overlay(&state.data_dir) }))
}

#[derive(Deserialize)]
struct TagIn {
    entity_id: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    preferred: bool,
    pub area: Option<String>,
}

fn valid_entity_id(id: &str) -> bool {
    let mut parts = id.split('.');
    matches!((parts.next(), parts.next(), parts.next()), (Some(d), Some(n), None) if !d.is_empty() && !n.is_empty() && id.len() <= 128)
}

async fn tag_entity(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<TagIn>,
) -> Result<Json<EntityRec>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    if !valid_entity_id(&body.entity_id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut overlay = load_overlay(&state.data_dir);
    overlay.aliases.insert(body.entity_id.clone(), body.aliases.clone());
    if let Some(area) = &body.area {
        overlay.areas.insert(body.entity_id.clone(), area.clone());
    }
    overlay.preferred.retain(|id| id != &body.entity_id);
    if body.preferred {
        overlay.preferred.push(body.entity_id.clone());
    }
    let _ = save_overlay(&state.data_dir, &overlay);
    let mut home = state.home.lock().await;
    if !home.entities.iter().any(|e| e.entity_id == body.entity_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    apply_overlay(&mut home, &overlay);
    if let Some(ent) = home.entities.iter_mut().find(|e| e.entity_id == body.entity_id) {
        if !body.aliases.is_empty() {
            ent.aliases = body.aliases;
        }
        let mut tags = ent.tags.clone();
        tags.retain(|t| t != "preferred");
        if body.preferred || body.tags.iter().any(|t| t == "preferred") {
            tags.push("preferred".into());
        }
        ent.tags = tags;
        return Ok(Json(ent.clone()));
    }
    Err(StatusCode::NOT_FOUND)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_assist_language_to_pack() {
        let base = Settings::default();
        assert_eq!(settings_for_parse(base.clone(), Some("en-US"), None).languages, vec!["en".to_string()]);
        assert_eq!(settings_for_parse(base.clone(), Some("de-DE"), None).languages, vec!["de".to_string()]);
        assert_eq!(settings_for_parse(base.clone(), None, None).languages, vec!["de".to_string(), "en".to_string()]);
        assert_eq!(settings_for_parse(base, Some("fr"), None).languages, vec!["de".to_string(), "en".to_string()]);
    }

    #[test]
    fn assist_personality_overrides_overlay() {
        let out = settings_for_parse(Settings::default(), None, Some(crate::types::Personality::Butler));
        assert_eq!(out.personality, crate::types::Personality::Butler);
    }

    #[test]
    fn loopback_writes_without_token() {
        let peer = "127.0.0.1:9".parse().unwrap();
        assert!(writes_allowed(Some(peer), &HeaderMap::new(), &None));
    }

    #[test]
    fn lan_writes_need_token() {
        let peer = "10.0.0.8:9".parse().unwrap();
        assert!(!writes_allowed(Some(peer), &HeaderMap::new(), &None));
        let mut headers = HeaderMap::new();
        headers.insert("x-klar-token", "secret".parse().unwrap());
        assert!(writes_allowed(Some(peer), &headers, &Some("secret".into())));
        assert!(!writes_allowed(Some(peer), &headers, &Some("other".into())));
    }

    #[test]
    fn lan_parse_needs_token() {
        let peer = "10.0.0.8:9".parse().unwrap();
        assert!(!writes_allowed(Some(peer), &HeaderMap::new(), &Some("secret".into())));
        let mut headers = HeaderMap::new();
        headers.insert("x-klar-token", "secret".parse().unwrap());
        assert!(writes_allowed(Some(peer), &headers, &Some("secret".into())));
    }

    #[test]
    fn entity_id_shape() {
        assert!(valid_entity_id("light.kuche"));
        assert!(!valid_entity_id("light"));
        assert!(!valid_entity_id("../etc/passwd"));
        assert!(!valid_entity_id("light.kuche.extra"));
    }
}
