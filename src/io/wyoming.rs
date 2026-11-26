use crate::io::auth::wyoming_allowed;
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::listen::{accept_failed, stopped};
use crate::io::parse_run::{run_parse, ParseRequest};
use crate::io::state::AppState;
use crate::nlu::legacy_result;
use crate::ops;
use crate::types::{ParseDecision, ParseOutcome};
use serde_json::{json, Value};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio::time::timeout;

const MAX_LINE: usize = 8192;
/// Cap on Wyoming `data_length` (JSON blob after the header). Intent text fits well under this.
const MAX_DATA: usize = 16 * 1024;
/// Cap on Wyoming `payload_length`. NLU ignores the payload but must consume it to stay in sync.
const MAX_PAYLOAD: usize = 64 * 1024;
const MAX_CONNS: usize = 32;
const IDLE: Duration = Duration::from_secs(30);
const WRITE: Duration = Duration::from_secs(2);
const REJECT_WRITE: Duration = Duration::from_millis(250);
const NLU_DESCRIPTION: &str = "On-device NLU";

/// Decrements the in-flight connection counter when dropped (panic, cancel, or normal exit).
struct InflightGuard(Arc<AtomicUsize>);

impl Drop for InflightGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
        ops::ops().record_wyoming_close();
    }
}

/// Serve Wyoming on `listener` until `stop` is set, then let the events that are under way
/// finish. A failed `accept` is logged and the listener carries on: it used to end the
/// listener for good, with the process still running.
pub async fn serve(listener: TcpListener, state: AppState, mut stop: watch::Receiver<bool>) -> std::io::Result<()> {
    state.wyoming_up.store(true, Ordering::Relaxed);
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
            ops::ops().record_wyoming_reject();
            tokio::spawn(write_reject(stream));
            continue;
        }
        inflight.fetch_add(1, Ordering::Relaxed);
        ops::ops().record_wyoming_connect();
        let state = state.clone();
        let inflight = inflight.clone();
        let (stop, alive) = (stop.clone(), alive.clone());
        tokio::spawn(async move {
            let _guard = InflightGuard(inflight);
            let _alive = alive;
            if let Err(err) = handle(stream, state, stop).await {
                tracing::debug!("Wyoming-Verbindung: {err}");
            }
        });
    }
    drop(listener);
    drop(alive);
    let _ = all_closed.recv().await;
    state.wyoming_up.store(false, Ordering::Relaxed);
    Ok(())
}

fn should_admit(peer: SocketAddr, inflight: usize) -> bool {
    wyoming_allowed(peer) && inflight < MAX_CONNS
}

async fn write_reject<W: AsyncWriteExt + Unpin>(mut writer: W) {
    let _ = timeout(REJECT_WRITE, write_event_raw(&mut writer, "error", json!({ "text": "" }))).await;
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
            read = timeout(IDLE, read_event(&mut reader)) => read,
            () = stopped(&mut stop) => break,
        };
        let event = match read {
            Ok(Ok(Some(event))) => event,
            Ok(Ok(None)) => break,
            Ok(Err(err)) => return Err(err),
            Err(_) => return Err(Error::new(ErrorKind::TimedOut, "wyoming idle")),
        };
        let typ = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match typ {
            "describe" => {
                let languages = describe_languages(&state).await;
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
            "ping" => {
                // Echo optional text so peers can match request/response (Wyoming ping/pong).
                let text = event.pointer("/data/text").cloned().unwrap_or_else(|| json!(""));
                write_event(&mut writer, "pong", json!({ "text": text })).await?;
            }
            "__bad_json__" => {
                write_not_recognized(&mut writer).await?;
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
                // Seed follow-ups with the satellite area already set (P-23). Wyoming events
                // often omit preferred_area: omitted = keep; present valid = set; present empty
                // or unknown = ignore (keep previous — safer than clearing a good satellite area).
                let satellite = event
                    .pointer("/data/preferred_area")
                    .and_then(|value| value.as_str())
                    .filter(|area| !area.is_empty() && area.len() <= 128 && home.areas.iter().any(|record| record.area_id == *area));
                let mut session = {
                    let mut guard = state.sessions.lock().await;
                    match satellite {
                        Some(area) => guard.take_with_area(conversation_id, area),
                        None => guard.take(conversation_id),
                    }
                };
                if let Some(area) = satellite {
                    session.preferred_area = Some(area.to_string());
                }
                let parsed = match run_parse(
                    &state.parse_limiter,
                    ParseRequest { text: text.to_string(), home, session, custom, settings, policies, speech_bank, match_controls },
                )
                .await
                {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        write_not_recognized(&mut writer).await?;
                        continue;
                    }
                };
                let mut outcome = parsed.outcome;
                let mut session = parsed.session;
                // D-18: Wyoming has no write token; never persist teach here.
                if let Some((entity_id, alias)) = session.pending_teach.take() {
                    outcome.teach = Some(crate::types::TeachProposal { entity_id, alias });
                }
                let last_names = session.last.iter().map(|turn| turn.name.clone()).collect();
                {
                    let mut sessions = state.sessions.lock().await;
                    sessions.put(session);
                    ops::ops().sync_sessions(sessions.created(), sessions.len() as u64);
                }
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

/// Languages advertised in `info`: configured list, or the default pack when empty (R-04).
async fn describe_languages(state: &AppState) -> Vec<String> {
    let settings = state.settings.lock().await;
    if settings.languages.is_empty() {
        vec![settings.default_pack().to_string()]
    } else {
        settings.languages.clone()
    }
}

/// Read one Wyoming event: JSONL header plus optional `data_length` / `payload_length` blobs.
/// Library clients put event fields in the data blob, not inline in the header (R-04).
async fn read_event<R: AsyncBufRead + Unpin>(reader: &mut R) -> std::io::Result<Option<Value>> {
    loop {
        let Some(line) = read_capped_line(reader).await? else {
            return Ok(None);
        };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(mut event) = serde_json::from_str::<Value>(&line) else {
            // Malformed header: use a private type the handler maps to not-recognized.
            return Ok(Some(json!({ "type": "__bad_json__" })));
        };
        let data_length = event.get("data_length").and_then(Value::as_u64).unwrap_or(0) as usize;
        let payload_length = event.get("payload_length").and_then(Value::as_u64).unwrap_or(0) as usize;
        if data_length > MAX_DATA || payload_length > MAX_PAYLOAD {
            return Err(Error::new(ErrorKind::InvalidData, "wyoming frame too large"));
        }
        if data_length > 0 {
            let mut buf = vec![0u8; data_length];
            reader.read_exact(&mut buf).await?;
            let extra: Value =
                serde_json::from_slice(&buf).map_err(|err| Error::new(ErrorKind::InvalidData, format!("wyoming data: {err}")))?;
            merge_event_data(&mut event, extra);
        }
        if payload_length > 0 {
            discard_exact(reader, payload_length).await?;
        }
        // Drop framing keys; handlers only look at type/data.
        if let Some(obj) = event.as_object_mut() {
            obj.remove("data_length");
            obj.remove("payload_length");
        }
        return Ok(Some(event));
    }
}

fn merge_event_data(event: &mut Value, extra: Value) {
    let Some(extra_obj) = extra.as_object() else {
        return;
    };
    let data = event.as_object_mut().expect("event object").entry("data").or_insert_with(|| json!({}));
    match data {
        Value::Object(existing) => {
            for (key, value) in extra_obj {
                existing.insert(key.clone(), value.clone());
            }
        }
        _ => {
            *data = Value::Object(extra_obj.clone());
        }
    }
}

async fn discard_exact<R: AsyncRead + Unpin>(reader: &mut R, mut n: usize) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    while n > 0 {
        let chunk = n.min(buf.len());
        reader.read_exact(&mut buf[..chunk]).await?;
        n -= chunk;
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
    match timeout(WRITE, write_event_raw(writer, typ, data)).await {
        Ok(result) => result,
        Err(_) => Err(Error::new(ErrorKind::TimedOut, "wyoming write")),
    }
}

async fn write_event_raw<W: AsyncWriteExt + Unpin>(writer: &mut W, typ: &str, data: Value) -> std::io::Result<()> {
    let event = json!({"type": typ, "data": data});
    writer.write_all(event.to_string().as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await
}

#[cfg(test)]
#[path = "wyoming_tests.rs"]
mod tests;
