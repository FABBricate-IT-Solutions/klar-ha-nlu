//! Language-free govern safety seeds. Bound with every pack; house ids replace.

use super::{matches_when, IntentPlan, PolicyEffect, PolicyHit, PolicyMatch, PolicyRule};
use std::collections::HashSet;
use std::sync::LazyLock;

pub const SEED_CONFIRM_LOCK: &str = "seed:confirm-lock";
pub const SEED_CONFIRM_COVER_CLOSE: &str = "seed:confirm-cover-close";
pub const SEED_BLOCK_AREA_LOCK: &str = "seed:block-area-lock";

pub fn govern_safety_seeds() -> &'static [PolicyRule] {
    static SEEDS: LazyLock<Vec<PolicyRule>> = LazyLock::new(|| {
        vec![
            seed(
                SEED_BLOCK_AREA_LOCK,
                "Area-wide lock",
                PolicyMatch { domain: Some("lock".into()), area_wide: true, ..PolicyMatch::default() },
                PolicyEffect::Block,
            ),
            seed(SEED_CONFIRM_LOCK, "Lock", PolicyMatch { domain: Some("lock".into()), ..PolicyMatch::default() }, PolicyEffect::Confirm),
            seed(
                SEED_CONFIRM_COVER_CLOSE,
                "Cover close",
                PolicyMatch { intent: Some("HassTurnOff".into()), domain: Some("cover".into()), ..PolicyMatch::default() },
                PolicyEffect::Confirm,
            ),
        ]
    });
    SEEDS.as_slice()
}

fn seed(id: &str, label: &str, when: PolicyMatch, effect: PolicyEffect) -> PolicyRule {
    PolicyRule { id: id.into(), enabled: true, label: label.into(), when, effect, prefer: None, payload: None }
}

pub fn is_seed_id(id: &str) -> bool {
    id.starts_with("seed:")
}

pub fn first_seed_match(house: &[PolicyRule], plan: &IntentPlan) -> Option<(&'static PolicyRule, PolicyHit)> {
    let replaced: HashSet<&str> = house.iter().map(|rule| rule.id.as_str()).collect();
    govern_safety_seeds().iter().filter(|rule| rule.enabled && !replaced.contains(rule.id.as_str())).find_map(|rule| {
        plan.steps.iter().any(|step| matches_when(&rule.when, &step.intent)).then_some((rule, PolicyHit::from_effect(rule.effect)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Intent;

    fn lock_entity() -> IntentPlan {
        IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("entity_id", "lock.wohnungstuer").with("domain", "lock")], 1.0, &[])
    }

    fn lock_area() -> IntentPlan {
        IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("area", "wohnzimmer").with("domain", "lock")], 1.0, &[])
    }

    #[test]
    fn entity_lock_hits_confirm_not_block() {
        let (rule, hit) = first_seed_match(&[], &lock_entity()).expect("seed");
        assert_eq!(rule.id, SEED_CONFIRM_LOCK);
        assert_eq!(hit, PolicyHit::Confirm);
    }

    #[test]
    fn area_lock_hits_block_seed() {
        let (rule, hit) = first_seed_match(&[], &lock_area()).expect("seed");
        assert_eq!(rule.id, SEED_BLOCK_AREA_LOCK);
        assert_eq!(hit, PolicyHit::Block);
    }

    #[test]
    fn house_id_replaces_seed() {
        let house = vec![seed(SEED_CONFIRM_LOCK, "off", PolicyMatch::default(), PolicyEffect::Confirm)];
        assert!(first_seed_match(&house, &lock_entity()).is_none());
        assert!(is_seed_id(SEED_CONFIRM_LOCK));
        assert!(!is_seed_id("block-ac"));
    }
}
