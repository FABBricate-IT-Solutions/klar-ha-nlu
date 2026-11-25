//! Sentences that must never change device state: negation, questions, reports in the past
//! tense, conditions, time expressions and exceptions, in English and German, spoken in the
//! collision home.
//!
//! The corpus is `tests/fixtures/must_not_act/<lang>.txt`. Sentences the engine still acts on
//! are listed in `tests/fixtures/must_not_act/baseline.txt`. The suite fails when a sentence
//! outside the baseline acts, and it fails when a baseline sentence no longer acts, so a fix
//! has to delete its lines. The baseline only shrinks: a new line is a regression, not a
//! bookkeeping step.
//!
//! "Acts" means the engine committed to a change: it executes one, asks "really?" before
//! one, or asks which device it should change. A status query, a rejection or a reminder
//! does not act. Run with `KLAR_MUST_NOT_ACT_SHOW=1` and `--no-capture` to print what every
//! sentence does today.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

mod common;
use common::acts::{intended_changes, reach, show};
use common::collision_home::{collision_home, collision_home_de, collision_home_en};
use common::home::{describe, Talk};
use klar_nlu::types::HomeGraph;

const LANGS: &[&str] = &["en", "de"];
const CATEGORIES: &[&str] = &["negation", "question", "past", "condition", "time", "exception"];

struct Case {
    category: String,
    text: String,
    /// Empty: no device may change. Otherwise: only these devices must stay untouched.
    protected: Vec<String>,
}

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/must_not_act").join(name);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn corpus(lang: &str) -> Vec<Case> {
    let mut category = String::new();
    let mut cases = Vec::new();
    for line in fixture(&format!("{lang}.txt")).lines().map(str::trim) {
        if let Some(name) = line.strip_prefix("## ") {
            category = name.trim().to_string();
        } else if !line.is_empty() && !line.starts_with('#') {
            assert!(!category.is_empty(), "{lang}.txt: sentence before the first category: {line}");
            let (text, protected) = line.split_once("=>").unwrap_or((line, ""));
            let protected = protected.split(',').map(str::trim).filter(|id| !id.is_empty()).map(String::from).collect();
            cases.push(Case { category: category.clone(), text: text.trim().to_string(), protected });
        }
    }
    cases
}

fn key(lang: &str, case: &Case) -> String {
    format!("{lang} | {} | {}", case.category, case.text)
}

fn baseline() -> Vec<String> {
    fixture("baseline.txt").lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#')).map(String::from).collect()
}

/// What the engine would change although the sentence forbids it, or `None`.
fn violation(lang: &str, home: &HomeGraph, case: &Case) -> Option<String> {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(&case.text);
    if std::env::var_os("KLAR_MUST_NOT_ACT_SHOW").is_some() {
        println!("{} => {} say={:?}", key(lang, case), describe(&out), out.speech);
    }
    let forbidden: Vec<String> = intended_changes(&out, &talk.session, home)
        .iter()
        .filter(|step| case.protected.is_empty() || reach(step, home).iter().any(|id| case.protected.contains(id)))
        .map(show)
        .collect();
    (!forbidden.is_empty()).then(|| format!("{}: {}", out.decision.type_name(), forbidden.join(" + ")))
}

fn check(lang: &str) {
    let home = collision_home(lang);
    let known: BTreeSet<String> = baseline().into_iter().collect();
    let mut regressions = Vec::new();
    let mut fixed = Vec::new();
    for case in corpus(lang) {
        let line = key(lang, &case);
        match (violation(lang, &home, &case), known.contains(&line)) {
            (Some(what), false) => regressions.push(format!("{line}\n        {what}")),
            (None, true) => fixed.push(line),
            (Some(_), true) | (None, false) => {}
        }
    }
    let mut report = String::new();
    if !regressions.is_empty() {
        report += &format!(
            "{} sentence(s) change device state and are not in the baseline. Fix the engine; do not add them to baseline.txt:\n{}\n",
            regressions.len(),
            regressions.join("\n")
        );
    }
    if !fixed.is_empty() {
        report += &format!(
            "{} baseline sentence(s) no longer act. Delete these lines from tests/fixtures/must_not_act/baseline.txt:\n{}\n",
            fixed.len(),
            fixed.join("\n")
        );
    }
    assert!(report.is_empty(), "{report}");
}

#[test]
fn english_sentences_that_must_not_act() {
    check("en");
}

#[test]
fn german_sentences_that_must_not_act() {
    check("de");
}

/// Categories whose sentences no longer act at all. The baseline stays empty for them.
const CLOSED: &[&str] = &["negation", "question", "past", "condition", "time", "exception"];

#[test]
fn closed_categories_have_no_baseline() {
    let open: Vec<String> =
        baseline().into_iter().filter(|line| line.split(" | ").nth(1).is_some_and(|category| CLOSED.contains(&category))).collect();
    assert!(open.is_empty(), "these categories are closed and take no baseline lines:\n{}", open.join("\n"));
}

#[test]
fn every_category_has_sentences_in_both_languages() {
    for lang in LANGS {
        let cases = corpus(lang);
        let mut count: BTreeMap<&str, usize> = BTreeMap::new();
        for case in &cases {
            assert!(CATEGORIES.contains(&case.category.as_str()), "{lang}.txt: unknown category {}", case.category);
            *count.entry(case.category.as_str()).or_default() += 1;
        }
        for category in CATEGORIES {
            assert!(count.get(category).copied().unwrap_or(0) >= 10, "{lang}.txt: too few sentences under {category}");
        }
        let texts: BTreeSet<&str> = cases.iter().map(|case| case.text.as_str()).collect();
        assert_eq!(texts.len(), cases.len(), "{lang}.txt lists a sentence twice");
    }
}

#[test]
fn protected_devices_exist_and_only_exceptions_name_them() {
    for lang in LANGS {
        let home = collision_home(lang);
        for case in corpus(lang) {
            assert_eq!(case.category == "exception", !case.protected.is_empty(), "{lang}: {}", case.text);
            for id in &case.protected {
                assert!(home.entities.iter().any(|e| &e.entity_id == id), "{lang}: {} protects unknown device {id}", case.text);
            }
        }
    }
}

#[test]
fn baseline_lists_only_corpus_sentences_once_and_sorted() {
    let corpus_lines: BTreeSet<String> =
        LANGS.iter().flat_map(|lang| corpus(lang).iter().map(|case| key(lang, case)).collect::<Vec<_>>()).collect();
    let lines = baseline();
    for line in &lines {
        assert!(corpus_lines.contains(line), "baseline.txt names a sentence that is not in the corpus: {line}");
    }
    for pair in lines.windows(2) {
        assert!(pair[0] < pair[1], "baseline.txt must stay sorted and free of duplicates; out of place: {}", pair[1]);
    }
}

/// Q-11: the home must really contain the collisions the smaller test homes hide.
#[test]
fn collision_homes_contain_the_collisions() {
    for (lang, home, numbered, command_word) in
        [("en", collision_home_en(), ["Lamp 1", "Lamp 2"], "On Air"), ("de", collision_home_de(), ["Lampe 1", "Lampe 2"], "Auf Sendung")]
    {
        let named = |name: &str| home.entities.iter().filter(|e| e.name == name).collect::<Vec<_>>();
        let rooms_with = |name: &str| named(name).iter().filter_map(|e| e.area.clone()).collect::<BTreeSet<_>>();
        let duplicated = home.entities.iter().filter(|e| rooms_with(&e.name).len() > 1).map(|e| e.name.as_str()).collect::<BTreeSet<_>>();
        assert!(duplicated.len() >= 3, "{lang}: want three names used in several rooms, got {duplicated:?}");
        for name in numbered {
            assert_eq!(named(name).len(), 1, "{lang}: {name}");
        }
        assert_eq!(named(command_word).len(), 1, "{lang}: a device name made of command words");
        for area in &home.areas {
            let lights = home.entities.iter().filter(|e| e.domain == "light" && e.area.as_deref() == Some(area.area_id.as_str())).count();
            assert!(lights >= 2, "{lang}: {} has {lights} light(s)", area.name);
            assert!(area.floor_id.is_some(), "{lang}: {} has no floor", area.name);
        }
        for (domain, least) in [("lock", 2), ("cover", 3), ("binary_sensor", 3), ("climate", 2), ("switch", 3), ("scene", 2), ("script", 1)]
        {
            let count = home.entities.iter().filter(|e| e.domain == domain).count();
            assert!(count >= least, "{lang}: {count} {domain} entities, want at least {least}");
        }
        let shared_word = home.areas.iter().filter(|a| a.name.starts_with("Guest") || a.name.starts_with("Gäste")).count();
        assert_eq!(shared_word, 2, "{lang}: two rooms whose names share a word");
        assert_eq!(home.assist.as_ref().map(|ids| ids.len()), Some(home.entities.len()), "{lang}: everything is exposed");
    }
}
