//! BCP-47 tags for request pinning and pack lookup.

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocaleError {
    Empty,
    Invalid(String),
    Unknown(String),
}

impl Display for LocaleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "language tag is empty"),
            Self::Invalid(tag) => write!(f, "invalid BCP-47 tag: {tag}"),
            Self::Unknown(tag) => write!(f, "unknown language pack: {tag}"),
        }
    }
}

impl std::error::Error for LocaleError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocaleId {
    pub language: String,
    pub script: Option<String>,
    pub region: Option<String>,
    pub tag: String,
}

impl LocaleId {
    pub fn parse(raw: &str) -> Result<Self, LocaleError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(LocaleError::Empty);
        }
        if trimmed.len() > 35 {
            return Err(LocaleError::Invalid(trimmed.to_string()));
        }
        let parts: Vec<&str> = trimmed.split(['-', '_']).filter(|part| !part.is_empty()).collect();
        let language = parts.first().copied().unwrap_or_default().to_ascii_lowercase();
        if !is_language(&language) {
            return Err(LocaleError::Invalid(trimmed.to_string()));
        }
        let mut script = None;
        let mut region = None;
        for part in parts.iter().skip(1) {
            if script.is_none() && is_script(part) {
                script = Some(title_script(part));
                continue;
            }
            if region.is_none() && is_region(part) {
                region = Some(part.to_ascii_uppercase());
                continue;
            }
            break;
        }
        Ok(Self { tag: compose(&language, script.as_deref(), region.as_deref()), language, script, region })
    }

    pub fn fallback_chain(&self) -> Vec<Self> {
        let mut chain = vec![self.clone()];
        if self.script.is_some() && self.region.is_some() {
            push_unique(&mut chain, compose(&self.language, None, self.region.as_deref()));
            push_unique(&mut chain, compose(&self.language, self.script.as_deref(), None));
        }
        push_unique(&mut chain, self.language.clone());
        chain
    }
}

fn push_unique(chain: &mut Vec<LocaleId>, tag: String) {
    if chain.iter().any(|item| item.tag == tag) {
        return;
    }
    if let Ok(locale) = LocaleId::parse(&tag) {
        chain.push(locale);
    }
}

fn compose(language: &str, script: Option<&str>, region: Option<&str>) -> String {
    match (script, region) {
        (Some(script), Some(region)) => format!("{language}-{script}-{region}"),
        (Some(script), None) => format!("{language}-{script}"),
        (None, Some(region)) => format!("{language}-{region}"),
        (None, None) => language.to_string(),
    }
}

fn is_language(part: &str) -> bool {
    (2..=3).contains(&part.len()) && part.chars().all(|c| c.is_ascii_alphabetic())
}

fn is_script(part: &str) -> bool {
    part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic())
}

fn is_region(part: &str) -> bool {
    (part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic())) || (part.len() == 3 && part.chars().all(|c| c.is_ascii_digit()))
}

fn title_script(part: &str) -> String {
    let lower = part.to_ascii_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => lower,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_falls_back() {
        let us = LocaleId::parse("en-US").unwrap();
        assert_eq!(us.tag, "en-US");
        assert_eq!(us.fallback_chain().iter().map(|item| item.tag.as_str()).collect::<Vec<_>>(), ["en-US", "en"]);
        let at = LocaleId::parse("de_at").unwrap();
        assert_eq!(at.tag, "de-AT");
        assert_eq!(at.fallback_chain().iter().map(|item| item.tag.as_str()).collect::<Vec<_>>(), ["de-AT", "de"]);
        let hans = LocaleId::parse("zh-Hans-CN").unwrap();
        assert_eq!(
            hans.fallback_chain().iter().map(|item| item.tag.as_str()).collect::<Vec<_>>(),
            ["zh-Hans-CN", "zh-CN", "zh-Hans", "zh"]
        );
    }

    #[test]
    fn rejects_empty_and_garbage() {
        assert!(LocaleId::parse("").is_err());
        assert!(LocaleId::parse("1").is_err());
        assert!(LocaleId::parse("english").is_err());
    }
}
