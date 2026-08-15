use crate::home::expose::assist_visible;
use crate::home::gaps::leftover;
use crate::home::overlay::{apply_overlay, load_overlay, save_overlay, Overlay};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::state::AppState;
use crate::parse::parse;
use crate::types::{known_intent, AreaRec, CustomSentence, EntityRec, Settings};
use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;

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
        .layer(DefaultBodyLimit::max(16 * 1024))
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../../web/index.html"))
}

fn gate(peer: SocketAddr, headers: &HeaderMap, token: &Option<String>) -> Result<(), StatusCode> {
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

async fn api_parse(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<ParseIn>,
) -> Result<Json<ParseOut>, StatusCode> {
    read_gate(peer, &headers, &state.token)?;
    if body.text.chars().count() > MAX_PARSE_CHARS {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let home = state.home.snapshot().await;
    let settings = settings_for_parse(state.settings.lock().await.clone(), body.language.as_deref(), body.personality);
    let custom = state.custom.lock().await.clone();
    let mut session = {
        let mut sessions = state.sessions.lock().await;
        sessions.take(body.conversation_id.as_deref())
    };
    let result = parse(&body.text, &home, &mut session, &custom, &settings);
    state.sessions.lock().await.put(session);
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
    read_gate(peer, &headers, &state.token)?;
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
    read_gate(peer, &headers, &state.token)?;
    Ok(Json(state.custom.lock().await.clone()))
}

async fn set_custom(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<Vec<CustomSentence>>,
) -> Result<Json<Vec<CustomSentence>>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    if body.len() > 64
        || body.iter().any(|row| row.phrase.len() > 200 || row.phrase.trim().chars().count() < 4 || !known_intent(&row.intent))
    {
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
    read_gate(peer, &headers, &state.token)?;
    let home = state.home.snapshot().await;
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
    read_gate(peer, &headers, &state.token)?;
    let home = state.home.snapshot().await;
    Ok(Json(GapsOut { leftover: leftover(&home), rooms: home.areas.clone(), overlay: load_overlay(&state.data_dir) }))
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
    let home = state.home.snapshot().await;
    if !home.entities.iter().any(|e| e.entity_id == body.entity_id) {
        return Err(StatusCode::NOT_FOUND);
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
    let updated = state
        .home
        .edit(|next| {
            apply_overlay(next, &overlay);
            let ent = next.entities.iter_mut().find(|e| e.entity_id == body.entity_id)?;
            if !body.aliases.is_empty() {
                ent.aliases = body.aliases;
            }
            let mut tags = ent.tags.clone();
            tags.retain(|t| t != "preferred");
            if body.preferred || body.tags.iter().any(|t| t == "preferred") {
                tags.push("preferred".into());
            }
            ent.tags = tags;
            Some(ent.clone())
        })
        .await;
    updated.map(Json).ok_or(StatusCode::NOT_FOUND)
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
        assert!(!reads_allowed(Some(peer), &HeaderMap::new(), &Some("secret".into())));
        let mut headers = HeaderMap::new();
        headers.insert("x-klar-token", "secret".parse().unwrap());
        assert!(reads_allowed(Some(peer), &headers, &Some("secret".into())));
    }

    #[test]
    fn supervisor_parse_without_token() {
        let core = "172.30.32.1:9".parse().unwrap();
        let addon = "172.30.33.4:9".parse().unwrap();
        assert!(reads_allowed(Some(core), &HeaderMap::new(), &None));
        assert!(reads_allowed(Some(addon), &HeaderMap::new(), &Some("secret".into())));
        assert!(!writes_allowed(Some(core), &HeaderMap::new(), &None));
        assert!(!reads_allowed(Some("10.0.0.8:9".parse().unwrap()), &HeaderMap::new(), &None));
    }

    #[test]
    fn entity_id_shape() {
        assert!(valid_entity_id("light.kuche"));
        assert!(!valid_entity_id("light"));
        assert!(!valid_entity_id("../etc/passwd"));
        assert!(!valid_entity_id("light.kuche.extra"));
    }
}
