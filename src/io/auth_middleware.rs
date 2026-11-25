//! Capability middleware: reject before the body extractor runs (A-03).

use crate::io::auth::{require, route_capability};
use crate::io::state::AppState;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::net::SocketAddr;

pub async fn authorize(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    if let Some(capability) = route_capability(&method, &path) {
        require(capability, Some(peer), request.headers(), &state.token)?;
    }
    Ok(next.run(request).await)
}
