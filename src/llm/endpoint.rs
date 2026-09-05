use super::types::LlmError;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE: &str = "https://api.openai.com/v1";

#[derive(Clone)]
pub struct LlmEndpoint {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmPublic {
    pub configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl std::fmt::Debug for LlmEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmEndpoint")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &if self.api_key.is_empty() { "" } else { "***" })
            .finish()
    }
}

impl LlmEndpoint {
    pub fn from_env() -> Option<Self> {
        let model = std::env::var("KLAR_LLM_MODEL").ok()?.trim().to_string();
        if model.is_empty() {
            return None;
        }
        Self::from_parts(
            std::env::var("KLAR_LLM_BASE_URL").ok().as_deref().unwrap_or(DEFAULT_BASE),
            std::env::var("KLAR_LLM_API_KEY").ok().as_deref().unwrap_or(""),
            &model,
        )
        .ok()
    }

    pub fn from_parts(base_url: &str, api_key: &str, model: &str) -> Result<Self, LlmError> {
        let model = model.trim();
        if model.is_empty() || model.len() > 128 || model.chars().any(char::is_control) {
            return Err(LlmError::InvalidEndpoint("model"));
        }
        Ok(Self { base_url: normalize_base(base_url)?, api_key: sanitize_key(api_key)?, model: model.to_string() })
    }

    /// List models without persisting a chat model. Never used by `nlu::parse`.
    pub fn for_discovery(base_url: &str, api_key: &str) -> Result<Self, LlmError> {
        Ok(Self { base_url: normalize_base(base_url)?, api_key: sanitize_key(api_key)?, model: String::new() })
    }

    pub fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    pub fn models_url(&self) -> String {
        format!("{}/models", self.base_url)
    }

    pub fn public(&self) -> LlmPublic {
        LlmPublic { configured: true, base_url: Some(self.base_url.clone()), model: Some(self.model.clone()) }
    }
}

impl LlmPublic {
    pub fn empty() -> Self {
        Self { configured: false, base_url: None, model: None }
    }
}

fn sanitize_key(api_key: &str) -> Result<String, LlmError> {
    if api_key.len() > 512 || api_key.chars().any(char::is_control) {
        return Err(LlmError::InvalidEndpoint("api_key"));
    }
    Ok(api_key.to_string())
}

fn normalize_base(raw: &str) -> Result<String, LlmError> {
    let trimmed = raw.trim();
    let base = if trimmed.is_empty() { DEFAULT_BASE } else { trimmed };
    if base.len() > 512 {
        return Err(LlmError::InvalidEndpoint("base_url"));
    }
    let no_slash = base.trim_end_matches('/');
    if !(no_slash.starts_with("http://") || no_slash.starts_with("https://")) {
        return Err(LlmError::InvalidEndpoint("base_url"));
    }
    if no_slash.contains('@') || no_slash.contains(char::is_whitespace) {
        return Err(LlmError::InvalidEndpoint("base_url"));
    }
    let path = no_slash.splitn(4, '/').nth(3).unwrap_or("");
    if path.is_empty() {
        Ok(format!("{no_slash}/v1"))
    } else {
        Ok(no_slash.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_host_gets_v1() {
        let ep = LlmEndpoint::from_parts("https://api.openai.com", "sk-test", "gpt-4o-mini").unwrap();
        assert_eq!(ep.base_url, "https://api.openai.com/v1");
        assert_eq!(ep.chat_url(), "https://api.openai.com/v1/chat/completions");
        assert_eq!(ep.models_url(), "https://api.openai.com/v1/models");
    }

    #[test]
    fn discovery_skips_model_name() {
        let ep = LlmEndpoint::for_discovery("http://127.0.0.1:11434/v1", "").unwrap();
        assert_eq!(ep.models_url(), "http://127.0.0.1:11434/v1/models");
        assert!(ep.model.is_empty());
    }

    #[test]
    fn ollama_keeps_v1_path() {
        let ep = LlmEndpoint::from_parts("http://192.168.1.8:11434/v1", "", "llama3").unwrap();
        assert_eq!(ep.base_url, "http://192.168.1.8:11434/v1");
    }

    #[test]
    fn rejects_userinfo_and_file() {
        assert!(LlmEndpoint::from_parts("https://u:p@api.openai.com/v1", "k", "m").is_err());
        assert!(LlmEndpoint::from_parts("file:///etc/passwd", "k", "m").is_err());
        assert!(LlmEndpoint::from_parts("https://api.openai.com/v1", "k", "").is_err());
    }

    #[test]
    fn debug_hides_key() {
        let ep = LlmEndpoint::from_parts("https://api.openai.com/v1", "sk-secret", "m").unwrap();
        let text = format!("{ep:?}");
        assert!(!text.contains("sk-secret"));
        assert!(text.contains("***"));
    }
}
