use super::harness::serve_models;
use crate::llm::{list_models, LlmEndpoint};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use tokio::net::TcpListener;

#[tokio::test]
async fn lists_openai_model_ids_sorted() {
    let (base, handle) = serve_models(json!({"data":[{"id":"gpt-4o-mini"},{"id":"llama3"},{"name":"skip-me"}]})).await;
    let endpoint = LlmEndpoint::for_discovery(&base, "sk-test").unwrap();
    let models = list_models(&endpoint).await.unwrap();
    assert_eq!(models, vec!["gpt-4o-mini", "llama3", "skip-me"]);
    handle.abort();
}

#[tokio::test]
async fn lists_ollama_name_rows() {
    let (base, handle) = serve_models(json!({"models":[{"name":"llama3:latest"},{"id":"qwen2.5"}]})).await;
    let endpoint = LlmEndpoint::for_discovery(&base, "").unwrap();
    let models = list_models(&endpoint).await.unwrap();
    assert_eq!(models, vec!["llama3:latest", "qwen2.5"]);
    handle.abort();
}

#[test]
fn lemonade_v1_also_tries_api_v1() {
    let urls = super::super::client::model_list_urls("http://192.168.178.15:8000/v1");
    assert_eq!(urls, vec!["http://192.168.178.15:8000/v1/models".to_string(), "http://192.168.178.15:8000/api/v1/models".to_string()]);
}

#[tokio::test]
async fn lists_models_from_lemonade_api_v1_fallback() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let app = Router::new().route("/api/v1/models", get(|| async { Json(json!({"data":[{"id":"Qwen3-0.6B-GGUF"}]})) }));
        axum::serve(listener, app).await.unwrap();
    });
    let endpoint = LlmEndpoint::for_discovery(&format!("http://{addr}/v1"), "").unwrap();
    let models = list_models(&endpoint).await.unwrap();
    assert_eq!(models, vec!["Qwen3-0.6B-GGUF"]);
    handle.abort();
}

#[tokio::test]
async fn drops_empty_and_control_model_ids() {
    let (base, handle) = serve_models(json!({"data":[{"id":""},{"id":"ok"},{"id":"bad\nid"}]})).await;
    let endpoint = LlmEndpoint::for_discovery(&base, "").unwrap();
    let models = list_models(&endpoint).await.unwrap();
    assert_eq!(models, vec!["ok"]);
    handle.abort();
}

#[tokio::test]
async fn lemonade_lists_chat_models_only() {
    let body = json!({
        "data": [
            {"id": "Qwen3-Chat", "labels": ["chat", "tool-calling"]},
            {"id": "Flux-Image", "labels": ["image"]},
            {"id": "ACE-Music", "labels": ["audio-generation"]}
        ]
    });
    let (base, handle) = serve_models(body).await;
    let endpoint = LlmEndpoint::for_discovery(&base, "").unwrap();
    let models = list_models(&endpoint).await.unwrap();
    assert_eq!(models, vec!["Qwen3-Chat"]);
    handle.abort();
}
