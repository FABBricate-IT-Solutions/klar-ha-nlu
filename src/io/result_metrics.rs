//! Local result metrics for operators (package 6.8): decision mix, top rejection
//! forms, repeats within 30 s, matcher hit rates, and an optional redacted export.

use crate::io::auth::reads_allowed;
use crate::io::state::AppState;
use crate::types::{ParseDecision, ParseOutcome, RejectReason};
use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::Serialize;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const WEEK_MS: u64 = 7 * 24 * 60 * 60 * 1000;
const REPEAT_MS: u64 = 30_000;
const MAX_EVENTS: usize = 4_000;
const TOP_N: usize = 10;
const MAX_CANDIDATE_POLICIES: usize = 8;

#[derive(Debug, Clone)]
struct ResultEvent {
    ts_ms: u64,
    decision: &'static str,
    reject_reason: Option<&'static str>,
    fingerprint: u64,
    matcher: Option<String>,
    candidate_policies: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct RejectionCount {
    pub reason: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct MatcherHitRate {
    pub id: String,
    pub hits: usize,
    pub attempts: usize,
    /// hits / attempts, 0.0 when attempts is zero.
    pub rate: f64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct DecisionMixCounts {
    pub execute: usize,
    pub confirm: usize,
    pub clarify: usize,
    pub reject: usize,
    pub chat: usize,
    pub error: usize,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct ResultWindow {
    pub total: usize,
    pub window_ms: u64,
    pub decision_mix: DecisionMixCounts,
    /// Up to ten most frequent reject reasons in the rolling week.
    pub top_rejections: Vec<RejectionCount>,
    pub repeats_within_30s: usize,
    pub matcher_hit_rates: Vec<MatcherHitRate>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct ResultExport {
    pub schema: &'static str,
    pub generated_ts_ms: u64,
    pub window: ResultWindow,
    /// Day buckets of decision counts only — no utterances.
    pub by_day: Vec<ResultDay>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct ResultDay {
    pub day: String,
    pub execute: usize,
    pub confirm: usize,
    pub clarify: usize,
    pub reject: usize,
    pub chat: usize,
    pub error: usize,
}

#[derive(Default)]
pub struct ResultMetricsStore {
    inner: Mutex<Vec<ResultEvent>>,
}

impl ResultMetricsStore {
    pub fn record(&self, outcome: &ParseOutcome) {
        self.record_at(outcome, now_ms());
    }

    /// Test hook: inject an event at a fixed timestamp.
    pub fn record_at(&self, outcome: &ParseOutcome, ts_ms: u64) {
        let decision = decision_name(&outcome.decision);
        let reject_reason = match &outcome.decision {
            ParseDecision::Reject { reason } => Some(reject_wire(reason)),
            _ => None,
        };
        let fingerprint = fingerprint_text(&outcome.text);
        let matcher = selected_matcher(outcome);
        let candidate_policies =
            outcome.candidates.iter().filter_map(|row| sanitize_policy(&row.policy)).take(MAX_CANDIDATE_POLICIES).collect();
        let event = ResultEvent { ts_ms, decision, reject_reason, fingerprint, matcher, candidate_policies };
        let mut events = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        events.push(event);
        prune(&mut events, ts_ms);
    }

    pub fn snapshot(&self) -> ResultWindow {
        let mut events = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let now = now_ms();
        prune(&mut events, now);
        window_of(&events)
    }

    pub fn export(&self) -> ResultExport {
        let mut events = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let now = now_ms();
        prune(&mut events, now);
        ResultExport { schema: "klar.result_metrics.v1", generated_ts_ms: now, window: window_of(&events), by_day: by_day_of(&events) }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v2/result-metrics/export", get(export_metrics))
}

async fn export_metrics(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let body = serde_json::to_vec_pretty(&state.result_metrics.export()).unwrap_or_else(|_| b"{}".to_vec());
    let mut response = body.into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename=\"klar-result-metrics.json\""));
    Ok(response)
}

fn window_of(events: &[ResultEvent]) -> ResultWindow {
    let mut mix = DecisionMixCounts::default();
    let mut rejects: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut attempts: BTreeMap<String, usize> = BTreeMap::new();
    let mut hits: BTreeMap<String, usize> = BTreeMap::new();
    let mut repeats = 0usize;
    for (index, event) in events.iter().enumerate() {
        bump_mix(&mut mix, event.decision);
        if let Some(reason) = event.reject_reason {
            *rejects.entry(reason).or_default() += 1;
        }
        for policy in &event.candidate_policies {
            *attempts.entry(policy.clone()).or_default() += 1;
        }
        if let Some(matcher) = &event.matcher {
            *hits.entry(matcher.clone()).or_default() += 1;
            attempts.entry(matcher.clone()).or_insert(0);
        }
        if is_repeat(events, index) {
            repeats += 1;
        }
    }
    let mut top_rejections: Vec<RejectionCount> =
        rejects.into_iter().map(|(reason, count)| RejectionCount { reason: reason.into(), count }).collect();
    top_rejections.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.reason.cmp(&b.reason)));
    top_rejections.truncate(TOP_N);

    let mut matcher_hit_rates: Vec<MatcherHitRate> = attempts
        .into_iter()
        .map(|(id, attempt_count)| {
            let hit_count = hits.get(&id).copied().unwrap_or(0);
            let rate = if attempt_count == 0 { 0.0 } else { hit_count as f64 / attempt_count as f64 };
            MatcherHitRate { id, hits: hit_count, attempts: attempt_count, rate }
        })
        .collect();
    matcher_hit_rates.sort_by(|a, b| b.attempts.cmp(&a.attempts).then_with(|| a.id.cmp(&b.id)));
    matcher_hit_rates.truncate(TOP_N);

    ResultWindow {
        total: events.len(),
        window_ms: WEEK_MS,
        decision_mix: mix,
        top_rejections,
        repeats_within_30s: repeats,
        matcher_hit_rates,
    }
}

fn by_day_of(events: &[ResultEvent]) -> Vec<ResultDay> {
    let mut days: BTreeMap<String, ResultDay> = BTreeMap::new();
    for event in events {
        let day = day_bucket(event.ts_ms);
        let row = days.entry(day.clone()).or_insert_with(|| ResultDay { day, ..ResultDay::default() });
        match event.decision {
            "execute" => row.execute += 1,
            "confirm" => row.confirm += 1,
            "clarify" => row.clarify += 1,
            "reject" => row.reject += 1,
            "chat" => row.chat += 1,
            _ => row.error += 1,
        }
    }
    days.into_values().collect()
}

fn is_repeat(events: &[ResultEvent], index: usize) -> bool {
    let event = &events[index];
    if event.fingerprint == 0 {
        return false;
    }
    let start = event.ts_ms.saturating_sub(REPEAT_MS);
    events[..index].iter().any(|prior| prior.fingerprint == event.fingerprint && prior.ts_ms >= start && prior.ts_ms <= event.ts_ms)
}

fn bump_mix(mix: &mut DecisionMixCounts, decision: &str) {
    match decision {
        "execute" => mix.execute += 1,
        "confirm" => mix.confirm += 1,
        "clarify" => mix.clarify += 1,
        "reject" => mix.reject += 1,
        "chat" => mix.chat += 1,
        _ => mix.error += 1,
    }
}

fn prune(events: &mut Vec<ResultEvent>, now: u64) {
    let cutoff = now.saturating_sub(WEEK_MS);
    events.retain(|event| event.ts_ms >= cutoff);
    if events.len() > MAX_EVENTS {
        let drop = events.len() - MAX_EVENTS;
        events.drain(0..drop);
    }
}

fn decision_name(decision: &ParseDecision) -> &'static str {
    match decision {
        ParseDecision::Execute => "execute",
        ParseDecision::Clarify { .. } => "clarify",
        ParseDecision::Confirm { .. } => "confirm",
        ParseDecision::Reject { .. } => "reject",
        ParseDecision::Chat => "chat",
        ParseDecision::Error { .. } => "error",
    }
}

fn reject_wire(reason: &RejectReason) -> &'static str {
    match reason {
        RejectReason::EmptyInput => "empty_input",
        RejectReason::InvalidInput => "invalid_input",
        RejectReason::NoAction => "no_action",
        RejectReason::OutOfDomain => "out_of_domain",
        RejectReason::NoTarget => "no_target",
        RejectReason::Ambiguous => "ambiguous",
        RejectReason::Unsafe => "unsafe",
        RejectReason::Unsupported => "unsupported",
    }
}

fn selected_matcher(outcome: &ParseOutcome) -> Option<String> {
    let id = outcome.selected_candidate_id.as_deref()?;
    outcome.candidates.iter().find(|row| row.id == id).and_then(|row| sanitize_policy(&row.policy))
}

fn sanitize_policy(raw: &str) -> Option<String> {
    let id = raw.trim();
    if id.is_empty() || id.chars().count() > 64 || id.chars().any(char::is_control) {
        return None;
    }
    Some(id.to_string())
}

fn fingerprint_text(text: &str) -> u64 {
    let normalized: String = text.chars().filter(|ch| !ch.is_control()).flat_map(char::to_lowercase).collect();
    let trimmed = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.is_empty() {
        return 0;
    }
    fnv1a64(trimmed.as_bytes())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

fn day_bucket(ts_ms: u64) -> String {
    format!("d{}", ts_ms / 86_400_000)
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Evidence, IntentCandidate, IntentPlan, PlanStep};

    fn outcome(decision: ParseDecision, text: &str) -> ParseOutcome {
        ParseOutcome {
            schema_version: "2.0".into(),
            text: text.into(),
            conversation_id: "c1".into(),
            decision,
            speech: String::new(),
            confidence: 0.0,
            margin: 0.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            news_topic: None,
            retrieval: None,
            policy_trace: None,
            clock_answer: false,
            quiet_ack_eligible: false,
            refine_band: None,
            teach: None,
        }
    }

    fn with_matcher(mut outcome: ParseOutcome, policy: &str) -> ParseOutcome {
        let plan = IntentPlan {
            confidence: 0.9,
            margin: 0.1,
            evidence: vec![Evidence { kind: "action".into(), source: "t".into(), value: "on".into(), score: 1.0, exact: true }],
            steps: vec![PlanStep { index: 0, intent: crate::types::Intent::new("HassTurnOn"), confidence: 0.9, evidence: Vec::new() }],
        };
        outcome.selected_candidate_id = Some("sel".into());
        outcome.candidates = vec![IntentCandidate {
            id: "sel".into(),
            plan,
            score: 0.9,
            margin: 0.1,
            policy: policy.into(),
            precedence: 1,
            evidence: Vec::new(),
        }];
        outcome
    }

    #[test]
    fn top_ten_weekly_rejections_ranked_by_count() {
        let store = ResultMetricsStore::default();
        let now = now_ms();
        for _ in 0..5 {
            store.record_at(&outcome(ParseDecision::Reject { reason: RejectReason::NoAction }, "x"), now);
        }
        for _ in 0..3 {
            store.record_at(&outcome(ParseDecision::Reject { reason: RejectReason::NoTarget }, "y"), now);
        }
        for reason in [
            RejectReason::OutOfDomain,
            RejectReason::Ambiguous,
            RejectReason::Unsafe,
            RejectReason::Unsupported,
            RejectReason::EmptyInput,
            RejectReason::InvalidInput,
            RejectReason::OutOfDomain,
            RejectReason::Ambiguous,
        ] {
            store.record_at(&outcome(ParseDecision::Reject { reason }, "z"), now);
        }
        // Outside the week — must not appear.
        store.record_at(&outcome(ParseDecision::Reject { reason: RejectReason::NoAction }, "old"), now.saturating_sub(WEEK_MS + 1));
        let snap = store.snapshot();
        assert_eq!(snap.top_rejections.len(), 8);
        assert_eq!(snap.top_rejections[0].reason, "no_action");
        assert_eq!(snap.top_rejections[0].count, 5);
        assert_eq!(snap.top_rejections[1].reason, "no_target");
        assert_eq!(snap.top_rejections[1].count, 3);
        assert!(snap.top_rejections.len() <= TOP_N);
        assert_eq!(snap.decision_mix.reject, 5 + 3 + 8);
    }

    #[test]
    fn counts_repeats_within_thirty_seconds() {
        let store = ResultMetricsStore::default();
        let now = now_ms();
        store.record_at(&outcome(ParseDecision::Chat, "lights please"), now);
        store.record_at(&outcome(ParseDecision::Chat, "lights please"), now + 10_000);
        // 40s later still repeats the event 30s earlier (now+10s).
        store.record_at(&outcome(ParseDecision::Chat, "lights please"), now + 40_000);
        store.record_at(&outcome(ParseDecision::Chat, "other phrase"), now + 50_000);
        let snap = store.snapshot();
        assert_eq!(snap.repeats_within_30s, 2);
        assert_eq!(snap.decision_mix.chat, 4);
    }

    #[test]
    fn matcher_hit_rates_from_selected_candidate() {
        let store = ResultMetricsStore::default();
        let now = now_ms();
        store.record_at(&with_matcher(outcome(ParseDecision::Execute, "on"), "grounded_entities"), now);
        store.record_at(&with_matcher(outcome(ParseDecision::Execute, "on"), "grounded_entities"), now);
        store.record_at(&with_matcher(outcome(ParseDecision::Clarify { prompt: "?".into(), options: vec![] }, "which"), "all_lights"), now);
        let snap = store.snapshot();
        let grounded = snap.matcher_hit_rates.iter().find(|row| row.id == "grounded_entities").expect("grounded");
        assert_eq!(grounded.hits, 2);
        assert_eq!(grounded.attempts, 2);
        assert!((grounded.rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn export_is_redacted_aggregates_only() {
        let store = ResultMetricsStore::default();
        store.record(&outcome(ParseDecision::Reject { reason: RejectReason::NoAction }, "secret bedroom phrase"));
        let json = serde_json::to_string(&store.export()).unwrap();
        assert!(!json.contains("secret"));
        assert!(!json.contains("bedroom"));
        assert!(json.contains("no_action"));
        assert!(json.contains("klar.result_metrics.v1"));
    }
}
