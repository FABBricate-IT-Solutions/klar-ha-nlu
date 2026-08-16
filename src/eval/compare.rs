//! Assist-oracle comparison on a frozen HomeGraph.

use crate::home::load_home_config;
use crate::nlu;
use crate::session::Session;
use crate::types::{HomeGraph, Intent, ParseDecision, Settings};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::scorecard::Comparison;

#[derive(Debug, Deserialize)]
struct AssistCase {
    #[serde(default)]
    sentences: AssistSentences,
    #[serde(default)]
    conditions: Vec<AssistCondition>,
    #[serde(default)]
    forbid: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AssistSentences {
    Flat(Vec<String>),
    Turns(Vec<Vec<String>>),
}

impl Default for AssistSentences {
    fn default() -> Self {
        Self::Flat(Vec::new())
    }
}

#[derive(Debug, Deserialize)]
struct AssistCondition {
    #[serde(rename = "type", default = "default_kind")]
    kind: String,
    #[serde(default)]
    entity_id: Option<String>,
    #[serde(default)]
    area: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    state: Option<String>,
}

fn default_kind() -> String {
    "action".into()
}

pub fn compare_assist(name: &str, graph: &str, home: &HomeGraph, language: &str, dir: &Path) -> Result<Comparison, String> {
    let settings = Settings { languages: vec![language.into()], ..Settings::default() };
    let mut cases = 0usize;
    let mut ok = 0usize;
    for path in yaml_files(dir)? {
        for case in load_cases(&path)? {
            for turns in dialogues(&case.sentences) {
                cases += 1;
                let mut session = Session::new();
                let mut last = None;
                for turn in &turns {
                    last = Some(nlu::parse(turn, home, &mut session, &[], &settings));
                }
                let Some(outcome) = last else {
                    continue;
                };
                let intents = match &outcome.decision {
                    ParseDecision::Execute => outcome.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default(),
                    _ => Vec::new(),
                };
                if assist_ok(&case, &intents, home) {
                    ok += 1;
                }
            }
        }
    }
    Ok(Comparison { name: name.into(), graph: graph.into(), cases, ok })
}

pub fn wohnung_mittel_home() -> Result<HomeGraph, String> {
    load_home_config(&datasets_root().join("wohnung_mittel/home_config.yaml"))
}

pub fn wohnung_live_home() -> Result<HomeGraph, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wohnung_live.json");
    let raw = std::fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

pub fn assist_dir(suite: &str) -> PathBuf {
    datasets_root().join(suite).join("assist")
}

fn datasets_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datasets")
}

fn yaml_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !dir.is_dir() {
        return Err(format!("assist corpus missing: {}", dir.display()));
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|err| format!("{}: {err}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("yaml"))
        .collect();
    files.sort();
    Ok(files)
}

fn load_cases(path: &Path) -> Result<Vec<AssistCase>, String> {
    let raw = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let value: serde_yaml::Value = serde_yaml::from_str(&raw).map_err(|err| format!("{}: {err}", path.display()))?;
    if value.as_sequence().is_some() {
        serde_yaml::from_value(value).map_err(|err| format!("{}: {err}", path.display()))
    } else {
        serde_yaml::from_value(value).map(|case| vec![case]).map_err(|err| format!("{}: {err}", path.display()))
    }
}

fn dialogues(sentences: &AssistSentences) -> Vec<Vec<String>> {
    match sentences {
        AssistSentences::Flat(sentences) => sentences.iter().map(|sentence| vec![sentence.clone()]).collect(),
        AssistSentences::Turns(turns) => turns.clone(),
    }
}

fn assist_ok(case: &AssistCase, intents: &[Intent], home: &HomeGraph) -> bool {
    if case.conditions.is_empty() {
        return false;
    }
    if case.forbid.iter().any(|bad| intents.iter().any(|intent| intent.slot("entity_id") == Some(bad) || intent.slot("area") == Some(bad)))
    {
        return false;
    }
    case.conditions.iter().all(|condition| condition_ok(condition, intents, home))
}

fn condition_ok(condition: &AssistCondition, intents: &[Intent], home: &HomeGraph) -> bool {
    let names = expected_names(condition);
    intents.iter().any(|intent| names.contains(&intent.name.as_str()) && target_ok(intent, condition, home))
}

fn expected_names(condition: &AssistCondition) -> Vec<&'static str> {
    if condition.kind == "query" {
        return vec!["HassGetState", "HassClimateGetTemperature"];
    }
    match condition.state.as_deref() {
        Some("off" | "closed" | "unlocked") => vec!["HassTurnOff"],
        _ => vec!["HassTurnOn"],
    }
}

fn target_ok(intent: &Intent, condition: &AssistCondition, home: &HomeGraph) -> bool {
    if let Some(wanted) = condition.entity_id.as_deref() {
        if intent.slot("entity_id") == Some(wanted) {
            return true;
        }
        return home.entities.iter().any(|entity| {
            entity.entity_id == wanted
                && intent.slot("area") == entity.area.as_deref()
                && intent.slot("domain").is_none_or(|domain| domain == entity.domain)
        });
    }
    if let Some(area) = condition.area.as_deref() {
        if intent.slot("area") == Some(area) {
            return condition.domain.as_deref().is_none_or(|domain| intent.slot("domain").is_none_or(|got| got == domain));
        }
        return intent.slot("entity_id").is_some_and(|entity_id| {
            home.entities.iter().any(|entity| {
                entity.entity_id == entity_id
                    && entity.area.as_deref() == Some(area)
                    && condition.domain.as_deref().is_none_or(|domain| entity.domain == domain)
            })
        });
    }
    true
}
