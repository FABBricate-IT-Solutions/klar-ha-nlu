//! External YAML language packs. A third language extends a builtin pack; it does not edit parser code.

use super::locale::LocaleId;
use super::morphology::{LinkingMorpheme, Morphology};
use super::verbs::VerbKind;
use super::LangId;
use crate::types::CustomSentence;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const PACK_FORMAT: &str = "2.0";

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalPack {
    #[serde(default = "default_format")]
    pub klar_lang_pack: String,
    pub id: String,
    #[serde(default)]
    pub bcp47: Vec<String>,
    pub extends: String,
    #[serde(default)]
    pub verbs: HashMap<String, String>,
    #[serde(default)]
    pub sets: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub maps: ExternalMaps,
    #[serde(default)]
    pub morphology: ExternalMorphology,
    #[serde(default)]
    pub intents: Vec<PackIntent>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExternalMaps {
    #[serde(default)]
    pub domain_map: HashMap<String, String>,
    #[serde(default)]
    pub colors: HashMap<String, String>,
    #[serde(default)]
    pub numbers: HashMap<String, i32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExternalMorphology {
    #[serde(default)]
    pub room_suffixes: Vec<String>,
    #[serde(default)]
    pub color_suffixes: Vec<String>,
    #[serde(default)]
    pub linking: Vec<ExternalLinking>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalLinking {
    pub morpheme: String,
    #[serde(default = "default_min_rest")]
    pub min_rest_len: usize,
    #[serde(default)]
    pub require_noun: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackIntent {
    pub phrase: String,
    pub intent: String,
    #[serde(default)]
    pub slots: HashMap<String, String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackRegistry {
    #[serde(default = "default_registry_version")]
    pub version: u32,
    #[serde(default)]
    pub packs: HashMap<String, RegistryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryEntry {
    #[serde(default)]
    pub builtin: bool,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub extends: Option<String>,
}

fn default_format() -> String {
    PACK_FORMAT.to_string()
}

fn default_registry_version() -> u32 {
    1
}

const fn default_min_rest() -> usize {
    1
}

impl ExternalPack {
    pub fn from_yaml(raw: &str) -> Result<Self, String> {
        serde_yaml::from_str(raw).map_err(|err| err.to_string())
    }

    pub fn load_file(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let mut pack = Self::from_yaml(&raw)?;
        pack.source = Some(path.display().to_string());
        Ok(pack)
    }

    pub fn locales(&self) -> Result<Vec<LocaleId>, String> {
        let tags = if self.bcp47.is_empty() { vec![self.id.clone()] } else { self.bcp47.clone() };
        tags.iter().map(|tag| LocaleId::parse(tag).map_err(|err| err.to_string())).collect()
    }

    pub fn base_lang(&self) -> Result<LangId, String> {
        LangId::from_code(&self.extends).ok_or_else(|| format!("external pack {} extends unknown builtin {}", self.id, self.extends))
    }

    pub fn verb_entries(&self) -> Result<Vec<(String, VerbKind)>, String> {
        self.verbs
            .iter()
            .map(|(token, kind)| kind.parse::<VerbKind>().map(|parsed| (token.clone(), parsed)).map_err(|err| format!("{token}: {err}")))
            .collect()
    }

    pub fn custom_intents(&self) -> Vec<CustomSentence> {
        self.intents
            .iter()
            .map(|intent| CustomSentence { phrase: intent.phrase.clone(), intent: intent.intent.clone(), slots: intent.slots.clone() })
            .collect()
    }

    pub fn morphology(&self) -> Morphology {
        Morphology {
            room_suffixes: self.morphology.room_suffixes.iter().map(|item| leak(item)).collect(),
            color_suffixes: self.morphology.color_suffixes.iter().map(|item| leak(item)).collect(),
            linking: self
                .morphology
                .linking
                .iter()
                .map(|item| LinkingMorpheme {
                    morpheme: leak(&item.morpheme),
                    min_rest_len: item.min_rest_len,
                    require_noun: item.require_noun,
                })
                .collect(),
        }
    }
}

impl PackRegistry {
    pub fn load_file(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        serde_yaml::from_str(&raw).map_err(|err| err.to_string())
    }

    pub fn load_dir(dir: &Path) -> Result<Vec<ExternalPack>, String> {
        let registry = Self::load_file(&dir.join("registry.yaml"))?;
        let mut packs = Vec::new();
        let mut paths: Vec<PathBuf> = registry
            .packs
            .values()
            .filter(|entry| !entry.builtin)
            .filter_map(|entry| entry.path.as_ref().map(|rel| dir.join(rel)))
            .collect();
        paths.sort();
        for path in paths {
            packs.push(ExternalPack::load_file(&path)?);
        }
        Ok(packs)
    }
}

pub(super) fn leak(value: &str) -> &'static str {
    Box::leak(value.to_string().into_boxed_str())
}
