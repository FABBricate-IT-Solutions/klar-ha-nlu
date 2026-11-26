use super::bench::BenchReport;
use super::metrics::EvalMetrics;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SCHEMA_VERSION: &str = "m7.2";
pub const UNKNOWN_COMMIT: &str = "unknown";

pub fn commit_sha() -> String {
    ["KLAR_COMMIT_SHA", "GITHUB_SHA"]
        .iter()
        .find_map(|name| std::env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty()))
        .unwrap_or_else(|| UNKNOWN_COMMIT.into())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageCard {
    pub language: String,
    pub utterances: usize,
    /// Items that are not out-of-domain: commands, queries, follow-ups and confirmations.
    pub in_domain: usize,
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
    /// Commit the card was produced for (`KLAR_COMMIT_SHA` or `GITHUB_SHA`), `unknown` outside CI.
    pub commit: String,
    pub languages: Vec<LanguageCard>,
    pub bench: BenchReport,
    pub comparison: Vec<Comparison>,
}

impl Scorecard {
    pub fn assemble(languages: Vec<LanguageCard>, bench: BenchReport, comparison: Vec<Comparison>) -> Self {
        Self { schema_version: SCHEMA_VERSION.into(), commit: commit_sha(), languages, bench, comparison }
    }
}

pub fn write_scorecard(path: &Path, card: &Scorecard) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let json = serde_json::to_string_pretty(card).map_err(|err| err.to_string())?;
    std::fs::write(path, json).map_err(|err| err.to_string())
}
