use crate::home::expose::assist_visible;
use crate::home::policy::{fallback_climate, fallback_cover_area};
use crate::lang::catalog;
use crate::parse::action::{detect_actions, domain_for, guess_action, is_hard_command, Action};
use crate::parse::clause_support::{last_matching, named_scene_policy, preferred_area_domain, resolve_targets};
use crate::parse::compound::{apply_compound_light, area_slots, query_keeps_entity, wants_light_clarify};
use crate::parse::infer::{infer_action, looks_like_named_device, looks_like_question, prefer_action};
use crate::parse::media::media_clause;
use crate::parse::numbers::first_number;
use crate::parse::policy::{candidate, media_claimed_empty, media_fallback_allowed};
pub(crate) use crate::parse::policy::{ClauseCandidate, PolicyId};
use crate::parse::resolve::{
    climates_of_kind, entity_has_name_evidence, entity_name_is_mentioned, has_fuzzy_target_token, known_target_token,
    light_rooms_for_clarify, query_grounded, unique_in_area,
};
use crate::parse::slots::{
    all_lights_clause, fill_intent, fill_list_intent, intent_from_action, laundry_switch_clause, pick_singular_lamp, timer_clause,
    ClauseOut,
};
use crate::parse::split::{follow_fixture, wants_group_clarify};
use crate::session::Session;
use crate::types::{HomeGraph, Intent, Settings};

struct Clause<'a> {
    tokens: &'a [String],
    raw: &'a [String],
    home: &'a HomeGraph,
    session: &'a Session,
    action: Action,
    number: Option<i32>,
    domain: Option<&'a str>,
    question: bool,
    resolved: crate::parse::resolve::Resolved,
    light_areas: &'a [String],
}

type PolicyFn = for<'policy> fn(&Clause<'policy>) -> Option<ClauseOut>;

#[cfg(debug_assertions)]
pub(crate) fn parse_clause(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    settings: &Settings,
    light_areas: &[String],
) -> ClauseOut {
    parse_clause_candidates(tokens, raw, home, session, settings, light_areas)
        .into_iter()
        .next()
        .map(|candidate| candidate.outcome)
        .unwrap_or_else(|| ClauseOut::Intents(Vec::new()))
}

#[cfg(debug_assertions)]
pub(crate) fn parse_clause_candidates(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    settings: &Settings,
    light_areas: &[String],
) -> Vec<ClauseCandidate> {
    parse_clause_candidates_for_action(tokens, raw, home, session, settings, light_areas, None)
}

pub(crate) fn parse_clause_candidates_for_action(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    settings: &Settings,
    light_areas: &[String],
    forced_action: Option<Action>,
) -> Vec<ClauseCandidate> {
    let cat = catalog();
    let free_text_payload = cat.any(tokens, &cat.list_nouns) || cat.any(tokens, &cat.media_nouns);
    if !free_text_payload && tokens.iter().any(|token| cat.is_protected_typo(token) && !known_target_token(token, home)) {
        return Vec::new();
    }
    let actions = detect_actions(tokens);
    let fuzzy_action = !tokens.iter().any(|token| cat.verb(token).is_some()) && !actions.is_empty();
    if !free_text_payload && fuzzy_action && has_fuzzy_target_token(tokens, home) {
        return Vec::new();
    }
    let question = looks_like_question(tokens);
    let number = first_number(tokens);
    let command = prefer_action(&actions);
    let hard = is_hard_command(command, tokens);
    let guessed = forced_action.unwrap_or_else(|| {
        if question && number.is_none() && !hard {
            Action::GetState
        } else {
            command.or_else(|| actions.first().map(|(_, a)| *a)).unwrap_or_else(|| guess_action(tokens, session, number))
        }
    });
    let early = infer_action(guessed, tokens, number, question, session, None);
    let domain = domain_for(early, tokens);

    let mut candidates = Vec::new();
    if let Some(outcome) = laundry_switch_clause(tokens, home, early, number, domain) {
        candidates.push(candidate(PolicyId::LaundrySwitch, early, outcome));
    }
    if let Some(outcome) = timer_clause(tokens, home, early, number, domain) {
        candidates.push(candidate(PolicyId::Timer, early, outcome));
    }

    let mut resolved = resolve_targets(tokens, home, settings, domain, early);
    apply_compound_light(home, tokens, light_areas, &mut resolved);
    let target = resolved.entities.first().map(|e| e.domain.as_str());
    let action = match target {
        Some(domain) => infer_action(guessed, tokens, number, question, session, Some(domain)),
        None => early,
    };
    let domain = domain_for(action, tokens);
    let ctx = Clause { tokens, raw, home, session, action, number, domain, question, resolved, light_areas };
    let policies: &[(PolicyId, PolicyFn)] = &[
        (PolicyId::List, list),
        (PolicyId::Media, media),
        (PolicyId::NamedScene, named_scene),
        (PolicyId::AllLights, all_lights),
        (PolicyId::FollowNamed, follow_named),
        (PolicyId::PreferredAreaCommand, preferred_area_command),
        (PolicyId::AreaCommand, area_command),
        (PolicyId::FloorCommand, floor_command),
        (PolicyId::QueryArea, query_area),
        (PolicyId::QueryUngrounded, query_ungrounded),
        (PolicyId::MultiArea, multi_area),
        (PolicyId::GroundedEntities, grounded_entities),
        (PolicyId::GroundedAmbiguous, grounded_ambiguous),
        (PolicyId::GroundedAreas, grounded_areas),
        (PolicyId::SessionClimateCover, session_climate_cover),
        (PolicyId::SessionEntities, session_entities),
        (PolicyId::SessionAreas, session_areas),
        (PolicyId::LightRoomsClarify, light_rooms_clarify),
        (PolicyId::FallbackTemp, fallback_temp),
        (PolicyId::FallbackCover, fallback_cover),
        (PolicyId::LeftoverCommand, leftover_command),
    ];
    for (policy, evaluate) in policies {
        if let Some(outcome) = evaluate(&ctx) {
            candidates.push(candidate(*policy, action, outcome));
        }
    }
    if media_claimed_empty(&candidates) {
        let transfer = ctx.tokens.iter().any(|token| matches!(token.as_str(), "verschiebe" | "move" | "transfer"));
        let named = !transfer && ctx.resolved.entities.iter().any(|entity| entity_has_name_evidence(ctx.tokens, entity, ctx.home));
        candidates.retain(|candidate| match candidate.policy {
            PolicyId::GroundedEntities | PolicyId::GroundedAmbiguous => named,
            policy => media_fallback_allowed(policy),
        });
    }
    candidates
}

fn media(ctx: &Clause) -> Option<ClauseOut> {
    media_clause(ctx.tokens, ctx.raw, ctx.home, ctx.session, ctx.action, ctx.number, &ctx.resolved)
}

fn list(ctx: &Clause) -> Option<ClauseOut> {
    if !matches!(ctx.action, Action::ListAdd | Action::ListComplete) {
        return None;
    }
    if !ctx.resolved.ambiguous.is_empty() || ctx.resolved.entities.len() > 1 {
        let choices = ctx.resolved.ambiguous.iter().chain(ctx.resolved.entities.iter()).map(|entity| entity.entity_id.clone()).collect();
        return Some(ClauseOut::Clarify(choices, intent_from_action(ctx.action, ctx.tokens)));
    }
    let target = ctx.resolved.entities.first().filter(|entity| entity_has_name_evidence(ctx.tokens, entity, ctx.home));
    if target.is_none()
        && ctx
            .home
            .entities
            .iter()
            .filter(|entity| entity.domain == "todo")
            .any(|entity| entity_name_is_mentioned(ctx.tokens, entity, ctx.home))
    {
        return Some(ClauseOut::Intents(Vec::new()));
    }
    Some(ClauseOut::Intents(vec![fill_list_intent(ctx.action, ctx.tokens, target)]))
}

fn named_scene(ctx: &Clause) -> Option<ClauseOut> {
    named_scene_policy(ctx.tokens, ctx.home, ctx.question, ctx.action)
}

fn all_lights(ctx: &Clause) -> Option<ClauseOut> {
    if !ctx.resolved.floors.is_empty() && ctx.resolved.areas.is_empty() {
        return None;
    }
    all_lights_clause(ctx.tokens, ctx.home, ctx.action, ctx.number, &ctx.resolved.areas)
}

fn floor_command(ctx: &Clause) -> Option<ClauseOut> {
    if ctx.resolved.floors.is_empty() || !ctx.resolved.areas.is_empty() || !ctx.resolved.entities.is_empty() {
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
    let domain = ctx.domain.or_else(|| preferred_area_domain(ctx.domain, ctx.action, ctx.tokens));
    let intents = ctx
        .resolved
        .floors
        .iter()
        .map(|floor| fill_intent(ctx.action, ctx.tokens, ctx.number, None, None, domain).with("floor", floor))
        .collect();
    Some(finish_intents(intents, ctx))
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

fn preferred_area_command(ctx: &Clause) -> Option<ClauseOut> {
    let area = ctx.session.preferred_area.as_deref()?;
    if !ctx.resolved.areas.is_empty()
        || !ctx.resolved.floors.is_empty()
        || !ctx.resolved.entities.is_empty()
        || !ctx.resolved.ambiguous.is_empty()
        || ctx.tokens.iter().any(|token| catalog().is_all(token))
        || ctx.home.areas.iter().any(|rec| crate::home::policy::is_whole_home(rec) && rec.area_id == area)
    {
        return None;
    }
    let domain = preferred_area_domain(ctx.domain, ctx.action, ctx.tokens)?;
    let (id, area_slot, dom) = area_slots(ctx.action, area, Some(domain), ctx.home, ctx.tokens);
    let intent = fill_intent(ctx.action, ctx.tokens, ctx.number, id.as_deref(), area_slot.as_deref(), dom.as_deref());
    (intent.name != "Unknown").then(|| ClauseOut::Intents(vec![intent]))
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
    if ctx.domain != Some("light") && !crate::parse::action::has_light_noun(ctx.tokens) {
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
        let role_domain = ctx.domain.or_else(|| crate::parse::action::has_light_noun(ctx.tokens).then_some("light"));
        if let Some(role_domain) = role_domain {
            for area in &ctx.resolved.areas {
                for entity in crate::home::roles::role_siblings(ctx.home, area, role_domain) {
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
