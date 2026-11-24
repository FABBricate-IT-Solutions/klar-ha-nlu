use crate::types::{
    Intent, IntentPlan, ParseDecision, ParseOutcome, MAX_CLARIFY_OPTIONS, MAX_DETAIL_CHARS, MAX_EVIDENCE_PER_ITEM, MAX_PLAN_STEPS,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

const MAX_SESSIONS: usize = 256;
const SESSION_TTL: Duration = Duration::from_secs(2 * 60 * 60);
/// An open question waits this long for its answer. After that a "yes" answers nothing.
pub const PENDING_TTL: Duration = Duration::from_secs(60);
const RECENT_FOLLOW_TTL: Duration = Duration::from_secs(15 * 60);
#[cfg(test)]
const FOLLOWUP_ID: &str = "klar-followup";
const LAST_KEEP: usize = 8;
const WRONG_LOG_KEEP: usize = 32;

#[derive(Debug, Clone)]
pub struct LastTurn {
    pub entity: Option<String>,
    pub area: Option<String>,
    pub name: String,
    pub domain: Option<String>,
    pub turn: u32,
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
    /// The question as it was asked, so it can be asked again word for word.
    pub prompt: String,
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

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub last: Vec<LastTurn>,
    pending: Option<PendingInteraction>,
    pending_since: Option<Instant>,
    /// This copy took the open question out of the store; only it may answer or drop it.
    owns_pending: bool,
    pub wrong_log: Vec<String>,
    pub briefing: bool,
    pub news_topic: Option<String>,
    pub preferred_area: Option<String>,
    pub last_execute: Vec<Intent>,
    pub last_heard: Option<LastHeard>,
    pub pending_teach: Option<(String, String)>,
    last_turn_id: u32,
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
            pending_since: None,
            owns_pending: false,
            wrong_log: Vec::new(),
            briefing: false,
            news_topic: None,
            preferred_area: None,
            last_execute: Vec::new(),
            last_heard: None,
            pending_teach: None,
            last_turn_id: 0,
            last_used: Instant::now(),
        }
    }

    pub fn begin_remember_batch(&mut self) {
        self.last_turn_id = self.last_turn_id.wrapping_add(1);
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
        self.last.insert(0, LastTurn { entity, area, name: intent.name.clone(), domain, turn: self.last_turn_id });
        self.last.truncate(LAST_KEEP);
    }

    /// The open question, if one was asked within `PENDING_TTL`.
    pub fn pending(&self) -> Option<&PendingInteraction> {
        let asked = self.pending_since?;
        if Instant::now().duration_since(asked) >= PENDING_TTL {
            return None;
        }
        self.pending.as_ref()
    }

    pub fn pending_clarify(&self) -> Option<&ClarifyState> {
        match self.pending() {
            Some(PendingInteraction::Clarify(state)) => Some(state),
            Some(PendingInteraction::Confirm(_)) | None => None,
        }
    }

    pub fn pending_confirm(&self) -> Option<&ConfirmState> {
        match self.pending() {
            Some(PendingInteraction::Confirm(state)) => Some(state),
            Some(PendingInteraction::Clarify(_)) | None => None,
        }
    }

    /// How a complete new command sees the session while a question is open: nothing was
    /// asked, and "it" or a bare "50" have no earlier device to fall back on.
    pub fn as_new_command(&self) -> Self {
        let mut copy = self.clone();
        copy.pending = None;
        copy.pending_since = None;
        copy.last.clear();
        copy
    }

    /// Moves the open question into the past, as if it had been asked `by` earlier.
    pub fn backdate_pending(&mut self, by: Duration) {
        self.pending_since = self.pending_since.and_then(|asked| asked.checked_sub(by));
    }

    pub fn set_clarify(&mut self, options: Vec<String>, template: Intent) {
        self.set_clarify_prompt(options, template, String::new());
    }

    pub fn set_clarify_prompt(&mut self, mut options: Vec<String>, template: Intent, mut prompt: String) {
        options.truncate(MAX_CLARIFY_OPTIONS);
        for option in &mut options {
            truncate_chars(option, 128);
        }
        truncate_chars(&mut prompt, MAX_DETAIL_CHARS);
        self.pending = Some(PendingInteraction::Clarify(ClarifyState { options, template, prompt }));
        self.pending_since = Some(Instant::now());
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
        self.pending_since = Some(Instant::now());
        true
    }

    pub fn clear_pending(&mut self) {
        self.pending = None;
        self.pending_since = None;
    }

    /// Hands the open question to the caller and leaves none behind.
    fn claim_pending(&mut self) -> Self {
        let mut taken = self.clone();
        taken.owns_pending = self.pending.take().is_some();
        self.pending_since = None;
        taken
    }

    pub fn mark_wrong(&mut self) {
        self.wrong_log.push(self.id.clone());
        if self.wrong_log.len() > WRONG_LOG_KEEP {
            self.wrong_log.drain(..self.wrong_log.len() - WRONG_LOG_KEEP);
        }
    }
}

#[derive(Default)]
pub struct Sessions {
    inner: HashMap<String, Session>,
    recent_id: Option<String>,
}

impl Sessions {
    pub fn get_or_create(&mut self, id: Option<&str>) -> &mut Session {
        self.sweep_ttl();
        match id.filter(|id| valid_id(id)) {
            Some(existing) if self.inner.contains_key(existing) => {
                let session = self.inner.get_mut(existing).expect("checked");
                session.last_used = Instant::now();
                session
            }
            Some(existing) => {
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

    /// A working copy for one parse. An open question travels with the first copy only:
    /// two answers that arrive together cannot both confirm the same plan.
    pub fn take(&mut self, id: Option<&str>) -> Session {
        let mut session = self.get_or_create(id).claim_pending();
        self.seed_followup(&mut session);
        session
    }

    fn seed_followup(&self, session: &mut Session) {
        if !session.last.is_empty() {
            return;
        }
        let Some(recent_id) = self.recent_id.as_deref() else {
            return;
        };
        if recent_id == session.id {
            return;
        }
        let Some(recent) = self.inner.get(recent_id) else {
            return;
        };
        if Instant::now().duration_since(recent.last_used) >= RECENT_FOLLOW_TTL || recent.last.is_empty() {
            return;
        }
        if !same_satellite_hint(session, recent) {
            return;
        }
        session.last = recent.last.clone();
        session.last_execute = recent.last_execute.clone();
        session.last_turn_id = recent.last_turn_id;
        drop_media_followup(session);
    }

    pub fn take_with_area(&mut self, id: Option<&str>, area: &str) -> Session {
        let mut session = self.get_or_create(id).claim_pending();
        session.preferred_area = Some(area.to_string());
        self.seed_followup(&mut session);
        session
    }

    pub fn put(&mut self, mut session: Session) {
        self.sweep_ttl();
        session.last_used = Instant::now();
        if !valid_id(&session.id) {
            session.id = Uuid::new_v4().to_string();
        }
        // A copy that did not hold the question must not erase one asked in the meantime.
        if !session.owns_pending && session.pending.is_none() {
            if let Some(stored) = self.inner.get_mut(&session.id) {
                session.pending = stored.pending.take();
                session.pending_since = stored.pending_since;
            }
        }
        session.owns_pending = false;
        if !session.last.is_empty() {
            self.recent_id = Some(session.id.clone());
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
        if self.recent_id.as_ref().is_some_and(|id| !self.inner.contains_key(id)) {
            self.recent_id = None;
        }
    }

    fn make_room(&mut self) {
        // Sessions that wait for an answer go last: new ids cannot push an open confirmation out.
        while self.inner.len() >= MAX_SESSIONS {
            let oldest = self.inner.iter().min_by_key(|(_, s)| (s.pending().is_some(), s.last_used)).map(|(id, _)| id.clone());
            if let Some(id) = oldest {
                self.inner.remove(&id);
            } else {
                break;
            }
        }
    }
}

/// A conversation id the store accepts as a key: not empty, at most 128 bytes, no control characters.
pub fn valid_id(id: &str) -> bool {
    !id.trim().is_empty() && id.len() <= 128 && !id.chars().any(char::is_control)
}

fn same_satellite_hint(session: &Session, recent: &Session) -> bool {
    matches!(
        (session.preferred_area.as_deref(), recent.preferred_area.as_deref()),
        (Some(left), Some(right)) if !left.is_empty() && left == right
    )
}

fn drop_media_followup(session: &mut Session) {
    session.last.retain(|turn| !media_last(turn));
    session.last_execute.retain(|intent| !media_execute(intent));
}

fn media_last(turn: &LastTurn) -> bool {
    turn.domain.as_deref() == Some("media_player") || turn.entity.as_deref().is_some_and(|id| id.starts_with("media_player."))
}

fn media_execute(intent: &Intent) -> bool {
    intent.slot("entity_id").is_some_and(|id| id.starts_with("media_player."))
        || intent.slot("domain") == Some("media_player")
        || intent.name.contains("Volume")
        || intent.name.contains("Media")
        || intent.name.starts_with("Mass")
}

fn truncate_chars(value: &mut String, maximum: usize) {
    if value.chars().count() > maximum {
        *value = value.chars().take(maximum).collect();
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
