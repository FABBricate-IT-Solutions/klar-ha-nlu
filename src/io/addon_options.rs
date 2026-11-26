//! Home Assistant add-on options from `/data/options.json` (A-14).
//!
//! Supervisor writes JSON; sed-based parsing truncated tokens that contain
//! quotes or backslashes. Serde keeps the full string.

use serde::Deserialize;
use std::path::Path;

/// Fields Klar reads from the Supervisor options document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct AddonOptions {
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub support_bundle: Option<bool>,
    #[serde(default)]
    pub ui_locale: Option<String>,
}

impl AddonOptions {
    /// Load options from `data_dir/options.json`. Missing file → empty defaults.
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let path = data_dir.join("options.json");
        if !path.is_file() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
        Self::parse(&raw).map_err(|err| format!("{}: {err}", path.display()))
    }

    /// Parse a Supervisor options JSON body.
    pub fn parse(raw: &str) -> Result<Self, String> {
        serde_json::from_str(raw).map_err(|err| err.to_string())
    }

    /// Apply non-empty values onto process environment (entry / bootstrap).
    pub fn apply_env(&self) {
        if let Some(token) = self.token.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            if std::env::var_os("KLAR_TOKEN").is_none() {
                std::env::set_var("KLAR_TOKEN", token);
            }
        }
        if self.support_bundle == Some(true) && std::env::var_os("KLAR_SUPPORT_BUNDLE").is_none() {
            std::env::set_var("KLAR_SUPPORT_BUNDLE", "1");
        }
        if let Some(locale) = self.ui_locale.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            if std::env::var_os("KLAR_UI_LOCALE").is_none() {
                std::env::set_var("KLAR_UI_LOCALE", locale);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn keeps_quotes_and_backslashes_in_token() {
        let opts = AddonOptions::parse(r#"{"token":"ab\"cd\\ef","support_bundle":true,"ui_locale":"de"}"#).unwrap();
        assert_eq!(opts.token.as_deref(), Some("ab\"cd\\ef"));
        assert_eq!(opts.support_bundle, Some(true));
        assert_eq!(opts.ui_locale.as_deref(), Some("de"));
    }

    #[test]
    fn ignores_injected_token_inside_ui_locale_string() {
        let opts = AddonOptions::parse(r#"{"token":"s3cret","support_bundle":false,"ui_locale":"x\" , \"token\": \"evil"}"#).unwrap();
        assert_eq!(opts.token.as_deref(), Some("s3cret"));
        assert_eq!(opts.ui_locale.as_deref(), Some(r#"x" , "token": "evil"#));
    }

    #[test]
    fn decodes_unicode_escapes_and_slash() {
        let opts = AddonOptions::parse(r#"{"token":"p\u00e4ss/w\/rd"}"#).unwrap();
        assert_eq!(opts.token.as_deref(), Some("päss/w/rd"));
    }

    #[test]
    fn empty_token_stays_empty() {
        let opts = AddonOptions::parse(r#"{"token":"","support_bundle":false,"ui_locale":"token"}"#).unwrap();
        assert_eq!(opts.token.as_deref(), Some(""));
        assert_eq!(opts.ui_locale.as_deref(), Some("token"));
    }

    #[test]
    fn load_missing_file_is_default() {
        let dir = tempdir().unwrap();
        assert_eq!(AddonOptions::load(dir.path()).unwrap(), AddonOptions::default());
    }

    #[test]
    fn load_reads_file() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("options.json"), r#"{"token":"ab\"cd\\ef"}"#).unwrap();
        let opts = AddonOptions::load(dir.path()).unwrap();
        assert_eq!(opts.token.as_deref(), Some("ab\"cd\\ef"));
    }
}
