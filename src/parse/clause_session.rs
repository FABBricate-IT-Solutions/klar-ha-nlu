use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::clause::{finish_intents, Clause};
use crate::parse::resolve::unique_in_area;
use crate::parse::slots::{fill_intent, ClauseOut};
use crate::session::{LastTurn, Session};
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
    if !allows_session_replay(ctx) || which_lights_query(ctx.tokens) {
        return None;
    }
    let intents = replay_session_intents(ctx.session, ctx.home, ctx.tokens, ctx.action, ctx.number, ctx.domain);
    (!intents.is_empty()).then(|| finish_intents(intents, ctx))
}

pub(crate) fn session_areas(ctx: &Clause) -> Option<ClauseOut> {
    if !allows_session_replay(ctx) {
        return None;
    }
    let areas =
        if which_lights_query(ctx.tokens) { session_light_areas(ctx.session, ctx.home) } else { last_turn_areas(ctx.session, ctx.home) };
    (!areas.is_empty()).then(|| {
        let intents = areas
            .into_iter()
            .map(|area| {
                let id = ctx
                    .domain
                    .filter(|d| matches!(*d, "media_player" | "fan") || (*d == "climate" && !matches!(ctx.action, Action::GetState)))
                    .and_then(|d| unique_in_area(ctx.home, &area, d, ctx.tokens));
                let domain = ctx.domain.or_else(|| remembered_domain(ctx.session, ctx.home, &area));
                fill_intent(ctx.action, ctx.tokens, ctx.number, id.as_deref(), Some(&area), domain)
            })
            .collect();
        finish_intents(intents, ctx)
    })
}

/// The one kind of device the last turn touched in `area`. "Turn them off" after "turn on
/// the lights in the garage" means those lights, not everything in the garage.
fn remembered_domain<'a>(session: &'a Session, home: &HomeGraph, area: &str) -> Option<&'a str> {
    let turn = session.last.first()?.turn;
    let there = |item: &&LastTurn| {
        item.turn == turn
            && (item.area.as_deref() == Some(area) || item.entity.as_deref().and_then(|id| entity_area(home, id)).as_deref() == Some(area))
    };
    let mut domains = session.last.iter().filter(there).map(|item| item.domain.as_deref());
    let first = domains.next()??;
    domains.all(|domain| domain == Some(first)).then_some(first)
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
        let domain = domain.or_else(|| remembered_domain(session, home, &area));
        intents.push(fill_intent(action, tokens, number, id.as_deref(), Some(&area), domain));
    }
    intents
}

fn allows_session_replay(ctx: &Clause) -> bool {
    !matches!(ctx.action, Action::GetState) || wants_status_query(ctx.tokens)
}

pub(crate) fn which_lights_query(tokens: &[String]) -> bool {
    let cat = catalog();
    // Plural only: "which lights are on" — not OOD trivia with singular "light".
    if !cat.any(tokens, cat.light_plural()) {
        return false;
    }
    // True list asks only: which / are any / how many — not "status of the lights"
    // or named group "Alle Lichter" (those stay entity GetState).
    let which = tokens.iter().any(|token| matches!(token.as_str(), "which" | "welche" | "welcher" | "welches" | "welchen" | "welch"));
    let how_many = tokens
        .windows(2)
        .any(|pair| (pair[0] == "how" && pair[1] == "many") || (pair[0] == "wie" && matches!(pair[1].as_str(), "viele" | "viel")));
    let any_on = tokens.iter().any(|token| matches!(token.as_str(), "any" | "irgendwelche" | "irgendeine" | "irgendein"));
    (which || how_many || any_on) && wants_status_query(tokens)
}

pub(crate) fn session_has_light_areas(session: &Session, home: &HomeGraph) -> bool {
    !session_light_areas(session, home).is_empty()
}

fn session_light_areas(session: &Session, home: &HomeGraph) -> Vec<String> {
    let mut areas = Vec::new();
    for item in session.last.iter().rev() {
        let light = item.entity.as_deref().is_some_and(|id| id.starts_with("light."))
            || item.domain.as_deref() == Some("light")
            || matches!(item.name.as_str(), "HassTurnOn" | "HassTurnOff" | "HassLightSet");
        if !light {
            continue;
        }
        if let Some(area) = item.area.as_deref() {
            if !areas.iter().any(|have| have == area) {
                areas.push(area.to_string());
            }
            continue;
        }
        if let Some(area) = item.entity.as_deref().and_then(|id| entity_area(home, id)) {
            if !areas.contains(&area) {
                areas.push(area);
            }
        }
    }
    areas
}

fn wants_status_query(tokens: &[String]) -> bool {
    let cat = catalog();
    if cat.any(tokens, cat.status_words()) {
        return true;
    }
    tokens.iter().enumerate().any(|(index, token)| {
        if cat.is_question_word(token) {
            return true;
        }
        // Bare mid-sentence "ist"/"are" is ASR chatter, not a status ask. Keep real
        // "Ist Flur an" / "Which lights are on?" via start position or power neighbor.
        if matches!(token.as_str(), "ist" | "sind" | "is" | "are") {
            return copula_status_context(tokens, index);
        }
        cat.is_query_hint(token) || cat.is_question_start(token)
    })
}

fn copula_status_context(tokens: &[String], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    let cat = catalog();
    let power = |token: &str| cat.on_words().contains(token) || cat.off_words().contains(token);
    tokens[..index].iter().any(|prev| {
        cat.is_question_word(prev)
            || cat.status_words().contains(prev.as_str())
            || matches!(prev.as_str(), "if" | "ob" | "whether" | "confirm" | "which" | "welche")
            || (cat.is_query_hint(prev) && !matches!(prev.as_str(), "ist" | "sind" | "is" | "are"))
    }) || tokens.get(index + 1).is_some_and(|next| power(next))
        || index.checked_sub(1).and_then(|i| tokens.get(i)).is_some_and(|prev| power(prev))
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
    // Weather (and similar read-only domains) must not become "turn it off" targets (P-14).
    if id.starts_with("weather.") {
        return false;
    }
    home.entities.iter().any(|entity| {
        entity.entity_id == id
            && assist_visible(entity, home)
            && !is_infra(entity)
            && entity.domain != "weather"
            && domain.is_none_or(|wanted| entity.domain == wanted || id.starts_with(&format!("{wanted}.")))
    })
}
