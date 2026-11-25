//! Engine and operator-chrome settings Lotse may change. Never the LLM endpoint.

use crate::home::overlay::load_overlay;
use crate::io::state::{save_failed, AppState};
use crate::io::trainer_reads::with_view;
use crate::lang::{check_selection, pin_language, LangId};
use crate::types::{checked_news_feeds, Mode, NewsFeed, Personality, RefineBand, Settings, UnitSystem, VoiceTraits};
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
    "refine_bands",
    "calendar_llm",
    "quiet_ack",
    "satellite_followup",
    "allow_llm_tools",
    "extra_prompt",
    "unit_system",
    "custom_voice",
    "custom_voice_name",
    "custom_voice_seed",
    "custom_voice_traits",
    "news_feeds",
];

/// Fields the Home Assistant integration may PATCH onto `/api/settings`.
const HTTP_EXTRA_KEYS: &[&str] = &["time_zone", "default_language", "experimental_languages", "fallback_llm"];

const UI_KEYS: &[&str] = &["theme", "locale"];

pub fn engine_view(settings: &Settings, theme: &str, locale: &str, locale_set: bool, configured: bool) -> Value {
    with_view(
        "engine",
        json!({
            "languages": settings.languages,
            "personality": settings.personality,
            "mode": settings.mode,
            "refine_speech": settings.refine_speech,
            "refine_bands": settings.refine_bands,
            "nlu_rag": settings.nlu_rag,
            "calendar_llm": settings.calendar_llm,
            "quiet_ack": settings.quiet_ack,
            "satellite_followup": settings.satellite_followup,
            "allow_llm_tools": settings.allow_llm_tools,
            "confirm_risky_actions": settings.confirm_risky_actions,
            "semantic_adapters": settings.semantic_adapters,
            "support_bundle": settings.support_bundle,
            "support_bundle_raw_text": settings.support_bundle_raw_text,
            "extra_prompt": settings.extra_prompt,
            "unit_system": settings.unit_system,
            "custom_voice": settings.custom_voice,
            "custom_voice_name": settings.custom_voice_name,
            "custom_voice_seed": settings.custom_voice_seed,
            "custom_voice_traits": settings.custom_voice_traits,
            "news_feeds": settings.news_feeds,
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
    let ui = state
        .overlay
        .update(|overlay| {
            overlay.settings = Some(next.clone());
            overlay.ui.clone()
        })
        .map_err(save_failed)
        .map_err(|_| "save overlay")?;
    *state.settings.lock().await = next.clone();
    let configured = state.llm.lock().await.is_some();
    Ok(engine_view(&next, &ui.theme, &ui.locale, ui.locale_set, configured))
}

pub async fn apply_ui(state: &AppState, args: &Value) -> Result<Value, String> {
    let ui = state
        .overlay
        .try_update(|overlay| {
            overlay.ui = patch_ui(&overlay.ui, args)?;
            Ok::<_, String>(overlay.ui.clone())
        })
        .map_err(save_failed)
        .map_err(|_| "save overlay")??;
    let settings = state.settings.lock().await.clone();
    let configured = state.llm.lock().await.is_some();
    Ok(engine_view(&settings, &ui.theme, &ui.locale, ui.locale_set, configured))
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
        "refine_bands": settings.refine_bands,
        "nlu_rag": settings.nlu_rag,
        "calendar_llm": settings.calendar_llm,
        "quiet_ack": settings.quiet_ack,
        "satellite_followup": settings.satellite_followup,
        "allow_llm_tools": settings.allow_llm_tools,
        "confirm_risky_actions": settings.confirm_risky_actions,
        "semantic_adapters": settings.semantic_adapters,
        "support_bundle": settings.support_bundle,
        "support_bundle_raw_text": settings.support_bundle_raw_text,
        "extra_prompt": settings.extra_prompt,
        "unit_system": settings.unit_system,
        "custom_voice": settings.custom_voice,
        "custom_voice_name": settings.custom_voice_name,
        "custom_voice_seed": settings.custom_voice_seed,
        "custom_voice_traits": settings.custom_voice_traits,
        "news_feeds": settings.news_feeds
    })
}

/// Merge a partial settings document for HTTP PATCH (D-08). Validates languages,
/// timezone, and string lengths (D-10).
pub(crate) fn patch_settings_http(current: &Settings, args: &Value) -> Result<Settings, String> {
    let mut allowed: Vec<&str> = ENGINE_KEYS.to_vec();
    allowed.extend_from_slice(HTTP_EXTRA_KEYS);
    reject_unknown(args, &allowed)?;
    let mut next = patch_engine_fields(current, args)?;
    if let Some(text) = args.get("time_zone").and_then(Value::as_str) {
        next.time_zone = checked_time_zone(text)?;
    }
    if let Some(text) = args.get("default_language").and_then(Value::as_str) {
        if text.chars().count() > 32 {
            return Err("default_language too long".into());
        }
        next.default_language = text.to_string();
    }
    if let Some(flag) = args.get("experimental_languages").and_then(Value::as_bool) {
        next.experimental_languages = flag;
    }
    if let Some(flag) = args.get("fallback_llm").and_then(Value::as_bool) {
        next.fallback_llm = flag;
    }
    crate::lang::check_settings(next, current)
}

pub(crate) fn checked_time_zone(raw: &str) -> Result<String, String> {
    let name = raw.trim();
    if name.is_empty() {
        return Ok(String::new());
    }
    if name.chars().count() > 64 {
        return Err("time_zone too long".into());
    }
    jiff::tz::TimeZone::get(name).map_err(|_| format!("unknown time_zone: {name}"))?;
    Ok(name.to_string())
}

fn patch_engine(current: &Settings, args: &Value) -> Result<Settings, String> {
    reject_unknown(args, ENGINE_KEYS)?;
    patch_engine_fields(current, args)
}

fn patch_engine_fields(current: &Settings, args: &Value) -> Result<Settings, String> {
    let mut next = current.clone();
    if let Some(value) = args.get("personality") {
        next.personality = serde_json::from_value::<Personality>(value.clone()).map_err(|_| "unknown personality")?;
    }
    if let Some(value) = args.get("mode") {
        next.mode = serde_json::from_value::<Mode>(value.clone()).map_err(|_| "unknown mode")?;
    }
    if let Some(value) = args.get("languages") {
        next.languages = parse_languages(value, current)?;
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
        if flag && next.refine_bands.is_empty() && args.get("refine_bands").is_none() {
            next.refine_bands = vec![RefineBand::Status];
        }
    }
    if let Some(value) = args.get("refine_bands") {
        next.refine_bands = serde_json::from_value(value.clone()).map_err(|_| "unknown refine_bands")?;
    }
    if let Some(flag) = args.get("calendar_llm").and_then(Value::as_bool) {
        next.calendar_llm = flag;
    }
    if let Some(flag) = args.get("quiet_ack").and_then(Value::as_bool) {
        next.quiet_ack = flag;
    }
    if let Some(flag) = args.get("satellite_followup").and_then(Value::as_bool) {
        next.satellite_followup = flag;
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
    if let Some(text) = args.get("custom_voice_name").and_then(Value::as_str) {
        if text.chars().count() > 64 {
            return Err("custom_voice_name too long".into());
        }
        next.custom_voice_name = text.to_string();
    }
    if let Some(text) = args.get("custom_voice_seed").and_then(Value::as_str) {
        if text.chars().count() > 500 {
            return Err("custom_voice_seed too long".into());
        }
        next.custom_voice_seed = text.to_string();
    }
    if let Some(value) = args.get("custom_voice_traits") {
        next.custom_voice_traits = serde_json::from_value::<VoiceTraits>(value.clone()).map_err(|_| "bad custom_voice_traits")?.clamp();
    }
    if let Some(value) = args.get("news_feeds") {
        let feeds = serde_json::from_value::<Vec<NewsFeed>>(value.clone()).map_err(|_| "bad news_feeds")?;
        next.news_feeds = checked_news_feeds(feeds)?;
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

/// A language the house already uses may stay; an experimental one cannot be added
/// while `experimental_languages` is off.
fn parse_languages(value: &Value, current: &Settings) -> Result<Vec<String>, String> {
    let rows = value.as_array().ok_or("languages must be an array")?;
    if rows.len() > 80 {
        return Err("too many languages".into());
    }
    let wanted =
        rows.iter().map(|row| row.as_str().map(str::to_string).ok_or("language must be a string")).collect::<Result<Vec<_>, _>>()?;
    check_selection(&wanted, &current.languages, current.experimental_languages)
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
        let key = key.as_str();
        if allowed.contains(&key) {
            continue;
        }
        if FORBIDDEN.contains(&key) {
            return Err(format!("{key} stays in Settings → LLM"));
        }
        return Err(format!("unknown setting {key}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::test_dir::TestDir;
    use crate::home::{default_home, LoadedHome};
    use crate::io::state::AppState;

    fn state() -> (TestDir, AppState) {
        let scratch = TestDir::new("lotse-settings");
        let dir = scratch.path().to_path_buf();
        let state = AppState::new(
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
        );
        (scratch, state)
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
        assert_eq!(next.refine_bands, vec![RefineBand::Status]);
        assert_eq!(next.personality, Personality::Jarvis);
        assert!(next.quiet_ack);
        let bands = patch_engine(&next, &json!({"refine_bands":["status","command"]})).unwrap();
        assert_eq!(bands.refine_bands, vec![RefineBand::Status, RefineBand::Command]);
        assert_eq!(next.languages, vec!["de"]);
        let imperial = patch_engine(&set, &json!({"unit_system":"imperial"})).unwrap();
        assert_eq!(imperial.unit_system, UnitSystem::Imperial);
        let custom = patch_engine(
            &set,
            &json!({"personality":"custom","custom_voice":"Voice: dry.","custom_voice_name":"Spock","custom_voice_seed":"You are Spock."}),
        )
        .unwrap();
        assert_eq!(custom.personality, Personality::Custom);
        assert_eq!(custom.custom_voice, "Voice: dry.");
        assert_eq!(custom.custom_voice_name, "Spock");
        assert_eq!(custom.custom_voice_seed, "You are Spock.");
    }

    #[test]
    fn nested_http_patches_keep_each_other() {
        // D-08 / acceptance: two nested patches both survive.
        let set = Settings::pinned("de");
        let first = patch_settings_http(&set, &json!({"quiet_ack": true, "extra_prompt": "Sprich kurz."})).unwrap();
        assert!(first.quiet_ack);
        assert_eq!(first.extra_prompt, "Sprich kurz.");
        let second = patch_settings_http(&first, &json!({"personality": "butler"})).unwrap();
        assert!(second.quiet_ack);
        assert_eq!(second.extra_prompt, "Sprich kurz.");
        assert_eq!(second.personality, Personality::Butler);
        assert!(patch_settings_http(&set, &json!({"time_zone": "Not/A_Zone"})).is_err());
        assert_eq!(patch_settings_http(&set, &json!({"time_zone": "Europe/Berlin"})).unwrap().time_zone, "Europe/Berlin");
    }

    #[tokio::test]
    async fn persist_theme_and_engine() {
        let (_scratch, state) = state();
        apply_ui(&state, &json!({"theme":"light","locale":"fr"})).await.unwrap();
        apply_engine(&state, &json!({"calendar_llm":true})).await.unwrap();
        let view = list_engine(&state).await.unwrap();
        assert_eq!(view["theme"], "light");
        assert_eq!(view["locale"], "fr");
        assert_eq!(view["calendar_llm"], true);
        assert!(preview_engine(&Settings::pinned("de"), &json!({"token":"x"})).is_err());
    }
}
