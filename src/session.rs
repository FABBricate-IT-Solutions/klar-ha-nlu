use crate::types::{
    Intent, IntentPlan, ParseDecision, ParseOutcome, MAX_CLARIFY_OPTIONS, MAX_DETAIL_CHARS, MAX_EVIDENCE_PER_ITEM, MAX_PLAN_STEPS,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

const MAX_SESSIONS: usize = 256;
const SESSION_TTL: Duration = Duration::from_secs(2 * 60 * 60);
const LAST_KEEP: usize = 8;
const WRONG_LOG_KEEP: usize = 32;

#[derive(Debug, Clone)]
pub struct LastTurn {
    pub entity: Option<String>,
    pub area: Option<String>,
    pub name: String,
    pub domain: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LastHeard {
    pub text: String,
    pub decision: String,
    pub speech: String,
    pub reason: Option<String>,
    pub area: Option<String>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClarifyState {
    pub options: Vec<String>,
    pub template: Intent,
}

#[derive(Debug, Clone)]
pub struct ConfirmState {
    pub candidate_id: String,
    pub plan: IntentPlan,
    pub prompt: String,
}

#[derive(Debug, Clone)]
pub enum PendingInteraction {
    Clarify(ClarifyState),
    Confirm(ConfirmState),
}

#[cfg(debug_assertions)]
#[derive(Debug, PartialEq)]
pub(crate) struct SessionParity {
    last: Vec<LastParity>,
    pending: Option<PendingParity>,
    wrong_log: Vec<String>,
    briefing: bool,
    preferred_area: Option<String>,
}

#[cfg(debug_assertions)]
#[derive(Debug, PartialEq)]
struct LastParity {
    entity: Option<String>,
    area: Option<String>,
    name: String,
    domain: Option<String>,
}

#[cfg(debug_assertions)]
#[derive(Debug, PartialEq)]
enum PendingParity {
    Clarify(Vec<String>, Intent),
    Confirm(String, IntentPlan, String),
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub last: Vec<LastTurn>,
    pub pending: Option<PendingInteraction>,
    pub wrong_log: Vec<String>,
    pub briefing: bool,
    pub preferred_area: Option<String>,
    pub last_execute: Vec<Intent>,
    pub last_heard: Option<LastHeard>,
    pub pending_teach: Option<(String, String)>,
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
            pending: None,
            wrong_log: Vec::new(),
            briefing: false,
            preferred_area: None,
            last_execute: Vec::new(),
            last_heard: None,
            pending_teach: None,
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

    pub fn note_heard(&mut self, outcome: &ParseOutcome) {
        let (decision, reason) = match &outcome.decision {
            ParseDecision::Execute => ("execute", None),
            ParseDecision::Clarify { .. } => ("clarify", None),
            ParseDecision::Confirm { .. } => ("confirm", None),
            ParseDecision::Reject { reason } => ("reject", Some(format!("{reason:?}"))),
            ParseDecision::Chat => ("chat", None),
            ParseDecision::Error { code, .. } => ("error", Some(code.clone())),
        };
        self.last_heard = Some(LastHeard {
            text: outcome.text.clone(),
            decision: decision.into(),
            speech: outcome.speech.clone(),
            reason,
            area: self.preferred_area.clone(),
            names: outcome.plan.as_ref().map(|plan| plan.intents().into_iter().map(|intent| intent.name).collect()).unwrap_or_default(),
        });
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

    pub fn pending_clarify(&self) -> Option<&ClarifyState> {
        match self.pending.as_ref() {
            Some(PendingInteraction::Clarify(state)) => Some(state),
            Some(PendingInteraction::Confirm(_)) | None => None,
        }
    }

    pub fn pending_confirm(&self) -> Option<&ConfirmState> {
        match self.pending.as_ref() {
            Some(PendingInteraction::Confirm(state)) => Some(state),
            Some(PendingInteraction::Clarify(_)) | None => None,
        }
    }

    pub fn set_clarify(&mut self, mut options: Vec<String>, template: Intent) {
        options.truncate(MAX_CLARIFY_OPTIONS);
        for option in &mut options {
            truncate_chars(option, 128);
        }
        self.pending = Some(PendingInteraction::Clarify(ClarifyState { options, template }));
    }

    pub fn set_confirm(&mut self, candidate_id: String, mut plan: IntentPlan, mut prompt: String) -> bool {
        if candidate_id.is_empty()
            || candidate_id.chars().count() > 128
            || candidate_id.chars().any(char::is_control)
            || plan.steps.is_empty()
            || plan.steps.len() > MAX_PLAN_STEPS
        {
            return false;
        }
        plan.evidence.truncate(MAX_EVIDENCE_PER_ITEM);
        for step in &mut plan.steps {
            step.evidence.truncate(MAX_EVIDENCE_PER_ITEM);
        }
        truncate_chars(&mut prompt, MAX_DETAIL_CHARS);
        self.pending = Some(PendingInteraction::Confirm(ConfirmState { candidate_id, plan, prompt }));
        true
    }

    pub fn clear_pending(&mut self) {
        self.pending = None;
    }

    pub fn mark_wrong(&mut self) {
        self.wrong_log.push(self.id.clone());
        if self.wrong_log.len() > WRONG_LOG_KEEP {
            self.wrong_log.drain(..self.wrong_log.len() - WRONG_LOG_KEEP);
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn parity_snapshot(&self) -> SessionParity {
        let clarify_template = self.pending_clarify().map(|state| &state.template);
        let last = self
            .last
            .iter()
            .filter(|turn| {
                clarify_template.is_none_or(|template| {
                    turn.name != template.name
                        || turn.entity.as_deref() != template.slot("entity_id")
                        || turn.area.as_deref() != template.slot("area")
                })
            })
            .map(|turn| LastParity {
                entity: turn.entity.clone(),
                area: turn.area.clone(),
                name: turn.name.clone(),
                domain: turn.domain.clone(),
            })
            .collect();
        let pending = match self.pending.as_ref() {
            Some(PendingInteraction::Clarify(state)) => Some(PendingParity::Clarify(state.options.clone(), state.template.clone())),
            Some(PendingInteraction::Confirm(state)) => {
                Some(PendingParity::Confirm(state.candidate_id.clone(), state.plan.clone(), state.prompt.clone()))
            }
            None => None,
        };
        SessionParity {
            last,
            pending,
            wrong_log: self.wrong_log.clone(),
            briefing: self.briefing,
            preferred_area: self.preferred_area.clone(),
        }
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
        if session.id.chars().count() > 128 || session.id.chars().any(char::is_control) {
            session.id = Uuid::new_v4().to_string();
        }
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

fn truncate_chars(value: &mut String, maximum: usize) {
    if value.chars().count() > maximum {
        *value = value.chars().take(maximum).collect();
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

    #[test]
    fn caps_wrong_log() {
        let mut session = Session::new();
        for _ in 0..100 {
            session.mark_wrong();
        }
        assert_eq!(session.wrong_log.len(), WRONG_LOG_KEEP);
    }

    #[test]
    fn caps_pending_interaction_storage() {
        let mut session = Session::new();
        session.set_clarify(vec!["x".repeat(200); MAX_CLARIFY_OPTIONS + 10], Intent::new("HassTurnOn"));
        let clarify = session.pending_clarify().expect("clarification");
        assert_eq!(clarify.options.len(), MAX_CLARIFY_OPTIONS);
        assert!(clarify.options.iter().all(|option| option.chars().count() <= 128));

        let evidence = crate::types::Evidence { kind: "test".into(), source: "test".into(), value: "test".into(), score: 1.0, exact: true };
        let mut plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", "lock.front")], 1.0, &[]);
        plan.evidence = vec![evidence.clone(); MAX_EVIDENCE_PER_ITEM + 10];
        plan.steps[0].evidence = vec![evidence; MAX_EVIDENCE_PER_ITEM + 10];
        assert!(session.set_confirm("candidate".into(), plan, "p".repeat(MAX_DETAIL_CHARS + 10)));
        let confirm = session.pending_confirm().expect("confirmation");
        assert_eq!(confirm.plan.evidence.len(), MAX_EVIDENCE_PER_ITEM);
        assert_eq!(confirm.plan.steps[0].evidence.len(), MAX_EVIDENCE_PER_ITEM);
        assert_eq!(confirm.prompt.chars().count(), MAX_DETAIL_CHARS);
    }

    #[test]
    fn rejects_invalid_pending_confirm_ids_and_steps() {
        let mut session = Session::new();
        let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", "lock.front")], 1.0, &[]);
        assert!(!session.set_confirm("x".repeat(129), plan.clone(), "confirm".into()));
        assert!(session.pending_confirm().is_none());
        let oversized = IntentPlan::from_intents(vec![Intent::new("HassTurnOn"); MAX_PLAN_STEPS + 1], 1.0, &[]);
        assert!(!session.set_confirm("candidate".into(), oversized, "confirm".into()));
        assert!(session.pending_confirm().is_none());
    }
}
