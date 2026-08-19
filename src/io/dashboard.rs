use crate::home::assignment::{build_dashboard, Dashboard};
use crate::home::overlay::{apply_overlay, load_overlay, save_overlay, UiApplyRow, UiState};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::state::AppState;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/dashboard", get(get_dashboard))
        .route("/api/ui", get(get_ui).post(set_ui))
        .route("/api/assignment/apply", post(apply_suggestions))
        .route("/api/assignment/undo", post(undo_apply))
}

async fn get_dashboard(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Dashboard>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let home = state.home.snapshot().await;
    let overlay = load_overlay(&state.data_dir);
    Ok(Json(build_dashboard(
        &home,
        &state.bundle.load(),
        &overlay.ui.dismissed,
        state.metrics.snapshot(),
        state.catalog_for_settings().await,
    )))
}

async fn get_ui(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<UiState>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(load_overlay(&state.data_dir).ui))
}

async fn set_ui(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<UiState>,
) -> Result<Json<UiState>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let mut overlay = load_overlay(&state.data_dir);
    overlay.ui = sanitize_ui(body);
    let _ = save_overlay(&state.data_dir, &overlay);
    Ok(Json(overlay.ui))
}

fn sanitize_ui(mut ui: UiState) -> UiState {
    if ui.locale != "de" && ui.locale != "en" {
        ui.locale = "de".into();
    }
    if ui.tab.is_empty() || ui.tab.len() > 32 {
        ui.tab = "home".into();
    }
    ui.dismissed.retain(|id| valid_entity_id(id));
    ui.dismissed.sort();
    ui.dismissed.dedup();
    ui.last_apply.retain(|row| valid_entity_id(&row.entity_id) && valid_area(&row.after));
    ui.graph.retain(|id, point| valid_entity_id(id) && point.x.is_finite() && point.y.is_finite());
    ui
}

fn valid_entity_id(id: &str) -> bool {
    let mut parts = id.split('.');
    matches!((parts.next(), parts.next(), parts.next()), (Some(d), Some(n), None) if !d.is_empty() && !n.is_empty() && id.len() <= 128)
}

fn valid_area(area: &str) -> bool {
    !area.is_empty() && area.len() <= 128 && area.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[derive(Serialize)]
struct ApplyOut {
    applied: usize,
    rows: Vec<UiApplyRow>,
}

async fn apply_suggestions(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<ApplyOut>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let home = state.home.snapshot().await;
    let mut overlay = load_overlay(&state.data_dir);
    let dashboard =
        build_dashboard(&home, &state.bundle.load(), &overlay.ui.dismissed, state.metrics.snapshot(), state.catalog_for_settings().await);
    let mut rows = Vec::new();
    for row in dashboard.assignment {
        let Some(suggestion) = row.suggested_area else {
            continue;
        };
        if suggestion.score < 3 || row.area.as_deref() == Some(suggestion.area_id.as_str()) {
            continue;
        }
        overlay.areas.insert(row.entity_id.clone(), suggestion.area_id.clone());
        rows.push(UiApplyRow { entity_id: row.entity_id, before: row.area, after: suggestion.area_id });
    }
    overlay.ui.last_apply = rows.clone();
    let _ = save_overlay(&state.data_dir, &overlay);
    apply_runtime_overlay(&state, overlay).await;
    Ok(Json(ApplyOut { applied: rows.len(), rows }))
}

async fn undo_apply(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<ApplyOut>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let mut overlay = load_overlay(&state.data_dir);
    let rows = overlay.ui.last_apply.clone();
    for row in &rows {
        overlay.areas.insert(row.entity_id.clone(), row.before.clone().unwrap_or_default());
    }
    overlay.ui.last_apply.clear();
    let _ = save_overlay(&state.data_dir, &overlay);
    apply_runtime_overlay(&state, overlay).await;
    Ok(Json(ApplyOut { applied: rows.len(), rows }))
}

async fn apply_runtime_overlay(state: &AppState, overlay: crate::home::overlay::Overlay) {
    state
        .home
        .edit(|next| {
            apply_overlay(next, &overlay);
            Some(())
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_ui_locale_and_ids() {
        let ui = sanitize_ui(UiState {
            tab: String::new(),
            locale: "fr".into(),
            dismissed: vec!["../x".into(), "light.ok".into()],
            ..Default::default()
        });
        assert_eq!(ui.tab, "home");
        assert_eq!(ui.locale, "de");
        assert_eq!(ui.dismissed, vec!["light.ok"]);
    }
}
