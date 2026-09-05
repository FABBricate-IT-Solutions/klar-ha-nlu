use super::endpoint::LlmEndpoint;
use super::sse::{delta_text, SseBuf};
use super::types::{ChatRequest, LlmError, SanitizedChat, UpstreamChat, UpstreamCompletion};
use futures_util::StreamExt;
use std::time::Duration;

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
    LlmClient::new(endpoint.clone())?.complete(request).await
}

pub async fn chat_stream<F>(endpoint: &LlmEndpoint, request: ChatRequest, mut on_delta: F) -> Result<String, LlmError>
where
    F: FnMut(&str),
{
    LlmClient::new(endpoint.clone())?.stream(request, &mut on_delta).await
}

impl LlmClient {
    pub async fn complete(&self, request: ChatRequest) -> Result<String, LlmError> {
        let body = request.sanitize()?;
        let response = self.send(&body, false).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(LlmError::Upstream(status.as_u16()));
        }
        let parsed: UpstreamCompletion = response.json().await.map_err(|_| LlmError::Response)?;
        let text = parsed.choices.first().and_then(|choice| choice.message.as_ref()).map(|message| message.text()).unwrap_or_default();
        if text.is_empty() {
            Err(LlmError::Response)
        } else {
            Ok(text)
        }
    }

    pub async fn stream<F>(&self, request: ChatRequest, on_delta: &mut F) -> Result<String, LlmError>
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
        let mut bytes = response.bytes_stream();
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(|_| LlmError::Transport)?;
            let text = String::from_utf8_lossy(&chunk);
            for data in buf.push(&text) {
                if data == "[DONE]" {
                    return Ok(out);
                }
                if let Some(delta) = delta_text(&data) {
                    out.push_str(&delta);
                    on_delta(&delta);
                }
            }
        }
        if out.is_empty() {
            Err(LlmError::Response)
        } else {
            Ok(out)
        }
    }

    async fn send(&self, body: &SanitizedChat, stream: bool) -> Result<reqwest::Response, LlmError> {
        let mut req = self.http.post(self.endpoint.chat_url()).json(&UpstreamChat {
            model: &self.endpoint.model,
            messages: &body.messages,
            stream,
            temperature: body.temperature,
            max_tokens: body.max_tokens,
        });
        if !self.endpoint.api_key.is_empty() {
            req = req.bearer_auth(&self.endpoint.api_key);
        }
        req.send().await.map_err(LlmError::from)
    }
}
