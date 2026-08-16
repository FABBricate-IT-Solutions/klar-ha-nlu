//! Held-out evaluation, Assist comparison, process benches, and release scorecards.

mod bench;
mod compare;
mod corpus;
mod metrics;
mod scorecard;

pub use bench::{bench_warm, BenchReport};
pub use corpus::{load_corpus, EvalItem, GoldIntent, Split, MIN_HELD_OUT};
pub use metrics::{score_items, EvalMetrics};
pub use scorecard::{write_scorecard, Comparison, Scorecard};

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
        languages.push(scorecard::LanguageCard { language: language.into(), utterances: items.len(), metrics });
    }
    let home = family_home("en")?;
    let bench = bench_warm(&home, "Turn on the Entryway Light", 128);
    let comparison = vec![
        compare::compare_assist(
            "assist_oracle",
            "wohnung_mittel",
            &compare::wohnung_mittel_home()?,
            "de",
            &compare::assist_dir("wohnung_mittel"),
        )?,
        compare::compare_assist(
            "assist_oracle",
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
    use super::{bench_warm, family_home, gate_scorecard, run_scorecard};
    use std::path::PathBuf;

    pub fn run(command: Command) -> Result<String, String> {
        match command {
            Command::Scorecard { out } => {
                let card = run_scorecard(out.as_deref())?;
                Ok(serde_json::to_string_pretty(&card).map_err(|err| err.to_string())?)
            }
            Command::Gate { out } => {
                let card = run_scorecard(out.as_deref())?;
                gate_scorecard(&card)?;
                Ok(serde_json::to_string_pretty(&card).map_err(|err| err.to_string())?)
            }
            Command::Bench { language, repeat } => {
                let home = family_home(&language)?;
                let text = if language.starts_with("de") { "Mach das Eingangslicht an" } else { "Turn on the Entryway Light" };
                let report = bench_warm(&home, text, repeat.max(8));
                Ok(format!(
                    "language={language} n={} p50_us={} p95_us={} p99_us={} rss_hint_kb={}",
                    report.samples, report.p50_us, report.p95_us, report.p99_us, report.rss_kb
                ))
            }
        }
    }

    pub enum Command {
        Scorecard { out: Option<PathBuf> },
        Gate { out: Option<PathBuf> },
        Bench { language: String, repeat: u32 },
    }
}
