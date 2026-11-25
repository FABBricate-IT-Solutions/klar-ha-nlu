//! Tiers are decided here, not by opinion.
//!
//! Every compiled locale runs its on/off minimal pairs (`tests/datasets/minimal_pairs`,
//! built from the locale's own generator lexicon). A locale is stable when every "on"
//! sentence switches on, every "off" sentence switches off, and neither a question nor a
//! negated command switches anything. A regional variant also has to get the pairs of its
//! base language right. Everything else is experimental and sits behind
//! `Settings::experimental_languages`. The registry must say what this test finds.
use klar_nlu::home::default_home;
use klar_nlu::lang::{catalog_for, LangId, Tier};
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, ParseDecision, Settings};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Pairs {
    on: Vec<String>,
    off: Vec<String>,
    #[serde(default)]
    also: Option<String>,
}

fn pairs() -> BTreeMap<String, Pairs> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/datasets/minimal_pairs/pairs.yaml");
    let raw = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"));
    serde_yaml::from_str(&raw).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// What one sentence would switch, now or after a "yes".
struct Effect {
    on: bool,
    off: bool,
    shown: String,
}

fn effect(code: &str, home: &HomeGraph, text: &str) -> Effect {
    let mut session = Session::new();
    let out = parse(text, home, &mut session, &[], &Settings::pinned(code));
    let plan = match &out.decision {
        ParseDecision::Confirm { .. } => session.pending_confirm().map(|pending| pending.plan.clone()),
        _ => out.plan.clone(),
    };
    let names: Vec<String> = plan.map(|plan| plan.intents().into_iter().map(|intent| intent.name).collect()).unwrap_or_default();
    let kind = match &out.decision {
        ParseDecision::Execute => "execute".to_string(),
        ParseDecision::Confirm { .. } => "confirm".to_string(),
        ParseDecision::Clarify { .. } => "clarify".to_string(),
        ParseDecision::Reject { reason } => format!("reject {reason:?}"),
        ParseDecision::Chat => "chat".to_string(),
        ParseDecision::Error { code, .. } => format!("error {code}"),
    };
    Effect {
        on: names.iter().any(|name| name == "HassTurnOn"),
        off: names.iter().any(|name| name == "HassTurnOff"),
        shown: format!("{kind} {}", names.join("+")).trim().to_string(),
    }
}

/// Why a locale is not stable. Empty when it passes every check.
fn findings(code: &str, all: &BTreeMap<String, Pairs>, home: &HomeGraph) -> Vec<String> {
    let own = all.get(code).unwrap_or_else(|| panic!("no minimal pairs for {code}"));
    let mut found = Vec::new();
    let mut sets = vec![(own, "its own lexicon".to_string())];
    if let Some(base) = &own.also {
        sets.push((all.get(base).unwrap_or_else(|| panic!("no minimal pairs for {base}")), format!("the pairs of {base}")));
    }
    for (set, whose) in &sets {
        for (texts, wants_on) in [(&set.on, true), (&set.off, false)] {
            for text in texts {
                let got = effect(code, home, text);
                let (right, wrong) = if wants_on { (got.on, got.off) } else { (got.off, got.on) };
                if wrong {
                    found.push(format!("opposite action: {text:?} ({whose}) gives {}", got.shown));
                } else if !right {
                    found.push(format!("basic command fails: {text:?} ({whose}) gives {}", got.shown));
                }
            }
        }
    }
    for text in [own.on.first(), own.off.first()].into_iter().flatten() {
        let got = effect(code, home, &format!("{text}?"));
        if got.on || got.off {
            found.push(format!("acts on a question: \"{text}?\" gives {}", got.shown));
        }
    }
    let negation = catalog_for(&[code.to_string()]).moods().find_map(|mood| mood.negation.first().copied());
    match (negation, own.on.first()) {
        (Some(word), Some(text)) => {
            let got = effect(code, home, &format!("{word} {text}"));
            if got.on || got.off {
                found.push(format!("acts on a negated command: \"{word} {text}\" gives {}", got.shown));
            }
        }
        _ => found.push("no negation words: the pack cannot tell a negated command from a command".to_string()),
    }
    found
}

#[test]
fn every_locale_is_in_the_tier_its_minimal_pairs_put_it_in() {
    let all = pairs();
    let home = default_home();
    let mut wrong = Vec::new();
    for lang in LangId::all() {
        let code = lang.code();
        let found = findings(code, &all, &home);
        let earned = if found.is_empty() { Tier::Stable } else { Tier::Experimental };
        let listed = lang.meta().expect("registry row").tier;
        println!("TIER {code}: {earned:?} {}", found.join(" | "));
        if earned != listed {
            wrong.push(format!("{code} is listed as {listed:?} but its pairs say {earned:?}: {}", found.join(" | ")));
        }
    }
    assert!(wrong.is_empty(), "fix STABLE in scripts/lang_packs/tiers.py and regenerate:\n{}", wrong.join("\n"));
}

/// The acceptance of package 1.12 in one sentence: no locale that is offered without the
/// switch turns an "on" sentence into "off" or the other way round.
#[test]
fn no_stable_locale_switches_the_opposite_way() {
    let all = pairs();
    let home = default_home();
    let mut stable = 0;
    for lang in LangId::all().iter().filter(|lang| lang.meta().is_some_and(|meta| meta.tier == Tier::Stable)) {
        stable += 1;
        let code = lang.code();
        let own = &all[code];
        for set in [Some(own), own.also.as_ref().map(|base| &all[base])].into_iter().flatten() {
            for text in &set.on {
                let got = effect(code, &home, text);
                assert!(got.on && !got.off, "{code} {text:?} gives {}", got.shown);
            }
            for text in &set.off {
                let got = effect(code, &home, text);
                assert!(got.off && !got.on, "{code} {text:?} gives {}", got.shown);
            }
        }
    }
    assert!(stable >= 2, "de and en are stable");
}

#[test]
fn every_compiled_locale_has_an_on_and_an_off_pair() {
    let all = pairs();
    for lang in LangId::all() {
        let row = all.get(lang.code()).unwrap_or_else(|| panic!("no minimal pairs for {}", lang.code()));
        assert!(!row.on.is_empty() && !row.off.is_empty(), "{} needs an on and an off sentence", lang.code());
    }
    assert_eq!(all.len(), LangId::all().len(), "pairs for a locale that is not compiled");
}

fn first_step(settings: &Settings, text: &str) -> String {
    let home = default_home();
    let mut session = Session::new();
    let out = parse(text, &home, &mut session, &[], settings);
    let names: Vec<String> = out.plan.map(|plan| plan.intents().into_iter().map(|intent| intent.name).collect()).unwrap_or_default();
    format!("{:?} {}", out.decision, names.join("+"))
}

/// R-20: empty `languages` is the default. It bound an empty catalog, and
/// "turn on the living room light" became a state query.
#[test]
fn empty_languages_bind_the_configured_default_pack() {
    let unset = Settings::default();
    assert!(unset.languages.is_empty() && unset.default_language.is_empty());
    assert_eq!(first_step(&unset, "turn on the living room light"), "Execute HassTurnOn");
    assert_eq!(first_step(&unset, "turn off the living room light"), "Execute HassTurnOff");

    let german = Settings { default_language: "de".into(), ..Settings::default() };
    assert_eq!(first_step(&german, "schalte das licht im wohnzimmer ein"), "Execute HassTurnOn");
    assert_eq!(first_step(&german, "schalte das licht im wohnzimmer aus"), "Execute HassTurnOff");

    // A default no pack answers falls back to English instead of binding nothing.
    let broken = Settings { default_language: "xx".into(), ..Settings::default() };
    assert_eq!(broken.default_pack(), "en");
    assert_eq!(first_step(&broken, "turn on the living room light"), "Execute HassTurnOn");

    // An explicit list still wins over the default.
    let pinned = Settings { languages: vec!["de".into()], default_language: "en".into(), ..Settings::default() };
    assert_eq!(first_step(&pinned, "schalte das licht im wohnzimmer ein"), "Execute HassTurnOn");
}

/// A list of every locale cannot be merged into one catalog; it used to bind nothing as well.
#[test]
fn a_list_of_every_locale_binds_the_default_pack_too() {
    let everything = Settings { languages: LangId::compiled_codes(), ..Settings::default() };
    assert_eq!(first_step(&everything, "turn on the living room light"), "Execute HassTurnOn");
}
