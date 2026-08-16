use crate::types::Intent;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

const MAX_SESSIONS: usize = 256;
const SESSION_TTL: Duration = Duration::from_secs(2 * 60 * 60);
const LAST_KEEP: usize = 8;

#[derive(Debug, Clone)]
pub struct LastTurn {
    pub entity: Option<String>,
    pub area: Option<String>,
    pub name: String,
    pub domain: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub last: Vec<LastTurn>,
    pub last_intent_template: Option<Intent>,
    pub pending_clarify: Option<Vec<String>>,
    pub wrong_log: Vec<String>,
    pub briefing: bool,
    pub preferred_area: Option<String>,
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
            last: Vec::new(),
            last_intent_template: None,
            pending_clarify: None,
            wrong_log: Vec::new(),
            briefing: false,
            preferred_area: None,
            last_used: Instant::now(),
        }
    }

    pub fn last_entities(&self) -> impl Iterator<Item = &str> {
        self.last.iter().filter_map(|turn| turn.entity.as_deref())
    }

    pub fn last_areas(&self) -> impl Iterator<Item = &str> {
        self.last.iter().filter_map(|turn| turn.area.as_deref())
    }

    pub fn last_names(&self) -> impl Iterator<Item = &str> {
        self.last.iter().map(|turn| turn.name.as_str())
    }

    pub fn last_domains(&self) -> impl Iterator<Item = &str> {
        self.last.iter().filter_map(|turn| turn.domain.as_deref())
    }

    pub fn remember_entity(&mut self, entity_id: impl Into<String>) {
        self.remember(&Intent::new("HassTurnOn").with("entity_id", entity_id.into()));
    }

    pub fn remember(&mut self, intent: &Intent) {
        let entity = intent.slot("entity_id").map(str::to_string);
        let area = intent.slot("area").map(str::to_string);
        let domain =
            intent.slot("domain").map(str::to_string).or_else(|| entity.as_deref().and_then(|id| id.split('.').next()).map(str::to_string));
        if let Some(id) = &entity {
            self.last.retain(|turn| turn.entity.as_deref() != Some(id.as_str()));
        }
        self.last.insert(0, LastTurn { entity, area, name: intent.name.clone(), domain });
        self.last.truncate(LAST_KEEP);
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
        self.sweep_ttl();
        match id {
            Some(existing) if existing.len() <= 128 && self.inner.contains_key(existing) => {
                let session = self.inner.get_mut(existing).expect("checked");
                session.last_used = Instant::now();
                session
            }
            Some(existing) if existing.len() <= 128 => {
                self.make_room();
                let mut s = Session::new();
                s.id = existing.to_string();
                self.inner.insert(existing.to_string(), s);
                self.inner.get_mut(existing).expect("inserted")
            }
            _ => {
                self.make_room();
                let s = Session::new();
                let key = s.id.clone();
                self.inner.insert(key.clone(), s);
                self.inner.get_mut(&key).expect("inserted")
            }
        }
    }

    pub fn take(&mut self, id: Option<&str>) -> Session {
        self.get_or_create(id).clone()
    }

    pub fn put(&mut self, mut session: Session) {
        self.sweep_ttl();
        session.last_used = Instant::now();
        self.inner.insert(session.id.clone(), session);
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }

    fn sweep_ttl(&mut self) {
        let now = Instant::now();
        self.inner.retain(|_, session| now.duration_since(session.last_used) < SESSION_TTL);
    }

    fn make_room(&mut self) {
        while self.inner.len() >= MAX_SESSIONS {
            let oldest = self.inner.iter().min_by_key(|(_, s)| s.last_used).map(|(id, _)| id.clone());
            if let Some(id) = oldest {
                self.inner.remove(&id);
            } else {
                break;
            }
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
        sessions.get_or_create(Some("assist-1")).remember_entity("light.a");
        assert_eq!(sessions.get_or_create(Some("assist-1")).last_entities().collect::<Vec<_>>(), ["light.a"]);
    }

    #[test]
    fn evicts_when_over_cap() {
        let mut sessions = Sessions::default();
        for i in 0..300 {
            sessions.get_or_create(Some(&format!("id-{i}")));
        }
        assert!(sessions.len() <= MAX_SESSIONS, "{}", sessions.len());
    }
}
