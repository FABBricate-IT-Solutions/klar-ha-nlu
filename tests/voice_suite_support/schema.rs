use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Case {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) setup: Vec<StateRecord>,
    #[serde(default)]
    pub(crate) world_expect: Vec<StateRecord>,
    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,
    pub(crate) sentences: Sentences,
    #[serde(default)]
    pub(crate) forbid: Vec<String>,
    #[serde(default)]
    pub(crate) speech_has: Vec<String>,
    #[serde(default)]
    pub(crate) speech_forbids: Vec<String>,
    #[serde(default)]
    pub(crate) nlu_expect: Option<NluExpectation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NluExpectation {
    #[serde(default)]
    pub(crate) intents: Option<Vec<ExpectedIntent>>,
    #[serde(default)]
    pub(crate) reject: Option<bool>,
    #[serde(default)]
    pub(crate) clarify: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExpectedIntent {
    pub(crate) intent: String,
    #[serde(default)]
    pub(crate) slots: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StateRecord {
    #[serde(flatten)]
    pub(crate) values: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum Sentences {
    Turns(Vec<Vec<String>>),
    Flat(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub(crate) struct Condition {
    #[serde(rename = "type", default = "default_action")]
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) entity_id: Option<String>,
    #[serde(default)]
    pub(crate) area: Option<String>,
    #[serde(default)]
    pub(crate) domain: Option<String>,
    #[serde(default)]
    pub(crate) state: Option<String>,
    #[serde(default)]
    pub(crate) attributes: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub(crate) minutes: Option<i64>,
    #[serde(default)]
    pub(crate) hours: Option<i64>,
    #[serde(default)]
    pub(crate) seconds: Option<i64>,
    #[serde(default)]
    pub(crate) item: Option<String>,
    #[serde(flatten)]
    pub(crate) extra: BTreeMap<String, serde_yaml::Value>,
}

impl Case {
    pub(crate) fn validate_schema(&self) -> Result<(), String> {
        let legacy = !self.conditions.is_empty();
        let exact = self.nlu_expect.is_some();
        if legacy == exact {
            return Err("case must define exactly one oracle: non-empty conditions or nlu_expect".into());
        }
        let Some(expected) = &self.nlu_expect else {
            if !self.world_expect.is_empty() {
                return Err("world_expect requires nlu_expect".into());
            }
            return Ok(());
        };
        if expected.intents.is_none() && expected.reject.is_none() && expected.clarify.is_none() {
            return Err("nlu_expect must assert intents, reject, or clarify".into());
        }
        if expected.reject == Some(true) && expected.clarify == Some(true) {
            return Err("reject and clarify cannot both be true".into());
        }
        if expected.reject == Some(true) && expected.intents.as_ref().is_some_and(|intents| !intents.is_empty()) {
            return Err("a rejected case cannot expect intents".into());
        }
        Ok(())
    }
}

fn default_action() -> String {
    "action".into()
}
