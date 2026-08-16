use crate::nlu;
use crate::session::Session;
use crate::types::{HomeGraph, Settings};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchReport {
    pub samples: u32,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub rss_kb: u64,
}

pub fn bench_warm(home: &HomeGraph, text: &str, repeat: u32) -> BenchReport {
    let settings = Settings { languages: vec!["en".into()], ..Settings::default() };
    let mut session = Session::new();
    let _ = nlu::parse(text, home, &mut session, &[], &settings);
    let mut samples = Vec::with_capacity(repeat as usize);
    for _ in 0..repeat {
        let mut session = Session::new();
        let started = Instant::now();
        let _ = nlu::parse(text, home, &mut session, &[], &settings);
        samples.push(started.elapsed().as_micros() as u64);
    }
    samples.sort_unstable();
    BenchReport {
        samples: repeat,
        p50_us: percentile(&samples, 50),
        p95_us: percentile(&samples, 95),
        p99_us: percentile(&samples, 99),
        rss_kb: current_rss_kb(),
    }
}

fn percentile(sorted: &[u64], pct: u32) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((pct as usize).saturating_mul(sorted.len().saturating_sub(1))) / 100;
    sorted[index]
}

fn current_rss_kb() -> u64 {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}
