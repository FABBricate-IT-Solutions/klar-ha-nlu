#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use klar_nlu::io::{run, RuntimeArgs};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "klar", about = "Deutsche NLU für Home Assistant")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
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
    /// External BCP-47 language packs (`KLAR_LANG_DIR`)
    #[arg(long)]
    lang_dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Probe the local HTTP health document (container HEALTHCHECK; no shell)
    Healthcheck {
        /// Host:port of the HTTP listener
        #[arg(long, default_value = "127.0.0.1:10520")]
        http: String,
    },
    /// Validate, preview, or import language packs
    Lang {
        #[command(subcommand)]
        command: LangCommand,
    },
    /// Held-out scorecard and process benches
    Eval {
        #[command(subcommand)]
        command: EvalCommand,
    },
    /// One-shot V1 overlay import
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
}

#[derive(Subcommand, Debug)]
enum LangCommand {
    Validate {
        path: PathBuf,
    },
    Preview {
        #[arg(long)]
        text: String,
        #[arg(long)]
        language: String,
        #[arg(long)]
        pack: Option<PathBuf>,
        #[arg(long)]
        pack_dir: Option<PathBuf>,
    },
    ImportHassil {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        into: Option<PathBuf>,
        #[arg(long)]
        language: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Append a phrase→intent case to packs/assist/{language}.yaml (one-file pack fix).
    NewCase {
        #[arg(long)]
        language: String,
        #[arg(long)]
        phrase: String,
        #[arg(long)]
        intent: String,
        /// Slot as key=value; repeatable
        #[arg(long = "slot")]
        slots: Vec<String>,
        #[arg(long)]
        into: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum EvalCommand {
    Scorecard {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Gate {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Bench {
        #[arg(long, default_value = "en")]
        language: String,
        #[arg(long, default_value_t = 128)]
        repeat: u32,
    },
}

#[derive(Subcommand, Debug)]
enum MigrateCommand {
    Import {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        into: Option<PathBuf>,
        #[arg(long)]
        home: Option<PathBuf>,
        #[arg(long)]
        apply: bool,
    },
}

/// How long work that is still running in the background gets after the engine has stopped.
const RUNTIME_SHUTDOWN: Duration = Duration::from_millis(300);

/// One HTTP GET against `/api/v2/health` — used as the image HEALTHCHECK on scratch/distroless.
fn probe_health(http: &str) -> Result<(), String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration as StdDuration;

    let mut stream = TcpStream::connect(http).map_err(|err| err.to_string())?;
    stream.set_read_timeout(Some(StdDuration::from_secs(3))).map_err(|err| err.to_string())?;
    stream.set_write_timeout(Some(StdDuration::from_secs(3))).map_err(|err| err.to_string())?;
    let req = format!("GET /api/v2/health HTTP/1.1\r\nHost: {http}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).map_err(|err| err.to_string())?;
    let mut body = String::new();
    stream.read_to_string(&mut body).map_err(|err| err.to_string())?;
    if body.contains("\"ok\":true") || body.contains("\"ok\": true") {
        Ok(())
    } else {
        Err(format!("unexpected response from {http}"))
    }
}

fn main() -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("tokio runtime: {err}");
            return ExitCode::FAILURE;
        }
    };
    let code = runtime.block_on(run_command());
    // Dropping the runtime would wait for blocking work, a name lookup for example, without
    // limit. A stop must not hang on that.
    runtime.shutdown_timeout(RUNTIME_SHUTDOWN);
    code
}

async fn run_command() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "klar=info,klar_nlu=info".into()))
        .init();

    let args = Args::parse();
    if let Some(Command::Healthcheck { http }) = args.command {
        return match probe_health(&http) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("healthcheck: {err}");
                ExitCode::FAILURE
            }
        };
    }
    if let Some(Command::Lang { command }) = args.command {
        let result = match command {
            LangCommand::Validate { path } => klar_nlu::lang::validate_path(&path),
            LangCommand::Preview { text, language, pack, pack_dir } => {
                klar_nlu::lang::preview(&text, &language, pack.as_deref(), pack_dir.as_deref())
            }
            LangCommand::ImportHassil { from, into, language, dry_run } => {
                klar_nlu::lang::import_hassil(&from, into.as_deref(), language.as_deref(), dry_run)
            }
            LangCommand::NewCase { language, phrase, intent, slots, into } => {
                klar_nlu::lang::new_case(&language, &phrase, &intent, &slots, into.as_deref())
            }
        };
        return match result {
            Ok(out) => {
                println!("{out}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
        };
    }
    if let Some(Command::Eval { command }) = args.command {
        let result = match command {
            EvalCommand::Scorecard { out } => klar_nlu::eval::run_cli(klar_nlu::eval::EvalCommand::Scorecard { out }),
            EvalCommand::Gate { out } => klar_nlu::eval::run_cli(klar_nlu::eval::EvalCommand::Gate { out }),
            EvalCommand::Bench { language, repeat } => klar_nlu::eval::run_cli(klar_nlu::eval::EvalCommand::Bench { language, repeat }),
        };
        return match result {
            Ok(out) => {
                println!("{out}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
        };
    }
    if let Some(Command::Migrate { command }) = args.command {
        let result = match command {
            MigrateCommand::Import { from, into, home, apply } => {
                klar_nlu::migrate::run_cli(&from, into.as_deref(), home.as_deref(), apply)
            }
        };
        return match result {
            Ok(out) => {
                println!("{out}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
        };
    }

    let ended = run(RuntimeArgs {
        http: args.http,
        wyoming: args.wyoming,
        config_dir: args.config_dir,
        data_dir: args.data_dir,
        token: args.token,
        token_file: args.token_file,
        lang_dir: args.lang_dir,
    })
    .await;
    match ended {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!("Klar beendet sich: {err}");
            ExitCode::FAILURE
        }
    }
}
