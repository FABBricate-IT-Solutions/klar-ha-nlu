//! User language overlay: set deltas, validation, preview helpers, and rollback snapshots.

use super::validate::set_field;
use crate::types::{known_intent, CustomSentence};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

pub const MAX_USER_INTENTS: usize = 64;
pub const MAX_HISTORY: usize = 8;
const MIN_PHRASE: usize = 4;
const MAX_PHRASE: usize = 200;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetDelta {
    #[serde(default)]
    pub add: Vec<String>,
    #[serde(default)]
    pub remove: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageOverlay {
    #[serde(default)]
    pub sets: HashMap<String, SetDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageRevision {
    pub hash: String,
    pub label: String,
    pub saved_at: String,
    pub custom: Vec<CustomSentence>,
    #[serde(default)]
    pub language: LanguageOverlay,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverlayIssue {
    pub path: String,
    pub message: String,
}

pub fn validate_custom(custom: &[CustomSentence]) -> Vec<OverlayIssue> {
    let mut issues = Vec::new();
    if custom.len() > MAX_USER_INTENTS {
        issues.push(OverlayIssue { path: "custom".into(), message: format!("at most {MAX_USER_INTENTS} phrases") });
    }
    for (index, row) in custom.iter().enumerate() {
        let chars = row.phrase.trim().chars().count();
        if !(MIN_PHRASE..=MAX_PHRASE).contains(&chars) {
            issues.push(OverlayIssue { path: format!("custom.{index}.phrase"), message: "phrase must be 4–200 characters".into() });
        }
        if !known_intent(&row.intent) {
            issues.push(OverlayIssue { path: format!("custom.{index}.intent"), message: format!("unknown intent {}", row.intent) });
        }
    }
    issues
}

pub fn validate_language(overlay: &LanguageOverlay) -> Vec<OverlayIssue> {
    let mut issues = Vec::new();
    for (path, delta) in &overlay.sets {
        if set_field(path).is_none() {
            issues.push(OverlayIssue { path: format!("language.sets.{path}"), message: "unknown set path".into() });
        }
        if delta.add.iter().chain(delta.remove.iter()).any(|word| word.trim().is_empty()) {
            issues.push(OverlayIssue { path: format!("language.sets.{path}"), message: "empty token".into() });
        }
    }
    issues
}

pub fn revision_hash(custom: &[CustomSentence], language: &LanguageOverlay) -> String {
    let mut hasher = DefaultHasher::new();
    let raw = serde_json::to_string(&(custom, language)).unwrap_or_default();
    raw.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn stamp() -> String {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs().to_string()
}

pub fn push_revision(history: &mut Vec<LanguageRevision>, custom: Vec<CustomSentence>, language: LanguageOverlay, label: String) {
    let hash = revision_hash(&custom, &language);
    if history.last().is_some_and(|row| row.hash == hash) {
        return;
    }
    history.push(LanguageRevision { hash, label, saved_at: stamp(), custom, language });
    if history.len() > MAX_HISTORY {
        let drop = history.len() - MAX_HISTORY;
        history.drain(0..drop);
    }
}

pub fn user_overlay_key(overlay: &LanguageOverlay) -> String {
    revision_hash(&[], overlay)
}

pub fn select_revision(history: &[LanguageRevision], hash: Option<&str>) -> Option<LanguageRevision> {
    match hash {
        Some(hash) => history.iter().rev().find(|row| row.hash == hash).cloned(),
        None => history.last().cloned(),
    }
}
