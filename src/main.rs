#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use klar_nlu::io::{run, RuntimeArgs};
use std::path::PathBuf;
use std::process::ExitCode;

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

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "klar=info,klar_nlu=info".into()))
        .init();

    let args = Args::parse();
    if let Some(Command::Lang { command }) = args.command {
        let result = match command {
            LangCommand::Validate { path } => klar_nlu::lang::validate_path(&path),
            LangCommand::Preview { text, language, pack, pack_dir } => {
                klar_nlu::lang::preview(&text, &language, pack.as_deref(), pack_dir.as_deref())
            }
            LangCommand::ImportHassil { from, into, language, dry_run } => {
                klar_nlu::lang::import_hassil(&from, into.as_deref(), language.as_deref(), dry_run)
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

    run(RuntimeArgs {
        http: args.http,
        wyoming: args.wyoming,
        config_dir: args.config_dir,
        data_dir: args.data_dir,
        token: args.token,
        token_file: args.token_file,
        lang_dir: args.lang_dir,
    })
    .await;
    ExitCode::SUCCESS
}
