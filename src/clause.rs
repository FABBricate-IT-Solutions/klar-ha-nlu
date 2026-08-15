use crate::compound::{apply_compound_light, area_slots, named_scene_or_script, query_keeps_entity, wants_light_clarify};
use crate::expose::assist_visible;
use crate::home_policy::{fallback_climate, fallback_cover_area, is_infra};
use crate::lang::catalog;
use crate::lexicon::{detect_actions, domain_for, Action};
use crate::numbers::first_number;
use crate::parse_help::{infer_action, looks_like_named_device, looks_like_question, prefer_action};
use crate::parse_slots::{
    all_lights_clause, fill_intent, intent_from_action, laundry_switch_clause, pick_singular_lamp, timer_clause, ClauseOut,
};
use crate::resolve::{climates_of_kind, light_rooms_for_clarify, query_grounded, resolve, unique_in_area};
use crate::session::Session;
use crate::split::{follow_fixture, wants_group_clarify};
use crate::types::{HomeGraph, Intent, Mode, Settings};

struct Clause<'a> {
    tokens: &'a [String],
    raw: &'a [String],
    home: &'a HomeGraph,
    session: &'a Session,
    action: Action,
    number: Option<i32>,
    domain: Option<&'a str>,
    question: bool,
    resolved: crate::resolve::Resolved,
    light_areas: &'a [String],
}

pub(crate) fn parse_clause(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    settings: &Settings,
    light_areas: &[String],
) -> ClauseOut {
    let actions = detect_actions(tokens);
    let question = looks_like_question(tokens);
    let number = first_number(tokens);
    let command = prefer_action(&actions);
    let hard = is_hard_command(command, tokens);
    let guessed = if question && number.is_none() && !hard {
        Action::GetState
    } else {
        command.or_else(|| actions.first().map(|(_, a)| *a)).unwrap_or_else(|| guess_action(tokens, session, number))
    };
    let early = infer_action(guessed, tokens, number, question, session, None);
    let domain = domain_for(early, tokens);

    if let Some(out) =
        laundry_switch_clause(tokens, home, early, number, domain).or_else(|| timer_clause(tokens, home, early, number, domain))
    {
        return out;
    }

    let mut resolved = resolve_targets(tokens, home, session, settings, domain, early);
    apply_compound_light(home, tokens, light_areas, &mut resolved);
    let target = resolved.entities.first().map(|e| e.domain.as_str());
    let action = match target {
        Some(domain) => infer_action(guessed, tokens, number, question, session, Some(domain)),
        None => early,
    };
    let domain = domain_for(action, tokens);
    let ctx = Clause { tokens, raw, home, session, action, number, domain, question, resolved, light_areas };
    // First match wins. Later policies must not steal a more specific bind
    // (named scene/device, compound light, LightAim room group).
    for policy in [
        named_scene,
        all_lights,
        follow_named,
        area_command,
        query_area,
        query_ungrounded,
        multi_area,
        grounded_entities,
        grounded_ambiguous,
        grounded_areas,
        session_climate_cover,
        session_entities,
        session_areas,
        light_rooms_clarify,
        fallback_temp,
        fallback_cover,
        leftover_command,
    ] {
        if let Some(out) = policy(&ctx) {
            return out;
        }
    }
    ClauseOut::Intents(Vec::new())
}

fn is_hard_command(command: Option<Action>, tokens: &[String]) -> bool {
    matches!(
        command,
        Some(
            Action::SetLight
                | Action::SetTemp
                | Action::CoverSet
                | Action::FanSpeed
                | Action::Scene
                | Action::MediaPause
                | Action::MediaPlay
                | Action::MediaNext
                | Action::MediaMute
                | Action::VacuumStart
                | Action::VacuumDock
                | Action::TimerStart
                | Action::TimerAdd
                | Action::TimerCancel
                | Action::TimerPause
                | Action::ListAdd
                | Action::ListComplete
        )
    ) || (matches!(command, Some(Action::On))
        && tokens.iter().any(|t| catalog().scene_nouns.contains(t.as_str()) || catalog().script_words.contains(t.as_str())))
}

fn guess_action(tokens: &[String], session: &Session, number: Option<i32>) -> Action {
    if number.is_some() {
        crate::numbers::guess_numbered_action(
            tokens,
            session.last_entities().any(|e| e.starts_with("climate."))
                || session.last_names().any(|n| n.contains("Climate"))
                || session.last_domains().any(|d| d == "climate"),
            session.last_entities().any(|e| e.starts_with("cover.")) || session.last_domains().any(|d| d == "cover"),
            session.last_entities().any(|e| e.starts_with("fan.")) || session.last_domains().any(|d| d == "fan"),
        )
    } else {
        Action::GetState
    }
}

fn resolve_targets(
    tokens: &[String],
    home: &HomeGraph,
    _session: &Session,
    settings: &Settings,
    domain: Option<&str>,
    action: Action,
) -> crate::resolve::Resolved {
    if settings.mode == Mode::ContextOnly {
        return crate::resolve::Resolved { areas: resolve(tokens, home, None).areas, entities: Vec::new(), ambiguous: Vec::new() };
    }
    let first = resolve(tokens, home, domain);
    if domain.is_none() && matches!(action, Action::On | Action::Off | Action::Toggle) {
        let named_other = first.entities.iter().any(|entity| entity.domain != "light");
        let skip_lights = named_other
            || catalog().any(tokens, &catalog().skip_light)
            || (catalog().any(tokens, &catalog().laundry_area) && !catalog().any(tokens, &catalog().light_nouns));
        if !skip_lights {
            let lights = resolve(tokens, home, Some("light"));
            if lights.ambiguous.is_empty() && (!lights.entities.is_empty() || !lights.areas.is_empty()) {
                return lights;
            }
        }
    }
    first
}

fn named_scene(ctx: &Clause) -> Option<ClauseOut> {
    if ctx.question || !matches!(ctx.action, Action::On | Action::Scene | Action::GetState) {
        return None;
    }
    let id = named_scene_or_script(ctx.tokens, ctx.home)?;
    Some(ClauseOut::Intents(vec![Intent::new("HassTurnOn")
        .with("entity_id", &id)
        .with("domain", if id.starts_with("script.") { "script" } else { "scene" })]))
}

fn all_lights(ctx: &Clause) -> Option<ClauseOut> {
    all_lights_clause(ctx.tokens, ctx.home, ctx.action, ctx.number, &ctx.resolved.areas)
}

fn follow_named(ctx: &Clause) -> Option<ClauseOut> {
    if !looks_like_named_device(ctx.tokens) || !ctx.resolved.areas.is_empty() {
        return None;
    }
    let areas: Vec<String> = ctx.session.last_areas().map(str::to_string).collect();
    let id = follow_fixture(ctx.tokens, ctx.home, &areas)?;
    let act = if ctx.session.last_names().any(|n| n == "HassTurnOff") { Action::Off } else { Action::On };
    let mut intents = vec![fill_intent(act, ctx.tokens, ctx.number, Some(&id), areas.first().map(String::as_str), Some("light"))];
    intents.retain(|i| i.name != "Unknown");
    Some(ClauseOut::Intents(intents))
}

fn area_command(ctx: &Clause) -> Option<ClauseOut> {
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
                    && !crate::home_policy::is_infra_light(e)
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

fn query_area(ctx: &Clause) -> Option<ClauseOut> {
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

fn query_ungrounded(ctx: &Clause) -> Option<ClauseOut> {
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

fn multi_area(ctx: &Clause) -> Option<ClauseOut> {
    (ctx.resolved.areas.len() > 1).then(|| {
        let intents =
            ctx.resolved.areas.iter().map(|area| fill_intent(ctx.action, ctx.tokens, ctx.number, None, Some(area), ctx.domain)).collect();
        finish_intents(intents, ctx)
    })
}

fn grounded_entities(ctx: &Clause) -> Option<ClauseOut> {
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

fn grounded_ambiguous(ctx: &Clause) -> Option<ClauseOut> {
    (!ctx.resolved.ambiguous.is_empty()).then(|| {
        let names = ctx.resolved.ambiguous.iter().map(|e| e.entity_id.clone()).collect();
        ClauseOut::Clarify(names, intent_from_action(ctx.action, ctx.tokens))
    })
}

fn grounded_areas(ctx: &Clause) -> Option<ClauseOut> {
    (!ctx.resolved.areas.is_empty()).then(|| {
        let intents =
            ctx.resolved.areas.iter().map(|area| fill_intent(ctx.action, ctx.tokens, ctx.number, None, Some(area), ctx.domain)).collect();
        finish_intents(intents, ctx)
    })
}

fn session_climate_cover(ctx: &Clause) -> Option<ClauseOut> {
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

fn session_entities(ctx: &Clause) -> Option<ClauseOut> {
    let prev = last_matching(ctx.session, ctx.home, ctx.domain);
    (!prev.is_empty()).then(|| {
        let intents = prev.into_iter().map(|id| fill_intent(ctx.action, ctx.tokens, ctx.number, Some(id), None, ctx.domain)).collect();
        finish_intents(intents, ctx)
    })
}

fn session_areas(ctx: &Clause) -> Option<ClauseOut> {
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

fn light_rooms_clarify(ctx: &Clause) -> Option<ClauseOut> {
    if !matches!(ctx.action, Action::On | Action::Off | Action::Toggle) {
        return None;
    }
    if ctx.domain != Some("light") && !crate::lexicon::has_light_noun(ctx.tokens) {
        return None;
    }
    if ctx.session.last_entities().next().is_some() || ctx.session.last_areas().next().is_some() {
        return None;
    }
    let rooms = light_rooms_for_clarify(ctx.home);
    if rooms.len() > 1 {
        return Some(ClauseOut::Clarify(rooms, intent_from_action(ctx.action, ctx.tokens).with("domain", "light")));
    }
    Some(finish_intents(vec![fill_intent(ctx.action, ctx.tokens, ctx.number, None, None, ctx.domain)], ctx))
}

fn fallback_temp(ctx: &Clause) -> Option<ClauseOut> {
    matches!(ctx.action, Action::SetTemp).then(|| {
        let hits = climates_of_kind(ctx.home, ctx.tokens);
        let intent = if hits.len() == 1 {
            fill_intent(ctx.action, ctx.tokens, ctx.number, Some(&hits[0]), None, Some("climate"))
        } else if let Some(id) = fallback_climate(ctx.home) {
            fill_intent(ctx.action, ctx.tokens, ctx.number, Some(id), None, Some("climate"))
        } else {
            fill_intent(ctx.action, ctx.tokens, ctx.number, None, None, Some("climate"))
        };
        finish_intents(vec![intent], ctx)
    })
}

fn fallback_cover(ctx: &Clause) -> Option<ClauseOut> {
    (matches!(ctx.action, Action::CoverClose | Action::CoverOpen) && catalog().any(ctx.tokens, &catalog().curtain_nouns)).then(|| {
        let area = fallback_cover_area(ctx.home);
        finish_intents(vec![fill_intent(ctx.action, ctx.tokens, ctx.number, None, area.as_deref(), Some("cover"))], ctx)
    })
}

fn leftover_command(ctx: &Clause) -> Option<ClauseOut> {
    (!matches!(ctx.action, Action::On | Action::Off | Action::Toggle))
        .then(|| finish_intents(vec![fill_intent(ctx.action, ctx.tokens, ctx.number, None, None, ctx.domain)], ctx))
}

fn finish_intents(mut intents: Vec<Intent>, ctx: &Clause) -> ClauseOut {
    if matches!(ctx.action, Action::On | Action::Off | Action::Toggle | Action::GetState) {
        let role_domain = ctx.domain.or_else(|| crate::lexicon::has_light_noun(ctx.tokens).then_some("light"));
        if let Some(role_domain) = role_domain {
            for area in &ctx.resolved.areas {
                for entity in crate::roles::role_siblings(ctx.home, area, role_domain) {
                    if !intents.iter().any(|i| i.slot("entity_id") == Some(entity.entity_id.as_str())) {
                        intents.push(fill_intent(
                            ctx.action,
                            ctx.tokens,
                            ctx.number,
                            Some(&entity.entity_id),
                            entity.area.as_deref(),
                            Some(&entity.domain),
                        ));
                    }
                }
            }
        }
    }
    intents.retain(|i| i.name != "Unknown");
    ClauseOut::Intents(intents)
}

pub(crate) fn last_visible<'a>(session: &'a Session, home: &'a HomeGraph) -> Option<&'a str> {
    last_matching(session, home, None).into_iter().next()
}

fn last_matching<'a>(session: &'a Session, home: &'a HomeGraph, domain: Option<&str>) -> Vec<&'a str> {
    session
        .last_entities()
        .filter(|id| home.entities.iter().any(|e| e.entity_id == *id && assist_visible(e, home) && !is_infra(e)))
        .filter(|id| domain.is_none_or(|d| id.starts_with(&format!("{d}."))))
        .collect()
}
