use crate::io::auth::reads_allowed;
use crate::io::state::AppState;
use crate::types::{ParseDecision, ParseOutcome};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_TURNS: usize = 200;
const TTL_MS: u64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationTurn {
    pub conversation_id: String,
    pub ts_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub decision: String,
    pub speech: String,
    pub confidence: f64,
    pub briefing: bool,
    #[serde(default)]
    pub evidence_kinds: Vec<String>,
    #[serde(default)]
    pub last_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_id: Option<String>,
}

#[derive(Clone)]
pub struct ConversationJournal {
    path: PathBuf,
    lock: std::sync::Arc<Mutex<Vec<ConversationTurn>>>,
}

impl ConversationJournal {
    pub fn open(data_dir: &FsPath) -> Self {
        let path = data_dir.join("conversations.jsonl");
        let turns = read_turns(&path);
        Self { path, lock: std::sync::Arc::new(Mutex::new(turns)) }
    }

    pub fn append(&self, turn: ConversationTurn) {
        let mut turns = self.lock.lock().unwrap_or_else(|err| err.into_inner());
        let now = turn.ts_ms;
        turns.retain(|item| now.saturating_sub(item.ts_ms) <= TTL_MS);
        turns.push(turn);
        if turns.len() > MAX_TURNS {
            let drop = turns.len() - MAX_TURNS;
            turns.drain(0..drop);
        }
        let _ = write_turns(&self.path, &turns);
    }

    pub fn list(&self) -> Vec<ConversationTurn> {
        self.lock.lock().unwrap_or_else(|err| err.into_inner()).clone()
    }

    pub fn by_id(&self, conversation_id: &str) -> Vec<ConversationTurn> {
        self.list().into_iter().filter(|turn| turn.conversation_id == conversation_id).collect()
    }
}

pub fn turn_from_outcome(outcome: &ParseOutcome, include_text: bool, last_names: Vec<String>) -> ConversationTurn {
    let (confirm_prompt, candidate_id) = match &outcome.decision {
        ParseDecision::Confirm { prompt, candidate_id } => (Some(prompt.clone()), Some(candidate_id.clone())),
        ParseDecision::Clarify { prompt, .. } => (Some(prompt.clone()), None),
        _ => (None, None),
    };
    ConversationTurn {
        conversation_id: outcome.conversation_id.clone(),
        ts_ms: now_ms(),
        text: include_text.then(|| outcome.text.clone()),
        decision: decision_name(&outcome.decision).into(),
        speech: outcome.speech.clone(),
        confidence: outcome.confidence,
        briefing: outcome.briefing,
        evidence_kinds: outcome.evidence.iter().map(|item| item.kind.clone()).take(16).collect(),
        last_names,
        confirm_prompt,
        candidate_id,
    }
}

fn decision_name(decision: &ParseDecision) -> &'static str {
    match decision {
        ParseDecision::Execute => "execute",
        ParseDecision::Clarify { .. } => "clarify",
        ParseDecision::Confirm { .. } => "confirm",
        ParseDecision::Reject { .. } => "reject",
        ParseDecision::Chat => "chat",
        ParseDecision::Error { .. } => "error",
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn read_turns(path: &FsPath) -> Vec<ConversationTurn> {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    raw.lines().filter_map(|line| serde_json::from_str(line).ok()).collect()
}

fn write_turns(path: &FsPath, turns: &[ConversationTurn]) -> std::io::Result<()> {
    let mut body = String::new();
    for turn in turns {
        body.push_str(&serde_json::to_string(turn).unwrap_or_default());
        body.push('\n');
    }
    std::fs::write(path, body)
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v2/conversations", get(list_conversations)).route("/api/v2/conversations/{id}", get(get_conversation))
}

async fn list_conversations(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Vec<ConversationTurn>>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(state.journal.list()))
}

async fn get_conversation(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Vec<ConversationTurn>>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if id.len() > 128 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(Json(state.journal.by_id(&id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ParseDecision, ParseOutcome, RejectReason};

    #[test]
    fn confirm_turn_hides_plan() {
        let outcome = ParseOutcome {
            schema_version: "2.0".into(),
            text: "lock the door".into(),
            conversation_id: "c1".into(),
            decision: ParseDecision::Confirm { prompt: "Really?".into(), candidate_id: "sel".into() },
            speech: "Really?".into(),
            confidence: 0.7,
            margin: 1.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            retrieval: None,
            policy_trace: None,
        };
        let turn = turn_from_outcome(&outcome, false, Vec::new());
        assert_eq!(turn.decision, "confirm");
        assert!(turn.text.is_none());
        assert_eq!(turn.confirm_prompt.as_deref(), Some("Really?"));
        assert_eq!(turn.candidate_id.as_deref(), Some("sel"));
        let json = serde_json::to_string(&turn).expect("turn json");
        assert!(json.contains("\"last_names\":[]"), "{json}");
        assert!(json.contains("\"evidence_kinds\":[]"), "{json}");
    }

    #[test]
    fn reject_turn_has_no_candidate() {
        let outcome = ParseOutcome {
            schema_version: "2.0".into(),
            text: "x".into(),
            conversation_id: "c1".into(),
            decision: ParseDecision::Reject { reason: RejectReason::NoAction },
            speech: String::new(),
            confidence: 0.0,
            margin: 0.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            retrieval: None,
            policy_trace: None,
        };
        let turn = turn_from_outcome(&outcome, true, vec!["Kugel".into()]);
        assert_eq!(turn.text.as_deref(), Some("x"));
        assert!(turn.candidate_id.is_none());
    }
}
