use crate::home::overlay::{apply_overlay, load_overlay};
use crate::home::snapshot::{ingest, HomeSnapshot, SnapshotError};
use crate::io::auth::writes_allowed;
use crate::io::limits::MAX_HOME_SNAPSHOT_BYTES;
use crate::io::state::AppState;
use crate::types::HomeGraph;
use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use std::net::SocketAddr;
use std::sync::atomic::Ordering;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v2/home", post(api_home)).layer(DefaultBodyLimit::max(MAX_HOME_SNAPSHOT_BYTES))
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
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let ingested = match ingest(body) {
        Ok(graph) => graph,
        Err(SnapshotError::Caps) => return Err(StatusCode::PAYLOAD_TOO_LARGE),
        Err(SnapshotError::Schema | SnapshotError::Malformed(_)) => return Err(StatusCode::BAD_REQUEST),
    };
    let graph = apply_live_overlays(&state, ingested);
    let ack = HomeAck {
        entities: graph.entities.len(),
        areas: graph.areas.len(),
        floors: graph.floors.len(),
        assist: graph.assist.as_ref().map(std::collections::HashSet::len),
    };
    state.home.replace(graph).await;
    state.live_sync.store(true, Ordering::Relaxed);
    Ok(Json(ack))
}

fn apply_live_overlays(state: &AppState, mut graph: HomeGraph) -> HomeGraph {
    apply_overlay(&mut graph, &load_overlay(&state.config_dir));
    if state.data_dir != state.config_dir {
        apply_overlay(&mut graph, &load_overlay(&state.data_dir));
    }
    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::snapshot::HOME_SCHEMA_VERSION;
    use crate::home::LoadedHome;
    use crate::types::Settings;
    use serde_json::json;

    #[tokio::test]
    async fn snapshot_replaces_graph_and_enables_live_sync() {
        let dir = std::env::temp_dir().join(format!("klar-home-sync-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let state = AppState::new(
            LoadedHome { graph: HomeGraph::default(), settings: Settings::default(), custom: Vec::new(), language: Default::default() },
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
    }

    #[tokio::test]
    async fn malformed_snapshot_is_rejected() {
        let dir = std::env::temp_dir().join(format!("klar-home-sync-bad-{}", std::process::id()));
        let state = AppState::new(
            LoadedHome { graph: HomeGraph::default(), settings: Settings::default(), custom: Vec::new(), language: Default::default() },
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
}
