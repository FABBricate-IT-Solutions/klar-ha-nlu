use crate::home::overlay::{apply_overlay, load_overlay};
use crate::home::registry::load_home;
use crate::home::sample::default_home;
use crate::home::snapshot::{load_home_snapshot, HOME_SNAPSHOT_FILE};
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

/// Demo home only when explicitly requested (`KLAR_DEMO_HOME=1`). Otherwise an absent
/// registry yields an empty graph (D-16).
pub fn demo_home_enabled() -> bool {
    matches!(std::env::var("KLAR_DEMO_HOME").ok().as_deref().map(str::trim), Some("1" | "true" | "TRUE" | "yes" | "on"))
}

pub fn has_persisted_snapshot(data_dir: &Path) -> bool {
    data_dir.join(HOME_SNAPSHOT_FILE).is_file()
}

pub fn load_merged(config_dir: &Path, data_dir: &Path) -> LoadedHome {
    let fallback = if demo_home_enabled() { default_home() } else { HomeGraph::default() };
    let mut graph = load_home(config_dir, fallback);
    // D-17: last successful push wins over a cold registry read after restart.
    if let Some(saved) = load_home_snapshot(data_dir) {
        graph = saved;
    }
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

    crate::home::labels::rebuild_label_index(&mut graph);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::snapshot::{save_home_snapshot, HomeSnapshot, SnapshotArea, SnapshotEntity, HOME_SCHEMA_VERSION};
    use crate::home::test_dir::TestDir;

    #[test]
    fn without_registry_or_flag_home_is_empty() {
        let scratch = TestDir::new("load-empty");
        std::env::remove_var("KLAR_DEMO_HOME");
        let loaded = load_merged(scratch.path(), scratch.path());
        assert!(loaded.graph.entities.is_empty());
    }

    #[test]
    fn demo_flag_loads_sample_home() {
        let scratch = TestDir::new("load-demo");
        std::env::set_var("KLAR_DEMO_HOME", "1");
        let loaded = load_merged(scratch.path(), scratch.path());
        std::env::remove_var("KLAR_DEMO_HOME");
        assert!(!loaded.graph.entities.is_empty());
        assert!(loaded.graph.entities.iter().any(|entity| entity.entity_id == "light.wohnzimmer"));
    }

    #[test]
    fn persisted_snapshot_is_restored() {
        let scratch = TestDir::new("load-snap");
        std::env::remove_var("KLAR_DEMO_HOME");
        let snapshot = HomeSnapshot {
            schema_version: HOME_SCHEMA_VERSION.into(),
            entities: vec![SnapshotEntity {
                entity_id: "light.saved".into(),
                name: Some("Saved".into()),
                original_name: None,
                has_entity_name: false,
                area_id: Some("room".into()),
                device_id: None,
                platform: None,
                aliases: Vec::new(),
                labels: Vec::new(),
                disabled: false,
                device_class: None,
            }],
            devices: Vec::new(),
            areas: vec![SnapshotArea { id: "room".into(), name: "Room".into(), aliases: Vec::new(), floor_id: None }],
            floors: Vec::new(),
            labels: Vec::new(),
            assist: Some(vec!["light.saved".into()]),
            registered_intents: Vec::new(),
        };
        save_home_snapshot(scratch.path(), &snapshot).expect("save");
        let loaded = load_merged(scratch.path(), scratch.path());
        assert!(has_persisted_snapshot(scratch.path()));
        assert_eq!(loaded.graph.entities.len(), 1);
        assert_eq!(loaded.graph.entities[0].entity_id, "light.saved");
    }
}
