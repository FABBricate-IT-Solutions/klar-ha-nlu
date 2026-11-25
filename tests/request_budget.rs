//! Package 3.4: request budget and scaling gates (R-07..R-10, Q-12).

mod common;

use common::home::{area, ent, home};
use klar_nlu::home::rebuild_label_index;
use klar_nlu::home::snapshot::{ingest, HomeSnapshot, SnapshotEntity, HOME_SCHEMA_VERSION, MAX_ALIAS_CHARS, MAX_SNAPSHOT_ENTITIES};
use klar_nlu::nlu::parse;
use klar_nlu::parse::normalize::tokenize;
use klar_nlu::parse::resolve::{resolve, MAX_FUZZY_TOKENS};
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Settings};
use std::time::{Duration, Instant};

fn large_home(count: usize) -> HomeGraph {
    let areas = (0..24).map(|index| area(&format!("area_{index}"), &format!("Area {index}"))).collect();
    let entities = (0..count)
        .map(|index| ent(&format!("light.fixture_{index}"), &format!("Fixture {index}"), &format!("area_{}", index % 24)))
        .collect();
    let mut graph = home(areas, entities, &[]);
    rebuild_label_index(&mut graph);
    graph
}

fn percentile(sorted: &[Duration], fraction: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let index = ((sorted.len() as f64 - 1.0) * fraction).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[test]
fn r07_long_unknown_cjk_stays_under_budget() {
    let _zh = klar_nlu::lang::bind(&["zh-CN".into()]);
    let text: String = std::iter::repeat_n('灯', 2000).collect();
    let started = Instant::now();
    let tokens = tokenize(&text);
    let elapsed = started.elapsed();
    assert_eq!(tokens.len(), 2000);
    assert!(elapsed.as_millis() < 250, "R-07: tokenize(2000 CJK) took {} ms", elapsed.as_millis());
}

#[test]
fn r08_many_unknown_latin_tokens_stay_under_budget() {
    let graph = large_home(256);
    let mut tokens: Vec<String> = vec!["turn".into(), "on".into()];
    tokens.extend((0..256).map(|index| format!("zz{index:03}")));
    let started = Instant::now();
    let _ = resolve(&tokens, &graph, Some("light"));
    let elapsed = started.elapsed();
    assert!(elapsed.as_millis() < 500, "R-08: resolve(256 unknown tokens) took {} ms (fuzzy cap {MAX_FUZZY_TOKENS})", elapsed.as_millis());
}

#[test]
fn r09_large_home_parse_p95_under_budget() {
    let graph = large_home(MAX_SNAPSHOT_ENTITIES);
    let settings = Settings::pinned("en");
    let mut samples = Vec::with_capacity(40);
    // Warm the catalog and label paths once.
    let _ = parse("turn on fixture 1", &graph, &mut Session::new(), &[], &settings);
    for index in 0..40 {
        let text = format!("turn on fixture {}", index % 64);
        let started = Instant::now();
        let _ = parse(&text, &graph, &mut Session::new(), &[], &settings);
        samples.push(started.elapsed());
    }
    samples.sort();
    let p95 = percentile(&samples, 0.95);
    // Acceptance target is 20 ms on x86 Release. Test/CI debug builds stay looser.
    let limit_ms = if cfg!(debug_assertions) { 250 } else { 20 };
    assert!(
        p95.as_millis() <= limit_ms,
        "R-09: p95 {} ms over {limit_ms} ms at {} entities (debug={})",
        p95.as_millis(),
        MAX_SNAPSHOT_ENTITIES,
        cfg!(debug_assertions)
    );
}

#[test]
fn r10_filter_then_cap_keeps_oversized_homes() {
    let entities = (0..MAX_SNAPSHOT_ENTITIES + 800)
        .map(|index| SnapshotEntity {
            entity_id: format!("sensor.meter_{index}"),
            name: Some("Meter".into()),
            original_name: None,
            has_entity_name: false,
            area_id: None,
            device_id: None,
            platform: None,
            aliases: Vec::new(),
            labels: Vec::new(),
            disabled: false,
            device_class: None,
        })
        .chain(std::iter::once(SnapshotEntity {
            entity_id: "light.kept".into(),
            name: Some("Kept".into()),
            original_name: None,
            has_entity_name: false,
            area_id: None,
            device_id: None,
            platform: None,
            aliases: vec!["x".repeat(MAX_ALIAS_CHARS + 40)],
            labels: Vec::new(),
            disabled: false,
            device_class: None,
        }))
        .collect();
    let home = ingest(HomeSnapshot {
        schema_version: HOME_SCHEMA_VERSION.into(),
        entities,
        devices: Vec::new(),
        areas: Vec::new(),
        floors: Vec::new(),
        labels: Vec::new(),
        assist: None,
        registered_intents: Vec::new(),
    })
    .expect("filter then cap");
    assert_eq!(home.entities.len(), 1);
    assert_eq!(home.entities[0].aliases[0].chars().count(), MAX_ALIAS_CHARS);
    assert!(home.label_index.is_some());
}

#[test]
fn q12_leak_gate_repeated_cjk_tokenize_stays_flat() {
    let _zh = klar_nlu::lang::bind(&["zh-CN".into()]);
    let text: String = std::iter::repeat_n('灯', 1200).collect();
    let mut times = Vec::new();
    for _ in 0..30 {
        let started = Instant::now();
        let _ = tokenize(&text);
        times.push(started.elapsed());
    }
    let first = times[..5].iter().sum::<Duration>() / 5;
    let last = times[times.len() - 5..].iter().sum::<Duration>() / 5;
    assert!(
        last <= first.saturating_mul(3).saturating_add(Duration::from_millis(50)),
        "Q-12 leak: early avg {:?} vs late avg {:?}",
        first,
        last
    );
}
