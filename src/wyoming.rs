use crate::parse::parse;
use crate::session::Sessions;
use crate::types::{HomeGraph, Settings};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub async fn serve(
    bind: &str,
    home: Arc<Mutex<HomeGraph>>,
    sessions: Arc<Mutex<Sessions>>,
    settings: Arc<Mutex<Settings>>,
    custom: Arc<Mutex<Vec<crate::types::CustomSentence>>>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    tracing::info!("Wyoming lauscht auf {bind}");
    loop {
        let (stream, _) = listener.accept().await?;
        let home = home.clone();
        let sessions = sessions.clone();
        let settings = settings.clone();
        let custom = custom.clone();
        tokio::spawn(async move {
            if let Err(err) = handle(stream, home, sessions, settings, custom).await {
                tracing::debug!("Wyoming-Verbindung: {err}");
            }
        });
    }
}

async fn handle(
    stream: TcpStream,
    home: Arc<Mutex<HomeGraph>>,
    sessions: Arc<Mutex<Sessions>>,
    settings: Arc<Mutex<Settings>>,
    custom: Arc<Mutex<Vec<crate::types::CustomSentence>>>,
) -> std::io::Result<()> {
    let (reader, writer) = stream.into_split();
    handle_io(BufReader::new(reader), writer, home, sessions, settings, custom).await
}

async fn handle_io<R, W>(
    reader: R,
    mut writer: W,
    home: Arc<Mutex<HomeGraph>>,
    sessions: Arc<Mutex<Sessions>>,
    settings: Arc<Mutex<Settings>>,
    custom: Arc<Mutex<Vec<crate::types::CustomSentence>>>,
) -> std::io::Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(event) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let typ = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match typ {
            "describe" => {
                let languages = settings.lock().await.languages.clone();
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
                let text = event.pointer("/data/text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let conversation_id = event
                    .pointer("/data/conversation_id")
                    .or_else(|| event.pointer("/data/context/id"))
                    .or_else(|| event.pointer("/data/context/conversation_id"))
                    .and_then(|v| v.as_str());
                let home = home.lock().await.clone();
                let settings = settings.lock().await.clone();
                let custom = custom.lock().await.clone();
                let result = {
                    let mut sessions = sessions.lock().await;
                    let session = sessions.get_or_create(conversation_id);
                    parse(&text, &home, session, &custom, &settings)
                };
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
    use crate::lexicon::default_home;
    use crate::types::CustomSentence;
    use tokio::io::AsyncWriteExt;
    use tokio::time::{timeout, Duration};

    async fn read_line(lines: &mut tokio::io::Lines<BufReader<tokio::net::tcp::OwnedReadHalf>>) -> String {
        timeout(Duration::from_secs(2), lines.next_line()).await.expect("wyoming reply").unwrap().expect("eof")
    }

    #[tokio::test]
    async fn describe_and_recognize_reuse_conversation() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let home = Arc::new(Mutex::new(default_home()));
        let sessions = Arc::new(Mutex::new(Sessions::default()));
        let settings = Arc::new(Mutex::new(Settings::default()));
        let custom = Arc::new(Mutex::new(Vec::<CustomSentence>::new()));
        let task = tokio::spawn({
            let home = home.clone();
            let sessions = sessions.clone();
            let settings = settings.clone();
            async move {
                let (stream, _) = listener.accept().await.unwrap();
                handle(stream, home, sessions, settings, custom).await.unwrap();
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
        let last = sessions.lock().await.get_or_create(Some("c1")).last_entities.clone();
        assert!(last.iter().any(|id| id.starts_with("light.")), "{last:?}");
    }
}
