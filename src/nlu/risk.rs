//! What the user's rules and the built-in risk classes say about one plan, step by step.

use crate::types::{allow_permitted_step, step_rules, IntentPlan, PolicyHit, PolicyRule};

use super::validation::{risky_intent, BROAD_PLAN_STEPS};

pub(super) struct Verdict<'a> {
    /// The rule that decided the outcome, for the trace and for rule speech.
    pub rule: Option<(&'a PolicyRule, PolicyHit)>,
    pub blocked: bool,
    pub risky: bool,
}

/// `inherited` marks a plan that reverses an action which itself needed a confirmation.
pub(super) fn judge<'a>(rules: &'a [PolicyRule], plan: &IntentPlan, confirm_on: bool, inherited: bool) -> Verdict<'a> {
    let hits = step_rules(rules, plan);
    let first = |wanted: PolicyHit| hits.iter().flatten().find(|(_, hit)| *hit == wanted).copied();
    if let Some(block) = first(PolicyHit::Block) {
        return Verdict { rule: Some(block), blocked: true, risky: false };
    }
    let allowed: Vec<bool> = plan
        .steps
        .iter()
        .zip(&hits)
        .map(|(step, hit)| matches!(hit, Some((_, PolicyHit::Allow))) && allow_permitted_step(&step.intent))
        .collect();
    let every_step_allowed = allowed.iter().all(|allowed| *allowed);
    let asked_by_rule = confirm_on && first(PolicyHit::Confirm).is_some();
    let step_risk = plan.steps.iter().zip(&allowed).any(|(step, allowed)| !allowed && risky_intent(&step.intent));
    let plan_risk = !every_step_allowed && (inherited || plan.steps.len() >= BROAD_PLAN_STEPS);
    let risky = asked_by_rule || (confirm_on && (step_risk || plan_risk));
    let rule = if asked_by_rule {
        first(PolicyHit::Confirm)
    } else if risky {
        // An allow on another step did not decide this prompt; do not speak for it.
        hits.iter().flatten().find(|(_, hit)| *hit != PolicyHit::Allow).copied()
    } else {
        hits.iter().flatten().next().copied()
    };
    Verdict { rule, blocked: false, risky }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Intent, PolicyEffect, PolicyMatch};

    fn rule(entity: &str, effect: PolicyEffect) -> PolicyRule {
        PolicyRule {
            id: format!("rule-{entity}"),
            enabled: true,
            label: entity.into(),
            when: PolicyMatch { entity_id: Some(entity.into()), ..PolicyMatch::default() },
            effect,
            prefer: None,
            payload: None,
        }
    }

    fn plan(intents: Vec<Intent>) -> IntentPlan {
        IntentPlan::from_intents(intents, 1.0, &[])
    }

    fn close(entity: &str) -> Intent {
        Intent::new("HassTurnOff").with("entity_id", entity)
    }

    #[test]
    fn allow_lifts_only_the_step_it_matched() {
        let rules = [rule("cover.blinds", PolicyEffect::Allow)];
        assert!(!judge(&rules, &plan(vec![close("cover.blinds")]), true, false).risky);
        let mixed = judge(&rules, &plan(vec![close("cover.blinds"), close("lock.front")]), true, false);
        assert!(mixed.risky, "the lock was never allowed");
        assert!(mixed.rule.is_none(), "the allow rule did not decide the prompt");
    }

    #[test]
    fn block_on_a_later_step_is_not_shadowed_by_an_earlier_allow() {
        let rules = [rule("cover.blinds", PolicyEffect::Allow), rule("lock.front", PolicyEffect::Block)];
        let verdict = judge(&rules, &plan(vec![close("cover.blinds"), close("lock.front")]), true, false);
        assert!(verdict.blocked);
        assert_eq!(verdict.rule.map(|(rule, _)| rule.id.as_str()), Some("rule-lock.front"));
    }

    #[test]
    fn inherited_risk_asks_unless_every_step_is_allowed() {
        let open = plan(vec![Intent::new("HassTurnOn").with("entity_id", "cover.blinds")]);
        assert!(!judge(&[], &open, true, false).risky);
        assert!(judge(&[], &open, true, true).risky);
        assert!(!judge(&[rule("cover.blinds", PolicyEffect::Allow)], &open, true, true).risky);
        assert!(!judge(&[], &open, false, true).risky, "confirmations are switched off");
    }

    #[test]
    fn confirm_rule_asks_for_a_harmless_step() {
        let rules = [rule("light.desk", PolicyEffect::Confirm)];
        let verdict = judge(&rules, &plan(vec![Intent::new("HassTurnOn").with("entity_id", "light.desk")]), true, false);
        assert!(verdict.risky);
        assert_eq!(verdict.rule.map(|(_, hit)| hit), Some(PolicyHit::Confirm));
        assert!(!judge(&rules, &plan(vec![Intent::new("HassTurnOn").with("entity_id", "light.desk")]), false, false).risky);
    }
}
