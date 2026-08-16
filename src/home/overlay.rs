use crate::lang::{LanguageOverlay, LanguageRevision};
use crate::types::{CustomSentence, HomeGraph, Settings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
    #[serde(default)]
    pub dismissed: Vec<String>,
    #[serde(default)]
    pub last_apply: Vec<UiApplyRow>,
    #[serde(default)]
    pub graph: HashMap<String, UiPoint>,
}

impl Default for UiState {
    fn default() -> Self {
        Self { tab: default_tab(), locale: default_locale(), dismissed: Vec::new(), last_apply: Vec::new(), graph: HashMap::new() }
    }
}

fn default_tab() -> String {
    "dashboard".into()
}

fn default_locale() -> String {
    "de".into()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Overlay {
    #[serde(default)]
    pub aliases: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub preferred: Vec<String>,
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
}

pub fn overlay_path(dir: &Path) -> std::path::PathBuf {
    dir.join("klar_nlu.json")
}

pub fn load_overlay(dir: &Path) -> Overlay {
    let raw = std::fs::read_to_string(overlay_path(dir)).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_overlay(dir: &Path, overlay: &Overlay) -> std::io::Result<()> {
    let path = overlay_path(dir);
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(overlay).unwrap_or_default())?;
    std::fs::rename(tmp, path)
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
                dismissed: vec!["light.hue_play_1".into()],
                last_apply: vec![UiApplyRow {
                    entity_id: "light.hue_play_1".into(),
                    before: Some("wohnzimmer".into()),
                    after: "schlafzimmer".into(),
                }],
                graph: [("light.schlafzimmer".into(), UiPoint { x: 120.0, y: 40.0 })].into(),
            },
            ..Default::default()
        };
        save_overlay(&dir, &overlay).unwrap();
        let loaded = load_overlay(&dir);
        assert_eq!(loaded.ui.tab, "graph");
        assert_eq!(loaded.ui.locale, "en");
        assert_eq!(loaded.ui.dismissed, vec!["light.hue_play_1"]);
        assert_eq!(loaded.ui.last_apply[0].before.as_deref(), Some("wohnzimmer"));
        assert_eq!(loaded.ui.graph["light.schlafzimmer"].x, 120.0);
        let _ = std::fs::remove_dir_all(&dir);
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
}
