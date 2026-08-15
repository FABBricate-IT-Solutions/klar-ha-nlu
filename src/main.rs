use clap::Parser;
use klar_nlu::compound::{apply_overlay, load_overlay};
use klar_nlu::lexicon::default_home;
use klar_nlu::registry::load_home;
use klar_nlu::session::Sessions;
use klar_nlu::types::{CustomSentence, HomeGraph};
use klar_nlu::web::{router, AppState};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "klar", about = "Deutsche NLU für Home Assistant")]
struct Args {
    /// HTTP-UI und Parse-API
    #[arg(long, default_value = "127.0.0.1:10520")]
    http: String,
    /// Wyoming Intent-Server
    #[arg(long, default_value = "127.0.0.1:10500")]
    wyoming: String,
    /// Home-Assistant-Config (read-only), z. B. /config
    #[arg(long, default_value = "/config")]
    config_dir: PathBuf,
    /// Beschreibbares Verzeichnis für Kalibrierung. Addon: /data
    #[arg(long, default_value = "/data")]
    data_dir: PathBuf,
    /// Shared secret for overlay writes from non-loopback clients (`KLAR_TOKEN`)
    #[arg(long)]
    token: Option<String>,
    /// Create or read a write token at this path when --token is empty
    #[arg(long)]
    token_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "klar=info,klar_nlu=info".into()))
        .init();

    let args = Args::parse();
    let data_dir = if args.data_dir.is_dir() { args.data_dir.clone() } else { args.config_dir.clone() };
    let token =
        resolve_token(args.token.or_else(|| std::env::var("KLAR_TOKEN").ok().filter(|s| !s.is_empty())), args.token_file.as_deref());
    if token.is_none() {
        tracing::warn!("Kein Token: HTTP-API nur von localhost");
    }
    let mut home = load_home(&args.config_dir, default_home());
    let config_overlay = load_overlay(&args.config_dir);
    apply_overlay(&mut home, &config_overlay);
    let mut settings = config_overlay.settings.clone().unwrap_or_default();
    let mut custom = config_overlay.custom.clone();
    if data_dir != args.config_dir {
        let data_overlay = load_overlay(&data_dir);
        apply_overlay(&mut home, &data_overlay);
        if let Some(saved) = data_overlay.settings {
            settings = saved;
        }
        if !data_overlay.custom.is_empty() {
            custom = data_overlay.custom;
        }
    }
    let state = AppState {
        home: Arc::new(Mutex::new(home)),
        sessions: Arc::new(Mutex::new(Sessions::default())),
        settings: Arc::new(Mutex::new(settings)),
        custom: Arc::new(Mutex::new(custom)),
        data_dir: data_dir.clone(),
        token,
    };

    let reload_state = state.clone();
    let config_dir = args.config_dir.clone();
    tokio::spawn(async move { reload_home(reload_state, config_dir, data_dir).await });

    let http_state = state.clone();
    let http_addr: SocketAddr = args.http.parse().expect("http bind");
    let http = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await.expect("http");
        tracing::info!("UI auf http://{http_addr}");
        axum::serve(listener, router(http_state).into_make_service_with_connect_info::<SocketAddr>()).await.expect("axum");
    });

    let wyoming_bind = args.wyoming.clone();
    let wy = tokio::spawn(async move {
        klar_nlu::wyoming::serve(&wyoming_bind, state.home.clone(), state.sessions.clone(), state.settings.clone(), state.custom.clone())
            .await
    });

    let _ = tokio::join!(http, wy);
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

fn registry_stamp(config_dir: &Path) -> String {
    ["core.entity_registry", "core.area_registry", "core.device_registry", "homeassistant.exposed_entities"]
        .into_iter()
        .map(|name| {
            let meta = std::fs::metadata(config_dir.join(".storage").join(name));
            match meta.and_then(|m| m.modified()) {
                Ok(time) => format!("{time:?}"),
                Err(_) => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn load_graph(config_dir: &Path, data_dir: &Path) -> (HomeGraph, Vec<CustomSentence>) {
    let mut home = load_home(config_dir, default_home());
    let config_overlay = load_overlay(config_dir);
    apply_overlay(&mut home, &config_overlay);
    let mut custom = config_overlay.custom;
    if data_dir != config_dir {
        let data_overlay = load_overlay(data_dir);
        apply_overlay(&mut home, &data_overlay);
        if !data_overlay.custom.is_empty() {
            custom = data_overlay.custom;
        }
    }
    (home, custom)
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
        let (home, custom) = load_graph(&config_dir, &data_dir);
        let n = home.entities.len();
        *state.home.lock().await = home;
        *state.custom.lock().await = custom;
        tracing::info!("Home-Graph neu geladen ({n} Entitäten)");
    }
}
