//! Exercise the assembled production router, including auth and body extractors.
use crate::home::snapshot::*;
use crate::home::test_dir::TestDir;
use crate::home::LoadedHome;
use crate::io::limits::MAX_HOME_SNAPSHOT_BYTES;
use crate::io::{state::AppState, web};
use crate::types::{HomeGraph, Settings};
use axum::body::{to_bytes, Body, Bytes};
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tower::ServiceExt;

struct Fixture {
    state: AppState,
    _scratch: TestDir,
}

impl Fixture {
    fn new() -> Self {
        let scratch = TestDir::new("home-http");
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
            Some("home-test-token".into()),
        );
        Self { state, _scratch: scratch }
    }

    async fn post(&self, path: &str, bytes: Vec<u8>, streamed: bool) -> axum::response::Response {
        let body = if streamed {
            // No Content-Length and an unknown size hint, like a chunked request.
            let chunks: Vec<_> = bytes.chunks(16 * 1024).map(|chunk| Ok::<_, std::io::Error>(Bytes::copy_from_slice(chunk))).collect();
            Body::from_stream(tokio_stream::iter(chunks))
        } else {
            Body::from(bytes)
        };
        let mut request =
            Request::post(path).header("content-type", "application/json").header("x-klar-token", "home-test-token").body(body).unwrap();
        request.extensions_mut().insert(ConnectInfo("10.0.0.8:9".parse::<SocketAddr>().unwrap()));
        web::router(self.state.clone()).oneshot(request).await.unwrap()
    }
}

fn large_snapshot(count: usize) -> Value {
    let entities: Vec<_> = (0..count)
        .map(|index| {
            json!({
                "entity_id": format!("light.fixture_{index}"),
                "name": format!("Fixture {index}"), "original_name": "Original fixture name ".repeat(4),
                "has_entity_name": true, "area_id": null, "device_id": format!("device_{}", index % 300),
                "platform": "fixture", "aliases": [format!("fixture alias {index}"), "alias ".repeat(12)],
                "labels": ["lighting"], "disabled": false,
            })
        })
        .collect();
    json!({
        "schema_version": HOME_SCHEMA_VERSION,
        "entities": entities,
        "devices": (0..300).map(|index| json!({"id": format!("device_{index}"), "name": format!("Device {index}"),
            "name_by_user": null, "area_id": format!("area_{}", index % 24)})).collect::<Vec<_>>(),
        "areas": (0..24).map(|index| json!({"id": format!("area_{index}"), "name": format!("Area {index}"),
            "aliases": [format!("room {index}")], "floor_id": format!("floor_{}", index % 4)})).collect::<Vec<_>>(),
        "floors": (0..4).map(|index| json!({"floor_id": format!("floor_{index}"), "name": format!("Floor {index}"),
            "aliases": [], "level": index})).collect::<Vec<_>>(),
        "labels": [{"label_id": "lighting", "name": "Lighting"}],
        "assist": (0..count).step_by(2).map(|index| format!("light.fixture_{index}")).collect::<Vec<_>>(),
        "registered_intents": ["FixtureIntent"]
    })
}

#[tokio::test]
async fn full_large_snapshots_sync_without_truncation_through_production_router() {
    for count in [2216, MAX_SNAPSHOT_ENTITIES] {
        let fixture = Fixture::new();
        let snapshot = large_snapshot(count);
        assert!(serde_json::to_vec(&snapshot["entities"]).unwrap().len() > 538_682);
        let bytes = serde_json::to_vec(&snapshot).unwrap();
        assert!(bytes.len() < MAX_HOME_SNAPSHOT_BYTES);
        let response = fixture.post("/api/v2/home", bytes, false).await;
        assert_eq!(response.status(), StatusCode::OK);
        let ack: Value = serde_json::from_slice(&to_bytes(response.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(ack["entities"], count);
        assert_eq!(ack["areas"], 24);
        assert_eq!(ack["floors"], 4);
        assert_eq!(ack["assist"], count / 2);
        assert!(fixture.state.live_sync.load(Ordering::Relaxed));
        let home = fixture.state.home.snapshot().await;
        assert_eq!(home.entities.len(), count);
        for (index, entity) in home.entities.iter().enumerate() {
            assert_eq!(entity.entity_id, format!("light.fixture_{index}"));
            assert_eq!(entity.area.as_deref(), Some(format!("area_{}", index % 300 % 24).as_str()));
            assert_eq!(entity.aliases.len(), 2);
            assert!(entity.tags.iter().any(|tag| tag == "Lighting"));
        }
        assert_eq!(home.areas[0].floor_id.as_deref(), Some("floor_0"));
        assert_eq!(home.registered_intents, ["FixtureIntent"]);
    }
}

#[tokio::test]
async fn body_byte_boundary_is_enforced_for_fixed_and_streamed_requests() {
    for streamed in [false, true] {
        for extra in [0, 1] {
            let fixture = Fixture::new();
            let before = fixture.state.home.snapshot().await;
            let mut bytes = serde_json::to_vec(&json!({"schema_version": HOME_SCHEMA_VERSION})).unwrap();
            bytes.resize(MAX_HOME_SNAPSHOT_BYTES + extra, b' ');
            let response = fixture.post("/api/v2/home", bytes, streamed).await;
            assert_eq!(
                response.status(),
                if extra == 0 { StatusCode::OK } else { StatusCode::PAYLOAD_TOO_LARGE },
                "streamed={streamed} extra={extra}"
            );
            if extra > 0 {
                assert!(Arc::ptr_eq(&before, &fixture.state.home.snapshot().await));
                assert!(!fixture.state.live_sync.load(Ordering::Relaxed));
            }
        }
    }
}

#[tokio::test]
async fn invalid_snapshots_keep_the_last_successful_graph_and_live_sync_flag() {
    let fixture = Fixture::new();
    assert_eq!(fixture.post("/api/v2/home", serde_json::to_vec(&large_snapshot(2216)).unwrap(), false).await.status(), StatusCode::OK);
    let before = fixture.state.home.snapshot().await;
    for (body, expected) in [
        (json!({"schema_version": "unsupported"}), StatusCode::BAD_REQUEST),
        (json!({"schema_version": HOME_SCHEMA_VERSION, "extra": true}), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({"schema_version": HOME_SCHEMA_VERSION, "entities": [{"entity_id": "bad-id"}]}), StatusCode::BAD_REQUEST),
        (
            json!({"schema_version": HOME_SCHEMA_VERSION, "areas": [{"id": "room", "name": "x".repeat(MAX_NAME_CHARS + 1)}]}),
            StatusCode::BAD_REQUEST,
        ),
    ] {
        assert_eq!(fixture.post("/api/v2/home", serde_json::to_vec(&body).unwrap(), false).await.status(), expected);
        assert!(Arc::ptr_eq(&before, &fixture.state.home.snapshot().await));
        assert!(fixture.state.live_sync.load(Ordering::Relaxed));
    }
    let response = fixture.post("/api/v2/home", b"{broken-json".to_vec(), false).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(Arc::ptr_eq(&before, &fixture.state.home.snapshot().await));
}

#[tokio::test]
async fn over_limit_collections_are_capped_instead_of_rejecting_the_graph() {
    let fixture = Fixture::new();
    // R-10: filter then cap — oversized keep_domain lists sync, truncated to the cap.
    let mut body = json!({"schema_version": HOME_SCHEMA_VERSION});
    body["entities"] = Value::Array(vec![json!({"entity_id": "light.fixture"}); MAX_SNAPSHOT_ENTITIES + 1]);
    let response = fixture.post("/api/v2/home", serde_json::to_vec(&body).unwrap(), false).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(fixture.state.home.snapshot().await.entities.len(), MAX_SNAPSHOT_ENTITIES);
    assert!(fixture.state.live_sync.load(Ordering::Relaxed));

    let long_alias = "x".repeat(MAX_ALIAS_CHARS + 20);
    let body = json!({
        "schema_version": HOME_SCHEMA_VERSION,
        "entities": [{"entity_id": "light.fixture", "aliases": [long_alias.clone(), "ok"]}]
    });
    assert_eq!(fixture.post("/api/v2/home", serde_json::to_vec(&body).unwrap(), false).await.status(), StatusCode::OK);
    let home = fixture.state.home.snapshot().await;
    assert_eq!(home.entities[0].aliases[0].chars().count(), MAX_ALIAS_CHARS);
    assert!(home.entities[0].aliases.iter().any(|alias| alias == "ok"));
}

#[tokio::test]
async fn other_json_routes_keep_their_small_body_limit() {
    let fixture = Fixture::new();
    for path in ["/api/v2/parse", "/api/settings", "/api/custom"] {
        let mut bytes = b"{}".to_vec();
        bytes.resize(16 * 1024 + 1, b' ');
        assert_eq!(fixture.post(path, bytes, false).await.status(), StatusCode::PAYLOAD_TOO_LARGE, "{path}");
    }
}

#[tokio::test]
async fn unauthorized_large_snapshot_is_rejected_before_reading_the_body() {
    let fixture = Fixture::new();
    let before = fixture.state.home.snapshot().await;
    for (peer, token) in [("10.0.0.8:9", None), ("172.30.32.1:9", None), ("10.0.0.8:9", Some("wrong-token"))] {
        let polled = Arc::new(AtomicBool::new(false));
        let seen = polled.clone();
        let body = Body::from_stream(futures_util::stream::poll_fn(move |_| {
            seen.store(true, Ordering::Relaxed);
            std::task::Poll::Ready(Some(Err::<Bytes, _>(std::io::Error::other("unauthorized body must not be read"))))
        }));
        let mut builder =
            Request::post("/api/v2/home").header("content-type", "application/json").header("content-length", MAX_HOME_SNAPSHOT_BYTES + 1);
        if let Some(token) = token {
            builder = builder.header("x-klar-token", token);
        }
        let mut request = builder.body(body).unwrap();
        request.extensions_mut().insert(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
        let response = web::router(fixture.state.clone()).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(!polled.load(Ordering::Relaxed));
        assert!(Arc::ptr_eq(&before, &fixture.state.home.snapshot().await));
        assert!(!fixture.state.live_sync.load(Ordering::Relaxed));
    }
}

#[tokio::test]
async fn gaps_omits_the_overlay() {
    let fixture = Fixture::new();
    let mut request = Request::get("/api/gaps").header("x-klar-token", "home-test-token").body(Body::empty()).unwrap();
    request.extensions_mut().insert(ConnectInfo("10.0.0.8:9".parse::<SocketAddr>().unwrap()));
    let response = web::router(fixture.state.clone()).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert!(body.get("leftover").is_some(), "{body}");
    assert!(body.get("rooms").is_some(), "{body}");
    assert!(body.get("overlay").is_none(), "U-33: /api/gaps must not ship the overlay: {body}");
}
