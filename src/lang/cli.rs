//! `klar lang` validate / preview / HassIL import / new-case.

use super::external::{ExternalPack, PackIntent, PackRegistry, PACK_FORMAT};
use super::hassil::import_hassil_file;
use super::locale::LocaleId;
use super::resolver::{install_runtime_packs, overlay_for, pin_language};
use super::validate::validate_pack;
use crate::home::default_home;
use crate::nlu::parse;
use crate::session::Session;
use crate::types::{known_intent, ParseDecision, Settings};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn validate_path(path: &Path) -> Result<String, String> {
    if path.ends_with("registry.yaml") {
        let dir = path.parent().unwrap_or(path);
        let packs = PackRegistry::load_dir(dir)?;
        let mut lines = vec![format!("registry {} packs", packs.len())];
        for pack in &packs {
            lines.push(report_pack(pack)?);
        }
        return Ok(lines.join("\n"));
    }
    if path.is_dir() {
        let packs = PackRegistry::load_dir(path)?;
        let mut lines = Vec::new();
        for pack in &packs {
            lines.push(report_pack(pack)?);
        }
        return Ok(lines.join("\n"));
    }
    report_pack(&ExternalPack::load_file(path)?)
}

pub fn preview(text: &str, language: &str, pack: Option<&Path>, pack_dir: Option<&Path>) -> Result<String, String> {
    if let Some(dir) = pack_dir {
        let _ = super::resolver::load_runtime_dir(dir)?;
    }
    if let Some(path) = pack {
        let loaded = ExternalPack::load_file(path)?;
        let mut packs = super::resolver::installed_packs();
        packs.push(loaded);
        install_runtime_packs(packs);
    }
    let tag = match pin_language(language) {
        Ok(tag) => tag,
        Err(err) => {
            if overlay_for(language).is_some() {
                LocaleId::parse(language).map(|locale| locale.tag).map_err(|parse_err| parse_err.to_string())?
            } else {
                return Err(err.to_string());
            }
        }
    };
    let settings = Settings { languages: vec![tag.clone()], ..Settings::default() };
    let home = default_home();
    let mut session = Session::default();
    let outcome = parse(text, &home, &mut session, &[], &settings);
    let decision = match &outcome.decision {
        ParseDecision::Execute => "execute",
        ParseDecision::Confirm { .. } => "confirm",
        ParseDecision::Clarify { .. } => "clarify",
        ParseDecision::Reject { .. } => "reject",
        ParseDecision::Chat => "chat",
        ParseDecision::Error { .. } => "error",
    };
    let intents = outcome
        .plan
        .as_ref()
        .map(|plan| plan.intents().into_iter().map(|intent| intent.name).collect::<Vec<_>>().join(","))
        .unwrap_or_default();
    Ok(format!("language={tag} decision={decision} confidence={:.2} intents={intents}", outcome.confidence))
}

/// Append one regression intent to `packs/assist/{language}.yaml` (package 6.9).
/// Creates the assist file when missing. A pack correction stays one data file.
pub fn new_case(language: &str, phrase: &str, intent: &str, slots: &[String], pack_path: Option<&Path>) -> Result<String, String> {
    let phrase = phrase.trim();
    if phrase.chars().count() < 3 {
        return Err("phrase too short".into());
    }
    if !known_intent(intent) {
        return Err(format!("unknown intent `{intent}`"));
    }
    let locale = LocaleId::parse(language).map_err(|err| err.to_string())?;
    let code = super::LangId::from_tag(&locale.tag).map(|id| id.code().to_string()).unwrap_or_else(|| locale.tag.clone());
    let path = pack_path.map(PathBuf::from).unwrap_or_else(|| PathBuf::from(format!("packs/assist/{code}.yaml")));
    let extends =
        super::LangId::from_code(&code).and_then(|id| id.parent().map(|parent| parent.code().to_string())).unwrap_or_else(|| code.clone());
    let mut pack = if path.is_file() {
        ExternalPack::load_file(&path)?
    } else {
        ExternalPack {
            klar_lang_pack: PACK_FORMAT.into(),
            id: code.clone(),
            bcp47: vec![locale.tag.clone()],
            extends: extends.clone(),
            verbs: HashMap::new(),
            sets: HashMap::new(),
            maps: Default::default(),
            morphology: Default::default(),
            intents: Vec::new(),
            source: None,
        }
    };
    if pack.id.is_empty() {
        pack.id = code.clone();
    }
    if pack.extends.is_empty() {
        pack.extends = extends;
    }
    if pack.bcp47.is_empty() {
        pack.bcp47 = vec![locale.tag.clone()];
    }
    let mut slot_map = HashMap::new();
    for raw in slots {
        let (key, value) = raw.split_once('=').ok_or_else(|| format!("slot must be key=value, got `{raw}`"))?;
        if key.trim().is_empty() || value.trim().is_empty() {
            return Err(format!("slot must be key=value, got `{raw}`"));
        }
        slot_map.insert(key.trim().to_string(), value.trim().to_string());
    }
    if pack.intents.iter().any(|row| row.phrase == phrase) {
        return Err(format!("phrase already present in {}", path.display()));
    }
    pack.intents.push(PackIntent { phrase: phrase.to_string(), intent: intent.to_string(), slots: slot_map, source: None });
    let yaml = serde_yaml::to_string(&assist_yaml(&pack)).map_err(|err| err.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let header = format!("# Assist pack data for {code} (package 6.9).\n# Fix a phrase here — one file per locale.\n");
    std::fs::write(&path, format!("{header}{yaml}")).map_err(|err| err.to_string())?;
    Ok(format!("wrote case to {}", path.display()))
}

pub fn import_hassil(from: &Path, into: Option<&Path>, language: Option<&str>, dry_run: bool) -> Result<String, String> {
    let imported = import_hassil_file(from, language)?;
    let report = format!("imported={} unsupported={}", imported.imported, imported.unsupported.len());
    let yaml = serde_yaml::to_string(&preview_yaml(&imported.pack)).map_err(|err| err.to_string())?;
    if dry_run || into.is_none() {
        let extras = if imported.unsupported.is_empty() { String::new() } else { format!("\n{}", imported.unsupported.join("\n")) };
        return Ok(format!("{report}{extras}\n{yaml}"));
    }
    let fallback = PathBuf::from("overrides.yaml");
    let dest = into.unwrap_or(&fallback);
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(dest, yaml).map_err(|err| err.to_string())?;
    Ok(format!("{report} wrote {}", dest.display()))
}

fn report_pack(pack: &ExternalPack) -> Result<String, String> {
    let base = pack.base_lang().ok().map(|id| super::catalog_for(&[id.code().to_string()]));
    let report = validate_pack(pack, base.as_deref());
    if report.ok() {
        Ok(format!("{} ok bcp47={}", pack.id, pack.bcp47.join(",")))
    } else {
        let details = report.errors.iter().map(|issue| format!("{}: {}", issue.path, issue.message)).collect::<Vec<_>>().join("; ");
        Err(format!("{} invalid: {details}", pack.id))
    }
}

#[derive(serde::Serialize)]
struct PreviewYaml {
    klar_lang_pack: String,
    id: String,
    bcp47: Vec<String>,
    extends: String,
    intents: Vec<PreviewIntent>,
}

#[derive(serde::Serialize)]
struct PreviewIntent {
    phrase: String,
    intent: String,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    slots: std::collections::HashMap<String, String>,
}

fn preview_yaml(pack: &ExternalPack) -> PreviewYaml {
    PreviewYaml {
        klar_lang_pack: pack.klar_lang_pack.clone(),
        id: pack.id.clone(),
        bcp47: pack.bcp47.clone(),
        extends: pack.extends.clone(),
        intents: pack
            .intents
            .iter()
            .map(|intent| PreviewIntent { phrase: intent.phrase.clone(), intent: intent.intent.clone(), slots: intent.slots.clone() })
            .collect(),
    }
}

#[derive(serde::Serialize)]
struct AssistYaml {
    klar_lang_pack: String,
    id: String,
    bcp47: Vec<String>,
    extends: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    verbs: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    sets: HashMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    intents: Vec<PreviewIntent>,
}

fn assist_yaml(pack: &ExternalPack) -> AssistYaml {
    AssistYaml {
        klar_lang_pack: pack.klar_lang_pack.clone(),
        id: pack.id.clone(),
        bcp47: pack.bcp47.clone(),
        extends: pack.extends.clone(),
        verbs: pack.verbs.clone(),
        sets: pack.sets.clone(),
        intents: pack
            .intents
            .iter()
            .map(|intent| PreviewIntent { phrase: intent.phrase.clone(), intent: intent.intent.clone(), slots: intent.slots.clone() })
            .collect(),
    }
}
