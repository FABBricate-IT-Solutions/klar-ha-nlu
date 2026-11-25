use crate::types::{HomeGraph, Intent};
use std::collections::BTreeSet;

pub(super) fn dedup_intents(intents: &mut Vec<Intent>, home: &HomeGraph) {
    let mut seen = BTreeSet::new();
    intents.retain(|intent| {
        let entity_domain = intent.slot("entity_id").and_then(|id| id.split_once('.').map(|(domain, _)| domain));
        let mut slots: Vec<_> = intent
            .slots
            .iter()
            .filter(|slot| slot.name != "domain" || entity_domain != Some(slot.value.as_str()))
            .map(|slot| format!("{}={}", slot.name, slot.value))
            .collect();
        slots.sort();
        seen.insert(format!("{}|{}", intent.name, slots.join("|")))
    });
    // Drop area steps only when entity steps already cover every device there (P-32).
    let drop: Vec<bool> =
        intents.iter().map(|intent| intent.slot("entity_id").is_none() && area_fully_covered(intent, intents, home)).collect();
    let mut index = 0;
    intents.retain(|_| {
        let remove = drop[index];
        index += 1;
        !remove
    });
}

fn area_fully_covered(intent: &Intent, intents: &[Intent], home: &HomeGraph) -> bool {
    let Some(area) = intent.slot("area") else {
        return false;
    };
    let domain = intent.slot("domain").unwrap_or("");
    let room: Vec<_> = home
        .entities
        .iter()
        .filter(|entity| {
            entity.area.as_deref() == Some(area)
                && (domain.is_empty() || entity.domain == domain)
                && crate::home::expose::assist_visible(entity, home)
                && !crate::home::policy::is_infra(entity)
        })
        .map(|entity| entity.entity_id.as_str())
        .collect();
    !room.is_empty() && room.iter().all(|id| intents.iter().any(|step| step.name == intent.name && step.slot("entity_id") == Some(*id)))
}
