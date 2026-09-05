//! Trainer tool handlers. Writes merge and always validate first.

use crate::home::gaps::leftover;
use crate::home::overlay::{load_overlay, save_overlay};
use crate::io::lang_api::persist_language_overlay;
use crate::io::state::AppState;
use crate::io::trainer::{validate, ProposalIn};
use crate::lang::{catalog_for, is_lexicon_path, lexicon_set_paths, LanguageOverlay, SetDelta};
use crate::parse::{match_catalog, sanitize_match_controls};
use crate::types::{sanitize_rules, MatchControl, PolicyRule};
use serde_json::{json, Value};

pub async fn dispatch(state: &AppState, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "list_languages" => Ok(json!({ "languages": state.settings.lock().await.languages })),
        "search_house" => search_house(state, args).await,
        "get_entity" => get_entity(state, args).await,
        "list_lexicon_paths" => Ok(json!({ "paths": lexicon_set_paths() })),
        "get_lexicon" => get_lexicon(state, args).await,
        "list_matchers" => list_matchers(state).await,
        "list_policies" => Ok(json!({ "policies": state.policies.lock().await.clone() })),
        "list_gaps" => list_gaps(state).await,
        "validate_proposal" => validate_now(state, args).await,
        "apply_lexicon" => apply_lexicon(state, args).await,
        "apply_match" => apply_match(state, args).await,
        "apply_house" => apply_house(state, args).await,
        "apply_aliases" => apply_aliases(state, args).await,
        _ => Err(format!("unknown tool {name}")),
    }
}

pub async fn preview_write(state: &AppState, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "apply_lexicon" => {
            let proposal = lexicon_proposal(state, args).await?;
            let out = validate_now(state, &proposal).await?;
            Ok(out)
        }
        "apply_match" => {
            let mut body = args.clone();
            body["layer"] = json!("match");
            validate_now(state, &body).await
        }
        "apply_house" => {
            let mut body = args.clone();
            body["layer"] = json!("house");
            validate_now(state, &body).await
        }
        "apply_aliases" => {
            let entity_id = arg_str(args, "entity_id")?;
            let aliases = string_list(args, "aliases");
            if aliases.is_empty() {
                return Err("aliases required".into());
            }
            let home = state.home.snapshot().await;
            if !home.entities.iter().any(|entity| entity.entity_id == entity_id) {
                return Err("entity is not on the graph".into());
            }
            Ok(json!({"ok": true, "errors": [], "warnings": [], "dry_run": []}))
        }
        _ => Err(format!("{name} is not a write tool")),
    }
}

pub fn write_summary(name: &str, args: &Value) -> String {
    match name {
        "apply_lexicon" => format!(
            "lexicon {} +{} −{}",
            args.get("path").and_then(Value::as_str).unwrap_or("?"),
            string_list(args, "add").len(),
            string_list(args, "remove").len()
        ),
        "apply_match" => format!("match ×{}", args.get("match_controls").and_then(Value::as_array).map(Vec::len).unwrap_or(0)),
        "apply_house" => format!("house ×{}", args.get("policies").and_then(Value::as_array).map(Vec::len).unwrap_or(0)),
        "apply_aliases" => {
            format!("aliases {} +{}", args.get("entity_id").and_then(Value::as_str).unwrap_or("?"), string_list(args, "aliases").len())
        }
        _ => name.to_string(),
    }
}

async fn search_house(state: &AppState, args: &Value) -> Result<Value, String> {
    let query = arg_str(args, "q")?.to_lowercase();
    let home = state.home.snapshot().await;
    let entities: Vec<Value> = home
        .entities
        .iter()
        .filter(|entity| {
            entity.entity_id.to_lowercase().contains(&query)
                || entity.name.to_lowercase().contains(&query)
                || entity.aliases.iter().any(|alias| alias.to_lowercase().contains(&query))
        })
        .take(24)
        .map(|entity| json!({"entity_id": entity.entity_id, "name": entity.name, "area": entity.area, "aliases": entity.aliases}))
        .collect();
    let areas: Vec<Value> = home
        .areas
        .iter()
        .filter(|area| area.area_id.to_lowercase().contains(&query) || area.name.to_lowercase().contains(&query))
        .take(12)
        .map(|area| json!({"area_id": area.area_id, "name": area.name}))
        .collect();
    Ok(json!({ "entities": entities, "areas": areas }))
}

async fn get_entity(state: &AppState, args: &Value) -> Result<Value, String> {
    let entity_id = arg_str(args, "entity_id")?;
    let home = state.home.snapshot().await;
    home.entities
        .iter()
        .find(|entity| entity.entity_id == entity_id)
        .map(|entity| serde_json::to_value(entity).unwrap_or(json!({})))
        .ok_or_else(|| "entity is not on the graph".into())
}

async fn get_lexicon(state: &AppState, args: &Value) -> Result<Value, String> {
    let overlay = load_overlay(&state.data_dir).language;
    if let Some(path) = args.get("path").and_then(Value::as_str).filter(|path| !path.is_empty()) {
        return Ok(json!({ "path": path, "delta": overlay.sets.get(path) }));
    }
    Ok(json!({ "sets": overlay.sets }))
}

async fn list_matchers(state: &AppState) -> Result<Value, String> {
    let overlay = state.match_controls.lock().await.clone();
    let rows: Vec<Value> = match_catalog()
        .into_iter()
        .map(|row| {
            let hit = overlay.iter().find(|item| item.id == row.id);
            json!({"id": row.id, "precedence": hit.and_then(|item| item.precedence).unwrap_or(row.precedence), "enabled": hit.map(|item| item.enabled).unwrap_or(true)})
        })
        .collect();
    Ok(json!({ "matchers": rows }))
}

async fn list_gaps(state: &AppState) -> Result<Value, String> {
    let settings = state.settings.lock().await.clone();
    let home = state.home.snapshot().await;
    let catalog = catalog_for(&settings.languages);
    let gaps: Vec<Value> = leftover(&home, catalog)
        .into_iter()
        .map(|entity| json!({"entity_id": entity.entity_id, "name": entity.name, "area": entity.area}))
        .collect();
    Ok(json!({ "gaps": gaps }))
}

async fn validate_now(state: &AppState, args: &Value) -> Result<Value, String> {
    let proposal: ProposalIn = serde_json::from_value(args.clone()).map_err(|_| "invalid proposal")?;
    let settings = state.settings.lock().await.clone();
    let language = proposal.language.clone().or_else(|| settings.languages.first().cloned()).unwrap_or_else(|| "en".into());
    let home = state.home.snapshot().await;
    let house = match proposal.policies.clone() {
        Some(rules) => rules,
        None => state.policies.lock().await.clone(),
    };
    let match_controls = match proposal.match_controls.clone() {
        Some(rows) => rows,
        None => state.match_controls.lock().await.clone(),
    };
    let overlay = match proposal.language_overlay.clone() {
        Some(language) => language,
        None => load_overlay(&state.data_dir).language,
    };
    let speech_bank = state.speech_bank.lock().await.clone();
    let out = validate(
        &home,
        &settings,
        &language,
        proposal.layer.as_deref().unwrap_or("all"),
        house,
        match_controls,
        overlay,
        &speech_bank,
        proposal.utterances.as_deref().unwrap_or(&[]),
    );
    serde_json::to_value(out).map_err(|_| "validate encode".into())
}

async fn apply_lexicon(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_write(state, "apply_lexicon", args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let language = merged_lexicon(state, args).await?;
    persist_language_overlay(state, language, "trainer lexicon").await.map_err(|_| "persist lexicon")?;
    Ok(json!({ "ok": true }))
}

async fn lexicon_proposal(state: &AppState, args: &Value) -> Result<Value, String> {
    let language = arg_str(args, "language")?;
    let overlay = merged_lexicon(state, args).await?;
    Ok(json!({"layer": "language", "language": language, "language_overlay": overlay}))
}

async fn merged_lexicon(state: &AppState, args: &Value) -> Result<LanguageOverlay, String> {
    let path = arg_str(args, "path")?;
    if !is_lexicon_path(path) {
        return Err(format!("unknown set path {path}"));
    }
    let mut overlay = load_overlay(&state.data_dir).language;
    let delta = overlay.sets.entry(path.to_string()).or_insert_with(SetDelta::default);
    for word in string_list(args, "add") {
        if !delta.add.iter().any(|item| item == &word) {
            delta.add.push(word.clone());
        }
        delta.remove.retain(|item| item != &word);
    }
    for word in string_list(args, "remove") {
        if !delta.remove.iter().any(|item| item == &word) {
            delta.remove.push(word.clone());
        }
        delta.add.retain(|item| item != &word);
    }
    Ok(overlay)
}

async fn apply_match(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_write(state, "apply_match", args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let incoming: Vec<MatchControl> =
        serde_json::from_value(args.get("match_controls").cloned().unwrap_or(json!([]))).map_err(|_| "invalid match_controls")?;
    let incoming = sanitize_match_controls(incoming).map_err(|err| err)?;
    let merged = {
        let mut current = state.match_controls.lock().await.clone();
        for row in incoming {
            if let Some(existing) = current.iter_mut().find(|item| item.id == row.id) {
                existing.enabled = row.enabled;
                if row.precedence.is_some() {
                    existing.precedence = row.precedence;
                }
            } else {
                current.push(row);
            }
        }
        current
    };
    persist_policy_bundle(state, None, Some(merged)).await?;
    Ok(json!({ "ok": true }))
}

async fn apply_house(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_write(state, "apply_house", args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let incoming: Vec<PolicyRule> =
        serde_json::from_value(args.get("policies").cloned().unwrap_or(json!([]))).map_err(|_| "invalid policies")?;
    let incoming = sanitize_rules(incoming).map_err(|err| err)?;
    let merged = {
        let mut current = state.policies.lock().await.clone();
        for rule in incoming {
            if let Some(existing) = current.iter_mut().find(|item| item.id == rule.id) {
                *existing = rule;
            } else {
                current.push(rule);
            }
        }
        current
    };
    persist_policy_bundle(state, Some(merged), None).await?;
    Ok(json!({ "ok": true }))
}

async fn apply_aliases(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_write(state, "apply_aliases", args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let entity_id = arg_str(args, "entity_id")?;
    for alias in string_list(args, "aliases") {
        state.apply_teach(entity_id, &alias).await;
    }
    Ok(json!({ "ok": true }))
}

async fn persist_policy_bundle(
    state: &AppState,
    policies: Option<Vec<PolicyRule>>,
    match_controls: Option<Vec<MatchControl>>,
) -> Result<(), String> {
    let mut overlay = load_overlay(&state.data_dir);
    if let Some(policies) = policies {
        overlay.policies = policies.clone();
        *state.policies.lock().await = policies;
    }
    if let Some(match_controls) = match_controls {
        overlay.match_controls = match_controls.clone();
        *state.match_controls.lock().await = match_controls;
    }
    overlay.speech_bank = state.speech_bank.lock().await.clone();
    save_overlay(&state.data_dir, &overlay).map_err(|_| "save overlay")?;
    Ok(())
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key).and_then(Value::as_str).filter(|item| !item.is_empty()).ok_or_else(|| format!("{key} required"))
}

fn string_list(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(Value::as_str).map(str::trim).filter(|item| !item.is_empty()).map(str::to_string).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::io::state::AppState;
    use crate::types::Settings;

    fn state(tag: &str) -> AppState {
        let dir = std::env::temp_dir().join(format!("klar-trainer-apply-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        AppState::new(
            LoadedHome {
                graph: default_home(),
                settings: Settings::pinned("de"),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        )
    }

    #[tokio::test]
    async fn merge_aliases_and_reject_unknown_entity() {
        let state = state("alias");
        let ok = apply_aliases(&state, &json!({"entity_id":"light.wohnzimmer","aliases":["decke"]})).await.unwrap();
        assert_eq!(ok["ok"], true);
        let home = state.home.snapshot().await;
        let entity = home.entities.iter().find(|item| item.entity_id == "light.wohnzimmer").unwrap();
        assert!(entity.aliases.iter().any(|alias| alias == "decke"));
        assert!(preview_write(&state, "apply_aliases", &json!({"entity_id":"light.missing","aliases":["x"]})).await.is_err());
    }

    #[tokio::test]
    async fn lexicon_merge_keeps_existing_and_validates() {
        let state = state("lex");
        let first = json!({"language":"de","path":"nouns.light_nouns","add":["kugelchen"]});
        apply_lexicon(&state, &first).await.unwrap();
        apply_lexicon(&state, &json!({"language":"de","path":"nouns.light_nouns","add":["lampe"]})).await.unwrap();
        let overlay = load_overlay(&state.data_dir).language;
        let add = &overlay.sets.get("nouns.light_nouns").unwrap().add;
        assert!(add.contains(&"kugelchen".into()));
        assert!(add.contains(&"lampe".into()));
        let bad = preview_write(&state, "apply_lexicon", &json!({"language":"de","path":"nouns.light_nouns","add":["an"]})).await.unwrap();
        assert_eq!(bad["ok"], false);
    }
}
