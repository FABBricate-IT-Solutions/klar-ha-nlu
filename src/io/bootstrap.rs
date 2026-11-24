use crate::home::{load_merged, registry_stamp};
use crate::io::http_server::{self, HttpLimits};
use crate::io::listen::bind;
use crate::io::llm::secure_endpoint_file;
use crate::io::state::AppState;
use crate::io::{web, wyoming};
use crate::types::Settings;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::signal::unix::{signal, Signal, SignalKind};
use tokio::sync::watch;
use uuid::Uuid;

pub struct RuntimeArgs {
    pub http: String,
    pub wyoming: String,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub token: Option<String>,
    pub token_file: Option<PathBuf>,
    pub lang_dir: Option<PathBuf>,
}

/// How long connections get to finish after a stop signal. Home Assistant gives an add-on
/// ten seconds before it kills it; this stays well inside that.
const SHUTDOWN_GRACE: Duration = Duration::from_millis(1200);

/// Run the engine until SIGTERM or SIGINT. `Err` means it could not start or one of the two
/// servers ended on its own; the caller exits non-zero then.
pub async fn run(args: RuntimeArgs) -> std::io::Result<()> {
    // Both ports before anything else: a taken port stops the start instead of leaving an
    // engine that answers on one of them only.
    let http_listener = bind("HTTP", &args.http).await?;
    let wyoming_listener = bind("Wyoming", &args.wyoming).await?;
    // Handlers before the slow part of the start. As PID 1 in a container the kernel only
    // delivers signals that have a handler.
    let mut signals = StopSignals::install()?;

    let _ = std::fs::create_dir_all(&args.data_dir);
    let data_dir = if args.data_dir.is_dir() { args.data_dir.clone() } else { args.config_dir.clone() };
    let token =
        resolve_token(args.token.or_else(|| std::env::var("KLAR_TOKEN").ok().filter(|s| !s.is_empty())), args.token_file.as_deref());
    if token.is_none() {
        tracing::warn!("Kein Token: Schreiben nur von localhost; Supervisor-Netz darf lesen");
    }

    load_language_packs(args.lang_dir.as_deref());
    let loaded = load_merged(&args.config_dir, &data_dir);
    if loaded.custom.is_empty() {
        tracing::warn!("keine eigenen Sätze in klar_nlu.json — Custom-Phrasen fehlen bis zum Speichern");
    }
    if !loaded.language.sets.is_empty() {
        crate::lang::install_user_overlay(Some(loaded.language.clone()));
    }
    let mut state = AppState::new(loaded, data_dir.clone(), token);
    state.config_dir = args.config_dir.clone();
    state.overlay.checkpoint();
    secure_endpoint_file(&state.data_dir);
    enable_bundle_from_env(&state).await;

    let reload_state = state.clone();
    let config_dir = args.config_dir.clone();
    tokio::spawn(async move { reload_home(reload_state, config_dir, data_dir).await });

    let (stop, stopping) = watch::channel(false);
    tracing::info!("UI auf http://{}", args.http);
    let mut http = tokio::spawn(http_server::serve(http_listener, web::router(state.clone()), HttpLimits::default(), stopping.clone()));
    tracing::info!("Wyoming lauscht auf {}", args.wyoming);
    let mut wyoming = tokio::spawn(wyoming::serve(wyoming_listener, state, stopping));

    tokio::select! {
        ended = &mut http => Err(server_ended("HTTP", ended)),
        ended = &mut wyoming => Err(server_ended("Wyoming", ended)),
        signal = signals.next() => {
            tracing::info!("{signal} empfangen, fahre herunter");
            let _ = stop.send(true);
            let drained = tokio::time::timeout(SHUTDOWN_GRACE, async { tokio::join!(http, wyoming) }).await;
            match drained {
                Ok(_) => tracing::info!("Beide Server beendet"),
                Err(_) => tracing::warn!("Nach {SHUTDOWN_GRACE:?} sind noch Verbindungen offen, beende trotzdem"),
            }
            Ok(())
        }
    }
}

/// A server task that came back without being asked to stop. Whatever the reason, the
/// engine is no longer whole.
fn server_ended(server: &str, ended: Result<std::io::Result<()>, tokio::task::JoinError>) -> std::io::Error {
    let reason = match ended {
        Ok(Ok(())) => "ohne Fehler".to_string(),
        Ok(Err(err)) => err.to_string(),
        Err(err) => err.to_string(),
    };
    std::io::Error::other(format!("{server}-Server hat sich beendet: {reason}"))
}

struct StopSignals {
    terminate: Signal,
    interrupt: Signal,
}

impl StopSignals {
    fn install() -> std::io::Result<Self> {
        Ok(Self { terminate: signal(SignalKind::terminate())?, interrupt: signal(SignalKind::interrupt())? })
    }

    /// The name of the next stop signal.
    async fn next(&mut self) -> &'static str {
        tokio::select! {
            _ = self.terminate.recv() => "SIGTERM",
            _ = self.interrupt.recv() => "SIGINT",
        }
    }
}

fn env_on(name: &str) -> bool {
    matches!(std::env::var(name).ok().as_deref().map(str::trim), Some("1" | "true" | "TRUE" | "yes" | "on"))
}

async fn enable_bundle_from_env(state: &AppState) {
    if !env_on("KLAR_SUPPORT_BUNDLE") {
        return;
    }
    let mut settings = state.settings.lock().await;
    let wanted = settings.clone();
    // The saved choice wins: the switch only fills in settings nobody has saved yet.
    let saved = state.overlay.try_update(|overlay| {
        if overlay.settings.is_some() || wanted.support_bundle {
            return Err(());
        }
        overlay.settings = Some(Settings { support_bundle: true, ..wanted });
        Ok(())
    });
    match saved {
        Ok(Ok(())) => {
            settings.support_bundle = true;
            tracing::info!("Support-Bundle an (KLAR_SUPPORT_BUNDLE)");
        }
        Ok(Err(())) => {}
        Err(err) => tracing::error!("Support-Bundle (KLAR_SUPPORT_BUNDLE) nicht eingeschaltet, klar_nlu.json nicht gespeichert: {err}"),
    }
}

fn resolve_token(explicit: Option<String>, file: Option<&Path>) -> Option<String> {
    if let Some(token) = explicit.filter(|s| !s.is_empty()) {
        return Some(token);
    }
    let path = file?;
    if let Ok(existing) = std::fs::read_to_string(path) {
        let trimmed = existing.trim().to_string();
        if !trimmed.is_empty() {
            tighten_token_mode(path);
            return Some(trimmed);
        }
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if write_token_file(path, &token).is_ok() {
        tracing::warn!("Write-Token nach {} geschrieben", path.display());
        Some(token)
    } else {
        None
    }
}

fn tighten_token_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        if perms.mode() & 0o777 != 0o600 {
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
}

fn write_token_file(path: &Path, token: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let mut file = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path)?;
    file.write_all(token.as_bytes())?;
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

async fn reload_home(state: AppState, config_dir: PathBuf, data_dir: PathBuf) {
    let mut stamp = registry_stamp(&config_dir);
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let next = registry_stamp(&config_dir);
        if next == stamp || state.live_sync.load(Ordering::Relaxed) {
            continue;
        }
        stamp = next;
        let loaded = load_merged(&config_dir, &data_dir);
        let n = loaded.graph.entities.len();
        state.home.replace(loaded.graph).await;
        *state.custom.lock().await = loaded.custom;
        *state.settings.lock().await = loaded.settings;
        crate::lang::install_user_overlay(if loaded.language.sets.is_empty() { None } else { Some(loaded.language) });
        tracing::info!("Home-Graph neu geladen ({n} Entitäten)");
    }
}

fn load_language_packs(explicit: Option<&Path>) {
    let dir = explicit
        .map(PathBuf::from)
        .or_else(|| std::env::var("KLAR_LANG_DIR").ok().map(PathBuf::from))
        .or_else(|| ["/usr/share/klar/packs", "packs"].into_iter().map(PathBuf::from).find(|path| path.join("registry.yaml").is_file()));
    let Some(dir) = dir else {
        return;
    };
    match crate::lang::load_runtime_dir(&dir) {
        Ok(count) => tracing::info!("Sprachpakete aus {} geladen ({count})", dir.display()),
        Err(err) => tracing::warn!("Sprachpakete {}: {err}", dir.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn token_file_is_owner_rw_only() {
        let dir = std::env::temp_dir().join(format!("klar-token-{}", uuid::Uuid::new_v4().simple()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("klar.token");
        let token = resolve_token(None, Some(&path)).expect("token");
        assert!(!token.is_empty());
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
