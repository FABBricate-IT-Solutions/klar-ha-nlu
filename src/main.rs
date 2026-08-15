use clap::Parser;
use klar_nlu::compound::{apply_overlay, load_overlay};
use klar_nlu::lexicon::default_home;
use klar_nlu::registry::load_home;
use klar_nlu::session::Sessions;
use klar_nlu::types::CustomSentence;
use klar_nlu::web::{router, AppState};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(name = "klar", about = "Deutsche NLU für Home Assistant")]
struct Args {
    /// HTTP-UI und Parse-API
    #[arg(long, default_value = "0.0.0.0:10520")]
    http: String,
    /// Wyoming Intent-Server
    #[arg(long, default_value = "0.0.0.0:10500")]
    wyoming: String,
    /// Home-Assistant-Config (read-only), z. B. /config
    #[arg(long, default_value = "/config")]
    config_dir: PathBuf,
    /// Beschreibbares Verzeichnis für Kalibrierung. Addon: /data
    #[arg(long, default_value = "/data")]
    data_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "klar=info,klar_nlu=info".into()),
        )
        .init();

    let args = Args::parse();
    let data_dir = if args.data_dir.is_dir() {
        args.data_dir.clone()
    } else {
        args.config_dir.clone()
    };
    let mut home = load_home(&args.config_dir, default_home());
    let config_overlay = load_overlay(&args.config_dir);
    apply_overlay(&mut home, &config_overlay);
    let mut settings = config_overlay.settings.clone().unwrap_or_default();
    if data_dir != args.config_dir {
        let data_overlay = load_overlay(&data_dir);
        apply_overlay(&mut home, &data_overlay);
        if let Some(saved) = data_overlay.settings {
            settings = saved;
        }
    }
    let state = AppState {
        home: Arc::new(Mutex::new(home)),
        sessions: Arc::new(Mutex::new(Sessions::default())),
        settings: Arc::new(Mutex::new(settings)),
        custom: Arc::new(Mutex::new(Vec::<CustomSentence>::new())),
        data_dir,
    };

    let http_state = state.clone();
    let http_addr: SocketAddr = args.http.parse().expect("http bind");
    let http = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await.expect("http");
        tracing::info!("UI auf http://{http_addr}");
        axum::serve(listener, router(http_state)).await.expect("axum");
    });

    let wyoming_bind = args.wyoming.clone();
    let wy = tokio::spawn(async move {
        klar_nlu::wyoming::serve(
            &wyoming_bind,
            state.home.clone(),
            state.sessions.clone(),
            state.settings.clone(),
            state.custom.clone(),
        )
        .await
    });

    let _ = tokio::join!(http, wy);
}
