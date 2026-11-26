use crate::home::labels::rebuild_label_index;
use crate::home::overlay::{apply_overlay, load_overlay};
use crate::home::snapshot::{ingest, save_home_snapshot, HomeSnapshot, SnapshotError};
use crate::io::auth::home_writes_allowed;
use crate::io::limits::MAX_HOME_SNAPSHOT_BYTES;
use crate::io::state::AppState;
use crate::types::HomeGraph;
use axum::extract::{ConnectInfo, DefaultBodyLimit, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v2/home", post(api_home))
        .layer(DefaultBodyLimit::max(MAX_HOME_SNAPSHOT_BYTES))
        .layer(from_fn_with_state(state, authorize_home))
}

// Reject untrusted writers before Json buffers/deserializes a multi-megabyte body.
async fn authorize_home(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !home_writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(request).await)
}

#[derive(Debug, serde::Serialize)]
struct HomeAck {
    entities: usize,
    areas: usize,
    floors: usize,
    assist: Option<usize>,
}

async fn api_home(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<HomeSnapshot>,
) -> Result<Json<HomeAck>, StatusCode> {
    if !home_writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let ingested = match ingest(body.clone()) {
        Ok(graph) => graph,
        Err(SnapshotError::Caps) => return Err(StatusCode::PAYLOAD_TOO_LARGE),
        Err(SnapshotError::Schema | SnapshotError::Malformed(_)) => return Err(StatusCode::BAD_REQUEST),
    };
    // D-17: keep the last push on disk so a restart does not drop the live home.
    if let Err(err) = save_home_snapshot(&state.data_dir, &body) {
        tracing::error!("home_snapshot.json nicht gespeichert: {err}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    *state.live_base.lock().await = Some(Arc::new(ingested.clone()));
    let graph = apply_live_overlays(&state, ingested);
    let ack = HomeAck {
        entities: graph.entities.len(),
        areas: graph.areas.len(),
        floors: graph.floors.len(),
        assist: graph.assist.as_ref().map(std::collections::HashSet::len),
    };
    state.home.replace(graph).await;
    state.live_sync.store(true, Ordering::Relaxed);
    let synced_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    state.home_synced_ms.store(synced_ms, Ordering::Relaxed);
    Ok(Json(ack))
}

/// The pushed home with the calibration of both directories on top. The label index is built
/// again afterwards: the one from ingest does not know the aliases the overlays add.
pub(crate) fn apply_live_overlays(state: &AppState, mut graph: HomeGraph) -> HomeGraph {
    apply_overlay(&mut graph, &load_overlay(&state.config_dir));
    if state.data_dir != state.config_dir {
        apply_overlay(&mut graph, &load_overlay(&state.data_dir));
    }
    rebuild_label_index(&mut graph);
    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::snapshot::HOME_SCHEMA_VERSION;
    use crate::home::test_dir::TestDir;
    use crate::home::LoadedHome;
    use crate::types::Settings;
    use serde_json::json;

    #[tokio::test]
    async fn snapshot_replaces_graph_and_enables_live_sync() {
        let scratch = TestDir::new("home-sync");
        let dir = scratch.path().to_path_buf();
        let state = AppState::new(
            LoadedHome {
                graph: HomeGraph::default(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        );
        let body = serde_json::from_value::<HomeSnapshot>(json!({
            "schema_version": HOME_SCHEMA_VERSION,
            "entities": [{"entity_id": "light.living", "name": "Living", "area_id": "living"}],
            "areas": [{"id": "living", "name": "Wohnzimmer", "floor_id": "upper"}],
            "floors": [{"floor_id": "upper", "name": "Upper Floor", "level": 1}],
            "assist": ["light.living"]
        }))
        .expect("snapshot json");
        let ack = api_home(State(state.clone()), ConnectInfo("127.0.0.1:9".parse().unwrap()), HeaderMap::new(), Json(body))
            .await
            .expect("accepted")
            .0;
        assert_eq!(ack.entities, 1);
        assert_eq!(ack.floors, 1);
        assert!(state.live_sync.load(Ordering::Relaxed));
        let home = state.home.snapshot().await;
        assert_eq!(home.areas[0].floor_id.as_deref(), Some("upper"));
        assert!(state.data_dir.join(crate::home::snapshot::HOME_SNAPSHOT_FILE).is_file());
        let restored = crate::home::snapshot::load_home_snapshot(&state.data_dir).expect("persisted");
        assert_eq!(restored.entities.len(), 1);
    }

    #[tokio::test]
    async fn empty_snapshot_is_rejected_and_does_not_clear_home() {
        let scratch = TestDir::new("home-sync-empty");
        let dir = scratch.path().to_path_buf();
        let state = AppState::new(
            LoadedHome {
                graph: crate::home::default_home(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        );
        let before = state.home.snapshot().await.entities.len();
        let body = serde_json::from_value::<HomeSnapshot>(json!({
            "schema_version": HOME_SCHEMA_VERSION,
            "entities": [],
            "assist": []
        }))
        .expect("snapshot json");
        let err =
            api_home(State(state.clone()), ConnectInfo("127.0.0.1:9".parse().unwrap()), HeaderMap::new(), Json(body)).await.unwrap_err();
        assert_eq!(err, StatusCode::BAD_REQUEST);
        assert_eq!(state.home.snapshot().await.entities.len(), before);
    }

    #[tokio::test]
    async fn malformed_snapshot_is_rejected() {
        let scratch = TestDir::new("home-sync-bad");
        let dir = scratch.path().to_path_buf();
        let state = AppState::new(
            LoadedHome {
                graph: HomeGraph::default(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        );
        let body = serde_json::from_value::<HomeSnapshot>(json!({
            "schema_version": HOME_SCHEMA_VERSION,
            "entities": [{"entity_id": "not-an-id"}]
        }))
        .expect("snapshot json");
        let err = api_home(State(state), ConnectInfo("127.0.0.1:9".parse().unwrap()), HeaderMap::new(), Json(body)).await.unwrap_err();
        assert_eq!(err, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn supervisor_snapshot_without_token_is_rejected() {
        let scratch = TestDir::new("home-sync-sup");
        let dir = scratch.path().to_path_buf();
        let state = AppState::new(
            LoadedHome {
                graph: HomeGraph::default(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        );
        let body = serde_json::from_value::<HomeSnapshot>(json!({
            "schema_version": HOME_SCHEMA_VERSION,
            "entities": [{"entity_id": "light.living", "name": "Living", "area_id": "living"}],
            "areas": [{"id": "living", "name": "Wohnzimmer"}],
            "assist": ["light.living"]
        }))
        .expect("snapshot json");
        let err = api_home(State(state), ConnectInfo("172.30.32.1:9".parse().unwrap()), HeaderMap::new(), Json(body)).await.unwrap_err();
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn supervisor_snapshot_with_token_is_accepted() {
        let scratch = TestDir::new("home-sync-sup-tok");
        let dir = scratch.path().to_path_buf();
        let state = AppState::new(
            LoadedHome {
                graph: HomeGraph::default(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            Some("secret".into()),
        );
        let body = serde_json::from_value::<HomeSnapshot>(json!({
            "schema_version": HOME_SCHEMA_VERSION,
            "entities": [{"entity_id": "light.living", "name": "Living", "area_id": "living"}],
            "areas": [{"id": "living", "name": "Wohnzimmer"}],
            "assist": ["light.living"]
        }))
        .expect("snapshot json");
        let mut headers = HeaderMap::new();
        headers.insert("x-klar-token", "secret".parse().unwrap());
        let ack =
            api_home(State(state.clone()), ConnectInfo("172.30.32.1:9".parse().unwrap()), headers, Json(body)).await.expect("accepted").0;
        assert_eq!(ack.entities, 1);
        assert!(state.live_sync.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn lan_snapshot_without_token_is_rejected() {
        let scratch = TestDir::new("home-sync-lan");
        let dir = scratch.path().to_path_buf();
        let state = AppState::new(
            LoadedHome {
                graph: HomeGraph::default(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        );
        let body = serde_json::from_value::<HomeSnapshot>(json!({
            "schema_version": HOME_SCHEMA_VERSION,
            "entities": [{"entity_id": "light.living", "name": "Living", "area_id": "living"}]
        }))
        .expect("snapshot json");
        let err = api_home(State(state), ConnectInfo("10.0.0.8:9".parse().unwrap()), HeaderMap::new(), Json(body)).await.unwrap_err();
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }
}

#[cfg(test)]
#[path = "home_sync_http_tests.rs"]
mod http_tests;
