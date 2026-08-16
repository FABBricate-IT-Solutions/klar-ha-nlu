//! Import Home Assistant HassIL custom sentences as Klar pack intents.

use super::external::{ExternalPack, PackIntent, PACK_FORMAT};
use serde_yaml::Value;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HassilImport {
    pub pack: ExternalPack,
    pub unsupported: Vec<String>,
    pub imported: usize,
}

pub fn import_hassil(raw: &str, language: &str, source: &str) -> Result<HassilImport, String> {
    let root: Value = serde_yaml::from_str(raw).map_err(|err| err.to_string())?;
    let mut unsupported = Vec::new();
    let mut intents = Vec::new();
    let file_language = root.get("language").and_then(Value::as_str).unwrap_or(language);
    let Some(intent_map) = root.get("intents").and_then(Value::as_mapping) else {
        return Err("HassIL file has no intents map".into());
    };
    for (name, body) in intent_map {
        let intent = name.as_str().unwrap_or_default();
        if intent.is_empty() {
            unsupported.push(format!("{source}: unnamed intent"));
            continue;
        }
        let Some(rows) = body.get("data").and_then(Value::as_sequence) else {
            unsupported.push(format!("{source}:{intent}: missing data list"));
            continue;
        };
        for (index, row) in rows.iter().enumerate() {
            if row.get("expansion_rules").is_some() {
                unsupported.push(format!("{source}:{intent}[{index}]: expansion_rules"));
            }
            if row.get("lists").is_some() {
                unsupported.push(format!("{source}:{intent}[{index}]: lists"));
            }
            let Some(sentences) = row.get("sentences").and_then(Value::as_sequence) else {
                unsupported.push(format!("{source}:{intent}[{index}]: missing sentences"));
                continue;
            };
            let slots: std::collections::HashMap<String, String> = row
                .get("slots")
                .and_then(Value::as_mapping)
                .map(|map| map.iter().filter_map(|(key, value)| Some((key.as_str()?.to_string(), scalar(value)?))).collect())
                .unwrap_or_default();
            for sentence in sentences {
                let Some(phrase) = sentence.as_str() else {
                    unsupported.push(format!("{source}:{intent}[{index}]: non-string sentence"));
                    continue;
                };
                if phrase.contains('{') || phrase.contains('<') {
                    unsupported.push(format!("{source}:{intent}: template not imported: {phrase}"));
                    continue;
                }
                intents.push(PackIntent {
                    phrase: phrase.to_string(),
                    intent: intent.to_string(),
                    slots: slots.clone(),
                    source: Some(format!("hassil:{source}")),
                });
            }
        }
    }
    let imported = intents.len();
    Ok(HassilImport {
        pack: ExternalPack {
            klar_lang_pack: PACK_FORMAT.to_string(),
            id: format!("hassil-{file_language}"),
            bcp47: vec![file_language.to_string()],
            extends: primary_or_en(file_language),
            verbs: Default::default(),
            sets: Default::default(),
            maps: Default::default(),
            morphology: Default::default(),
            intents,
            source: Some(source.to_string()),
        },
        unsupported,
        imported,
    })
}

pub fn import_hassil_file(path: &Path, language: Option<&str>) -> Result<HassilImport, String> {
    let raw = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let fallback = language.unwrap_or("de");
    import_hassil(&raw, fallback, &path.display().to_string())
}

fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn primary_or_en(tag: &str) -> String {
    let language = super::locale::LocaleId::parse(tag).map(|locale| locale.language).unwrap_or_else(|_| "en".into());
    if super::LangId::from_code(&language).is_some() {
        language
    } else {
        "en".into()
    }
}
