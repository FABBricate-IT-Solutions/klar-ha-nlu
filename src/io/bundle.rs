use crate::home::paths::{read_confined, read_to_string_confined, remove_confined, write_atomic_confined};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::state::AppState;
use crate::types::{Intent, ParseResult};
use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ENTRIES: usize = 2000;
const KEEP_ENTRIES: usize = 1500;
const BUNDLE_FILE: &str = "support_bundle.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleRequest {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleResponse {
    pub intents: Vec<Intent>,
    pub speech: String,
    pub clarify: bool,
    #[serde(default)]
    pub chat: bool,
    #[serde(default)]
    pub briefing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleEntry {
    #[serde(default)]
    pub id: String,
    pub ts_ms: u64,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<String>,
    pub request: BundleRequest,
    pub response: BundleResponse,
}

#[derive(Clone)]
pub struct BundleStore {
    dir: PathBuf,
    lock: std::sync::Arc<Mutex<()>>,
}

impl BundleStore {
    pub fn open(data_dir: &Path) -> Self {
        Self { dir: data_dir.to_path_buf(), lock: std::sync::Arc::new(Mutex::new(())) }
    }

    pub fn path(&self) -> PathBuf {
        crate::home::paths::confined_file(&self.dir, BUNDLE_FILE).unwrap_or_else(|_| PathBuf::from(BUNDLE_FILE))
    }

    pub fn append(&self, mut entry: BundleEntry) {
        if entry.id.is_empty() {
            entry.id = new_id(entry.ts_ms);
        }
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut entries = read_entries(&self.dir);
        let dirty = ensure_ids(&mut entries);
        entries.push(entry);
        if entries.len() > MAX_ENTRIES {
            entries = entries.split_off(entries.len() - KEEP_ENTRIES);
        } else if !dirty && entries.len() > 1 {
            if let Ok(line) = serde_json::to_string(entries.last().unwrap()) {
                return append_line(&self.dir, &line);
            }
        }
        write_entries(&self.dir, &entries);
    }

    pub fn load(&self) -> Vec<BundleEntry> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut entries = read_entries(&self.dir);
        if ensure_ids(&mut entries) {
            write_entries(&self.dir, &entries);
        }
        entries
    }

    pub fn remove(&self, ids: &[String]) -> usize {
        if ids.is_empty() {
            return 0;
        }
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut entries = read_entries(&self.dir);
        ensure_ids(&mut entries);
        let before = entries.len();
        entries.retain(|e| !ids.iter().any(|id| id == &e.id));
        let removed = before - entries.len();
        if removed > 0 {
            write_entries(&self.dir, &entries);
        }
        removed
    }

    pub fn protocol_bytes(&self) -> Vec<u8> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        read_confined(&self.dir, BUNDLE_FILE).unwrap_or_default()
    }

    pub fn clear(&self) {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let _ = remove_confined(&self.dir, BUNDLE_FILE);
    }

    pub fn dataset_yaml(&self) -> String {
        dataset_yaml(&self.load())
    }
}

pub fn entry_from_parse(source: &str, language: Option<&str>, result: &ParseResult) -> BundleEntry {
    let ts_ms = now_ms();
    BundleEntry {
        id: new_id(ts_ms),
        ts_ms,
        source: source.to_string(),
        language: language.filter(|s| !s.is_empty()).map(str::to_string),
        tokens: crate::parse::normalize::tokenize(&result.text),
        request: BundleRequest {
            text: result.text.clone(),
            conversation_id: (!result.conversation_id.is_empty()).then(|| result.conversation_id.clone()),
        },
        response: BundleResponse {
            intents: result.intents.clone(),
            speech: result.speech.clone(),
            clarify: result.clarify,
            chat: result.chat,
            briefing: result.briefing,
        },
    }
}

pub fn dataset_yaml(entries: &[BundleEntry]) -> String {
    #[derive(Serialize)]
    struct Case {
        name: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        conditions: Vec<BTreeMap<String, String>>,
        sentences: Vec<String>,
    }
    let cases: Vec<Case> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.request.text.trim().is_empty())
        .map(|(i, entry)| Case {
            name: format!("rec_{i:04}_{}", entry.source),
            conditions: entry.response.intents.iter().map(condition_from_intent).collect(),
            sentences: vec![entry.request.text.clone()],
        })
        .collect();
    serde_yaml::to_string(&cases).unwrap_or_else(|_| "[]\n".into())
}

fn condition_from_intent(intent: &Intent) -> BTreeMap<String, String> {
    let mut cond = BTreeMap::new();
    match intent.name.as_str() {
        "HassGetState" | "HassClimateGetTemperature" | "HassTimerStatus" => {
            cond.insert("type".into(), "query".into());
        }
        "HassTurnOff" => {
            cond.insert("type".into(), "action".into());
            cond.insert("state".into(), "off".into());
        }
        _ => {
            cond.insert("type".into(), "action".into());
            cond.insert("state".into(), "on".into());
        }
    }
    for key in [
        "entity_id",
        "area",
        "domain",
        "brightness",
        "temperature",
        "color",
        "percentage",
        "position",
        "search_query",
        "media_id",
        "media_type",
        "artist",
        "enqueue",
        "radio_mode",
        "volume_level",
        "volume_step",
    ] {
        if let Some(value) = intent.slot(key) {
            cond.insert(key.to_string(), value.to_string());
        }
    }
    cond
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn new_id(ts_ms: u64) -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();
    format!("{ts_ms}-{nanos:08x}")
}

fn read_entries(dir: &Path) -> Vec<BundleEntry> {
    let raw = read_to_string_confined(dir, BUNDLE_FILE).unwrap_or_default();
    raw.lines().filter(|l| !l.is_empty()).filter_map(|line| serde_json::from_str(line).ok()).collect()
}

fn ensure_ids(entries: &mut [BundleEntry]) -> bool {
    let mut dirty = false;
    for (i, entry) in entries.iter_mut().enumerate() {
        if entry.id.is_empty() {
            entry.id = format!("legacy-{}-{i}", entry.ts_ms);
            dirty = true;
        }
    }
    dirty
}

fn write_entries(dir: &Path, entries: &[BundleEntry]) {
    if entries.is_empty() {
        let _ = remove_confined(dir, BUNDLE_FILE);
        return;
    }
    let mut body = String::new();
    for entry in entries {
        if let Ok(line) = serde_json::to_string(entry) {
            body.push_str(&line);
            body.push('\n');
        }
    }
    let _ = write_atomic_confined(dir, BUNDLE_FILE, body.as_bytes());
}

fn append_line(dir: &Path, line: &str) {
    let mut body = read_to_string_confined(dir, BUNDLE_FILE).unwrap_or_default();
    if !body.is_empty() && !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str(line);
    body.push('\n');
    let _ = write_atomic_confined(dir, BUNDLE_FILE, body.as_bytes());
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/bundle", get(bundle_status).delete(clear_bundle))
        .route("/api/bundle/entries", get(list_entries).post(delete_entries))
        .route("/api/bundle/dataset", get(download_dataset))
        .route("/api/bundle/protocol", get(download_protocol))
        .route("/api/bundle/clear", post(clear_bundle))
}

const LIST_CAP: usize = 400;

#[derive(Serialize)]
struct BundleStatus {
    enabled: bool,
    count: usize,
    bytes: usize,
}

#[derive(Serialize)]
struct BundleListItem {
    id: String,
    ts_ms: u64,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    text: String,
    speech: String,
    intents: Vec<String>,
    clarify: bool,
    chat: bool,
}

#[derive(Serialize)]
struct BundleList {
    enabled: bool,
    count: usize,
    bytes: usize,
    entries: Vec<BundleListItem>,
}

#[derive(Deserialize)]
struct DeleteIn {
    #[serde(default)]
    ids: Vec<String>,
}

fn list_item(entry: BundleEntry) -> BundleListItem {
    BundleListItem {
        id: entry.id,
        ts_ms: entry.ts_ms,
        source: entry.source,
        language: entry.language,
        text: entry.request.text,
        speech: entry.response.speech,
        intents: entry.response.intents.iter().map(|i| i.name.clone()).collect(),
        clarify: entry.response.clarify,
        chat: entry.response.chat,
    }
}

async fn bundle_status(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<BundleStatus>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let enabled = state.settings.lock().await.support_bundle;
    let raw = state.bundle.protocol_bytes();
    let count = raw.split(|b| *b == b'\n').filter(|l| !l.is_empty()).count();
    Ok(Json(BundleStatus { enabled, count, bytes: raw.len() }))
}

async fn list_entries(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<BundleList>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let enabled = state.settings.lock().await.support_bundle;
    let mut entries = state.bundle.load();
    let count = entries.len();
    let bytes = state.bundle.protocol_bytes().len();
    if entries.len() > LIST_CAP {
        entries = entries.split_off(entries.len() - LIST_CAP);
    }
    entries.reverse();
    Ok(Json(BundleList { enabled, count, bytes, entries: entries.into_iter().map(list_item).collect() }))
}

async fn delete_entries(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<DeleteIn>,
) -> Result<Json<BundleList>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if body.ids.is_empty() || body.ids.len() > MAX_ENTRIES {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.bundle.remove(&body.ids);
    list_entries(State(state), ConnectInfo(peer), headers).await
}

async fn download_dataset(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let include_raw = state.settings.lock().await.support_bundle_raw_text;
    let yaml = crate::io::privacy::redact_entries(&state.bundle.load(), include_raw);
    Ok(attachment("klar-assist-dataset.yaml", "application/yaml; charset=utf-8", dataset_yaml(&yaml).into_bytes()))
}

async fn download_protocol(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let include_raw = state.settings.lock().await.support_bundle_raw_text;
    let mut body = String::new();
    for entry in crate::io::privacy::redact_entries(&state.bundle.load(), include_raw) {
        if let Ok(line) = serde_json::to_string(&entry) {
            body.push_str(&line);
            body.push('\n');
        }
    }
    Ok(attachment("klar-support-bundle.jsonl", "application/x-ndjson; charset=utf-8", body.into_bytes()))
}

async fn clear_bundle(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<BundleStatus>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    state.bundle.clear();
    let enabled = state.settings.lock().await.support_bundle;
    Ok(Json(BundleStatus { enabled, count: 0, bytes: 0 }))
}

fn attachment(name: &str, content_type: &'static str, body: Vec<u8>) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    (headers, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Intent;

    fn sample(text: &str, intent: Intent) -> BundleEntry {
        BundleEntry {
            id: String::new(),
            ts_ms: 1,
            source: "http".into(),
            language: Some("de".into()),
            tokens: crate::parse::normalize::tokenize(text),
            request: BundleRequest { text: text.into(), conversation_id: Some("c1".into()) },
            response: BundleResponse { intents: vec![intent], speech: "ok".into(), clarify: false, chat: false, briefing: false },
        }
    }

    #[test]
    fn dataset_maps_query_and_action() {
        let query = sample("Wie ist der Status der Küche", Intent::new("HassGetState").with("area", "kuche"));
        let off = sample("Licht aus", Intent::new("HassTurnOff").with("entity_id", "light.alle_lichter"));
        let yaml = dataset_yaml(&[query, off]);
        assert!(yaml.contains("Wie ist der Status der Küche"), "{yaml}");
        assert!(yaml.contains("type: query"), "{yaml}");
        assert!(yaml.contains("area: kuche"), "{yaml}");
        assert!(yaml.contains("Licht aus"), "{yaml}");
        assert!(yaml.contains("state: off"), "{yaml}");
        assert!(yaml.contains("light.alle_lichter"), "{yaml}");
    }

    #[test]
    fn append_and_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!("klar-bundle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let store = BundleStore::open(&dir);
        store.append(sample("Kugel an", Intent::new("HassTurnOn").with("entity_id", "light.schlafzimmer")));
        let loaded = store.load();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].request.text, "Kugel an");
        assert!(store.dataset_yaml().contains("Kugel an"));
        store.clear();
        assert!(store.load().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_keeps_other_rows() {
        let dir = std::env::temp_dir().join(format!("klar-bundle-rm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let store = BundleStore::open(&dir);
        store.append(sample("Kugel an", Intent::new("HassTurnOn")));
        store.append(sample("Licht aus", Intent::new("HassTurnOff")));
        let loaded = store.load();
        assert_eq!(loaded.len(), 2);
        assert!(!loaded[0].id.is_empty());
        store.remove(&[loaded[0].id.clone()]);
        let left = store.load();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].request.text, "Licht aus");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
