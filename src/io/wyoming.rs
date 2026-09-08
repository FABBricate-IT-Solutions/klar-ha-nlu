use crate::io::auth::wyoming_allowed;
use crate::io::limits::MAX_PARSE_CHARS;
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
use tokio::time::timeout;

const MAX_LINE: usize = 8192;
const MAX_CONNS: usize = 32;
const IDLE: Duration = Duration::from_secs(30);
const REJECT_WRITE: Duration = Duration::from_millis(250);
const NLU_DESCRIPTION: &str = "On-device NLU";

pub async fn serve(bind: &str, state: AppState) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    let inflight = Arc::new(AtomicUsize::new(0));
    tracing::info!("Wyoming lauscht auf {bind}");
    loop {
        let (stream, peer) = listener.accept().await?;
        if !should_admit(peer, inflight.load(Ordering::Relaxed)) {
            // Auth and capacity still reject; a short error is best-effort so clients do not hang.
            tokio::spawn(write_reject(stream));
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

fn should_admit(peer: SocketAddr, inflight: usize) -> bool {
    wyoming_allowed(peer) && inflight < MAX_CONNS
}

async fn write_reject<W: AsyncWriteExt + Unpin>(mut writer: W) {
    let _ = timeout(REJECT_WRITE, write_event(&mut writer, "error", json!({ "text": "" }))).await;
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
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::session::Session;
    use crate::types::Settings;
    use tokio::io::{duplex, AsyncWriteExt};

    async fn read_line<R: AsyncBufRead + Unpin>(lines: &mut tokio::io::Lines<R>) -> String {
        timeout(Duration::from_secs(2), lines.next_line()).await.expect("wyoming reply").unwrap().expect("eof")
    }

    fn test_state() -> AppState {
        let dir = std::env::temp_dir().join(format!("klar-wy-{}-{}", std::process::id(), uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&dir);
        AppState::new(
            LoadedHome {
                graph: default_home(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        )
    }

    async fn exchange(state: AppState, requests: &[&str]) -> Vec<String> {
        let (client, server) = duplex(16 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, state));
        let (client_read, mut client_write) = tokio::io::split(client);
        let mut lines = BufReader::new(client_read).lines();
        let mut replies = Vec::new();
        for request in requests {
            client_write.write_all(request.as_bytes()).await.unwrap();
            if !request.ends_with('\n') {
                client_write.write_all(b"\n").await.unwrap();
            }
            client_write.flush().await.unwrap();
            replies.push(read_line(&mut lines).await);
        }
        client_write.shutdown().await.unwrap();
        drop(client_write);
        timeout(Duration::from_secs(2), task).await.expect("server exit").unwrap().unwrap();
        replies
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
                match_controls: Vec::new(),
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
        assert!(info.contains(NLU_DESCRIPTION), "{info}");
        assert!(!info.contains("Deterministische"), "{info}");
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

    #[tokio::test]
    async fn recognize_errors_write_not_recognized() {
        let oversized = format!(r#"{{"type":"recognize","data":{{"text":"{}"}}}}"#, "x".repeat(MAX_PARSE_CHARS + 1));
        let long_id = format!(r#"{{"type":"recognize","data":{{"text":"hi","conversation_id":"{}"}}}}"#, "c".repeat(129));
        let replies = exchange(
            test_state(),
            &["{not-json", &oversized, &long_id, r#"{"type":"recognize","data":{"text":"Licht im Wohnzimmer an","language":"de"}}"#],
        )
        .await;
        assert!(replies[0].contains("\"type\":\"not-recognized\""), "{}", replies[0]);
        assert!(replies[1].contains("\"type\":\"not-recognized\""), "{}", replies[1]);
        assert!(replies[2].contains("\"type\":\"not-recognized\""), "{}", replies[2]);
        assert!(replies[3].contains("HassTurnOn"), "{}", replies[3]);
    }

    async fn area_of(state: &AppState, id: &str) -> Option<String> {
        state.sessions.lock().await.get_or_create(Some(id)).preferred_area.clone()
    }

    #[tokio::test]
    async fn preferred_area_omitted_keeps_invalid_ignored() {
        let state = test_state();
        let mut seeded = Session::new();
        seeded.id = "area-1".into();
        seeded.preferred_area = Some("wohnzimmer".into());
        state.sessions.lock().await.put(seeded);

        let (client, server) = duplex(16 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, state.clone()));
        let (client_read, mut writer) = tokio::io::split(client);
        let mut lines = BufReader::new(client_read).lines();
        for request in [
            r#"{"type":"recognize","data":{"text":"Licht aus","conversation_id":"area-1","language":"de"}}"#,
            r#"{"type":"recognize","data":{"text":"Licht an","conversation_id":"area-1","language":"de","preferred_area":""}}"#,
            r#"{"type":"recognize","data":{"text":"Licht an","conversation_id":"area-1","language":"de","preferred_area":"nope"}}"#,
        ] {
            writer.write_all(request.as_bytes()).await.unwrap();
            writer.write_all(b"\n").await.unwrap();
            writer.flush().await.unwrap();
            let reply = read_line(&mut lines).await;
            assert!(reply.contains("intent") || reply.contains("not-recognized"), "{reply}");
            assert_eq!(area_of(&state, "area-1").await.as_deref(), Some("wohnzimmer"));
        }

        writer
            .write_all(
                br#"{"type":"recognize","data":{"text":"Licht an","conversation_id":"area-1","language":"de","preferred_area":"kuche"}}"#,
            )
            .await
            .unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
        let set = read_line(&mut lines).await;
        assert!(set.contains("intent") || set.contains("not-recognized"), "{set}");
        assert_eq!(area_of(&state, "area-1").await.as_deref(), Some("kuche"));

        writer.shutdown().await.unwrap();
        drop(writer);
        timeout(Duration::from_secs(2), task).await.expect("server exit").unwrap().unwrap();
    }

    #[test]
    fn admit_keeps_auth_and_cap() {
        assert!(should_admit("127.0.0.1:9".parse().unwrap(), 0));
        assert!(should_admit("172.30.32.2:9".parse().unwrap(), 0));
        assert!(!should_admit("10.1.2.3:9".parse().unwrap(), 0));
        assert!(!should_admit("127.0.0.1:9".parse().unwrap(), MAX_CONNS));
    }

    #[tokio::test]
    async fn reject_writes_error_before_close() {
        let (client, server) = duplex(1024);
        let task = tokio::spawn(write_reject(server));
        let mut lines = BufReader::new(client).lines();
        let line = read_line(&mut lines).await;
        assert!(line.contains("\"type\":\"error\""), "{line}");
        timeout(Duration::from_secs(2), task).await.expect("reject").unwrap();
    }
}
