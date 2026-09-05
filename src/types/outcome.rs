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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PolicyTrace {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_rule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit: Option<String>,
    #[serde(default)]
    pub compiled_risky: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(default, rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_node: Option<PolicyTraceMatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<PolicyTraceLayer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub house: Option<PolicyTraceLayer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub band: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discarded: Vec<PolicyTraceDiscarded>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyTraceMatch {
    pub id: String,
    pub score: f64,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyTraceLayer {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit: Option<String>,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyTraceDiscarded {
    pub id: String,
    pub score: f64,
    pub reason: String,
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

impl ParseDecision {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Execute => "execute",
            Self::Clarify { .. } => "clarify",
            Self::Confirm { .. } => "confirm",
            Self::Reject { .. } => "reject",
            Self::Chat => "chat",
            Self::Error { .. } => "error",
        }
    }
}

impl PolicyTraceMatch {
    pub fn engine(id: impl Into<String>, score: f64) -> Self {
        Self { id: id.into(), score: score.clamp(0.0, 1.0), origin: "engine".into() }
    }

    pub fn from_candidate(candidate: &IntentCandidate) -> Option<Self> {
        let id = candidate.policy.trim();
        (!id.is_empty()).then(|| Self::engine(id, candidate.score))
    }
}

impl PolicyTraceLayer {
    pub fn house(id: impl Into<String>, hit: impl Into<String>) -> Self {
        Self { id: id.into(), hit: Some(hit.into()), origin: "operator".into() }
    }

    pub fn seed(id: impl Into<String>, hit: impl Into<String>) -> Self {
        Self { id: id.into(), hit: Some(hit.into()), origin: "seed".into() }
    }
}

impl PolicyTraceDiscarded {
    pub fn from_alternative(item: &DiscardedAlternative) -> Self {
        Self { id: item.policy.clone(), score: item.score.clamp(0.0, 1.0), reason: "lower_score".into() }
    }
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
        if let Some(policy) = &mut self.policy_trace {
            cap_policy_trace(policy);
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

fn cap_policy_trace(trace: &mut PolicyTrace) {
    if let Some(node) = &mut trace.match_node {
        truncate_chars(&mut node.id, 128);
        truncate_chars(&mut node.origin, 32);
        node.score = node.score.clamp(0.0, 1.0);
    }
    for layer in [&mut trace.seed, &mut trace.house].into_iter().flatten() {
        truncate_chars(&mut layer.id, 128);
        truncate_chars(&mut layer.origin, 32);
        if let Some(hit) = &mut layer.hit {
            truncate_chars(hit, 32);
        }
    }
    if let Some(band) = &mut trace.band {
        truncate_chars(band, 32);
    }
    if let Some(rule) = &mut trace.matched_rule {
        truncate_chars(rule, 64);
    }
    if let Some(hit) = &mut trace.hit {
        truncate_chars(hit, 32);
    }
    if let Some(payload) = &mut trace.payload {
        truncate_chars(payload, 500);
    }
    trace.discarded.truncate(MAX_TRACE_DISCARDED);
    for item in &mut trace.discarded {
        truncate_chars(&mut item.id, 128);
        truncate_chars(&mut item.reason, MAX_DETAIL_CHARS);
        item.score = item.score.clamp(0.0, 1.0);
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
