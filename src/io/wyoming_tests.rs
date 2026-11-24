use super::*;
use crate::home::{default_home, LoadedHome};
use crate::session::Session;
use crate::types::Settings;
use tokio::io::{duplex, AsyncWriteExt};

/// A stop flag nobody sets.
fn never_stopped() -> watch::Receiver<bool> {
    watch::channel(false).1
}

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
    let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, state, never_stopped()));
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
            handle(stream, state, never_stopped()).await.unwrap();
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
            r#"{"type":"recognize","data":{"text":"Wohnungstür abschließen","conversation_id":"confirm-1","language":"de"}}"#.as_bytes(),
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
    let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, state.clone(), never_stopped()));
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

#[tokio::test]
async fn a_stop_ends_an_idle_connection_after_answering_what_was_sent() {
    let (client, server) = duplex(16 * 1024);
    let (server_read, server_write) = tokio::io::split(server);
    let (stop, stopping) = watch::channel(false);
    let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, test_state(), stopping));
    let (client_read, mut client_write) = tokio::io::split(client);
    let mut lines = BufReader::new(client_read).lines();
    client_write.write_all(b"{\"type\":\"describe\"}\n").await.unwrap();
    assert!(read_line(&mut lines).await.contains("\"type\":\"info\""));

    // The peer keeps the connection open and sends nothing. Without a stop this waits out `IDLE`.
    stop.send(true).unwrap();
    timeout(Duration::from_secs(2), task).await.expect("the handler ends on a stop").unwrap().unwrap();
}
