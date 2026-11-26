//! Per-home regression dry-run (package 6.3).
//!
//! Saved marked-wrong cases run dry before trainer writes and after engine updates.
//! The update dialog lists how many cases change and which sentences are affected.

use crate::home::labels::rebuild_label_index;
use crate::home::paths::{read_to_string_confined, write_atomic_confined};
use crate::io::auth::{reads_allowed, writes_allowed};
use crate::io::state::AppState;
use crate::io::wrong_case::{IntendedExpect, WrongCase};
use crate::nlu::parse_with_controls;
use crate::session::Session;
use crate::types::{CustomSentence, MatchControl, ParseDecision, ParseOutcome, PolicyRule, Settings, SpeechBank};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::Path;

const BASELINE_FILE: &str = "wrong_case_baseline.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangedCase {
    pub id: String,
    pub text: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RegressionImpact {
    pub total: usize,
    pub changed: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<ChangedCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DryRunReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<DryRunCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DryRunCase {
    pub id: String,
    pub text: String,
    pub fingerprint: String,
    pub matches_intended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateDialog {
    pub engine_version: String,
    pub baseline_version: Option<String>,
    pub is_update: bool,
    pub total: usize,
    pub changed: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<ChangedCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Baseline {
    engine_version: String,
    #[serde(default)]
    outcomes: BTreeMap<String, String>,
}

/// Config stack used when dry-running a case's redacted home snippet.
pub struct RunConfig<'a> {
    pub settings: &'a Settings,
    pub house: &'a [PolicyRule],
    pub match_controls: &'a [MatchControl],
    pub custom: &'a [CustomSentence],
    pub speech_bank: &'a SpeechBank,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/wrong-cases/dry-run", get(dry_run_http))
        .route("/api/v2/wrong-cases/update-dialog", get(update_dialog_http).post(ack_update_dialog))
}

async fn dry_run_http(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<DryRunReport>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let cases = state.wrong_cases.load();
    let cfg = live_config(&state).await;
    Ok(Json(dry_run_cases(&cases, &cfg.borrow())))
}

async fn update_dialog_http(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<UpdateDialog>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let cases = state.wrong_cases.load();
    let cfg = live_config(&state).await;
    Ok(Json(update_dialog(&state.data_dir, &cases, &cfg.borrow())))
}

async fn ack_update_dialog(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<UpdateDialog>, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let cases = state.wrong_cases.load();
    let cfg = live_config(&state).await;
    let report = dry_run_cases(&cases, &cfg.borrow());
    write_baseline(&state.data_dir, &baseline_from_report(engine_version(), &report));
    Ok(Json(UpdateDialog {
        engine_version: engine_version().into(),
        baseline_version: Some(engine_version().into()),
        is_update: false,
        total: report.total,
        changed: 0,
        cases: Vec::new(),
    }))
}

async fn live_config(state: &AppState) -> OwnedConfig {
    OwnedConfig {
        settings: state.settings.lock().await.clone(),
        house: state.policies.lock().await.clone(),
        match_controls: state.match_controls.lock().await.clone(),
        custom: state.custom.lock().await.clone(),
        speech_bank: state.speech_bank.lock().await.clone(),
    }
}

struct OwnedConfig {
    settings: Settings,
    house: Vec<PolicyRule>,
    match_controls: Vec<MatchControl>,
    custom: Vec<CustomSentence>,
    speech_bank: SpeechBank,
}

impl OwnedConfig {
    fn borrow(&self) -> RunConfig<'_> {
        RunConfig {
            settings: &self.settings,
            house: &self.house,
            match_controls: &self.match_controls,
            custom: &self.custom,
            speech_bank: &self.speech_bank,
        }
    }
}

fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn dry_run_cases(cases: &[WrongCase], cfg: &RunConfig<'_>) -> DryRunReport {
    let mut rows = Vec::with_capacity(cases.len());
    let mut passed = 0usize;
    for case in cases {
        let fingerprint = run_fingerprint(case, cfg);
        let matches_intended = matches_intended(case, cfg);
        if matches_intended {
            passed += 1;
        }
        rows.push(DryRunCase { id: case.id.clone(), text: case.text.clone(), fingerprint, matches_intended });
    }
    let total = rows.len();
    DryRunReport { total, passed, failed: total.saturating_sub(passed), cases: rows }
}

/// Diff saved cases under a current stack vs a proposed trainer stack.
pub fn trainer_impact(cases: &[WrongCase], before: &RunConfig<'_>, after: &RunConfig<'_>) -> RegressionImpact {
    let mut changed = Vec::new();
    for case in cases {
        let left = run_fingerprint(case, before);
        let right = run_fingerprint(case, after);
        if left != right {
            changed.push(ChangedCase { id: case.id.clone(), text: case.text.clone(), before: left, after: right });
        }
    }
    RegressionImpact { total: cases.len(), changed: changed.len(), cases: changed }
}

pub fn update_dialog(data_dir: &Path, cases: &[WrongCase], cfg: &RunConfig<'_>) -> UpdateDialog {
    let version = engine_version().to_string();
    let report = dry_run_cases(cases, cfg);
    let baseline = read_baseline(data_dir);
    let Some(baseline) = baseline else {
        // First visit: seed quietly so the next engine bump has something to compare.
        write_baseline(data_dir, &baseline_from_report(&version, &report));
        return UpdateDialog {
            engine_version: version,
            baseline_version: None,
            is_update: false,
            total: report.total,
            changed: 0,
            cases: Vec::new(),
        };
    };
    if baseline.engine_version == version {
        return UpdateDialog {
            engine_version: version,
            baseline_version: Some(baseline.engine_version),
            is_update: false,
            total: report.total,
            changed: 0,
            cases: Vec::new(),
        };
    }
    let mut changed = Vec::new();
    for row in &report.cases {
        let before = baseline.outcomes.get(&row.id).cloned().unwrap_or_else(|| "(new)".into());
        if before != row.fingerprint {
            changed.push(ChangedCase { id: row.id.clone(), text: row.text.clone(), before, after: row.fingerprint.clone() });
        }
    }
    // Cases removed from the store but present in the baseline do not surface here; the dialog
    // is about sentences that still exist and whose behaviour moved.
    UpdateDialog {
        engine_version: version,
        baseline_version: Some(baseline.engine_version),
        is_update: true,
        total: report.total,
        changed: changed.len(),
        cases: changed,
    }
}

fn baseline_from_report(version: &str, report: &DryRunReport) -> Baseline {
    let mut outcomes = BTreeMap::new();
    for row in &report.cases {
        outcomes.insert(row.id.clone(), row.fingerprint.clone());
    }
    Baseline { engine_version: version.to_string(), outcomes }
}

fn read_baseline(dir: &Path) -> Option<Baseline> {
    let raw = read_to_string_confined(dir, BASELINE_FILE).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_baseline(dir: &Path, baseline: &Baseline) {
    match serde_json::to_vec_pretty(baseline) {
        Ok(body) => {
            if let Err(err) = write_atomic_confined(dir, BASELINE_FILE, &body) {
                tracing::warn!("{BASELINE_FILE} nicht gespeichert: {err}");
            }
        }
        Err(err) => tracing::warn!("{BASELINE_FILE} nicht serialisiert: {err}"),
    }
}

pub fn run_fingerprint(case: &WrongCase, cfg: &RunConfig<'_>) -> String {
    fingerprint(&run_case(case, cfg))
}

fn matches_intended(case: &WrongCase, cfg: &RunConfig<'_>) -> bool {
    intended_ok(&case.intended, &run_case(case, cfg))
}

fn run_case(case: &WrongCase, cfg: &RunConfig<'_>) -> ParseOutcome {
    let mut home = case.home.clone();
    rebuild_label_index(&mut home);
    let mut session = Session::new();
    let mut settings = cfg.settings.clone();
    if let Some(language) = case.language.as_deref().filter(|code| !code.is_empty()) {
        settings.languages = vec![language.to_string()];
    } else if settings.languages.is_empty() {
        settings.languages = vec!["de".into(), "en".into()];
    }
    parse_with_controls(&case.text, &home, &mut session, cfg.custom, &settings, cfg.house, cfg.speech_bank, cfg.match_controls)
}

fn fingerprint(outcome: &ParseOutcome) -> String {
    let decision = outcome.decision.type_name();
    let intents = match &outcome.decision {
        ParseDecision::Execute | ParseDecision::Confirm { .. } => outcome.plan.as_ref().map_or_else(Vec::new, |plan| plan.intents()),
        ParseDecision::Clarify { .. } | ParseDecision::Reject { .. } | ParseDecision::Chat | ParseDecision::Error { .. } => Vec::new(),
    };
    let mut parts = vec![decision.to_string()];
    for intent in intents {
        let mut slots: Vec<_> = intent.slots.iter().map(|slot| format!("{}={}", slot.name, slot.value)).collect();
        slots.sort();
        parts.push(format!("{}[{}]", intent.name, slots.join(",")));
    }
    parts.join("|")
}

fn intended_ok(intended: &IntendedExpect, outcome: &ParseOutcome) -> bool {
    let legacy = crate::nlu::legacy_result(outcome.clone());
    if intended.reject {
        return legacy.intents.is_empty() && !legacy.clarify && !legacy.chat;
    }
    if intended.clarify {
        return legacy.clarify;
    }
    if legacy.intents.len() != 1 {
        return false;
    }
    let got = &legacy.intents[0];
    if got.name != intended.intent {
        return false;
    }
    let actual: BTreeMap<_, _> = got.slots.iter().map(|slot| (slot.name.clone(), slot.value.clone())).collect();
    actual == intended.slots
}

/// Test helper: force a baseline for a prior engine version.
#[cfg(test)]
pub fn write_baseline_for_test(dir: &Path, version: &str, outcomes: BTreeMap<String, String>) {
    write_baseline(dir, &Baseline { engine_version: version.to_string(), outcomes });
}

#[cfg(test)]
#[path = "house_regression_tests.rs"]
mod tests;
