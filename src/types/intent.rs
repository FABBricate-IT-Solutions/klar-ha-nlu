use serde::{Deserialize, Serialize};

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

pub const KNOWN_INTENTS: &[&str] = &[
    "HassTurnOn",
    "HassTurnOff",
    "HassToggle",
    "HassLightSet",
    "HassClimateSetTemperature",
    "HassClimateGetTemperature",
    "HassGetState",
    "HassMediaPause",
    "HassMediaNext",
    "HassMediaPlayerMute",
    "HassFanSetSpeed",
    "HassVacuumStart",
    "HassVacuumReturnToBase",
    "HassSetPosition",
    "HassStartTimer",
    "HassIncreaseTimer",
    "HassDecreaseTimer",
    "HassCancelTimer",
    "HassPauseTimer",
    "HassListAddItem",
    "HassListCompleteItem",
    "HassShoppingListAddItem",
    "HassShoppingListCompleteItem",
];

pub fn known_intent(name: &str) -> bool {
    KNOWN_INTENTS.contains(&name)
}

impl Intent {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), slots: Vec::new() }
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.slots.push(Slot { name: name.into(), value: value.into() });
        self
    }

    pub fn slot(&self, name: &str) -> Option<&str> {
        self.slots.iter().find(|s| s.name == name).map(|s| s.value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub text: String,
    pub intents: Vec<Intent>,
    pub speech: String,
    pub clarify: bool,
    pub conversation_id: String,
    #[serde(default)]
    pub chat: bool,
    #[serde(default)]
    pub briefing: bool,
}
