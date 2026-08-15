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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub personality: Personality,
    pub mode: Mode,
    /// BCP-47-ish codes of enabled packs (`de`, `en`, later `fr`, ...).
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { personality: Personality::Default, mode: Mode::Full, languages: default_languages() }
    }
}
