use crate::parse::parse;
use crate::session::Sessions;
use crate::types::{CustomSentence, EntityRec, HomeGraph, ParseResult, Settings};
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
}

#[derive(Deserialize)]
pub struct ParseIn {
    pub text: String,
    pub conversation_id: Option<String>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/parse", post(api_parse))
        .route("/api/settings", get(get_settings).post(set_settings))
        .route("/api/custom", get(get_custom).post(set_custom))
        .route("/api/entities", get(get_entities).post(tag_entity))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn api_parse(State(state): State<AppState>, Json(body): Json<ParseIn>) -> Json<ParseResult> {
    let home = state.home.lock().await.clone();
    let settings = state.settings.lock().await.clone();
    let custom = state.custom.lock().await.clone();
    let mut sessions = state.sessions.lock().await;
    let session = sessions.get_or_create(body.conversation_id.as_deref());
    Json(parse(&body.text, &home, session, &custom, &settings))
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
    Json(state.home.lock().await.entities.clone())
}

#[derive(Deserialize)]
struct TagIn {
    entity_id: String,
    tags: Vec<String>,
}

async fn tag_entity(State(state): State<AppState>, Json(body): Json<TagIn>) -> Json<EntityRec> {
    let mut home = state.home.lock().await;
    if let Some(ent) = home
        .entities
        .iter_mut()
        .find(|e| e.entity_id == body.entity_id)
    {
        ent.tags = body.tags;
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
