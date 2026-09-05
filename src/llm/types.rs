use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_MESSAGES: usize = 48;
pub const MAX_MESSAGE_CHARS: usize = 32_768;
pub const MAX_TOKENS_LIMIT: u32 = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ToolFn {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ToolCall {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub function: ToolFn,
}

impl ToolCall {
    pub fn function(id: impl Into<String>, name: impl Into<String>, arguments: impl Into<String>) -> Self {
        Self { id: id.into(), kind: "function".into(), function: ToolFn { name: name.into(), arguments: arguments.into() } }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
}

impl ChatMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: role.into(), content: content.into(), ..Self::default() }
    }

    pub fn assistant_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self { role: "assistant".into(), content: content.into(), tool_calls, ..Self::default() }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: "tool".into(), content: content.into(), tool_call_id: Some(tool_call_id.into()), ..Self::default() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompletionTurn {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ChatRequest {
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub tool_choice: Option<serde_json::Value>,
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
    Consent {
        call_id: String,
        tool: String,
        summary: String,
        validate: serde_json::Value,
    },
    Session {
        yolo: bool,
        allowed: Vec<String>,
    },
    ToolCall {
        id: String,
        name: String,
        arguments: String,
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
            messages.push(sanitize_message(message)?);
        }
        let temperature = self.temperature.unwrap_or(0.2);
        if !(0.0..=2.0).contains(&temperature) {
            return Err(LlmError::InvalidRequest("temperature"));
        }
        let max_tokens = self.max_tokens.unwrap_or(2048).clamp(1, MAX_TOKENS_LIMIT);
        Ok(SanitizedChat {
            messages,
            stream: self.stream.unwrap_or(true),
            temperature,
            max_tokens,
            tools: self.tools,
            tool_choice: self.tool_choice,
        })
    }
}

fn sanitize_message(message: ChatMessage) -> Result<ChatMessage, LlmError> {
    let role = message.role.trim();
    if !matches!(role, "system" | "user" | "assistant" | "tool") {
        return Err(LlmError::InvalidRequest("role"));
    }
    if message.content.chars().count() > MAX_MESSAGE_CHARS {
        return Err(LlmError::InvalidRequest("content"));
    }
    if message.content.chars().any(|ch| ch.is_control() && ch != '\n' && ch != '\t') {
        return Err(LlmError::InvalidRequest("content"));
    }
    match role {
        "tool" => {
            if message.tool_call_id.as_deref().unwrap_or("").is_empty() {
                return Err(LlmError::InvalidRequest("tool_call_id"));
            }
        }
        "assistant" => {
            if message.content.is_empty() && message.tool_calls.is_empty() {
                return Err(LlmError::InvalidRequest("content"));
            }
        }
        _ => {
            if message.content.is_empty() {
                return Err(LlmError::InvalidRequest("content"));
            }
        }
    }
    Ok(ChatMessage { role: role.to_string(), content: message.content, tool_call_id: message.tool_call_id, tool_calls: message.tool_calls })
}

#[derive(Debug, Clone)]
pub struct SanitizedChat {
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub temperature: f32,
    pub max_tokens: u32,
    pub tools: Option<Vec<serde_json::Value>>,
    pub tool_choice: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ChatTemplateKwargs {
    pub enable_thinking: bool,
}

#[derive(Debug, Serialize)]
pub struct UpstreamChat<'a> {
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    pub stream: bool,
    pub temperature: f32,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<&'a [serde_json::Value]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<&'a serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_template_kwargs: Option<ChatTemplateKwargs>,
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpstreamMessage {
    #[serde(default)]
    pub content: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_calls: Option<Vec<UpstreamToolCall>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpstreamToolCall {
    #[serde(default)]
    pub index: Option<usize>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub function: Option<UpstreamToolFn>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpstreamToolFn {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
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

    pub fn tool_calls(&self) -> Vec<ToolCall> {
        let Some(rows) = &self.tool_calls else {
            return Vec::new();
        };
        rows.iter()
            .filter_map(|row| {
                let function = row.function.as_ref()?;
                let name = function.name.as_deref().unwrap_or("").trim();
                if name.is_empty() {
                    return None;
                }
                Some(ToolCall::function(row.id.clone().unwrap_or_default(), name, function.arguments.clone().unwrap_or_default()))
            })
            .collect()
    }
}

#[derive(Default)]
pub struct ToolCallAssembler {
    slots: Vec<ToolCall>,
}

impl ToolCallAssembler {
    pub fn push(&mut self, rows: &[UpstreamToolCall]) {
        for row in rows {
            let index = row.index.unwrap_or(self.slots.len());
            if self.slots.len() <= index {
                self.slots.resize(index + 1, ToolCall::default());
            }
            let slot = &mut self.slots[index];
            if let Some(id) = row.id.as_deref().filter(|id| !id.is_empty()) {
                slot.id = id.to_string();
            }
            if let Some(kind) = row.kind.as_deref().filter(|kind| !kind.is_empty()) {
                slot.kind = kind.to_string();
            }
            if let Some(function) = &row.function {
                if let Some(name) = function.name.as_deref().filter(|name| !name.is_empty()) {
                    slot.function.name = name.to_string();
                }
                if let Some(arguments) = &function.arguments {
                    slot.function.arguments.push_str(arguments);
                }
            }
        }
    }

    pub fn finish(self) -> Vec<ToolCall> {
        self.slots
            .into_iter()
            .filter(|call| !call.function.name.is_empty())
            .map(|mut call| {
                if call.kind.is_empty() {
                    call.kind = "function".into();
                }
                if call.id.is_empty() {
                    call.id = format!("call_{}", call.function.name);
                }
                call
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assistant_tool_message_may_omit_text() {
        let req = ChatRequest {
            messages: vec![
                ChatMessage::new("user", "alias the ceiling"),
                ChatMessage::assistant_tools("", vec![ToolCall::function("c1", "apply_aliases", "{}")]),
                ChatMessage::tool("c1", "ok"),
            ],
            stream: Some(false),
            ..ChatRequest::default()
        };
        assert!(req.sanitize().is_ok());
    }

    #[test]
    fn assembles_chunked_tool_calls() {
        let mut assembler = ToolCallAssembler::default();
        assembler.push(&[UpstreamToolCall {
            index: Some(0),
            id: Some("call_1".into()),
            kind: Some("function".into()),
            function: Some(UpstreamToolFn { name: Some("get_entity".into()), arguments: Some("{\"e".into()) }),
        }]);
        assembler.push(&[UpstreamToolCall {
            index: Some(0),
            id: None,
            kind: None,
            function: Some(UpstreamToolFn { name: None, arguments: Some("ntity_id\":\"light.x\"}".into()) }),
        }]);
        let calls = assembler.finish();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_1");
        assert_eq!(calls[0].function.name, "get_entity");
        assert_eq!(calls[0].function.arguments, r#"{"entity_id":"light.x"}"#);
    }
}
