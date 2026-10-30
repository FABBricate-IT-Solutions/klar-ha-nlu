use super::score::{fuzzy_tokens, score_entity};
use super::{entity_name_is_mentioned, resolve_from_candidates, token_hit, Resolved};
use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::home::roles::matches_domain;
use crate::lang::catalog;
use crate::types::{EntityRec, HomeGraph};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static ACTIVE: RefCell<Option<HashMap<(String, String), ResolveReport>>> = const { RefCell::new(None) };
}

pub(crate) struct Guard {
    previous: Option<HashMap<(String, String), ResolveReport>>,
}

impl Guard {
    pub(crate) fn enter() -> Self {
        let previous = ACTIVE.with(|slot| slot.borrow_mut().take());
        ACTIVE.with(|slot| *slot.borrow_mut() = Some(HashMap::new()));
        Self { previous }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        ACTIVE.with(|slot| *slot.borrow_mut() = previous);
    }
}

fn cached(tokens: &[String], domain: Option<&str>) -> Option<ResolveReport> {
    ACTIVE.with(|slot| {
        let map = slot.borrow();
        map.as_ref()?.get(&key(tokens, domain)).cloned()
    })
}

fn store(tokens: &[String], domain: Option<&str>, report: ResolveReport) {
    ACTIVE.with(|slot| {
        if let Some(map) = slot.borrow_mut().as_mut() {
            map.insert(key(tokens, domain), report);
        }
    });
}

fn key(tokens: &[String], domain: Option<&str>) -> (String, String) {
    let mut text = String::new();
    for token in tokens {
        text.push('\u{1f}');
        text.push_str(token);
    }
    (text, domain.unwrap_or("").to_string())
}

#[derive(Debug, Clone)]
pub(crate) struct ResolveEvidence {
    pub target: String,
    pub kind: &'static str,
    pub score: f64,
    pub exact: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolveReport {
    pub resolved: Resolved,
    pub ranked: Vec<ResolveEvidence>,
    pub margin: f64,
}

pub(crate) fn resolve_scored(tokens: &[String], home: &HomeGraph, domain: Option<&str>) -> ResolveReport {
    if let Some(hit) = cached(tokens, domain) {
        return hit;
    }
    let fuzzy = fuzzy_tokens(tokens, home);
    let scored = collect_scored(tokens, home, domain, &fuzzy);
    let candidates = scored.iter().filter(|(_, entity)| !crate::parse::calendar::is_calendar_control(entity)).cloned().collect();
    let resolved = resolve_from_candidates(tokens, home, domain, candidates);
    let report = rank(tokens, home, scored, resolved);
    store(tokens, domain, report.clone());
    report
}

fn collect_scored(tokens: &[String], home: &HomeGraph, domain: Option<&str>, fuzzy: &[&str]) -> Vec<(f64, EntityRec)> {
    home.entities
        .iter()
        .filter(|entity| assist_visible(entity, home) && !is_infra(entity) && !crate::home::policy::is_nlu_ignored(entity))
        .filter(|entity| domain.is_none_or(|wanted| matches_domain(entity, wanted, catalog())))
        .filter_map(|entity| score_entity(tokens, fuzzy, entity, home).map(|score| (score, entity.clone())))
        .collect()
}

fn rank(tokens: &[String], home: &HomeGraph, scored: Vec<(f64, EntityRec)>, resolved: Resolved) -> ResolveReport {
    let mut ranked: Vec<ResolveEvidence> = scored
        .into_iter()
        .map(|(score, entity)| ResolveEvidence { target: entity.entity_id, kind: "entity", score, exact: score >= 0.9 })
        .collect();
    for entity in &resolved.entities {
        if ranked.iter().any(|row| row.target == entity.entity_id) {
            continue;
        }
        let explicit = entity_name_is_mentioned(tokens, entity, home);
        ranked.push(ResolveEvidence {
            target: entity.entity_id.clone(),
            kind: "entity",
            score: if explicit { 0.92 } else { 0.82 },
            exact: explicit,
        });
    }
    for area in &resolved.areas {
        let exact = tokens.iter().any(|token| token == area)
            || home
                .areas
                .iter()
                .find(|record| record.area_id == *area)
                .is_some_and(|record| token_hit(tokens, &record.name) || record.aliases.iter().any(|alias| token_hit(tokens, alias)));
        ranked.push(ResolveEvidence { target: area.clone(), kind: "area", score: if exact { 1.0 } else { 0.88 }, exact });
    }
    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.kind.cmp(right.kind))
            .then_with(|| left.target.cmp(&right.target))
    });
    ranked.dedup_by(|left, right| left.target == right.target && left.kind == right.kind);
    let margin = ranked.first().map(|best| best.score - ranked.get(1).map_or(0.0, |next| next.score)).unwrap_or(0.0);
    ResolveReport { resolved, ranked, margin }
}
