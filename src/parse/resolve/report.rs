use super::score::{fuzzy_tokens, score_entity};
use super::{entity_name_is_mentioned, resolve, token_hit, Resolved};
use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::home::roles::matches_domain;
use crate::lang::catalog;
use crate::types::HomeGraph;

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
    let resolved = resolve(tokens, home, domain);
    let fuzzy = fuzzy_tokens(tokens, home);
    let mut ranked: Vec<ResolveEvidence> = home
        .entities
        .iter()
        .filter(|entity| assist_visible(entity, home) && !is_infra(entity))
        .filter(|entity| domain.is_none_or(|wanted| matches_domain(entity, wanted, catalog())))
        .filter_map(|entity| {
            score_entity(tokens, &fuzzy, entity, home).map(|score| ResolveEvidence {
                target: entity.entity_id.clone(),
                kind: "entity",
                score,
                exact: score >= 0.9,
            })
        })
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
