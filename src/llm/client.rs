use super::endpoint::{LlmEndpoint, LlmProviderKind};
use super::sse::{delta_text, delta_tool_calls, SseBuf};
use super::types::{
    ChatRequest, CompletionTurn, LlmError, SanitizedChat, ToolCallAssembler, UpstreamChat, UpstreamCompletion, UpstreamMessage,
};
use futures_util::StreamExt;
use serde_json::Value;
use std::collections::BTreeSet;
use std::time::Duration;

const MAX_MODELS: usize = 500;
const MAX_MODEL_ID: usize = 256;

#[derive(Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    endpoint: LlmEndpoint,
}

impl LlmClient {
    pub fn new(endpoint: LlmEndpoint) -> Result<Self, LlmError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| LlmError::Transport)?;
        Ok(Self { http, endpoint })
    }
}

pub async fn chat(endpoint: &LlmEndpoint, request: ChatRequest) -> Result<String, LlmError> {
    let turn = chat_turn(endpoint, request).await?;
    if turn.text.is_empty() && turn.tool_calls.is_empty() {
        Err(LlmError::Response)
    } else {
        Ok(turn.text)
    }
}

pub async fn chat_turn(endpoint: &LlmEndpoint, request: ChatRequest) -> Result<CompletionTurn, LlmError> {
    LlmClient::new(endpoint.clone())?.complete_turn(request).await
}

pub async fn chat_stream<F>(endpoint: &LlmEndpoint, request: ChatRequest, mut on_delta: F) -> Result<String, LlmError>
where
    F: FnMut(&str),
{
    let turn = chat_stream_turn(endpoint, request, &mut on_delta).await?;
    if turn.text.is_empty() && turn.tool_calls.is_empty() {
        Err(LlmError::Response)
    } else {
        Ok(turn.text)
    }
}

pub async fn chat_stream_turn<F>(endpoint: &LlmEndpoint, request: ChatRequest, mut on_delta: F) -> Result<CompletionTurn, LlmError>
where
    F: FnMut(&str),
{
    LlmClient::new(endpoint.clone())?.stream_turn(request, &mut on_delta).await
}

pub async fn list_models(endpoint: &LlmEndpoint) -> Result<Vec<String>, LlmError> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(8))
        .build()
        .map_err(|_| LlmError::Transport)?;
    let mut last_err = LlmError::Response;
    let mut saw_empty = false;
    for url in model_list_urls(&endpoint.base_url) {
        match list_models_at(&http, endpoint, &url).await {
            Ok(ids) if !ids.is_empty() => return Ok(ids),
            Ok(_) => saw_empty = true,
            Err(err) => last_err = err,
        }
    }
    if saw_empty {
        Ok(Vec::new())
    } else {
        Err(last_err)
    }
}

pub(crate) fn model_list_urls(base_url: &str) -> Vec<String> {
    let base = base_url.trim_end_matches('/');
    let mut urls = vec![format!("{base}/models")];
    if let Some(root) = base.strip_suffix("/api/v1") {
        let alt = format!("{root}/v1/models");
        if !urls.contains(&alt) {
            urls.push(alt);
        }
    } else if let Some(root) = base.strip_suffix("/v1") {
        let alt = format!("{root}/api/v1/models");
        if !urls.contains(&alt) {
            urls.push(alt);
        }
    }
    urls
}

async fn list_models_at(http: &reqwest::Client, endpoint: &LlmEndpoint, url: &str) -> Result<Vec<String>, LlmError> {
    let req = apply_provider_headers(http.get(url), endpoint);
    let response = req.send().await.map_err(LlmError::from)?;
    let status = response.status();
    if !status.is_success() {
        return Err(LlmError::Upstream(status.as_u16()));
    }
    let parsed: Value = response.json().await.map_err(|_| LlmError::Response)?;
    Ok(parse_model_ids(&parsed))
}

fn parse_model_ids(value: &Value) -> Vec<String> {
    let rows =
        value.as_array().or_else(|| value.get("data").and_then(Value::as_array)).or_else(|| value.get("models").and_then(Value::as_array));
    let Some(rows) = rows else {
        return Vec::new();
    };
    let mut parsed = Vec::new();
    for row in rows {
        let raw = row
            .as_str()
            .map(str::to_string)
            .or_else(|| ["id", "name", "model"].iter().find_map(|key| row.get(*key).and_then(Value::as_str).map(str::to_string)));
        if let Some(id) = raw.and_then(|text| sanitize_model_id(&text)) {
            parsed.push((id, row_labels(row)));
        }
    }
    let chat: Vec<String> =
        parsed.iter().filter(|(_, labels)| labels.iter().any(|label| label == "chat")).map(|(id, _)| id.clone()).collect();
    let ids = if chat.is_empty() { parsed.into_iter().map(|(id, _)| id).collect() } else { chat };
    let mut unique = BTreeSet::new();
    for id in ids {
        if unique.len() >= MAX_MODELS {
            break;
        }
        unique.insert(id);
    }
    unique.into_iter().collect()
}

fn row_labels(row: &Value) -> Vec<String> {
    row.get("labels")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).map(str::to_ascii_lowercase).collect())
        .unwrap_or_default()
}

fn sanitize_model_id(raw: &str) -> Option<String> {
    let id = raw.trim();
    if id.is_empty() || id.len() > MAX_MODEL_ID || id.chars().any(char::is_control) {
        None
    } else {
        Some(id.to_string())
    }
}

impl LlmClient {
    pub async fn complete(&self, request: ChatRequest) -> Result<String, LlmError> {
        let turn = self.complete_turn(request).await?;
        if turn.text.is_empty() && turn.tool_calls.is_empty() {
            Err(LlmError::Response)
        } else {
            Ok(turn.text)
        }
    }

    pub async fn complete_turn(&self, request: ChatRequest) -> Result<CompletionTurn, LlmError> {
        let body = request.sanitize()?;
        let response = self.send(&body, false).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(LlmError::Upstream(status.as_u16()));
        }
        let parsed: UpstreamCompletion = response.json().await.map_err(|_| LlmError::Response)?;
        let message = parsed.choices.first().and_then(|choice| choice.message.as_ref());
        let text = message.map(UpstreamMessage::text).unwrap_or_default();
        let tool_calls = message.map(UpstreamMessage::tool_calls).unwrap_or_default();
        if text.is_empty() && tool_calls.is_empty() {
            Err(LlmError::Response)
        } else {
            Ok(CompletionTurn { text, tool_calls })
        }
    }

    pub async fn stream<F>(&self, request: ChatRequest, on_delta: &mut F) -> Result<String, LlmError>
    where
        F: FnMut(&str),
    {
        let turn = self.stream_turn(request, on_delta).await?;
        if turn.text.is_empty() && turn.tool_calls.is_empty() {
            Err(LlmError::Response)
        } else {
            Ok(turn.text)
        }
    }

    pub async fn stream_turn<F>(&self, request: ChatRequest, on_delta: &mut F) -> Result<CompletionTurn, LlmError>
    where
        F: FnMut(&str),
    {
        let body = request.sanitize()?;
        let response = self.send(&body, true).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(LlmError::Upstream(status.as_u16()));
        }
        let mut buf = SseBuf::default();
        let mut out = String::new();
        let mut assembler = ToolCallAssembler::default();
        let mut bytes = response.bytes_stream();
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(|_| LlmError::Transport)?;
            let text = String::from_utf8_lossy(&chunk);
            for data in buf.push(&text) {
                if data == "[DONE]" {
                    return Ok(CompletionTurn { text: out, tool_calls: assembler.finish() });
                }
                if let Some(delta) = delta_text(&data) {
                    out.push_str(&delta);
                    on_delta(&delta);
                }
                if let Some(calls) = delta_tool_calls(&data) {
                    assembler.push(&calls);
                }
            }
        }
        let tool_calls = assembler.finish();
        if out.is_empty() && tool_calls.is_empty() {
            Err(LlmError::Response)
        } else {
            Ok(CompletionTurn { text: out, tool_calls })
        }
    }

    async fn send(&self, body: &SanitizedChat, stream: bool) -> Result<reqwest::Response, LlmError> {
        let extra = self.endpoint.thinking_extras();
        let req = apply_provider_headers(
            self.http.post(self.endpoint.chat_url()).json(&UpstreamChat {
                model: &self.endpoint.model,
                messages: &body.messages,
                stream,
                temperature: body.temperature,
                max_tokens: body.max_tokens,
                tools: body.tools.as_deref(),
                tool_choice: body.tool_choice.as_ref(),
                chat_template_kwargs: extra.chat_template_kwargs,
                reasoning_effort: extra.reasoning_effort,
                thinking: extra.thinking,
                extra_body: extra.extra_body,
            }),
            &self.endpoint,
        );
        req.send().await.map_err(LlmError::from)
    }
}

fn apply_provider_headers(mut req: reqwest::RequestBuilder, endpoint: &LlmEndpoint) -> reqwest::RequestBuilder {
    if endpoint.api_key.is_empty() {
        return req;
    }
    match endpoint.provider {
        LlmProviderKind::Anthropic => {
            req = req.header("x-api-key", &endpoint.api_key).header("anthropic-version", "2023-06-01");
            req
        }
        _ => req.bearer_auth(&endpoint.api_key),
    }
}
