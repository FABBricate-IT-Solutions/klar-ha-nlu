//! Exceptions: "all lights except the hall light", "the kitchen lights but not the
//! counter". The command runs without the devices the speaker took out of it. A device
//! that was excepted is never part of the plan; if the sentence cannot be read that way,
//! it changes nothing.

use crate::home::policy::is_whole_home;
use crate::home::roles::matches_domain;
use crate::lang::Catalog;
use crate::parse::compound::expand_compounds;
use crate::parse::exception::{self, Exception, Names};
use crate::parse::infer::wants_all_lights;
use crate::parse::normalize::strip_fillers_for_areas;
use crate::parse::resolve::{domain_hint, resolve};
use crate::parse::respond::speak_unknown;
use crate::parse::slots::is_home_group_light;
use crate::types::{EntityRec, HomeGraph, Intent, ParseDecision, RejectReason};
use std::collections::HashSet;

use super::context::ParseContext;
use super::draft::{execute, reject, Draft};
use super::mood_plan::controls_device;
use super::pipeline::probe;
use super::reach::{reach, without};

/// What a sentence with an exception word says.
pub(super) struct Excepted {
    /// The command, and the devices it must not touch. `None`: the exception cannot be
    /// read, or it names nothing this home knows.
    read: Option<(Exception, HashSet<String>)>,
    /// "All lights except …" with no room in the command. The all-lights clause reads this
    /// form itself, room by room; its plan is trusted once the excepted devices are out.
    every_light: bool,
}

/// `None` if the sentence takes nothing out of its command.
pub(super) fn read(context: &ParseContext<'_>, raw: &[String]) -> Option<Excepted> {
    let home = context.home;
    if !exception::hinted(raw, context.catalog) {
        return None;
    }
    let found = exception::read(raw, context.catalog, &Names::of(home))?;
    let read = found.ok().and_then(|exception| {
        let mut protected = HashSet::new();
        for item in &exception.items {
            protected.extend(named_by(&normal(item, home), home, context.catalog)?);
        }
        Some((exception, protected))
    });
    let every_light = read.as_ref().is_some_and(|(exception, _)| {
        let command = normal(&exception.command, home);
        let aimed = resolve(&command, home, Some("light"));
        let rooms = aimed.areas.iter().any(|id| home.areas.iter().any(|area| area.area_id == *id && !is_whole_home(area)));
        wants_all_lights(&normal(raw, home)) && !rooms && aimed.floors.is_empty()
    });
    Some(Excepted { read, every_light })
}

impl Excepted {
    /// The command read on its own, without the excepted devices. `None` leaves the
    /// sentence to the other routes; `heed` then checks what they made of it.
    pub(super) fn route(&self, context: &ParseContext<'_>) -> Option<Draft> {
        let (exception, protected) = self.read.as_ref()?;
        if self.every_light {
            return None;
        }
        let head = probe(context, &exception.command.join(" "));
        if !matches!(head.decision, ParseDecision::Execute | ParseDecision::Confirm { .. }) || !head.intents.iter().any(changes) {
            return None;
        }
        Some(match spared(&head.intents, context.home, protected) {
            Some(kept) => execute(context, kept, "exception", head.confidence, head.margin, false, false),
            None => reject(RejectReason::NoTarget, speak_unknown()),
        })
    }

    /// The last word on a draft from another route. A plan from the all-lights clause runs
    /// without the excepted devices. Anything else that would change a device is dropped:
    /// its exception was not understood, so the plan may hold exactly what was excepted.
    pub(super) fn heed(&self, context: &ParseContext<'_>, draft: Draft) -> Draft {
        let steps = match &draft.decision {
            ParseDecision::Execute => draft.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default(),
            ParseDecision::Clarify { .. } => draft.commit.clarify.iter().map(|(_, template)| template.clone()).collect(),
            _ => Vec::new(),
        };
        if !steps.iter().any(changes) {
            return draft;
        }
        let kept = match (&self.read, &draft.decision) {
            (Some((_, protected)), ParseDecision::Execute) if self.every_light => spared(&steps, context.home, protected),
            _ => None,
        };
        match kept {
            Some(kept) if kept == steps => draft,
            Some(kept) => {
                let mut next = execute(context, kept, "exception", draft.confidence, draft.margin, draft.commit.clear_pending, false);
                next.competing = draft.competing;
                next
            }
            None => reject(RejectReason::NoTarget, speak_unknown()),
        }
    }
}

fn changes(intent: &Intent) -> bool {
    controls_device(&intent.name)
}

/// The plan without the protected devices. `None` if nothing is left to change, or if a
/// step starts a scene or a script or switches a group: what those reach is not known, so
/// nothing can be taken out of them.
fn spared(intents: &[Intent], home: &HomeGraph, protected: &HashSet<String>) -> Option<Vec<Intent>> {
    let opaque = |intent: &Intent| {
        let domain = intent.slot("entity_id").and_then(|id| id.split_once('.')).map(|(domain, _)| domain).or(intent.slot("domain"));
        let group =
            reach(intent, home).into_iter().any(|entity| entity.platform.as_deref() == Some("group") || is_home_group_light(entity, home));
        changes(intent) && (group || matches!(domain, Some("scene" | "script" | "group")))
    };
    if intents.iter().any(opaque) {
        return None;
    }
    let kept = without(intents, home, protected);
    kept.iter().any(changes).then_some(kept)
}

/// The tokens as the resolver expects them: fillers out, room names and compounds joined.
fn normal(tokens: &[String], home: &HomeGraph) -> Vec<String> {
    expand_compounds(&strip_fillers_for_areas(tokens, &home.areas), home).tokens
}

/// Every device one excepted item stands for. A device name protects that device, and
/// every device of that name if several share it. A room or a floor protects what is in
/// it, narrowed to a kind of device if the item names one ("the lights in the bedroom").
/// A device named with its room ("the lamp in the living room") protects that device. A kind alone ("the blinds") protects every device of that kind. `None` if the item
/// names nothing in this home.
fn named_by(tokens: &[String], home: &HomeGraph, cat: &Catalog) -> Option<HashSet<String>> {
    let resolved = resolve(tokens, home, None);
    let mut named: HashSet<String> = resolved.entities.iter().chain(&resolved.ambiguous).map(|entity| entity.entity_id.clone()).collect();
    let kind = domain_hint(tokens);
    let on_floors = resolved.floors.iter().flat_map(|floor| home.areas_on_floor(floor)).map(|area| area.area_id.as_str());
    let areas: Vec<&str> = resolved.areas.iter().map(String::as_str).chain(on_floors).collect();
    if areas.is_empty() {
        if named.is_empty() {
            let kind = kind?;
            named.extend(home.entities.iter().filter(|entity| matches_domain(entity, kind, cat)).map(|entity| entity.entity_id.clone()));
        }
        return Some(named);
    }
    // "The lamp in the living room" names one device; "the living room" names the room.
    let placed = |token: &String| {
        let alone = resolve(std::slice::from_ref(token), home, None);
        !alone.areas.is_empty() || !alone.floors.is_empty()
    };
    if !named.is_empty() && !tokens.iter().all(placed) {
        return Some(named);
    }
    let there: Vec<&EntityRec> =
        home.entities.iter().filter(|entity| entity.area.as_deref().is_some_and(|area| areas.contains(&area))).collect();
    let of_kind: Vec<&EntityRec> =
        there.iter().copied().filter(|entity| kind.is_some_and(|kind| matches_domain(entity, kind, cat))).collect();
    let inside = if of_kind.is_empty() { there } else { of_kind };
    named.extend(inside.into_iter().map(|entity| entity.entity_id.clone()));
    Some(named)
}
