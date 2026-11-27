use crate::home::paths::{append_confined, read_to_string_confined, write_atomic_confined};
use crate::io::auth::reads_allowed;
use crate::io::privacy::replay_tokens;
use crate::io::state::AppState;
use crate::types::{ParseDecision, ParseOutcome, RefineBand, RejectReason};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

const MAX_TURNS: usize = 200;
const TTL_MS: u64 = 24 * 60 * 60 * 1000;
const JOURNAL_FILE: &str = "conversations.jsonl";
const LIVE_CAP: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationTurn {
    pub conversation_id: String,
    pub ts_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default)]
    pub tokens: Vec<String>,
    pub decision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<String>,
    pub speech: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refine_band: Option<String>,
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
    live: broadcast::Sender<ConversationTurn>,
}

impl ConversationJournal {
    pub fn open(data_dir: &FsPath) -> Self {
        let turns = read_turns(data_dir);
        let (live, _) = broadcast::channel(LIVE_CAP);
        Self { dir: data_dir.to_path_buf(), lock: std::sync::Arc::new(Mutex::new(turns)), live }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConversationTurn> {
        self.live.subscribe()
    }

    fn publish(&self, turn: &ConversationTurn) {
        let _ = self.live.send(turn.clone());
    }

    pub fn append(&self, turn: ConversationTurn) {
        let mut turns = self.lock.lock().unwrap_or_else(|err| err.into_inner());
        let now = turn.ts_ms;
        let before = turns.len();
        turns.retain(|item| now.saturating_sub(item.ts_ms) <= TTL_MS);
        let compacted = turns.len() != before;
        turns.push(turn.clone());
        let overflow = turns.len() > MAX_TURNS;
        if overflow {
            let drop = turns.len() - MAX_TURNS;
            turns.drain(0..drop);
        }
        // D-11: append one line when the file still matches memory; rewrite atomically when
        // TTL or the turn cap drops older rows (kill mid-rename leaves the previous file).
        if compacted || overflow {
            if let Err(err) = rewrite_turns(&self.dir, &turns) {
                tracing::warn!("conversations.jsonl nicht gespeichert: {err}");
            }
        } else if let Err(err) = append_turn(&self.dir, &turn) {
            tracing::warn!("conversations.jsonl nicht angehängt: {err}");
        }
        self.publish(&turn);
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
            let published = turn.clone();
            if let Err(err) = rewrite_turns(&self.dir, &turns) {
                tracing::warn!("conversations.jsonl nicht gespeichert: {err}");
            }
            drop(turns);
            self.publish(&published);
            return;
        }
        let turn = llm_turn(id, speech, source);
        turns.push(turn.clone());
        if turns.len() > MAX_TURNS {
            let drop = turns.len() - MAX_TURNS;
            turns.drain(0..drop);
            if let Err(err) = rewrite_turns(&self.dir, &turns) {
                tracing::warn!("conversations.jsonl nicht gespeichert: {err}");
            }
        } else if let Err(err) = append_turn(&self.dir, &turn) {
            tracing::warn!("conversations.jsonl nicht angehängt: {err}");
        }
        drop(turns);
        self.publish(&turn);
    }
}

/// Builds the operator's journal line for one parse.
///
/// The journal is the local diagnostic view of this box: capped at 200 turns for 24 hours,
/// served only to read-scoped callers. It keeps the heard utterance, its replay tokens, and
/// the spoken answer, because the Conversations page, **Nochmal**, **Das war falsch**, and the
/// trainer's case export are unusable without them. Raw-text consent (D-20) still gates every
/// path that leaves the box: the support-bundle download (`bundle.rs`, `privacy.rs`), the
/// metrics, and the turns handed to the LLM trainer (`trainer_turns.rs`).
pub fn turn_from_outcome(outcome: &ParseOutcome, last_names: Vec<String>, preferred_area: Option<String>) -> ConversationTurn {
    let (confirm_prompt, candidate_id) = match &outcome.decision {
        ParseDecision::Confirm { prompt, candidate_id } => (Some(prompt.clone()), Some(candidate_id.clone())),
        ParseDecision::Clarify { prompt, .. } => (Some(prompt.clone()), None),
        _ => (None, None),
    };
    let reject_reason = match &outcome.decision {
        ParseDecision::Reject { reason } => Some(reject_reason_wire(reason).into()),
        _ => None,
    };
    ConversationTurn {
        conversation_id: outcome.conversation_id.clone(),
        ts_ms: now_ms(),
        text: Some(outcome.text.clone()),
        // Tokens rebuild the utterance for the replay button; exports redact them (D-20).
        tokens: replay_tokens(&outcome.text),
        decision: decision_name(&outcome.decision).into(),
        reject_reason,
        speech: outcome.speech.clone(),
        speech_source: None,
        refine_band: outcome.refine_band.map(band_name),
        confidence: outcome.confidence,
        briefing: outcome.briefing,
        evidence_kinds: outcome.evidence.iter().map(|item| item.kind.clone()).take(16).collect(),
        last_names,
        confirm_prompt,
        candidate_id,
        preferred_area,
    }
}

fn band_name(band: RefineBand) -> String {
    match band {
        RefineBand::Status => "status".into(),
        RefineBand::Command => "command".into(),
        RefineBand::Prompt => "prompt".into(),
        RefineBand::Reject => "reject".into(),
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

fn reject_reason_wire(reason: &RejectReason) -> &'static str {
    match reason {
        RejectReason::EmptyInput => "empty_input",
        RejectReason::InvalidInput => "invalid_input",
        RejectReason::NoAction => "no_action",
        RejectReason::OutOfDomain => "out_of_domain",
        RejectReason::NoTarget => "no_target",
        RejectReason::Ambiguous => "ambiguous",
        RejectReason::Unsafe => "unsafe",
        RejectReason::Unsupported => "unsupported",
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
        reject_reason: None,
        speech,
        speech_source: Some(source.to_string()),
        refine_band: None,
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
    let now = now_ms();
    let mut turns: Vec<ConversationTurn> = raw
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|turn: &ConversationTurn| now.saturating_sub(turn.ts_ms) <= TTL_MS)
        .collect();
    if turns.len() > MAX_TURNS {
        turns = turns.split_off(turns.len() - MAX_TURNS);
    }
    turns
}

fn append_turn(dir: &FsPath, turn: &ConversationTurn) -> std::io::Result<()> {
    let mut line = serde_json::to_string(turn).unwrap_or_default();
    line.push('\n');
    append_confined(dir, JOURNAL_FILE, line.as_bytes())
}

fn rewrite_turns(dir: &FsPath, turns: &[ConversationTurn]) -> std::io::Result<()> {
    let mut body = String::new();
    for turn in turns {
        body.push_str(&serde_json::to_string(turn).unwrap_or_default());
        body.push('\n');
    }
    write_atomic_confined(dir, JOURNAL_FILE, body.as_bytes())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/conversations", get(list_conversations))
        .route("/api/v2/conversations/live", get(crate::io::journal_live::conversations_live))
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
#[path = "conversations_tests.rs"]
mod tests;
