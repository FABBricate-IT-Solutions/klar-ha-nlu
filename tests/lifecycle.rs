//! The two listeners start together or not at all, a stop signal ends the engine in order,
//! a failed `accept` does not end a listener, and HTTP connections have deadlines.
mod state_support;

use state_support::process::{self, Launch, Running};
use state_support::TempDir;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

const DESCRIBE: Duration = Duration::from_secs(3);
/// The acceptance bound: the engine is gone this long after a stop signal.
const STOP_BOUND: Duration = Duration::from_secs(2);

fn loopback(port: u16) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// Start with one port already taken by this test and see how the start ends.
fn start_with_a_taken_port(tag: &str, taken_is_http: bool) {
    let root = TempDir::new(tag);
    let holder = TcpListener::bind("127.0.0.1:0").unwrap();
    let taken = holder.local_addr().unwrap();
    let free = loopback(process::free_port());
    let (http, wyoming) = if taken_is_http { (taken, free) } else { (free, taken) };

    let mut running = process::spawn_on(&root, http, wyoming, &[], Launch::Direct);
    let status = running.wait_exit(Duration::from_secs(15));
    let log = running.log();
    let status = status.unwrap_or_else(|| panic!("the engine kept running with one of its two ports taken:\n{log}"));
    assert_eq!(status.code(), Some(1), "{log}");
    assert!(log.contains(&taken.to_string()), "the log names the taken address:\n{log}");
    assert!(TcpStream::connect(free).is_err(), "nothing is left serving the other port");
}

#[test]
fn a_taken_http_port_ends_the_start_with_a_non_zero_exit() {
    start_with_a_taken_port("taken-http", true);
}

#[test]
fn a_taken_wyoming_port_ends_the_start_with_a_non_zero_exit() {
    start_with_a_taken_port("taken-wyoming", false);
}

/// Send `signal` to a running engine that has an idle HTTP and an idle Wyoming connection
/// open, and expect an orderly exit inside the bound.
fn stops_in_order(running: &mut Running, signal: &str) {
    let mut idle_http = TcpStream::connect(running.http).unwrap();
    idle_http.write_all(b"GET /api/settings HTTP/1.1\r\nHost: klar.test\r\n\r\n").unwrap();
    let mut first = [0u8; 12];
    idle_http.read_exact(&mut first).unwrap();
    assert_eq!(&first, b"HTTP/1.1 200");
    let _idle_wyoming = TcpStream::connect(running.wyoming).unwrap();
    assert!(process::wyoming_describe(running.wyoming, DESCRIBE).unwrap().contains("\"info\""));

    let sent = Instant::now();
    running.signal(signal);
    let status = running.wait_exit(STOP_BOUND);
    let log = running.log();
    let status = status.unwrap_or_else(|| panic!("still running {STOP_BOUND:?} after {signal}:\n{log}"));
    assert_eq!(status.code(), Some(0), "an orderly stop exits with 0, after {:?}:\n{log}", sent.elapsed());
    assert!(log.contains(&format!("SIG{signal} empfangen")), "{log}");
    assert!(TcpStream::connect(running.http).is_err() && TcpStream::connect(running.wyoming).is_err(), "both ports are closed");
}

#[test]
fn sigterm_ends_the_engine_in_order_within_two_seconds() {
    let root = TempDir::new("sigterm");
    stops_in_order(&mut process::start(&root, &[]), "TERM");
}

#[test]
fn sigint_ends_the_engine_in_order_within_two_seconds() {
    let root = TempDir::new("sigint");
    stops_in_order(&mut process::start(&root, &[]), "INT");
}

#[test]
fn as_pid_1_sigterm_ends_the_engine_within_two_seconds() {
    if !process::pid_namespace_available() {
        eprintln!("skipped: this machine does not let an unprivileged process create a PID namespace");
        return;
    }
    let root = TempDir::new("pid1");
    let mut running = process::start_via(&root, &[], Launch::AsPidOne);
    assert_eq!(running.pid_in_namespace(), Some(1), "the engine must be PID 1 for this test to mean anything");
    stops_in_order(&mut running, "TERM");
}

#[test]
fn after_file_descriptors_ran_out_wyoming_accepts_again() {
    let root = TempDir::new("fd-limit");
    let mut running = process::start_via(&root, &[], Launch::FdLimit(96));
    assert!(process::wyoming_describe(running.wyoming, DESCRIBE).unwrap().contains("\"info\""));

    // More idle connections than the engine has descriptors for. The kernel completes the
    // handshakes; the engine cannot accept them all.
    let held: Vec<TcpStream> = (0..200).filter_map(|_| TcpStream::connect(running.http).ok()).collect();
    assert!(held.len() > 96, "only {} connections could be opened", held.len());
    std::thread::sleep(Duration::from_millis(600));
    // This connection reaches the Wyoming listener while it has no descriptor to accept with.
    let during = process::wyoming_describe(running.wyoming, Duration::from_millis(800));
    drop(held);

    let until = Instant::now() + Duration::from_secs(8);
    let mut after = process::wyoming_describe(running.wyoming, DESCRIBE);
    while after.is_err() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(200));
        after = process::wyoming_describe(running.wyoming, DESCRIBE);
    }
    let log = running.log();
    assert!(running.is_running(), "the engine died:\n{log}");
    let answer = after.unwrap_or_else(|err| panic!("Wyoming never answered again ({err}); during the shortage: {during:?}\n{log}"));
    assert!(answer.contains("\"info\""), "{answer}");
    assert_eq!(running.get("/api/settings").0, 200, "HTTP serves again too");
}

/// Read until the server closes the connection or `deadline` passes. `true` when it closed.
fn closed_by(mut stream: TcpStream, deadline: Instant) -> bool {
    let mut sink = [0u8; 4096];
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return false;
        }
        stream.set_read_timeout(Some(left)).unwrap();
        match stream.read(&mut sink) {
            Ok(0) => return true,
            Ok(_) => {}
            Err(err) if matches!(err.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => return false,
            Err(_) => return true,
        }
    }
}

#[test]
fn connections_that_stall_or_idle_are_closed_by_the_engine() {
    let root = TempDir::new("http-deadlines");
    let running = process::start(&root, &[]);
    let open = |sent: &[u8]| {
        let mut stream = TcpStream::connect(running.http).unwrap();
        stream.write_all(sent).unwrap();
        stream
    };
    let half_head = open(b"GET /api/settings HTTP/1.1\r\nHost: klar.test\r\n");
    let stalled_body =
        open(b"POST /api/custom HTTP/1.1\r\nHost: klar.test\r\nContent-Type: application/json\r\nContent-Length: 4096\r\n\r\n[{\"phra");
    let idle = open(b"GET /api/settings HTTP/1.1\r\nHost: klar.test\r\n\r\n");

    // The limits are 30 s for a request head and for idling, and 20 s between pieces of a body.
    let deadline = Instant::now() + Duration::from_secs(45);
    assert!(closed_by(stalled_body, deadline), "an upload that stalled was still open after 45 s");
    assert!(closed_by(half_head, deadline), "half a request head was still open after 45 s");
    assert!(closed_by(idle, deadline), "an idle connection was still open after 45 s");
    assert_eq!(running.get("/api/settings").0, 200);
}
