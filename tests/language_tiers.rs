//! Tiers are decided here, not by opinion.
//!
//! Every compiled locale runs its on/off minimal pairs (`tests/datasets/minimal_pairs`,
//! built from the locale's own generator lexicon). A locale earns:
//! - **Reference** when every "on"/"off" acts correctly, questions and negated
//!   commands never switch anything (de/en and their regional packs).
//! - **Beta** when basic on/off never reverse and questions stay inert; missing
//!   negation alone does not force Experimental (package 4.3).
//! - **Experimental** otherwise.
//!
//! Opposite action on a basic pair is a hard gate for every locale. The registry
//! must say what this test finds (`scripts/lang_packs/tiers.py`).
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

#[derive(Default)]
struct Findings {
    opposite: Vec<String>,
    basic: Vec<String>,
    question: Vec<String>,
    negation: Vec<String>,
}

impl Findings {
    fn all(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.extend(self.opposite.iter().cloned());
        out.extend(self.basic.iter().cloned());
        out.extend(self.question.iter().cloned());
        out.extend(self.negation.iter().cloned());
        out
    }

    fn earned(&self) -> Tier {
        if !self.opposite.is_empty() || !self.basic.is_empty() || !self.question.is_empty() {
            return Tier::Experimental;
        }
        if self.negation.is_empty() {
            Tier::Reference
        } else {
            Tier::Beta
        }
    }
}

/// Why a locale is not reference. Empty when it passes every check.
fn findings(code: &str, all: &BTreeMap<String, Pairs>, home: &HomeGraph) -> Findings {
    let own = all.get(code).unwrap_or_else(|| panic!("no minimal pairs for {code}"));
    let mut found = Findings::default();
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
                    found.opposite.push(format!("opposite action: {text:?} ({whose}) gives {}", got.shown));
                } else if !right {
                    found.basic.push(format!("basic command fails: {text:?} ({whose}) gives {}", got.shown));
                }
            }
        }
    }
    for text in [own.on.first(), own.off.first()].into_iter().flatten() {
        let got = effect(code, home, &format!("{text}?"));
        if got.on || got.off {
            found.question.push(format!("acts on a question: \"{text}?\" gives {}", got.shown));
        }
    }
    let negation = catalog_for(&[code.to_string()]).moods().find_map(|mood| mood.negation.first().copied());
    match (negation, own.on.first()) {
        (Some(word), Some(text)) => {
            let got = effect(code, home, &format!("{word} {text}"));
            if got.on || got.off {
                found.negation.push(format!("acts on a negated command: \"{word} {text}\" gives {}", got.shown));
            }
        }
        _ => found.negation.push("no negation words: the pack cannot tell a negated command from a command".to_string()),
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
        let earned = found.earned();
        let listed = lang.meta().expect("registry row").tier;
        // Reference is reserved for the hand-written line; a generated pack that
        // fully passes still sits in Beta until it is promoted in tiers.py.
        let expected = match (listed, earned) {
            (Tier::Reference, Tier::Reference) => Tier::Reference,
            (Tier::Reference, other) => other,
            (Tier::Beta, Tier::Reference | Tier::Beta) => Tier::Beta,
            (Tier::Beta, Tier::Experimental) => Tier::Experimental,
            (Tier::Experimental, _) => Tier::Experimental,
        };
        println!("TIER {code}: listed={listed:?} earned={earned:?} {}", found.all().join(" | "));
        if listed != expected && !(listed == Tier::Experimental && earned == Tier::Experimental) {
            if listed == Tier::Experimental && matches!(earned, Tier::Reference | Tier::Beta) {
                // Still experimental in the registry until promoted — fine.
                continue;
            }
            if listed == Tier::Beta && earned == Tier::Reference {
                continue;
            }
            wrong.push(format!("{code} is listed as {listed:?} but its pairs say {earned:?}: {}", found.all().join(" | ")));
        }
        if listed == Tier::Reference && earned != Tier::Reference {
            wrong.push(format!("{code} is listed as Reference but its pairs say {earned:?}: {}", found.all().join(" | ")));
        }
        if listed == Tier::Beta && earned == Tier::Experimental {
            wrong.push(format!("{code} is listed as Beta but its pairs say Experimental: {}", found.all().join(" | ")));
        }
    }
    assert!(wrong.is_empty(), "fix REFERENCE/BETA in scripts/lang_packs/tiers.py and regenerate:\n{}", wrong.join("\n"));
}

/// Hard gate: no locale switches the opposite way on its basic pairs (package 4.3).
#[test]
fn no_locale_switches_the_opposite_way() {
    let all = pairs();
    let home = default_home();
    let mut bad = Vec::new();
    for lang in LangId::all() {
        let code = lang.code();
        let own = &all[code];
        for set in [Some(own), own.also.as_ref().map(|base| &all[base])].into_iter().flatten() {
            for text in &set.on {
                let got = effect(code, &home, text);
                if got.off {
                    bad.push(format!("{code} {text:?} turns off ({})", got.shown));
                }
            }
            for text in &set.off {
                let got = effect(code, &home, text);
                if got.on {
                    bad.push(format!("{code} {text:?} turns on ({})", got.shown));
                }
            }
        }
    }
    assert!(bad.is_empty(), "opposite action:\n{}", bad.join("\n"));
}

/// The acceptance of package 1.12 / 4.3: no offered-without-switch locale turns an
/// "on" sentence into "off" or the other way round.
#[test]
fn no_offered_locale_switches_the_opposite_way() {
    let all = pairs();
    let home = default_home();
    let mut offered = 0;
    for lang in LangId::all().iter().filter(|lang| lang.meta().is_some_and(|meta| meta.tier.always_offered())) {
        offered += 1;
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
    assert!(offered >= 2, "de and en are reference");
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

    let broken = Settings { default_language: "xx".into(), ..Settings::default() };
    assert_eq!(broken.default_pack(), "en");
    assert_eq!(first_step(&broken, "turn on the living room light"), "Execute HassTurnOn");

    let pinned = Settings { languages: vec!["de".into()], default_language: "en".into(), ..Settings::default() };
    assert_eq!(first_step(&pinned, "schalte das licht im wohnzimmer ein"), "Execute HassTurnOn");
}

/// A list of every locale cannot be merged into one catalog; it used to bind nothing as well.
#[test]
fn a_list_of_every_locale_binds_the_default_pack_too() {
    let everything = Settings { languages: LangId::compiled_codes(), ..Settings::default() };
    assert_eq!(first_step(&everything, "turn on the living room light"), "Execute HassTurnOn");
}

/// R-21: bilingual homes answer in the language of the utterance, not the alphabetically first pack.
#[test]
fn bilingual_speech_follows_the_utterance_language() {
    let home = default_home();
    let settings = Settings { languages: vec!["de".into(), "en".into()], ..Settings::default() };
    let mut session = Session::new();
    let german = parse("Mach das Licht im Wohnzimmer an", &home, &mut session, &[], &settings);
    assert!(german.speech.contains("ist an") || german.speech.contains("an."), "german reply: {}", german.speech);
    let english = parse("turn on the light in the Wohnzimmer", &home, &mut session, &[], &settings);
    let lower = english.speech.to_lowercase();
    assert!(lower.contains("is on") || lower.ends_with(" on."), "english reply expected, got {}", english.speech);
    let clock = parse("what time is it", &home, &mut session, &[], &settings);
    let clock_l = clock.speech.to_lowercase();
    assert!(
        clock_l.starts_with("it is") || clock_l.contains("o'clock") || clock.speech.starts_with("It's"),
        "english clock expected, got {}",
        clock.speech
    );
}
