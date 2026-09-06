use super::types::{AnthropicThinking, ChatTemplateKwargs, LlmError, ThinkingExtras};
use serde::{Deserialize, Serialize};
use serde_json::json;

const DEFAULT_BASE: &str = "https://api.openai.com/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderKind {
    Openai,
    Anthropic,
    Google,
    Lemonade,
    Llamacpp,
    #[default]
    #[serde(other)]
    Custom,
}

#[derive(Clone)]
pub struct LlmEndpoint {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub enable_thinking: bool,
    pub provider: LlmProviderKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmPublic {
    pub configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub enable_thinking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl std::fmt::Debug for LlmEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmEndpoint")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("provider", &self.provider)
            .field("enable_thinking", &self.enable_thinking)
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
        .map(|endpoint| endpoint.with_thinking(env_thinking()))
    }

    pub fn from_parts(base_url: &str, api_key: &str, model: &str) -> Result<Self, LlmError> {
        let model = model.trim();
        if model.is_empty() || model.len() > 128 || model.chars().any(char::is_control) {
            return Err(LlmError::InvalidEndpoint("model"));
        }
        let base_url = normalize_base(base_url)?;
        let provider = LlmProviderKind::from_url(&base_url);
        Ok(Self { base_url, api_key: sanitize_key(api_key)?, model: model.to_string(), enable_thinking: false, provider })
    }

    pub fn with_thinking(mut self, enable_thinking: bool) -> Self {
        self.enable_thinking = enable_thinking;
        self
    }

    pub fn with_provider(mut self, provider: LlmProviderKind) -> Self {
        self.provider = provider;
        self
    }

    /// List models without persisting a chat model. Never used by `nlu::parse`.
    pub fn for_discovery(base_url: &str, api_key: &str) -> Result<Self, LlmError> {
        let base_url = normalize_base(base_url)?;
        let provider = LlmProviderKind::from_url(&base_url);
        Ok(Self { base_url, api_key: sanitize_key(api_key)?, model: String::new(), enable_thinking: false, provider })
    }

    pub fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    pub fn models_url(&self) -> String {
        format!("{}/models", self.base_url)
    }

    /// Gemma / Qwen / llama.cpp need `chat_template_kwargs`. Cloud APIs reject that field.
    pub fn thinking_extras(&self) -> ThinkingExtras {
        let kind = match self.provider {
            LlmProviderKind::Custom => LlmProviderKind::from_url(&self.base_url),
            other => other,
        };
        match kind {
            LlmProviderKind::Openai => openai_thinking(self.enable_thinking, &self.model),
            LlmProviderKind::Anthropic => anthropic_thinking(self.enable_thinking),
            LlmProviderKind::Google => google_thinking(self.enable_thinking),
            LlmProviderKind::Lemonade | LlmProviderKind::Llamacpp | LlmProviderKind::Custom => {
                local_template_thinking(self.enable_thinking)
            }
        }
    }

    pub fn chat_template_kwargs(&self) -> Option<ChatTemplateKwargs> {
        self.thinking_extras().chat_template_kwargs
    }

    pub fn public(&self) -> LlmPublic {
        LlmPublic {
            configured: true,
            base_url: Some(self.base_url.clone()),
            model: Some(self.model.clone()),
            enable_thinking: self.enable_thinking,
            provider: Some(self.provider.as_str().to_string()),
        }
    }
}

impl LlmProviderKind {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "openai" => Self::Openai,
            "anthropic" => Self::Anthropic,
            "google" => Self::Google,
            "lemonade" => Self::Lemonade,
            "llamacpp" => Self::Llamacpp,
            _ => Self::Custom,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::Lemonade => "lemonade",
            Self::Llamacpp => "llamacpp",
            Self::Custom => "custom",
        }
    }

    pub fn from_url(url: &str) -> Self {
        let raw = url.trim().trim_end_matches('/').to_ascii_lowercase();
        if raw.contains("anthropic.com") {
            Self::Anthropic
        } else if raw.contains("api.openai.com") {
            Self::Openai
        } else if raw.contains("googleapis.com") || raw.contains("generativelanguage") {
            Self::Google
        } else if raw.contains("lemonade")
            || raw.contains(":13305")
            || raw.contains("/api/v1")
            || (raw.contains(":8000") && raw.ends_with("/v1"))
        {
            Self::Lemonade
        } else if raw.contains(":8080") && raw.ends_with("/v1") {
            Self::Llamacpp
        } else {
            Self::Custom
        }
    }
}

impl LlmPublic {
    pub fn empty() -> Self {
        Self { configured: false, base_url: None, model: None, enable_thinking: false, provider: None }
    }
}

fn local_template_thinking(enable: bool) -> ThinkingExtras {
    ThinkingExtras { chat_template_kwargs: (!enable).then_some(ChatTemplateKwargs { enable_thinking: false }), ..ThinkingExtras::default() }
}

fn openai_reasoning_model(model: &str) -> bool {
    let raw = model.to_ascii_lowercase();
    raw.contains("o1") || raw.contains("o3") || raw.contains("o4") || raw.contains("gpt-5") || raw.contains("reasoning")
}

fn openai_thinking(enable: bool, model: &str) -> ThinkingExtras {
    if !openai_reasoning_model(model) {
        return ThinkingExtras::default();
    }
    ThinkingExtras { reasoning_effort: Some(if enable { "medium" } else { "none" }), ..ThinkingExtras::default() }
}

fn anthropic_thinking(enable: bool) -> ThinkingExtras {
    if enable {
        ThinkingExtras {
            thinking: Some(AnthropicThinking { kind: "enabled".into(), budget_tokens: Some(4096) }),
            ..ThinkingExtras::default()
        }
    } else {
        ThinkingExtras::default()
    }
}

fn google_thinking(enable: bool) -> ThinkingExtras {
    if enable {
        ThinkingExtras { reasoning_effort: Some("medium"), ..ThinkingExtras::default() }
    } else {
        ThinkingExtras {
            reasoning_effort: Some("none"),
            extra_body: Some(json!({ "google": { "thinking_config": { "thinking_budget": 0 } } })),
            ..ThinkingExtras::default()
        }
    }
}

fn env_thinking() -> bool {
    matches!(std::env::var("KLAR_LLM_ENABLE_THINKING").ok().as_deref().map(str::trim), Some("1" | "true" | "TRUE" | "yes" | "on"))
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
        assert!(!ep.enable_thinking);
    }

    #[test]
    fn thinking_defaults_off_and_can_be_set() {
        let off = LlmEndpoint::from_parts("http://127.0.0.1:11434/v1", "k", "m").unwrap();
        assert!(!off.public().enable_thinking);
        assert_eq!(serde_json::to_value(off.chat_template_kwargs()).unwrap()["enable_thinking"], false);
        let on = off.with_thinking(true);
        assert!(on.enable_thinking);
        assert!(on.public().enable_thinking);
        assert!(!LlmPublic::empty().enable_thinking);
        assert!(on.chat_template_kwargs().is_none());
    }

    #[test]
    fn thinking_payload_matches_provider() {
        let local = LlmEndpoint::from_parts("http://192.168.178.15:8000/v1", "", "Qwen3").unwrap().with_provider(LlmProviderKind::Lemonade);
        assert!(local.thinking_extras().chat_template_kwargs.is_some());
        assert!(local.clone().with_thinking(true).thinking_extras().chat_template_kwargs.is_none());

        let openai = LlmEndpoint::from_parts("https://api.openai.com/v1", "k", "gpt-4o-mini").unwrap();
        assert!(openai.thinking_extras().chat_template_kwargs.is_none());
        assert!(openai.thinking_extras().reasoning_effort.is_none());
        let o3 = LlmEndpoint::from_parts("https://api.openai.com/v1", "k", "o3-mini").unwrap();
        assert_eq!(o3.thinking_extras().reasoning_effort, Some("none"));
        assert_eq!(o3.with_thinking(true).thinking_extras().reasoning_effort, Some("medium"));

        let anthropic = LlmEndpoint::from_parts("https://api.anthropic.com/v1", "k", "claude-sonnet-4-5").unwrap();
        assert!(anthropic.thinking_extras().thinking.is_none());
        assert_eq!(anthropic.with_thinking(true).thinking_extras().thinking.unwrap().kind, "enabled");

        let google = LlmEndpoint::from_parts("https://generativelanguage.googleapis.com/v1beta/openai/", "k", "gemini-2.5-flash").unwrap();
        assert_eq!(google.thinking_extras().reasoning_effort, Some("none"));
        assert!(google.thinking_extras().extra_body.is_some());
        assert_eq!(google.with_thinking(true).thinking_extras().reasoning_effort, Some("medium"));
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
    fn lemonade_open_ai_port_is_lemonade() {
        assert_eq!(LlmProviderKind::from_url("http://192.168.178.15:8000/v1"), LlmProviderKind::Lemonade);
        let raw = serde_json::from_str::<LlmProviderKind>("\"unknown-host\"").unwrap();
        assert_eq!(raw, LlmProviderKind::Custom);
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
