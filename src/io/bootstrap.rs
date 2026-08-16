use crate::home::overlay::{load_overlay, save_overlay};
use crate::home::{load_merged, registry_stamp};
use crate::io::state::AppState;
use crate::io::{web, wyoming};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

pub struct RuntimeArgs {
    pub http: String,
    pub wyoming: String,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub token: Option<String>,
    pub token_file: Option<PathBuf>,
}

pub async fn run(args: RuntimeArgs) {
    let _ = std::fs::create_dir_all(&args.data_dir);
    let data_dir = if args.data_dir.is_dir() { args.data_dir.clone() } else { args.config_dir.clone() };
    let token =
        resolve_token(args.token.or_else(|| std::env::var("KLAR_TOKEN").ok().filter(|s| !s.is_empty())), args.token_file.as_deref());
    if token.is_none() {
        tracing::warn!("Kein Token: HTTP-API nur von localhost und dem Supervisor-Netz");
    }

    let loaded = load_merged(&args.config_dir, &data_dir);
    let state = AppState::new(loaded, data_dir.clone(), token);
    enable_bundle_from_env(&state).await;

    let reload_state = state.clone();
    let config_dir = args.config_dir.clone();
    tokio::spawn(async move { reload_home(reload_state, config_dir, data_dir).await });

    let http_state = state.clone();
    let http_addr: SocketAddr = args.http.parse().expect("http bind");
    let http = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await.expect("http");
        tracing::info!("UI auf http://{http_addr}");
        axum::serve(listener, web::router(http_state).into_make_service_with_connect_info::<SocketAddr>()).await.expect("axum");
    });

    let wyoming_bind = args.wyoming.clone();
    let wy = tokio::spawn(async move { wyoming::serve(&wyoming_bind, state).await });

    let _ = tokio::join!(http, wy);
}

fn env_on(name: &str) -> bool {
    matches!(std::env::var(name).ok().as_deref().map(str::trim), Some("1" | "true" | "TRUE" | "yes" | "on"))
}

async fn enable_bundle_from_env(state: &AppState) {
    if !env_on("KLAR_SUPPORT_BUNDLE") {
        return;
    }
    if load_overlay(&state.data_dir).settings.is_some() {
        return;
    }
    let mut settings = state.settings.lock().await;
    if settings.support_bundle {
        return;
    }
    settings.support_bundle = true;
    let mut overlay = load_overlay(&state.data_dir);
    overlay.settings = Some(settings.clone());
    let _ = save_overlay(&state.data_dir, &overlay);
    tracing::info!("Support-Bundle an (KLAR_SUPPORT_BUNDLE)");
}

fn resolve_token(explicit: Option<String>, file: Option<&Path>) -> Option<String> {
    if let Some(token) = explicit.filter(|s| !s.is_empty()) {
        return Some(token);
    }
    let path = file?;
    if let Ok(existing) = std::fs::read_to_string(path) {
        let trimmed = existing.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::write(path, &token).is_ok() {
        tracing::warn!("Write-Token nach {} geschrieben", path.display());
        Some(token)
    } else {
        None
    }
}

async fn reload_home(state: AppState, config_dir: PathBuf, data_dir: PathBuf) {
    let mut stamp = registry_stamp(&config_dir);
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let next = registry_stamp(&config_dir);
        if next == stamp {
            continue;
        }
        stamp = next;
        let loaded = load_merged(&config_dir, &data_dir);
        let n = loaded.graph.entities.len();
        state.home.replace(loaded.graph).await;
        *state.custom.lock().await = loaded.custom;
        *state.settings.lock().await = loaded.settings;
        tracing::info!("Home-Graph neu geladen ({n} Entitäten)");
    }
}
