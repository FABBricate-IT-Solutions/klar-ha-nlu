use crate::types::Intent;
use serde::{Deserialize, Serialize};

pub const PARSE_SCHEMA_VERSION: &str = "2.0";
pub const MAX_PLAN_STEPS: usize = 32;
pub const MAX_CANDIDATES: usize = 64;
pub const MAX_EVIDENCE: usize = 128;
pub const MAX_EVIDENCE_PER_ITEM: usize = 16;
pub const MAX_CLARIFY_OPTIONS: usize = 32;
pub const MAX_TRACE_STAGES: usize = 16;
pub const MAX_TRACE_DISCARDED: usize = 64;
pub const MAX_DETAIL_CHARS: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParseOutcome {
    pub schema_version: String,
    pub text: String,
    pub conversation_id: String,
    pub decision: ParseDecision,
    pub speech: String,
    pub confidence: f64,
    pub margin: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_candidate_id: Option<String>,
    pub candidates: Vec<IntentCandidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<IntentPlan>,
    pub evidence: Vec<Evidence>,
    pub trace: ParseTrace,
    #[serde(default)]
    pub briefing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval: Option<Retrieval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_trace: Option<PolicyTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PolicyTrace {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_rule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit: Option<String>,
    #[serde(default)]
    pub compiled_risky: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParseDecision {
    Execute,
    Clarify { prompt: String, options: Vec<String> },
    Confirm { prompt: String, candidate_id: String },
    Reject { reason: RejectReason },
    Chat,
    Error { code: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentCandidate {
    pub id: String,
    pub plan: IntentPlan,
    pub score: f64,
    pub margin: f64,
    pub policy: String,
    pub precedence: u16,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentPlan {
    pub confidence: f64,
    pub margin: f64,
    pub evidence: Vec<Evidence>,
    pub steps: Vec<PlanStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanStep {
    pub index: usize,
    pub intent: Intent,
    pub confidence: f64,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub kind: String,
    pub source: String,
    pub value: String,
    pub score: f64,
    pub exact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RetrievalHit {
    pub entity_id: String,
    pub name: String,
    pub domain: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Retrieval {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<RetrievalHit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub areas: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub last: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RejectReason {
    EmptyInput,
    InvalidInput,
    NoAction,
    NoTarget,
    Ambiguous,
    Unsafe,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ParseTrace {
    pub stages: Vec<StageTrace>,
    pub discarded: Vec<DiscardedAlternative>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub normalized: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StageTrace {
    pub stage: String,
    pub duration_us: u64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscardedAlternative {
    pub candidate_id: String,
    pub policy: String,
    pub score: f64,
    pub reason: String,
}

impl IntentPlan {
    pub fn from_intents(intents: Vec<Intent>, confidence: f64, evidence: &[Evidence]) -> Self {
        let steps = intents
            .into_iter()
            .enumerate()
            .map(|(index, intent)| PlanStep { index, intent, confidence, evidence: evidence.to_vec() })
            .collect();
        Self { confidence, margin: 0.0, evidence: evidence.to_vec(), steps }
    }

    pub fn from_steps(steps: Vec<PlanStep>, margin: f64) -> Self {
        let confidence = steps.iter().map(|step| step.confidence).reduce(f64::min).unwrap_or(0.0);
        let mut evidence = Vec::new();
        for item in steps.iter().flat_map(|step| &step.evidence) {
            if !evidence.contains(item) {
                evidence.push(item.clone());
            }
        }
        evidence.truncate(MAX_EVIDENCE_PER_ITEM);
        Self { confidence, margin, evidence, steps }
    }

    pub fn intents(&self) -> Vec<Intent> {
        self.steps.iter().map(|step| step.intent.clone()).collect()
    }
}

impl ParseOutcome {
    pub fn schema_version() -> String {
        PARSE_SCHEMA_VERSION.to_string()
    }

    pub fn enforce_output_caps(&mut self) {
        if !matches!(self.decision, ParseDecision::Execute) {
            self.plan = None;
            self.selected_candidate_id = None;
            self.candidates.clear();
        }
        if !matches!(self.decision, ParseDecision::Chat | ParseDecision::Reject { .. }) {
            self.retrieval = None;
        }
        if let Some(retrieval) = &mut self.retrieval {
            retrieval.entities.truncate(8);
            retrieval.areas.truncate(8);
            retrieval.last.truncate(8);
            retrieval.custom.truncate(8);
            retrieval.tokens.truncate(32);
            for hit in &mut retrieval.entities {
                truncate_chars(&mut hit.entity_id, 128);
                truncate_chars(&mut hit.name, 128);
                truncate_chars(&mut hit.domain, 32);
                if let Some(area) = &mut hit.area {
                    truncate_chars(area, 128);
                }
            }
        }
        self.candidates.truncate(MAX_CANDIDATES);
        self.trace.stages.truncate(MAX_TRACE_STAGES);
        self.trace.discarded.truncate(MAX_TRACE_DISCARDED);
        self.trace.tokens.truncate(MAX_TRACE_STAGES.saturating_mul(16));
        truncate_chars(&mut self.trace.normalized, MAX_DETAIL_CHARS);
        match &mut self.decision {
            ParseDecision::Clarify { prompt, options } => {
                options.truncate(MAX_CLARIFY_OPTIONS);
                truncate_chars(prompt, MAX_DETAIL_CHARS);
            }
            ParseDecision::Confirm { prompt, candidate_id } => {
                truncate_chars(prompt, MAX_DETAIL_CHARS);
                if candidate_id.chars().count() > 128 {
                    candidate_id.clear();
                }
            }
            ParseDecision::Error { code, message } => {
                truncate_chars(code, 64);
                truncate_chars(message, MAX_DETAIL_CHARS);
            }
            ParseDecision::Execute | ParseDecision::Reject { .. } | ParseDecision::Chat => {}
        }
        cap_evidence(&mut self.evidence, MAX_EVIDENCE);
        for candidate in &mut self.candidates {
            truncate_chars(&mut candidate.policy, 128);
            cap_evidence(&mut candidate.evidence, MAX_EVIDENCE_PER_ITEM);
            cap_plan(&mut candidate.plan);
        }
        if let Some(plan) = &mut self.plan {
            cap_plan(plan);
        }
        for stage in &mut self.trace.stages {
            truncate_chars(&mut stage.detail, MAX_DETAIL_CHARS);
        }
        for discarded in &mut self.trace.discarded {
            truncate_chars(&mut discarded.candidate_id, 128);
            truncate_chars(&mut discarded.policy, 128);
            truncate_chars(&mut discarded.reason, MAX_DETAIL_CHARS);
        }
    }
}

fn cap_plan(plan: &mut IntentPlan) {
    cap_evidence(&mut plan.evidence, MAX_EVIDENCE_PER_ITEM);
    plan.steps.truncate(MAX_PLAN_STEPS);
    for step in &mut plan.steps {
        cap_evidence(&mut step.evidence, MAX_EVIDENCE_PER_ITEM);
    }
}

fn cap_evidence(evidence: &mut Vec<Evidence>, maximum: usize) {
    evidence.truncate(maximum);
    for item in evidence {
        truncate_chars(&mut item.kind, 64);
        truncate_chars(&mut item.source, 128);
        truncate_chars(&mut item.value, 512);
    }
}

fn truncate_chars(value: &mut String, max: usize) {
    if value.chars().count() <= max {
        return;
    }
    *value = value.chars().take(max).collect();
}
