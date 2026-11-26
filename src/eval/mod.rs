//! Held-out evaluation, Assist comparison, process benches, and release scorecards.

mod bench;
mod compare;
mod corpus;
mod metrics;
mod scorecard;

pub use bench::{bench_language, bench_warm, format_language_bench, BenchReport, LanguageBench};
pub use corpus::{load_corpus, EvalItem, GoldIntent, Split, MIN_HELD_OUT, MIN_IN_DOMAIN};
pub use metrics::{score_items, EvalMetrics};
pub use scorecard::{commit_sha, write_scorecard, Comparison, Scorecard, SCHEMA_VERSION, UNKNOWN_COMMIT};

use crate::home::load_home_config;
use crate::nlu;
use crate::session::Session;
use crate::types::{HomeGraph, ParseOutcome, Settings};
use std::path::Path;

pub const MIN_INTENT_MACRO_F1: f64 = 0.98;
pub const MIN_SLOT_MICRO_F1: f64 = 0.99;
pub const MIN_PAIRING: f64 = 0.97;
pub const MIN_ASR_RECOVERY: f64 = 0.92;
pub const MIN_CLARIFY_PRECISION: f64 = 0.95;
/// Out-of-domain sentences that must end in a reject. Chat counts as a miss; the corpus has 440 per language.
pub const MAX_REJECT_FNR: f64 = 0.01;
/// In-domain sentences that may be rejected. Typo items are in-domain, so this also caps typo-induced rejects.
pub const MAX_REJECT_FPR: f64 = 0.02;
/// Out-of-domain sentences Klar may execute or ask about. A sentence outside the home must never act on it.
pub const MAX_REJECT_UNSAFE: f64 = 0.0;
/// Floor for the typo split (one dropped letter in an entity name). Measured, not aspired to: see docs/en/testing.md.
pub const MIN_TYPO_RECOVERY: f64 = 0.7;

pub fn evaluate_corpus(items: &[EvalItem], home: &HomeGraph, settings: &Settings) -> (EvalMetrics, Vec<ParseOutcome>) {
    let mut outcomes = Vec::with_capacity(items.len());
    for item in items {
        let mut session = Session::new();
        let mut last = None;
        for turn in &item.turns {
            last = Some(nlu::parse(turn, home, &mut session, &[], settings));
        }
        outcomes.push(last.expect("eval item has a turn"));
    }
    (score_items(items, &outcomes), outcomes)
}

pub fn evaluate_dir(dir: &Path, home: &HomeGraph, language: &str) -> Result<(Vec<EvalItem>, EvalMetrics), String> {
    let items = load_corpus(dir, language)?;
    let settings = Settings { languages: vec![language.into()], ..Settings::default() };
    let (metrics, _) = evaluate_corpus(&items, home, &settings);
    Ok((items, metrics))
}

pub fn family_home(language: &str) -> Result<HomeGraph, String> {
    let name = if language.starts_with("de") { "familienhaus_de" } else { "family_home_en" };
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datasets").join(name).join("home_config.yaml");
    load_home_config(&path)
}

pub fn heldout_dir(language: &str) -> std::path::PathBuf {
    let name = if language.starts_with("de") { "familienhaus_de" } else { "family_home_en" };
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datasets").join(name).join("m7_heldout")
}

pub fn run_scorecard(out: Option<&Path>) -> Result<Scorecard, String> {
    let mut languages = Vec::new();
    for language in ["en", "de"] {
        let home = family_home(language)?;
        let (items, metrics) = evaluate_dir(&heldout_dir(language), &home, language)?;
        languages.push(scorecard::LanguageCard {
            language: language.into(),
            utterances: items.len(),
            in_domain: items.iter().filter(|item| item.in_domain()).count(),
            metrics,
        });
    }
    let home = family_home("en")?;
    let bench = bench_warm(&home, "Turn on the Entryway Light", "en", 128);
    let comparison = vec![
        compare::compare_assist(
            "klar_on_assist_cases",
            "wohnung_mittel",
            &compare::wohnung_mittel_home()?,
            "de",
            &compare::assist_dir("wohnung_mittel"),
        )?,
        compare::compare_assist(
            "klar_on_assist_cases",
            "wohnung_live",
            &compare::wohnung_live_home()?,
            "de",
            &compare::assist_dir("wohnung_live"),
        )?,
    ];
    let card = Scorecard::assemble(languages, bench, comparison);
    if let Some(path) = out {
        write_scorecard(path, &card)?;
    }
    Ok(card)
}

pub fn gate_scorecard(card: &Scorecard) -> Result<(), String> {
    let mut errors = Vec::new();
    for language in &card.languages {
        if language.utterances < MIN_HELD_OUT {
            errors.push(format!("{} utterances {} < {MIN_HELD_OUT}", language.language, language.utterances));
        }
        if language.in_domain < MIN_IN_DOMAIN {
            errors.push(format!("{} in-domain utterances {} < {MIN_IN_DOMAIN}", language.language, language.in_domain));
        }
        if language.metrics.reject_fnr > MAX_REJECT_FNR + 1e-9 {
            errors.push(format!("{} reject miss rate {} > {MAX_REJECT_FNR}", language.language, language.metrics.reject_fnr));
        }
        if language.metrics.reject_fpr > MAX_REJECT_FPR + 1e-9 {
            errors.push(format!("{} in-domain reject rate {} > {MAX_REJECT_FPR}", language.language, language.metrics.reject_fpr));
        }
        if language.metrics.reject_unsafe > MAX_REJECT_UNSAFE + 1e-9 {
            errors.push(format!("{} out-of-domain acted on {} > {MAX_REJECT_UNSAFE}", language.language, language.metrics.reject_unsafe));
        }
        if language.metrics.typo_recovery + 1e-9 < MIN_TYPO_RECOVERY {
            errors.push(format!("{} typo recovery {} < {MIN_TYPO_RECOVERY}", language.language, language.metrics.typo_recovery));
        }
        if language.metrics.intent_macro_f1 + 1e-9 < MIN_INTENT_MACRO_F1 {
            errors.push(format!("{} intent F1 {}", language.language, language.metrics.intent_macro_f1));
        }
        if language.metrics.slot_micro_f1 + 1e-9 < MIN_SLOT_MICRO_F1 {
            errors.push(format!("{} slot F1 {}", language.language, language.metrics.slot_micro_f1));
        }
        if language.metrics.intent_slot_pairing + 1e-9 < MIN_PAIRING {
            errors.push(format!("{} pairing {}", language.language, language.metrics.intent_slot_pairing));
        }
        if language.metrics.asr_recovery + 1e-9 < MIN_ASR_RECOVERY {
            errors.push(format!("{} ASR {}", language.language, language.metrics.asr_recovery));
        }
        if language.metrics.clarify_precision + 1e-9 < MIN_CLARIFY_PRECISION {
            errors.push(format!("{} clarify P {}", language.language, language.metrics.clarify_precision));
        }
    }
    for row in &card.comparison {
        if row.cases == 0 || row.ok != row.cases {
            errors.push(format!("{} {} {}/{}", row.name, row.graph, row.ok, row.cases));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

pub use cli::{run as run_cli, Command as EvalCommand};

mod cli {
    use super::{bench_language, family_home, format_language_bench, gate_scorecard, run_scorecard, UNKNOWN_COMMIT};
    use std::path::PathBuf;

    pub fn run(command: Command) -> Result<String, String> {
        match command {
            Command::Scorecard { out } => {
                let card = run_scorecard(out.as_deref())?;
                Ok(serde_json::to_string_pretty(&card).map_err(|err| err.to_string())?)
            }
            Command::Gate { out } => {
                let card = run_scorecard(out.as_deref())?;
                if std::env::var_os("CI").is_some() && card.commit == UNKNOWN_COMMIT {
                    return Err("CI scorecard needs KLAR_COMMIT_SHA or GITHUB_SHA".into());
                }
                gate_scorecard(&card)?;
                Ok(serde_json::to_string_pretty(&card).map_err(|err| err.to_string())?)
            }
            Command::Bench { language, repeat } => {
                let home = family_home(&language)?;
                let bench = bench_language(&home, &language, repeat)?;
                Ok(format_language_bench(repeat, &bench))
            }
        }
    }

    pub enum Command {
        Scorecard { out: Option<PathBuf> },
        Gate { out: Option<PathBuf> },
        Bench { language: String, repeat: u32 },
    }
}
