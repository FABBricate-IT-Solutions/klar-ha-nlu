use axum::http::{header, HeaderMap, Method, StatusCode};
use std::net::{IpAddr, SocketAddr};

/// Named HTTP capability checked before the request body is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Snapshot / parse / dry-run. Never persists calibration.
    Read,
    /// Overlay, settings, restore, and other mutating routes.
    Write,
    /// Confirm a teach proposal from Assist. Same gate as Write; kept distinct so the
    /// route table can prove parse is not a storing path.
    PersistParse,
}

/// Unwrap IPv4-mapped IPv6 (`::ffff:127.0.0.1` → `127.0.0.1`) so [::] listeners treat
/// IPv4 loopback and supervisor peers as local (A-08).
pub fn canonical_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(IpAddr::V6(v6)),
        other => other,
    }
}

pub fn request_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-klar-token")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .or_else(|| headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")))
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
    match canonical_ip(ip) {
        IpAddr::V4(v4) => {
            let oct = v4.octets();
            oct[0] == 172 && oct[1] == 30 && (oct[2] == 32 || oct[2] == 33)
        }
        IpAddr::V6(_) => false,
    }
}

pub fn loopback_peer(ip: IpAddr) -> bool {
    canonical_ip(ip).is_loopback()
}

pub fn trusted_peer(ip: IpAddr) -> bool {
    loopback_peer(ip) || supervisor_peer(ip)
}

/// Browser CSRF check: an `Origin` must match `Host`. No Origin means a non-browser client.
pub fn same_origin(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()).filter(|s| !s.is_empty()) else {
        return true;
    };
    if origin.eq_ignore_ascii_case("null") {
        return false;
    }
    let Some(host) = headers.get(header::HOST).and_then(|v| v.to_str().ok()).filter(|s| !s.is_empty()) else {
        return false;
    };
    origin_host(origin).is_some_and(|origin_host| origin_host.eq_ignore_ascii_case(host))
}

/// Loopback writers that send a Host must use a local one. Missing Host is fine for curl and
/// the integration; a foreign Host on loopback is not (A-02).
pub fn host_allowed(peer: Option<SocketAddr>, headers: &HeaderMap) -> bool {
    let Some(host) = headers.get(header::HOST).and_then(|v| v.to_str().ok()).filter(|s| !s.is_empty()) else {
        return true;
    };
    if peer.is_some_and(|addr| loopback_peer(addr.ip())) {
        return local_host(host);
    }
    true
}

fn origin_host(origin: &str) -> Option<&str> {
    let rest = origin.strip_prefix("http://").or_else(|| origin.strip_prefix("https://"))?;
    let host = rest.split('/').next().filter(|part| !part.is_empty())?;
    Some(host)
}

fn local_host(host: &str) -> bool {
    let bare = host.split_once('@').map_or(host, |(_, rest)| rest);
    let name = match bare.rfind(':') {
        Some(idx) if !bare.starts_with('[') => &bare[..idx],
        Some(idx) if bare.starts_with('[') && bare.as_bytes().get(idx.saturating_sub(1)) == Some(&b']') => &bare[1..idx - 1],
        _ if bare.starts_with('[') && bare.ends_with(']') => &bare[1..bare.len() - 1],
        _ => bare,
    };
    name.eq_ignore_ascii_case("localhost")
        || matches!(name, "127.0.0.1" | "::1" | "0.0.0.0")
        || name.parse::<IpAddr>().is_ok_and(loopback_peer)
}

pub fn writes_allowed(peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> bool {
    if !same_origin(headers) || !host_allowed(peer, headers) {
        return false;
    }
    if peer.is_some_and(|addr| loopback_peer(addr.ip())) {
        return true;
    }
    let Some(expected) = token.as_deref().filter(|s| !s.is_empty()) else {
        return false;
    };
    request_token(headers).is_some_and(|got| token_eq(got, expected))
}

pub fn reads_allowed(peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> bool {
    // Reads stay open for loopback and Supervisor; Origin/Host only gate writes (A-02).
    if peer.is_some_and(|addr| trusted_peer(addr.ip())) {
        return true;
    }
    let Some(expected) = token.as_deref().filter(|s| !s.is_empty()) else {
        return false;
    };
    request_token(headers).is_some_and(|got| token_eq(got, expected))
}

pub fn home_writes_allowed(peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> bool {
    writes_allowed(peer, headers, token)
}

pub fn wyoming_allowed(peer: SocketAddr) -> bool {
    trusted_peer(peer.ip())
}

/// Capability gate used by middleware and handlers. Cross-origin / bad Host → 403.
pub fn require(capability: Capability, peer: Option<SocketAddr>, headers: &HeaderMap, token: &Option<String>) -> Result<(), StatusCode> {
    match capability {
        Capability::Read => {
            if reads_allowed(peer, headers, token) {
                Ok(())
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Capability::Write | Capability::PersistParse => {
            if !same_origin(headers) || !host_allowed(peer, headers) {
                return Err(StatusCode::FORBIDDEN);
            }
            if writes_allowed(peer, headers, token) {
                Ok(())
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

/// Every authenticated HTTP API route and the capability checked before the body.
pub fn route_capability(method: &Method, path: &str) -> Option<Capability> {
    let path = path.trim_end_matches('/');
    ROUTES.iter().find_map(|(m, pattern, cap)| (*m == *method && path_matches(pattern, path)).then_some(*cap))
}

fn path_matches(pattern: &str, path: &str) -> bool {
    if !pattern.contains('{') {
        return pattern == path;
    }
    let mut pattern_parts = pattern.split('/');
    let mut path_parts = path.split('/');
    loop {
        match (pattern_parts.next(), path_parts.next()) {
            (None, None) => return true,
            (Some(expected), Some(_got)) if expected.starts_with('{') && expected.ends_with('}') => continue,
            (Some(expected), Some(got)) if expected == got => continue,
            _ => return false,
        }
    }
}

type RouteRow = (&'static Method, &'static str, Capability);

static ROUTES: &[RouteRow] = &[
    (&Method::GET, "/api/v2/setup/token", Capability::Read),
    (&Method::POST, "/api/v2/parse", Capability::Read),
    (&Method::GET, "/api/settings", Capability::Read),
    (&Method::POST, "/api/settings", Capability::Write),
    (&Method::PATCH, "/api/settings", Capability::Write),
    (&Method::GET, "/api/custom", Capability::Read),
    (&Method::POST, "/api/custom", Capability::Write),
    (&Method::GET, "/api/entities", Capability::Read),
    (&Method::POST, "/api/entities", Capability::PersistParse),
    (&Method::GET, "/api/gaps", Capability::Read),
    (&Method::GET, "/api/v2/policies", Capability::Read),
    (&Method::POST, "/api/v2/policies", Capability::Write),
    (&Method::GET, "/api/v2/policies/catalog", Capability::Read),
    (&Method::POST, "/api/v2/policies/evaluate", Capability::Read),
    (&Method::POST, "/api/v2/policies/preview-turns", Capability::Read),
    (&Method::GET, "/api/v2/policies/history", Capability::Read),
    (&Method::POST, "/api/v2/policies/rollback", Capability::Write),
    (&Method::GET, "/api/v2/policies/trainer-context", Capability::Read),
    (&Method::POST, "/api/v2/policies/propose/validate", Capability::Read),
    (&Method::GET, "/api/v2/settings/history", Capability::Read),
    (&Method::POST, "/api/v2/settings/rollback", Capability::Write),
    (&Method::POST, "/api/v2/speech/render", Capability::Read),
    (&Method::GET, "/api/v2/speech/asr_boost", Capability::Read),
    (&Method::GET, "/api/v2/settings/backup", Capability::Write),
    (&Method::POST, "/api/v2/settings/restore", Capability::Write),
    (&Method::GET, "/api/bundle", Capability::Read),
    (&Method::DELETE, "/api/bundle", Capability::Write),
    (&Method::GET, "/api/bundle/entries", Capability::Read),
    (&Method::POST, "/api/bundle/entries", Capability::Write),
    (&Method::GET, "/api/bundle/dataset", Capability::Read),
    (&Method::GET, "/api/bundle/protocol", Capability::Read),
    (&Method::POST, "/api/bundle/clear", Capability::Write),
    (&Method::GET, "/api/v2/conversations", Capability::Read),
    (&Method::GET, "/api/v2/conversations/live", Capability::Read),
    (&Method::GET, "/api/v2/conversations/{id}", Capability::Read),
    (&Method::POST, "/api/v2/conversations/mark-wrong", Capability::Write),
    (&Method::GET, "/api/v2/wrong-cases", Capability::Read),
    (&Method::GET, "/api/v2/wrong-cases/dataset", Capability::Read),
    (&Method::GET, "/api/v2/wrong-cases/dry-run", Capability::Read),
    (&Method::GET, "/api/v2/wrong-cases/update-dialog", Capability::Read),
    (&Method::POST, "/api/v2/wrong-cases/update-dialog", Capability::Write),
    (&Method::GET, "/api/v2/last-turn", Capability::Read),
    (&Method::GET, "/api/dashboard", Capability::Read),
    (&Method::GET, "/api/v2/result-metrics/export", Capability::Read),
    (&Method::GET, "/api/ui", Capability::Read),
    (&Method::POST, "/api/ui", Capability::Write),
    (&Method::POST, "/api/assignment/apply", Capability::Write),
    (&Method::POST, "/api/assignment/undo", Capability::Write),
    (&Method::POST, "/api/v2/home", Capability::Write),
    (&Method::POST, "/api/v2/session/undo-priors", Capability::Write),
    (&Method::GET, "/api/v2/languages", Capability::Read),
    (&Method::GET, "/api/v2/intents", Capability::Read),
    (&Method::GET, "/api/lang/overlay", Capability::Read),
    (&Method::POST, "/api/lang/overlay", Capability::Write),
    (&Method::POST, "/api/lang/preview", Capability::Read),
    (&Method::POST, "/api/lang/explain", Capability::Read),
    (&Method::POST, "/api/lang/rollback", Capability::Write),
    (&Method::GET, "/api/v2/llm/endpoint", Capability::Read),
    (&Method::POST, "/api/v2/llm/endpoint", Capability::Write),
    (&Method::GET, "/api/v2/llm/voice", Capability::Read),
    (&Method::POST, "/api/v2/llm/custom-voice", Capability::Write),
    (&Method::POST, "/api/v2/llm/models", Capability::Write),
    (&Method::POST, "/api/v2/llm/chat", Capability::Write),
    (&Method::POST, "/api/v2/llm/refine", Capability::Write),
    (&Method::POST, "/api/v2/llm/assist", Capability::Write),
    (&Method::POST, "/api/v2/policies/trainer/chat", Capability::Write),
    (&Method::POST, "/api/v2/policies/trainer/consent", Capability::Write),
];

#[allow(dead_code)] // exercised by auth route-table tests
pub fn all_routes() -> &'static [RouteRow] {
    ROUTES
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "auth_http_tests.rs"]
mod http_tests;
