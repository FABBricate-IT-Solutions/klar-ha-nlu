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

pub fn writes_allowed(peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> bool {
    if peer.is_some_and(|addr| addr.ip().is_loopback()) {
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
}
