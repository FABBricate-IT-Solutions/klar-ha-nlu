use super::*;
use axum::routing::{get, post};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinHandle;
use tokio::time::timeout;

const QUICK: HttpLimits = HttpLimits { head: Duration::from_millis(300), body: Duration::from_millis(300), connections: 16 };
/// Longer than any deadline in these tests, so a read that takes this long is a hang.
const HANG: Duration = Duration::from_secs(5);

struct Served {
    addr: SocketAddr,
    stop: watch::Sender<bool>,
    task: JoinHandle<io::Result<()>>,
}

async fn start(limits: HttpLimits) -> Served {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = Router::new()
        .route("/peer", get(|ConnectInfo(peer): ConnectInfo<SocketAddr>| async move { peer.ip().to_string() }))
        .route("/echo", post(|body: String| async move { body }))
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(300)).await;
                "fertig"
            }),
        );
    let (stop, stopping) = watch::channel(false);
    let task = tokio::spawn(serve(listener, router, limits, stopping));
    Served { addr, stop, task }
}

async fn connect(served: &Served, sent: &str) -> TcpStream {
    let mut stream = TcpStream::connect(served.addr).await.unwrap();
    stream.write_all(sent.as_bytes()).await.unwrap();
    stream
}

/// Everything the server sends until it closes the connection. `None` when it is still open
/// after [`HANG`].
async fn until_closed(stream: &mut TcpStream) -> Option<String> {
    let mut received = Vec::new();
    timeout(HANG, stream.read_to_end(&mut received)).await.ok()?.ok()?;
    Some(String::from_utf8_lossy(&received).into_owned())
}

const ONE_SHOT: &str = "GET /peer HTTP/1.1\r\nHost: klar.test\r\nConnection: close\r\n\r\n";

#[tokio::test]
async fn a_request_is_answered_and_the_handler_sees_the_peer() {
    let served = start(QUICK).await;
    let answer = until_closed(&mut connect(&served, ONE_SHOT).await).await.expect("answer");
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.ends_with("127.0.0.1"), "{answer}");
}

#[tokio::test]
async fn half_a_request_head_is_closed_after_the_deadline() {
    let served = start(QUICK).await;
    let began = Instant::now();
    let mut stream = connect(&served, "GET /peer HTTP/1.1\r\nHost: klar.test\r\n").await;
    let answer = until_closed(&mut stream).await.expect("the server must close a connection that never finishes its head");
    assert!(began.elapsed() >= QUICK.head, "closed before the deadline: {:?}", began.elapsed());
    assert!(!answer.starts_with("HTTP/1.1 200"), "{answer}");
}

#[tokio::test]
async fn an_idle_keep_alive_connection_is_closed_after_the_deadline() {
    let served = start(QUICK).await;
    let mut stream = connect(&served, "GET /peer HTTP/1.1\r\nHost: klar.test\r\n\r\n").await;
    let began = Instant::now();
    let answer = until_closed(&mut stream).await.expect("the server must close a connection that sends nothing more");
    assert!(answer.starts_with("HTTP/1.1 200") && answer.ends_with("127.0.0.1"), "{answer}");
    assert!(began.elapsed() >= QUICK.head, "closed before the deadline: {:?}", began.elapsed());
}

#[tokio::test]
async fn a_body_that_stalls_is_given_up_after_the_deadline() {
    let served = start(QUICK).await;
    let head = "POST /echo HTTP/1.1\r\nHost: klar.test\r\nContent-Type: text/plain\r\nContent-Length: 4096\r\n\r\nzehn bytes";
    let mut stream = connect(&served, head).await;
    let answer = until_closed(&mut stream).await.expect("the server must not wait for the rest of the body for ever");
    assert!(!answer.starts_with("HTTP/1.1 200"), "{answer}");

    // A body that arrives is still answered.
    let whole = "POST /echo HTTP/1.1\r\nHost: klar.test\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhallo";
    let answer = until_closed(&mut connect(&served, whole).await).await.expect("answer");
    assert!(answer.starts_with("HTTP/1.1 200") && answer.ends_with("hallo"), "{answer}");
}

#[tokio::test]
async fn no_more_connections_are_served_than_the_limit() {
    let served = start(HttpLimits { head: Duration::from_secs(30), body: Duration::from_secs(30), connections: 2 }).await;
    // Two connections that never finish their head hold both slots.
    let first = connect(&served, "GET /peer HTTP/1.1\r\n").await;
    let _second = connect(&served, "GET /peer HTTP/1.1\r\n").await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut third = connect(&served, ONE_SHOT).await;
    let mut byte = [0u8; 1];
    let waiting = timeout(Duration::from_millis(400), third.read(&mut byte)).await;
    assert!(waiting.is_err(), "a third connection was served beyond the limit of two");

    drop(first);
    let answer = until_closed(&mut third).await.expect("a freed slot serves the waiting connection");
    assert!(answer.contains("200 OK") || answer.ends_with("127.0.0.1"), "{answer}");
}

#[tokio::test]
async fn a_stop_finishes_the_request_under_way_and_ends_the_server() {
    let served = start(QUICK).await;
    let mut slow = connect(&served, "GET /slow HTTP/1.1\r\nHost: klar.test\r\n\r\n").await;
    let idle = connect(&served, "").await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    served.stop.send(true).unwrap();

    let answer = until_closed(&mut slow).await.expect("answer");
    assert!(answer.starts_with("HTTP/1.1 200") && answer.ends_with("fertig"), "{answer}");
    timeout(HANG, served.task).await.expect("the server ends once its connections are done").unwrap().unwrap();
    drop(idle);
    assert!(TcpStream::connect(served.addr).await.is_err(), "the port is closed after a stop");
}
