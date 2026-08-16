use crate::home::{HomeStore, LoadedHome};
use crate::io::bundle::{entry_from_parse, BundleStore};
use crate::io::metrics::MetricsStore;
use crate::session::Sessions;
use crate::types::{CustomSentence, ParseResult, Settings};
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
}
