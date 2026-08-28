use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::clause::{finish_intents, Clause};
use crate::parse::resolve::unique_in_area;
use crate::parse::slots::{fill_intent, ClauseOut};
use crate::session::Session;
use crate::types::{HomeGraph, Intent};

pub(crate) fn session_climate_cover(ctx: &Clause) -> Option<ClauseOut> {
    let areas = last_turn_areas(ctx.session, ctx.home);
    if areas.len() <= 1 || !matches!(ctx.domain, Some("climate") | Some("cover")) {
        return None;
    }
    let intents = areas
        .into_iter()
        .map(|area| {
            let id = ctx.domain.and_then(|d| unique_in_area(ctx.home, &area, d, ctx.tokens));
            fill_intent(ctx.action, ctx.tokens, ctx.number, id.as_deref(), Some(&area), ctx.domain)
        })
        .collect();
    Some(finish_intents(intents, ctx))
}

pub(crate) fn session_entities(ctx: &Clause) -> Option<ClauseOut> {
    if !allows_session_replay(ctx) {
        return None;
    }
    let intents = replay_session_intents(ctx.session, ctx.home, ctx.tokens, ctx.action, ctx.number, ctx.domain);
    (!intents.is_empty()).then(|| finish_intents(intents, ctx))
}

pub(crate) fn session_areas(ctx: &Clause) -> Option<ClauseOut> {
    if !allows_session_replay(ctx) {
        return None;
    }
    let areas = last_turn_areas(ctx.session, ctx.home);
    (!areas.is_empty()).then(|| {
        let intents = areas
            .into_iter()
            .map(|area| {
                let id = ctx
                    .domain
                    .filter(|d| matches!(*d, "climate" | "media_player" | "fan"))
                    .and_then(|d| unique_in_area(ctx.home, &area, d, ctx.tokens));
                fill_intent(ctx.action, ctx.tokens, ctx.number, id.as_deref(), Some(&area), ctx.domain)
            })
            .collect();
        finish_intents(intents, ctx)
    })
}

pub(crate) fn replay_session_intents(
    session: &Session,
    home: &HomeGraph,
    tokens: &[String],
    action: Action,
    number: Option<i32>,
    domain: Option<&str>,
) -> Vec<Intent> {
    let (entities, areas) = last_turn_targets(session, home, domain);
    if entities.is_empty() && areas.is_empty() {
        return Vec::new();
    }
    let mut intents = Vec::new();
    let mut covered_areas = Vec::new();
    for id in &entities {
        if let Some(area) = entity_area(home, id) {
            if !covered_areas.contains(&area) {
                covered_areas.push(area);
            }
        }
        intents.push(fill_intent(action, tokens, number, Some(id), None, domain));
    }
    for area in areas {
        if covered_areas.iter().any(|have| have == &area) {
            continue;
        }
        let id = domain
            .filter(|wanted| matches!(*wanted, "climate" | "media_player" | "fan"))
            .and_then(|wanted| unique_in_area(home, &area, wanted, tokens));
        intents.push(fill_intent(action, tokens, number, id.as_deref(), Some(&area), domain));
    }
    intents
}

fn allows_session_replay(ctx: &Clause) -> bool {
    !matches!(ctx.action, Action::GetState) || wants_status_query(ctx.tokens)
}

fn wants_status_query(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, cat.status_words())
        || tokens.iter().any(|token| cat.is_query_hint(token) || cat.is_question_word(token) || cat.is_question_start(token))
}

fn last_turn_targets(session: &Session, home: &HomeGraph, domain: Option<&str>) -> (Vec<String>, Vec<String>) {
    let Some(head) = session.last.first() else {
        return (Vec::new(), Vec::new());
    };
    let turn = head.turn;
    let mut entities = Vec::new();
    let mut areas = Vec::new();
    for item in session.last.iter().filter(|item| item.turn == turn) {
        if let Some(id) = item.entity.as_deref() {
            if visible_domain(home, id, domain) && !entities.iter().any(|have| have == id) {
                entities.push(id.to_string());
            }
            if let Some(area) = entity_area(home, id) {
                if !areas.contains(&area) {
                    areas.push(area);
                }
            }
        }
        if let Some(area) = item.area.as_deref() {
            if !areas.iter().any(|have| have == area) {
                areas.push(area.to_string());
            }
        }
    }
    (entities, areas)
}

fn last_turn_areas(session: &Session, home: &HomeGraph) -> Vec<String> {
    last_turn_targets(session, home, None).1
}

fn entity_area(home: &HomeGraph, id: &str) -> Option<String> {
    home.entities.iter().find(|entity| entity.entity_id == id).and_then(|entity| entity.area.clone())
}

fn visible_domain(home: &HomeGraph, id: &str, domain: Option<&str>) -> bool {
    home.entities.iter().any(|entity| {
        entity.entity_id == id
            && assist_visible(entity, home)
            && !is_infra(entity)
            && domain.is_none_or(|wanted| entity.domain == wanted || id.starts_with(&format!("{wanted}.")))
    })
}
