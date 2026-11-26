use super::harness::{request, serve};
use crate::llm::{chat, chat_stream, LlmEndpoint, LlmError};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[tokio::test]
async fn streams_openai_chunks() {
    let (base, handle) = serve(vec!["Hel".into(), "lo".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "sk-test", "test-model").unwrap();
    let mut seen = String::new();
    let text = chat_stream(&endpoint, request("hi"), |delta| {
        seen.push_str(delta);
        true
    })
    .await
    .unwrap();
    assert_eq!(text, "Hello");
    assert_eq!(seen, "Hello");
    handle.abort();
}

#[tokio::test]
async fn does_not_follow_redirect_to_other_host() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let hits = Arc::new(AtomicUsize::new(0));
    let victim_hits = hits.clone();
    let victim = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let victim_addr = victim.local_addr().unwrap();
    let victim_handle = tokio::spawn(async move {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move || {
                let hits = victim_hits.clone();
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    let body = json!({"choices":[{"message":{"role":"assistant","content":"leaked"}}]}).to_string();
                    ([(axum::http::header::CONTENT_TYPE, "application/json")], body)
                }
            }),
        );
        axum::serve(victim, app).await.unwrap();
    });

    let location = format!("http://{victim_addr}/v1/chat/completions");
    let redirector = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let redirector_addr = redirector.local_addr().unwrap();
    let redirect_handle = tokio::spawn(async move {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move || {
                let location = location.clone();
                async move { (axum::http::StatusCode::FOUND, [(axum::http::header::LOCATION, location)]) }
            }),
        );
        axum::serve(redirector, app).await.unwrap();
    });

    let endpoint = LlmEndpoint::from_parts(&format!("http://{redirector_addr}/v1"), "", "test-model").unwrap();
    let err = chat(&endpoint, request("hi")).await.unwrap_err();
    assert!(matches!(err, LlmError::Upstream(302)));
    assert_eq!(hits.load(Ordering::SeqCst), 0);
    victim_handle.abort();
    redirect_handle.abort();
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
async fn thinking_on_omits_template_kwargs() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move |Json(body): Json<Value>| async move {
                assert!(body.get("chat_template_kwargs").is_none());
                let payload = json!({"choices":[{"message":{"role":"assistant","content":"ok"}}]}).to_string();
                ([(axum::http::header::CONTENT_TYPE, "application/json")], payload)
            }),
        );
        axum::serve(listener, app).await.unwrap();
    });
    let base = format!("http://{addr}/v1");
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap().with_thinking(true);
    let text = chat(&endpoint, request("hi")).await.unwrap();
    assert_eq!(text, "ok");
    handle.abort();
}

#[tokio::test]
async fn stream_keeps_split_umlaut_intact() {
    use axum::body::{Body, Bytes};
    use axum::http::{header, StatusCode};
    use axum::response::Response;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move || async move {
                // Split the UTF-8 bytes of "ü" (c3 bc) across body chunks.
                let mut first = b"data: {\"choices\":[{\"delta\":{\"content\":\"".to_vec();
                first.push(0xc3);
                let mut second = vec![0xbc];
                second.extend_from_slice(b"\"}}]}\n\ndata: [DONE]\n\n");
                let chunks = vec![Ok::<_, std::io::Error>(Bytes::from(first)), Ok(Bytes::from(second))];
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/event-stream")
                    .body(Body::from_stream(tokio_stream::iter(chunks)))
                    .unwrap()
            }),
        );
        axum::serve(listener, app).await.unwrap();
    });
    let endpoint = LlmEndpoint::from_parts(&format!("http://{addr}/v1"), "", "test-model").unwrap();
    let text = chat_stream(&endpoint, request("hi"), |_| true).await.unwrap();
    assert_eq!(text, "ü");
    assert!(!text.contains('\u{FFFD}'));
    handle.abort();
}

#[tokio::test]
async fn stream_aborts_after_16_mib() {
    use crate::llm::sse::MAX_STREAM_BYTES;
    use axum::body::{Body, Bytes};
    use axum::http::{header, StatusCode};
    use axum::response::Response;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move || async move {
                let chunk = Bytes::from(vec![b'x'; 1024 * 1024]);
                let n = MAX_STREAM_BYTES / chunk.len() + 1;
                let chunks: Vec<_> = (0..n).map(|_| Ok::<_, std::io::Error>(chunk.clone())).collect();
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/event-stream")
                    .body(Body::from_stream(tokio_stream::iter(chunks)))
                    .unwrap()
            }),
        );
        axum::serve(listener, app).await.unwrap();
    });
    let endpoint = LlmEndpoint::from_parts(&format!("http://{addr}/v1"), "", "test-model").unwrap();
    let err = chat_stream(&endpoint, request("hi"), |_| true).await.unwrap_err();
    assert!(matches!(err, LlmError::TooLarge), "{err:?}");
    handle.abort();
}
