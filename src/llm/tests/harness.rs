use crate::llm::{AssistRequest, ChatEvent, ChatMessage, ChatRequest};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

pub(super) async fn serve(chunks: Vec<String>, stream: bool) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move |Json(body): Json<Value>| {
                let chunks = chunks.clone();
                async move {
                    assert_eq!(body["model"], "test-model");
                    assert_eq!(body["chat_template_kwargs"]["enable_thinking"], false);
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

pub(super) fn chat_assist_req() -> AssistRequest {
    AssistRequest {
        text: "hi there".into(),
        language: "en".into(),
        personality: "default".into(),
        kind: "chat".into(),
        allow_tools: false,
        nlu_rag: false,
        retrieval: None,
        facts: None,
        history: vec![],
        extra_system: None,
        extra_prompt: None,
        custom_voice: None,
        conversation_id: String::new(),
        stream: Some(true),
        tools: None,
        tool_messages: vec![],
    }
}

pub(super) fn request(text: &str) -> ChatRequest {
    ChatRequest {
        messages: vec![ChatMessage::new("user", text)],
        stream: None,
        temperature: Some(0.0),
        max_tokens: Some(32),
        tools: None,
        tool_choice: None,
    }
}

pub(super) async fn serve_models(body: Value) -> (String, tokio::task::JoinHandle<()>) {
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

pub(super) fn yarn_assist_req(text: &str) -> AssistRequest {
    AssistRequest {
        text: text.into(),
        language: "de".into(),
        personality: "default".into(),
        kind: "auto".into(),
        allow_tools: false,
        nlu_rag: false,
        retrieval: None,
        facts: None,
        history: vec![],
        extra_system: None,
        extra_prompt: None,
        custom_voice: None,
        conversation_id: String::new(),
        stream: Some(true),
        tools: None,
        tool_messages: vec![],
    }
}

pub(super) fn rag_assist_req(text: &str, stream: bool) -> AssistRequest {
    AssistRequest {
        text: text.into(),
        language: "de".into(),
        personality: "default".into(),
        kind: "auto".into(),
        allow_tools: false,
        nlu_rag: true,
        retrieval: None,
        facts: None,
        history: vec![],
        extra_system: None,
        extra_prompt: None,
        custom_voice: None,
        conversation_id: String::new(),
        stream: Some(stream),
        tools: None,
        tool_messages: vec![],
    }
}

pub(super) fn live_deltas(events: &[ChatEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|event| match event {
            ChatEvent::Delta { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}
