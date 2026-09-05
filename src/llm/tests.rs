use super::{chat, chat_stream, ChatMessage, ChatRequest, LlmEndpoint};
use axum::routing::post;
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
                    assert_eq!(body["messages"][0]["role"], "user");
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
