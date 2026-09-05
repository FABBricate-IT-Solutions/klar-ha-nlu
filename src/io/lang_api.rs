//! Preview, explain, and rollback for user language rules.

use crate::home::overlay::{load_overlay, save_overlay};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::state::AppState;
use crate::lang::{
    bind_preview_user, install_user_overlay, pin_language, push_revision, select_revision, validate_custom, validate_language,
    LanguageOverlay, LanguageRevision,
};
use crate::nlu::parse;
use crate::session::Session;
use crate::types::{CustomSentence, ParseDecision, ParseOutcome, PolicyTrace, Settings};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/languages", get(list_languages))
        .route("/api/v2/intents", get(list_intents))
        .route("/api/lang/overlay", get(get_overlay).post(set_overlay))
        .route("/api/lang/preview", post(preview))
        .route("/api/lang/explain", post(explain))
        .route("/api/lang/rollback", post(rollback))
}

async fn list_intents() -> Json<Vec<&'static str>> {
    Json(crate::types::KNOWN_INTENTS.to_vec())
}

async fn list_languages() -> Json<Vec<LanguageOut>> {
    Json(
        crate::lang::languages()
            .iter()
            .map(|meta| LanguageOut {
                code: meta.code.to_string(),
                native_name: meta.native_name.to_string(),
                script: meta.script.to_string(),
                variants: meta.variants.iter().map(|item| (*item).to_string()).collect(),
            })
            .collect(),
    )
}

#[derive(Serialize)]
struct LanguageOut {
    code: String,
    native_name: String,
    script: String,
    variants: Vec<String>,
}

#[derive(Serialize)]
struct OverlayOut {
    custom: Vec<CustomSentence>,
    language: LanguageOverlay,
    history: Vec<HistoryRow>,
}

#[derive(Serialize)]
struct HistoryRow {
    hash: String,
    label: String,
    saved_at: String,
}

#[derive(Deserialize)]
struct OverlayIn {
    #[serde(default)]
    custom: Vec<CustomSentence>,
    #[serde(default)]
    language: Option<LanguageOverlay>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Deserialize)]
struct PreviewIn {
    text: String,
    language: Option<String>,
    #[serde(default)]
    custom: Option<Vec<CustomSentence>>,
    #[serde(default)]
    language_overlay: Option<LanguageOverlay>,
}

#[derive(Deserialize)]
struct RollbackIn {
    hash: Option<String>,
}

#[derive(Serialize)]
struct ExplainOut {
    language: String,
    decision: String,
    confidence: f64,
    speech: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    reply: String,
    stages: Vec<String>,
    evidence: Vec<String>,
    matched_custom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_trace: Option<PolicyTrace>,
}

fn write_gate(peer: SocketAddr, headers: &HeaderMap, token: &Option<String>) -> Result<(), StatusCode> {
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

async fn get_overlay(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<OverlayOut>, StatusCode> {
    read_gate(peer, &headers, &state.token)?;
    let overlay = load_overlay(&state.data_dir);
    Ok(Json(overlay_out(&overlay.custom, &overlay.language, &overlay.language_history)))
}

async fn set_overlay(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<OverlayIn>,
) -> Result<Json<OverlayOut>, StatusCode> {
    write_gate(peer, &headers, &state.token)?;
    let language = match body.language {
        Some(language) => language,
        None => load_overlay(&state.data_dir).language,
    };
    persist_rules(&state, body.custom, language, body.label.unwrap_or_else(|| "save".into())).await
}

async fn preview(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<PreviewIn>,
) -> Result<Json<ParseOutcome>, StatusCode> {
    read_gate(peer, &headers, &state.token)?;
    if body.text.chars().count() > MAX_PARSE_CHARS {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let settings = pin_settings(state.settings.lock().await.clone(), body.language.as_deref())?;
    let custom = match &body.custom {
        Some(rows) => {
            if !validate_custom(rows).is_empty() {
                return Err(StatusCode::UNPROCESSABLE_ENTITY);
            }
            rows.clone()
        }
        None => state.custom.lock().await.clone(),
    };
    if let Some(language) = &body.language_overlay {
        if !validate_language(language).is_empty() {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }
    let home = state.home.snapshot().await;
    let mut session = Session::default();
    let _preview = body.language_overlay.clone().map(|overlay| bind_preview_user(Some(overlay)));
    let outcome = parse(&body.text, &home, &mut session, &custom, &settings);
    Ok(Json(outcome))
}

async fn explain(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<PreviewIn>,
) -> Result<Json<ExplainOut>, StatusCode> {
    let language = body.language.clone().unwrap_or_default();
    let Json(outcome) = preview(State(state.clone()), ConnectInfo(peer), headers, Json(body)).await?;
    let mut out = explain_outcome(&language, &outcome);
    out.language = language;
    Ok(Json(out))
}

async fn rollback(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<RollbackIn>,
) -> Result<Json<OverlayOut>, StatusCode> {
    write_gate(peer, &headers, &state.token)?;
    let overlay = load_overlay(&state.data_dir);
    let revision = select_revision(&overlay.language_history, body.hash.as_deref()).ok_or(StatusCode::NOT_FOUND)?;
    persist_rules(&state, revision.custom, revision.language, format!("rollback {}", revision.hash)).await
}

async fn persist_rules(
    state: &AppState,
    custom: Vec<CustomSentence>,
    language: LanguageOverlay,
    label: String,
) -> Result<Json<OverlayOut>, StatusCode> {
    if !validate_custom(&custom).is_empty() || !validate_language(&language).is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let mut overlay = load_overlay(&state.data_dir);
    push_revision(&mut overlay.language_history, overlay.custom.clone(), overlay.language.clone(), label);
    overlay.custom = custom.clone();
    overlay.language = language.clone();
    save_overlay(&state.data_dir, &overlay).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    *state.custom.lock().await = custom.clone();
    install_user_overlay(if language.sets.is_empty() { None } else { Some(language.clone()) });
    Ok(Json(overlay_out(&custom, &language, &overlay.language_history)))
}

fn overlay_out(custom: &[CustomSentence], language: &LanguageOverlay, history: &[LanguageRevision]) -> OverlayOut {
    OverlayOut {
        custom: custom.to_vec(),
        language: language.clone(),
        history: history
            .iter()
            .rev()
            .map(|row| HistoryRow { hash: row.hash.clone(), label: row.label.clone(), saved_at: row.saved_at.clone() })
            .collect(),
    }
}

fn pin_settings(mut settings: Settings, language: Option<&str>) -> Result<Settings, StatusCode> {
    let Some(raw) = language.filter(|value| !value.is_empty()) else {
        return Ok(settings);
    };
    match pin_language(raw) {
        Ok(tag) => {
            settings.languages = vec![tag];
            Ok(settings)
        }
        Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }
}

fn explain_outcome(language: &str, outcome: &ParseOutcome) -> ExplainOut {
    let decision = match &outcome.decision {
        ParseDecision::Execute => "execute",
        ParseDecision::Confirm { .. } => "confirm",
        ParseDecision::Clarify { .. } => "clarify",
        ParseDecision::Reject { .. } => "reject",
        ParseDecision::Chat => "chat",
        ParseDecision::Error { .. } => "error",
    };
    let matched_custom =
        outcome.evidence.iter().find(|row| row.source.contains("custom") || row.kind.contains("custom")).map(|row| row.value.clone());
    ExplainOut {
        language: String::new(),
        decision: decision.into(),
        confidence: outcome.confidence,
        speech: path_explain_speech(language, outcome, decision),
        reply: outcome.speech.clone(),
        stages: outcome.trace.stages.iter().map(|stage| format!("{}: {}", stage.stage, stage.detail)).collect(),
        evidence: outcome.evidence.iter().map(|row| format!("{} {} {}", row.kind, row.source, row.value)).collect(),
        matched_custom,
        policy_trace: outcome.policy_trace.clone(),
    }
}

fn path_explain_speech(language: &str, outcome: &ParseOutcome, decision: &str) -> String {
    let trace = outcome.policy_trace.as_ref();
    let match_id = trace.and_then(|row| row.match_node.as_ref()).map(|node| node.id.as_str()).filter(|id| !id.is_empty());
    let seed_id = trace.and_then(|row| row.seed.as_ref()).map(|node| node.id.as_str()).filter(|id| !id.is_empty());
    let house_id =
        trace.and_then(|row| row.house.as_ref().map(|node| node.id.as_str()).or(row.matched_rule.as_deref())).filter(|id| !id.is_empty());
    let band = trace.and_then(|row| row.band.as_deref()).filter(|id| !id.is_empty()).unwrap_or(decision);
    let de = language == "de" || language.starts_with("de-");
    let mut parts = Vec::new();
    if let Some(id) = match_id {
        parts.push(format!("Match `{id}`"));
    }
    if let Some(id) = seed_id {
        parts.push(format!("Seed `{id}`"));
    }
    if let Some(id) = house_id {
        if de {
            parts.push(format!("Haus `{id}`"));
        } else {
            parts.push(format!("house `{id}`"));
        }
    }
    parts.push(if de { band_de(band) } else { band.into() });
    if parts.len() == 1 {
        return parts.pop().unwrap_or_default();
    }
    parts.join(", ") + "."
}

fn band_de(band: &str) -> String {
    match band {
        "execute" => "ausgeführt".into(),
        "confirm" => "bestätigen".into(),
        "clarify" => "nachfragen".into(),
        "reject" => "abgelehnt".into(),
        "chat" => "chat".into(),
        "error" => "fehler".into(),
        other => other.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::lang::{installed_user_overlay, reset_runtime_packs, SetDelta};
    use crate::types::CustomSentence;
    use std::collections::HashMap;
    use std::sync::OnceLock;
    use tokio::sync::{Mutex, MutexGuard};

    fn overlay_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    async fn lock_overlay() -> MutexGuard<'static, ()> {
        overlay_lock().lock().await
    }

    fn state(tag: &str) -> AppState {
        reset_runtime_packs();
        let dir = std::env::temp_dir().join(format!("klar-m5-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::remove_file(dir.join("klar_nlu.json"));
        AppState::new(
            LoadedHome {
                graph: default_home(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        )
    }

    fn peer() -> ConnectInfo<SocketAddr> {
        ConnectInfo("127.0.0.1:9".parse().unwrap())
    }

    fn rule(phrase: &str, intent: &str) -> CustomSentence {
        CustomSentence { phrase: phrase.into(), intent: intent.into(), slots: HashMap::new() }
    }

    #[tokio::test]
    async fn omitted_language_keeps_set_deltas() {
        let _guard = lock_overlay().await;
        let state = state("keep");
        let language =
            LanguageOverlay { sets: [("nouns.light_nouns".into(), SetDelta { add: vec!["kugelchen".into()], remove: vec![] })].into() };
        let _ = set_overlay(
            State(state.clone()),
            peer(),
            HeaderMap::new(),
            Json(OverlayIn { custom: vec![rule("filmabend", "HassTurnOn")], language: Some(language.clone()), label: Some("sets".into()) }),
        )
        .await
        .expect("save with sets");
        let kept = set_overlay(
            State(state.clone()),
            peer(),
            HeaderMap::new(),
            Json(
                serde_json::from_value(serde_json::json!({
                    "custom": [{"phrase": "filmabend zwei", "intent": "HassTurnOn", "slots": {}}],
                    "label": "phrase-only"
                }))
                .unwrap(),
            ),
        )
        .await
        .expect("save without language")
        .0;
        assert_eq!(kept.custom[0].phrase, "filmabend zwei");
        assert_eq!(kept.language, language);
        assert!(installed_user_overlay().is_some_and(|overlay| overlay.sets.contains_key("nouns.light_nouns")));
        reset_runtime_packs();
    }

    #[tokio::test]
    async fn default_rollback_restores_latest_history() {
        let _guard = lock_overlay().await;
        let state = state("roll");
        let _ = set_overlay(
            State(state.clone()),
            peer(),
            HeaderMap::new(),
            Json(OverlayIn { custom: vec![rule("erste regel", "HassTurnOn")], language: None, label: Some("a".into()) }),
        )
        .await
        .expect("save a");
        let after_b = set_overlay(
            State(state.clone()),
            peer(),
            HeaderMap::new(),
            Json(OverlayIn { custom: vec![rule("zweite regel", "HassTurnOff")], language: None, label: Some("b".into()) }),
        )
        .await
        .expect("save b")
        .0;
        assert_eq!(after_b.custom[0].phrase, "zweite regel");
        let named =
            rollback(State(state.clone()), peer(), HeaderMap::new(), Json(RollbackIn { hash: Some(after_b.history[0].hash.clone()) }))
                .await
                .expect("named rollback")
                .0;
        assert_eq!(named.custom[0].phrase, "erste regel");
        let _ = set_overlay(
            State(state.clone()),
            peer(),
            HeaderMap::new(),
            Json(OverlayIn { custom: vec![rule("zweite regel", "HassTurnOff")], language: None, label: Some("b2".into()) }),
        )
        .await
        .expect("save b again");
        let rolled = rollback(State(state), peer(), HeaderMap::new(), Json(RollbackIn { hash: None })).await.expect("default rollback").0;
        assert_eq!(rolled.custom[0].phrase, "erste regel");
        reset_runtime_packs();
    }

    #[tokio::test]
    async fn preview_does_not_install_live_overlay() {
        let _guard = lock_overlay().await;
        let state = state("prev");
        let proposed =
            LanguageOverlay { sets: [("nouns.light_nouns".into(), SetDelta { add: vec!["vorschauwort".into()], remove: vec![] })].into() };
        let outcome = preview(
            State(state),
            peer(),
            HeaderMap::new(),
            Json(PreviewIn { text: "Licht an".into(), language: Some("de".into()), custom: None, language_overlay: Some(proposed) }),
        )
        .await
        .expect("preview")
        .0;
        assert!(matches!(
            outcome.decision,
            ParseDecision::Execute | ParseDecision::Clarify { .. } | ParseDecision::Reject { .. } | ParseDecision::Confirm { .. }
        ));
        assert!(installed_user_overlay().is_none());
        reset_runtime_packs();
    }

    #[test]
    fn explain_speech_speaks_path_ids() {
        let outcome = ParseOutcome {
            schema_version: "2.0".into(),
            text: "Licht an".into(),
            conversation_id: "t".into(),
            decision: ParseDecision::Execute,
            speech: "Wohnzimmerlicht ist an.".into(),
            confidence: 0.9,
            margin: 0.1,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: crate::types::ParseTrace { stages: Vec::new(), discarded: Vec::new(), tokens: Vec::new(), normalized: String::new() },
            briefing: false,
            retrieval: None,
            policy_trace: Some(crate::types::PolicyTrace {
                match_node: Some(crate::types::PolicyTraceMatch { id: "area_command".into(), score: 0.93, origin: "engine".into() }),
                house: Some(crate::types::PolicyTraceLayer {
                    id: "prefer-ceiling".into(),
                    hit: Some("prefer_entity".into()),
                    origin: "operator".into(),
                }),
                band: Some("execute".into()),
                ..crate::types::PolicyTrace::default()
            }),
            quiet_ack_eligible: false,
        };
        let out = explain_outcome("en", &outcome);
        assert_eq!(out.decision, "execute");
        assert_eq!(out.reply, "Wohnzimmerlicht ist an.");
        assert!(out.speech.contains("Match `area_command`"));
        assert!(out.speech.contains("house `prefer-ceiling`"));
        assert!(out.speech.contains("execute"));
        assert_eq!(
            out.policy_trace.as_ref().and_then(|trace| trace.match_node.as_ref()).map(|node| node.id.as_str()),
            Some("area_command")
        );
        let de = explain_outcome("de", &outcome);
        assert!(de.speech.contains("Haus `prefer-ceiling`"));
        assert!(de.speech.contains("ausgeführt"));
    }
}
