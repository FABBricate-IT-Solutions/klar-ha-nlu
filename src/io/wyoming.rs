use crate::io::auth::wyoming_allowed;
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::listen::{accept_failed, stopped};
use crate::io::state::AppState;
use crate::nlu::{legacy_result, parse_with_controls};
use crate::types::{ParseDecision, ParseOutcome};
use serde_json::{json, Value};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio::time::timeout;

const MAX_LINE: usize = 8192;
const MAX_CONNS: usize = 32;
const IDLE: Duration = Duration::from_secs(30);
const REJECT_WRITE: Duration = Duration::from_millis(250);
const NLU_DESCRIPTION: &str = "On-device NLU";

/// Serve Wyoming on `listener` until `stop` is set, then let the events that are under way
/// finish. A failed `accept` is logged and the listener carries on: it used to end the
/// listener for good, with the process still running.
pub async fn serve(listener: TcpListener, state: AppState, mut stop: watch::Receiver<bool>) -> std::io::Result<()> {
    let inflight = Arc::new(AtomicUsize::new(0));
    // Every connection holds a sender. When the last one is dropped, `recv` returns.
    let (alive, mut all_closed) = mpsc::channel::<()>(1);
    loop {
        let (stream, peer) = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok(accepted) => accepted,
                Err(err) => {
                    accept_failed("Wyoming", &err).await;
                    continue;
                }
            },
            () = stopped(&mut stop) => break,
        };
        if !should_admit(peer, inflight.load(Ordering::Relaxed)) {
            // Auth and capacity still reject; a short error is best-effort so clients do not hang.
            tokio::spawn(write_reject(stream));
            continue;
        }
        inflight.fetch_add(1, Ordering::Relaxed);
        let state = state.clone();
        let inflight = inflight.clone();
        let (stop, alive) = (stop.clone(), alive.clone());
        tokio::spawn(async move {
            if let Err(err) = handle(stream, state, stop).await {
                tracing::debug!("Wyoming-Verbindung: {err}");
            }
            inflight.fetch_sub(1, Ordering::Relaxed);
            drop(alive);
        });
    }
    drop(listener);
    drop(alive);
    let _ = all_closed.recv().await;
    Ok(())
}

fn should_admit(peer: SocketAddr, inflight: usize) -> bool {
    wyoming_allowed(peer) && inflight < MAX_CONNS
}

async fn write_reject<W: AsyncWriteExt + Unpin>(mut writer: W) {
    let _ = timeout(REJECT_WRITE, write_event(&mut writer, "error", json!({ "text": "" }))).await;
}

async fn handle(stream: TcpStream, state: AppState, stop: watch::Receiver<bool>) -> std::io::Result<()> {
    let (reader, writer) = stream.into_split();
    handle_io(BufReader::new(reader), writer, state, stop).await
}

/// Answer events until the peer closes, goes idle, or `stop` is set. A stop is only looked
/// at between events, so an event that was read is always answered.
async fn handle_io<R, W>(mut reader: R, mut writer: W, state: AppState, mut stop: watch::Receiver<bool>) -> std::io::Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWriteExt + Unpin,
{
    loop {
        let read = tokio::select! {
            read = timeout(IDLE, read_capped_line(&mut reader)) => read,
            () = stopped(&mut stop) => break,
        };
        let line = match read {
            Ok(Ok(Some(line))) => line,
            Ok(Ok(None)) => break,
            Ok(Err(err)) => return Err(err),
            Err(_) => return Err(Error::new(ErrorKind::TimedOut, "wyoming idle")),
        };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(event) = serde_json::from_str::<Value>(&line) else {
            write_not_recognized(&mut writer).await?;
            continue;
        };
        let typ = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match typ {
            "describe" => {
                let languages = state.settings.lock().await.languages.clone();
                write_event(
                    &mut writer,
                    "info",
                    json!({
                        "intent": [{
                            "name": "Klar NLU",
                            "installed": true,
                            "description": NLU_DESCRIPTION,
                            "version": env!("CARGO_PKG_VERSION"),
                            "contract_version": crate::types::PARSE_SCHEMA_VERSION,
                            "languages": languages,
                            "attribution": {
                                "name": "Klar NLU",
                                "url": "https://github.com/FABBricate-IT-Solutions/klar-ha-nlu"
                            }
                        }]
                    }),
                )
                .await?;
            }
            "recognize" => {
                let text = event.pointer("/data/text").and_then(|v| v.as_str()).unwrap_or("");
                if text.chars().count() > MAX_PARSE_CHARS
                    || text.chars().any(|character| character.is_control() && !character.is_whitespace())
                {
                    write_not_recognized(&mut writer).await?;
                    continue;
                }
                let conversation_id = event
                    .pointer("/data/conversation_id")
                    .or_else(|| event.pointer("/data/context/id"))
                    .or_else(|| event.pointer("/data/context/conversation_id"))
                    .and_then(|v| v.as_str());
                if conversation_id.is_some_and(|value| value.len() > 128) {
                    write_not_recognized(&mut writer).await?;
                    continue;
                }
                let home = state.home.snapshot().await;
                let mut settings = state.settings.lock().await.clone();
                let language = event.pointer("/data/language").and_then(|value| value.as_str());
                if let Some(raw) = language.filter(|value| !value.is_empty()) {
                    match crate::lang::pin_language(raw) {
                        Ok(tag) => settings.languages = vec![tag],
                        Err(_) => {
                            write_not_recognized(&mut writer).await?;
                            continue;
                        }
                    }
                }
                let custom = state.custom.lock().await.clone();
                let policies = state.policies.lock().await.clone();
                let speech_bank = state.speech_bank.lock().await.clone();
                let match_controls = state.match_controls.lock().await.clone();
                let mut session = {
                    let mut guard = state.sessions.lock().await;
                    guard.take(conversation_id)
                };
                // HTTP always assigns ParseIn.preferred_area (JSON includes the key). Wyoming
                // events often omit it: omitted = keep; present valid = set; present empty
                // or unknown = ignore (keep previous — safer than clearing a good satellite area).
                if let Some(area) = event
                    .pointer("/data/preferred_area")
                    .and_then(|value| value.as_str())
                    .filter(|area| !area.is_empty() && area.len() <= 128 && home.areas.iter().any(|record| record.area_id == *area))
                {
                    session.preferred_area = Some(area.to_string());
                }
                let outcome = parse_with_controls(text, &home, &mut session, &custom, &settings, &policies, &speech_bank, &match_controls);
                if let Some((entity_id, alias)) = session.pending_teach.take() {
                    state.apply_teach(&entity_id, &alias).await;
                }
                let last_names = session.last.iter().map(|turn| turn.name.clone()).collect();
                state.sessions.lock().await.put(session);
                state.record_parse("wyoming", language, &legacy_result(outcome.clone())).await;
                state.record_outcome(&outcome, last_names).await;
                let intents = match &outcome.decision {
                    ParseDecision::Execute => outcome.plan.as_ref().map_or_else(Vec::new, |plan| plan.intents()),
                    ParseDecision::Clarify { .. }
                    | ParseDecision::Confirm { .. }
                    | ParseDecision::Reject { .. }
                    | ParseDecision::Chat
                    | ParseDecision::Error { .. } => Vec::new(),
                };
                if intents.is_empty() {
                    write_event(
                        &mut writer,
                        "not-recognized",
                        json!({
                            "schema_version": outcome.schema_version,
                            "text": outcome.speech,
                            "outcome": outcome_json(&outcome),
                        }),
                    )
                    .await?;
                } else if intents.len() == 1 {
                    let mut data = intent_json(&intents[0], &outcome.speech);
                    data["schema_version"] = json!(outcome.schema_version);
                    data["outcome"] = outcome_json(&outcome);
                    write_event(&mut writer, "intent", data).await?;
                } else {
                    write_event(
                        &mut writer,
                        "intents-start",
                        json!({ "schema_version": outcome.schema_version, "outcome": outcome_json(&outcome) }),
                    )
                    .await?;
                    for (index, intent) in intents.iter().enumerate() {
                        let speech = if index + 1 == intents.len() { outcome.speech.as_str() } else { "" };
                        write_event(&mut writer, "intent", intent_json(intent, speech)).await?;
                    }
                    write_event(&mut writer, "intents-stop", json!({ "schema_version": outcome.schema_version })).await?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

async fn read_capped_line<R: AsyncBufRead + Unpin>(reader: &mut R) -> std::io::Result<Option<String>> {
    let mut buf = Vec::new();
    loop {
        let data = reader.fill_buf().await?;
        if data.is_empty() {
            return if buf.is_empty() { Ok(None) } else { Ok(Some(String::from_utf8_lossy(&buf).into_owned())) };
        }
        if let Some(pos) = data.iter().position(|&b| b == b'\n') {
            buf.extend_from_slice(&data[..=pos]);
            reader.consume(pos + 1);
            break;
        }
        let n = data.len();
        buf.extend_from_slice(data);
        reader.consume(n);
        if buf.len() > MAX_LINE {
            return Err(Error::new(ErrorKind::InvalidData, "wyoming line too long"));
        }
    }
    if buf.len() > MAX_LINE {
        return Err(Error::new(ErrorKind::InvalidData, "wyoming line too long"));
    }
    while matches!(buf.last(), Some(b'\n' | b'\r')) {
        buf.pop();
    }
    Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
}

fn intent_json(intent: &crate::types::Intent, speech: &str) -> Value {
    let entities: Vec<Value> = intent.slots.iter().map(|s| json!({"name": s.name, "value": s.value})).collect();
    json!({
        "name": intent.name,
        "entities": entities,
        "text": speech,
    })
}

fn outcome_json(outcome: &ParseOutcome) -> Value {
    json!({
        "schema_version": outcome.schema_version,
        "decision": outcome.decision,
        "confidence": outcome.confidence,
        "evidence": outcome.evidence,
        "trace": outcome.trace,
        "briefing": outcome.briefing,
    })
}

async fn write_not_recognized<W: AsyncWriteExt + Unpin>(writer: &mut W) -> std::io::Result<()> {
    write_event(writer, "not-recognized", json!({ "text": "" })).await
}

async fn write_event<W: AsyncWriteExt + Unpin>(writer: &mut W, typ: &str, data: Value) -> std::io::Result<()> {
    let event = json!({"type": typ, "data": data});
    writer.write_all(event.to_string().as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await
}

#[cfg(test)]
#[path = "wyoming_tests.rs"]
mod tests;
