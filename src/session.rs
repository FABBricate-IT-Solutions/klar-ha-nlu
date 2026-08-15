use crate::types::Intent;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

const MAX_SESSIONS: usize = 256;
const SESSION_TTL: Duration = Duration::from_secs(2 * 60 * 60);

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub last_entities: Vec<String>,
    pub last_areas: Vec<String>,
    pub last_names: Vec<String>,
    pub last_domains: Vec<String>,
    pub last_intent_template: Option<Intent>,
    pub pending_clarify: Option<Vec<String>>,
    pub wrong_log: Vec<String>,
    last_used: Instant,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            last_entities: Vec::new(),
            last_areas: Vec::new(),
            last_names: Vec::new(),
            last_domains: Vec::new(),
            last_intent_template: None,
            pending_clarify: None,
            wrong_log: Vec::new(),
            last_used: Instant::now(),
        }
    }

    pub fn remember(&mut self, intent: &Intent) {
        if let Some(id) = intent.slot("entity_id") {
            self.last_entities.retain(|e| e != id);
            self.last_entities.insert(0, id.to_string());
            self.last_entities.truncate(8);
        }
        if let Some(area) = intent.slot("area") {
            self.last_areas.retain(|a| a != area);
            self.last_areas.insert(0, area.to_string());
            self.last_areas.truncate(8);
        }
        self.last_names.retain(|n| n != &intent.name);
        self.last_names.insert(0, intent.name.clone());
        self.last_names.truncate(8);
        if let Some(d) = intent.slot("domain") {
            self.last_domains.retain(|x| x != d);
            self.last_domains.insert(0, d.to_string());
            self.last_domains.truncate(8);
        } else if let Some(id) = intent.slot("entity_id") {
            if let Some(d) = id.split('.').next() {
                let d = d.to_string();
                self.last_domains.retain(|x| x != &d);
                self.last_domains.insert(0, d);
                self.last_domains.truncate(8);
            }
        }
    }

    pub fn clear_clarify(&mut self) {
        self.pending_clarify = None;
        self.last_intent_template = None;
    }

    pub fn mark_wrong(&mut self) {
        self.wrong_log.push(self.id.clone());
    }
}

#[derive(Default)]
pub struct Sessions {
    inner: HashMap<String, Session>,
}

impl Sessions {
    pub fn get_or_create(&mut self, id: Option<&str>) -> &mut Session {
        self.evict();
        match id {
            Some(existing) if existing.len() <= 128 && self.inner.contains_key(existing) => {
                let session = self.inner.get_mut(existing).expect("checked");
                session.last_used = Instant::now();
                session
            }
            Some(existing) if existing.len() <= 128 => {
                let mut s = Session::new();
                s.id = existing.to_string();
                self.inner.insert(existing.to_string(), s);
                self.inner.get_mut(existing).expect("inserted")
            }
            _ => {
                let s = Session::new();
                let key = s.id.clone();
                self.inner.insert(key.clone(), s);
                self.inner.get_mut(&key).expect("inserted")
            }
        }
    }

    fn evict(&mut self) {
        let now = Instant::now();
        self.inner.retain(|_, session| now.duration_since(session.last_used) < SESSION_TTL);
        if self.inner.len() <= MAX_SESSIONS {
            return;
        }
        let mut ids: Vec<(Instant, String)> = self.inner.iter().map(|(id, s)| (s.last_used, id.clone())).collect();
        ids.sort_by_key(|(used, _)| *used);
        let drop_n = self.inner.len() - MAX_SESSIONS;
        for (_, id) in ids.into_iter().take(drop_n) {
            self.inner.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_conversation_id() {
        let mut sessions = Sessions::default();
        let huge = "x".repeat(200);
        let session = sessions.get_or_create(Some(&huge));
        assert_ne!(session.id, huge);
        assert!(session.id.len() <= 128);
    }

    #[test]
    fn reuses_same_id() {
        let mut sessions = Sessions::default();
        sessions.get_or_create(Some("assist-1")).last_entities.push("light.a".into());
        assert_eq!(sessions.get_or_create(Some("assist-1")).last_entities, ["light.a"]);
    }
}
