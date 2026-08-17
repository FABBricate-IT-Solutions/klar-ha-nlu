use crate::home::{HomeStore, LoadedHome};
use crate::io::bundle::{entry_from_parse, BundleStore};
use crate::io::conversations::{turn_from_outcome, ConversationJournal};
use crate::io::metrics::MetricsStore;
use crate::session::Sessions;
use crate::types::{CustomSentence, ParseOutcome, ParseResult, PolicyRule, Settings, SpeechBank};
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
    pub journal: ConversationJournal,
    pub bundle: BundleStore,
    pub metrics: Arc<MetricsStore>,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub live_sync: Arc<AtomicBool>,
    pub token: Option<String>,
}

impl AppState {
    pub fn new(loaded: LoadedHome, data_dir: PathBuf, token: Option<String>) -> Self {
        Self {
            home: HomeStore::new(loaded.graph),
            sessions: Arc::new(Mutex::new(Sessions::default())),
            settings: Arc::new(Mutex::new(loaded.settings)),
            custom: Arc::new(Mutex::new(loaded.custom)),
            policies: Arc::new(Mutex::new(loaded.policies)),
            speech_bank: Arc::new(Mutex::new(loaded.speech_bank)),
            journal: ConversationJournal::open(&data_dir),
            bundle: BundleStore::open(&data_dir),
            metrics: Arc::new(MetricsStore::default()),
            config_dir: data_dir.clone(),
            data_dir,
            live_sync: Arc::new(AtomicBool::new(false)),
            token,
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
        self.journal.append(turn_from_outcome(outcome, include_text, last_names));
    }
}
