//! Read-only Lotse tools. Each payload carries a `view` the operator UI renders.

use crate::home::gaps::leftover;
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::state::AppState;
use crate::lang::catalog_for;
use crate::nlu::parse_with_controls;
use crate::session::Session;
use serde_json::{json, Value};

pub fn with_view(view: &str, mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.insert("view".into(), json!(view));
    }
    value
}

pub fn explain_klar(args: &Value) -> Result<Value, String> {
    let topic = args.get("topic").and_then(Value::as_str).unwrap_or("architecture");
    match topic {
        "setup" => Ok(with_view(
            "guide",
            json!({
                "title": "Household path",
                "steps": [
                    {"id": "install", "label": "HACS + engine", "hint": "Same CalVer. One engine host. POST /api/v2/parse."},
                    {"id": "expose", "label": "Expose devices", "hint": "Assist must see the entity or Klar cannot steer it."},
                    {"id": "pipeline", "label": "Pipeline", "hint": "Conversation engine = Klar NLU. Never an HA LLM agent."},
                    {"id": "lab", "label": "Five sentences", "hint": "Lab is the Assist path for the pinned language."},
                    {"id": "map", "label": "House mapping", "hint": "Aliases and rooms sit on HA names. HA stays the database."}
                ]
            }),
        )),
        "tradeoffs" => Ok(with_view(
            "guide",
            json!({
                "title": "Trade-offs",
                "steps": [
                    {"id": "pro-local", "label": "Local parse", "hint": "nlu::parse has no model. The house still works if the LLM is down."},
                    {"id": "pro-lanes", "label": "Visible lanes", "hint": "Match, language, and house are overlays you can see and roll back."},
                    {"id": "con-slang", "label": "New slang", "hint": "Needs a lexicon token or a custom sentence. Klar will not invent matchers."},
                    {"id": "con-expose", "label": "Unexposed looks missing", "hint": "Generic words in a multi-light room clarify. Hidden entities never bind."}
                ]
            }),
        )),
        "llm" => Ok(with_view(
            "guide",
            json!({
                "title": "Engine LLM",
                "steps": [
                    {"id": "one", "label": "One endpoint", "hint": "Settings only. Assist chat, refine, calendar, and Lotse share it."},
                    {"id": "refine", "label": "Refine", "hint": "Restyles speech Klar already produced. Not a second engine."},
                    {"id": "tools", "label": "Assist tools", "hint": "Off by default. If on, only after Klar parse on chat/reject."},
                    {"id": "think", "label": "Thinking models", "hint": "Leave thinking off or Gemma fills reasoning_content and leaves content empty."}
                ]
            }),
        )),
        _ => Ok(with_view(
            "architecture",
            json!({
                "title": "One sentence, three lanes",
                "steps": [
                    {"id": "match", "label": "Match", "hint": "Compiled PolicyId catalog. Overlay may enable or reorder. No new ids."},
                    {"id": "language", "label": "Language", "hint": "Pack lexicon, slang overlay, and govern seeds shipped with every pack."},
                    {"id": "house", "label": "House", "hint": "This graph's rules and aliases. First house hit wins over a seed."}
                ]
            }),
        )),
    }
}

pub async fn try_sentence(state: &AppState, args: &Value) -> Result<Value, String> {
    let text = arg_str(args, "text")?;
    if text.chars().count() > MAX_PARSE_CHARS {
        return Err("text too long".into());
    }
    let mut settings = state.settings.lock().await.clone();
    if let Some(language) = args.get("language").and_then(Value::as_str).filter(|item| !item.is_empty()) {
        settings.languages = vec![language.to_string()];
    }
    let policies = state.policies.lock().await.clone();
    let match_controls = state.match_controls.lock().await.clone();
    let speech_bank = state.speech_bank.lock().await.clone();
    let custom = state.custom.lock().await.clone();
    let home = state.home.snapshot().await;
    let mut session = Session::new();
    let outcome = parse_with_controls(text, &home, &mut session, &custom, &settings, &policies, &speech_bank, &match_controls);
    Ok(with_view(
        "path",
        json!({
            "text": text,
            "speech": outcome.speech,
            "policy_trace": outcome.policy_trace
        }),
    ))
}

pub async fn list_areas(state: &AppState) -> Result<Value, String> {
    let home = state.home.snapshot().await;
    let areas: Vec<Value> = home.areas.iter().map(|area| json!({"area_id": area.area_id, "name": area.name})).collect();
    Ok(with_view("areas", json!({ "areas": areas })))
}

pub async fn count_house(state: &AppState) -> Result<Value, String> {
    let settings = state.settings.lock().await.clone();
    let home = state.home.snapshot().await;
    let leftover = leftover(&home, catalog_for(&settings.languages)).len();
    Ok(with_view(
        "counts",
        json!({
            "entities": home.entities.len(),
            "areas": home.areas.len(),
            "leftover": leftover
        }),
    ))
}

pub async fn list_phrases(state: &AppState) -> Result<Value, String> {
    let custom = state.custom.lock().await.clone();
    let phrases: Vec<Value> = custom.into_iter().take(32).map(|row| json!({"phrase": row.phrase, "intent": row.intent})).collect();
    Ok(with_view("phrases", json!({ "phrases": phrases })))
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key).and_then(Value::as_str).filter(|item| !item.is_empty()).ok_or_else(|| format!("{key} required"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_klar_sets_view() {
        let architecture = explain_klar(&json!({})).unwrap();
        assert_eq!(architecture["view"], "architecture");
        let setup = explain_klar(&json!({"topic":"setup"})).unwrap();
        assert_eq!(setup["view"], "guide");
        assert!(setup["steps"].as_array().unwrap().len() >= 4);
    }

    #[tokio::test]
    async fn try_sentence_returns_path_view() {
        let dir = std::env::temp_dir().join(format!("klar-lotse-read-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let state = crate::io::state::AppState::new(
            crate::home::LoadedHome {
                graph: crate::home::default_home(),
                settings: crate::types::Settings::pinned("de"),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        );
        let out = try_sentence(&state, &json!({"text":"licht wohnzimmer an"})).await.unwrap();
        assert_eq!(out["view"], "path");
        assert!(out["speech"].as_str().unwrap_or("").len() > 0 || out.get("policy_trace").is_some());
    }
}
