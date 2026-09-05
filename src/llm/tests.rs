use super::{assist, chat, chat_stream, list_models, refine, AssistRequest, ChatMessage, ChatRequest, LlmEndpoint, RefineRequest};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

async fn serve(chunks: Vec<String>, stream: bool) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move |Json(body): Json<Value>| {
                let chunks = chunks.clone();
                async move {
                    assert_eq!(body["model"], "test-model");
                    if stream {
                        let mut payload = String::new();
                        for chunk in &chunks {
                            payload.push_str("data: {\"choices\":[{\"delta\":{\"content\":\"");
                            payload.push_str(chunk);
                            payload.push_str("\"}}]}\n\n");
                        }
                        payload.push_str("data: [DONE]\n\n");
                        ([(axum::http::header::CONTENT_TYPE, "text/event-stream")], payload)
                    } else {
                        let text = chunks.concat();
                        let body = json!({"choices":[{"message":{"role":"assistant","content":text}}]}).to_string();
                        ([(axum::http::header::CONTENT_TYPE, "application/json")], body)
                    }
                }
            }),
        );
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}/v1"), handle)
}

fn request(text: &str) -> ChatRequest {
    ChatRequest {
        messages: vec![ChatMessage { role: "user".into(), content: text.into() }],
        stream: None,
        temperature: Some(0.0),
        max_tokens: Some(32),
    }
}

#[tokio::test]
async fn streams_openai_chunks() {
    let (base, handle) = serve(vec!["Hel".into(), "lo".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "sk-test", "test-model").unwrap();
    let mut seen = String::new();
    let text = chat_stream(&endpoint, request("hi"), |delta| seen.push_str(delta)).await.unwrap();
    assert_eq!(text, "Hello");
    assert_eq!(seen, "Hello");
    handle.abort();
}

#[tokio::test]
async fn completes_without_stream() {
    let (base, handle) = serve(vec!["done".into()], false).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let text = chat(&endpoint, request("hi")).await.unwrap();
    assert_eq!(text, "done");
    handle.abort();
}

#[tokio::test]
async fn refine_accepts_safe_rewrite() {
    let (base, handle) = serve(vec!["Das Licht im Wohnzimmer ist an.".into()], false).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let out = refine(
        &endpoint,
        RefineRequest {
            speech: "Wohnzimmer Licht ist an.".into(),
            language: "de".into(),
            personality: "default".into(),
            extra_prompt: String::new(),
            stream: Some(false),
        },
    )
    .await
    .unwrap();
    assert!(out.accepted);
    assert_eq!(out.text, "Das Licht im Wohnzimmer ist an.");
    handle.abort();
}

#[tokio::test]
async fn refine_returns_original_when_accept_rejects() {
    let (base, handle) = serve(vec!["Tomorrow will be sunny.".into()], false).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let out = refine(
        &endpoint,
        RefineRequest {
            speech: "Nothing tomorrow.".into(),
            language: "en".into(),
            personality: "default".into(),
            extra_prompt: String::new(),
            stream: Some(false),
        },
    )
    .await
    .unwrap();
    assert!(!out.accepted);
    assert_eq!(out.text, "Nothing tomorrow.");
    handle.abort();
}

#[tokio::test]
async fn assist_emits_structured_parse_tool() {
    let (base, handle) = serve(vec!["KLAR_PARSE: licht an".into()], false).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let out = assist(
        &endpoint,
        AssistRequest {
            text: "mach das licht an".into(),
            language: "de".into(),
            personality: "default".into(),
            kind: "rag".into(),
            allow_tools: false,
            nlu_rag: true,
            retrieval: None,
            facts: None,
            history: vec![],
            extra_system: None,
            extra_prompt: None,
            stream: Some(false),
        },
    )
    .await
    .unwrap();
    assert!(out.tool.is_some());
    assert_eq!(out.tool.as_ref().unwrap().tool, "klar.parse");
    let json = serde_json::to_value(&out.events[0]).unwrap();
    assert_eq!(json["type"], "tool");
    assert_eq!(json["text"], "licht an");
    handle.abort();
}

#[tokio::test]
async fn assist_returns_canned_yarn_when_model_asks() {
    let (base, handle) = serve(vec!["Soll ich dir eine Geschichte erzählen?".into()], false).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let out = assist(
        &endpoint,
        AssistRequest {
            text: "erzähl eine Geschichte".into(),
            language: "de".into(),
            personality: "default".into(),
            kind: "yarn".into(),
            allow_tools: false,
            nlu_rag: false,
            retrieval: None,
            facts: None,
            history: vec![],
            extra_system: None,
            extra_prompt: None,
            stream: Some(false),
        },
    )
    .await
    .unwrap();
    assert!(out.text.contains("Fuchs"));
    assert!(!out.text.contains("Soll ich"));
    handle.abort();
}

async fn serve_models(body: Value) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let app = Router::new().route(
            "/v1/models",
            get(move || {
                let body = body.clone();
                async move { Json(body) }
            }),
        );
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}/v1"), handle)
}

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

#[tokio::test]
async fn drops_empty_and_control_model_ids() {
    let (base, handle) = serve_models(json!({"data":[{"id":""},{"id":"ok"},{"id":"bad\nid"}]})).await;
    let endpoint = LlmEndpoint::for_discovery(&base, "").unwrap();
    let models = list_models(&endpoint).await.unwrap();
    assert_eq!(models, vec!["ok"]);
    handle.abort();
}
