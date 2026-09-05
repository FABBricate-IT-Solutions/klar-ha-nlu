//! Assist product LLM: engine owns prompts, RAG protocol, and yarn canned replies.

use super::assist_prompt::{history_prompt, system_for, with_personality, AssistKind};
use super::assist_rag::{holds_klar_tool_prefix, parse_tool_reply, KlarTool};
use super::assist_yarn::{yarn_asks_permission, yarn_canned, yarn_nudge};
use super::client::chat;
use super::endpoint::LlmEndpoint;
use super::refine_prompt::refine_prompt;
use super::types::{ChatEvent, ChatMessage, ChatRequest, LlmError};
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
    pub stream: Option<bool>,
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
    #[allow(dead_code)]
    stream: bool,
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
        let voice = refine_prompt(&self.language, &self.personality, Some(self.extra_prompt.as_str()));
        with_personality(&base, &voice)
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
    let mut raw = complete(endpoint, &system, &asked).await?;
    if kind == AssistKind::Yarn && yarn_asks_permission(&raw) {
        let nudged = yarn_nudge(&body.language, &system);
        raw = complete(endpoint, &nudged, &asked).await.unwrap_or(raw);
        if yarn_asks_permission(&raw) {
            raw = yarn_canned(&body.language, &body.text);
        }
    }
    if kind == AssistKind::Rag || body.nlu_rag {
        if let Some(tool) = parse_tool_reply(&raw) {
            return Ok(AssistOutcome {
                text: tool.spoken_line(),
                tool: Some(tool.clone()),
                events: vec![tool.event(), ChatEvent::Done { text: String::new() }],
            });
        }
        if holds_klar_tool_prefix(raw.trim_start()) && parse_tool_reply(&raw).is_none() {
            // Incomplete prefix — do not speak it.
            raw = String::new();
        }
    }
    Ok(AssistOutcome { text: raw.clone(), tool: None, events: vec![ChatEvent::Delta { text: raw.clone() }, ChatEvent::Done { text: raw }] })
}

async fn complete(endpoint: &LlmEndpoint, system: &str, user: &str) -> Result<String, LlmError> {
    chat(
        endpoint,
        ChatRequest {
            messages: vec![
                ChatMessage { role: "system".into(), content: system.to_string() },
                ChatMessage { role: "user".into(), content: user.to_string() },
            ],
            stream: Some(false),
            temperature: Some(ASSIST_TEMPERATURE),
            max_tokens: Some(ASSIST_MAX_TOKENS),
        },
    )
    .await
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
            stream: Some(true),
        };
        let body = req.sanitize().unwrap();
        assert_eq!(body.resolved_kind(), AssistKind::Yarn);
        let system = body.system_prompt();
        assert!(system.contains("Witz"));
        assert!(system.contains("Stimme:") || system.contains("Voice:"));
        assert!(!system.contains("KLAR_PARSE:"));
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
            stream: None,
        };
        assert!(req.sanitize().is_err());
    }
}
