use crate::home::assignment::{Traffic, TrafficPoint, TrafficRecent};
use crate::types::ParseResult;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const RECENT_CAP: usize = 30;

#[derive(Default)]
struct MetricsInner {
    total: usize,
    by_source: BTreeMap<String, usize>,
    by_intent: BTreeMap<String, usize>,
    by_day: BTreeMap<String, usize>,
    clarify: usize,
    chat: usize,
    empty: usize,
    recent: Vec<TrafficRecent>,
}

#[derive(Default)]
pub struct MetricsStore {
    inner: Mutex<MetricsInner>,
}

impl MetricsStore {
    pub fn record(&self, source: &str, language: Option<&str>, result: &ParseResult) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let ts_ms = now_ms();
        inner.total += 1;
        *inner.by_source.entry(source.into()).or_default() += 1;
        *inner.by_day.entry(day_bucket(ts_ms)).or_default() += 1;
        if result.clarify {
            inner.clarify += 1;
        }
        if result.chat {
            inner.chat += 1;
        }
        if result.intents.is_empty() {
            inner.empty += 1;
        }
        for intent in &result.intents {
            *inner.by_intent.entry(intent.name.clone()).or_default() += 1;
        }
        let id = format!("{ts_ms}-{}", inner.total);
        inner.recent.push(TrafficRecent {
            id,
            ts_ms,
            source: source.into(),
            language: language.map(str::to_string),
            text: result.text.clone(),
            speech: result.speech.clone(),
            intents: result.intents.iter().map(|intent| intent.name.clone()).collect(),
            clarify: result.clarify,
            chat: result.chat,
        });
        if inner.recent.len() > RECENT_CAP {
            let drop = inner.recent.len() - RECENT_CAP;
            inner.recent.drain(0..drop);
        }
    }

    pub fn snapshot(&self) -> Traffic {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        Traffic {
            total: inner.total,
            by_source: inner.by_source.clone(),
            by_intent: inner.by_intent.clone(),
            by_day: inner.by_day.iter().map(|(day, count)| TrafficPoint { day: day.clone(), count: *count }).collect(),
            clarify: inner.clarify,
            chat: inner.chat,
            empty: inner.empty,
            recent: inner.recent.clone(),
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn day_bucket(ts_ms: u64) -> String {
    format!("d{}", ts_ms / 86_400_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Intent;

    #[test]
    fn records_live_parse_metrics() {
        let store = MetricsStore::default();
        let result = ParseResult {
            text: "Licht an".into(),
            intents: vec![Intent::new("HassTurnOn")],
            speech: "ok".into(),
            clarify: false,
            conversation_id: "c1".into(),
            chat: false,
            briefing: false,
        };
        store.record("http", Some("de"), &result);
        let out = store.snapshot();
        assert_eq!(out.total, 1);
        assert_eq!(out.by_source["http"], 1);
        assert_eq!(out.by_intent["HassTurnOn"], 1);
        assert_eq!(out.recent[0].text, "Licht an");
    }
}
