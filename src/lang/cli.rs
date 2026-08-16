//! `klar lang` validate / preview / HassIL import.

use super::external::{ExternalPack, PackRegistry};
use super::hassil::import_hassil_file;
use super::locale::LocaleId;
use super::resolver::{install_runtime_packs, overlay_for, pin_language};
use super::validate::validate_pack;
use crate::home::default_home;
use crate::nlu::parse;
use crate::session::Session;
use crate::types::{ParseDecision, Settings};
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
    let report = validate_pack(pack, base);
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
