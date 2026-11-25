use super::*;
use crate::home::test_dir::TestDir;
use crate::home::{default_home, LoadedHome};
use crate::session::Session;
use crate::types::Settings;
use std::io::ErrorKind;
use tokio::io::{duplex, AsyncWriteExt};

/// A stop flag nobody sets.
fn never_stopped() -> watch::Receiver<bool> {
    watch::channel(false).1
}

async fn read_line<R: AsyncBufRead + Unpin>(lines: &mut tokio::io::Lines<R>) -> String {
    timeout(Duration::from_secs(2), lines.next_line()).await.expect("wyoming reply").unwrap().expect("eof")
}

/// The state and the scratch directory it writes to; keep the directory alive as long as the state.
fn test_state() -> (TestDir, AppState) {
    let scratch = TestDir::new("wy");
    let dir = scratch.path().to_path_buf();
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
    (scratch, state)
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
    let (_scratch, state) = test_state();
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
    // R-04: IntentModel.languages must be present and non-empty even with default settings.
    assert!(info.contains("\"languages\":["), "{info}");
    assert!(!info.contains("\"languages\":[]"), "{info}");

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
    let (_scratch, state) = test_state();
    let replies = exchange(
        state,
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
    let (_scratch, state) = test_state();
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
    let (_scratch, state) = test_state();
    let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, state, stopping));
    let (client_read, mut client_write) = tokio::io::split(client);
    let mut lines = BufReader::new(client_read).lines();
    client_write.write_all(b"{\"type\":\"describe\"}\n").await.unwrap();
    assert!(read_line(&mut lines).await.contains("\"type\":\"info\""));

    // The peer keeps the connection open and sends nothing. Without a stop this waits out `IDLE`.
    stop.send(true).unwrap();
    timeout(Duration::from_secs(2), task).await.expect("the handler ends on a stop").unwrap().unwrap();
}

/// Reference Wyoming library framing: data lives in a `data_length` blob after the header (R-04).
fn library_framed(typ: &str, data: &str) -> Vec<u8> {
    let mut out = format!(r#"{{"type":"{typ}","version":"1.5.0","data_length":{},"payload_length":0}}"#, data.len()).into_bytes();
    out.push(b'\n');
    out.extend_from_slice(data.as_bytes());
    out
}

#[tokio::test]
async fn library_framing_recognize_returns_hass_turn_on() {
    let (_scratch, state) = test_state();
    let (client, server) = duplex(16 * 1024);
    let (server_read, server_write) = tokio::io::split(server);
    let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, state, never_stopped()));
    let (client_read, mut writer) = tokio::io::split(client);
    let mut lines = BufReader::new(client_read).lines();

    let framed = library_framed("recognize", r#"{"text":"Licht im Wohnzimmer an","language":"de"}"#);
    writer.write_all(&framed).await.unwrap();
    writer.flush().await.unwrap();
    let reply = read_line(&mut lines).await;
    assert!(reply.contains("HassTurnOn"), "acceptance: framed recognize → HassTurnOn; got {reply}");
    assert!(reply.contains("\"type\":\"intent\""), "{reply}");

    writer.shutdown().await.unwrap();
    drop(writer);
    timeout(Duration::from_secs(2), task).await.expect("server exit").unwrap().unwrap();
}

#[tokio::test]
async fn ping_answers_pong_with_echoed_text() {
    let (_scratch, state) = test_state();
    let replies = exchange(state, &[r#"{"type":"ping","data":{"text":"probe-1"}}"#]).await;
    assert!(replies[0].contains("\"type\":\"pong\""), "{}", replies[0]);
    assert!(replies[0].contains("probe-1"), "{}", replies[0]);
}

#[tokio::test]
async fn library_framing_ping_and_payload_skip_stay_in_sync() {
    let (_scratch, state) = test_state();
    let (client, server) = duplex(16 * 1024);
    let (server_read, server_write) = tokio::io::split(server);
    let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, state, never_stopped()));
    let (client_read, mut writer) = tokio::io::split(client);
    let mut lines = BufReader::new(client_read).lines();

    // Ping with data blob + a tiny payload that must be consumed before the next event.
    let data = r#"{"text":"sync"}"#;
    let payload = b"xxxx";
    let header = format!(r#"{{"type":"ping","version":"1.5.0","data_length":{},"payload_length":{}}}"#, data.len(), payload.len());
    writer.write_all(header.as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.write_all(data.as_bytes()).await.unwrap();
    writer.write_all(payload).await.unwrap();
    writer.flush().await.unwrap();
    let pong = read_line(&mut lines).await;
    assert!(pong.contains("\"type\":\"pong\""), "{pong}");
    assert!(pong.contains("sync"), "{pong}");

    // Next event still frames correctly after the payload was skipped.
    let framed = library_framed("recognize", r#"{"text":"Licht im Wohnzimmer an","language":"de"}"#);
    writer.write_all(&framed).await.unwrap();
    writer.flush().await.unwrap();
    let reply = read_line(&mut lines).await;
    assert!(reply.contains("HassTurnOn"), "{reply}");

    writer.shutdown().await.unwrap();
    drop(writer);
    timeout(Duration::from_secs(2), task).await.expect("server exit").unwrap().unwrap();
}

#[tokio::test]
async fn write_times_out_when_peer_does_not_read() {
    // Tiny duplex buffer so a describe reply blocks once the peer never reads (R-05).
    let (client, server) = duplex(64);
    let (server_read, server_write) = tokio::io::split(server);
    let (_scratch, state) = test_state();
    let task = tokio::spawn(handle_io(BufReader::new(server_read), server_write, state, never_stopped()));
    let (_client_read, mut client_write) = tokio::io::split(client);
    client_write.write_all(br#"{"type":"describe"}"#).await.unwrap();
    client_write.write_all(b"\n").await.unwrap();
    client_write.flush().await.unwrap();
    // Keep the writer half open so the server stays blocked on write until WRITE fires.
    let result = timeout(Duration::from_secs(5), task).await.expect("handler ends on write timeout");
    let err = result.expect("join").expect_err("write timeout closes the handler");
    assert_eq!(err.kind(), ErrorKind::TimedOut, "{err}");
}

#[test]
fn inflight_guard_decrements_on_drop() {
    let counter = Arc::new(AtomicUsize::new(1));
    {
        let _guard = InflightGuard(counter.clone());
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
    assert_eq!(counter.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn write_timeout_frees_admit_capacity_for_next_peer() {
    // R-05: 32 duplex peers that never read block on write; after WRITE the counter drops
    // and a fresh peer is admitted.
    let inflight = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::new();
    let mut holders = Vec::new();
    for _ in 0..MAX_CONNS {
        let (client, server) = duplex(64);
        let (server_read, server_write) = tokio::io::split(server);
        let (_scratch, state) = test_state();
        inflight.fetch_add(1, Ordering::Relaxed);
        let guard_counter = inflight.clone();
        tasks.push(tokio::spawn(async move {
            let _guard = InflightGuard(guard_counter);
            handle_io(BufReader::new(server_read), server_write, state, never_stopped()).await
        }));
        let (_client_read, mut client_write) = tokio::io::split(client);
        client_write.write_all(br#"{"type":"describe"}"#).await.unwrap();
        client_write.write_all(b"\n").await.unwrap();
        let _ = client_write.flush().await;
        holders.push(client_write);
    }

    assert!(!should_admit("127.0.0.1:9".parse().unwrap(), inflight.load(Ordering::Relaxed)));

    for task in tasks {
        let joined = timeout(Duration::from_secs(5), task).await.expect("write timeout");
        joined.expect("join").expect_err("handler ends with write timeout");
    }
    assert_eq!(inflight.load(Ordering::Relaxed), 0);
    assert!(should_admit("127.0.0.1:9".parse().unwrap(), inflight.load(Ordering::Relaxed)));
    drop(holders);
}
