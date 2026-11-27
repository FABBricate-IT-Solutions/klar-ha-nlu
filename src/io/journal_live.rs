//! Live journal stream for operators (package 6.7): new turns without a page reload.

use crate::io::auth::reads_allowed;
use crate::io::conversations::ConversationTurn;
use crate::io::state::AppState;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug, Clone, Serialize)]
struct LiveEvent<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    turn: ConversationTurn,
}

pub async fn conversations_live(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<axum::response::Response, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let mut rx = state.journal.subscribe();
    let snapshot = state.journal.list().into_iter().next_back();
    let (tx, out) = mpsc::unbounded_channel::<Result<Event, Infallible>>();
    tokio::spawn(async move {
        if let Some(turn) = snapshot {
            if tx.send(Ok(json_turn("snapshot", turn))).is_err() {
                return;
            }
        }
        loop {
            match rx.recv().await {
                Ok(turn) => {
                    if tx.send(Ok(json_turn("turn", turn))).is_err() {
                        break;
                    }
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    });
    Ok(Sse::new(UnboundedReceiverStream::new(out)).keep_alive(KeepAlive::default()).into_response())
}

fn json_turn(kind: &str, turn: ConversationTurn) -> Event {
    let payload = LiveEvent { kind, turn };
    Event::default().json_data(&payload).unwrap_or_else(|_| Event::default().data("{}"))
}

#[cfg(test)]
mod tests {
    use crate::home::test_dir::TestDir;
    use crate::io::conversations::ConversationJournal;
    use crate::types::{ParseDecision, ParseOutcome};

    #[tokio::test]
    async fn append_reaches_live_subscriber() {
        let scratch = TestDir::new("journal-live");
        let journal = ConversationJournal::open(scratch.path());
        let mut rx = journal.subscribe();
        let outcome = ParseOutcome {
            schema_version: "2.0".into(),
            text: "licht an".into(),
            conversation_id: "c-live".into(),
            decision: ParseDecision::Chat,
            speech: String::new(),
            confidence: 0.1,
            margin: 0.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            news_topic: None,
            retrieval: None,
            policy_trace: None,
            clock_answer: false,
            quiet_ack_eligible: false,
            refine_band: None,
            teach: None,
        };
        journal.append(crate::io::conversations::turn_from_outcome(&outcome, Vec::new(), None));
        let got = rx.recv().await.expect("live turn");
        assert_eq!(got.conversation_id, "c-live");
        assert_eq!(got.text.as_deref(), Some("licht an"));
    }
}
