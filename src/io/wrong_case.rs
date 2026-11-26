//! Operator “that was wrong”: mark a turn with the intended intent/target and export
//! an `nlu_expect` regression case plus a redacted home snippet (package 6.2).

use crate::home::labels::rebuild_label_index;
use crate::home::paths::{append_confined, read_to_string_confined, remove_confined, write_atomic_confined};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::conversations::ConversationTurn;
use crate::io::privacy::NameMap;
use crate::io::state::AppState;
use crate::types::{known_intent, AreaRec, EntityRec, HomeGraph};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const FILE: &str = "wrong_cases.jsonl";
const MAX_CASES: usize = 500;
const KEEP_CASES: usize = 400;
const MAX_TEXT: usize = 512;
const MAX_SLOT_CHARS: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntendedExpect {
    pub intent: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub slots: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reject: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub clarify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrongCase {
    pub id: String,
    pub ts_ms: u64,
    pub conversation_id: String,
    pub turn_ts_ms: u64,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub intended: IntendedExpect,
    pub home: HomeGraph,
}

#[derive(Clone)]
pub struct WrongCaseStore {
    dir: PathBuf,
    lock: std::sync::Arc<Mutex<()>>,
}

impl WrongCaseStore {
    pub fn open(data_dir: &Path) -> Self {
        Self { dir: data_dir.to_path_buf(), lock: std::sync::Arc::new(Mutex::new(())) }
    }

    pub fn append(&self, case: WrongCase) {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut cases = read_cases(&self.dir);
        cases.push(case);
        if cases.len() > MAX_CASES {
            cases = cases.split_off(cases.len() - KEEP_CASES);
            write_cases(&self.dir, &cases);
        } else if let Ok(line) = serde_json::to_string(cases.last().unwrap()) {
            append_line(&self.dir, &line);
        }
    }

    pub fn load(&self) -> Vec<WrongCase> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        read_cases(&self.dir)
    }

    pub fn clear(&self) {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        if let Err(err) = remove_confined(&self.dir, FILE) {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("{FILE} nicht gelöscht: {err}");
            }
        }
    }

    pub fn dataset_yaml(&self) -> String {
        export_dataset(&self.load())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkWrongIn {
    pub conversation_id: String,
    pub ts_ms: u64,
    #[serde(default)]
    pub text: Option<String>,
    pub intent: String,
    #[serde(default)]
    pub slots: BTreeMap<String, String>,
    #[serde(default)]
    pub reject: bool,
    #[serde(default)]
    pub clarify: bool,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MarkWrongOut {
    pub id: String,
    pub case_yaml: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/conversations/mark-wrong", post(mark_wrong))
        .route("/api/v2/wrong-cases", get(list_wrong_cases))
        .route("/api/v2/wrong-cases/dataset", get(download_wrong_dataset))
}

async fn list_wrong_cases(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Vec<WrongCaseSummary>>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let rows = state
        .wrong_cases
        .load()
        .into_iter()
        .map(|case| WrongCaseSummary {
            id: case.id,
            ts_ms: case.ts_ms,
            conversation_id: case.conversation_id,
            turn_ts_ms: case.turn_ts_ms,
            text: case.text,
            intent: case.intended.intent,
        })
        .collect();
    Ok(Json(rows))
}

#[derive(Debug, Serialize)]
struct WrongCaseSummary {
    id: String,
    ts_ms: u64,
    conversation_id: String,
    turn_ts_ms: u64,
    text: String,
    intent: String,
}

async fn download_wrong_dataset(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let body = state.wrong_cases.dataset_yaml().into_bytes();
    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/yaml; charset=utf-8"));
    if let Ok(value) = axum::http::HeaderValue::from_str("attachment; filename=\"klar-wrong-cases.yaml\"") {
        headers.insert(axum::http::header::CONTENT_DISPOSITION, value);
    }
    Ok((headers, body))
}

async fn mark_wrong(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<MarkWrongIn>,
) -> Result<Json<MarkWrongOut>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let turns = state.journal.list();
    let turn = find_turn(&turns, &body.conversation_id, body.ts_ms).ok_or(StatusCode::NOT_FOUND)?;
    let text = resolve_text(&body, turn).ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    let intended = sanitize_intended(&body).ok_or(StatusCode::BAD_REQUEST)?;
    let home = state.home.snapshot().await;
    let (intended, home) = redact_case(&intended, &home);
    let ts_ms = now_ms();
    let id = format!("{ts_ms}-{:08x}", (ts_ms as u32).wrapping_mul(0x9e37_79b9));
    let case = WrongCase {
        id: id.clone(),
        ts_ms,
        conversation_id: body.conversation_id,
        turn_ts_ms: body.ts_ms,
        text,
        language: body.language.filter(|code| !code.is_empty() && code.chars().count() <= 16),
        intended,
        home,
    };
    let case_yaml = export_one(&case);
    state.wrong_cases.append(case);
    Ok(Json(MarkWrongOut { id, case_yaml }))
}

fn find_turn<'a>(turns: &'a [ConversationTurn], conversation_id: &str, ts_ms: u64) -> Option<&'a ConversationTurn> {
    turns.iter().find(|turn| turn.conversation_id == conversation_id && turn.ts_ms == ts_ms)
}

fn resolve_text(body: &MarkWrongIn, turn: &ConversationTurn) -> Option<String> {
    let raw = body
        .text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
        .or_else(|| turn.text.as_deref().map(str::trim).filter(|text| !text.is_empty()).map(str::to_string))
        .or_else(|| {
            let joined = turn.tokens.join(" ");
            let trimmed = joined.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })?;
    if raw.chars().count() > MAX_TEXT || raw.chars().any(char::is_control) {
        return None;
    }
    Some(raw)
}

fn sanitize_intended(body: &MarkWrongIn) -> Option<IntendedExpect> {
    if body.reject && body.clarify {
        return None;
    }
    if body.reject && !body.slots.is_empty() {
        return None;
    }
    let intent = body.intent.trim();
    if body.reject {
        return Some(IntendedExpect { intent: String::new(), slots: BTreeMap::new(), reject: true, clarify: false });
    }
    if intent.is_empty() || intent.chars().count() > 64 || !known_intent(intent) {
        return None;
    }
    let mut slots = BTreeMap::new();
    for (name, value) in &body.slots {
        let name = name.trim();
        let value = value.trim();
        if name.is_empty() || value.is_empty() {
            return None;
        }
        if name.chars().count() > 32 || value.chars().count() > MAX_SLOT_CHARS {
            return None;
        }
        if name.chars().any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_')) {
            return None;
        }
        if value.chars().any(char::is_control) {
            return None;
        }
        slots.insert(name.to_string(), value.to_string());
    }
    if !body.clarify && slots.is_empty() {
        return None;
    }
    Some(IntendedExpect { intent: intent.to_string(), slots, reject: false, clarify: body.clarify })
}

/// Keep only entities/areas needed for the intended target, then pseudonymize.
pub fn redact_case(intended: &IntendedExpect, home: &HomeGraph) -> (IntendedExpect, HomeGraph) {
    let mut names = NameMap::default();
    let snippet = home_snippet(intended, home);
    let mut redacted = HomeGraph {
        entities: snippet
            .entities
            .iter()
            .map(|entity| EntityRec {
                entity_id: names.entity(&entity.entity_id),
                name: entity.name.clone(),
                domain: entity.domain.clone(),
                platform: None,
                area: entity.area.as_deref().map(|area| names.area(area)),
                aliases: Vec::new(),
                tags: Vec::new(),
            })
            .collect(),
        areas: snippet
            .areas
            .iter()
            .map(|area| AreaRec { area_id: names.area(&area.area_id), name: area.name.clone(), aliases: Vec::new(), floor_id: None })
            .collect(),
        floors: Vec::new(),
        scene_members: Default::default(),
        assist: None,
        policy: Default::default(),
        registered_intents: Vec::new(),
        device_classes: Default::default(),
        label_index: None,
    };
    rebuild_label_index(&mut redacted);
    let mut out = intended.clone();
    if let Some(entity_id) = out.slots.get("entity_id").cloned() {
        out.slots.insert("entity_id".into(), names.entity(&entity_id));
    }
    if let Some(area) = out.slots.get("area").cloned() {
        out.slots.insert("area".into(), names.area(&area));
    }
    (out, redacted)
}

fn home_snippet(intended: &IntendedExpect, home: &HomeGraph) -> HomeGraph {
    let mut entity_ids = HashSet::new();
    let mut area_ids = HashSet::new();
    if let Some(entity_id) = intended.slots.get("entity_id") {
        entity_ids.insert(entity_id.clone());
        if let Some(entity) = home.entities.iter().find(|row| row.entity_id == *entity_id) {
            if let Some(area) = entity.area.as_deref() {
                area_ids.insert(area.to_string());
            }
        }
    }
    if let Some(area) = intended.slots.get("area") {
        area_ids.insert(area.clone());
        for entity in &home.entities {
            if entity.area.as_deref() == Some(area.as_str()) {
                entity_ids.insert(entity.entity_id.clone());
            }
        }
    }
    let entities: Vec<EntityRec> = home.entities.iter().filter(|entity| entity_ids.contains(&entity.entity_id)).cloned().collect();
    for entity in &entities {
        if let Some(area) = entity.area.as_deref() {
            area_ids.insert(area.to_string());
        }
    }
    let areas: Vec<AreaRec> = home.areas.iter().filter(|area| area_ids.contains(&area.area_id)).cloned().collect();
    HomeGraph {
        entities,
        areas,
        floors: Vec::new(),
        scene_members: Default::default(),
        assist: None,
        policy: Default::default(),
        registered_intents: Vec::new(),
        device_classes: Default::default(),
        label_index: None,
    }
}

#[derive(Serialize, Deserialize)]
struct ExportCase {
    name: String,
    sentences: Vec<String>,
    nlu_expect: ExportExpect,
    home: HomeGraph,
}

#[derive(Serialize, Deserialize)]
struct ExportExpect {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    intents: Vec<ExportIntent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reject: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    clarify: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct ExportIntent {
    intent: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    slots: BTreeMap<String, String>,
}

pub fn export_one(case: &WrongCase) -> String {
    serde_yaml::to_string(&export_case(case, 0)).unwrap_or_else(|_| String::new())
}

pub fn export_dataset(cases: &[WrongCase]) -> String {
    let rows: Vec<ExportCase> = cases.iter().enumerate().map(|(index, case)| export_case(case, index)).collect();
    serde_yaml::to_string(&rows).unwrap_or_else(|_| "[]\n".into())
}

fn export_case(case: &WrongCase, index: usize) -> ExportCase {
    let intents = if case.intended.reject || case.intended.intent.is_empty() {
        Vec::new()
    } else {
        vec![ExportIntent { intent: case.intended.intent.clone(), slots: case.intended.slots.clone() }]
    };
    ExportCase {
        name: format!("wrong_{index:04}"),
        sentences: vec![case.text.clone()],
        nlu_expect: ExportExpect { intents, reject: case.intended.reject.then_some(true), clarify: case.intended.clarify.then_some(true) },
        home: case.home.clone(),
    }
}

/// Run an exported case against its home snippet. Err means the suite would go red.
#[allow(dead_code)] // used by crate tests and as the acceptance harness for 6.2
pub fn evaluate_exported_yaml(yaml: &str) -> Result<(), String> {
    let case = parse_export_yaml(yaml)?;
    let mut home = case.home;
    rebuild_label_index(&mut home);
    let sentence = case.sentences.first().ok_or("case has no sentence")?;
    let mut session = crate::session::Session::new();
    let settings = crate::types::Settings { languages: vec!["de".into(), "en".into()], ..crate::types::Settings::default() };
    let result = crate::parse::parse(sentence, &home, &mut session, &[], &settings);
    if case.nlu_expect.reject == Some(true) {
        let actual_reject = result.intents.is_empty() && !result.clarify && !result.chat;
        return if actual_reject {
            Ok(())
        } else {
            Err(format!("expected reject, got intents={:?} clarify={} chat={}", result.intents, result.clarify, result.chat))
        };
    }
    if case.nlu_expect.clarify == Some(true) {
        return if result.clarify { Ok(()) } else { Err(format!("expected clarify, got intents={:?}", result.intents)) };
    }
    let wanted = &case.nlu_expect.intents;
    if wanted.len() != result.intents.len() {
        return Err(format!("exact intents: expected {}, got {}: {:?}", wanted.len(), result.intents.len(), result.intents));
    }
    for (index, (want, got)) in wanted.iter().zip(&result.intents).enumerate() {
        if want.intent != got.name {
            return Err(format!("exact intent[{index}]: expected {}, got {}", want.intent, got.name));
        }
        let actual: BTreeMap<_, _> = got.slots.iter().map(|slot| (slot.name.clone(), slot.value.clone())).collect();
        if want.slots != actual {
            return Err(format!("exact intent[{index}] {} slots: expected {:?}, got {actual:?}", want.intent, want.slots));
        }
    }
    Ok(())
}

fn parse_export_yaml(yaml: &str) -> Result<ExportCase, String> {
    if let Ok(case) = serde_yaml::from_str::<ExportCase>(yaml) {
        return Ok(case);
    }
    let rows: Vec<ExportCase> = serde_yaml::from_str(yaml).map_err(|err| err.to_string())?;
    rows.into_iter().next().ok_or_else(|| "empty dataset".into())
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn read_cases(dir: &Path) -> Vec<WrongCase> {
    let raw = read_to_string_confined(dir, FILE).unwrap_or_default();
    raw.lines().filter(|line| !line.is_empty()).filter_map(|line| serde_json::from_str(line).ok()).collect()
}

fn write_cases(dir: &Path, cases: &[WrongCase]) {
    if cases.is_empty() {
        let _ = remove_confined(dir, FILE);
        return;
    }
    let mut body = String::new();
    for case in cases {
        if let Ok(line) = serde_json::to_string(case) {
            body.push_str(&line);
            body.push('\n');
        }
    }
    if let Err(err) = write_atomic_confined(dir, FILE, body.as_bytes()) {
        tracing::warn!("{FILE} nicht gespeichert: {err}");
    }
}

fn append_line(dir: &Path, line: &str) {
    let mut payload = String::new();
    if let Ok(existing) = read_to_string_confined(dir, FILE) {
        if !existing.is_empty() && !existing.ends_with('\n') {
            payload.push('\n');
        }
    }
    payload.push_str(line);
    payload.push('\n');
    if let Err(err) = append_confined(dir, FILE, payload.as_bytes()) {
        tracing::warn!("{FILE} nicht angehängt: {err}");
    }
}

#[cfg(test)]
#[path = "wrong_case_tests.rs"]
mod tests;
