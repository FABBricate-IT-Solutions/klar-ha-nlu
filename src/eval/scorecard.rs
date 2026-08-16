use super::bench::BenchReport;
use super::metrics::EvalMetrics;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageCard {
    pub language: String,
    pub utterances: usize,
    pub metrics: EvalMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comparison {
    pub name: String,
    pub graph: String,
    pub cases: usize,
    pub ok: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scorecard {
    pub schema_version: String,
    pub languages: Vec<LanguageCard>,
    pub bench: BenchReport,
    pub comparison: Vec<Comparison>,
}

impl Scorecard {
    pub fn assemble(languages: Vec<LanguageCard>, bench: BenchReport, comparison: Vec<Comparison>) -> Self {
        Self { schema_version: "m7.1".into(), languages, bench, comparison }
    }
}

pub fn write_scorecard(path: &Path, card: &Scorecard) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let json = serde_json::to_string_pretty(card).map_err(|err| err.to_string())?;
    std::fs::write(path, json).map_err(|err| err.to_string())
}
