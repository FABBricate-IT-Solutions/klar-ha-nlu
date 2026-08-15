use clap::Parser;
use klar_nlu::io::{run, RuntimeArgs};
use std::path::PathBuf;

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
    run(RuntimeArgs {
        http: args.http,
        wyoming: args.wyoming,
        config_dir: args.config_dir,
        data_dir: args.data_dir,
        token: args.token,
        token_file: args.token_file,
    })
    .await;
}
