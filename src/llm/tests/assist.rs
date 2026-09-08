use super::harness::{chat_assist_req, live_deltas, rag_assist_req, serve, yarn_assist_req};
use crate::llm::{assist, assist_on, AssistRequest, ChatEvent, LlmEndpoint};

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
            custom_voice: None,
            conversation_id: String::new(),
            stream: Some(false),
            tools: None,
            tool_messages: vec![],
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
            custom_voice: None,
            conversation_id: String::new(),
            stream: Some(false),
            tools: None,
            tool_messages: vec![],
        },
    )
    .await
    .unwrap();
    assert!(out.text.contains("Fuchs"));
    assert!(!out.text.contains("Soll ich"));
    handle.abort();
}

#[tokio::test]
async fn assist_streams_chat_deltas() {
    let (base, handle) = serve(vec!["Hel".into(), "lo".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let out = assist(&endpoint, chat_assist_req()).await.unwrap();
    assert_eq!(out.text, "Hello");
    let deltas: Vec<&str> = out
        .events
        .iter()
        .filter_map(|event| match event {
            ChatEvent::Delta { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(deltas, ["Hel", "lo"]);
    handle.abort();
}

#[tokio::test]
async fn assist_on_forwards_deltas_before_return() {
    let (base, handle) = serve(vec!["Hel".into(), "lo".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let mut live = Vec::new();
    let out = assist_on(&endpoint, chat_assist_req(), |event| live.push(event.clone())).await.unwrap();
    assert_eq!(out.text, "Hello");
    let live_deltas: Vec<&str> = live
        .iter()
        .filter_map(|event| match event {
            ChatEvent::Delta { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(live_deltas, ["Hel", "lo"]);
    assert!(matches!(live.last(), Some(ChatEvent::Done { text }) if text == "Hello"));
    handle.abort();
}

#[tokio::test]
async fn assist_on_streams_yarn_story_and_joke() {
    let (base, handle) = serve(vec!["Es war ".into(), "ein Fuchs.".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let mut live = Vec::new();
    let out = assist_on(&endpoint, yarn_assist_req("erzähl eine lange Geschichte"), |event| live.push(event.clone())).await.unwrap();
    assert_eq!(out.text, "Es war ein Fuchs.");
    assert_eq!(live_deltas(&live), ["Es war ", "ein Fuchs."]);
    handle.abort();

    let (base, handle) = serve(vec!["Warum ".into(), "Geister?".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let mut live = Vec::new();
    let out = assist_on(&endpoint, yarn_assist_req("erzähl einen Witz"), |event| live.push(event.clone())).await.unwrap();
    assert_eq!(out.text, "Warum Geister?");
    assert_eq!(live_deltas(&live), ["Warum ", "Geister?"]);
    handle.abort();
}

#[tokio::test]
async fn assist_on_streams_world_knowledge_with_nlu_rag() {
    let (base, handle) = serve(vec!["Die Hauptstadt ".into(), "ist Paris.".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let mut live = Vec::new();
    let out = assist_on(&endpoint, rag_assist_req("Was ist die Hauptstadt von Frankreich", true), |event| live.push(event.clone()))
        .await
        .unwrap();
    assert_eq!(out.text, "Die Hauptstadt ist Paris.");
    assert_eq!(live_deltas(&live), ["Die Hauptstadt ", "ist Paris."]);
    assert!(out.tool.is_none());
    handle.abort();
}

#[tokio::test]
async fn assist_on_holds_rag_tool_prefix() {
    let (base, handle) = serve(vec!["KLAR_".into(), "PARSE: licht an".into()], true).await;
    let endpoint = LlmEndpoint::from_parts(&base, "", "test-model").unwrap();
    let mut live = Vec::new();
    let out = assist_on(&endpoint, rag_assist_req("mach das licht an", true), |event| live.push(event.clone())).await.unwrap();
    assert!(out.tool.is_some());
    assert!(live_deltas(&live).is_empty());
    let json = serde_json::to_value(&out.events[0]).unwrap();
    assert_eq!(json["type"], "tool");
    assert_eq!(json["text"], "licht an");
    handle.abort();
}
