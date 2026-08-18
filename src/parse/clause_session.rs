use crate::parse::clause::{finish_intents, Clause};
use crate::parse::clause_support::last_matching;
use crate::parse::resolve::unique_in_area;
use crate::parse::slots::{fill_intent, ClauseOut};

pub(crate) fn session_climate_cover(ctx: &Clause) -> Option<ClauseOut> {
    let areas: Vec<&str> = ctx.session.last_areas().collect();
    if areas.len() <= 1 || !matches!(ctx.domain, Some("climate") | Some("cover")) {
        return None;
    }
    let intents = areas
        .into_iter()
        .map(|area| {
            let id = ctx.domain.and_then(|d| unique_in_area(ctx.home, area, d, ctx.tokens));
            fill_intent(ctx.action, ctx.tokens, ctx.number, id.as_deref(), Some(area), ctx.domain)
        })
        .collect();
    Some(finish_intents(intents, ctx))
}

pub(crate) fn session_entities(ctx: &Clause) -> Option<ClauseOut> {
    let prev = last_matching(ctx.session, ctx.home, ctx.domain);
    (!prev.is_empty()).then(|| {
        let intents = prev.into_iter().map(|id| fill_intent(ctx.action, ctx.tokens, ctx.number, Some(id), None, ctx.domain)).collect();
        finish_intents(intents, ctx)
    })
}

pub(crate) fn session_areas(ctx: &Clause) -> Option<ClauseOut> {
    let areas: Vec<&str> = ctx.session.last_areas().collect();
    (!areas.is_empty()).then(|| {
        let intents = areas
            .into_iter()
            .map(|area| {
                let id = ctx
                    .domain
                    .filter(|d| matches!(*d, "climate" | "media_player" | "fan"))
                    .and_then(|d| unique_in_area(ctx.home, area, d, ctx.tokens));
                fill_intent(ctx.action, ctx.tokens, ctx.number, id.as_deref(), Some(area), ctx.domain)
            })
            .collect();
        finish_intents(intents, ctx)
    })
}
