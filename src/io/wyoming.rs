use crate::io::auth::wyoming_allowed;
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::state::AppState;
use crate::nlu::{legacy_result, parse_with_policies};
use crate::types::{ParseDecision, ParseOutcome};
use serde_json::{json, Value};
use std::io::{Error, ErrorKind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

const MAX_LINE: usize = 8192;
const MAX_CONNS: usize = 32;
const IDLE: Duration = Duration::from_secs(30);

pub async fn serve(bind: &str, state: AppState) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    let inflight = Arc::new(AtomicUsize::new(0));
    tracing::info!("Wyoming lauscht auf {bind}");
    loop {
        let (stream, peer) = listener.accept().await?;
        if !wyoming_allowed(peer) {
            continue;
        }
        if inflight.load(Ordering::Relaxed) >= MAX_CONNS {
            continue;
        }
        inflight.fetch_add(1, Ordering::Relaxed);
        let state = state.clone();
        let inflight = inflight.clone();
        tokio::spawn(async move {
            if let Err(err) = handle(stream, state).await {
                tracing::debug!("Wyoming-Verbindung: {err}");
            }
            inflight.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

async fn handle(stream: TcpStream, state: AppState) -> std::io::Result<()> {
    let (reader, writer) = stream.into_split();
    handle_io(BufReader::new(reader), writer, state).await
}

async fn handle_io<R, W>(mut reader: R, mut writer: W, state: AppState) -> std::io::Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWriteExt + Unpin,
{
    loop {
        let line = match timeout(IDLE, read_capped_line(&mut reader)).await {
            Ok(Ok(Some(line))) => line,
            Ok(Ok(None)) => break,
            Ok(Err(err)) => return Err(err),
            Err(_) => return Err(Error::new(ErrorKind::TimedOut, "wyoming idle")),
        };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(event) = serde_json::from_str::<Value>(&line) else {
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
                            "description": "Deterministische deutsche NLU",
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
                    continue;
                }
                let conversation_id = event
                    .pointer("/data/conversation_id")
                    .or_else(|| event.pointer("/data/context/id"))
                    .or_else(|| event.pointer("/data/context/conversation_id"))
                    .and_then(|v| v.as_str());
                if conversation_id.is_some_and(|value| value.len() > 128) {
                    continue;
                }
                let home = state.home.snapshot().await;
                let mut settings = state.settings.lock().await.clone();
                let language = event.pointer("/data/language").and_then(|value| value.as_str());
                if let Some(raw) = language.filter(|value| !value.is_empty()) {
                    match crate::lang::pin_language(raw) {
                        Ok(tag) => settings.languages = vec![tag],
                        Err(_) => {
                            write_event(&mut writer, "not-recognized", json!({ "text": "" })).await?;
                            continue;
                        }
                    }
                }
                let custom = state.custom.lock().await.clone();
                let policies = state.policies.lock().await.clone();
                let speech_bank = state.speech_bank.lock().await.clone();
                let mut session = {
                    let mut guard = state.sessions.lock().await;
                    guard.take(conversation_id)
                };
                if let Some(area) = event.pointer("/data/preferred_area").and_then(|value| value.as_str()) {
                    if area.len() <= 128 && home.areas.iter().any(|record| record.area_id == area) {
                        session.preferred_area = Some(area.to_string());
                    }
                }
                let outcome = parse_with_policies(text, &home, &mut session, &custom, &settings, &policies, &speech_bank);
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

async fn write_event<W: AsyncWriteExt + Unpin>(writer: &mut W, typ: &str, data: Value) -> std::io::Result<()> {
    let event = json!({"type": typ, "data": data});
    writer.write_all(event.to_string().as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::types::Settings;
    use tokio::io::AsyncWriteExt;

    async fn read_line(lines: &mut tokio::io::Lines<BufReader<tokio::net::tcp::OwnedReadHalf>>) -> String {
        timeout(Duration::from_secs(2), lines.next_line()).await.expect("wyoming reply").unwrap().expect("eof")
    }

    #[tokio::test]
    async fn describe_and_recognize_reuse_conversation() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let dir = std::env::temp_dir().join(format!("klar-wy-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let state = AppState::new(
            LoadedHome {
                graph: default_home(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
            },
            dir,
            None,
        );
        let task = tokio::spawn({
            let state = state.clone();
            async move {
                let (stream, _) = listener.accept().await.unwrap();
                handle(stream, state).await.unwrap();
            }
        });

        let stream = TcpStream::connect(addr).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        writer.write_all(br#"{"type":"describe"}"#).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
        let info = read_line(&mut lines).await;
        assert!(info.contains("Klar NLU"), "{info}");
        assert!(info.contains("\"type\":\"info\""), "{info}");
        assert!(info.contains("\"contract_version\":\"2.0\""), "{info}");

        writer
            .write_all(br#"{"type":"recognize","data":{"text":"Licht im Wohnzimmer an","conversation_id":"c1","language":"de"}}"#)
            .await
            .unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
        let first = read_line(&mut lines).await;
        assert!(first.contains("HassTurnOn"), "{first}");
        assert!(first.contains("\"schema_version\":\"2.0\""), "{first}");
        assert!(first.contains("\"outcome\":"), "{first}");

        writer.write_all(br#"{"type":"recognize","data":{"text":"aus","conversation_id":"c1","language":"de"}}"#).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
        let second = read_line(&mut lines).await;
        assert!(second.contains("HassTurnOff") || second.contains("intent"), "{second}");

        writer
            .write_all(
                r#"{"type":"recognize","data":{"text":"Wohnungstür abschließen","conversation_id":"confirm-1","language":"de"}}"#
                    .as_bytes(),
            )
            .await
            .unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
        let confirmation = read_line(&mut lines).await;
        assert!(confirmation.contains("\"type\":\"not-recognized\""), "{confirmation}");
        assert!(confirmation.contains("\"type\":\"confirm\""), "{confirmation}");
        assert!(!confirmation.contains("HassTurnOn"), "{confirmation}");
        for forbidden in ["\"plan\"", "\"intent\"", "\"slots\"", "\"selected_candidate_id\""] {
            assert!(!confirmation.contains(forbidden), "confirmation leaked {forbidden}: {confirmation}");
        }

        writer.write_all(br#"{"type":"recognize","data":{"text":"ja","conversation_id":"confirm-1","language":"de"}}"#).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
        let affirmed = read_line(&mut lines).await;
        assert!(affirmed.contains("\"type\":\"intent\""), "{affirmed}");
        assert!(affirmed.contains("HassTurnOn"), "{affirmed}");

        drop(writer);
        timeout(Duration::from_secs(2), task).await.expect("server exit").unwrap();
        let last: Vec<String> = state.sessions.lock().await.get_or_create(Some("c1")).last_entities().map(str::to_string).collect();
        assert!(last.iter().any(|id| id.starts_with("light.")), "{last:?}");
    }
}
