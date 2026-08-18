use crate::home::expose::assist_visible;
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::clause::{finish_intents, Clause};
use crate::parse::compound::{area_slots, query_keeps_entity, wants_light_clarify};
use crate::parse::infer::looks_like_named_device;
use crate::parse::resolve::{query_grounded, unique_in_area};
use crate::parse::slots::{fill_intent, intent_from_action, pick_singular_lamp, ClauseOut};
use crate::parse::split::wants_group_clarify;

pub(crate) fn area_command(ctx: &Clause) -> Option<ClauseOut> {
    if ctx.resolved.areas.is_empty() || looks_like_named_device(ctx.tokens) || !ctx.resolved.entities.is_empty() {
        return None;
    }
    if !matches!(
        ctx.action,
        Action::On
            | Action::Off
            | Action::Toggle
            | Action::SetLight
            | Action::SetTemp
            | Action::CoverOpen
            | Action::CoverClose
            | Action::CoverSet
            | Action::FanSpeed
            | Action::Lock
            | Action::Unlock
            | Action::GetState
    ) {
        return None;
    }
    if let Some(lamp) = pick_singular_lamp(ctx.tokens, ctx.home, &ctx.resolved.areas) {
        let force = !matches!(ctx.action, Action::On | Action::Off | Action::Toggle | Action::GetState)
            || catalog().any(ctx.raw, &catalog().command_hedges);
        if force {
            let mut intents = vec![fill_intent(
                ctx.action,
                ctx.tokens,
                ctx.number,
                Some(&lamp),
                ctx.resolved.areas.first().map(String::as_str),
                Some("light"),
            )];
            intents.retain(|i| i.name != "Unknown");
            return Some(ClauseOut::Intents(intents));
        }
    }
    if ctx.resolved.areas.len() == 1
        && matches!(ctx.action, Action::On | Action::Off | Action::Toggle)
        && ctx.number.is_none()
        && (wants_light_clarify(ctx.tokens, ctx.home, &ctx.resolved.areas) || wants_group_clarify(ctx.raw))
    {
        let lights: Vec<String> = ctx
            .home
            .entities
            .iter()
            .filter(|e| assist_visible(e, ctx.home))
            .filter(|e| {
                e.domain == "light"
                    && !crate::home::policy::is_infra_light(e)
                    && e.area.as_ref().is_some_and(|a| ctx.resolved.areas.contains(a))
            })
            .map(|e| e.entity_id.clone())
            .collect();
        if lights.len() > 1 {
            let mut template = intent_from_action(ctx.action, ctx.tokens);
            if let Some(area) = ctx.resolved.areas.first() {
                template = template.with("area", area).with("domain", "light");
            }
            return Some(ClauseOut::Clarify(lights, template));
        }
    }
    let mut intents = Vec::new();
    for area in &ctx.resolved.areas {
        let (id, area_slot, dom) = area_slots(ctx.action, area, ctx.domain, ctx.home, ctx.tokens);
        intents.push(fill_intent(ctx.action, ctx.tokens, ctx.number, id.as_deref(), area_slot.as_deref(), dom.as_deref()));
    }
    Some(finish_intents(intents, ctx))
}

pub(crate) fn query_area(ctx: &Clause) -> Option<ClauseOut> {
    if !matches!(ctx.action, Action::GetState)
        || ctx.resolved.areas.is_empty()
        || query_keeps_entity(ctx.tokens, ctx.home, &ctx.resolved, ctx.light_areas)
    {
        return None;
    }
    let intents =
        ctx.resolved.areas.iter().map(|area| fill_intent(ctx.action, ctx.tokens, ctx.number, None, Some(area), ctx.domain)).collect();
    Some(finish_intents(intents, ctx))
}

pub(crate) fn query_ungrounded(ctx: &Clause) -> Option<ClauseOut> {
    if matches!(ctx.action, Action::GetState)
        && ctx.resolved.entities.is_empty()
        && ctx.resolved.ambiguous.is_empty()
        && !query_grounded(ctx.tokens, ctx.home, false)
    {
        Some(ClauseOut::Intents(Vec::new()))
    } else {
        None
    }
}

pub(crate) fn multi_area(ctx: &Clause) -> Option<ClauseOut> {
    (ctx.resolved.areas.len() > 1).then(|| {
        let intents = ctx
            .resolved
            .areas
            .iter()
            .map(|area| {
                let entity_id = ctx.domain.and_then(|domain| unique_in_area(ctx.home, area, domain, ctx.tokens));
                fill_intent(ctx.action, ctx.tokens, ctx.number, entity_id.as_deref(), Some(area), ctx.domain)
            })
            .collect();
        finish_intents(intents, ctx)
    })
}

pub(crate) fn grounded_entities(ctx: &Clause) -> Option<ClauseOut> {
    (!ctx.resolved.entities.is_empty()).then(|| {
        let intents = ctx
            .resolved
            .entities
            .iter()
            .map(|ent| fill_intent(ctx.action, ctx.tokens, ctx.number, Some(&ent.entity_id), ent.area.as_deref(), Some(&ent.domain)))
            .collect();
        finish_intents(intents, ctx)
    })
}

pub(crate) fn grounded_ambiguous(ctx: &Clause) -> Option<ClauseOut> {
    (!ctx.resolved.ambiguous.is_empty()).then(|| {
        let names = ctx.resolved.ambiguous.iter().map(|e| e.entity_id.clone()).collect();
        ClauseOut::Clarify(names, intent_from_action(ctx.action, ctx.tokens))
    })
}

pub(crate) fn grounded_areas(ctx: &Clause) -> Option<ClauseOut> {
    (!ctx.resolved.areas.is_empty()).then(|| {
        let intents =
            ctx.resolved.areas.iter().map(|area| fill_intent(ctx.action, ctx.tokens, ctx.number, None, Some(area), ctx.domain)).collect();
        finish_intents(intents, ctx)
    })
}
