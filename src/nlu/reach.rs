//! Which devices a plan step reaches once Home Assistant expands rooms, floors and
//! domains, and how to take devices out of a plan.

use crate::home::entrance::is_entrance;
use crate::home::expose::assist_visible;
use crate::home::policy::is_nlu_ignored;
use crate::lang::Catalog;
use crate::types::{EntityRec, HomeGraph, Intent, IntentPlan};
use std::collections::HashSet;

use super::mood_plan::changes_state;

/// Every exposed device a step reaches. A step that names a device reaches that device. A
/// step that names a room or a floor reaches everything there, narrowed by the domain only
/// if the step carries one; that is how Home Assistant matches a room-wide intent.
pub(super) fn reach<'a>(intent: &Intent, home: &'a HomeGraph) -> Vec<&'a EntityRec> {
    if let Some(id) = intent.slot("entity_id") {
        return home.entities.iter().filter(|entity| entity.entity_id == id).collect();
    }
    let (area, floor, domain) = (intent.slot("area"), intent.slot("floor"), intent.slot("domain"));
    if area.is_none() && floor.is_none() {
        return Vec::new();
    }
    let on_floor = |area_id: &str, floor_id: &str| home.areas_on_floor(floor_id).any(|record| record.area_id == area_id);
    home.entities
        .iter()
        .filter(|entity| assist_visible(entity, home))
        .filter(|entity| area.is_none_or(|wanted| entity.area.as_deref() == Some(wanted)))
        .filter(|entity| floor.is_none_or(|wanted| entity.area.as_deref().is_some_and(|area_id| on_floor(area_id, wanted))))
        .filter(|entity| domain.is_none_or(|wanted| entity.domain == wanted))
        .collect()
}

/// A room or floor step that changes state and names no domain. Home Assistant would run
/// it on every device there: lights, but also the lock and the garage door.
pub(super) fn unbounded(intent: &Intent) -> bool {
    changes_state(&intent.name)
        && intent.slot("entity_id").is_none()
        && intent.slot("domain").is_none()
        && (intent.slot("area").is_some() || intent.slot("floor").is_some())
}

/// The step changes a lock, a garage door, a gate or a door, whichever way it moves it.
pub(super) fn moves_entrance(intent: &Intent, home: &HomeGraph, cat: &Catalog) -> bool {
    changes_state(&intent.name) && reach(intent, home).iter().any(|entity| is_entrance(entity, home, cat))
}

/// The same plan without the `excluded` devices. A room or floor step that reaches one of
/// them is replaced by one step per remaining device, so the excluded one is never part of
/// what Home Assistant expands. Steps that only read are kept as they are.
pub(super) fn without(intents: &[Intent], home: &HomeGraph, excluded: &HashSet<String>) -> Vec<Intent> {
    let mut kept = Vec::new();
    for intent in intents {
        let targets = reach(intent, home);
        if !changes_state(&intent.name) || !targets.iter().any(|entity| excluded.contains(&entity.entity_id)) {
            kept.push(intent.clone());
            continue;
        }
        kept.extend(targets.into_iter().filter(|entity| !excluded.contains(&entity.entity_id)).map(|entity| aimed(intent, entity)));
    }
    kept
}

/// The step aimed at one device instead of a room or a floor.
fn aimed(intent: &Intent, entity: &EntityRec) -> Intent {
    let mut step = Intent::new(intent.name.clone()).with("entity_id", entity.entity_id.clone());
    for slot in &intent.slots {
        let scope = matches!(slot.name.as_str(), "entity_id" | "area" | "floor");
        if !scope && (slot.name != "domain" || slot.value == entity.domain) {
            step = step.with(slot.name.clone(), slot.value.clone());
        }
    }
    step
}

/// The plan without the devices the user marked "ignore this target", or `None` if it
/// reaches none of them. The mark holds for every way a device can end up in a plan: by
/// name, as the only device of its kind, as a follow-up and as part of a room.
pub(super) fn without_ignored(plan: &IntentPlan, home: &HomeGraph) -> Option<IntentPlan> {
    let ignored: HashSet<String> =
        home.entities.iter().filter(|entity| is_nlu_ignored(entity)).map(|entity| entity.entity_id.clone()).collect();
    if ignored.is_empty() {
        return None;
    }
    let intents = plan.intents();
    let named = |intent: &Intent| intent.slot("entity_id").is_some_and(|id| ignored.contains(id));
    let kept: Vec<Intent> = without(&intents, home, &ignored).into_iter().filter(|intent| !named(intent)).collect();
    (kept != intents).then(|| {
        let mut cleared = IntentPlan::from_intents(kept, plan.confidence, &plan.evidence);
        cleared.margin = plan.margin;
        cleared
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AreaRec;

    fn entity(id: &str, area: &str) -> EntityRec {
        EntityRec {
            entity_id: id.into(),
            name: id.into(),
            domain: id.split('.').next().unwrap_or("").into(),
            platform: None,
            area: Some(area.into()),
            aliases: Vec::new(),
            tags: Vec::new(),
        }
    }

    fn home() -> HomeGraph {
        HomeGraph {
            areas: vec![AreaRec { area_id: "hall".into(), name: "Hall".into(), aliases: Vec::new(), floor_id: None }],
            entities: vec![entity("light.hall", "hall"), entity("light.floor_lamp", "hall"), entity("lock.front", "hall")],
            ..HomeGraph::default()
        }
    }

    fn ids(intents: &[Intent]) -> Vec<String> {
        intents.iter().map(|intent| intent.slot("entity_id").unwrap_or("-").to_string()).collect()
    }

    #[test]
    fn a_room_step_reaches_the_domain_or_everything() {
        let home = home();
        let lights = Intent::new("HassTurnOff").with("area", "hall").with("domain", "light");
        assert_eq!(reach(&lights, &home).len(), 2);
        let everything = Intent::new("HassTurnOff").with("area", "hall");
        assert_eq!(reach(&everything, &home).len(), 3);
        assert!(unbounded(&everything) && !unbounded(&lights));
        assert!(!unbounded(&Intent::new("HassGetState").with("area", "hall")), "reading a room changes nothing");
    }

    #[test]
    fn an_excluded_device_leaves_one_step_per_remaining_device() {
        let home = home();
        let lights = Intent::new("HassLightSet").with("area", "hall").with("domain", "light").with("brightness", "40");
        let kept = without(&[lights], &home, &HashSet::from(["light.floor_lamp".to_string()]));
        assert_eq!(ids(&kept), ["light.hall"]);
        assert_eq!(kept[0].slot("brightness"), Some("40"));
        assert_eq!(kept[0].slot("area"), None);
    }

    #[test]
    fn untouched_steps_and_queries_stay_as_they_are() {
        let home = home();
        let lights = Intent::new("HassTurnOff").with("area", "hall").with("domain", "light");
        let query = Intent::new("HassGetState").with("entity_id", "lock.front");
        let kept = without(&[lights.clone(), query.clone()], &home, &HashSet::from(["lock.front".to_string()]));
        assert_eq!(kept, vec![lights, query]);
    }
}
