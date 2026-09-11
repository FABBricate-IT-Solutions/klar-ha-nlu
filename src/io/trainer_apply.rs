//! Trainer tool handlers. Writes merge and always validate first.

use crate::home::overlay::load_overlay;
use crate::io::lang_api::persist_language_overlay;
use crate::io::state::AppState;
use crate::io::trainer::validate_args;
use crate::io::trainer_args::{arg_str, string_list};
use crate::io::trainer_reads::with_view;
use crate::io::{trainer_entity, trainer_house, trainer_phrases, trainer_reads, trainer_settings, trainer_turns};
use crate::lang::{is_lexicon_path, lexicon_set_paths, LanguageOverlay};
use crate::parse::{match_catalog, sanitize_match_controls};
use crate::types::MatchControl;
use serde_json::{json, Value};

pub async fn dispatch(state: &AppState, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "list_languages" => Ok(with_view("languages", json!({ "languages": state.settings.lock().await.languages }))),
        "search_house" => trainer_entity::search_house(state, args).await,
        "get_entity" => trainer_entity::get_entity(state, args).await,
        "list_lexicon_paths" => Ok(with_view("lexicon", json!({ "paths": lexicon_set_paths() }))),
        "get_lexicon" => get_lexicon(state, args).await,
        "list_matchers" => list_matchers(state).await,
        "list_policies" => Ok(with_view("policies", json!({ "policies": state.policies.lock().await.clone() }))),
        "list_seeds" => trainer_house::list_seeds(state).await,
        "list_speech" => trainer_house::list_speech(state).await,
        "list_gaps" => trainer_entity::list_gaps(state).await,
        "validate_proposal" => validate_args(state, args).await,
        "explain_klar" => trainer_reads::explain_klar(args),
        "try_sentence" => trainer_reads::try_sentence(state, args).await,
        "list_areas" => trainer_reads::list_areas(state).await,
        "list_floors" => trainer_reads::list_floors(state).await,
        "count_house" => trainer_reads::count_house(state).await,
        "list_engine" => trainer_settings::list_engine(state).await,
        "list_phrases" => trainer_reads::list_phrases(state).await,
        "list_turns" => trainer_turns::list_turns(state, args).await,
        "apply_lexicon" => apply_lexicon(state, args).await,
        "apply_phrases" => trainer_phrases::apply_phrases(state, args).await,
        "apply_match" => apply_match(state, args).await,
        "apply_house" => trainer_house::apply_house(state, args).await,
        "apply_aliases" => apply_aliases(state, args).await,
        "apply_entity" => trainer_entity::apply_entity(state, args).await,
        "apply_area" => apply_area(state, args).await,
        "apply_speech" => trainer_house::apply_speech(state, args).await,
        "apply_engine" => trainer_settings::apply_engine(state, args).await,
        "apply_ui" => trainer_settings::apply_ui(state, args).await,
        _ => Err(format!("unknown tool {name}")),
    }
}

pub async fn preview_write(state: &AppState, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "apply_lexicon" => {
            let proposal = lexicon_proposal(state, args).await?;
            validate_args(state, &proposal).await
        }
        "apply_phrases" => trainer_phrases::preview_phrases(state, args).await,
        "apply_match" => {
            let mut body = args.clone();
            body["layer"] = json!("match");
            validate_args(state, &body).await
        }
        "apply_house" => trainer_house::preview_house(state, args).await,
        "apply_aliases" => preview_aliases(state, args).await,
        "apply_entity" => trainer_entity::preview_entity(state, args).await,
        "apply_area" => {
            let assigned = resolve_area_rows(state, args).await?;
            Ok(json!({"ok": true, "errors": [], "warnings": [], "dry_run": [], "assigned": assigned}))
        }
        "apply_speech" => trainer_house::preview_speech(state, args).await,
        "apply_engine" => {
            let settings = state.settings.lock().await.clone();
            trainer_settings::preview_engine(&settings, args)
        }
        "apply_ui" => {
            let ui = crate::home::overlay::load_overlay(&state.data_dir).ui;
            trainer_settings::preview_ui(&ui.theme, &ui.locale, args)
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
        "apply_phrases" => format!(
            "phrases +{} −{}",
            args.get("add").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            string_list(args, "remove").len()
        ),
        "apply_match" => format!("match ×{}", args.get("match_controls").and_then(Value::as_array).map(Vec::len).unwrap_or(0)),
        "apply_house" => format!(
            "house ×{} −{}",
            args.get("policies").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            string_list(args, "remove").len()
        ),
        "apply_aliases" => {
            format!("aliases {} +{}", args.get("entity_id").and_then(Value::as_str).unwrap_or("?"), string_list(args, "aliases").len())
        }
        "apply_entity" => format!("entity {}", args.get("entity_id").and_then(Value::as_str).unwrap_or("?")),
        "apply_area" => {
            let n = args.get("assignments").and_then(Value::as_array).map(Vec::len).unwrap_or(1);
            format!("area ×{n}")
        }
        "apply_speech" => format!("speech {}", args.get("rule_id").and_then(Value::as_str).unwrap_or("?")),
        "apply_engine" => trainer_settings::engine_summary(args),
        "apply_ui" => trainer_settings::ui_summary(args),
        _ => name.to_string(),
    }
}

async fn preview_aliases(state: &AppState, args: &Value) -> Result<Value, String> {
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

async fn get_lexicon(state: &AppState, args: &Value) -> Result<Value, String> {
    let overlay = load_overlay(&state.data_dir).language;
    if let Some(path) = args.get("path").and_then(Value::as_str).filter(|path| !path.is_empty()) {
        return Ok(with_view("lexicon", json!({ "path": path, "delta": overlay.sets.get(path) })));
    }
    Ok(with_view("lexicon", json!({ "sets": overlay.sets })))
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
    Ok(with_view("matchers", json!({ "matchers": rows })))
}

async fn apply_lexicon(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_write(state, "apply_lexicon", args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let language = merged_lexicon(state, args).await?;
    persist_language_overlay(state, language, "trainer lexicon").await.map_err(|_| "persist lexicon")?;
    Ok(with_view("write", json!({ "ok": true })))
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
    let delta = overlay.sets.entry(path.to_string()).or_default();
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
    let incoming = sanitize_match_controls(incoming)?;
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
    trainer_house::persist_policy_bundle(state, None, Some(merged)).await?;
    Ok(with_view("write", json!({ "ok": true })))
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
    Ok(with_view("write", json!({ "ok": true, "entity_id": entity_id, "aliases": string_list(args, "aliases") })))
}

async fn apply_area(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_write(state, "apply_area", args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let assigned = resolve_area_rows(state, args).await?;
    let rows: Vec<(String, String)> = assigned
        .iter()
        .filter_map(|row| Some((row.get("entity_id")?.as_str()?.to_string(), row.get("area")?.as_str()?.to_string())))
        .collect();
    state.apply_areas(&rows).await;
    Ok(with_view("write", json!({ "ok": true, "assigned": assigned })))
}

async fn resolve_area_rows(state: &AppState, args: &Value) -> Result<Vec<Value>, String> {
    let home = state.home.snapshot().await;
    let mut out = Vec::new();
    for (entity_id, raw_area) in area_assignments(args)? {
        if !home.entities.iter().any(|entity| entity.entity_id == entity_id) {
            return Err(format!("entity is not on the graph: {entity_id}"));
        }
        let area = resolve_area(&home, &raw_area)?;
        out.push(json!({"entity_id": entity_id, "area": area}));
    }
    Ok(out)
}

fn area_assignments(args: &Value) -> Result<Vec<(String, String)>, String> {
    if let Some(rows) = args.get("assignments").and_then(Value::as_array) {
        if rows.is_empty() {
            return Err("assignments required".into());
        }
        if rows.len() > 40 {
            return Err("too many assignments".into());
        }
        return rows.iter().map(|row| Ok((arg_str(row, "entity_id")?.to_string(), arg_area(row)?))).collect();
    }
    Ok(vec![(arg_str(args, "entity_id")?.to_string(), arg_area(args)?)])
}

fn arg_area(args: &Value) -> Result<String, String> {
    args.get("area").and_then(Value::as_str).map(|item| item.trim().to_string()).ok_or_else(|| "area required".into())
}

fn resolve_area(home: &crate::types::HomeGraph, raw: &str) -> Result<String, String> {
    if raw.is_empty() {
        return Ok(String::new());
    }
    let folded = crate::parse::normalize::compact(raw);
    let hits: Vec<&str> = home
        .areas
        .iter()
        .filter(|area| {
            crate::parse::normalize::compact(&area.area_id) == folded
                || crate::parse::normalize::compact(&area.name) == folded
                || area.aliases.iter().any(|alias| crate::parse::normalize::compact(alias) == folded)
        })
        .map(|area| area.area_id.as_str())
        .collect();
    match hits.as_slice() {
        [area_id] => Ok((*area_id).to_string()),
        [] => Err(format!("area is not on the graph: {raw}")),
        _ => Err(format!("area is ambiguous: {raw}")),
    }
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
    async fn apply_area_sets_room_and_rejects_bad_ids() {
        let state = state("area");
        let ok = apply_area(&state, &json!({"entity_id":"light.wohnzimmer","area":"Büro"})).await.unwrap();
        assert_eq!(ok["ok"], true);
        assert_eq!(ok["assigned"][0]["area"], "arbeitszimmer");
        let home = state.home.snapshot().await;
        let entity = home.entities.iter().find(|item| item.entity_id == "light.wohnzimmer").unwrap();
        assert_eq!(entity.area.as_deref(), Some("arbeitszimmer"));
        let overlay = load_overlay(&state.data_dir);
        assert_eq!(overlay.areas.get("light.wohnzimmer").map(String::as_str), Some("arbeitszimmer"));
        let batch = apply_area(
            &state,
            &json!({"assignments":[
                {"entity_id":"light.wohnzimmer","area":"wohnzimmer"},
                {"entity_id":"light.kuche_kuche","area":""}
            ]}),
        )
        .await
        .unwrap();
        assert_eq!(batch["assigned"][1]["area"], "");
        assert!(preview_write(&state, "apply_area", &json!({"entity_id":"light.missing","area":"wohnzimmer"})).await.is_err());
        assert!(preview_write(&state, "apply_area", &json!({"entity_id":"light.wohnzimmer","area":"keller"})).await.is_err());
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
