//! Lotse house-policy writes: upsert/remove, govern seeds, speech bank.

use crate::home::overlay::{load_overlay, save_overlay};
use crate::io::state::AppState;
use crate::io::trainer::validate_args;
use crate::io::trainer_args::{arg_str, string_list};
use crate::io::trainer_reads::with_view;
use crate::types::{
    govern_safety_seeds, sanitize_rules, sanitize_speech_bank, MatchControl, PolicyHit, PolicyRule, SpeechBank, SpeechBankEntry,
    SpeechVariant,
};
use serde_json::{json, Value};

pub async fn list_seeds(state: &AppState) -> Result<Value, String> {
    let house = state.policies.lock().await.clone();
    let seeds: Vec<Value> = govern_safety_seeds()
        .iter()
        .map(|seed| {
            let overlay = house.iter().find(|rule| rule.id == seed.id);
            json!({
                "id": seed.id,
                "label": overlay.map(|rule| rule.label.clone()).unwrap_or_else(|| seed.label.clone()),
                "effect": PolicyHit::from_effect(overlay.map(|rule| rule.effect).unwrap_or(seed.effect)).as_str(),
                "enabled": overlay.map(|rule| rule.enabled).unwrap_or(seed.enabled),
                "overlay": overlay.is_some(),
            })
        })
        .collect();
    Ok(with_view("seeds", json!({ "seeds": seeds })))
}

pub async fn list_speech(state: &AppState) -> Result<Value, String> {
    let bank = state.speech_bank.lock().await.clone();
    Ok(with_view("speech", json!({ "entries": bank.entries })))
}

pub async fn preview_house(state: &AppState, args: &Value) -> Result<Value, String> {
    let merged = merge_house(state, args).await?;
    let mut body = json!({"layer": "house", "policies": merged});
    if let Some(language) = args.get("language") {
        body["language"] = language.clone();
    }
    validate_args(state, &body).await
}

pub async fn apply_house(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_house(state, args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let merged = merge_house(state, args).await?;
    persist_policy_bundle(state, Some(merged.clone()), None).await?;
    Ok(with_view("write", json!({ "ok": true, "policies": merged.len(), "removed": string_list(args, "remove") })))
}

pub async fn preview_speech(state: &AppState, args: &Value) -> Result<Value, String> {
    let bank = merge_speech(state, args).await?;
    sanitize_speech_bank(bank.clone())?;
    Ok(json!({"ok": true, "errors": [], "warnings": [], "dry_run": [], "entries": bank.entries.len()}))
}

pub async fn apply_speech(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_speech(state, args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let bank = merge_speech(state, args).await?;
    let bank = sanitize_speech_bank(bank)?;
    *state.speech_bank.lock().await = bank.clone();
    persist_policy_bundle(state, None, None).await?;
    Ok(with_view(
        "write",
        json!({ "ok": true, "rule_id": arg_str(args, "rule_id")?, "variants": bank.entries.iter().find(|row| row.rule_id == arg_str(args, "rule_id").unwrap_or("")).map(|row| row.variants.len()).unwrap_or(0) }),
    ))
}

async fn merge_house(state: &AppState, args: &Value) -> Result<Vec<PolicyRule>, String> {
    let incoming: Vec<PolicyRule> =
        serde_json::from_value(args.get("policies").cloned().unwrap_or(json!([]))).map_err(|_| "invalid policies")?;
    let incoming = if incoming.is_empty() { Vec::new() } else { sanitize_rules(incoming)? };
    let remove = string_list(args, "remove");
    if incoming.is_empty() && remove.is_empty() {
        return Err("policies or remove required".into());
    }
    let mut current = state.policies.lock().await.clone();
    for rule in incoming {
        if let Some(existing) = current.iter_mut().find(|item| item.id == rule.id) {
            *existing = rule;
        } else {
            current.push(rule);
        }
    }
    current.retain(|rule| !remove.iter().any(|id| id == &rule.id));
    Ok(current)
}

async fn merge_speech(state: &AppState, args: &Value) -> Result<SpeechBank, String> {
    let rule_id = arg_str(args, "rule_id")?.to_string();
    let variants = parse_variants(args)?;
    let replace = args.get("replace").and_then(Value::as_bool).unwrap_or(true);
    let mut bank = state.speech_bank.lock().await.clone();
    match bank.entries.iter_mut().find(|row| row.rule_id == rule_id) {
        Some(entry) if replace => entry.variants = variants,
        Some(entry) => {
            for variant in variants {
                if let Some(existing) =
                    entry.variants.iter_mut().find(|row| row.language == variant.language && row.personality == variant.personality)
                {
                    *existing = variant;
                } else {
                    entry.variants.push(variant);
                }
            }
        }
        None => bank.entries.push(SpeechBankEntry { rule_id, variants }),
    }
    Ok(bank)
}

fn parse_variants(args: &Value) -> Result<Vec<SpeechVariant>, String> {
    let Some(rows) = args.get("variants").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    rows.iter()
        .map(|row| {
            Ok(SpeechVariant {
                language: row.get("language").and_then(Value::as_str).unwrap_or("").trim().to_string(),
                personality: row.get("personality").and_then(Value::as_str).unwrap_or("").trim().to_string(),
                text: row.get("text").and_then(Value::as_str).unwrap_or("").trim().to_string(),
            })
        })
        .collect()
}

pub async fn persist_policy_bundle(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::types::{Settings, SEED_CONFIRM_LOCK};

    fn state(tag: &str) -> AppState {
        let dir = std::env::temp_dir().join(format!("klar-trainer-house-{tag}-{}", std::process::id()));
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
    async fn apply_house_removes_overlay_and_lists_seeds() {
        let state = state("house");
        apply_house(
            &state,
            &json!({"policies":[{"id":"guest-block","enabled":true,"label":"Guest","when":{"entity_id":"light.wohnzimmer"},"effect":"block"}]}),
        )
        .await
        .unwrap();
        assert_eq!(state.policies.lock().await.len(), 1);
        apply_house(&state, &json!({"remove":["guest-block"]})).await.unwrap();
        assert!(state.policies.lock().await.is_empty());
        apply_house(
            &state,
            &json!({"policies":[{"id":SEED_CONFIRM_LOCK,"enabled":false,"label":"Lock off","when":{"domain":"lock"},"effect":"confirm"}]}),
        )
        .await
        .unwrap();
        let seeds = list_seeds(&state).await.unwrap();
        let lock = seeds["seeds"].as_array().unwrap().iter().find(|row| row["id"] == SEED_CONFIRM_LOCK).unwrap();
        assert_eq!(lock["enabled"], false);
        assert_eq!(lock["overlay"], true);
        apply_house(&state, &json!({"remove":[SEED_CONFIRM_LOCK]})).await.unwrap();
        let seeds = list_seeds(&state).await.unwrap();
        let lock = seeds["seeds"].as_array().unwrap().iter().find(|row| row["id"] == SEED_CONFIRM_LOCK).unwrap();
        assert_eq!(lock["enabled"], true);
        assert_eq!(lock["overlay"], false);
    }

    #[tokio::test]
    async fn apply_speech_writes_bank() {
        let state = state("speech");
        apply_speech(&state, &json!({"rule_id":"guest-block","variants":[{"language":"de","personality":"default","text":"Gastmodus."}]}))
            .await
            .unwrap();
        let listed = list_speech(&state).await.unwrap();
        assert_eq!(listed["entries"][0]["rule_id"], "guest-block");
        assert_eq!(listed["entries"][0]["variants"][0]["text"], "Gastmodus.");
    }
}
