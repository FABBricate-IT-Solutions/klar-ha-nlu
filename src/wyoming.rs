use crate::parse::parse;
use crate::session::Sessions;
use crate::types::{HomeGraph, Settings};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
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
                                "url": "https://github.com/klar-nlu/klar"
                            }
                        }]
                    }),
                )
                .await?;
            }
            "recognize" => {
                let text = event
                    .pointer("/data/text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let home = home.lock().await.clone();
                let settings = settings.lock().await.clone();
                let custom = custom.lock().await.clone();
                let mut sessions = sessions.lock().await;
                let session = sessions.get_or_create(None);
                let result = parse(&text, &home, session, &custom, &settings);
                if result.intents.is_empty() {
                    write_event(
                        &mut writer,
                        "not-recognized",
                        json!({ "text": result.speech }),
                    )
                    .await?;
                } else if result.intents.len() == 1 {
                    let intent = &result.intents[0];
                    write_event(&mut writer, "intent", intent_json(intent, &result.speech)).await?;
                } else {
                    write_event(&mut writer, "intents-start", json!({})).await?;
                    for (i, intent) in result.intents.iter().enumerate() {
                        let speech = if i + 1 == result.intents.len() {
                            result.speech.as_str()
                        } else {
                            ""
                        };
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
    let entities: Vec<Value> = intent
        .slots
        .iter()
        .map(|s| json!({"name": s.name, "value": s.value}))
        .collect();
    json!({
        "name": intent.name,
        "entities": entities,
        "text": speech
    })
}

async fn write_event<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    typ: &str,
    data: Value,
) -> std::io::Result<()> {
    let event = json!({"type": typ, "data": data});
    writer.write_all(event.to_string().as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await
}
