//! One-shot V1 overlay import: dry-run report, then V2-only persist.

use crate::home::overlay::{overlay_path, save_overlay, Overlay};
use crate::home::registry::load_home_config;
use crate::lang::{validate_custom, validate_language};
use crate::types::{known_intent, HomeGraph, Settings};
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

const V2_KEYS: &[&str] = &[
    "aliases",
    "preferred",
    "areas",
    "settings",
    "custom",
    "infra_id",
    "infra_name",
    "timer_hints",
    "preferred_climate",
    "ui",
    "language",
    "language_history",
];

const RISKY_PREFIXES: &[&str] = &["lock.", "cover.", "alarm_control_panel."];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Issue {
    pub kind: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ImportReport {
    pub schema: String,
    pub dry_run: bool,
    pub applied: bool,
    pub conflicts: Vec<Issue>,
    pub orphans: Vec<Issue>,
    pub security: Vec<Issue>,
    pub accepted_custom: usize,
    pub dropped_custom: usize,
}

pub fn inspect(from: &Path, home: Option<&HomeGraph>) -> Result<ImportReport, String> {
    import(from, None, home, false)
}

pub fn apply(from: &Path, into: &Path, home: Option<&HomeGraph>) -> Result<ImportReport, String> {
    import(from, Some(into), home, true)
}

pub fn run_cli(from: &Path, into: Option<&Path>, home: Option<&Path>, apply_write: bool) -> Result<String, String> {
    let loaded = home.map(load_home_config).transpose()?;
    let dest = into.unwrap_or(from);
    let report = if apply_write { apply(from, dest, loaded.as_ref())? } else { inspect(from, loaded.as_ref())? };
    serde_json::to_string_pretty(&report).map_err(|err| err.to_string())
}

fn import(from: &Path, into: Option<&Path>, home: Option<&HomeGraph>, apply_write: bool) -> Result<ImportReport, String> {
    let raw = std::fs::read_to_string(from).map_err(|err| format!("{}: {err}", from.display()))?;
    let value: serde_json::Value = if raw.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&raw).map_err(|err| format!("{}: {err}", from.display()))?
    };
    let mut overlay: Overlay = serde_json::from_value(value.clone()).map_err(|err| format!("{}: {err}", from.display()))?;
    let mut conflicts = unknown_keys(&value);
    let mut orphans = Vec::new();
    let mut security = Vec::new();

    phrase_conflicts(&overlay, &mut conflicts);
    if let Some(home) = home {
        orphan_refs(&overlay, home, &mut orphans);
    }
    security_issues(&overlay, &mut security);

    let before = overlay.custom.len();
    let custom_issues = validate_custom(&overlay.custom);
    overlay.custom.retain(|row| known_intent(&row.intent) && (4..=200).contains(&row.phrase.trim().chars().count()));
    for issue in custom_issues {
        conflicts.push(Issue { kind: "invalid_custom".into(), path: issue.path, message: issue.message });
    }
    for issue in validate_language(&overlay.language) {
        let key = issue.path.strip_prefix("language.sets.").unwrap_or("").to_string();
        overlay.language.sets.remove(&key);
        conflicts.push(Issue { kind: "invalid_language".into(), path: issue.path, message: issue.message });
    }

    let mut settings = overlay.settings.take().unwrap_or_default();
    if !settings.confirm_risky_actions {
        security.push(Issue {
            kind: "unsafe_setting".into(),
            path: "settings.confirm_risky_actions".into(),
            message: "V1 disabled confirm; V2 forces it on".into(),
        });
        settings.confirm_risky_actions = true;
    }
    overlay.settings = Some(Settings { confirm_risky_actions: true, ..settings });

    let dropped_custom = before.saturating_sub(overlay.custom.len());
    let accepted_custom = overlay.custom.len();
    let mut applied = false;
    if apply_write {
        let dest = into.unwrap_or(from);
        let dir = if dest.is_dir() { dest.to_path_buf() } else { dest.parent().unwrap_or(dest).to_path_buf() };
        save_overlay(&dir, &overlay).map_err(|err| format!("{}: {err}", overlay_path(&dir).display()))?;
        applied = true;
    }

    Ok(ImportReport { schema: "v2".into(), dry_run: !apply_write, applied, conflicts, orphans, security, accepted_custom, dropped_custom })
}

fn unknown_keys(value: &serde_json::Value) -> Vec<Issue> {
    let Some(object) = value.as_object() else {
        return Vec::new();
    };
    object
        .keys()
        .filter(|key| !V2_KEYS.contains(&key.as_str()))
        .map(|key| Issue { kind: "unknown_key".into(), path: key.clone(), message: "dropped on V2 save".into() })
        .collect()
}

fn phrase_conflicts(overlay: &Overlay, conflicts: &mut Vec<Issue>) {
    let mut seen = HashSet::new();
    for (index, row) in overlay.custom.iter().enumerate() {
        let key = row.phrase.trim().to_ascii_lowercase();
        if !seen.insert(key) {
            conflicts.push(Issue {
                kind: "duplicate_phrase".into(),
                path: format!("custom.{index}.phrase"),
                message: "duplicate custom phrase".into(),
            });
        }
    }
}

fn orphan_refs(overlay: &Overlay, home: &HomeGraph, orphans: &mut Vec<Issue>) {
    let entities: BTreeSet<&str> = home.entities.iter().map(|entity| entity.entity_id.as_str()).collect();
    let areas: BTreeSet<&str> = home.areas.iter().map(|area| area.area_id.as_str()).collect();
    for entity_id in overlay.aliases.keys().chain(overlay.areas.keys()).chain(overlay.preferred.iter()) {
        if !entities.contains(entity_id.as_str()) {
            orphans.push(Issue { kind: "orphan_entity".into(), path: entity_id.clone(), message: "not in home graph".into() });
        }
    }
    for area in overlay.areas.values() {
        if !area.is_empty() && !areas.contains(area.as_str()) {
            orphans.push(Issue { kind: "orphan_area".into(), path: area.clone(), message: "area missing from home graph".into() });
        }
    }
}

fn security_issues(overlay: &Overlay, security: &mut Vec<Issue>) {
    for (index, row) in overlay.custom.iter().enumerate() {
        if !known_intent(&row.intent) {
            security.push(Issue {
                kind: "unknown_intent".into(),
                path: format!("custom.{index}.intent"),
                message: format!("unknown intent {}", row.intent),
            });
        }
        if let Some(entity_id) = row.slots.get("entity_id") {
            if RISKY_PREFIXES.iter().any(|prefix| entity_id.starts_with(prefix)) {
                security.push(Issue {
                    kind: "risky_custom".into(),
                    path: format!("custom.{index}.slots.entity_id"),
                    message: format!("custom phrase targets {entity_id}"),
                });
            }
        }
    }
}
