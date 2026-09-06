//! In-memory trainer write consent. Reload or restart asks again.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

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
}

#[derive(Default)]
struct Session {
    yolo: bool,
    allowed: HashSet<String>,
    pending: HashMap<String, PendingSlot>,
}

#[derive(Default)]
pub struct TrainerConsentHub {
    inner: Mutex<HashMap<String, Session>>,
}

impl TrainerConsentHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn session_key(token: &Option<String>, peer: SocketAddr) -> String {
        match token.as_deref().filter(|item| !item.is_empty()) {
            Some(token) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                token.hash(&mut hasher);
                format!("tok:{:x}", hasher.finish())
            }
            None => format!("peer:{}", peer.ip()),
        }
    }

    pub async fn snapshot(&self, key: &str) -> (bool, Vec<String>) {
        let guard = self.inner.lock().await;
        match guard.get(key) {
            Some(session) => {
                let mut allowed: Vec<String> = session.allowed.iter().cloned().collect();
                allowed.sort();
                (session.yolo, allowed)
            }
            None => (false, Vec::new()),
        }
    }

    pub async fn allows(&self, key: &str, tool: &str) -> bool {
        let guard = self.inner.lock().await;
        guard.get(key).is_some_and(|session| session.yolo || session.allowed.contains(tool))
    }

    pub async fn wait(&self, key: &str, call_id: String, write: PendingWrite) -> ConsentDecision {
        let (tx, rx) = oneshot::channel();
        {
            let mut guard = self.inner.lock().await;
            let session = guard.entry(key.to_string()).or_default();
            session.pending.insert(call_id.clone(), PendingSlot { write, tx: Some(tx) });
        }
        match tokio::time::timeout(std::time::Duration::from_secs(300), rx).await {
            Ok(Ok(decision)) => decision,
            _ => {
                let _ = self.decide(key, &call_id, ConsentDecision::Deny).await;
                ConsentDecision::Deny
            }
        }
    }

    pub async fn decide(&self, key: &str, call_id: &str, decision: ConsentDecision) -> Result<Option<PendingWrite>, &'static str> {
        let mut guard = self.inner.lock().await;
        let session = guard.entry(key.to_string()).or_default();
        match decision {
            ConsentDecision::AskAgain => {
                session.yolo = false;
                session.allowed.clear();
                Ok(None)
            }
            ConsentDecision::Yolo => {
                session.yolo = true;
                self.finish_pending(session, call_id, decision)
            }
            ConsentDecision::Allow => {
                if let Some(slot) = session.pending.get(call_id) {
                    session.allowed.insert(slot.write.name.clone());
                }
                self.finish_pending(session, call_id, decision)
            }
            ConsentDecision::AllowOnce | ConsentDecision::Deny => self.finish_pending(session, call_id, decision),
        }
    }

    fn finish_pending(
        &self,
        session: &mut Session,
        call_id: &str,
        decision: ConsentDecision,
    ) -> Result<Option<PendingWrite>, &'static str> {
        let Some(mut slot) = session.pending.remove(call_id) else {
            return Err("unknown call");
        };
        if let Some(tx) = slot.tx.take() {
            let _ = tx.send(decision);
        }
        Ok(Some(slot.write))
    }
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
        let wait = hub.wait(key, "c1".into(), pending);
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            hub.decide(key, "c1", ConsentDecision::AllowOnce).await.unwrap();
        };
        let (decision, _) = tokio::join!(wait, decide);
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
        let wait = hub.wait(key, "c1".into(), pending());
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            hub.decide(key, "c1", ConsentDecision::Allow).await.unwrap();
        };
        let _ = tokio::join!(wait, decide);
        assert!(hub.allows(key, "apply_aliases").await);
        assert!(!hub.allows(key, "apply_house").await);
        let wait = hub.wait(key, "c2".into(), pending());
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            hub.decide(key, "c2", ConsentDecision::Yolo).await.unwrap();
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
            "c1".into(),
            PendingWrite { name: "apply_house".into(), args: serde_json::json!({}), summary: "x".into(), preview: serde_json::json!({}) },
        );
        let decide = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            hub.decide("s", "c1", ConsentDecision::Deny).await.unwrap();
        };
        let (decision, _) = tokio::join!(wait, decide);
        assert_eq!(decision, ConsentDecision::Deny);
        assert!(!hub.allows("s", "apply_house").await);
    }
}
