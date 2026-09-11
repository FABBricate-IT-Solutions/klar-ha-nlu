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
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Full,
    ContextOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnitSystem {
    #[default]
    Metric,
    Imperial,
}

fn default_languages() -> Vec<String> {
    Vec::new()
}

const fn default_confirm_risky_actions() -> bool {
    true
}

const fn trait_mid() -> u8 {
    5
}

fn clamp_trait(value: u8) -> u8 {
    value.min(10)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RefineBand {
    Status,
    Command,
    Prompt,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceTraits {
    #[serde(default = "trait_mid")]
    pub warmth: u8,
    #[serde(default = "trait_mid")]
    pub humor: u8,
    #[serde(default)]
    pub sarcasm: u8,
    #[serde(default = "trait_mid")]
    pub formality: u8,
    #[serde(default = "trait_mid")]
    pub verbosity: u8,
    #[serde(default = "trait_mid")]
    pub energy: u8,
}

impl Default for VoiceTraits {
    fn default() -> Self {
        Self { warmth: 5, humor: 4, sarcasm: 2, formality: 5, verbosity: 4, energy: 5 }
    }
}

impl VoiceTraits {
    pub fn clamp(self) -> Self {
        Self {
            warmth: clamp_trait(self.warmth),
            humor: clamp_trait(self.humor),
            sarcasm: clamp_trait(self.sarcasm),
            formality: clamp_trait(self.formality),
            verbosity: clamp_trait(self.verbosity),
            energy: clamp_trait(self.energy),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SettingsJson")]
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
    /// Rewrite finished NLU speech with the engine LLM. Off by default.
    #[serde(default)]
    pub refine_speech: bool,
    /// Which spoken reply kinds the LLM may rewrite. Missing + refine on → status only.
    #[serde(default)]
    pub refine_bands: Vec<RefineBand>,
    /// Rewrite calendar list speech with the engine LLM. Off by default.
    #[serde(default)]
    pub calendar_llm: bool,
    /// Chime instead of TTS on simple on/off. Off by default.
    #[serde(default)]
    pub quiet_ack: bool,
    /// Allow Assist tools on the chit-chat LLM. Off by default.
    #[serde(default)]
    pub allow_llm_tools: bool,
    /// A Home Assistant leftover conversation agent is configured.
    #[serde(default)]
    pub fallback_llm: bool,
    /// Extra line on the packed personality prompt. Empty uses the pack voice only.
    #[serde(default)]
    pub extra_prompt: String,
    /// Operator temperatures for parse and speech. Default metric so old overlays stay Celsius.
    #[serde(default)]
    pub unit_system: UnitSystem,
    /// Packed-voice replacement when `personality` is `custom`. Empty falls back to default.
    #[serde(default)]
    pub custom_voice: String,
    /// Operator label for the custom voice.
    #[serde(default)]
    pub custom_voice_name: String,
    /// Character seed. Traits only refine how that seed speaks.
    #[serde(default)]
    pub custom_voice_seed: String,
    /// Delivery sliders 0–10. Independent of the seed identity.
    #[serde(default)]
    pub custom_voice_traits: VoiceTraits,
}

#[derive(Deserialize)]
struct SettingsJson {
    personality: Personality,
    mode: Mode,
    #[serde(default = "default_languages")]
    languages: Vec<String>,
    #[serde(default)]
    support_bundle: bool,
    #[serde(default)]
    support_bundle_raw_text: bool,
    #[serde(default = "default_confirm_risky_actions")]
    confirm_risky_actions: bool,
    #[serde(default)]
    semantic_adapters: bool,
    #[serde(default)]
    nlu_rag: bool,
    #[serde(default)]
    refine_speech: bool,
    #[serde(default)]
    refine_bands: Option<Vec<RefineBand>>,
    #[serde(default)]
    calendar_llm: bool,
    #[serde(default)]
    quiet_ack: bool,
    #[serde(default)]
    allow_llm_tools: bool,
    #[serde(default)]
    fallback_llm: bool,
    #[serde(default)]
    extra_prompt: String,
    #[serde(default)]
    unit_system: UnitSystem,
    #[serde(default)]
    custom_voice: String,
    #[serde(default)]
    custom_voice_name: String,
    #[serde(default)]
    custom_voice_seed: String,
    #[serde(default)]
    custom_voice_traits: VoiceTraits,
}

impl From<SettingsJson> for Settings {
    fn from(raw: SettingsJson) -> Self {
        let refine_bands = match raw.refine_bands {
            Some(bands) => bands,
            None if raw.refine_speech => vec![RefineBand::Status],
            None => Vec::new(),
        };
        Self {
            personality: raw.personality,
            mode: raw.mode,
            languages: raw.languages,
            support_bundle: raw.support_bundle,
            support_bundle_raw_text: raw.support_bundle_raw_text,
            confirm_risky_actions: raw.confirm_risky_actions,
            semantic_adapters: raw.semantic_adapters,
            nlu_rag: raw.nlu_rag,
            refine_speech: raw.refine_speech,
            refine_bands,
            calendar_llm: raw.calendar_llm,
            quiet_ack: raw.quiet_ack,
            allow_llm_tools: raw.allow_llm_tools,
            fallback_llm: raw.fallback_llm,
            extra_prompt: raw.extra_prompt,
            unit_system: raw.unit_system,
            custom_voice: raw.custom_voice,
            custom_voice_name: raw.custom_voice_name,
            custom_voice_seed: raw.custom_voice_seed,
            custom_voice_traits: raw.custom_voice_traits,
        }
    }
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
            refine_bands: Vec::new(),
            calendar_llm: false,
            quiet_ack: false,
            allow_llm_tools: false,
            fallback_llm: false,
            extra_prompt: String::new(),
            unit_system: UnitSystem::Metric,
            custom_voice: String::new(),
            custom_voice_name: String::new(),
            custom_voice_seed: String::new(),
            custom_voice_traits: VoiceTraits::default(),
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
        assert!(set.extra_prompt.is_empty());
        assert!(set.custom_voice.is_empty());
        assert!(set.custom_voice_name.is_empty());
        assert!(set.custom_voice_seed.is_empty());
        assert_eq!(set.custom_voice_traits, VoiceTraits::default());
        assert_eq!(set.unit_system, UnitSystem::Metric);
        assert_eq!(set.languages, vec!["de"]);
    }

    #[test]
    fn custom_personality_roundtrips() {
        let raw = r#"{"personality":"custom","mode":"full","custom_voice":"Voice: dry.","custom_voice_name":"Spock","custom_voice_seed":"You are Spock.","custom_voice_traits":{"sarcasm":3}}"#;
        let set: Settings = serde_json::from_str(raw).unwrap();
        assert_eq!(set.personality, Personality::Custom);
        assert_eq!(set.custom_voice, "Voice: dry.");
        assert_eq!(set.custom_voice_name, "Spock");
        assert_eq!(set.custom_voice_seed, "You are Spock.");
        assert_eq!(set.custom_voice_traits.sarcasm, 3);
        assert_eq!(set.custom_voice_traits.warmth, 5);
    }

    #[test]
    fn omitted_unit_system_stays_metric() {
        let raw = r#"{"personality":"default","mode":"full","languages":["de"]}"#;
        let set: Settings = serde_json::from_str(raw).unwrap();
        assert_eq!(set.unit_system, UnitSystem::Metric);
        assert_eq!(Settings::default().unit_system, UnitSystem::Metric);
    }

    #[test]
    fn omitted_languages_are_empty_not_de_en() {
        let raw = r#"{"personality":"default","mode":"full"}"#;
        let set: Settings = serde_json::from_str(raw).unwrap();
        assert!(set.languages.is_empty());
        assert!(Settings::default().languages.is_empty());
        assert!(set.refine_bands.is_empty());
    }

    #[test]
    fn missing_bands_with_refine_on_is_status_only() {
        let raw = r#"{"personality":"default","mode":"full","refine_speech":true}"#;
        let set: Settings = serde_json::from_str(raw).unwrap();
        assert!(set.refine_speech);
        assert_eq!(set.refine_bands, vec![RefineBand::Status]);
    }

    #[test]
    fn empty_bands_with_refine_on_stays_empty() {
        let raw = r#"{"personality":"default","mode":"full","refine_speech":true,"refine_bands":[]}"#;
        let set: Settings = serde_json::from_str(raw).unwrap();
        assert!(set.refine_speech);
        assert!(set.refine_bands.is_empty());
    }

    #[test]
    fn explicit_bands_roundtrip() {
        let raw = r#"{"personality":"default","mode":"full","refine_speech":true,"refine_bands":["status","command"]}"#;
        let set: Settings = serde_json::from_str(raw).unwrap();
        assert_eq!(set.refine_bands, vec![RefineBand::Status, RefineBand::Command]);
        let again: Settings = serde_json::from_str(&serde_json::to_string(&set).unwrap()).unwrap();
        assert_eq!(again.refine_bands, set.refine_bands);
    }
}
