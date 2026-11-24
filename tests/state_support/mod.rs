//! Helpers for the engine-state suites: a data directory that removes itself, a fixture home
//! of our own, and requests against the assembled production router.
#![allow(dead_code)]

pub mod process;

use axum::body::{to_bytes, Body};
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode};
use klar_nlu::home::overlay::{apply_overlay, load_overlay};
use klar_nlu::home::LoadedHome;
use klar_nlu::io::{web, AppState};
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph};
use serde_json::Value;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tower::ServiceExt;

pub const OVERLAY_FILE: &str = "klar_nlu.json";

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

/// The engine state as startup builds it from the data directory, without the listeners.
pub struct Engine {
    pub state: AppState,
}

impl Engine {
    pub fn start(dir: &Path) -> Self {
        let overlay = load_overlay(dir);
        let mut graph = house();
        apply_overlay(&mut graph, &overlay);
        let loaded = LoadedHome {
            graph,
            settings: overlay.settings.unwrap_or_default(),
            custom: overlay.custom,
            language: overlay.language,
            policies: overlay.policies,
            speech_bank: overlay.speech_bank,
            match_controls: overlay.match_controls,
        };
        Self { state: AppState::new(loaded, dir.to_path_buf(), None) }
    }

    /// One request from loopback through the production router.
    pub async fn send(&self, method: Method, path: &str, content_type: &str, body: Vec<u8>) -> (StatusCode, Vec<u8>) {
        let mut request = Request::builder().method(method).uri(path).header("content-type", content_type).body(Body::from(body)).unwrap();
        request.extensions_mut().insert(ConnectInfo("127.0.0.1:9".parse::<SocketAddr>().unwrap()));
        let response = web::router(self.state.clone()).oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 16 * 1024 * 1024).await.unwrap();
        (status, bytes.to_vec())
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
