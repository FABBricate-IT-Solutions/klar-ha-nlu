use crate::home::overlay::{apply_overlay, load_overlay};
use crate::home::registry::load_home;
use crate::home::sample::default_home;
use crate::lang::LanguageOverlay;
use crate::types::{CustomSentence, HomeGraph, Settings};
use std::path::Path;

pub struct LoadedHome {
    pub graph: HomeGraph,
    pub settings: Settings,
    pub custom: Vec<CustomSentence>,
    pub language: LanguageOverlay,
}

pub fn load_merged(config_dir: &Path, data_dir: &Path) -> LoadedHome {
    let mut graph = load_home(config_dir, default_home());
    let config_overlay = load_overlay(config_dir);
    apply_overlay(&mut graph, &config_overlay);
    let mut settings = config_overlay.settings.clone().unwrap_or_default();
    let mut custom = config_overlay.custom.clone();
    let mut language = config_overlay.language.clone();

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
    }

    LoadedHome { graph, settings, custom, language }
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
