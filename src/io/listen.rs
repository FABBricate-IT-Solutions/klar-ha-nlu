//! What both listeners share: binding before anything starts, the stop flag, and what to do
//! when `accept` fails.

use std::io;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::watch;

/// Pause after a failed `accept`. Out of file descriptors is the usual cause, and it passes
/// when connections close; retrying at once would only spin.
const ACCEPT_RETRY: Duration = Duration::from_millis(250);

/// Bind `addr` for the server called `what`. The error names both, so a taken port shows in
/// the log as what it is.
pub(crate) async fn bind(what: &str, addr: &str) -> io::Result<TcpListener> {
    TcpListener::bind(addr).await.map_err(|err| io::Error::new(err.kind(), format!("{what} kann {addr} nicht binden: {err}")))
}

/// Resolves when the stop flag is set. A dropped sender is not a stop: nothing asked for one.
pub(crate) async fn stopped(stop: &mut watch::Receiver<bool>) {
    if stop.wait_for(|stop| *stop).await.is_err() {
        std::future::pending::<()>().await;
    }
}

/// `accept` failed. A peer that went away before it was accepted is not worth a line; anything
/// else is logged, and the listener waits a moment and carries on.
pub(crate) async fn accept_failed(server: &str, err: &io::Error) {
    if matches!(err.kind(), io::ErrorKind::ConnectionAborted | io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionRefused) {
        return;
    }
    tracing::error!("{server}: accept fehlgeschlagen, der Listener macht weiter: {err}");
    tokio::time::sleep(ACCEPT_RETRY).await;
}
