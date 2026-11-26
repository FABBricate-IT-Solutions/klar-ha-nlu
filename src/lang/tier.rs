//! Tiers say how far a locale is tested, not whether it ships.
//!
//! - **Reference** (`de`, `en`, and their regional packs): full on/off/negation
//!   minimal pairs (`tests/language_tiers.rs`; list in `scripts/lang_packs/tiers.py`).
//! - **Beta**: gold natural-sentence gate and no opposite action on basic pairs.
//! - **Experimental**: everything else — keeps working where configured, but is
//!   only offered for a new selection when `Settings::experimental_languages` is on.

use super::{pin_language, LangId};
use crate::types::Settings;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Reference,
    Beta,
    Experimental,
}

impl Tier {
    /// May this tier be picked for a new selection without the experimental switch?
    pub const fn always_offered(self) -> bool {
        match self {
            Self::Reference | Self::Beta => true,
            Self::Experimental => false,
        }
    }
}

/// The tier of a language tag ("de-DE" answers for "de"). `None` for a tag no pack answers.
pub fn tier_of(tag: &str) -> Option<Tier> {
    LangId::from_tag(tag).and_then(LangId::meta).map(|meta| meta.tier)
}

fn same_pack(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right) || LangId::from_tag(left).is_some_and(|id| Some(id) == LangId::from_tag(right))
}

/// May this locale be picked now? Reference and Beta always, experimental ones with
/// the switch, and any locale that is configured already: an update never takes a
/// language away.
pub fn offered(tag: &str, configured: &[String], experimental: bool) -> bool {
    match tier_of(tag) {
        Some(Tier::Reference | Tier::Beta) => true,
        Some(Tier::Experimental) => experimental || configured.iter().any(|have| same_pack(have, tag)),
        None => false,
    }
}

/// Checks a new `languages` list against the one in use and returns it as pinned tags.
/// An experimental locale may stay, but it cannot be added while the switch is off.
/// A locale that only an installed overlay pack answers has no tier and passes.
pub fn check_selection(wanted: &[String], configured: &[String], experimental: bool) -> Result<Vec<String>, String> {
    let mut tags = Vec::with_capacity(wanted.len());
    for raw in wanted {
        let tag = pin_language(raw).map_err(|_| format!("unknown language {raw}"))?;
        if tier_of(&tag) == Some(Tier::Experimental) && !offered(&tag, configured, experimental) {
            return Err(format!("language {tag} is experimental; turn on experimental_languages to select it"));
        }
        tags.push(tag);
    }
    Ok(tags)
}

/// The languages a house uses already: its list and its default pack.
pub fn in_use(settings: &Settings) -> Vec<String> {
    let mut codes = settings.languages.clone();
    codes.push(settings.default_language.clone());
    codes
}

/// The language choices of a settings save, checked against the settings in use. The
/// switch travels in the same save, so turning it on and picking a locale is one step.
/// A code no pack answers stays in `languages` and is ignored, as it always was; the
/// default pack has to be one a pack answers.
pub fn check_settings(mut next: Settings, current: &Settings) -> Result<Settings, String> {
    let kept = in_use(current);
    let known: Vec<String> = next.languages.iter().filter(|raw| pin_language(raw).is_ok()).cloned().collect();
    check_selection(&known, &kept, next.experimental_languages)?;
    let default = next.default_language.trim().to_string();
    next.default_language = match default.is_empty() {
        true => default,
        false => check_selection(&[default], &kept, next.experimental_languages)?.remove(0),
    };
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(codes: &[&str]) -> Vec<String> {
        codes.iter().map(|code| (*code).to_string()).collect()
    }

    fn experimental_code() -> &'static str {
        LangId::all()
            .iter()
            .find(|id| id.meta().is_some_and(|meta| meta.tier == Tier::Experimental))
            .expect("an experimental locale")
            .code()
    }

    #[test]
    fn reference_and_beta_are_always_offered() {
        assert_eq!(tier_of("de"), Some(Tier::Reference));
        assert_eq!(tier_of("en-US"), Some(Tier::Reference));
        assert_eq!(tier_of("nl"), Some(Tier::Beta));
        assert_eq!(tier_of("es-ES"), Some(Tier::Beta));
        assert!(offered("de", &[], false) && offered("en", &[], false) && offered("nl", &[], false));
        assert_eq!(check_selection(&list(&["de", "nl", "en-US"]), &[], false), Ok(list(&["de", "nl", "en-US"])));
    }

    #[test]
    fn an_experimental_locale_needs_the_switch_for_a_new_selection() {
        let code = experimental_code();
        assert!(!offered(code, &[], false));
        assert!(offered(code, &[], true));
        let error = check_selection(&list(&["de", code]), &list(&["de"]), false).expect_err("new experimental selection");
        assert!(error.contains("experimental"), "{error}");
        assert_eq!(check_selection(&list(&[code]), &[], true), Ok(list(&[code])));
    }

    #[test]
    fn a_configured_experimental_locale_stays_selectable() {
        let code = experimental_code();
        let configured = list(&[code]);
        assert!(offered(code, &configured, false));
        assert_eq!(check_selection(&list(&[code, "de"]), &configured, false), Ok(list(&[code, "de"])));
    }

    #[test]
    fn a_settings_save_keeps_what_is_in_use_and_refuses_a_new_experimental_pick() {
        let code = experimental_code();
        let fresh = Settings::default();
        let pick = Settings { languages: list(&[code]), ..Settings::default() };
        assert!(check_settings(pick.clone(), &fresh).is_err(), "new pick without the switch");
        let with_switch = Settings { experimental_languages: true, ..pick.clone() };
        assert_eq!(check_settings(with_switch, &fresh).expect("switch on").languages, list(&[code]));
        let running = Settings { languages: list(&[code]), ..Settings::default() };
        assert_eq!(check_settings(pick, &running).expect("configured").languages, list(&[code]));
        let default_pick = Settings { default_language: code.to_string(), ..Settings::default() };
        assert!(check_settings(default_pick.clone(), &fresh).is_err(), "experimental default without the switch");
        assert_eq!(check_settings(default_pick, &running).expect("in use").default_language, code);
        let unknown = Settings { default_language: "xx".into(), ..Settings::default() };
        assert!(check_settings(unknown, &fresh).is_err());
        let ignored = Settings { languages: list(&["de", "xx"]), ..Settings::default() };
        assert_eq!(check_settings(ignored, &fresh).expect("unknown codes are ignored").languages, list(&["de", "xx"]));
    }

    #[test]
    fn an_unknown_language_is_refused_with_or_without_the_switch() {
        assert_eq!(tier_of("xx"), None);
        assert!(!offered("xx", &[], true));
        assert!(check_selection(&list(&["xx"]), &[], true).is_err());
    }
}
