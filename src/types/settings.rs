use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Personality {
    #[default]
    Default,
    Butler,
    Locker,
    Fuersorglich,
    Party,
    Grantig,
    Sarkastisch,
    Pirat,
    Hippie,
    Gollum,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Full,
    ContextOnly,
}

fn default_languages() -> Vec<String> {
    vec!["de".into(), "en".into()]
}

const fn default_confirm_risky_actions() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub personality: Personality,
    pub mode: Mode,
    /// BCP-47-ish codes of enabled packs (`de`, `en`, later `fr`, ...).
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
    /// Persist Assist/API traffic under the data dir for a downloadable dataset.
    #[serde(default)]
    pub support_bundle: bool,
    /// Include raw utterance and speech in downloaded bundles. Off by default.
    #[serde(default)]
    pub support_bundle_raw_text: bool,
    /// Require an affirmative follow-up before safety-relevant controls execute.
    #[serde(default = "default_confirm_risky_actions")]
    pub confirm_risky_actions: bool,
    /// Consult local semantic adapters after a ranking reject. Off by default.
    #[serde(default)]
    pub semantic_adapters: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            personality: Personality::Default,
            mode: Mode::Full,
            languages: default_languages(),
            support_bundle: false,
            support_bundle_raw_text: false,
            confirm_risky_actions: true,
            semantic_adapters: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_support_bundle_defaults_off() {
        let raw = r#"{"personality":"default","mode":"full","languages":["de"]}"#;
        let set: Settings = serde_json::from_str(raw).unwrap();
        assert!(!set.support_bundle);
        assert!(!set.support_bundle_raw_text);
        assert!(set.confirm_risky_actions);
        assert!(!set.semantic_adapters);
        assert_eq!(set.languages, vec!["de"]);
    }
}
