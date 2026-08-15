use crate::compound::{apply_overlay, load_overlay, save_overlay, Overlay};
use crate::gaps::{assist_visible, leftover};
use crate::parse::parse;
use crate::session::Sessions;
use crate::types::{AreaRec, CustomSentence, EntityRec, HomeGraph, ParseResult, Settings};
use std::path::PathBuf;
use axum::extract::State;
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub home: Arc<Mutex<HomeGraph>>,
    pub sessions: Arc<Mutex<Sessions>>,
    pub settings: Arc<Mutex<Settings>>,
    pub custom: Arc<Mutex<Vec<CustomSentence>>>,
    pub data_dir: PathBuf,
}

#[derive(Deserialize)]
pub struct ParseIn {
    pub text: String,
    pub conversation_id: Option<String>,
    /// BCP-47 tag from Assist (`de`, `en-US`, …). Pins the pack for this request.
    pub language: Option<String>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/parse", post(api_parse))
        .route("/api/settings", get(get_settings).post(set_settings))
        .route("/api/custom", get(get_custom).post(set_custom))
        .route("/api/entities", get(get_entities).post(tag_entity))
        .route("/api/gaps", get(get_gaps))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn api_parse(State(state): State<AppState>, Json(body): Json<ParseIn>) -> Json<ParseResult> {
    let home = state.home.lock().await.clone();
    let settings = settings_for_parse(
        state.settings.lock().await.clone(),
        body.language.as_deref(),
    );
    let custom = state.custom.lock().await.clone();
    let mut sessions = state.sessions.lock().await;
    let session = sessions.get_or_create(body.conversation_id.as_deref());
    Json(parse(&body.text, &home, session, &custom, &settings))
}

fn settings_for_parse(mut settings: Settings, language: Option<&str>) -> Settings {
    let Some(raw) = language.filter(|s| !s.is_empty()) else {
        return settings;
    };
    let code = raw
        .split(['-', '_'])
        .next()
        .unwrap_or(raw)
        .to_ascii_lowercase();
    if code == "de" || code == "en" {
        settings.languages = vec![code];
    }
    settings
}

async fn get_settings(State(state): State<AppState>) -> Json<Settings> {
    Json(state.settings.lock().await.clone())
}

async fn set_settings(State(state): State<AppState>, Json(body): Json<Settings>) -> Json<Settings> {
    *state.settings.lock().await = body.clone();
    Json(body)
}

async fn get_custom(State(state): State<AppState>) -> Json<Vec<CustomSentence>> {
    Json(state.custom.lock().await.clone())
}

async fn set_custom(
    State(state): State<AppState>,
    Json(body): Json<Vec<CustomSentence>>,
) -> Json<Vec<CustomSentence>> {
    *state.custom.lock().await = body.clone();
    Json(body)
}

async fn get_entities(State(state): State<AppState>) -> Json<Vec<EntityRec>> {
    let home = state.home.lock().await;
    Json(
        home.entities
            .iter()
            .filter(|e| assist_visible(e, &home))
            .cloned()
            .collect(),
    )
}

#[derive(serde::Serialize)]
struct GapsOut {
    leftover: Vec<EntityRec>,
    rooms: Vec<AreaRec>,
    overlay: Overlay,
}

async fn get_gaps(State(state): State<AppState>) -> Json<GapsOut> {
    let home = state.home.lock().await.clone();
    Json(GapsOut {
        leftover: leftover(&home),
        rooms: home.areas,
        overlay: load_overlay(&state.data_dir),
    })
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

async fn tag_entity(State(state): State<AppState>, Json(body): Json<TagIn>) -> Json<EntityRec> {
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
        return Json(ent.clone());
    }
    Json(EntityRec {
        entity_id: body.entity_id,
        name: String::new(),
        domain: String::new(),
        area: None,
        aliases: Vec::new(),
        tags: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_assist_language_to_pack() {
        let base = Settings::default();
        assert_eq!(
            settings_for_parse(base.clone(), Some("en-US")).languages,
            vec!["en".to_string()]
        );
        assert_eq!(
            settings_for_parse(base.clone(), Some("de-DE")).languages,
            vec!["de".to_string()]
        );
        assert_eq!(
            settings_for_parse(base.clone(), None).languages,
            vec!["de".to_string(), "en".to_string()]
        );
        assert_eq!(
            settings_for_parse(base, Some("fr")).languages,
            vec!["de".to_string(), "en".to_string()]
        );
    }
}
