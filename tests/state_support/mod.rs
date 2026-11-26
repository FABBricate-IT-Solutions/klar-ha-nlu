//! Helpers for the engine-state suites: a data directory that removes itself, a fixture home
//! of our own, and requests against the assembled production router.
#![allow(dead_code)]

pub mod fake_llm;
pub mod process;

use axum::body::{to_bytes, Body};
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode};
use flate2::write::GzEncoder;
use flate2::Compression;
use klar_nlu::home::overlay::{apply_overlay, load_overlay};
use klar_nlu::home::snapshot::HOME_SCHEMA_VERSION;
use klar_nlu::home::{load_merged, LoadedHome};
use klar_nlu::io::{web, AppState};
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph};
use serde_json::{json, Value};
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower::ServiceExt;

pub const OVERLAY_FILE: &str = "klar_nlu.json";
pub const LLM_FILE: &str = "llm_endpoint.json";

/// A directory under the system temp dir that is removed when the value is dropped,
/// also when the test panics.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("klar-state-{tag}-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    /// Names in the directory, sorted.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.path)
            .map(|entries| entries.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        names.sort();
        names
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub const ROOMS: [&str; 4] = ["werkstatt", "bibliothek", "wintergarten", "waschkeller"];

pub fn lamp_id(index: usize) -> String {
    format!("light.lampe_{index:02}")
}

/// Four rooms with six lamps each. Every lamp is exposed.
pub fn house() -> HomeGraph {
    let areas =
        ROOMS.iter().map(|room| AreaRec { area_id: (*room).into(), name: (*room).into(), aliases: Vec::new(), floor_id: None }).collect();
    let entities = (0..24)
        .map(|index| EntityRec {
            entity_id: lamp_id(index),
            name: format!("Lampe {index}"),
            domain: "light".into(),
            platform: None,
            area: Some(ROOMS[index % ROOMS.len()].into()),
            aliases: Vec::new(),
            tags: Vec::new(),
        })
        .collect();
    HomeGraph { areas, entities, ..HomeGraph::default() }
}

/// The fixture house the way Home Assistant pushes it to `/api/v2/home`.
pub fn house_snapshot() -> Value {
    let entities: Vec<Value> = (0..24)
        .map(|index| json!({"entity_id": lamp_id(index), "name": format!("Lampe {index}"), "area_id": ROOMS[index % ROOMS.len()]}))
        .collect();
    let areas: Vec<Value> = ROOMS.iter().map(|room| json!({"id": room, "name": room})).collect();
    let assist: Vec<String> = (0..24).map(lamp_id).collect();
    json!({"schema_version": HOME_SCHEMA_VERSION, "entities": entities, "areas": areas, "assist": assist})
}

/// The fixture house as registry files under `config/.storage`, the way a restart reads it.
pub fn write_registry(config: &Path) {
    let storage = config.join(".storage");
    std::fs::create_dir_all(&storage).expect("storage dir");
    let entities: Vec<Value> = (0..24)
        .map(|index| {
            json!({"entity_id": lamp_id(index), "name": format!("Lampe {index}"), "area_id": ROOMS[index % ROOMS.len()],
                "platform": "demo", "options": {"conversation": {"should_expose": true}}})
        })
        .collect();
    let areas: Vec<Value> = ROOMS.iter().map(|room| json!({"id": room, "name": room})).collect();
    std::fs::write(storage.join("core.entity_registry"), json!({"data": {"entities": entities}}).to_string()).expect("entity registry");
    std::fs::write(storage.join("core.area_registry"), json!({"data": {"areas": areas}}).to_string()).expect("area registry");
}

/// A settings backup with the given overlay file and, when given, an endpoint file.
pub fn settings_archive(overlay: &str, endpoint: Option<&str>) -> Vec<u8> {
    let mut files = vec![(OVERLAY_FILE, overlay.as_bytes().to_vec())];
    if let Some(endpoint) = endpoint {
        files.push((LLM_FILE, endpoint.as_bytes().to_vec()));
    }
    let names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
    let manifest = json!({"format": "klar-settings-v1", "engine": "test", "created_at": "t", "include_secrets": false, "files": names});
    files.insert(0, ("manifest.json", serde_json::to_vec(&manifest).unwrap()));
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        for (name, data) in &files {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, data.as_slice()).unwrap();
        }
        builder.finish().unwrap();
    }
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes).unwrap();
    encoder.finish().unwrap()
}

/// The engine state as startup builds it from the data directory, without the listeners.
pub struct Engine {
    pub state: AppState,
}

impl Engine {
    pub fn start(dir: &Path) -> Self {
        Self::start_with_token(dir, None)
    }

    /// Like [`Self::start`], with the write token the engine was started with.
    pub fn start_with_token(dir: &Path, token: Option<&str>) -> Self {
        let overlay = load_overlay(dir);
        let mut graph = house();
        apply_overlay(&mut graph, &overlay);
        let loaded = LoadedHome {
            graph: graph.clone(),
            settings: overlay.settings.unwrap_or_default(),
            custom: overlay.custom,
            language: overlay.language,
            policies: overlay.policies,
            speech_bank: overlay.speech_bank,
            match_controls: overlay.match_controls,
        };
        let state = AppState::new(loaded, dir.to_path_buf(), token.map(str::to_string));
        // D-16: keep the fixture home across settings restore when there is no HA registry.
        *state.live_base.try_lock().expect("live_base") = Some(Arc::new(graph));
        Self { state }
    }

    /// The state as startup builds it from a config directory with registry files and a
    /// separate data directory.
    pub fn from_dirs(config: &Path, data: &Path) -> Self {
        let mut state = AppState::new(load_merged(config, data), data.to_path_buf(), None);
        state.config_dir = config.to_path_buf();
        Self { state }
    }

    /// Push the fixture house the way the integration does. The engine is live-synced after.
    pub async fn push_home(&self) {
        assert_eq!(self.post("/api/v2/home", house_snapshot()).await, StatusCode::OK);
    }

    pub async fn restore(&self, archive: Vec<u8>) -> StatusCode {
        self.send(Method::POST, "/api/v2/settings/restore", "application/gzip", archive).await.0
    }

    /// One request from loopback through the production router.
    pub async fn send(&self, method: Method, path: &str, content_type: &str, body: Vec<u8>) -> (StatusCode, Vec<u8>) {
        self.send_as("127.0.0.1:9", None, method, path, content_type, body).await
    }

    /// One request from `peer`, with `X-Klar-Token` when a token is given.
    pub async fn send_as(
        &self,
        peer: &str,
        token: Option<&str>,
        method: Method,
        path: &str,
        content_type: &str,
        body: Vec<u8>,
    ) -> (StatusCode, Vec<u8>) {
        let mut builder = Request::builder().method(method).uri(path).header("content-type", content_type);
        if let Some(token) = token {
            builder = builder.header("x-klar-token", token);
        }
        let mut request = builder.body(Body::from(body)).unwrap();
        request.extensions_mut().insert(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
        let response = web::router(self.state.clone()).oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 16 * 1024 * 1024).await.unwrap();
        (status, bytes.to_vec())
    }

    /// `POST` JSON, answer as JSON (`null` when the body is not JSON).
    pub async fn post_json(&self, path: &str, body: Value) -> (StatusCode, Value) {
        let (status, bytes) = self.send(Method::POST, path, "application/json", serde_json::to_vec(&body).unwrap()).await;
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    pub async fn post(&self, path: &str, body: Value) -> StatusCode {
        self.send(Method::POST, path, "application/json", serde_json::to_vec(&body).unwrap()).await.0
    }

    pub async fn get(&self, path: &str) -> (StatusCode, Value) {
        let (status, bytes) = self.send(Method::GET, path, "application/json", Vec::new()).await;
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).expect("read file")).expect("file holds valid JSON")
}
