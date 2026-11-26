use super::*;
use axum::http::HeaderMap;
use std::net::SocketAddr;

#[test]
fn token_compare_rejects_prefix() {
    assert!(token_eq("secret", "secret"));
    assert!(!token_eq("secret", "secre"));
    assert!(!token_eq("secret", "secret!"));
}

#[test]
fn supervisor_ingress_cannot_write_without_klar_token() {
    let peer = "172.30.32.2:9".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("x-ingress-path", "/api/hassio_ingress/token".parse().unwrap());
    assert!(!writes_allowed(Some(peer), &headers, &None));
    assert!(!home_writes_allowed(Some(peer), &headers, &None));
    headers.insert("x-klar-token", "secret".parse().unwrap());
    assert!(writes_allowed(Some(peer), &headers, &Some("secret".into())));
    assert!(home_writes_allowed(Some(peer), &headers, &Some("secret".into())));
}

#[test]
fn plain_supervisor_write_still_needs_token() {
    let peer = "172.30.32.2:9".parse().unwrap();
    assert!(!writes_allowed(Some(peer), &HeaderMap::new(), &None));
    assert!(!home_writes_allowed(Some(peer), &HeaderMap::new(), &None));
    assert!(reads_allowed(Some(peer), &HeaderMap::new(), &None));
}

#[test]
fn loopback_writes_without_token() {
    let peer = "127.0.0.1:9".parse().unwrap();
    assert!(writes_allowed(Some(peer), &HeaderMap::new(), &None));
    assert!(home_writes_allowed(Some(peer), &HeaderMap::new(), &None));
}

#[test]
fn a08_mapped_ipv4_loopback_is_trusted() {
    let mapped: SocketAddr = "[::ffff:127.0.0.1]:9".parse().unwrap();
    assert!(loopback_peer(mapped.ip()));
    assert!(trusted_peer(mapped.ip()));
    assert!(writes_allowed(Some(mapped), &HeaderMap::new(), &None));
    let supervisor: SocketAddr = "[::ffff:172.30.32.2]:9".parse().unwrap();
    assert!(supervisor_peer(supervisor.ip()));
    assert!(reads_allowed(Some(supervisor), &HeaderMap::new(), &None));
    assert!(!writes_allowed(Some(supervisor), &HeaderMap::new(), &None));
}

#[test]
fn a02_cross_origin_loopback_write_is_forbidden() {
    let peer = "127.0.0.1:9".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("origin", "https://evil.example".parse().unwrap());
    headers.insert("host", "127.0.0.1:10520".parse().unwrap());
    assert!(!writes_allowed(Some(peer), &headers, &None));
    assert_eq!(require(Capability::Write, Some(peer), &headers, &None), Err(StatusCode::FORBIDDEN));
}

#[test]
fn ha_proxy_origin_with_write_token_is_allowed() {
    // HA sidebar iframe → /api/klar_nlu/ui → engine: Origin is the HA frontend,
    // Host is the engine. The proxy injects x-klar-token.
    let peer = "192.168.178.226:9".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("origin", "http://homeassistant.local:8123".parse().unwrap());
    headers.insert("host", "192.168.178.15:10520".parse().unwrap());
    headers.insert("x-klar-token", "secret".parse().unwrap());
    assert!(writes_allowed(Some(peer), &headers, &Some("secret".into())));
    assert!(require(Capability::Write, Some(peer), &headers, &Some("secret".into())).is_ok());
}

#[test]
fn same_origin_loopback_write_still_works() {
    let peer = "127.0.0.1:9".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("origin", "http://127.0.0.1:10520".parse().unwrap());
    headers.insert("host", "127.0.0.1:10520".parse().unwrap());
    assert!(writes_allowed(Some(peer), &headers, &None));
    assert!(require(Capability::Write, Some(peer), &headers, &None).is_ok());
}

#[test]
fn foreign_host_on_loopback_is_forbidden() {
    let peer = "127.0.0.1:9".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("host", "evil.example".parse().unwrap());
    assert!(!host_allowed(Some(peer), &headers));
    assert_eq!(require(Capability::Write, Some(peer), &headers, &None), Err(StatusCode::FORBIDDEN));
}

#[test]
fn parse_is_read_not_persist_parse() {
    assert_eq!(route_capability(&Method::POST, "/api/v2/parse"), Some(Capability::Read));
    assert_eq!(route_capability(&Method::POST, "/api/entities"), Some(Capability::PersistParse));
}

#[test]
fn every_listed_route_has_a_capability() {
    assert!(!all_routes().is_empty());
    for (method, path, expected) in all_routes() {
        let path = path.replace("{id}", "abc");
        assert_eq!(route_capability(method, &path), Some(*expected), "{method} {path}");
    }
}
