//! Install an unpacked settings backup without risk.
//!
//! Every section runs through the validator its write route uses, before anything is touched.
//! The files as they were are kept as `*.before-restore`. The endpoint file is prepared first
//! and renamed last, so either both files are replaced or neither. The running state is then
//! read again from disk, which also drops aliases and tags the backup does not contain.

use crate::home::load_merged;
use crate::home::overlay::Overlay;
use crate::home::paths::{read_confined, stage_private_confined, write_private_confined, Staged};
use crate::io::dashboard::{sanitize_ui, valid_entity_id};
use crate::io::home_sync::apply_live_overlays;
use crate::io::llm::{load_endpoint, load_stored_endpoint, StoredEndpoint, LLM_FILE};
use crate::io::state::AppState;
use crate::io::web::custom_rows_ok;
use crate::lang::{install_user_overlay, validate_custom, validate_language, MAX_HISTORY};
use crate::llm::LlmEndpoint;
use crate::types::{sanitize_rules, sanitize_speech_bank, CustomSentence};
use std::io;
use std::path::Path;

/// The endpoint file as it was before the last restore replaced it.
pub(crate) const LLM_BEFORE_RESTORE_FILE: &str = "llm_endpoint.json.before-restore";

/// What a backup holds, as read from the archive.
#[derive(Debug)]
pub(crate) struct Packed {
    pub overlay: Overlay,
    pub endpoint: Option<StoredEndpoint>,
}

/// A backup whose every section passed its validator.
#[derive(Debug)]
pub(crate) struct Checked {
    overlay: Overlay,
    endpoint: Option<StoredEndpoint>,
}

/// Run every section through the validator of its write route. `Err` names the section.
pub(crate) fn check(packed: Packed) -> Result<Checked, &'static str> {
    let mut overlay = packed.overlay;
    if !entity_ids(overlay.aliases.keys()) {
        return Err("aliases");
    }
    if !entity_ids(overlay.areas.keys()) || overlay.areas.values().any(|area| area.len() > 128 || area.chars().any(char::is_control)) {
        return Err("areas");
    }
    if !entity_ids(overlay.preferred.iter()) {
        return Err("preferred");
    }
    if !entity_ids(overlay.nlu_ignore.iter()) {
        return Err("nlu_ignore");
    }
    if !custom_ok(&overlay.custom) {
        return Err("custom");
    }
    if !validate_language(&overlay.language).is_empty() {
        return Err("language");
    }
    if overlay.language_history.len() > MAX_HISTORY
        || overlay.language_history.iter().any(|row| !custom_ok(&row.custom) || !validate_language(&row.language).is_empty())
    {
        return Err("language_history");
    }
    overlay.policies = sanitize_rules(overlay.policies).map_err(|_| "policies")?;
    overlay.speech_bank = sanitize_speech_bank(overlay.speech_bank).map_err(|_| "speech_bank")?;
    overlay.match_controls = crate::parse::sanitize_match_controls(overlay.match_controls).map_err(|_| "match_controls")?;
    overlay.ui = sanitize_ui(overlay.ui);
    let endpoint = packed.endpoint.map(checked_endpoint).transpose()?;
    Ok(Checked { overlay, endpoint })
}

fn entity_ids<'a>(mut ids: impl Iterator<Item = &'a String>) -> bool {
    ids.all(|id| valid_entity_id(id))
}

/// Both routes that save phrases: the rules editor and the plain list.
fn custom_ok(rows: &[CustomSentence]) -> bool {
    validate_custom(rows).is_empty() || custom_rows_ok(rows)
}

fn checked_endpoint(stored: StoredEndpoint) -> Result<StoredEndpoint, &'static str> {
    let endpoint = LlmEndpoint::from_parts(&stored.base_url, &stored.api_key, &stored.model).map_err(|_| "llm_endpoint")?;
    Ok(StoredEndpoint { base_url: endpoint.base_url, api_key: endpoint.api_key, model: endpoint.model, ..stored })
}

/// A backup without secrets carries no key. The key on this machine stays only when it would
/// still go to the address it was entered for.
fn with_local_key(mut incoming: StoredEndpoint, local: Option<&StoredEndpoint>) -> StoredEndpoint {
    if incoming.api_key.is_empty() {
        if let Some(local) = local.filter(|local| same_base(&local.base_url, &incoming.base_url)) {
            incoming.api_key.clone_from(&local.api_key);
        }
    }
    incoming
}

fn same_base(left: &str, right: &str) -> bool {
    let normal = |raw: &str| LlmEndpoint::for_discovery(raw, "").map(|endpoint| endpoint.base_url);
    matches!((normal(left), normal(right)), (Ok(left), Ok(right)) if left == right)
}

fn stage_endpoint(dir: &Path, incoming: StoredEndpoint) -> io::Result<Staged> {
    let local = load_stored_endpoint(dir);
    match read_confined(dir, LLM_FILE) {
        Ok(before) => write_private_confined(dir, LLM_BEFORE_RESTORE_FILE, &before)?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    let stored = with_local_key(incoming, local.as_ref());
    let bytes = serde_json::to_vec_pretty(&stored).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stage_private_confined(dir, LLM_FILE, &bytes)
}

/// Replace the files, then load the running state from them. On `Err` the disk holds what it
/// held before and the running state was not touched.
pub(crate) async fn install(state: &AppState, checked: Checked) -> io::Result<()> {
    install_with(state, checked, Staged::commit).await
}

/// [`install`] with the last step, the rename of the endpoint file, handed in.
async fn install_with(state: &AppState, checked: Checked, commit: impl FnOnce(Staged) -> io::Result<()>) -> io::Result<()> {
    let staged = checked.endpoint.map(|incoming| stage_endpoint(&state.data_dir, incoming)).transpose()?;
    state.overlay.restore(&checked.overlay)?;
    if let Some(Err(err)) = staged.map(commit) {
        if let Err(undo) = state.overlay.undo_restore() {
            tracing::error!("Restore abgebrochen, klar_nlu.json nicht zurückgesetzt: {undo}");
            reload_from_disk(state).await;
        }
        return Err(err);
    }
    reload_from_disk(state).await;
    Ok(())
}

/// Bring the running state to what a restart would load. The home is built again from its
/// base, so calibration that is no longer on disk is gone from memory too.
pub(crate) async fn reload_from_disk(state: &AppState) {
    let loaded = load_merged(&state.config_dir, &state.data_dir);
    let graph = match state.live_base.lock().await.clone() {
        Some(base) => apply_live_overlays(state, (*base).clone()),
        None => loaded.graph,
    };
    state.home.replace(graph).await;
    *state.settings.lock().await = loaded.settings;
    *state.custom.lock().await = loaded.custom;
    *state.policies.lock().await = loaded.policies;
    *state.speech_bank.lock().await = loaded.speech_bank;
    *state.match_controls.lock().await = loaded.match_controls;
    install_user_overlay(if loaded.language.sets.is_empty() { None } else { Some(loaded.language) });
    *state.llm.lock().await = load_endpoint(&state.data_dir);
}

#[cfg(test)]
#[path = "restore_tests.rs"]
mod tests;
