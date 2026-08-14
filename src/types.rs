use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Slot {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Intent {
    pub name: String,
    pub slots: Vec<Slot>,
}

impl Intent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            slots: Vec::new(),
        }
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.slots.push(Slot {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    pub fn slot(&self, name: &str) -> Option<&str> {
        self.slots
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub text: String,
    pub intents: Vec<Intent>,
    pub speech: String,
    pub clarify: bool,
    pub conversation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRec {
    pub entity_id: String,
    pub name: String,
    pub domain: String,
    pub area: Option<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaRec {
    pub area_id: String,
    pub name: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HomeGraph {
    pub entities: Vec<EntityRec>,
    pub areas: Vec<AreaRec>,
    #[serde(default)]
    pub scene_members: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSentence {
    pub phrase: String,
    pub intent: String,
    pub slots: HashMap<String, String>,
}

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
    /// BCP-47-ish codes of enabled packs (`de`, `en`, later `fr`, …).
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            personality: Personality::Default,
            mode: Mode::Full,
            languages: default_languages(),
        }
    }
}
