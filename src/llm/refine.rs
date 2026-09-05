//! Assist refine: engine builds the prompt, runs the model, applies accept.

use super::client::chat;
use super::endpoint::LlmEndpoint;
use super::refine_accept::accept_refined;
use super::refine_prompt::{refine_input, refine_prompt, usable_extra};
use super::types::{ChatMessage, ChatRequest, LlmError};
use serde::{Deserialize, Serialize};

pub const MAX_SPEECH_CHARS: usize = 4096;
pub const MAX_EXTRA_CHARS: usize = 2048;
pub const REFINE_TEMPERATURE: f32 = 0.65;
pub const REFINE_MAX_TOKENS: u32 = 192;

#[derive(Debug, Clone, Deserialize)]
pub struct RefineRequest {
    pub speech: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub extra_prompt: String,
    #[serde(default)]
    pub conversation_id: String,
    #[serde(default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RefineOutcome {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: String,
    pub accepted: bool,
}

impl RefineOutcome {
    pub fn done(text: String, accepted: bool) -> Self {
        Self { kind: "done", text, accepted }
    }
}

impl RefineRequest {
    pub fn sanitize(self) -> Result<SanitizedRefine, LlmError> {
        let speech = self.speech.trim();
        if speech.is_empty() || speech.chars().count() > MAX_SPEECH_CHARS {
            return Err(LlmError::InvalidRequest("speech"));
        }
        if speech.chars().any(|ch| ch.is_control() && ch != '\n' && ch != '\t') {
            return Err(LlmError::InvalidRequest("speech"));
        }
        if self.extra_prompt.chars().count() > MAX_EXTRA_CHARS {
            return Err(LlmError::InvalidRequest("extra_prompt"));
        }
        let language = self.language.trim();
        if language.is_empty() || language.chars().count() > 32 || language.chars().any(char::is_control) {
            return Err(LlmError::InvalidRequest("language"));
        }
        let personality = self.personality.trim();
        if personality.chars().count() > 32 || personality.chars().any(char::is_control) {
            return Err(LlmError::InvalidRequest("personality"));
        }
        Ok(SanitizedRefine {
            speech: speech.to_string(),
            language: language.to_string(),
            personality: personality.to_string(),
            extra_prompt: self.extra_prompt,
            stream: self.stream.unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SanitizedRefine {
    pub speech: String,
    pub language: String,
    pub personality: String,
    pub extra_prompt: String,
    pub stream: bool,
}

pub async fn refine(endpoint: &LlmEndpoint, request: RefineRequest) -> Result<RefineOutcome, LlmError> {
    let body = request.sanitize()?;
    let chat_req = ChatRequest {
        messages: refine_messages(&body),
        stream: Some(false),
        temperature: Some(REFINE_TEMPERATURE),
        max_tokens: Some(REFINE_MAX_TOKENS),
        tools: None,
        tool_choice: None,
    };
    let raw = chat(endpoint, chat_req).await?;
    match accept_refined(&body.speech, &raw) {
        Some(text) => Ok(RefineOutcome::done(text, true)),
        None => Ok(RefineOutcome::done(body.speech, false)),
    }
}

fn refine_messages(body: &SanitizedRefine) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::new("system", refine_prompt(&body.language, &body.personality))];
    if usable_extra(&body.extra_prompt, &body.language) {
        messages.push(ChatMessage::new("user", body.extra_prompt.trim()));
    }
    messages.push(ChatMessage::new("user", refine_input(&body.speech)));
    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_overlong_speech() {
        let req = RefineRequest {
            speech: "x".repeat(MAX_SPEECH_CHARS + 1),
            language: "de".into(),
            personality: "butler".into(),
            extra_prompt: String::new(),
            conversation_id: String::new(),
            stream: None,
        };
        assert!(req.sanitize().is_err());
    }

    #[test]
    fn accepts_typical_body() {
        let req = RefineRequest {
            speech: "Wohnzimmer Licht ist an.".into(),
            language: "de".into(),
            personality: "butler".into(),
            extra_prompt: String::new(),
            conversation_id: "c1".into(),
            stream: Some(false),
        };
        let body = req.sanitize().unwrap();
        assert!(!body.stream);
        assert_eq!(body.language, "de");
    }

    #[test]
    fn extra_is_user_not_system() {
        let body = RefineRequest {
            speech: "Wohnzimmer Licht ist an.".into(),
            language: "de".into(),
            personality: "jarvis".into(),
            extra_prompt: "Ein oder zwei Sätze.".into(),
            conversation_id: String::new(),
            stream: None,
        }
        .sanitize()
        .unwrap();
        let messages = refine_messages(&body);
        assert_eq!(messages[0].role, "system");
        assert!(messages[0].content.contains("Jarvis"));
        assert!(!messages[0].content.contains("Ein oder zwei Sätze."));
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "Ein oder zwei Sätze.");
        assert_eq!(messages[2].role, "user");
        assert_eq!(messages[2].content, "Wohnzimmer Licht ist an.");
    }
}
