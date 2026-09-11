//! Rolling window of engine LLM calls. No prompts or completions.

use crate::llm::TokenUsage;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const MAX_CALLS: usize = 200;
const TTL_MS: u64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LlmCall {
    pub kind: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted: Option<bool>,
    pub latency_ms: u64,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct LlmDay {
    pub day: String,
    pub refine: usize,
    pub assist: usize,
    pub chat: usize,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct LlmTokens {
    pub prompt: u64,
    pub completion: u64,
    pub total: u64,
    pub calls_with_usage: usize,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct LlmWindow {
    pub calls: usize,
    pub errors: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub by_day: Vec<LlmDay>,
    pub p50_ms: Option<u64>,
    pub p90_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<LlmTokens>,
}

#[derive(Default)]
pub struct LlmCallStore {
    lock: Mutex<Vec<LlmCall>>,
}

impl LlmCallStore {
    pub fn record(&self, call: LlmCall) {
        let mut calls = self.lock.lock().unwrap_or_else(|err| err.into_inner());
        calls.push(call);
        prune(&mut calls, now_ms());
    }

    pub fn snapshot(&self) -> LlmWindow {
        let mut calls = self.lock.lock().unwrap_or_else(|err| err.into_inner());
        prune(&mut calls, now_ms());
        window_of(&calls)
    }
}

pub fn record_llm(
    store: &LlmCallStore,
    kind: &str,
    started: Instant,
    model: &str,
    ok: bool,
    accepted: Option<bool>,
    usage: Option<&TokenUsage>,
) {
    store.record(LlmCall {
        kind: sanitize_kind(kind).into(),
        ok,
        accepted,
        latency_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        model: model.chars().take(128).collect(),
        prompt_tokens: usage.and_then(|row| row.prompt_tokens),
        completion_tokens: usage.and_then(|row| row.completion_tokens),
        total_tokens: usage.and_then(|row| row.total_tokens),
        ts_ms: now_ms(),
    });
}

fn sanitize_kind(kind: &str) -> &'static str {
    match kind {
        "refine" => "refine",
        "assist" => "assist",
        _ => "chat",
    }
}

fn prune(calls: &mut Vec<LlmCall>, now: u64) {
    let cutoff = now.saturating_sub(TTL_MS);
    calls.retain(|call| call.ts_ms >= cutoff);
    if calls.len() > MAX_CALLS {
        let drop = calls.len() - MAX_CALLS;
        calls.drain(0..drop);
    }
}

fn window_of(calls: &[LlmCall]) -> LlmWindow {
    let mut by_kind = BTreeMap::new();
    let mut by_day: BTreeMap<String, LlmDay> = BTreeMap::new();
    let mut latencies = Vec::new();
    let mut tokens = LlmTokens::default();
    let mut errors = 0;
    let mut accepted = 0;
    let mut rejected = 0;
    for call in calls {
        *by_kind.entry(call.kind.clone()).or_default() += 1;
        let day = day_bucket(call.ts_ms);
        let row = by_day.entry(day.clone()).or_insert_with(|| LlmDay { day, ..LlmDay::default() });
        match call.kind.as_str() {
            "refine" => row.refine += 1,
            "assist" => row.assist += 1,
            _ => row.chat += 1,
        }
        if !call.ok {
            errors += 1;
        }
        match call.accepted {
            Some(true) => accepted += 1,
            Some(false) => rejected += 1,
            None => {}
        }
        latencies.push(call.latency_ms);
        if call.prompt_tokens.is_some() || call.completion_tokens.is_some() || call.total_tokens.is_some() {
            tokens.calls_with_usage += 1;
            tokens.prompt += call.prompt_tokens.unwrap_or(0);
            tokens.completion += call.completion_tokens.unwrap_or(0);
            tokens.total += call.total_tokens.unwrap_or(call.prompt_tokens.unwrap_or(0) + call.completion_tokens.unwrap_or(0));
        }
    }
    latencies.sort_unstable();
    LlmWindow {
        calls: calls.len(),
        errors,
        accepted,
        rejected,
        by_kind,
        by_day: by_day.into_values().collect(),
        p50_ms: percentile(&latencies, 0.50),
        p90_ms: percentile(&latencies, 0.90),
        tokens: (tokens.calls_with_usage > 0).then_some(tokens),
    }
}

fn percentile(sorted: &[u64], p: f64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    Some(sorted[idx.min(sorted.len() - 1)])
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

    fn call(kind: &str, ok: bool, accepted: Option<bool>, latency_ms: u64, usage: Option<(u64, u64, u64)>) -> LlmCall {
        LlmCall {
            kind: kind.into(),
            ok,
            accepted,
            latency_ms,
            model: "gemma".into(),
            prompt_tokens: usage.map(|row| row.0),
            completion_tokens: usage.map(|row| row.1),
            total_tokens: usage.map(|row| row.2),
            ts_ms: now_ms(),
        }
    }

    #[test]
    fn snapshot_mixes_kinds_and_skips_tokens_without_usage() {
        let store = LlmCallStore::default();
        store.record(call("refine", true, Some(true), 40, None));
        store.record(call("refine", true, Some(false), 80, None));
        store.record(call("assist", false, None, 120, None));
        store.record(call("chat", true, None, 20, None));
        let snap = store.snapshot();
        assert_eq!(snap.calls, 4);
        assert_eq!(snap.errors, 1);
        assert_eq!(snap.accepted, 1);
        assert_eq!(snap.rejected, 1);
        assert_eq!(snap.by_kind.get("refine"), Some(&2));
        assert_eq!(snap.by_kind.get("chat"), Some(&1));
        assert_eq!(snap.p50_ms, Some(80));
        assert!(snap.tokens.is_none());
        let json = serde_json::to_value(&snap).unwrap();
        assert!(json.get("tokens").is_none());
        assert!(!json.to_string().contains("prompt"));
    }

    #[test]
    fn snapshot_keeps_usage_when_upstream_sent_it() {
        let store = LlmCallStore::default();
        store.record(call("chat", true, None, 10, Some((11, 7, 18))));
        let snap = store.snapshot();
        let tokens = snap.tokens.expect("usage");
        assert_eq!(tokens.prompt, 11);
        assert_eq!(tokens.completion, 7);
        assert_eq!(tokens.total, 18);
        assert_eq!(tokens.calls_with_usage, 1);
    }

    #[test]
    fn drops_old_and_caps_window() {
        let store = LlmCallStore::default();
        for i in 0..220 {
            store.record(LlmCall {
                kind: "chat".into(),
                ok: true,
                accepted: None,
                latency_ms: 1,
                model: "m".into(),
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
                ts_ms: now_ms().saturating_sub(if i < 10 { TTL_MS + 1 } else { 0 }),
            });
        }
        let snap = store.snapshot();
        assert!(snap.calls <= MAX_CALLS);
        assert!(snap.calls >= 200 - 10);
    }
}
