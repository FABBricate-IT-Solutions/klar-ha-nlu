use crate::home::expose::assist_visible;
use crate::home::gaps::leftover;
use crate::home::overlay::apply_overlay;
use crate::io::access_log::access_log;
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::auth_middleware::authorize;
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::parse_run::{run_parse, ParseRequest};
use crate::io::state::{save_failed, AppState};
use crate::nlu::legacy_result;
use crate::types::{known_intent, AreaRec, CustomSentence, EntityRec, ParseOutcome, Settings, TeachProposal};
use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

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
    let api = Router::new()
        .route("/api/v2/parse", post(api_parse))
        .route("/api/settings", get(get_settings).post(set_settings).patch(patch_settings))
        .route("/api/custom", get(get_custom).post(set_custom))
        .route("/api/entities", get(get_entities).post(tag_entity))
        .route("/api/gaps", get(get_gaps))
        .merge(crate::io::bundle::routes())
        .merge(crate::io::wrong_case::routes())
        .merge(crate::io::house_regression::routes())
        .merge(crate::io::dashboard::routes())
        .merge(crate::io::lang_api::routes())
        .merge(crate::io::conversations::routes())
        .merge(crate::io::policies::routes())
        .merge(crate::io::undo_priors::routes())
        .layer(DefaultBodyLimit::max(16 * 1024))
        .merge(crate::io::home_sync::routes(state.clone()))
        .merge(crate::io::backup::routes(state.clone()))
        .merge(crate::io::llm::routes())
        .merge(crate::io::speech::routes().layer(DefaultBodyLimit::max(crate::io::limits::MAX_SPEECH_RENDER_BYTES)))
        .layer(from_fn_with_state(state.clone(), authorize));
    Router::new()
        .route("/", get(index))
        // R-19: public health for watchdog / healthcheck / version compare (no token).
        .merge(crate::io::health::routes())
        .merge(api)
        // U-20: gzip UI assets when the client asks; long-cache hashed files, no-cache HTML.
        .fallback_service(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(SetResponseHeaderLayer::overriding(header::CACHE_CONTROL, |res: &axum::http::Response<_>| {
                    let ctype = res.headers().get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or("");
                    Some(if ctype.starts_with("text/html") {
                        HeaderValue::from_static("no-cache")
                    } else {
                        HeaderValue::from_static("public, max-age=31536000, immutable")
                    })
                }))
                .service(ServeDir::new(ui_dir())),
        )
        .with_state(state.clone())
        // 3.10: one access line per request (outside auth so denials are logged too).
        .layer(from_fn_with_state(state, access_log))
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
        || body.conversation_id.as_ref().is_some_and(|id| !id.is_empty() && !crate::session::valid_id(id))
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
    let match_controls = state.match_controls.lock().await.clone();
    // Seed follow-ups before preferred_area is set would miss the satellite hint (P-23).
    let mut session = {
        let mut sessions = state.sessions.lock().await;
        match body.preferred_area.as_deref().filter(|area| !area.is_empty()) {
            Some(area) => sessions.take_with_area(body.conversation_id.as_deref(), area),
            None => sessions.take(body.conversation_id.as_deref()),
        }
    };
    session.preferred_area = body.preferred_area.clone();
    let parsed = run_parse(
        &state.parse_limiter,
        ParseRequest { text: body.text.clone(), home, session, custom, settings, policies, speech_bank, match_controls },
    )
    .await?;
    let mut outcome = parsed.outcome;
    let mut session = parsed.session;
    // D-18: teach is a proposal; Assist confirms via POST /api/entities (PersistParse).
    if let Some((entity_id, alias)) = session.pending_teach.take() {
        outcome.teach = Some(TeachProposal { entity_id, alias });
    }
    let last_names = session.last.iter().map(|turn| turn.name.clone()).collect();
    {
        let mut sessions = state.sessions.lock().await;
        sessions.put(session);
        crate::ops::ops().sync_sessions(sessions.created(), sessions.len() as u64);
    }
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
    let current = state.settings.lock().await.clone();
    let mut body = crate::lang::check_settings(body, &current).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    // D-10: refuse a nonsense timezone instead of silently falling back to UTC.
    body.time_zone = crate::io::trainer_settings::checked_time_zone(&body.time_zone).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    if body.extra_prompt.chars().count() > 2000 || body.default_language.chars().count() > 32 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    state.overlay.update(|overlay| overlay.settings = Some(body.clone())).map_err(save_failed)?;
    *state.settings.lock().await = body.clone();
    Ok(Json(body))
}

async fn patch_settings(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(patch): Json<serde_json::Value>,
) -> Result<Json<Settings>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    let current = state.settings.lock().await.clone();
    let body = crate::io::trainer_settings::patch_settings_http(&current, &patch).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    state.overlay.update(|overlay| overlay.settings = Some(body.clone())).map_err(save_failed)?;
    *state.settings.lock().await = body.clone();
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

/// What `POST /api/custom` accepts. Restore holds a backup to the same rule.
pub(crate) fn custom_rows_ok(rows: &[CustomSentence]) -> bool {
    rows.len() <= 64
        && rows.iter().all(|row| row.phrase.len() <= 200 && row.phrase.trim().chars().count() >= 4 && known_intent(&row.intent))
}

async fn set_custom(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<Vec<CustomSentence>>,
) -> Result<Json<Vec<CustomSentence>>, StatusCode> {
    gate(peer, &headers, &state.token)?;
    if !custom_rows_ok(&body) {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .overlay
        .update(|overlay| {
            crate::lang::push_revision(&mut overlay.language_history, overlay.custom.clone(), overlay.language.clone(), "custom".into());
            overlay.custom = body.clone();
        })
        .map_err(save_failed)?;
    *state.custom.lock().await = body.clone();
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
}

async fn get_gaps(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<GapsOut>, StatusCode> {
    read_gate(peer, &headers, &state.token)?;
    let home = state.home.snapshot().await;
    Ok(Json(GapsOut { leftover: leftover(&home, state.catalog_for_settings().await.as_ref()), rooms: home.areas.clone() }))
}

#[derive(Deserialize)]
struct TagIn {
    entity_id: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    aliases: Option<Vec<String>>,
    #[serde(default)]
    preferred: Option<bool>,
    #[serde(default)]
    nlu_ignore: Option<bool>,
    pub area: Option<String>,
}

fn valid_entity_id(id: &str) -> bool {
    let mut parts = id.split('.');
    matches!((parts.next(), parts.next(), parts.next()), (Some(d), Some(n), None) if !d.is_empty() && !n.is_empty() && id.len() <= 128)
}

/// D-09: reject empty, overlong, and duplicate aliases. Empty `aliases: []` still means clear.
fn validate_aliases(raw: &[String]) -> Result<Vec<String>, StatusCode> {
    use crate::home::snapshot::{MAX_ALIAS_CHARS, MAX_SNAPSHOT_ALIASES};
    if raw.len() > MAX_SNAPSHOT_ALIASES {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut cleaned = Vec::with_capacity(raw.len());
    let mut seen = std::collections::HashSet::new();
    for alias in raw {
        let trimmed: String = alias.chars().filter(|ch| !ch.is_control()).collect();
        let trimmed = trimmed.trim();
        if trimmed.is_empty() || trimmed.chars().count() > MAX_ALIAS_CHARS {
            return Err(StatusCode::BAD_REQUEST);
        }
        if !seen.insert(trimmed.to_ascii_lowercase()) {
            return Err(StatusCode::BAD_REQUEST);
        }
        cleaned.push(trimmed.to_string());
    }
    Ok(cleaned)
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
    let cleaned_aliases = match &body.aliases {
        Some(aliases) if aliases.is_empty() => Some(Vec::new()),
        Some(aliases) => Some(validate_aliases(aliases)?),
        None => None,
    };
    let home = state.home.snapshot().await;
    if !home.entities.iter().any(|e| e.entity_id == body.entity_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    let previous_aliases = state
        .overlay
        .update(|overlay| {
            let previous = overlay.aliases.get(&body.entity_id).cloned();
            if let Some(aliases) = &cleaned_aliases {
                // Empty list clears operator aliases (U-36); a missing field leaves them alone.
                if aliases.is_empty() {
                    overlay.aliases.remove(&body.entity_id);
                } else {
                    overlay.aliases.insert(body.entity_id.clone(), aliases.clone());
                }
            }
            if let Some(area) = &body.area {
                // Empty string clears the room via apply_overlay; removing the key would not.
                overlay.areas.insert(body.entity_id.clone(), area.clone());
            }
            if let Some(preferred) = body.preferred {
                overlay.preferred.retain(|id| id != &body.entity_id);
                if preferred {
                    overlay.preferred.push(body.entity_id.clone());
                }
            }
            if let Some(ignore) = body.nlu_ignore {
                overlay.nlu_ignore.retain(|id| id != &body.entity_id);
                if ignore {
                    overlay.nlu_ignore.push(body.entity_id.clone());
                }
            }
            (previous, overlay.clone())
        })
        .map_err(save_failed)?;
    let (previous_operator, overlay) = previous_aliases;
    let updated = state
        .home
        .edit(|next| {
            // D-09: never replace the whole alias list with `[]` — strip only the previous
            // operator aliases, then let apply_overlay merge the new ones. Memory and file stay aligned.
            if cleaned_aliases.is_some() {
                if let (Some(previous), Some(ent)) =
                    (previous_operator.as_ref(), next.entities.iter_mut().find(|e| e.entity_id == body.entity_id))
                {
                    ent.aliases.retain(|alias| !previous.iter().any(|item| item == alias));
                }
            }
            apply_overlay(next, &overlay);
            let ent = next.entities.iter_mut().find(|e| e.entity_id == body.entity_id)?;
            let mut tags = ent.tags.clone();
            tags.retain(|t| t != "preferred" && t != "nlu_ignore");
            if overlay.preferred.iter().any(|id| id == &body.entity_id) || body.tags.iter().any(|t| t == "preferred") {
                tags.push("preferred".into());
            }
            if overlay.nlu_ignore.iter().any(|id| id == &body.entity_id) || body.tags.iter().any(|t| t == "nlu_ignore") {
                tags.push("nlu_ignore".into());
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
    use crate::home::test_dir::TestDir;
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
        assert!(settings_for_parse(base.clone(), None, None, None).unwrap().languages.is_empty());
        assert_eq!(settings_for_parse(base.clone(), Some("fr"), None, None).unwrap().languages, vec!["fr".to_string()]);
        assert_eq!(settings_for_parse(base.clone(), Some("pt-BR"), None, None).unwrap().languages, vec!["pt-BR".to_string()]);
        assert_eq!(settings_for_parse(base, Some("ru"), None, None).unwrap_err(), StatusCode::UNPROCESSABLE_ENTITY);
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

    fn parse_state(name: &str) -> (TestDir, AppState) {
        let scratch = TestDir::new(&format!("http-{name}"));
        let home = LoadedHome {
            graph: default_home(),
            settings: Settings::default(),
            custom: Vec::new(),
            language: Default::default(),
            policies: Vec::new(),
            speech_bank: Default::default(),
            match_controls: Vec::new(),
        };
        let state = AppState::new(home, scratch.path().to_path_buf(), None);
        (scratch, state)
    }

    async fn say(state: &AppState, text: &str, id: &str) -> Result<ParseOutcome, StatusCode> {
        let peer = ConnectInfo("127.0.0.1:9".parse().unwrap());
        let body = ParseIn {
            text: text.into(),
            conversation_id: Some(id.into()),
            language: Some("de".into()),
            personality: None,
            preferred_area: None,
            nlu_rag: None,
        };
        api_parse(State(state.clone()), peer, HeaderMap::new(), Json(body)).await.map(|Json(outcome)| outcome)
    }

    #[tokio::test]
    async fn confirmation_never_exposes_plan_before_affirmation() {
        let (_scratch, state) = parse_state("confirm");
        let first = say(&state, "Wohnungstür abschließen", "confirm-http").await.expect("confirmation response");
        assert!(matches!(first.decision, ParseDecision::Confirm { .. }), "{first:#?}");
        assert!(first.plan.is_none());
        assert_no_executable_shape(&serde_json::to_value(&first).expect("serialize confirmation"));
        assert!(state.sessions.lock().await.get_or_create(Some("confirm-http")).last.is_empty());

        let second = say(&state, "ja", "confirm-http").await.expect("affirmed response");
        assert!(matches!(second.decision, ParseDecision::Execute), "{second:#?}");
        assert_eq!(second.plan.as_ref().map(|plan| plan.steps.len()), Some(1));
    }

    #[tokio::test]
    async fn a_conversation_id_is_checked_at_the_door() {
        let (_scratch, state) = parse_state("ids");
        for bad in ["line\nbreak", "bell\u{7}", &"x".repeat(129)] {
            assert_eq!(say(&state, "Wohnungstür abschließen", bad).await.unwrap_err(), StatusCode::BAD_REQUEST, "{bad:?}");
        }
        // An empty id is no id: the question asked under it is not waiting for the next caller.
        let asked = say(&state, "Wohnungstür abschließen", "").await.expect("empty id is accepted");
        assert!(matches!(asked.decision, ParseDecision::Confirm { .. }), "{asked:#?}");
        let stranger = say(&state, "ja", "").await.expect("empty id is accepted");
        assert!(!matches!(stranger.decision, ParseDecision::Execute), "{stranger:#?}");
    }

    #[tokio::test]
    async fn set_custom_fails_closed_when_overlay_cannot_write() {
        // A file where the data directory should be: every write into it fails.
        let scratch = TestDir::new("custom-notdir");
        let path = scratch.file("not-a-directory");
        std::fs::write(&path, b"x").unwrap();
        let state = AppState::new(
            LoadedHome {
                graph: default_home(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            path.clone(),
            None,
        );
        let peer = ConnectInfo("127.0.0.1:9".parse().unwrap());
        let body = vec![CustomSentence { phrase: "good morning".into(), intent: "HassTurnOn".into(), slots: Default::default() }];
        let err = super::set_custom(State(state.clone()), peer, HeaderMap::new(), Json(body)).await.unwrap_err();
        assert_eq!(err, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(state.custom.lock().await.is_empty());
    }

    #[tokio::test]
    async fn tag_entity_clears_aliases_and_area() {
        let (_scratch, state) = parse_state("tag-clear");
        let peer = ConnectInfo("127.0.0.1:9".parse().unwrap());
        let entity_id = "light.wohnzimmer".to_string();
        let set = TagIn {
            entity_id: entity_id.clone(),
            tags: Vec::new(),
            aliases: Some(vec!["decke".into()]),
            preferred: None,
            nlu_ignore: None,
            area: Some("wohnzimmer".into()),
        };
        let Json(saved) = tag_entity(State(state.clone()), peer, HeaderMap::new(), Json(set)).await.expect("set");
        assert!(saved.aliases.iter().any(|alias| alias == "decke"));
        assert_eq!(saved.area.as_deref(), Some("wohnzimmer"));

        let clear = TagIn {
            entity_id: entity_id.clone(),
            tags: Vec::new(),
            aliases: Some(Vec::new()),
            preferred: None,
            nlu_ignore: None,
            area: Some(String::new()),
        };
        let Json(cleared) = tag_entity(State(state.clone()), peer, HeaderMap::new(), Json(clear)).await.expect("clear");
        // D-09: clearing operator aliases keeps the registry/demo aliases in memory and on disk.
        assert!(cleared.aliases.iter().any(|alias| alias == "wohnzimmer"), "{cleared:?}");
        assert!(!cleared.aliases.iter().any(|alias| alias == "decke"), "{cleared:?}");
        assert_eq!(cleared.area, None, "{cleared:?}");
        assert!(!crate::home::overlay::load_overlay(&state.data_dir).aliases.contains_key(&entity_id));
    }

    #[tokio::test]
    async fn tag_entity_rejects_bad_aliases() {
        let (_scratch, state) = parse_state("tag-alias");
        let peer = ConnectInfo("127.0.0.1:9".parse().unwrap());
        let entity_id = "light.wohnzimmer".to_string();
        for aliases in [vec![String::new()], vec!["x".repeat(9000)], vec!["decke".into(), "Decke".into()], vec!["ok".into(), "ok".into()]] {
            let body = TagIn {
                entity_id: entity_id.clone(),
                tags: Vec::new(),
                aliases: Some(aliases),
                preferred: None,
                nlu_ignore: None,
                area: None,
            };
            let err = tag_entity(State(state.clone()), peer, HeaderMap::new(), Json(body)).await.unwrap_err();
            assert_eq!(err, StatusCode::BAD_REQUEST);
        }
    }
}
