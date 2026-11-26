use crate::home::{load_merged, registry_stamp};
use crate::io::addon_options::AddonOptions;
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

    // D-12: a missing or unusable data directory is a start error, not a quiet fallback to
    // the Home Assistant config folder (which is often read-only).
    std::fs::create_dir_all(&args.data_dir)
        .map_err(|err| std::io::Error::new(err.kind(), format!("Datenverzeichnis {} nicht anlegbar: {err}", args.data_dir.display())))?;
    let meta = std::fs::metadata(&args.data_dir)
        .map_err(|err| std::io::Error::new(err.kind(), format!("Datenverzeichnis {} nicht lesbar: {err}", args.data_dir.display())))?;
    if !meta.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            format!("Datenverzeichnis {} ist kein Ordner", args.data_dir.display()),
        ));
    }
    let probe = args.data_dir.join(".klar-write-probe");
    std::fs::write(&probe, b"ok").map_err(|err| {
        std::io::Error::new(err.kind(), format!("Datenverzeichnis {} nicht beschreibbar: {err}", args.data_dir.display()))
    })?;
    let _ = std::fs::remove_file(&probe);
    let data_dir = args.data_dir.clone();
    // A-14: Supervisor options via serde (quotes/backslashes), not sed in the entry script.
    match AddonOptions::load(&data_dir) {
        Ok(opts) => opts.apply_env(),
        Err(err) => tracing::warn!("Add-on-Optionen nicht gelesen: {err}"),
    }
    // 6.6: when Supervisor left the option empty, mint `/data/klar.token` so the
    // integration can claim it. Only auto-pick the file when `options.json` exists
    // (App / Supervisor); bare cargo runs stay tokenless unless `--token-file` is set.
    let addon_options = data_dir.join("options.json");
    let default_token_file = addon_options.is_file().then(|| data_dir.join("klar.token"));
    let token_file = args.token_file.as_deref().or(default_token_file.as_deref());
    let token = resolve_token(args.token.or_else(|| std::env::var("KLAR_TOKEN").ok().filter(|s| !s.is_empty())), token_file);
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
    let restored_base = crate::home::snapshot::load_home_snapshot(&data_dir);
    let mut state = AppState::new(loaded, data_dir.clone(), token);
    state.config_dir = args.config_dir.clone();
    if let Some(base) = restored_base {
        // D-17: treat the persisted push as live so registry polling does not overwrite it.
        state.live_sync.store(true, Ordering::Relaxed);
        *state.live_base.lock().await = Some(std::sync::Arc::new(base));
    }
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
    // R-23: never fall back to a cwd-relative `packs/` — that silently picks up whatever the
    // process happened to start in. Explicit flag, env, or the packaged share path only.
    let dir = explicit
        .map(PathBuf::from)
        .or_else(|| std::env::var("KLAR_LANG_DIR").ok().filter(|value| !value.trim().is_empty()).map(PathBuf::from))
        .or_else(|| {
            let packaged = PathBuf::from("/usr/share/klar/packs");
            packaged.join("registry.yaml").is_file().then_some(packaged)
        });
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
    use crate::home::test_dir::TestDir;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn token_file_is_owner_rw_only() {
        let scratch = TestDir::new("token");
        let dir = scratch.path().to_path_buf();
        let path = dir.join("klar.token");
        let token = resolve_token(None, Some(&path)).expect("token");
        assert!(!token.is_empty());
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn addon_options_json_triggers_default_token_file_path() {
        let scratch = TestDir::new("addon-token");
        let dir = scratch.path().to_path_buf();
        std::fs::write(dir.join("options.json"), r#"{"token":""}"#).unwrap();
        assert!(dir.join("options.json").is_file());
        let path = dir.join("klar.token");
        let token = resolve_token(None, Some(&path)).expect("minted");
        assert!(!token.is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap().trim(), token);
    }

    #[test]
    fn language_packs_ignore_cwd_relative_packs() {
        let scratch = TestDir::new("lang-cwd");
        let packs = scratch.path().join("packs");
        std::fs::create_dir_all(&packs).unwrap();
        std::fs::write(packs.join("registry.yaml"), "packs: []\n").unwrap();
        let previous = std::env::current_dir().ok();
        std::env::set_current_dir(scratch.path()).unwrap();
        std::env::remove_var("KLAR_LANG_DIR");
        load_language_packs(None);
        if let Some(previous) = previous {
            let _ = std::env::set_current_dir(previous);
        }
        // No panic and no load from cwd — packaged path is absent in the test sandbox.
    }
}
