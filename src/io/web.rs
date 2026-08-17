use crate::home::expose::assist_visible;
use crate::home::gaps::leftover;
use crate::home::overlay::{apply_overlay, load_overlay, save_overlay, Overlay};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::state::AppState;
use crate::nlu::{legacy_result, parse_with_policies};
use crate::types::{known_intent, AreaRec, CustomSentence, EntityRec, ParseOutcome, Settings};
use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParseIn {
    pub text: String,
    pub conversation_id: Option<String>,
    /// BCP-47 tag from Assist (`de`, `en-US`, …). Pins the pack for this request.
    pub language: Option<String>,
    /// Home Assistant option. Wins over the addon overlay for this request.
    pub personality: Option<crate::types::Personality>,
    /// Area of the Assist satellite that heard the request.
    pub preferred_area: Option<String>,
    /// Opt-in NLU-as-RAG for this request. Overlay default stays off.
    pub nlu_rag: Option<bool>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/v2/parse", post(api_parse))
        .route("/api/settings", get(get_settings).post(set_settings))
        .route("/api/custom", get(get_custom).post(set_custom))
        .route("/api/entities", get(get_entities).post(tag_entity))
        .route("/api/gaps", get(get_gaps))
        .merge(crate::io::bundle::routes())
        .merge(crate::io::dashboard::routes())
        .merge(crate::io::home_sync::routes())
        .merge(crate::io::lang_api::routes())
        .merge(crate::io::conversations::routes())
        .merge(crate::io::policies::routes())
        .layer(DefaultBodyLimit::max(16 * 1024))
        .fallback_service(ServeDir::new(ui_dir()))
        .with_state(state)
}

async fn index() -> Html<String> {
    Html(std::fs::read_to_string(ui_dir().join("index.html")).unwrap_or_else(|_| {
        r#"<!doctype html><html><body style="background:#100e0c;color:#f3eee4;font:16px sans-serif;padding:32px">
        <h1>Klar UI nicht gebaut</h1><p>Bitte im web-Verzeichnis <code>npm run build</code> ausführen.</p>
        </body></html>"#
            .into()
    }))
}

fn ui_dir() -> PathBuf {
    std::env::var("KLAR_UI_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        let packaged = PathBuf::from("/usr/share/klar/ui");
        if packaged.is_dir() {
            packaged
        } else {
            PathBuf::from("web/dist")
        }
    })
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
) -> Result<Json<ParseOutcome>, StatusCode> {
    read_gate(peer, &headers, &state.token)?;
    if body.text.chars().count() > MAX_PARSE_CHARS {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    if body.text.chars().any(|character| character.is_control() && !character.is_whitespace())
        || body.language.as_ref().is_some_and(|language| language.len() > 35)
        || body.conversation_id.as_ref().is_some_and(|conversation_id| conversation_id.len() > 128)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let home = state.home.snapshot().await;
    if body.preferred_area.as_ref().is_some_and(|area| area.len() > 128 || !home.areas.iter().any(|record| record.area_id == *area)) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let settings = settings_for_parse(state.settings.lock().await.clone(), body.language.as_deref(), body.personality, body.nlu_rag)?;
    let custom = state.custom.lock().await.clone();
    let policies = state.policies.lock().await.clone();
    let speech_bank = state.speech_bank.lock().await.clone();
    let mut session = {
        let mut sessions = state.sessions.lock().await;
        sessions.take(body.conversation_id.as_deref())
    };
    session.preferred_area = body.preferred_area.clone();
    let outcome = parse_with_policies(&body.text, &home, &mut session, &custom, &settings, &policies, &speech_bank);
    let last_names = session.last.iter().map(|turn| turn.name.clone()).collect();
    state.sessions.lock().await.put(session);
    state.record_parse("http", body.language.as_deref(), &legacy_result(outcome.clone())).await;
    state.record_outcome(&outcome, last_names).await;
    Ok(Json(outcome))
}

fn settings_for_parse(
    mut settings: Settings,
    language: Option<&str>,
    personality: Option<crate::types::Personality>,
    nlu_rag: Option<bool>,
) -> Result<Settings, StatusCode> {
    if let Some(personality) = personality {
        settings.personality = personality;
    }
    if let Some(nlu_rag) = nlu_rag {
        settings.nlu_rag = nlu_rag;
    }
    let Some(raw) = language.filter(|s| !s.is_empty()) else {
        return Ok(settings);
    };
    match crate::lang::pin_language(raw) {
        Ok(tag) => {
            settings.languages = vec![tag];
            Ok(settings)
        }
        Err(crate::lang::LocaleError::Empty) => Ok(settings),
        Err(crate::lang::LocaleError::Invalid(_) | crate::lang::LocaleError::Unknown(_)) => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }
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
    crate::lang::push_revision(&mut overlay.language_history, overlay.custom.clone(), overlay.language.clone(), "custom".into());
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
    use crate::home::{default_home, LoadedHome};
    use crate::types::ParseDecision;

    fn assert_no_executable_shape(value: &serde_json::Value) {
        match value {
            serde_json::Value::Object(fields) => {
                for forbidden in ["plan", "intent", "slots", "selected_candidate_id"] {
                    assert!(!fields.contains_key(forbidden), "confirmation leaked {forbidden}: {value}");
                }
                for child in fields.values() {
                    assert_no_executable_shape(child);
                }
            }
            serde_json::Value::Array(values) => values.iter().for_each(assert_no_executable_shape),
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) | serde_json::Value::String(_) => {}
        }
    }

    #[test]
    fn pins_assist_language_to_pack() {
        let base = Settings::default();
        assert_eq!(settings_for_parse(base.clone(), Some("en-US"), None, None).unwrap().languages, vec!["en-US".to_string()]);
        assert_eq!(settings_for_parse(base.clone(), Some("de-DE"), None, None).unwrap().languages, vec!["de-DE".to_string()]);
        assert_eq!(settings_for_parse(base.clone(), None, None, None).unwrap().languages, vec!["de".to_string(), "en".to_string()]);
        assert_eq!(settings_for_parse(base, Some("fr"), None, None).unwrap_err(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn assist_personality_overrides_overlay() {
        let out = settings_for_parse(Settings::default(), None, Some(crate::types::Personality::Butler), None).unwrap();
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

    #[tokio::test]
    async fn confirmation_never_exposes_plan_before_affirmation() {
        let dir = std::env::temp_dir().join(format!("klar-http-confirm-{}", std::process::id()));
        let state = AppState::new(
            LoadedHome {
                graph: default_home(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
            },
            dir,
            None,
        );
        let peer = ConnectInfo("127.0.0.1:9".parse().unwrap());
        let first = api_parse(
            State(state.clone()),
            peer,
            HeaderMap::new(),
            Json(ParseIn {
                text: "Wohnungstür abschließen".into(),
                conversation_id: Some("confirm-http".into()),
                language: Some("de".into()),
                personality: None,
                preferred_area: None,
                nlu_rag: None,
            }),
        )
        .await
        .expect("confirmation response")
        .0;
        assert!(matches!(first.decision, ParseDecision::Confirm { .. }), "{first:#?}");
        assert!(first.plan.is_none());
        assert_no_executable_shape(&serde_json::to_value(&first).expect("serialize confirmation"));
        assert!(state.sessions.lock().await.get_or_create(Some("confirm-http")).last.is_empty());

        let second = api_parse(
            State(state.clone()),
            peer,
            HeaderMap::new(),
            Json(ParseIn {
                text: "ja".into(),
                conversation_id: Some("confirm-http".into()),
                language: Some("de".into()),
                personality: None,
                preferred_area: None,
                nlu_rag: None,
            }),
        )
        .await
        .expect("affirmed response")
        .0;
        assert!(matches!(second.decision, ParseDecision::Execute), "{second:#?}");
        assert_eq!(second.plan.as_ref().map(|plan| plan.steps.len()), Some(1));
    }
}
