//! In-memory trainer write consent. Reload or restart asks again.

use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

const PENDING_WAIT: Duration = Duration::from_secs(120);
const YOLO_TTL: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentDecision {
    AllowOnce,
    Allow,
    Yolo,
    Deny,
    AskAgain,
}

#[derive(Debug, Clone)]
pub struct PendingWrite {
    pub name: String,
    pub args: serde_json::Value,
    pub summary: String,
    pub preview: serde_json::Value,
}

struct PendingSlot {
    write: PendingWrite,
    tx: Option<oneshot::Sender<ConsentDecision>>,
    since: Instant,
}

#[derive(Default)]
struct Session {
    yolo: bool,
    yolo_until: Option<Instant>,
    allowed: HashSet<String>,
    pending: HashMap<String, PendingSlot>,
}

impl Session {
    fn yolo_active(&self) -> bool {
        self.yolo && self.yolo_until.is_some_and(|until| Instant::now() < until)
    }

    fn sweep_expired(&mut self) {
        if self.yolo && !self.yolo_active() {
            self.yolo = false;
            self.yolo_until = None;
        }
        let now = Instant::now();
        self.pending.retain(|_, slot| now.duration_since(slot.since) < PENDING_WAIT);
    }
}

#[derive(Default)]
pub struct TrainerConsentHub {
    inner: Mutex<HashMap<String, Session>>,
}

impl TrainerConsentHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Isolates YOLO/allow lists. Not authentication — spoofable headers can collide.
    ///
    /// Ingress users share the Supervisor proxy IP, so the key hashes the write token
    /// plus `x-ingress-path` and a user header (`x-hass-user-id` / `x-remote-user-id`)
    /// or `x-forwarded-for` when the proxy sent one. Home Assistant ingress often omits
    /// a user id; those operators then share consent if they share token and XFF.
    pub fn session_key(token: &Option<String>, peer: SocketAddr, headers: &HeaderMap) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        match token.as_deref().filter(|item| !item.is_empty()) {
            Some(token) => token.hash(&mut hasher),
            None => peer.ip().hash(&mut hasher),
        }
        if let Some(path) = header_text(headers, "x-ingress-path") {
            path.hash(&mut hasher);
        }
        if let Some(user) = operator_header(headers) {
            user.hash(&mut hasher);
        } else if let Some(xff) = header_text(headers, "x-forwarded-for") {
            xff.hash(&mut hasher);
        }
        format!("sess:{:x}", hasher.finish())
    }

    pub async fn snapshot(&self, key: &str) -> (bool, Vec<String>) {
        let mut guard = self.inner.lock().await;
        match guard.get_mut(key) {
            Some(session) => {
                session.sweep_expired();
                let mut allowed: Vec<String> = session.allowed.iter().cloned().collect();
                allowed.sort();
                (session.yolo_active(), allowed)
            }
            None => (false, Vec::new()),
        }
    }

    pub async fn allows(&self, key: &str, tool: &str) -> bool {
        let mut guard = self.inner.lock().await;
        let Some(session) = guard.get_mut(key) else {
            return false;
        };
        session.sweep_expired();
        session.yolo_active() || session.allowed.contains(tool)
    }

    /// Registers a pending write under a server-minted id. Caller emits the id, then [`await_decision`].
    pub async fn register(&self, key: &str, write: PendingWrite) -> (String, oneshot::Receiver<ConsentDecision>) {
        let call_id = Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();
        {
            let mut guard = self.inner.lock().await;
            let session = guard.entry(key.to_string()).or_default();
            session.sweep_expired();
            session.pending.insert(call_id.clone(), PendingSlot { write, tx: Some(tx), since: Instant::now() });
        }
        (call_id, rx)
    }

    pub async fn await_decision(&self, key: &str, call_id: &str, rx: oneshot::Receiver<ConsentDecision>) -> ConsentDecision {
        match tokio::time::timeout(PENDING_WAIT, rx).await {
            Ok(Ok(decision)) => decision,
            _ => {
                let _ = self.decide(key, call_id, ConsentDecision::Deny).await;
                ConsentDecision::Deny
            }
        }
    }

    /// Registers, then waits (tests and simple callers).
    pub async fn wait(&self, key: &str, write: PendingWrite) -> (String, ConsentDecision) {
        let (call_id, rx) = self.register(key, write).await;
        let decision = self.await_decision(key, &call_id, rx).await;
        (call_id, decision)
    }

    pub async fn deny_pending(&self, key: &str) {
        let mut guard = self.inner.lock().await;
        let Some(session) = guard.get_mut(key) else {
            return;
        };
        let ids: Vec<String> = session.pending.keys().cloned().collect();
        for id in ids {
            if let Some(mut slot) = session.pending.remove(&id) {
                if let Some(tx) = slot.tx.take() {
                    let _ = tx.send(ConsentDecision::Deny);
                }
            }
        }
    }

    pub async fn decide(&self, key: &str, call_id: &str, decision: ConsentDecision) -> Result<Option<PendingWrite>, &'static str> {
        let mut guard = self.inner.lock().await;
        let session = guard.entry(key.to_string()).or_default();
        session.sweep_expired();
        match decision {
            ConsentDecision::AskAgain => {
                session.yolo = false;
                session.yolo_until = None;
                session.allowed.clear();
                Ok(None)
            }
            ConsentDecision::Yolo => {
                // A-09: never flip YOLO on an unknown/expired call_id.
                if !session.pending.contains_key(call_id) {
                    return Err("unknown call");
                }
                let write = Self::finish_pending(session, call_id, decision)?;
                session.yolo = true;
                session.yolo_until = Some(Instant::now() + YOLO_TTL);
                Ok(write)
            }
            ConsentDecision::Allow => {
                if let Some(slot) = session.pending.get(call_id) {
                    session.allowed.insert(slot.write.name.clone());
                }
                Self::finish_pending(session, call_id, decision)
            }
            ConsentDecision::AllowOnce | ConsentDecision::Deny => Self::finish_pending(session, call_id, decision),
        }
    }

    fn finish_pending(session: &mut Session, call_id: &str, decision: ConsentDecision) -> Result<Option<PendingWrite>, &'static str> {
        let Some(mut slot) = session.pending.remove(call_id) else {
            return Err("unknown call");
        };
        if let Some(tx) = slot.tx.take() {
            let _ = tx.send(decision);
        }
        Ok(Some(slot.write))
    }
}

fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok()).map(str::trim).filter(|value| !value.is_empty())
}

fn operator_header(headers: &HeaderMap) -> Option<&str> {
    ["x-hass-user-id", "x-remote-user-id"].into_iter().find_map(|name| header_text(headers, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allow_once_does_not_remember_tool() {
        let hub = TrainerConsentHub::new();
        let key = "s";
        let pending = PendingWrite {
            name: "apply_aliases".into(),
            args: serde_json::json!({}),
            summary: "alias".into(),
            preview: serde_json::json!({"ok": true}),
        };
        let wait = hub.wait(key, pending);
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            let (yolo, _) = hub.snapshot(key).await;
            assert!(!yolo);
            let call_id = {
                let guard = hub.inner.lock().await;
                guard.get(key).unwrap().pending.keys().next().unwrap().clone()
            };
            hub.decide(key, &call_id, ConsentDecision::AllowOnce).await.unwrap();
        };
        let ((_, decision), _) = tokio::join!(wait, decide);
        assert_eq!(decision, ConsentDecision::AllowOnce);
        assert!(!hub.allows(key, "apply_aliases").await);
    }

    #[tokio::test]
    async fn allow_remembers_tool_yolo_all_ask_again_clears() {
        let hub = TrainerConsentHub::new();
        let key = "s";
        let pending = || PendingWrite {
            name: "apply_aliases".into(),
            args: serde_json::json!({}),
            summary: "alias".into(),
            preview: serde_json::json!({"ok": true}),
        };
        let wait = hub.wait(key, pending());
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            let call_id = {
                let guard = hub.inner.lock().await;
                guard.get(key).unwrap().pending.keys().next().unwrap().clone()
            };
            hub.decide(key, &call_id, ConsentDecision::Allow).await.unwrap();
        };
        let _ = tokio::join!(wait, decide);
        assert!(hub.allows(key, "apply_aliases").await);
        assert!(!hub.allows(key, "apply_house").await);
        let wait = hub.wait(key, pending());
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            let call_id = {
                let guard = hub.inner.lock().await;
                guard.get(key).unwrap().pending.keys().next().unwrap().clone()
            };
            hub.decide(key, &call_id, ConsentDecision::Yolo).await.unwrap();
        };
        let _ = tokio::join!(wait, decide);
        assert!(hub.allows(key, "apply_house").await);
        hub.decide(key, "", ConsentDecision::AskAgain).await.unwrap();
        assert!(!hub.allows(key, "apply_aliases").await);
        let (yolo, allowed) = hub.snapshot(key).await;
        assert!(!yolo);
        assert!(allowed.is_empty());
    }

    #[tokio::test]
    async fn deny_returns_without_allowing() {
        let hub = TrainerConsentHub::new();
        let wait = hub.wait(
            "s",
            PendingWrite { name: "apply_house".into(), args: serde_json::json!({}), summary: "x".into(), preview: serde_json::json!({}) },
        );
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            let call_id = {
                let guard = hub.inner.lock().await;
                guard.get("s").unwrap().pending.keys().next().unwrap().clone()
            };
            hub.decide("s", &call_id, ConsentDecision::Deny).await.unwrap();
        };
        let ((_, decision), _) = tokio::join!(wait, decide);
        assert_eq!(decision, ConsentDecision::Deny);
        assert!(!hub.allows("s", "apply_house").await);
    }

    #[tokio::test]
    async fn late_yolo_on_unknown_call_does_not_enable_yolo() {
        let hub = TrainerConsentHub::new();
        assert!(hub.decide("s", "missing", ConsentDecision::Yolo).await.is_err());
        assert!(!hub.allows("s", "apply_house").await);
        let (yolo, _) = hub.snapshot("s").await;
        assert!(!yolo);
    }

    #[tokio::test]
    async fn wait_mints_server_side_call_id() {
        let hub = TrainerConsentHub::new();
        let wait = hub.wait(
            "s",
            PendingWrite { name: "apply_house".into(), args: serde_json::json!({}), summary: "x".into(), preview: serde_json::json!({}) },
        );
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            let call_id = {
                let guard = hub.inner.lock().await;
                guard.get("s").unwrap().pending.keys().next().unwrap().clone()
            };
            assert!(!call_id.is_empty());
            hub.decide("s", &call_id, ConsentDecision::AllowOnce).await.unwrap();
        };
        let ((minted, _), _) = tokio::join!(wait, decide);
        assert!(!minted.is_empty());
    }

    #[test]
    fn session_key_splits_ingress_users_when_headers_differ() {
        let peer = "172.30.32.2:9".parse().unwrap();
        let token = Some("secret".into());
        let mut a = HeaderMap::new();
        a.insert("x-ingress-path", "/api/hassio_ingress/tok".parse().unwrap());
        a.insert("x-hass-user-id", "user-a".parse().unwrap());
        let mut b = a.clone();
        b.insert("x-hass-user-id", "user-b".parse().unwrap());
        assert_ne!(TrainerConsentHub::session_key(&token, peer, &a), TrainerConsentHub::session_key(&token, peer, &b));
        let mut xff_a = HeaderMap::new();
        xff_a.insert("x-forwarded-for", "10.0.0.8".parse().unwrap());
        let mut xff_b = HeaderMap::new();
        xff_b.insert("x-forwarded-for", "10.0.0.9".parse().unwrap());
        assert_ne!(TrainerConsentHub::session_key(&token, peer, &xff_a), TrainerConsentHub::session_key(&token, peer, &xff_b));
        assert_eq!(
            TrainerConsentHub::session_key(&token, peer, &HeaderMap::new()),
            TrainerConsentHub::session_key(&token, peer, &HeaderMap::new())
        );
    }
}
