//! Validate external packs against the schema and the builtin they extend.

use super::external::{ExternalPack, PACK_FORMAT};
use super::locale::LocaleId;
use super::Catalog;
use crate::types::known_intent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn validate_pack(pack: &ExternalPack, base: Option<&Catalog>) -> ValidationReport {
    let mut report = ValidationReport::default();
    if pack.klar_lang_pack != PACK_FORMAT {
        report.errors.push(issue("klar_lang_pack", format!("expected {PACK_FORMAT}, got {}", pack.klar_lang_pack)));
    }
    if pack.id.trim().is_empty() {
        report.errors.push(issue("id", "pack id is required"));
    }
    if LocaleId::parse(&pack.extends).is_err() {
        report.errors.push(issue("extends", format!("invalid builtin tag {}", pack.extends)));
    } else if pack.base_lang().is_err() {
        report.errors.push(issue("extends", format!("no builtin pack for {}", pack.extends)));
    }
    if pack.locales().is_err() {
        report.errors.push(issue("bcp47", "contains an invalid BCP-47 tag"));
    }
    match pack.verb_entries() {
        Ok(entries) => {
            if let Some(catalog) = base {
                for (token, kind) in entries {
                    if let Some(existing) = catalog.verb(&token) {
                        if existing != kind {
                            report.errors.push(issue(
                                &format!("verbs.{token}"),
                                format!("conflicts with builtin {} (use a new token or keep {existing:?})", pack.extends),
                            ));
                        }
                    }
                }
            }
        }
        Err(err) => report.errors.push(issue("verbs", err)),
    }
    for (path, words) in &pack.sets {
        if set_field(path).is_none() {
            report.errors.push(issue(&format!("sets.{path}"), "unknown set path"));
        }
        if words.iter().any(|word| word.trim().is_empty()) {
            report.errors.push(issue(&format!("sets.{path}"), "empty token"));
        }
    }
    for intent in &pack.intents {
        if intent.phrase.trim().len() < 2 {
            report.errors.push(issue("intents.phrase", "phrase is too short"));
        }
        if !known_intent(&intent.intent) {
            report.errors.push(issue("intents.intent", format!("unknown intent {}", intent.intent)));
        }
    }
    report
}

pub(super) fn set_field(path: &str) -> Option<fn(&mut Catalog) -> &mut std::collections::HashSet<&'static str>> {
    Some(match path {
        "talk.fillers" => |catalog| &mut catalog.fillers,
        "talk.action_keep" => |catalog| &mut catalog.action_keep,
        "talk.conjunctions" => |catalog| &mut catalog.conjunctions,
        "talk.particles" => |catalog| &mut catalog.particles,
        "talk.affirm" => |catalog| &mut catalog.affirm,
        "nouns.light_nouns" => |catalog| &mut catalog.light_nouns,
        "nouns.light_singular" => |catalog| &mut catalog.light_singular,
        "nouns.light_plural" => |catalog| &mut catalog.light_plural,
        "nouns.cover_nouns" => |catalog| &mut catalog.cover_nouns,
        "nouns.climate_nouns" => |catalog| &mut catalog.climate_nouns,
        "nouns.lock_nouns" => |catalog| &mut catalog.lock_nouns,
        "nouns.scene_nouns" => |catalog| &mut catalog.scene_nouns,
        "cues.on_words" => |catalog| &mut catalog.on_words,
        "cues.off_words" => |catalog| &mut catalog.off_words,
        "cues.extra_device_nouns" => |catalog| &mut catalog.extra_device_nouns,
        _ => return None,
    })
}

fn issue(path: &str, message: impl Into<String>) -> ValidationIssue {
    ValidationIssue { path: path.to_string(), message: message.into() }
}
