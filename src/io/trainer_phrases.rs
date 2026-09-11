//! Lotse custom-sentence writes. House-global; no language field.

use crate::io::lang_api::persist_custom;
use crate::io::state::AppState;
use crate::io::trainer_args::string_list;
use crate::io::trainer_reads::with_view;
use crate::lang::validate_custom;
use crate::types::CustomSentence;
use serde_json::{json, Value};
use std::collections::HashMap;

pub async fn preview_phrases(state: &AppState, args: &Value) -> Result<Value, String> {
    let next = merge_phrases(state, args).await?;
    let errors: Vec<Value> = validate_custom(&next).into_iter().map(|row| json!({"path": row.path, "message": row.message})).collect();
    Ok(json!({"ok": errors.is_empty(), "errors": errors, "warnings": [], "dry_run": [], "phrases": next.len()}))
}

pub async fn apply_phrases(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_phrases(state, args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let next = merge_phrases(state, args).await?;
    persist_custom(state, next.clone(), "trainer phrases").await.map_err(|_| "persist phrases")?;
    Ok(with_view("write", json!({ "ok": true, "phrases": next.len() })))
}

async fn merge_phrases(state: &AppState, args: &Value) -> Result<Vec<CustomSentence>, String> {
    let add = parse_add(args)?;
    let remove = string_list(args, "remove");
    if add.is_empty() && remove.is_empty() {
        return Err("add or remove required".into());
    }
    let mut next = state.custom.lock().await.clone();
    next.retain(|row| !remove.iter().any(|phrase| phrase == &row.phrase));
    for row in add {
        if let Some(existing) = next.iter_mut().find(|item| item.phrase == row.phrase) {
            *existing = row;
        } else {
            next.push(row);
        }
    }
    Ok(next)
}

fn parse_add(args: &Value) -> Result<Vec<CustomSentence>, String> {
    let Some(rows) = args.get("add").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    rows.iter()
        .map(|row| {
            let phrase =
                row.get("phrase").and_then(Value::as_str).map(str::trim).filter(|item| !item.is_empty()).ok_or("phrase required")?;
            let intent =
                row.get("intent").and_then(Value::as_str).map(str::trim).filter(|item| !item.is_empty()).ok_or("intent required")?;
            let slots = row
                .get("slots")
                .and_then(Value::as_object)
                .map(|map| {
                    map.iter()
                        .filter_map(|(key, value)| value.as_str().map(|item| (key.clone(), item.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default();
            Ok(CustomSentence { phrase: phrase.to_string(), intent: intent.to_string(), slots })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::types::Settings;

    fn state(tag: &str) -> AppState {
        let dir = std::env::temp_dir().join(format!("klar-trainer-phrases-{tag}-{}", std::process::id()));
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
    async fn apply_phrases_adds_and_removes() {
        let state = state("add");
        apply_phrases(
            &state,
            &json!({"add":[{"phrase":"filmabend starten","intent":"HassTurnOn","slots":{"entity_id":"scene.filmabend"}}]}),
        )
        .await
        .unwrap();
        let custom = state.custom.lock().await.clone();
        assert_eq!(custom.len(), 1);
        assert_eq!(custom[0].phrase, "filmabend starten");
        assert_eq!(custom[0].intent, "HassTurnOn");
        apply_phrases(&state, &json!({"remove":["filmabend starten"]})).await.unwrap();
        assert!(state.custom.lock().await.is_empty());
        let bad = preview_phrases(&state, &json!({"add":[{"phrase":"ok","intent":"HassTurnOn"}]})).await.unwrap();
        assert_eq!(bad["ok"], false);
        let unknown = preview_phrases(&state, &json!({"add":[{"phrase":"mach den kaffee","intent":"NoSuchIntent"}]})).await.unwrap();
        assert_eq!(unknown["ok"], false);
    }
}
