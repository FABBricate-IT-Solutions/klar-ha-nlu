//! Build a custom refine voice from a short operator interview.

use super::client::chat;
use super::endpoint::LlmEndpoint;
use super::refine::MAX_EXTRA_CHARS;
use super::refine_prompt::language_lock;
use super::types::{ChatMessage, ChatRequest, LlmError};
use serde::{Deserialize, Serialize};

const SYSTEM: &str = "You write a Klar NLU voice block only. \
Output the voice description and a few short example rewrites (source → spoken). \
Keep language lock and safety: no Home Assistant tools, no device control, \
digits stay digits, no new facts. Do not repeat the safety rules. \
Do not mention this instruction. No markdown fences. No title.";

#[derive(Debug, Clone, Deserialize)]
pub struct CustomVoiceRequest {
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tone: String,
    #[serde(default)]
    pub humor: String,
    #[serde(default)]
    pub length: String,
    #[serde(default)]
    pub taboo: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CustomVoiceOut {
    pub prompt: String,
}

#[derive(Debug, Clone)]
pub struct SanitizedInterview {
    pub language: String,
    pub address: &'static str,
    pub name: String,
    pub tone: &'static str,
    pub humor: &'static str,
    pub length: &'static str,
    pub taboo: String,
}

impl CustomVoiceRequest {
    pub fn sanitize(self) -> Result<SanitizedInterview, LlmError> {
        let language = self.language.trim();
        if language.is_empty() || language.chars().count() > 32 || language.chars().any(char::is_control) {
            return Err(LlmError::InvalidRequest("language"));
        }
        let address = match self.address.trim() {
            "du" => "du",
            "sie" => "sie",
            "name" => "name",
            _ => return Err(LlmError::InvalidRequest("address")),
        };
        let name = self.name.trim();
        if name.chars().count() > 64 || name.chars().any(char::is_control) {
            return Err(LlmError::InvalidRequest("name"));
        }
        if address == "name" && name.is_empty() {
            return Err(LlmError::InvalidRequest("name"));
        }
        let tone = match self.tone.trim() {
            "short" => "short",
            "warm" => "warm",
            "dry" => "dry",
            _ => return Err(LlmError::InvalidRequest("tone")),
        };
        let humor = match self.humor.trim() {
            "none" => "none",
            "light" => "light",
            "sharp" => "sharp",
            _ => return Err(LlmError::InvalidRequest("humor")),
        };
        let length = match self.length.trim() {
            "one" => "one",
            "more" => "more",
            _ => return Err(LlmError::InvalidRequest("length")),
        };
        let taboo = self.taboo.trim();
        if taboo.chars().count() > 200 || taboo.chars().any(|ch| ch.is_control() && ch != '\n' && ch != '\t') {
            return Err(LlmError::InvalidRequest("taboo"));
        }
        Ok(SanitizedInterview {
            language: language.to_string(),
            address,
            name: name.to_string(),
            tone,
            humor,
            length,
            taboo: taboo.to_string(),
        })
    }
}

pub fn interview_user_line(body: &SanitizedInterview) -> String {
    let address = if body.address == "name" { format!("address by first name ({})", body.name) } else { body.address.to_string() };
    let mut line = format!(
        "Language pack: {}.\nAddress: {}.\nTone: {}.\nHumor: {}.\nLength: {}.",
        body.language, address, body.tone, body.humor, body.length
    );
    if !body.taboo.is_empty() {
        line.push_str("\nDo not say: ");
        line.push_str(&body.taboo);
    }
    line
}

pub fn clip_voice(raw: &str) -> Result<String, LlmError> {
    let prompt = raw.trim().to_string();
    if prompt.is_empty() || prompt.chars().count() > MAX_EXTRA_CHARS {
        return Err(LlmError::Response);
    }
    Ok(prompt)
}

pub async fn generate_custom_voice(endpoint: &LlmEndpoint, request: CustomVoiceRequest) -> Result<CustomVoiceOut, LlmError> {
    let body = request.sanitize()?;
    let lock = language_lock(&body.language);
    let chat_req = ChatRequest {
        messages: vec![ChatMessage::new("system", format!("{SYSTEM}\n\n{lock}")), ChatMessage::new("user", interview_user_line(&body))],
        stream: Some(false),
        temperature: Some(0.4),
        max_tokens: Some(512),
        tools: None,
        tool_choice: None,
    };
    let prompt = clip_voice(&chat(endpoint, chat_req).await?)?;
    Ok(CustomVoiceOut { prompt })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_choices() {
        let bad = CustomVoiceRequest {
            language: "de".into(),
            address: "hey".into(),
            name: String::new(),
            tone: "warm".into(),
            humor: "none".into(),
            length: "one".into(),
            taboo: String::new(),
        };
        assert!(bad.sanitize().is_err());
    }

    #[test]
    fn formats_interview() {
        let body = CustomVoiceRequest {
            language: "de".into(),
            address: "name".into(),
            name: "Ines".into(),
            tone: "dry".into(),
            humor: "light".into(),
            length: "one".into(),
            taboo: "chef".into(),
        }
        .sanitize()
        .unwrap();
        let line = interview_user_line(&body);
        assert!(line.contains("Language pack: de."));
        assert!(line.contains("Ines"));
        assert!(line.contains("Do not say: chef"));
    }

    #[test]
    fn empty_generated_prompt_fails() {
        assert!(clip_voice("   ").is_err());
        assert!(clip_voice(&"x".repeat(MAX_EXTRA_CHARS + 1)).is_err());
        assert_eq!(clip_voice("Voice: dry.").unwrap(), "Voice: dry.");
    }
}
