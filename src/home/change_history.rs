//! Revision snapshots for house rules and engine settings (package 6.7).

use crate::types::{MatchControl, PolicyRule, Settings, SpeechBank};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_CHANGE_HISTORY: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PoliciesRevision {
    pub hash: String,
    pub label: String,
    pub saved_at: String,
    pub policies: Vec<PolicyRule>,
    #[serde(default)]
    pub speech_bank: SpeechBank,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub match_controls: Vec<MatchControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsRevision {
    pub hash: String,
    pub label: String,
    pub saved_at: String,
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryRow {
    pub hash: String,
    pub label: String,
    pub saved_at: String,
}

pub fn policies_hash(policies: &[PolicyRule], speech_bank: &SpeechBank, match_controls: &[MatchControl]) -> String {
    hash_json(&(policies, speech_bank, match_controls))
}

pub fn settings_hash(settings: &Settings) -> String {
    hash_json(settings)
}

fn hash_json(value: &impl Serialize) -> String {
    let mut hasher = DefaultHasher::new();
    let raw = serde_json::to_string(value).unwrap_or_default();
    raw.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn push_policies_revision(
    history: &mut Vec<PoliciesRevision>,
    policies: Vec<PolicyRule>,
    speech_bank: SpeechBank,
    match_controls: Vec<MatchControl>,
    label: String,
) {
    let hash = policies_hash(&policies, &speech_bank, &match_controls);
    if history.last().is_some_and(|row| row.hash == hash) {
        return;
    }
    history.push(PoliciesRevision { hash, label, saved_at: now_stamp(), policies, speech_bank, match_controls });
    trim(history);
}

pub fn push_settings_revision(history: &mut Vec<SettingsRevision>, settings: Settings, label: String) {
    let hash = settings_hash(&settings);
    if history.last().is_some_and(|row| row.hash == hash) {
        return;
    }
    history.push(SettingsRevision { hash, label, saved_at: now_stamp(), settings });
    trim(history);
}

fn now_stamp() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs().to_string()
}

fn trim<T>(history: &mut Vec<T>) {
    if history.len() > MAX_CHANGE_HISTORY {
        let drop = history.len() - MAX_CHANGE_HISTORY;
        history.drain(0..drop);
    }
}

pub fn select_policies(history: &[PoliciesRevision], hash: Option<&str>) -> Option<PoliciesRevision> {
    match hash {
        Some(hash) => history.iter().rev().find(|row| row.hash == hash).cloned(),
        None => history.last().cloned(),
    }
}

pub fn select_settings(history: &[SettingsRevision], hash: Option<&str>) -> Option<SettingsRevision> {
    match hash {
        Some(hash) => history.iter().rev().find(|row| row.hash == hash).cloned(),
        None => history.last().cloned(),
    }
}

pub fn history_rows_policies(history: &[PoliciesRevision]) -> Vec<HistoryRow> {
    history
        .iter()
        .rev()
        .map(|row| HistoryRow { hash: row.hash.clone(), label: row.label.clone(), saved_at: row.saved_at.clone() })
        .collect()
}

pub fn history_rows_settings(history: &[SettingsRevision]) -> Vec<HistoryRow> {
    history
        .iter()
        .rev()
        .map(|row| HistoryRow { hash: row.hash.clone(), label: row.label.clone(), saved_at: row.saved_at.clone() })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PolicyEffect;

    #[test]
    fn policies_history_dedupes_and_caps() {
        let mut history = Vec::new();
        let rule = PolicyRule {
            id: "r1".into(),
            enabled: true,
            label: "one".into(),
            when: Default::default(),
            effect: PolicyEffect::Confirm,
            prefer: None,
            payload: None,
        };
        for index in 0..10 {
            let mut policies = vec![rule.clone()];
            policies[0].label = format!("save-{index}");
            push_policies_revision(&mut history, policies, SpeechBank::default(), Vec::new(), "save".into());
        }
        assert_eq!(history.len(), MAX_CHANGE_HISTORY);
        let last = history.last().unwrap().policies.clone();
        push_policies_revision(&mut history, last, SpeechBank::default(), Vec::new(), "again".into());
        assert_eq!(history.len(), MAX_CHANGE_HISTORY);
    }
}
