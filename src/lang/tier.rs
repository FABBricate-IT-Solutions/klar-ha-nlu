//! Tiers say how far a locale is tested, not whether it ships.
//!
//! - **Reference**: full on/off/negation minimal pairs (`tests/language_tiers.rs`;
//!   list in `scripts/lang_packs/tiers.py`). Every shipped locale earns it today.
//! - **Beta**: gold natural-sentence gate and no opposite action on basic pairs.
//! - **Experimental**: a locale that is new or not yet listed — keeps working where
//!   configured, but is only offered for a new selection when
//!   `Settings::experimental_languages` is on.

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
    offered_in(tier_of, tag, configured, experimental)
}

/// `offered` with the tier lookup passed in, so the switch can be tested while no
/// shipped locale is experimental.
fn offered_in(tier: impl Fn(&str) -> Option<Tier>, tag: &str, configured: &[String], experimental: bool) -> bool {
    tier(tag).is_some_and(|tier| offered_as(tier, tag, configured, experimental))
}

/// May a locale of this tier be picked now?
pub fn offered_as(tier: Tier, tag: &str, configured: &[String], experimental: bool) -> bool {
    match tier {
        Tier::Reference | Tier::Beta => true,
        Tier::Experimental => experimental || configured.iter().any(|have| same_pack(have, tag)),
    }
}

/// Checks a new `languages` list against the one in use and returns it as pinned tags.
/// An experimental locale may stay, but it cannot be added while the switch is off.
/// A locale that only an installed overlay pack answers has no tier and passes.
pub fn check_selection(wanted: &[String], configured: &[String], experimental: bool) -> Result<Vec<String>, String> {
    check_selection_in(tier_of, wanted, configured, experimental)
}

fn check_selection_in(
    tier: impl Fn(&str) -> Option<Tier> + Copy,
    wanted: &[String],
    configured: &[String],
    experimental: bool,
) -> Result<Vec<String>, String> {
    let mut tags = Vec::with_capacity(wanted.len());
    for raw in wanted {
        let tag = pin_language(raw).map_err(|_| format!("unknown language {raw}"))?;
        if tier(&tag) == Some(Tier::Experimental) && !offered_in(tier, &tag, configured, experimental) {
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
pub fn check_settings(next: Settings, current: &Settings) -> Result<Settings, String> {
    check_settings_in(tier_of, next, current)
}

fn check_settings_in(tier: impl Fn(&str) -> Option<Tier> + Copy, mut next: Settings, current: &Settings) -> Result<Settings, String> {
    let kept = in_use(current);
    let known: Vec<String> = next.languages.iter().filter(|raw| pin_language(raw).is_ok()).cloned().collect();
    check_selection_in(tier, &known, &kept, next.experimental_languages)?;
    let default = next.default_language.trim().to_string();
    next.default_language = match default.is_empty() {
        true => default,
        false => check_selection_in(tier, &[default], &kept, next.experimental_languages)?.remove(0),
    };
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(codes: &[&str]) -> Vec<String> {
        codes.iter().map(|code| (*code).to_string()).collect()
    }

    /// Every shipped locale is reference, so the switch is tested with a lookup that
    /// calls Dutch experimental.
    const CODE: &str = "nl";

    fn tier_with_experimental_nl(tag: &str) -> Option<Tier> {
        match tag.split('-').next() == Some(CODE) {
            true => Some(Tier::Experimental),
            false => tier_of(tag),
        }
    }

    #[test]
    fn every_shipped_locale_is_always_offered() {
        assert_eq!(tier_of("de"), Some(Tier::Reference));
        assert_eq!(tier_of("en-US"), Some(Tier::Reference));
        assert_eq!(tier_of("nl"), Some(Tier::Reference));
        assert_eq!(tier_of("es-ES"), Some(Tier::Reference));
        assert!(
            LangId::all().iter().all(|id| id.meta().is_some_and(|meta| meta.tier == Tier::Reference)),
            "every shipped locale is reference"
        );
        assert!(offered("de", &[], false) && offered("en", &[], false) && offered("nl", &[], false));
        assert_eq!(check_selection(&list(&["de", "nl", "en-US"]), &[], false), Ok(list(&["de", "nl", "en-US"])));
    }

    #[test]
    fn an_experimental_locale_needs_the_switch_for_a_new_selection() {
        let code = CODE;
        assert!(!offered_in(tier_with_experimental_nl, code, &[], false));
        assert!(offered_in(tier_with_experimental_nl, code, &[], true));
        let error = check_selection_in(tier_with_experimental_nl, &list(&["de", code]), &list(&["de"]), false)
            .expect_err("new experimental selection");
        assert!(error.contains("experimental"), "{error}");
        assert_eq!(check_selection_in(tier_with_experimental_nl, &list(&[code]), &[], true), Ok(list(&[code])));
        assert!(offered(code, &[], false), "the shipped Dutch pack is reference and needs no switch");
    }

    #[test]
    fn a_configured_experimental_locale_stays_selectable() {
        let code = CODE;
        let configured = list(&[code]);
        assert!(offered_in(tier_with_experimental_nl, code, &configured, false));
        assert_eq!(check_selection_in(tier_with_experimental_nl, &list(&[code, "de"]), &configured, false), Ok(list(&[code, "de"])));
    }

    #[test]
    fn a_settings_save_keeps_what_is_in_use_and_refuses_a_new_experimental_pick() {
        let code = CODE;
        let fresh = Settings::default();
        let pick = Settings { languages: list(&[code]), ..Settings::default() };
        assert!(check_settings_in(tier_with_experimental_nl, pick.clone(), &fresh).is_err(), "new pick without the switch");
        let with_switch = Settings { experimental_languages: true, ..pick.clone() };
        assert_eq!(check_settings_in(tier_with_experimental_nl, with_switch, &fresh).expect("switch on").languages, list(&[code]));
        let running = Settings { languages: list(&[code]), ..Settings::default() };
        assert_eq!(check_settings_in(tier_with_experimental_nl, pick, &running).expect("configured").languages, list(&[code]));
        let default_pick = Settings { default_language: code.to_string(), ..Settings::default() };
        assert!(
            check_settings_in(tier_with_experimental_nl, default_pick.clone(), &fresh).is_err(),
            "experimental default without the switch"
        );
        assert_eq!(check_settings_in(tier_with_experimental_nl, default_pick, &running).expect("in use").default_language, code);
        let unknown = Settings { default_language: "xx".into(), ..Settings::default() };
        assert!(check_settings_in(tier_with_experimental_nl, unknown, &fresh).is_err());
        let ignored = Settings { languages: list(&["de", "xx"]), ..Settings::default() };
        assert_eq!(
            check_settings_in(tier_with_experimental_nl, ignored, &fresh).expect("unknown codes are ignored").languages,
            list(&["de", "xx"])
        );
    }

    #[test]
    fn an_unknown_language_is_refused_with_or_without_the_switch() {
        assert_eq!(tier_of("xx"), None);
        assert!(!offered("xx", &[], true));
        assert!(check_selection(&list(&["xx"]), &[], true).is_err());
    }
}
