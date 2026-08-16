use crate::io::auth::wyoming_allowed;
use crate::io::limits::MAX_PARSE_CHARS;
use crate::io::state::AppState;
use crate::parse::parse;
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
                if text.chars().count() > MAX_PARSE_CHARS {
                    continue;
                }
                let conversation_id = event
                    .pointer("/data/conversation_id")
                    .or_else(|| event.pointer("/data/context/id"))
                    .or_else(|| event.pointer("/data/context/conversation_id"))
                    .and_then(|v| v.as_str());
                let home = state.home.snapshot().await;
                let settings = state.settings.lock().await.clone();
                let custom = state.custom.lock().await.clone();
                let mut session = {
                    let mut guard = state.sessions.lock().await;
                    guard.take(conversation_id)
                };
                let result = parse(text, &home, &mut session, &custom, &settings);
                state.sessions.lock().await.put(session);
                state.record_parse("wyoming", None, &result).await;
                if result.intents.is_empty() {
                    write_event(&mut writer, "not-recognized", json!({ "text": result.speech })).await?;
                } else if result.intents.len() == 1 {
                    let intent = &result.intents[0];
                    write_event(&mut writer, "intent", intent_json(intent, &result.speech)).await?;
                } else {
                    write_event(&mut writer, "intents-start", json!({})).await?;
                    for (i, intent) in result.intents.iter().enumerate() {
                        let speech = if i + 1 == result.intents.len() { result.speech.as_str() } else { "" };
                        write_event(&mut writer, "intent", intent_json(intent, speech)).await?;
                    }
                    write_event(&mut writer, "intents-stop", json!({})).await?;
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
        "text": speech
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
            LoadedHome { graph: default_home(), settings: Settings::default(), custom: Vec::new() },
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

        writer.write_all(br#"{"type":"recognize","data":{"text":"Licht im Wohnzimmer an","conversation_id":"c1"}}"#).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
        let first = read_line(&mut lines).await;
        assert!(first.contains("HassTurnOn"), "{first}");

        writer.write_all(br#"{"type":"recognize","data":{"text":"aus","conversation_id":"c1"}}"#).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
        let second = read_line(&mut lines).await;
        assert!(second.contains("HassTurnOff") || second.contains("intent"), "{second}");

        drop(writer);
        timeout(Duration::from_secs(2), task).await.expect("server exit").unwrap();
        let last: Vec<String> = state.sessions.lock().await.get_or_create(Some("c1")).last_entities().map(str::to_string).collect();
        assert!(last.iter().any(|id| id.starts_with("light.")), "{last:?}");
    }
}
