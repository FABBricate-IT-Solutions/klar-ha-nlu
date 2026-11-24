//! The HTTP listener.
//!
//! `axum::serve` has no limits: a client could hold a connection open for ever with half a
//! request, and nothing bounded how many connections were open. This loop serves the same
//! router with a deadline for the request head, a deadline between pieces of a body, a cap
//! on open connections, and an orderly stop.

use crate::io::listen::{accept_failed, stopped};
use axum::extract::ConnectInfo;
use axum::Router;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::{TokioIo, TokioTimer};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch, OwnedSemaphorePermit, Semaphore};
use tower::Service;
use tower_http::timeout::RequestBodyTimeoutLayer;

#[derive(Debug, Clone, Copy)]
pub struct HttpLimits {
    /// How long a connection may take to deliver a complete request head, counted from when
    /// the server is ready for one. That covers a slow sender and an idle keep-alive
    /// connection alike: hyper runs one timer for both.
    pub head: Duration,
    /// The longest pause between two pieces of a request body.
    pub body: Duration,
    /// Connections served at once. Further ones wait in the listen backlog.
    pub connections: usize,
}

impl Default for HttpLimits {
    fn default() -> Self {
        // Home Assistant's HTTP client closes its idle connections after 15 s, so 30 s lets
        // the client close first.
        Self { head: Duration::from_secs(30), body: Duration::from_secs(20), connections: 256 }
    }
}

/// Serve `router` until `stop` is set, then finish the requests that are under way.
pub async fn serve(listener: TcpListener, router: Router, limits: HttpLimits, mut stop: watch::Receiver<bool>) -> io::Result<()> {
    let router = router.layer(RequestBodyTimeoutLayer::new(limits.body));
    let slots = Arc::new(Semaphore::new(limits.connections));
    // Every connection holds a sender. When the last one is dropped, `recv` returns.
    let (alive, mut all_closed) = mpsc::channel::<()>(1);
    loop {
        let slot = tokio::select! {
            slot = slots.clone().acquire_owned() => slot.map_err(io::Error::other)?,
            () = stopped(&mut stop) => break,
        };
        let (stream, peer) = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok(accepted) => accepted,
                Err(err) => {
                    accept_failed("HTTP", &err).await;
                    continue;
                }
            },
            () = stopped(&mut stop) => break,
        };
        tokio::spawn(connection(stream, peer, router.clone(), limits, stop.clone(), slot, alive.clone()));
    }
    drop(listener);
    drop(alive);
    let _ = all_closed.recv().await;
    Ok(())
}

async fn connection(
    stream: TcpStream,
    peer: SocketAddr,
    router: Router,
    limits: HttpLimits,
    mut stop: watch::Receiver<bool>,
    _slot: OwnedSemaphorePermit,
    _alive: mpsc::Sender<()>,
) {
    let service = service_fn(move |mut request: Request<Incoming>| {
        request.extensions_mut().insert(ConnectInfo(peer));
        let mut router = router.clone();
        async move { router.call(request).await }
    });
    let mut builder = http1::Builder::new();
    builder.timer(TokioTimer::new()).header_read_timeout(limits.head);
    let serving = builder.serve_connection(TokioIo::new(stream), service).with_upgrades();
    tokio::pin!(serving);
    let ended = tokio::select! {
        ended = serving.as_mut() => ended,
        () = stopped(&mut stop) => {
            // Answer the request that is under way, take no further one.
            serving.as_mut().graceful_shutdown();
            serving.await
        }
    };
    if let Err(err) = ended {
        tracing::debug!("HTTP-Verbindung {peer}: {err}");
    }
}

#[cfg(test)]
#[path = "http_server_tests.rs"]
mod tests;
