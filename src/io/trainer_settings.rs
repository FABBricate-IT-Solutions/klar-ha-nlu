//! Engine and operator-chrome settings Lotse may change. Never the LLM endpoint.

use crate::home::overlay::{load_overlay, save_overlay};
use crate::io::state::AppState;
use crate::io::trainer_reads::with_view;
use crate::lang::{pin_language, LangId};
use crate::types::{Mode, Personality, Settings, UnitSystem};
use serde_json::{json, Value};

const FORBIDDEN: &[&str] = &["url", "base_url", "token", "api_key", "model", "endpoint", "llm_url", "fallback_llm", "fallback_agent"];

const ENGINE_KEYS: &[&str] = &[
    "personality",
    "mode",
    "languages",
    "support_bundle",
    "support_bundle_raw_text",
    "confirm_risky_actions",
    "semantic_adapters",
    "nlu_rag",
    "refine_speech",
    "calendar_llm",
    "quiet_ack",
    "allow_llm_tools",
    "extra_prompt",
    "unit_system",
    "custom_voice",
];

const UI_KEYS: &[&str] = &["theme", "locale"];

pub fn engine_view(settings: &Settings, theme: &str, locale: &str, locale_set: bool, configured: bool) -> Value {
    with_view(
        "engine",
        json!({
            "languages": settings.languages,
            "personality": settings.personality,
            "mode": settings.mode,
            "refine_speech": settings.refine_speech,
            "nlu_rag": settings.nlu_rag,
            "calendar_llm": settings.calendar_llm,
            "quiet_ack": settings.quiet_ack,
            "allow_llm_tools": settings.allow_llm_tools,
            "confirm_risky_actions": settings.confirm_risky_actions,
            "semantic_adapters": settings.semantic_adapters,
            "support_bundle": settings.support_bundle,
            "support_bundle_raw_text": settings.support_bundle_raw_text,
            "extra_prompt": settings.extra_prompt,
            "unit_system": settings.unit_system,
            "custom_voice": settings.custom_voice,
            "theme": theme,
            "locale": locale,
            "locale_set": locale_set,
            "llm_configured": configured
        }),
    )
}

pub async fn list_engine(state: &AppState) -> Result<Value, String> {
    let settings = state.settings.lock().await.clone();
    let ui = load_overlay(&state.data_dir).ui;
    let configured = state.llm.lock().await.is_some();
    Ok(engine_view(&settings, &ui.theme, &ui.locale, ui.locale_set, configured))
}

pub async fn apply_engine(state: &AppState, args: &Value) -> Result<Value, String> {
    let next = patch_engine(&state.settings.lock().await.clone(), args)?;
    *state.settings.lock().await = next.clone();
    let mut overlay = load_overlay(&state.data_dir);
    overlay.settings = Some(next.clone());
    save_overlay(&state.data_dir, &overlay).map_err(|_| "save overlay")?;
    let ui = overlay.ui;
    let configured = state.llm.lock().await.is_some();
    Ok(engine_view(&next, &ui.theme, &ui.locale, ui.locale_set, configured))
}

pub async fn apply_ui(state: &AppState, args: &Value) -> Result<Value, String> {
    let mut overlay = load_overlay(&state.data_dir);
    overlay.ui = patch_ui(&overlay.ui, args)?;
    save_overlay(&state.data_dir, &overlay).map_err(|_| "save overlay")?;
    let settings = state.settings.lock().await.clone();
    let configured = state.llm.lock().await.is_some();
    Ok(engine_view(&settings, &overlay.ui.theme, &overlay.ui.locale, overlay.ui.locale_set, configured))
}

pub fn preview_engine(current: &Settings, args: &Value) -> Result<Value, String> {
    let next = patch_engine(current, args)?;
    Ok(json!({"ok": true, "errors": [], "warnings": [], "dry_run": [], "settings": public_settings(&next)}))
}

pub fn preview_ui(theme: &str, locale: &str, args: &Value) -> Result<Value, String> {
    let mut ui =
        crate::home::overlay::UiState { theme: theme.into(), locale: locale.into(), locale_set: !locale.is_empty(), ..Default::default() };
    ui = patch_ui(&ui, args)?;
    Ok(json!({"ok": true, "errors": [], "warnings": [], "dry_run": [], "theme": ui.theme, "locale": ui.locale}))
}

pub fn engine_summary(args: &Value) -> String {
    let keys: Vec<&str> =
        args.as_object().map(|obj| obj.keys().map(String::as_str).filter(|key| ENGINE_KEYS.contains(key)).collect()).unwrap_or_default();
    format!("engine {}", keys.join(" "))
}

pub fn ui_summary(args: &Value) -> String {
    format!(
        "ui theme={} locale={}",
        args.get("theme").and_then(Value::as_str).unwrap_or("—"),
        args.get("locale").and_then(Value::as_str).unwrap_or("—")
    )
}

fn public_settings(settings: &Settings) -> Value {
    json!({
        "languages": settings.languages,
        "personality": settings.personality,
        "mode": settings.mode,
        "refine_speech": settings.refine_speech,
        "nlu_rag": settings.nlu_rag,
        "calendar_llm": settings.calendar_llm,
        "quiet_ack": settings.quiet_ack,
        "allow_llm_tools": settings.allow_llm_tools,
        "confirm_risky_actions": settings.confirm_risky_actions,
        "semantic_adapters": settings.semantic_adapters,
        "support_bundle": settings.support_bundle,
        "support_bundle_raw_text": settings.support_bundle_raw_text,
        "extra_prompt": settings.extra_prompt,
        "unit_system": settings.unit_system,
        "custom_voice": settings.custom_voice
    })
}

fn patch_engine(current: &Settings, args: &Value) -> Result<Settings, String> {
    reject_unknown(args, ENGINE_KEYS)?;
    let mut next = current.clone();
    if let Some(value) = args.get("personality") {
        next.personality = serde_json::from_value::<Personality>(value.clone()).map_err(|_| "unknown personality")?;
    }
    if let Some(value) = args.get("mode") {
        next.mode = serde_json::from_value::<Mode>(value.clone()).map_err(|_| "unknown mode")?;
    }
    if let Some(value) = args.get("languages") {
        next.languages = parse_languages(value)?;
    }
    if let Some(flag) = args.get("support_bundle").and_then(Value::as_bool) {
        next.support_bundle = flag;
    }
    if let Some(flag) = args.get("support_bundle_raw_text").and_then(Value::as_bool) {
        next.support_bundle_raw_text = flag;
    }
    if let Some(flag) = args.get("confirm_risky_actions").and_then(Value::as_bool) {
        next.confirm_risky_actions = flag;
    }
    if let Some(flag) = args.get("semantic_adapters").and_then(Value::as_bool) {
        next.semantic_adapters = flag;
    }
    if let Some(flag) = args.get("nlu_rag").and_then(Value::as_bool) {
        next.nlu_rag = flag;
    }
    if let Some(flag) = args.get("refine_speech").and_then(Value::as_bool) {
        next.refine_speech = flag;
    }
    if let Some(flag) = args.get("calendar_llm").and_then(Value::as_bool) {
        next.calendar_llm = flag;
    }
    if let Some(flag) = args.get("quiet_ack").and_then(Value::as_bool) {
        next.quiet_ack = flag;
    }
    if let Some(flag) = args.get("allow_llm_tools").and_then(Value::as_bool) {
        next.allow_llm_tools = flag;
    }
    if let Some(text) = args.get("extra_prompt").and_then(Value::as_str) {
        if text.chars().count() > 2000 {
            return Err("extra_prompt too long".into());
        }
        next.extra_prompt = text.to_string();
    }
    if let Some(value) = args.get("unit_system") {
        next.unit_system = serde_json::from_value::<UnitSystem>(value.clone()).map_err(|_| "unknown unit_system")?;
    }
    if let Some(text) = args.get("custom_voice").and_then(Value::as_str) {
        if text.chars().count() > 2048 {
            return Err("custom_voice too long".into());
        }
        next.custom_voice = text.to_string();
    }
    Ok(next)
}

fn patch_ui(current: &crate::home::overlay::UiState, args: &Value) -> Result<crate::home::overlay::UiState, String> {
    reject_unknown(args, UI_KEYS)?;
    let mut next = current.clone();
    if let Some(theme) = args.get("theme").and_then(Value::as_str) {
        if theme != "dark" && theme != "light" {
            return Err("theme must be dark or light".into());
        }
        next.theme = theme.to_string();
    }
    if let Some(locale) = args.get("locale").and_then(Value::as_str).filter(|item| !item.is_empty()) {
        next.locale = resolve_locale(locale)?;
        next.locale_set = true;
    }
    Ok(next)
}

fn parse_languages(value: &Value) -> Result<Vec<String>, String> {
    let rows = value.as_array().ok_or("languages must be an array")?;
    if rows.len() > 80 {
        return Err("too many languages".into());
    }
    let mut out = Vec::new();
    for row in rows {
        let raw = row.as_str().ok_or("language must be a string")?;
        out.push(pin_language(raw).map_err(|_| format!("unknown language {raw}"))?);
    }
    Ok(out)
}

fn resolve_locale(raw: &str) -> Result<String, String> {
    if let Some(id) = LangId::from_code(raw) {
        return Ok(id.code().to_string());
    }
    pin_language(raw).map_err(|_| format!("unknown locale {raw}"))
}

fn reject_unknown(args: &Value, allowed: &[&str]) -> Result<(), String> {
    let obj = args.as_object().ok_or("object required")?;
    if obj.is_empty() {
        return Err("no settings to change".into());
    }
    for key in obj.keys() {
        if FORBIDDEN.contains(&key.as_str()) {
            return Err(format!("{key} stays in Settings → LLM"));
        }
        if !allowed.contains(&key.as_str()) {
            return Err(format!("unknown setting {key}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::io::state::AppState;

    fn state() -> AppState {
        let dir = std::env::temp_dir().join(format!("klar-lotse-settings-{}", std::process::id()));
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

    #[test]
    fn refuses_llm_endpoint_and_fallback() {
        let set = Settings::pinned("de");
        assert!(patch_engine(&set, &json!({"url":"http://x"})).is_err());
        assert!(patch_engine(&set, &json!({"model":"g"})).is_err());
        assert!(patch_engine(&set, &json!({"fallback_llm":true})).is_err());
        assert!(patch_engine(&set, &json!({})).is_err());
    }

    #[test]
    fn patches_refine_and_personality() {
        let set = Settings::pinned("de");
        let next = patch_engine(&set, &json!({"refine_speech":true,"personality":"jarvis","quiet_ack":true})).unwrap();
        assert!(next.refine_speech);
        assert_eq!(next.personality, Personality::Jarvis);
        assert!(next.quiet_ack);
        assert_eq!(next.languages, vec!["de"]);
        let imperial = patch_engine(&set, &json!({"unit_system":"imperial"})).unwrap();
        assert_eq!(imperial.unit_system, UnitSystem::Imperial);
        let custom = patch_engine(&set, &json!({"personality":"custom","custom_voice":"Voice: dry."})).unwrap();
        assert_eq!(custom.personality, Personality::Custom);
        assert_eq!(custom.custom_voice, "Voice: dry.");
    }

    #[tokio::test]
    async fn persist_theme_and_engine() {
        let state = state();
        apply_ui(&state, &json!({"theme":"light","locale":"fr"})).await.unwrap();
        apply_engine(&state, &json!({"calendar_llm":true})).await.unwrap();
        let view = list_engine(&state).await.unwrap();
        assert_eq!(view["theme"], "light");
        assert_eq!(view["locale"], "fr");
        assert_eq!(view["calendar_llm"], true);
        assert!(preview_engine(&Settings::pinned("de"), &json!({"token":"x"})).is_err());
    }
}
