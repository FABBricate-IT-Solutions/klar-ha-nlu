//! A stand-in for an OpenAI-compatible server on a loopback port chosen at run time. It
//! answers the model list and one chat turn and remembers the credentials it was sent.
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

pub struct FakeLlm {
    pub base_url: String,
    seen: Arc<Mutex<Vec<Option<String>>>>,
    task: tokio::task::JoinHandle<()>,
}

/// The credential of one request: the bearer token or the `x-api-key` header.
fn credential(headers: &HeaderMap) -> Option<String> {
    ["authorization", "x-api-key"].iter().find_map(|name| headers.get(*name).and_then(|value| value.to_str().ok()).map(str::to_string))
}

impl FakeLlm {
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind port 0");
        let addr = listener.local_addr().expect("local addr");
        let seen: Arc<Mutex<Vec<Option<String>>>> = Arc::default();
        let (models_seen, chat_seen) = (seen.clone(), seen.clone());
        let app = Router::new()
            .route(
                "/v1/models",
                get(move |headers: HeaderMap| async move {
                    models_seen.lock().unwrap().push(credential(&headers));
                    Json(json!({"data": [{"id": "kleines-modell"}]}))
                }),
            )
            .route(
                "/v1/chat/completions",
                post(move |headers: HeaderMap, Json(_): Json<Value>| async move {
                    chat_seen.lock().unwrap().push(credential(&headers));
                    Json(json!({"choices": [{"message": {"role": "assistant", "content": "moin"}}]}))
                }),
            );
        let task = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        Self { base_url: format!("http://{addr}/v1"), seen, task }
    }

    /// The credential of every request so far, in order. `None` is a request without one.
    pub fn credentials(&self) -> Vec<Option<String>> {
        self.seen.lock().unwrap().clone()
    }
}

impl Drop for FakeLlm {
    fn drop(&mut self) {
        self.task.abort();
    }
}
