use super::paths::{confined_file, read_to_string_confined, write_atomic_confined};
use crate::lang::{LanguageOverlay, LanguageRevision};
use crate::types::{CustomSentence, HomeGraph, MatchControl, PolicyRule, Settings, SpeechBank};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiApplyRow {
    pub entity_id: String,
    pub before: Option<String>,
    pub after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    #[serde(default = "default_tab")]
    pub tab: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    /// True after the operator saves a UI language. Until then `KLAR_UI_LOCALE` may seed chrome.
    #[serde(default)]
    pub locale_set: bool,
    #[serde(default)]
    pub dismissed: Vec<String>,
    #[serde(default)]
    pub last_apply: Vec<UiApplyRow>,
    #[serde(default)]
    pub graph: HashMap<String, UiPoint>,
    #[serde(default)]
    pub wizard_done: bool,
    #[serde(default = "default_house_view")]
    pub house_view: String,
    #[serde(default = "default_rules_view")]
    pub rules_view: String,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            tab: default_tab(),
            locale: default_locale(),
            locale_set: false,
            dismissed: Vec::new(),
            last_apply: Vec::new(),
            graph: HashMap::new(),
            wizard_done: false,
            house_view: default_house_view(),
            rules_view: default_rules_view(),
            theme: default_theme(),
        }
    }
}

fn default_tab() -> String {
    "home".into()
}

fn default_locale() -> String {
    String::new()
}

fn default_house_view() -> String {
    "calibrate".into()
}

fn default_rules_view() -> String {
    "routines".into()
}

fn default_theme() -> String {
    "dark".into()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Overlay {
    #[serde(default)]
    pub aliases: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub preferred: Vec<String>,
    #[serde(default)]
    pub nlu_ignore: Vec<String>,
    #[serde(default)]
    pub areas: HashMap<String, String>,
    #[serde(default)]
    pub settings: Option<Settings>,
    #[serde(default)]
    pub custom: Vec<CustomSentence>,
    #[serde(default)]
    pub infra_id: Vec<String>,
    #[serde(default)]
    pub infra_name: Vec<String>,
    #[serde(default)]
    pub timer_hints: HashMap<i32, String>,
    #[serde(default)]
    pub preferred_climate: Option<String>,
    #[serde(default)]
    pub ui: UiState,
    #[serde(default)]
    pub language: LanguageOverlay,
    #[serde(default)]
    pub language_history: Vec<LanguageRevision>,
    #[serde(default)]
    pub policies: Vec<PolicyRule>,
    #[serde(default)]
    pub speech_bank: SpeechBank,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub match_controls: Vec<MatchControl>,
}

const OVERLAY_FILE: &str = "klar_nlu.json";

pub fn overlay_path(dir: &Path) -> std::path::PathBuf {
    confined_file(dir, OVERLAY_FILE).unwrap_or_else(|_| PathBuf::from(OVERLAY_FILE))
}

pub fn load_overlay(dir: &Path) -> Overlay {
    let raw = read_to_string_confined(dir, OVERLAY_FILE).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_overlay(dir: &Path, overlay: &Overlay) -> std::io::Result<()> {
    write_atomic_confined(dir, OVERLAY_FILE, &serde_json::to_vec_pretty(overlay).unwrap_or_default())
}

pub fn apply_overlay(home: &mut HomeGraph, overlay: &Overlay) {
    for ent in &mut home.entities {
        if let Some(extra) = overlay.aliases.get(&ent.entity_id) {
            for alias in extra {
                if !ent.aliases.iter().any(|a| a == alias) {
                    ent.aliases.push(alias.clone());
                }
            }
        }
        if overlay.preferred.iter().any(|id| id == &ent.entity_id) && !ent.tags.iter().any(|t| t == "preferred") {
            ent.tags.push("preferred".into());
        }
        if overlay.nlu_ignore.iter().any(|id| id == &ent.entity_id) && !ent.tags.iter().any(|t| t == "nlu_ignore") {
            ent.tags.push("nlu_ignore".into());
        }
        if let Some(area) = overlay.areas.get(&ent.entity_id) {
            ent.area = if area.is_empty() { None } else { Some(area.clone()) };
        }
        if !ent.tags.iter().any(|t| t == "infra") && infra_match(ent, overlay) {
            ent.tags.push("infra".into());
        }
    }
    merge_policy(home, overlay);
}

fn infra_match(ent: &crate::types::EntityRec, overlay: &Overlay) -> bool {
    let id = ent.entity_id.to_ascii_lowercase();
    let name = crate::parse::normalize::compact(&ent.name);
    overlay.infra_id.iter().any(|needle| id.contains(&needle.to_ascii_lowercase()))
        || overlay.infra_name.iter().any(|needle| name.contains(&needle.to_ascii_lowercase()))
}

fn merge_policy(home: &mut HomeGraph, overlay: &Overlay) {
    for needle in &overlay.infra_id {
        if !home.policy.infra_id.iter().any(|n| n == needle) {
            home.policy.infra_id.push(needle.clone());
        }
    }
    for needle in &overlay.infra_name {
        if !home.policy.infra_name.iter().any(|n| n == needle) {
            home.policy.infra_name.push(needle.clone());
        }
    }
    home.policy.timer_hints.extend(overlay.timer_hints.clone());
    if overlay.preferred_climate.is_some() {
        home.policy.preferred_climate = overlay.preferred_climate.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EntityRec;

    #[test]
    fn overlay_sets_and_clears_area() {
        let mut home = HomeGraph {
            entities: vec![EntityRec {
                entity_id: "light.orphan".into(),
                name: "Hue play 2".into(),
                domain: "light".into(),
                platform: None,
                area: None,
                aliases: Vec::new(),
                tags: Vec::new(),
            }],
            ..Default::default()
        };
        apply_overlay(&mut home, &Overlay { areas: [("light.orphan".into(), "wohnzimmer".into())].into(), ..Default::default() });
        assert_eq!(home.entities[0].area.as_deref(), Some("wohnzimmer"));
        apply_overlay(&mut home, &Overlay { areas: [("light.orphan".into(), String::new())].into(), ..Default::default() });
        assert_eq!(home.entities[0].area, None);
    }

    #[test]
    fn settings_survive_roundtrip() {
        let dir = std::env::temp_dir().join(format!("klar-overlay-set-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let overlay = Overlay { settings: Some(Settings { support_bundle: true, ..Settings::default() }), ..Default::default() };
        save_overlay(&dir, &overlay).unwrap();
        let loaded = load_overlay(&dir);
        assert!(loaded.settings.unwrap().support_bundle);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ui_survives_roundtrip() {
        let dir = std::env::temp_dir().join(format!("klar-overlay-ui-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let overlay = Overlay {
            ui: UiState {
                tab: "graph".into(),
                locale: "en".into(),
                locale_set: true,
                dismissed: vec!["light.hue_play_1".into()],
                last_apply: vec![UiApplyRow {
                    entity_id: "light.hue_play_1".into(),
                    before: Some("wohnzimmer".into()),
                    after: "schlafzimmer".into(),
                }],
                graph: [("light.schlafzimmer".into(), UiPoint { x: 120.0, y: 40.0 })].into(),
                wizard_done: true,
                house_view: "entities".into(),
                rules_view: "policies".into(),
                theme: "light".into(),
            },
            ..Default::default()
        };
        save_overlay(&dir, &overlay).unwrap();
        let loaded = load_overlay(&dir);
        assert_eq!(loaded.ui.tab, "graph");
        assert_eq!(loaded.ui.locale, "en");
        assert!(loaded.ui.locale_set);
        assert_eq!(loaded.ui.dismissed, vec!["light.hue_play_1"]);
        assert_eq!(loaded.ui.last_apply[0].before.as_deref(), Some("wohnzimmer"));
        assert_eq!(loaded.ui.graph["light.schlafzimmer"].x, 120.0);
        assert!(loaded.ui.wizard_done);
        assert_eq!(loaded.ui.house_view, "entities");
        assert_eq!(loaded.ui.rules_view, "policies");
        assert_eq!(loaded.ui.theme, "light");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ui_overlay_defaults_missing_fields() {
        let ui: UiState = serde_json::from_str(r#"{"tab":"home"}"#).unwrap();
        assert!(!ui.wizard_done);
        assert_eq!(ui.house_view, "calibrate");
        assert_eq!(ui.rules_view, "routines");
        assert_eq!(ui.theme, "dark");
        assert_eq!(ui.locale, "");
        assert!(!ui.locale_set);
    }

    #[test]
    fn custom_sentences_survive_roundtrip() {
        let dir = std::env::temp_dir().join(format!("klar-overlay-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let overlay = Overlay {
            custom: vec![CustomSentence { phrase: "filmabend".into(), intent: "HassTurnOn".into(), slots: HashMap::new() }],
            ..Default::default()
        };
        save_overlay(&dir, &overlay).unwrap();
        let loaded = load_overlay(&dir);
        assert_eq!(loaded.custom[0].phrase, "filmabend");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn overlay_marks_apartment_infra() {
        let mut home = HomeGraph {
            entities: vec![EntityRec {
                entity_id: "light.u7_pro_led".into(),
                name: "U7 Pro LED".into(),
                domain: "light".into(),
                platform: None,
                area: Some("flur".into()),
                aliases: Vec::new(),
                tags: Vec::new(),
            }],
            ..Default::default()
        };
        apply_overlay(
            &mut home,
            &Overlay { infra_id: vec!["u7_pro".into()], timer_hints: [(90, "laundry".into())].into(), ..Default::default() },
        );
        assert!(home.entities[0].tags.iter().any(|t| t == "infra"));
        assert_eq!(home.policy.timer_hints.get(&90).map(String::as_str), Some("laundry"));
        assert!(crate::home::policy::is_infra(&home.entities[0]));
    }

    #[test]
    fn overlay_marks_nlu_ignore() {
        let mut home = HomeGraph {
            entities: vec![EntityRec {
                entity_id: "switch.create_calendar_event".into(),
                name: "Create Calendar Event".into(),
                domain: "switch".into(),
                platform: None,
                area: None,
                aliases: Vec::new(),
                tags: Vec::new(),
            }],
            ..Default::default()
        };
        apply_overlay(&mut home, &Overlay { nlu_ignore: vec!["switch.create_calendar_event".into()], ..Default::default() });
        assert!(home.entities[0].tags.iter().any(|tag| tag == "nlu_ignore"));
        assert!(crate::home::policy::is_nlu_ignored(&home.entities[0]));
    }

    #[test]
    fn match_controls_survive_roundtrip() {
        let dir = std::env::temp_dir().join(format!("klar-overlay-match-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let overlay = Overlay {
            match_controls: vec![MatchControl { id: "media".into(), enabled: false, precedence: Some(3) }],
            ..Default::default()
        };
        save_overlay(&dir, &overlay).unwrap();
        let loaded = load_overlay(&dir);
        assert_eq!(loaded.match_controls.len(), 1);
        assert_eq!(loaded.match_controls[0].id, "media");
        assert!(!loaded.match_controls[0].enabled);
        assert_eq!(loaded.match_controls[0].precedence, Some(3));
        let empty: Overlay = serde_json::from_str("{}").unwrap();
        assert!(empty.match_controls.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
