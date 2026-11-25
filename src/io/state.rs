use crate::home::overlay::apply_overlay;
use crate::home::{HomeStore, LoadedHome, OverlayStore};
use crate::io::bundle::{entry_from_parse, BundleStore};
use crate::io::conversations::{turn_from_outcome, ConversationJournal};
use crate::io::llm::load_endpoint;
use crate::io::llm_calls::LlmCallStore;
use crate::io::metrics::MetricsStore;
use crate::io::parse_run::ParseLimiter;
use crate::io::trainer_consent::TrainerConsentHub;
use crate::llm::LlmEndpoint;
use crate::session::Sessions;
use crate::types::{CustomSentence, HomeGraph, MatchControl, ParseOutcome, ParseResult, PolicyRule, Settings, SpeechBank};
use axum::http::StatusCode;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub home: HomeStore,
    pub sessions: Arc<Mutex<Sessions>>,
    pub settings: Arc<Mutex<Settings>>,
    pub custom: Arc<Mutex<Vec<CustomSentence>>>,
    pub policies: Arc<Mutex<Vec<PolicyRule>>>,
    pub speech_bank: Arc<Mutex<SpeechBank>>,
    pub match_controls: Arc<Mutex<Vec<MatchControl>>>,
    pub journal: ConversationJournal,
    pub bundle: BundleStore,
    pub metrics: Arc<MetricsStore>,
    pub llm_calls: Arc<LlmCallStore>,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    /// The only writer of `klar_nlu.json` in `data_dir`.
    pub overlay: OverlayStore,
    pub live_sync: Arc<AtomicBool>,
    /// The home as Home Assistant last pushed it, before the overlays. `None` until the
    /// first push; the registry files are the base until then.
    pub live_base: Arc<Mutex<Option<Arc<HomeGraph>>>>,
    pub token: Option<String>,
    pub llm: Arc<Mutex<Option<LlmEndpoint>>>,
    pub trainer_consent: Arc<TrainerConsentHub>,
    /// CPU slots + deadline for parse (package 3.4).
    pub parse_limiter: ParseLimiter,
}

impl AppState {
    pub fn new(loaded: LoadedHome, data_dir: PathBuf, token: Option<String>) -> Self {
        let llm = load_endpoint(&data_dir);
        Self {
            home: HomeStore::new(loaded.graph),
            sessions: Arc::new(Mutex::new(Sessions::default())),
            settings: Arc::new(Mutex::new(loaded.settings)),
            custom: Arc::new(Mutex::new(loaded.custom)),
            policies: Arc::new(Mutex::new(loaded.policies)),
            speech_bank: Arc::new(Mutex::new(loaded.speech_bank)),
            match_controls: Arc::new(Mutex::new(loaded.match_controls)),
            journal: ConversationJournal::open(&data_dir),
            bundle: BundleStore::open(&data_dir),
            metrics: Arc::new(MetricsStore::default()),
            llm_calls: Arc::new(LlmCallStore::default()),
            config_dir: data_dir.clone(),
            overlay: OverlayStore::open(&data_dir),
            data_dir,
            live_sync: Arc::new(AtomicBool::new(false)),
            live_base: Arc::new(Mutex::new(None)),
            token,
            llm: Arc::new(Mutex::new(llm)),
            trainer_consent: TrainerConsentHub::new(),
            parse_limiter: ParseLimiter::new(),
        }
    }

    pub async fn record_parse(&self, source: &str, language: Option<&str>, result: &ParseResult) {
        let include_text = self.settings.lock().await.support_bundle_raw_text;
        self.metrics.record(source, language, result, include_text);
        if !self.settings.lock().await.support_bundle {
            return;
        }
        self.bundle.append(entry_from_parse(source, language, result, include_text));
    }

    pub async fn record_outcome(&self, outcome: &ParseOutcome, last_names: Vec<String>) {
        let include_text = self.settings.lock().await.support_bundle_raw_text;
        let area = outcome.evidence.iter().find(|item| item.kind == "preferred_area").map(|item| item.value.clone());
        self.journal.append(turn_from_outcome(outcome, include_text, last_names, area));
    }

    pub async fn catalog_for_settings(&self) -> std::sync::Arc<crate::lang::Catalog> {
        crate::lang::bound_catalog(&*self.settings.lock().await)
    }

    pub async fn apply_teach(&self, entity_id: &str, alias: &str) {
        if entity_id.split('.').count() != 2
            || alias.chars().count() < 2
            || alias.chars().count() > 40
            || alias.chars().any(char::is_control)
        {
            return;
        }
        let home = self.home.snapshot().await;
        if !home.entities.iter().any(|entity| entity.entity_id == entity_id) {
            return;
        }
        let saved = self.overlay.update(|overlay| {
            let aliases = overlay.aliases.entry(entity_id.to_string()).or_default();
            if !aliases.iter().any(|existing| existing == alias) {
                aliases.push(alias.to_string());
            }
            overlay.clone()
        });
        let Ok(overlay) = saved.map_err(save_failed) else {
            return;
        };
        self.home
            .edit(|next| {
                apply_overlay(next, &overlay);
                None::<()>
            })
            .await;
    }

    pub async fn apply_areas(&self, rows: &[(String, String)]) {
        if rows.is_empty() {
            return;
        }
        let saved = self.overlay.update(|overlay| {
            for (entity_id, area) in rows {
                overlay.areas.insert(entity_id.clone(), area.clone());
            }
            overlay.clone()
        });
        let Ok(overlay) = saved.map_err(save_failed) else {
            return;
        };
        self.home
            .edit(|next| {
                apply_overlay(next, &overlay);
                None::<()>
            })
            .await;
    }
}

/// A change that did not reach `klar_nlu.json`: log why and answer 500. The caller must not
/// have changed the running state yet.
pub(crate) fn save_failed(err: std::io::Error) -> StatusCode {
    tracing::error!("klar_nlu.json nicht gespeichert: {err}");
    StatusCode::INTERNAL_SERVER_ERROR
}
