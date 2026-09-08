//! Spoken path summary for `/api/lang/explain`.

use crate::types::{ParseDecision, ParseOutcome, PolicyTrace};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct ExplainOut {
    pub language: String,
    pub decision: String,
    pub confidence: f64,
    pub speech: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reply: String,
    pub stages: Vec<String>,
    pub evidence: Vec<String>,
    pub matched_custom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_trace: Option<PolicyTrace>,
}

pub(crate) fn explain_outcome(language: &str, outcome: &ParseOutcome) -> ExplainOut {
    let decision = match &outcome.decision {
        ParseDecision::Execute => "execute",
        ParseDecision::Confirm { .. } => "confirm",
        ParseDecision::Clarify { .. } => "clarify",
        ParseDecision::Reject { .. } => "reject",
        ParseDecision::Chat => "chat",
        ParseDecision::Error { .. } => "error",
    };
    let matched_custom =
        outcome.evidence.iter().find(|row| row.source.contains("custom") || row.kind.contains("custom")).map(|row| row.value.clone());
    ExplainOut {
        language: String::new(),
        decision: decision.into(),
        confidence: outcome.confidence,
        speech: path_explain_speech(language, outcome, decision),
        reply: outcome.speech.clone(),
        stages: outcome.trace.stages.iter().map(|stage| format!("{}: {}", stage.stage, stage.detail)).collect(),
        evidence: outcome.evidence.iter().map(|row| format!("{} {} {}", row.kind, row.source, row.value)).collect(),
        matched_custom,
        policy_trace: outcome.policy_trace.clone(),
    }
}

fn path_explain_speech(language: &str, outcome: &ParseOutcome, decision: &str) -> String {
    let trace = outcome.policy_trace.as_ref();
    let match_id = trace.and_then(|row| row.match_node.as_ref()).map(|node| node.id.as_str()).filter(|id| !id.is_empty());
    let seed_id = trace.and_then(|row| row.seed.as_ref()).map(|node| node.id.as_str()).filter(|id| !id.is_empty());
    let house_id =
        trace.and_then(|row| row.house.as_ref().map(|node| node.id.as_str()).or(row.matched_rule.as_deref())).filter(|id| !id.is_empty());
    let band = trace.and_then(|row| row.band.as_deref()).filter(|id| !id.is_empty()).unwrap_or(decision);
    let de = language == "de" || language.starts_with("de-");
    let mut parts = Vec::new();
    if let Some(id) = match_id {
        parts.push(format!("Match `{id}`"));
    }
    if let Some(id) = seed_id {
        parts.push(format!("Seed `{id}`"));
    }
    if let Some(id) = house_id {
        if de {
            parts.push(format!("Haus `{id}`"));
        } else {
            parts.push(format!("house `{id}`"));
        }
    }
    parts.push(if de { band_de(band) } else { band.into() });
    if parts.len() == 1 {
        return parts.pop().unwrap_or_default();
    }
    parts.join(", ") + "."
}

fn band_de(band: &str) -> String {
    match band {
        "execute" => "ausgeführt".into(),
        "confirm" => "bestätigen".into(),
        "clarify" => "nachfragen".into(),
        "reject" => "abgelehnt".into(),
        "chat" => "chat".into(),
        "error" => "fehler".into(),
        other => other.into(),
    }
}
