use crate::home::overlay::{apply_overlay, load_overlay, save_overlay};
use crate::home::{HomeStore, LoadedHome};
use crate::io::bundle::{entry_from_parse, BundleStore};
use crate::io::conversations::{turn_from_outcome, ConversationJournal};
use crate::io::metrics::MetricsStore;
use crate::io::llm::load_endpoint;
use crate::llm::LlmEndpoint;
use crate::session::Sessions;
use crate::types::{CustomSentence, MatchControl, ParseOutcome, ParseResult, PolicyRule, Settings, SpeechBank};
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
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub live_sync: Arc<AtomicBool>,
    pub token: Option<String>,
    pub llm: Arc<Mutex<Option<LlmEndpoint>>>,
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
            config_dir: data_dir.clone(),
            data_dir,
            live_sync: Arc::new(AtomicBool::new(false)),
            token,
            llm: Arc::new(Mutex::new(llm)),
        }
    }

    pub async fn record_parse(&self, source: &str, language: Option<&str>, result: &ParseResult) {
        self.metrics.record(source, language, result);
        if !self.settings.lock().await.support_bundle {
            return;
        }
        self.bundle.append(entry_from_parse(source, language, result));
    }

    pub async fn record_outcome(&self, outcome: &ParseOutcome, last_names: Vec<String>) {
        let include_text = self.settings.lock().await.support_bundle_raw_text;
        let area = outcome.evidence.iter().find(|item| item.kind == "preferred_area").map(|item| item.value.clone());
        self.journal.append(turn_from_outcome(outcome, include_text, last_names, area));
    }

    pub async fn catalog_for_settings(&self) -> &'static crate::lang::Catalog {
        crate::lang::catalog_for(&self.settings.lock().await.languages)
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
        let mut overlay = load_overlay(&self.data_dir);
        let aliases = overlay.aliases.entry(entity_id.to_string()).or_default();
        if !aliases.iter().any(|existing| existing == alias) {
            aliases.push(alias.to_string());
        }
        let _ = save_overlay(&self.data_dir, &overlay);
        self.home
            .edit(|next| {
                apply_overlay(next, &overlay);
                None::<()>
            })
            .await;
    }
}
