use crate::home::paths::{read_to_string_confined, write_confined};
use crate::io::auth::reads_allowed;
use crate::io::privacy::replay_tokens;
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
const JOURNAL_FILE: &str = "conversations.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationTurn {
    pub conversation_id: String,
    pub ts_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default)]
    pub tokens: Vec<String>,
    pub decision: String,
    pub speech: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech_source: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_area: Option<String>,
}

#[derive(Clone)]
pub struct ConversationJournal {
    dir: PathBuf,
    lock: std::sync::Arc<Mutex<Vec<ConversationTurn>>>,
}

impl ConversationJournal {
    pub fn open(data_dir: &FsPath) -> Self {
        let turns = read_turns(data_dir);
        Self { dir: data_dir.to_path_buf(), lock: std::sync::Arc::new(Mutex::new(turns)) }
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
        let _ = write_turns(&self.dir, &turns);
    }

    pub fn list(&self) -> Vec<ConversationTurn> {
        self.lock.lock().unwrap_or_else(|err| err.into_inner()).clone()
    }

    pub fn by_id(&self, conversation_id: &str) -> Vec<ConversationTurn> {
        self.list().into_iter().filter(|turn| turn.conversation_id == conversation_id).collect()
    }

    pub fn note_spoken(&self, conversation_id: Option<&str>, speech: &str, source: &str) {
        let Some(id) = sanitize_conversation_id(conversation_id) else {
            return;
        };
        let Some(speech) = sanitize_journal_speech(speech) else {
            return;
        };
        let mut turns = self.lock.lock().unwrap_or_else(|err| err.into_inner());
        if let Some(turn) = turns.iter_mut().rev().find(|turn| turn.conversation_id == id) {
            turn.speech = speech;
            turn.speech_source = Some(source.to_string());
            turn.ts_ms = now_ms();
        } else {
            turns.push(llm_turn(id, speech, source));
            if turns.len() > MAX_TURNS {
                let drop = turns.len() - MAX_TURNS;
                turns.drain(0..drop);
            }
        }
        let _ = write_turns(&self.dir, &turns);
    }
}

pub fn turn_from_outcome(
    outcome: &ParseOutcome,
    include_text: bool,
    last_names: Vec<String>,
    preferred_area: Option<String>,
) -> ConversationTurn {
    let (confirm_prompt, candidate_id) = match &outcome.decision {
        ParseDecision::Confirm { prompt, candidate_id } => (Some(prompt.clone()), Some(candidate_id.clone())),
        ParseDecision::Clarify { prompt, .. } => (Some(prompt.clone()), None),
        _ => (None, None),
    };
    ConversationTurn {
        conversation_id: outcome.conversation_id.clone(),
        ts_ms: now_ms(),
        text: include_text.then(|| outcome.text.clone()),
        tokens: replay_tokens(&outcome.text),
        decision: decision_name(&outcome.decision).into(),
        speech: outcome.speech.clone(),
        speech_source: None,
        confidence: outcome.confidence,
        briefing: outcome.briefing,
        evidence_kinds: outcome.evidence.iter().map(|item| item.kind.clone()).take(16).collect(),
        last_names,
        confirm_prompt,
        candidate_id,
        preferred_area,
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

fn sanitize_conversation_id(raw: Option<&str>) -> Option<String> {
    let id = raw?.trim();
    if id.is_empty() || id.chars().count() > 128 || id.chars().any(char::is_control) {
        return None;
    }
    Some(id.to_string())
}

fn sanitize_journal_speech(raw: &str) -> Option<String> {
    let speech = raw.trim();
    if speech.is_empty() || speech.starts_with("KLAR_") || speech.chars().count() > 4096 {
        return None;
    }
    Some(speech.to_string())
}

fn llm_turn(conversation_id: String, speech: String, source: &str) -> ConversationTurn {
    ConversationTurn {
        conversation_id,
        ts_ms: now_ms(),
        text: None,
        tokens: Vec::new(),
        decision: if source == "chat" { "chat".into() } else { "execute".into() },
        speech,
        speech_source: Some(source.to_string()),
        confidence: 0.0,
        briefing: false,
        evidence_kinds: Vec::new(),
        last_names: Vec::new(),
        confirm_prompt: None,
        candidate_id: None,
        preferred_area: None,
    }
}

fn read_turns(dir: &FsPath) -> Vec<ConversationTurn> {
    let raw = read_to_string_confined(dir, JOURNAL_FILE).unwrap_or_default();
    raw.lines().filter_map(|line| serde_json::from_str(line).ok()).collect()
}

fn write_turns(dir: &FsPath, turns: &[ConversationTurn]) -> std::io::Result<()> {
    let mut body = String::new();
    for turn in turns {
        body.push_str(&serde_json::to_string(turn).unwrap_or_default());
        body.push('\n');
    }
    write_confined(dir, JOURNAL_FILE, body.as_bytes())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/conversations", get(list_conversations))
        .route("/api/v2/conversations/{id}", get(get_conversation))
        .route("/api/v2/last-turn", get(last_turn))
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

async fn last_turn(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Option<ConversationTurn>>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(state.journal.list().into_iter().next_back()))
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
            quiet_ack_eligible: false,
        };
        let turn = turn_from_outcome(&outcome, false, Vec::new(), None);
        assert_eq!(turn.decision, "confirm");
        assert!(turn.text.is_none());
        assert_eq!(turn.tokens, replay_tokens("lock the door"));
        assert_eq!(turn.confirm_prompt.as_deref(), Some("Really?"));
        assert_eq!(turn.candidate_id.as_deref(), Some("sel"));
        let json = serde_json::to_string(&turn).expect("turn json");
        assert!(json.contains("\"last_names\":[]"), "{json}");
        assert!(json.contains("\"evidence_kinds\":[]"), "{json}");
        assert!(!json.contains("lock the door"), "{json}");
        assert!(json.contains("\"tokens\""), "{json}");
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
            quiet_ack_eligible: false,
        };
        let turn = turn_from_outcome(&outcome, true, vec!["Kugel".into()], Some("kueche".into()));
        assert_eq!(turn.preferred_area.as_deref(), Some("kueche"));
        assert_eq!(turn.text.as_deref(), Some("x"));
        assert_eq!(turn.tokens, replay_tokens("x"));
        assert!(turn.candidate_id.is_none());
    }

    #[test]
    fn default_journal_persists_tokens_not_raw_text() {
        let spoken = "Mach das Küchenlicht an!";
        let outcome = ParseOutcome {
            schema_version: "2.0".into(),
            text: spoken.into(),
            conversation_id: "c1".into(),
            decision: ParseDecision::Chat,
            speech: "Verstanden.".into(),
            confidence: 0.2,
            margin: 0.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            retrieval: None,
            policy_trace: None,
            quiet_ack_eligible: false,
        };
        let turn = turn_from_outcome(&outcome, false, vec!["HassTurnOn".into()], None);
        assert!(turn.text.is_none());
        assert_eq!(turn.tokens, replay_tokens(spoken));
        assert!(!turn.tokens.is_empty());
        let json = serde_json::to_string(&turn).expect("turn json");
        assert!(!json.contains(spoken), "{json}");
        assert!(!json.contains("Küchenlicht"), "{json}");
        assert!(json.contains("\"tokens\""), "{json}");
    }

    #[test]
    fn journal_roundtrip_keeps_tokens_without_raw_text() {
        let dir = std::env::temp_dir().join(format!("klar-journal-tokens-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp journal dir");
        let spoken = "Küche AN!";
        let outcome = ParseOutcome {
            schema_version: "2.0".into(),
            text: spoken.into(),
            conversation_id: "c9".into(),
            decision: ParseDecision::Execute,
            speech: "An.".into(),
            confidence: 0.9,
            margin: 1.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            retrieval: None,
            policy_trace: None,
            quiet_ack_eligible: false,
        };
        let journal = ConversationJournal::open(&dir);
        journal.append(turn_from_outcome(&outcome, false, Vec::new(), None));
        let reloaded = ConversationJournal::open(&dir);
        let turns = reloaded.list();
        assert_eq!(turns.len(), 1);
        assert!(turns[0].text.is_none());
        assert_eq!(turns[0].tokens, replay_tokens(spoken));
        let raw = crate::home::paths::read_to_string_confined(&dir, JOURNAL_FILE).expect("journal file");
        assert!(!raw.contains(spoken), "{raw}");
        assert!(!raw.contains("Küche"), "{raw}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_journal_line_defaults_empty_tokens() {
        let line = r#"{"conversation_id":"c1","ts_ms":1,"decision":"chat","speech":"hi","confidence":0.1,"briefing":false}"#;
        let turn: ConversationTurn = serde_json::from_str(line).expect("legacy turn");
        assert!(turn.tokens.is_empty());
        assert!(turn.text.is_none());
        assert!(turn.speech_source.is_none());
    }

    #[test]
    fn note_spoken_patches_chat_reply_on_latest_turn() {
        let dir = std::env::temp_dir().join(format!("klar-journal-chat-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp journal dir");
        let outcome = ParseOutcome {
            schema_version: "2.0".into(),
            text: "erzähl einen witz".into(),
            conversation_id: "c-chat".into(),
            decision: ParseDecision::Chat,
            speech: "Verstanden.".into(),
            confidence: 0.2,
            margin: 0.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            retrieval: None,
            policy_trace: None,
            quiet_ack_eligible: false,
        };
        let journal = ConversationJournal::open(&dir);
        journal.append(turn_from_outcome(&outcome, false, Vec::new(), None));
        journal.note_spoken(Some("c-chat"), "Zwei Roboter gehen in eine Bar.", "chat");
        journal.note_spoken(Some("c-chat"), "KLAR_PARSE: licht an", "chat");
        let turns = journal.list();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].speech, "Zwei Roboter gehen in eine Bar.");
        assert_eq!(turns[0].speech_source.as_deref(), Some("chat"));
        assert_eq!(turns[0].decision, "chat");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
