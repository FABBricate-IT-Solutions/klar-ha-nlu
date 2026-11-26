use crate::home::policy::{fallback_climate, fallback_cover_area};
use crate::home::roles::{is_music_assistant_player, is_music_player};
use crate::lang::catalog;
use crate::parse::action::{detect_actions, domain_for_linked, is_hard_command, Action};
use crate::parse::also::guess_action;
use crate::parse::clause_area::{
    area_command, area_slots, grounded_ambiguous, grounded_areas, grounded_entities, multi_area, query_area, query_ungrounded,
};
use crate::parse::clause_session::{session_areas, session_climate_cover, session_entities, which_lights_query};
use crate::parse::clause_support::{named_instead_of_light, named_scene_policy, preferred_area_domain, resolve_targets};
use crate::parse::compound::apply_compound_light;
use crate::parse::infer::{except_tail, infer_action_in, looks_like_named_device, looks_like_question, prefer_action, wants_all_lights};
use crate::parse::media::{media_clause, now_playing_status};
use crate::parse::numbers::first_number;
use crate::parse::policy::{media_claimed_empty, overlaid_candidate, retain_after_media_claim, MatchOverlay};
pub(crate) use crate::parse::policy::{ClauseCandidate, PolicyId};
use crate::parse::resolve::{
    climates_of_kind, entity_has_name_evidence, entity_name_is_mentioned, has_fuzzy_target_token, known_target_token,
    light_rooms_for_clarify,
};
use crate::parse::resolve::{free_tokens, link_named_spans};
use crate::parse::slots::{all_lights_clause, fill_intent, fill_list_intents, intent_from_action, ClauseOut};
use crate::parse::split::follow_fixture;
use crate::session::Session;
use crate::types::{HomeGraph, Intent, Settings, UnitSystem};

pub(crate) struct Clause<'a> {
    pub tokens: &'a [String],
    pub raw: &'a [String],
    pub home: &'a HomeGraph,
    pub session: &'a Session,
    pub action: Action,
    pub number: Option<i32>,
    pub domain: Option<&'a str>,
    pub question: bool,
    pub resolved: crate::parse::resolve::Resolved,
    pub light_areas: &'a [String],
}

type PolicyFn = for<'policy> fn(&Clause<'policy>) -> Option<ClauseOut>;

pub(crate) fn parse_clause_candidates_for_action(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    settings: &Settings,
    light_areas: &[String],
    forced_action: Option<Action>,
    match_controls: &[crate::types::MatchControl],
) -> Vec<ClauseCandidate> {
    let cat = &catalog();
    let free_text_payload =
        cat.any(tokens, cat.list_nouns()) || cat.any(tokens, cat.media_nouns()) || crate::parse::calendar::mentions_calendar(tokens);
    let in_number = crate::parse::numbers::number_token_indices(tokens);
    if !free_text_payload
        && tokens
            .iter()
            .enumerate()
            .any(|(index, token)| !in_number.contains(&index) && cat.is_protected_typo(token) && !known_target_token(token, home))
    {
        return Vec::new();
    }
    // Names first: numbers and colours inside a device label are not values.
    let linked = link_named_spans(tokens, home, &[]);
    let free = free_tokens(tokens, &linked);
    let value_tokens = mask_name_values(tokens, &linked);
    let actions = detect_actions(&value_tokens);
    let fuzzy_action = !tokens.iter().any(|token| cat.verb(token).is_some()) && !actions.is_empty();
    if !free_text_payload && fuzzy_action && has_fuzzy_target_token(tokens, home) {
        return Vec::new();
    }
    let question = looks_like_question(tokens);
    let number = first_number(&free).or_else(|| crate::parse::infer::cover_halfway_position(&free));
    let command = prefer_action(&actions);
    let hard = is_hard_command(command, &value_tokens);
    let guessed = forced_action.unwrap_or_else(|| {
        if which_lights_query(tokens) {
            // "which lights are on" is a list status ask, not TurnOn (P-13).
            Action::GetState
        } else if crate::parse::compound::named_scene_or_script(tokens, home).is_some()
            && !(cat.any(tokens, cat.question_words()) || tokens.first().is_some_and(|token| cat.is_question_start(token)))
        {
            Action::Scene
        } else if !question && crate::parse::infer::cover_halfway(&free) && crate::parse::action::has_cover_noun(tokens) {
            Action::CoverSet
        } else if !question && (crate::parse::infer::color_word(&free).is_some() || crate::parse::infer::light_level(&free).is_some()) {
            Action::SetLight
        } else if question && number.is_none() && !hard {
            Action::GetState
        } else {
            command.or_else(|| actions.first().map(|(_, a)| *a)).unwrap_or_else(|| guess_action(&value_tokens, session, number))
        }
    });
    let early = infer_action_in(guessed, tokens, number, question, session, None, Some(home));
    let named = named_instead_of_light(early, tokens, number, home, settings);
    let early = named.map_or(early, |domain| infer_action_in(guessed, tokens, number, question, session, Some(domain), Some(home)));
    let domain = domain_for_linked(early, tokens, &linked);

    let overlay = MatchOverlay::new(match_controls);
    let mut candidates = crate::parse::clause_early::early_special_clauses(tokens, raw, home, early, number, domain, overlay);

    let mut resolved = resolve_targets(tokens, home, settings, domain, early);
    apply_compound_light(home, tokens, light_areas, &mut resolved);
    crate::parse::resolve::narrow_shared_rooms(&mut resolved, tokens, home, session.preferred_area.as_deref());
    narrow_to_satellite(&mut resolved, tokens, session.preferred_area.as_deref(), domain, early);
    // "send robby home": "home" also matches weather.home — keep the vacuum (P-30).
    if crate::parse::infer::dock_hint(tokens) {
        let vacua: Vec<_> = resolved.entities.iter().filter(|entity| entity.domain == "vacuum").cloned().collect();
        if !vacua.is_empty() {
            resolved.entities = vacua;
            resolved.ambiguous.retain(|entity| entity.domain == "vacuum");
        }
    }
    let target = resolved.entities.first().map(|e| e.domain.as_str());
    let action = match target {
        Some(domain) => infer_action_in(guessed, tokens, number, question, session, Some(domain), Some(home)),
        None => early,
    };
    // "send the vacuum home" with a vacuum named Robby: kind word alone, one vacuum in the home.
    if matches!(action, Action::VacuumDock | Action::VacuumStart) && !resolved.entities.iter().any(|entity| entity.domain == "vacuum") {
        let vacs: Vec<_> = home
            .entities
            .iter()
            .filter(|entity| {
                entity.domain == "vacuum" && crate::home::expose::assist_visible(entity, home) && !crate::home::policy::is_infra(entity)
            })
            .cloned()
            .collect();
        if vacs.len() == 1 {
            resolved.entities = vacs;
        }
    }
    let domain = domain_for_linked(action, tokens, &linked);
    let ctx = Clause { tokens, raw, home, session, action, number, domain, question, resolved, light_areas };
    let except_all = except_tail(tokens).is_some() && wants_all_lights(tokens);
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
        if !overlay.enabled(*policy) {
            continue;
        }
        if !ctx.resolved.ask_areas.is_empty() && *policy != PolicyId::AreaCommand {
            continue;
        }
        if except_all
            && matches!(
                *policy,
                PolicyId::FollowNamed
                    | PolicyId::PreferredAreaCommand
                    | PolicyId::AreaCommand
                    | PolicyId::GroundedEntities
                    | PolicyId::GroundedAmbiguous
                    | PolicyId::GroundedAreas
            )
            || (*policy == PolicyId::GroundedAmbiguous && now_playing_status(tokens))
        {
            continue;
        }
        if let Some(outcome) = evaluate(&ctx) {
            candidates.push(overlaid_candidate(*policy, action, outcome, &overlay));
        }
    }
    if candidates.iter().any(|candidate| {
        candidate.policy == PolicyId::NamedScene && matches!(&candidate.outcome, ClauseOut::Intents(intents) if !intents.is_empty())
    }) {
        candidates.retain(|candidate| candidate.policy == PolicyId::NamedScene);
    }
    rewrite_climate_temps(&mut candidates, tokens, settings.unit_system);
    if media_claimed_empty(&candidates) {
        let transfer = ctx.tokens.iter().any(|token| matches!(token.as_str(), "verschiebe" | "move" | "transfer"));
        let named = !transfer
            && ctx.resolved.entities.iter().any(|entity| {
                entity_has_name_evidence(ctx.tokens, entity, ctx.home) && (is_music_assistant_player(entity) || is_music_player(entity))
            });
        retain_after_media_claim(&mut candidates, named);
    } else if candidates.iter().any(|candidate| {
        candidate.policy == PolicyId::Media && matches!(&candidate.outcome, ClauseOut::Intents(intents) if !intents.is_empty())
    }) {
        candidates.retain(|candidate| !matches!(candidate.policy, PolicyId::GroundedAmbiguous | PolicyId::GroundedEntities));
    }
    candidates
}

fn rewrite_climate_temps(candidates: &mut [ClauseCandidate], tokens: &[String], unit: UnitSystem) {
    for candidate in candidates {
        if let ClauseOut::Intents(intents) = &mut candidate.outcome {
            for intent in intents {
                crate::units::bind_set_temp(intent, tokens, unit);
            }
        }
    }
}

fn media(ctx: &Clause) -> Option<ClauseOut> {
    media_clause(ctx.tokens, ctx.raw, ctx.home, ctx.session, ctx.action, ctx.number, &ctx.resolved)
}

fn list(ctx: &Clause) -> Option<ClauseOut> {
    if !matches!(ctx.action, Action::ListAdd | Action::ListComplete | Action::ListRemove) {
        return None;
    }
    // Calendar nouns win over list when the utterance targets a calendar (not a todo list).
    if crate::parse::calendar::mentions_calendar(ctx.tokens)
        && !catalog().any(ctx.tokens, catalog().list_nouns())
        && !catalog().any(ctx.tokens, catalog().chores())
    {
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
    let intents = fill_list_intents(ctx.action, ctx.tokens, target);
    (!intents.is_empty()).then_some(ClauseOut::Intents(intents))
}

fn named_scene(ctx: &Clause) -> Option<ClauseOut> {
    named_scene_policy(ctx.tokens, ctx.home, ctx.question, ctx.action)
}

fn all_lights(ctx: &Clause) -> Option<ClauseOut> {
    if matches!(ctx.action, Action::CoverOpen | Action::CoverClose | Action::CoverStop | Action::CoverSet) {
        return crate::parse::all_lights::all_covers_clause(ctx.tokens, ctx.home, ctx.action, ctx.number, &ctx.resolved.areas);
    }
    if which_lights_query(ctx.tokens) {
        // After multi-turn light commands, SessionAreas answers per prior room.
        if crate::parse::clause_session::session_has_light_areas(ctx.session, ctx.home) {
            return None;
        }
        return lights_status_list(ctx);
    }
    if !ctx.resolved.floors.is_empty() && ctx.resolved.areas.is_empty() {
        return None;
    }
    all_lights_clause(ctx.tokens, ctx.home, ctx.action, ctx.number, &ctx.resolved.areas)
}

/// "which lights are on" / "are any lights on": list status, never a TurnOn clarify (P-13).
fn lights_status_list(ctx: &Clause) -> Option<ClauseOut> {
    let cat = &catalog();
    let mut intent = Intent::new("HassGetState").with("domain", "light").with("response", "list");
    if cat.any(ctx.tokens, cat.on_words()) {
        intent = intent.with("state", "on");
    } else if cat.any(ctx.tokens, cat.off_words()) {
        intent = intent.with("state", "off");
    }
    if let Some(area) = ctx.resolved.areas.first() {
        intent = intent.with("area", area);
    }
    Some(ClauseOut::Intents(vec![intent]))
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
            | Action::CoverStop
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

/// Keep domain-noun commands ("the fan", "the blinds") in the satellite room (P-22).
fn narrow_to_satellite(
    resolved: &mut crate::parse::resolve::Resolved,
    tokens: &[String],
    preferred: Option<&str>,
    domain: Option<&str>,
    action: Action,
) {
    let Some(area) = preferred else {
        return;
    };
    if !resolved.areas.is_empty() || tokens.iter().any(|token| catalog().is_all(token)) {
        return;
    }
    let domain = preferred_area_domain(domain, action, tokens).or(domain);
    let Some(domain) = domain else {
        return;
    };
    if !domain_noun_only(tokens, domain) {
        return;
    }
    let local: Vec<_> = resolved
        .entities
        .iter()
        .chain(resolved.ambiguous.iter())
        .filter(|entity| entity.area.as_deref() == Some(area) && entity.domain == domain)
        .cloned()
        .collect();
    if !local.is_empty() {
        resolved.entities = local;
        resolved.ambiguous.clear();
        return;
    }
    // No device in the satellite room: clear foreign hits so preferred_area_command can bind.
    if resolved.entities.iter().chain(resolved.ambiguous.iter()).any(|entity| entity.domain == domain) {
        resolved.entities.clear();
        resolved.ambiguous.clear();
    }
}

fn domain_noun_only(tokens: &[String], domain: &str) -> bool {
    let cat = &catalog();
    match domain {
        "light" => crate::parse::action::has_light_noun(tokens),
        "fan" => cat.any(tokens, cat.fan_nouns()),
        "cover" => crate::parse::action::has_cover_noun(tokens),
        "climate" => crate::parse::action::has_climate_noun(tokens),
        "media_player" => cat.any(tokens, cat.media_nouns()) || crate::home::roles::tv_asked(tokens),
        _ => false,
    }
}

fn light_rooms_clarify(ctx: &Clause) -> Option<ClauseOut> {
    if ctx.question || !ctx.resolved.areas.is_empty() {
        return None;
    }
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
    (matches!(ctx.action, Action::CoverClose | Action::CoverOpen | Action::CoverStop)
        && catalog().any(ctx.tokens, catalog().curtain_nouns()))
    .then(|| {
        let area = fallback_cover_area(ctx.home);
        finish_intents(vec![fill_intent(ctx.action, ctx.tokens, ctx.number, None, area.as_deref(), Some("cover"))], ctx)
    })
}

fn leftover_command(ctx: &Clause) -> Option<ClauseOut> {
    (!matches!(ctx.action, Action::On | Action::Off | Action::Toggle | Action::GetState))
        .then(|| finish_intents(vec![fill_intent(ctx.action, ctx.tokens, ctx.number, None, None, ctx.domain)], ctx))
}

pub(crate) fn finish_intents(mut intents: Vec<Intent>, ctx: &Clause) -> ClauseOut {
    if matches!(ctx.action, Action::On | Action::Off | Action::Toggle | Action::GetState) {
        let role_domain = ctx.domain.or_else(|| crate::parse::action::has_light_noun(ctx.tokens).then_some("light"));
        if let Some(role_domain) = role_domain {
            for area in &ctx.resolved.areas {
                for entity in crate::home::roles::role_siblings(ctx.home, area, role_domain, &catalog()) {
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

/// Replace numbers and colour words that sit inside a linked device name so action
/// detection does not treat them as brightness or a colour set.
fn mask_name_values(tokens: &[String], linked: &crate::parse::resolve::LinkedNames) -> Vec<String> {
    let cat = &catalog();
    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            if !linked.consumed.contains(&index) {
                return token.clone();
            }
            let is_value = token.parse::<i32>().is_ok()
                || cat.number(token).is_some()
                || cat.color(token).is_some()
                || cat.timer_nouns().contains(token.as_str());
            if is_value {
                String::new()
            } else {
                token.clone()
            }
        })
        .collect()
}
