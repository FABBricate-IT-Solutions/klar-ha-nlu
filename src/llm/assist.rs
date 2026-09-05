//! Assist product LLM: engine owns prompts, RAG protocol, and yarn canned replies.

use super::assist_prompt::{history_prompt, system_for, with_personality, AssistKind};
use super::assist_rag::{holds_klar_tool_prefix, parse_tool_reply, KlarTool};
use super::assist_yarn::{yarn_asks_permission, yarn_canned, yarn_nudge};
use super::client::chat;
use super::endpoint::LlmEndpoint;
use super::refine_prompt::{refine_prompt, usable_extra};
use super::types::{ChatEvent, ChatMessage, ChatRequest, CompletionTurn, LlmError, ToolCall};
use serde::Deserialize;

pub const ASSIST_TEMPERATURE: f32 = 0.65;
pub const ASSIST_MAX_TOKENS: u32 = 768;
pub const MAX_TEXT_CHARS: usize = 4096;
pub const MAX_EXTRA_CHARS: usize = 2048;
pub const MAX_FACTS: usize = 16;
pub const MAX_HISTORY: usize = 8;

#[derive(Debug, Clone, Deserialize)]
pub struct AssistRequest {
    pub text: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub allow_tools: bool,
    #[serde(default)]
    pub nlu_rag: bool,
    #[serde(default)]
    pub retrieval: Option<serde_json::Value>,
    #[serde(default)]
    pub facts: Option<AssistFacts>,
    #[serde(default)]
    pub history: Vec<(String, String)>,
    #[serde(default)]
    pub extra_system: Option<String>,
    #[serde(default)]
    pub extra_prompt: Option<String>,
    #[serde(default)]
    pub conversation_id: String,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub tool_messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AssistFacts {
    Text(String),
    Lines(Vec<String>),
}

impl AssistFacts {
    fn lines(&self) -> Vec<String> {
        match self {
            Self::Text(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    Vec::new()
                } else {
                    vec![trimmed.to_string()]
                }
            }
            Self::Lines(lines) => {
                lines.iter().map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).take(MAX_FACTS).collect()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssistOutcome {
    pub text: String,
    pub tool: Option<KlarTool>,
    pub tool_calls: Vec<ToolCall>,
    pub events: Vec<ChatEvent>,
}

struct SanitizedAssist {
    text: String,
    language: String,
    personality: String,
    kind: AssistKind,
    allow_tools: bool,
    nlu_rag: bool,
    retrieval: Option<serde_json::Value>,
    facts: Vec<String>,
    history: Vec<(String, String)>,
    extra_system: String,
    extra_prompt: String,
    stream: bool,
    tools: Option<Vec<serde_json::Value>>,
    tool_messages: Vec<ChatMessage>,
}

impl AssistRequest {
    fn sanitize(self) -> Result<SanitizedAssist, LlmError> {
        let text = self.text.trim();
        if text.is_empty() || text.chars().count() > MAX_TEXT_CHARS {
            return Err(LlmError::InvalidRequest("text"));
        }
        if text.chars().any(|ch| ch.is_control() && ch != '\n' && ch != '\t') {
            return Err(LlmError::InvalidRequest("text"));
        }
        let language = self.language.trim();
        if language.is_empty() || language.chars().count() > 32 || language.chars().any(char::is_control) {
            return Err(LlmError::InvalidRequest("language"));
        }
        let personality = self.personality.trim();
        if personality.chars().count() > 32 || personality.chars().any(char::is_control) {
            return Err(LlmError::InvalidRequest("personality"));
        }
        let kind = AssistKind::parse(&self.kind).map_err(LlmError::InvalidRequest)?;
        let extra_system = self.extra_system.unwrap_or_default();
        if extra_system.chars().count() > MAX_EXTRA_CHARS {
            return Err(LlmError::InvalidRequest("extra_system"));
        }
        let extra_prompt = self.extra_prompt.unwrap_or_default();
        if extra_prompt.chars().count() > MAX_EXTRA_CHARS {
            return Err(LlmError::InvalidRequest("extra_prompt"));
        }
        let mut facts = self.facts.map(|facts| facts.lines()).unwrap_or_default();
        facts.truncate(MAX_FACTS);
        let mut history = self.history;
        history.truncate(MAX_HISTORY);
        Ok(SanitizedAssist {
            text: text.to_string(),
            language: language.to_string(),
            personality: personality.to_string(),
            kind,
            allow_tools: self.allow_tools,
            nlu_rag: self.nlu_rag,
            retrieval: self.retrieval,
            facts,
            history,
            extra_system,
            extra_prompt,
            stream: self.stream.unwrap_or(true),
            tools: if self.nlu_rag { None } else { self.tools },
            tool_messages: if self.nlu_rag { Vec::new() } else { self.tool_messages },
        })
    }
}

impl SanitizedAssist {
    fn resolved_kind(&self) -> AssistKind {
        self.kind.resolve(&self.text, self.nlu_rag)
    }

    fn system_prompt(&self) -> String {
        let kind = self.resolved_kind();
        let extra = if self.extra_system.trim().is_empty() { None } else { Some(self.extra_system.as_str()) };
        let mut base = system_for(&self.language, kind, &self.text, extra, self.allow_tools, self.retrieval.as_ref(), &self.facts);
        if kind != AssistKind::Yarn {
            let history = history_prompt(&self.language, &self.history);
            if !history.is_empty() {
                base = format!("{base}\n\n{history}");
            }
        }
        let voice = refine_prompt(&self.language, &self.personality);
        let mut prompt = with_personality(&base, &voice);
        if self.uses_ha_tools() {
            prompt = format!(
                "{prompt}\n\nKlar already parsed device commands. Tools are for live context and leftovers. Use tool names exactly as provided."
            );
        }
        prompt
    }

    fn uses_ha_tools(&self) -> bool {
        self.allow_tools && !self.nlu_rag && self.tools.as_ref().is_some_and(|tools| !tools.is_empty())
    }

    fn user_text(&self) -> String {
        if self.resolved_kind() == AssistKind::Calendar {
            super::assist_prompt::calendar_readback(&self.language, &self.facts.join("\n"))
        } else {
            self.text.clone()
        }
    }
}

pub async fn assist(endpoint: &LlmEndpoint, request: AssistRequest) -> Result<AssistOutcome, LlmError> {
    let body = request.sanitize()?;
    let kind = body.resolved_kind();
    let system = body.system_prompt();
    let asked = body.user_text();
    let stream = body.stream && kind != AssistKind::Yarn && kind != AssistKind::Rag && !body.nlu_rag;
    if stream {
        return complete_stream(endpoint, &body, &system, &asked).await;
    }
    let mut turn = complete_turn(endpoint, &body, &system, &asked).await?;
    if kind == AssistKind::Yarn && yarn_asks_permission(&turn.text) {
        let nudged = yarn_nudge(&body.language, &system);
        turn.text = complete(endpoint, chat_body(&body, &nudged, &asked, false)).await.unwrap_or(turn.text);
        if yarn_asks_permission(&turn.text) {
            turn.text = yarn_canned(&body.language, &body.text);
            turn.tool_calls.clear();
        }
    }
    if kind == AssistKind::Rag || body.nlu_rag {
        if let Some(tool) = parse_tool_reply(&turn.text) {
            return Ok(AssistOutcome {
                text: tool.spoken_line(),
                tool: Some(tool.clone()),
                tool_calls: Vec::new(),
                events: vec![tool.event(), ChatEvent::Done { text: String::new() }],
            });
        }
        if holds_klar_tool_prefix(turn.text.trim_start()) && parse_tool_reply(&turn.text).is_none() {
            turn.text = String::new();
        }
    }
    Ok(outcome_from_turn(turn, false))
}

fn chat_messages(body: &SanitizedAssist, system: &str, user: &str) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::new("system", system)];
    if usable_extra(&body.extra_prompt, &body.language) {
        messages.push(ChatMessage::new("user", body.extra_prompt.trim()));
    }
    messages.push(ChatMessage::new("user", user));
    messages.extend(body.tool_messages.iter().cloned());
    messages
}

fn chat_body(body: &SanitizedAssist, system: &str, user: &str, stream: bool) -> ChatRequest {
    ChatRequest {
        messages: chat_messages(body, system, user),
        stream: Some(stream),
        temperature: Some(ASSIST_TEMPERATURE),
        max_tokens: Some(ASSIST_MAX_TOKENS),
        tools: if body.uses_ha_tools() { body.tools.clone() } else { None },
        tool_choice: None,
    }
}

fn outcome_from_turn(turn: CompletionTurn, streamed: bool) -> AssistOutcome {
    let mut events = Vec::new();
    if !streamed && !turn.text.is_empty() {
        events.push(ChatEvent::Delta { text: turn.text.clone() });
    }
    for call in &turn.tool_calls {
        events.push(ChatEvent::ToolCall {
            id: call.id.clone(),
            name: call.function.name.clone(),
            arguments: call.function.arguments.clone(),
        });
    }
    events.push(ChatEvent::Done { text: turn.text.clone() });
    AssistOutcome { text: turn.text, tool: None, tool_calls: turn.tool_calls, events }
}

async fn complete(endpoint: &LlmEndpoint, request: ChatRequest) -> Result<String, LlmError> {
    chat(endpoint, request).await
}

async fn complete_turn(endpoint: &LlmEndpoint, body: &SanitizedAssist, system: &str, user: &str) -> Result<CompletionTurn, LlmError> {
    super::client::chat_turn(endpoint, chat_body(body, system, user, false)).await
}

async fn complete_stream(endpoint: &LlmEndpoint, body: &SanitizedAssist, system: &str, user: &str) -> Result<AssistOutcome, LlmError> {
    let mut events = Vec::new();
    let turn = super::client::chat_stream_turn(endpoint, chat_body(body, system, user, true), |delta| {
        if !delta.is_empty() {
            events.push(ChatEvent::Delta { text: delta.to_string() });
        }
    })
    .await?;
    for call in &turn.tool_calls {
        events.push(ChatEvent::ToolCall {
            id: call.id.clone(),
            name: call.function.name.clone(),
            arguments: call.function.arguments.clone(),
        });
    }
    events.push(ChatEvent::Done { text: turn.text.clone() });
    Ok(AssistOutcome { text: turn.text, tool: None, tool_calls: turn.tool_calls, events })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_and_resolves_yarn() {
        let req = AssistRequest {
            text: "erzähl einen Witz".into(),
            language: "de".into(),
            personality: "butler".into(),
            kind: "auto".into(),
            allow_tools: false,
            nlu_rag: false,
            retrieval: None,
            facts: None,
            history: vec![],
            extra_system: None,
            extra_prompt: None,
            conversation_id: String::new(),
            stream: Some(true),
            tools: None,
            tool_messages: vec![],
        };
        let body = req.sanitize().unwrap();
        assert_eq!(body.resolved_kind(), AssistKind::Yarn);
        let system = body.system_prompt();
        assert!(system.contains("Witz"));
        assert!(system.contains("Stimme:") || system.contains("Voice:"));
        assert!(!system.contains("KLAR_PARSE:"));
        assert!(!system.contains("Ein oder zwei Sätze."));
        let with_extra = AssistRequest { extra_prompt: Some("Ein oder zwei Sätze.".into()), ..req_yarn() }.sanitize().unwrap();
        assert!(!with_extra.system_prompt().contains("Ein oder zwei Sätze."));
        let messages = chat_messages(&with_extra, &with_extra.system_prompt(), "hi");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "Ein oder zwei Sätze.");
    }

    fn req_yarn() -> AssistRequest {
        AssistRequest {
            text: "erzähl einen Witz".into(),
            language: "de".into(),
            personality: "butler".into(),
            kind: "auto".into(),
            allow_tools: false,
            nlu_rag: false,
            retrieval: None,
            facts: None,
            history: vec![],
            extra_system: None,
            extra_prompt: None,
            conversation_id: String::new(),
            stream: Some(true),
            tools: None,
            tool_messages: vec![],
        }
    }

    #[test]
    fn rejects_unknown_kind() {
        let req = AssistRequest {
            text: "hi".into(),
            language: "en".into(),
            personality: "default".into(),
            kind: "nope".into(),
            allow_tools: false,
            nlu_rag: false,
            retrieval: None,
            facts: None,
            history: vec![],
            extra_system: None,
            extra_prompt: None,
            conversation_id: String::new(),
            stream: None,
            tools: None,
            tool_messages: vec![],
        };
        assert!(req.sanitize().is_err());
    }

    #[test]
    fn nlu_rag_drops_ha_tools() {
        let req = AssistRequest {
            text: "licht an".into(),
            language: "de".into(),
            personality: "default".into(),
            kind: "rag".into(),
            allow_tools: true,
            nlu_rag: true,
            retrieval: None,
            facts: None,
            history: vec![],
            extra_system: None,
            extra_prompt: None,
            conversation_id: String::new(),
            stream: Some(false),
            tools: Some(vec![serde_json::json!({"type":"function","function":{"name":"intent__HassTurnOn"}})]),
            tool_messages: vec![],
        };
        let body = req.sanitize().unwrap();
        assert!(body.tools.is_none());
        assert!(!body.uses_ha_tools());
        assert!(!body.system_prompt().contains("intent__HassTurnOn"));
        assert!(!body.system_prompt().contains("HassTurnOn"));
    }
}
