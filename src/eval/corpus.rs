use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const MIN_HELD_OUT: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Split {
    Control,
    Asr,
    Ood,
    Clarify,
    Multi,
    Adversarial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldIntent {
    pub name: String,
    pub slots: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EvalItem {
    pub name: String,
    pub split: Split,
    pub language: String,
    pub turns: Vec<String>,
    pub expect_intents: Option<Vec<GoldIntent>>,
    pub expect_reject: bool,
    pub expect_clarify: bool,
}

#[derive(Debug, Deserialize)]
struct FileCase {
    #[serde(default)]
    name: String,
    #[serde(default)]
    split: Option<Split>,
    #[serde(default)]
    language: Option<String>,
    sentences: Sentences,
    #[serde(default)]
    nlu_expect: Option<NluExpectation>,
}

#[derive(Debug, Deserialize)]
struct NluExpectation {
    #[serde(default)]
    intents: Option<Vec<ExpectedIntent>>,
    #[serde(default)]
    reject: Option<bool>,
    #[serde(default)]
    clarify: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ExpectedIntent {
    intent: String,
    #[serde(default)]
    slots: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Sentences {
    Turns(Vec<Vec<String>>),
    Flat(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FileDoc {
    Many(Vec<FileCase>),
    One(FileCase),
}

pub fn load_corpus(dir: &Path, language: &str) -> Result<Vec<EvalItem>, String> {
    if !dir.is_dir() {
        return Err(format!("held-out corpus missing: {}", dir.display()));
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|err| format!("{}: {err}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("yaml"))
        .collect();
    files.sort();
    let mut items = Vec::new();
    for path in files {
        items.extend(load_file(&path, language)?);
    }
    Ok(items)
}

fn load_file(path: &Path, language: &str) -> Result<Vec<EvalItem>, String> {
    let raw = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let doc: FileDoc = serde_yaml::from_str(&raw).map_err(|err| format!("{}: {err}", path.display()))?;
    let cases = match doc {
        FileDoc::Many(cases) => cases,
        FileDoc::One(case) => vec![case],
    };
    let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or("case");
    let mut items = Vec::new();
    for (index, case) in cases.into_iter().enumerate() {
        let expect = case.nlu_expect.as_ref();
        let expect_intents = expect.and_then(|value| value.intents.as_ref()).map(|intents| {
            intents
                .iter()
                .map(|intent| GoldIntent {
                    name: intent.intent.clone(),
                    slots: intent.slots.iter().filter_map(|(key, value)| scalar(value).map(|text| (key.clone(), text))).collect(),
                })
                .collect()
        });
        let expect_reject = expect.and_then(|value| value.reject).unwrap_or(false);
        let expect_clarify = expect.and_then(|value| value.clarify).unwrap_or(false);
        let split = case.split.unwrap_or(Split::Control);
        let lang = case.language.unwrap_or_else(|| language.into());
        if lang != language && !lang.starts_with(language) {
            continue;
        }
        let name = if case.name.is_empty() { format!("{stem}_{index}") } else { case.name };
        match case.sentences {
            Sentences::Flat(sentences) => {
                for (turn_index, sentence) in sentences.into_iter().enumerate() {
                    items.push(EvalItem {
                        name: format!("{name}::{turn_index}"),
                        split,
                        language: lang.clone(),
                        turns: vec![sentence],
                        expect_intents: expect_intents.clone(),
                        expect_reject,
                        expect_clarify,
                    });
                }
            }
            Sentences::Turns(dialogues) => {
                for (turn_index, turns) in dialogues.into_iter().enumerate() {
                    items.push(EvalItem {
                        name: format!("{name}::dlg{turn_index}"),
                        split,
                        language: lang.clone(),
                        turns,
                        expect_intents: expect_intents.clone(),
                        expect_reject,
                        expect_clarify,
                    });
                }
            }
        }
    }
    Ok(items)
}

fn scalar(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::Null => Some("null".into()),
        serde_yaml::Value::Bool(value) => Some(value.to_string()),
        serde_yaml::Value::Number(value) => Some(value.to_string()),
        serde_yaml::Value::String(value) => Some(value.clone()),
        _ => None,
    }
}
