use crate::home::overlay::{apply_overlay, load_overlay};
use crate::home::registry::load_home;
use crate::home::sample::default_home;
use crate::lang::LanguageOverlay;
use crate::types::{CustomSentence, HomeGraph, MatchControl, PolicyRule, Settings, SpeechBank};
use std::path::Path;

pub struct LoadedHome {
    pub graph: HomeGraph,
    pub settings: Settings,
    pub custom: Vec<CustomSentence>,
    pub language: LanguageOverlay,
    pub policies: Vec<PolicyRule>,
    pub speech_bank: SpeechBank,
    pub match_controls: Vec<MatchControl>,
}

pub fn load_merged(config_dir: &Path, data_dir: &Path) -> LoadedHome {
    let mut graph = load_home(config_dir, default_home());
    let config_overlay = load_overlay(config_dir);
    apply_overlay(&mut graph, &config_overlay);
    let mut settings = config_overlay.settings.clone().unwrap_or_default();
    let mut custom = config_overlay.custom.clone();
    let mut language = config_overlay.language.clone();
    let mut policies = config_overlay.policies.clone();
    let mut speech_bank = config_overlay.speech_bank.clone();
    let mut match_controls = config_overlay.match_controls.clone();

    if data_dir != config_dir {
        let data_overlay = load_overlay(data_dir);
        apply_overlay(&mut graph, &data_overlay);
        if let Some(saved) = data_overlay.settings {
            settings = saved;
        }
        if !data_overlay.custom.is_empty() {
            custom = data_overlay.custom;
        }
        if !data_overlay.language.sets.is_empty() {
            language = data_overlay.language;
        }
        if !data_overlay.policies.is_empty() {
            policies = data_overlay.policies;
        }
        if !data_overlay.speech_bank.entries.is_empty() {
            speech_bank = data_overlay.speech_bank;
        }
        if !data_overlay.match_controls.is_empty() {
            match_controls = data_overlay.match_controls;
        }
    }

    LoadedHome { graph, settings, custom, language, policies, speech_bank, match_controls }
}

pub fn registry_stamp(config_dir: &Path) -> String {
    [
        "core.entity_registry",
        "core.area_registry",
        "core.device_registry",
        "core.floor_registry",
        "core.label_registry",
        "homeassistant.exposed_entities",
    ]
    .into_iter()
    .map(|name| {
        let meta = std::fs::metadata(config_dir.join(".storage").join(name));
        match meta.and_then(|m| m.modified()) {
            Ok(time) => format!("{time:?}"),
            Err(_) => String::new(),
        }
    })
    .collect::<Vec<_>>()
    .join("|")
}
