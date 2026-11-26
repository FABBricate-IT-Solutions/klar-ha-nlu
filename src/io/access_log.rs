//! One structured access line per HTTP request (package 3.10).

use crate::io::auth::{loopback_peer, route_capability, supervisor_peer, Capability};
use crate::io::state::AppState;
use crate::ops;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

/// Peer class for access logs: loopback, Supervisor Docker net, or other.
pub fn peer_class(ip: IpAddr) -> &'static str {
    if loopback_peer(ip) {
        "loopback"
    } else if supervisor_peer(ip) {
        "supervisor"
    } else {
        "other"
    }
}

fn gate_label(capability: Option<Capability>) -> &'static str {
    match capability {
        Some(Capability::Read) => "read",
        Some(Capability::Write) => "write",
        Some(Capability::PersistParse) => "persist",
        None => "public",
    }
}

fn peer_from(request: &Request) -> Option<SocketAddr> {
    request.extensions().get::<axum::extract::ConnectInfo<SocketAddr>>().map(|info| info.0)
}

/// Outer middleware: log route, peer class, auth gate, status and duration, then bump ops counters.
pub async fn access_log(State(_state): State<AppState>, request: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let peer = peer_from(&request);
    let peer = peer.map(|addr| peer_class(addr.ip())).unwrap_or("unknown");
    let gate = gate_label(route_capability(&method, &path));
    let route = format!("{method} {path}");

    let response = next.run(request).await;
    let status = response.status().as_u16();
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    ops::ops().record_request(duration_ms);
    tracing::info!(
        target: "klar_nlu::access",
        route = %route,
        peer = %peer,
        gate = %gate,
        status,
        duration_ms,
        "request"
    );
    response
}

/// Gate name used in tests for a method/path pair.
#[cfg(test)]
fn gate_for(method: &axum::http::Method, path: &str) -> &'static str {
    gate_label(route_capability(method, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::test_dir::TestDir;
    use crate::home::LoadedHome;
    use crate::types::{HomeGraph, Settings};
    use axum::body::Body;
    use axum::extract::ConnectInfo;
    use axum::http::{Method, Request, StatusCode};
    use std::io::{self, Write};
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct LogBuf(Arc<Mutex<Vec<u8>>>);

    impl Write for LogBuf {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("log buf").extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for LogBuf {
        type Writer = LogBuf;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn state() -> (AppState, TestDir) {
        let scratch = TestDir::new("access-log");
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
            scratch.path().to_path_buf(),
            Some("secret".into()),
        );
        (state, scratch)
    }

    #[tokio::test]
    async fn request_emits_structured_access_line_and_moves_counters() {
        let (state, _scratch) = state();
        let before = ops::ops().snapshot();
        let buf = LogBuf::default();
        let subscriber = tracing_subscriber::fmt().with_writer(buf.clone()).with_max_level(tracing::Level::INFO).finish();
        let _guard = tracing::subscriber::set_default(subscriber);
        let mut request = Request::get("/api/v2/health").body(Body::empty()).unwrap();
        request.extensions_mut().insert(ConnectInfo("127.0.0.1:9".parse::<SocketAddr>().unwrap()));
        let response = crate::io::web::router(state).oneshot(request).await.unwrap();
        drop(_guard);
        assert_eq!(response.status(), StatusCode::OK);
        let after = ops::ops().snapshot();
        assert!(after.requests > before.requests);
        assert!(after.latency_total_ms >= before.latency_total_ms);
        let text = String::from_utf8_lossy(&buf.0.lock().expect("log buf")).to_string();
        assert!(text.contains("request"), "{text}");
        assert!(text.contains("route=\"GET /api/v2/health\"") || text.contains("GET /api/v2/health"), "{text}");
        assert!(text.contains("peer=\"loopback\"") || text.contains("loopback"), "{text}");
        assert!(text.contains("gate=\"public\"") || text.contains("public"), "{text}");
        assert!(text.contains("status=200") || text.contains("status: 200"), "{text}");
        assert!(text.contains("duration_ms"), "{text}");
    }

    #[test]
    fn gate_labels_match_capabilities() {
        assert_eq!(gate_for(&Method::GET, "/api/v2/health"), "public");
        assert_eq!(gate_for(&Method::POST, "/api/v2/parse"), "read");
        assert_eq!(gate_for(&Method::POST, "/api/settings"), "write");
        assert_eq!(gate_for(&Method::POST, "/api/entities"), "persist");
    }
}
