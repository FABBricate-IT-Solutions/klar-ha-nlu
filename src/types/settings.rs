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
    Jarvis,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Full,
    ContextOnly,
}

fn default_languages() -> Vec<String> {
    Vec::new()
}

const fn default_confirm_risky_actions() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub personality: Personality,
    pub mode: Mode,
    /// Enabled pack codes. Empty means every compiled locale is enabled;
    /// the catalog still binds per request (do not merge all lexicons).
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
    /// Opt-in NLU-as-RAG: matched-slice retrieval and Klar tools. Off by default.
    #[serde(default)]
    pub nlu_rag: bool,
    /// HA: rewrite finished NLU speech with the fallback LLM.
    #[serde(default)]
    pub refine_speech: bool,
    /// HA: rewrite calendar list speech with the fallback LLM.
    #[serde(default)]
    pub calendar_llm: bool,
    /// HA: chime instead of TTS on simple on/off.
    #[serde(default)]
    pub quiet_ack: bool,
    /// HA: allow Assist tools on the chit-chat LLM.
    #[serde(default)]
    pub allow_llm_tools: bool,
    /// HA: a fallback conversation agent is configured.
    #[serde(default)]
    pub fallback_llm: bool,
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
            nlu_rag: false,
            refine_speech: false,
            calendar_llm: false,
            quiet_ack: false,
            allow_llm_tools: false,
            fallback_llm: false,
        }
    }
}

impl Settings {
    pub fn pinned(code: impl Into<String>) -> Self {
        Self { languages: vec![code.into()], ..Self::default() }
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
        assert!(!set.nlu_rag);
        assert!(!set.refine_speech);
        assert!(!set.calendar_llm);
        assert!(!set.quiet_ack);
        assert!(!set.allow_llm_tools);
        assert!(!set.fallback_llm);
        assert_eq!(set.languages, vec!["de"]);
    }

    #[test]
    fn omitted_languages_are_empty_not_de_en() {
        let raw = r#"{"personality":"default","mode":"full"}"#;
        let set: Settings = serde_json::from_str(raw).unwrap();
        assert!(set.languages.is_empty());
        assert!(Settings::default().languages.is_empty());
    }
}
