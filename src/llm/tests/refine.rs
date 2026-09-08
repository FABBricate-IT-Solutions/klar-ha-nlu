use super::harness::{live_deltas, serve};
use crate::llm::{refine, refine_on, LlmEndpoint, RefineRequest};

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
            custom_voice: String::new(),
            conversation_id: String::new(),
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
            custom_voice: String::new(),
            conversation_id: String::new(),
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
async fn refine_on_forwards_deltas_before_done() {
    let (base, handle) = serve(vec!["Das Licht ".into(), "im Wohnzimmer ist an.".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let mut live = Vec::new();
    let out = refine_on(
        &endpoint,
        RefineRequest {
            speech: "Wohnzimmer Licht ist an.".into(),
            language: "de".into(),
            personality: "default".into(),
            extra_prompt: String::new(),
            custom_voice: String::new(),
            conversation_id: String::new(),
            stream: Some(true),
        },
        |event| live.push(event.clone()),
    )
    .await
    .unwrap();
    assert!(out.accepted);
    assert_eq!(out.text, "Das Licht im Wohnzimmer ist an.");
    assert_eq!(live_deltas(&live), ["Das Licht ", "im Wohnzimmer ist an."]);
    handle.abort();
}

#[tokio::test]
async fn refine_on_holds_rejected_prefix() {
    let (base, handle) = serve(vec!["Tomorrow will be sunny.".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let mut live = Vec::new();
    let out = refine_on(
        &endpoint,
        RefineRequest {
            speech: "Nothing tomorrow.".into(),
            language: "en".into(),
            personality: "default".into(),
            extra_prompt: String::new(),
            custom_voice: String::new(),
            conversation_id: String::new(),
            stream: Some(true),
        },
        |event| live.push(event.clone()),
    )
    .await
    .unwrap();
    assert!(!out.accepted);
    assert_eq!(out.text, "Nothing tomorrow.");
    assert!(live_deltas(&live).is_empty());
    handle.abort();
}
