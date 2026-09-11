use super::*;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("klar-llm-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn persists_endpoint_and_keeps_key_off_public() {
    let dir = temp_dir("save");
    let endpoint = LlmEndpoint::from_parts("http://127.0.0.1:11434/v1", "sk-secret", "llama3").unwrap();
    save_endpoint(&dir, &endpoint).unwrap();
    let loaded = from_file(&dir).unwrap();
    assert_eq!(loaded.model, "llama3");
    assert_eq!(loaded.api_key, "sk-secret");
    assert_eq!(loaded.base_url, "http://127.0.0.1:11434/v1");
    let public = loaded.public();
    let json = serde_json::to_string(&public).unwrap();
    assert!(!json.contains("sk-secret"));
    assert!(json.contains("llama3"));
    assert!(!loaded.enable_thinking);
    assert!(!public.enable_thinking);
    assert_eq!(public.provider.as_deref(), Some("custom"));
    clear_endpoint(&dir).unwrap();
    assert!(from_file(&dir).is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unknown_provider_still_loads() {
    let dir = temp_dir("unknown-provider");
    std::fs::write(
        dir.join("llm_endpoint.json"),
        r#"{"base_url":"http://192.168.178.15:8000/v1","api_key":"k","model":"gemma","provider":"not-a-host"}"#,
    )
    .unwrap();
    let loaded = from_file(&dir).unwrap();
    assert_eq!(loaded.model, "gemma");
    assert_eq!(loaded.base_url, "http://192.168.178.15:8000/v1");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn old_file_defaults_thinking_off_and_roundtrips_on() {
    let dir = temp_dir("legacy");
    std::fs::write(dir.join("llm_endpoint.json"), r#"{"base_url":"http://127.0.0.1:8000/v1","api_key":"k","model":"gemma"}"#).unwrap();
    let loaded = from_file(&dir).unwrap();
    assert!(!loaded.enable_thinking);
    save_endpoint(&dir, &loaded.clone().with_thinking(true)).unwrap();
    let again = from_file(&dir).unwrap();
    assert!(again.enable_thinking);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn assist_sse_forwards_deltas_live() {
    let src = include_str!("llm.rs");
    let start = src.find("pub async fn llm_assist").expect("llm_assist");
    let end = src.find("pub fn json_event").expect("json_event");
    let body = &src[start..end];
    assert!(body.contains("assist_on"));
    assert!(body.contains("unbounded_channel"));
    assert!(!body.contains("blocking_send"));
    assert!(!body.contains("try_send"));
    assert!(!body.contains("for event in out.events"));
}

#[test]
fn custom_voice_route_requires_write_and_endpoint() {
    let src = include_str!("llm.rs");
    let start = src.find("async fn make_custom_voice").expect("make_custom_voice");
    let body = &src[start..start + 900];
    assert!(body.contains("writes_allowed"));
    assert!(body.contains("SERVICE_UNAVAILABLE"));
    assert!(body.contains("generate_custom_voice"));
}

#[test]
fn model_list_uses_read_gate() {
    let src = include_str!("llm.rs");
    let start = src.find("async fn list_endpoint_models").expect("list_endpoint_models");
    let end = src.find("pub async fn llm_chat").expect("llm_chat");
    let body = &src[start..end];
    assert!(body.contains("reads_allowed"));
    assert!(!body.contains("writes_allowed"));
}

#[test]
fn inference_routes_record_llm_calls() {
    let src = include_str!("llm.rs");
    assert!(src.contains("record_llm(&calls, \"chat\""));
    assert!(src.contains("record_llm(&calls, \"refine\""));
    assert!(src.contains("record_llm(&calls, \"assist\""));
    assert!(!src.contains("messages:"));
}

#[test]
fn clear_missing_file_is_ok() {
    let dir = temp_dir("missing");
    clear_endpoint(&dir).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}
