//! Process-wide counters for structured observability (package 3.10).
//!
//! Lives outside `io` so home, lang and HTTP can bump the same surface without cycles.

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

/// Process-wide counters for latency, sessions, overlay writes, catalog cache and Wyoming.
#[derive(Default)]
pub struct OpsCounters {
    requests: AtomicU64,
    latency_total_ms: AtomicU64,
    latency_max_ms: AtomicU64,
    sessions_created: AtomicU64,
    sessions_active: AtomicU64,
    overlay_writes: AtomicU64,
    overlay_write_errors: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_entries: AtomicU64,
    wyoming_connects: AtomicU64,
    wyoming_rejects: AtomicU64,
    wyoming_inflight: AtomicU64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct OpsSnapshot {
    pub requests: u64,
    pub latency_total_ms: u64,
    pub latency_max_ms: u64,
    pub sessions_created: u64,
    pub sessions_active: u64,
    pub overlay_writes: u64,
    pub overlay_write_errors: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_entries: u64,
    pub wyoming_connects: u64,
    pub wyoming_rejects: u64,
    pub wyoming_inflight: u64,
}

pub fn ops() -> &'static OpsCounters {
    static OPS: OnceLock<OpsCounters> = OnceLock::new();
    OPS.get_or_init(OpsCounters::default)
}

impl OpsCounters {
    pub fn record_request(&self, duration_ms: u64) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.latency_total_ms.fetch_add(duration_ms, Ordering::Relaxed);
        self.latency_max_ms.fetch_max(duration_ms, Ordering::Relaxed);
    }

    pub fn record_session_created(&self, active: u64) {
        self.sessions_created.fetch_add(1, Ordering::Relaxed);
        self.sessions_active.store(active, Ordering::Relaxed);
    }

    pub fn sync_sessions(&self, created: u64, active: u64) {
        self.sessions_created.store(created, Ordering::Relaxed);
        self.sessions_active.store(active, Ordering::Relaxed);
    }

    pub fn record_overlay_write(&self) {
        self.overlay_writes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_overlay_write_error(&self) {
        self.overlay_write_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self, entries: u64) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
        self.cache_entries.store(entries, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self, entries: u64) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        self.cache_entries.store(entries, Ordering::Relaxed);
    }

    pub fn record_wyoming_connect(&self) {
        self.wyoming_connects.fetch_add(1, Ordering::Relaxed);
        self.wyoming_inflight.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_wyoming_reject(&self) {
        self.wyoming_rejects.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_wyoming_close(&self) {
        let _ = self.wyoming_inflight.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| Some(n.saturating_sub(1)));
    }

    pub fn snapshot(&self) -> OpsSnapshot {
        OpsSnapshot {
            requests: self.requests.load(Ordering::Relaxed),
            latency_total_ms: self.latency_total_ms.load(Ordering::Relaxed),
            latency_max_ms: self.latency_max_ms.load(Ordering::Relaxed),
            sessions_created: self.sessions_created.load(Ordering::Relaxed),
            sessions_active: self.sessions_active.load(Ordering::Relaxed),
            overlay_writes: self.overlay_writes.load(Ordering::Relaxed),
            overlay_write_errors: self.overlay_write_errors.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            cache_entries: self.cache_entries.load(Ordering::Relaxed),
            wyoming_connects: self.wyoming_connects.load(Ordering::Relaxed),
            wyoming_rejects: self.wyoming_rejects.load(Ordering::Relaxed),
            wyoming_inflight: self.wyoming_inflight.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ops_counters_move_when_exercised() {
        let before = ops().snapshot();
        ops().record_request(12);
        ops().record_session_created(3);
        ops().record_overlay_write();
        ops().record_overlay_write_error();
        ops().record_cache_hit(2);
        ops().record_cache_miss(3);
        ops().record_wyoming_connect();
        ops().record_wyoming_reject();
        ops().record_wyoming_close();
        let after = ops().snapshot();
        assert_eq!(after.requests, before.requests + 1);
        assert_eq!(after.latency_total_ms, before.latency_total_ms + 12);
        assert!(after.latency_max_ms >= 12);
        assert_eq!(after.sessions_created, before.sessions_created + 1);
        assert_eq!(after.sessions_active, 3);
        assert_eq!(after.overlay_writes, before.overlay_writes + 1);
        assert_eq!(after.overlay_write_errors, before.overlay_write_errors + 1);
        assert_eq!(after.cache_hits, before.cache_hits + 1);
        assert_eq!(after.cache_misses, before.cache_misses + 1);
        assert_eq!(after.cache_entries, 3);
        assert_eq!(after.wyoming_connects, before.wyoming_connects + 1);
        assert_eq!(after.wyoming_rejects, before.wyoming_rejects + 1);
        assert_eq!(after.wyoming_inflight, before.wyoming_inflight);
    }
}
