use axum::http::HeaderMap;
use std::net::{IpAddr, SocketAddr};

pub fn request_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-klar-token")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .or_else(|| headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")))
}

pub fn token_eq(provided: &str, expected: &str) -> bool {
    let left = provided.as_bytes();
    let right = expected.as_bytes();
    let len = left.len().max(right.len());
    let mut diff = (left.len() ^ right.len()) as u8;
    for i in 0..len {
        diff |= left.get(i).copied().unwrap_or(0) ^ right.get(i).copied().unwrap_or(0);
    }
    diff == 0
}

pub fn supervisor_peer(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let oct = v4.octets();
            oct[0] == 172 && oct[1] == 30 && (oct[2] == 32 || oct[2] == 33)
        }
        IpAddr::V6(_) => false,
    }
}

pub fn trusted_peer(ip: IpAddr) -> bool {
    ip.is_loopback() || supervisor_peer(ip)
}

fn ingress_request(headers: &HeaderMap) -> bool {
    headers.contains_key("x-ingress-path") || headers.contains_key("x-hass-source") || headers.contains_key("x-supervisor-ingress")
}

pub fn writes_allowed(peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> bool {
    if peer.is_some_and(|addr| addr.ip().is_loopback()) {
        return true;
    }
    if peer.is_some_and(|addr| supervisor_peer(addr.ip())) && ingress_request(headers) {
        return true;
    }
    let Some(expected) = token.as_deref().filter(|s| !s.is_empty()) else {
        return false;
    };
    request_token(headers).is_some_and(|got| token_eq(got, expected))
}

pub fn reads_allowed(peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> bool {
    writes_allowed(peer, headers, token) || peer.is_some_and(|addr| supervisor_peer(addr.ip()))
}

pub fn home_writes_allowed(peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> bool {
    reads_allowed(peer, headers, token)
}

pub fn wyoming_allowed(peer: SocketAddr) -> bool {
    trusted_peer(peer.ip())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_compare_rejects_prefix() {
        assert!(token_eq("secret", "secret"));
        assert!(!token_eq("secret", "secre"));
        assert!(!token_eq("secret", "secret!"));
    }

    #[test]
    fn supervisor_ingress_can_write_without_klar_token() {
        let peer = "172.30.32.2:9".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("x-ingress-path", "/api/hassio_ingress/token".parse().unwrap());
        assert!(writes_allowed(Some(peer), &headers, &None));
    }

    #[test]
    fn plain_supervisor_write_still_needs_token() {
        let peer = "172.30.32.2:9".parse().unwrap();
        assert!(!writes_allowed(Some(peer), &HeaderMap::new(), &None));
        assert!(home_writes_allowed(Some(peer), &HeaderMap::new(), &None));
    }
}
