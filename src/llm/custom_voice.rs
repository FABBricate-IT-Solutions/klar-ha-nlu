//! Build a custom refine voice from a seed character plus delivery sliders.

use super::client::chat;
use super::endpoint::LlmEndpoint;
use super::refine::MAX_EXTRA_CHARS;
use super::refine_prompt::language_lock;
use super::types::{ChatMessage, ChatRequest, LlmError};
use serde::{Deserialize, Serialize};

const SYSTEM: &str = "You write a Klar NLU voice block only. \
If a seed character is given, keep that identity. \
Sliders 0-10 only refine delivery (warmth, humor, sarcasm, formality, verbosity, energy). \
They must not replace or ignore the seed. \
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
    pub voice_name: String,
    #[serde(default)]
    pub seed: String,
    #[serde(default = "five")]
    pub warmth: u8,
    #[serde(default = "five")]
    pub humor: u8,
    #[serde(default)]
    pub sarcasm: u8,
    #[serde(default = "five")]
    pub formality: u8,
    #[serde(default = "five")]
    pub verbosity: u8,
    #[serde(default = "five")]
    pub energy: u8,
    #[serde(default)]
    pub taboo: String,
}

const fn five() -> u8 {
    5
}

fn clamp_trait(value: u8) -> u8 {
    value.min(10)
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
    pub voice_name: String,
    pub seed: String,
    pub warmth: u8,
    pub humor: u8,
    pub sarcasm: u8,
    pub formality: u8,
    pub verbosity: u8,
    pub energy: u8,
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
        let voice_name = self.voice_name.trim();
        if voice_name.chars().count() > 64 || voice_name.chars().any(char::is_control) {
            return Err(LlmError::InvalidRequest("voice_name"));
        }
        let seed = self.seed.trim();
        if seed.chars().count() > 500 || seed.chars().any(|ch| ch.is_control() && ch != '\n' && ch != '\t') {
            return Err(LlmError::InvalidRequest("seed"));
        }
        let taboo = self.taboo.trim();
        if taboo.chars().count() > 200 || taboo.chars().any(|ch| ch.is_control() && ch != '\n' && ch != '\t') {
            return Err(LlmError::InvalidRequest("taboo"));
        }
        Ok(SanitizedInterview {
            language: language.to_string(),
            address,
            name: name.to_string(),
            voice_name: voice_name.to_string(),
            seed: seed.to_string(),
            warmth: clamp_trait(self.warmth),
            humor: clamp_trait(self.humor),
            sarcasm: clamp_trait(self.sarcasm),
            formality: clamp_trait(self.formality),
            verbosity: clamp_trait(self.verbosity),
            energy: clamp_trait(self.energy),
            taboo: taboo.to_string(),
        })
    }
}

pub fn interview_user_line(body: &SanitizedInterview) -> String {
    let address = if body.address == "name" { format!("address by first name ({})", body.name) } else { body.address.to_string() };
    let mut line = format!(
        "Voice name: {}.\nLanguage pack: {}.\nAddress the operator: {}.\nTraits 0-10 (delivery only): warmth={}, humor={}, sarcasm={}, formality={}, verbosity={}, energy={}.",
        if body.voice_name.is_empty() { "custom" } else { &body.voice_name },
        body.language,
        address,
        body.warmth,
        body.humor,
        body.sarcasm,
        body.formality,
        body.verbosity,
        body.energy
    );
    if !body.seed.is_empty() {
        line.push_str("\nSeed character (keep this identity; sliders only refine delivery):\n");
        line.push_str(&body.seed);
    }
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

    fn sample() -> CustomVoiceRequest {
        CustomVoiceRequest {
            language: "de".into(),
            address: "name".into(),
            name: "Ines".into(),
            voice_name: "Spock".into(),
            seed: "You are Spock from Star Trek.".into(),
            warmth: 2,
            humor: 1,
            sarcasm: 3,
            formality: 9,
            verbosity: 4,
            energy: 3,
            taboo: "chef".into(),
        }
    }

    #[test]
    fn rejects_unknown_choices() {
        let mut bad = sample();
        bad.address = "hey".into();
        assert!(bad.sanitize().is_err());
    }

    #[test]
    fn formats_interview() {
        let line = interview_user_line(&sample().sanitize().unwrap());
        assert!(line.contains("Language pack: de."));
        assert!(line.contains("Ines"));
        assert!(line.contains("Voice name: Spock."));
        assert!(line.contains("You are Spock from Star Trek."));
        assert!(line.contains("sarcasm=3"));
        assert!(line.contains("Do not say: chef"));
    }

    #[test]
    fn sliders_are_independent_of_seed() {
        let line = interview_user_line(&sample().sanitize().unwrap());
        assert!(line.contains("Seed character"));
        assert!(line.contains("delivery only"));
    }

    #[test]
    fn empty_generated_prompt_fails() {
        assert!(clip_voice("   ").is_err());
        assert!(clip_voice(&"x".repeat(MAX_EXTRA_CHARS + 1)).is_err());
        assert_eq!(clip_voice("Voice: dry.").unwrap(), "Voice: dry.");
    }
}
