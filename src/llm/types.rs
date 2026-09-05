use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_MESSAGES: usize = 32;
pub const MAX_MESSAGE_CHARS: usize = 32_768;
pub const MAX_TOKENS_LIMIT: u32 = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    Delta {
        text: String,
    },
    Done {
        text: String,
    },
    Error {
        message: String,
    },
    Proposal {
        value: serde_json::Value,
    },
    Validate {
        value: serde_json::Value,
    },
    Tool {
        tool: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        intent: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        slots: Option<std::collections::BTreeMap<String, String>>,
    },
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("llm is not configured")]
    NotConfigured,
    #[error("invalid llm {0}")]
    InvalidEndpoint(&'static str),
    #[error("invalid chat request: {0}")]
    InvalidRequest(&'static str),
    #[error("llm upstream {0}")]
    Upstream(u16),
    #[error("llm timeout")]
    Timeout,
    #[error("llm transport")]
    Transport,
    #[error("llm response")]
    Response,
}

impl ChatRequest {
    pub fn sanitize(self) -> Result<SanitizedChat, LlmError> {
        if self.messages.is_empty() || self.messages.len() > MAX_MESSAGES {
            return Err(LlmError::InvalidRequest("messages"));
        }
        let mut messages = Vec::with_capacity(self.messages.len());
        for message in self.messages {
            let role = message.role.trim();
            if !matches!(role, "system" | "user" | "assistant") {
                return Err(LlmError::InvalidRequest("role"));
            }
            if message.content.is_empty() || message.content.chars().count() > MAX_MESSAGE_CHARS {
                return Err(LlmError::InvalidRequest("content"));
            }
            if message.content.chars().any(|ch| ch.is_control() && ch != '\n' && ch != '\t') {
                return Err(LlmError::InvalidRequest("content"));
            }
            messages.push(ChatMessage { role: role.to_string(), content: message.content });
        }
        let temperature = self.temperature.unwrap_or(0.2);
        if !(0.0..=2.0).contains(&temperature) {
            return Err(LlmError::InvalidRequest("temperature"));
        }
        let max_tokens = self.max_tokens.unwrap_or(2048).clamp(1, MAX_TOKENS_LIMIT);
        Ok(SanitizedChat { messages, stream: self.stream.unwrap_or(true), temperature, max_tokens })
    }
}

#[derive(Debug, Clone)]
pub struct SanitizedChat {
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct UpstreamChat<'a> {
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    pub stream: bool,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct UpstreamCompletion {
    pub choices: Vec<UpstreamChoice>,
}

#[derive(Debug, Deserialize)]
pub struct UpstreamChoice {
    #[serde(default)]
    pub message: Option<UpstreamMessage>,
    #[serde(default)]
    pub delta: Option<UpstreamMessage>,
}

#[derive(Debug, Deserialize)]
pub struct UpstreamMessage {
    #[serde(default)]
    pub content: Option<serde_json::Value>,
}

impl UpstreamMessage {
    pub fn text(&self) -> String {
        match &self.content {
            Some(serde_json::Value::String(text)) => text.clone(),
            Some(serde_json::Value::Array(parts)) => {
                parts.iter().filter_map(|part| part.get("text").and_then(|text| text.as_str())).collect()
            }
            _ => String::new(),
        }
    }
}

impl From<reqwest::Error> for LlmError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else if err.is_status() {
            Self::Upstream(err.status().map(|code| code.as_u16()).unwrap_or(502))
        } else {
            Self::Transport
        }
    }
}
