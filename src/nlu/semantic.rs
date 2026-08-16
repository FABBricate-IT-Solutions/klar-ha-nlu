//! Provider-independent semantic adapters. Off by default; proposals are typed and revalidated.

use crate::home::expose::assist_visible;
use crate::lang::VerbKind;
use crate::types::{Evidence, HomeGraph, Intent, IntentPlan};
use std::cell::RefCell;

use super::context::ParseContext;
use super::draft::{self, Draft};
use super::validation::validate_plan;

pub const ADAPTER_CONFIDENCE_CAP: f64 = 0.86;

#[derive(Debug, Clone)]
pub struct SemanticProposal {
    pub provider: String,
    pub plan: IntentPlan,
    pub reason: String,
}

pub struct SemanticRequest<'a> {
    pub text: &'a str,
    pub tokens: &'a [String],
    pub home: &'a HomeGraph,
}

pub trait SemanticAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn propose(&self, request: &SemanticRequest<'_>) -> Vec<SemanticProposal>;
}

thread_local! {
    static FIXTURES: RefCell<Vec<SemanticProposal>> = const { RefCell::new(Vec::new()) };
}

pub fn with_fixture_proposals<T>(proposals: Vec<SemanticProposal>, operation: impl FnOnce() -> T) -> T {
    FIXTURES.with(|slot| *slot.borrow_mut() = proposals);
    let result = operation();
    FIXTURES.with(|slot| slot.borrow_mut().clear());
    result
}

pub(super) fn consider(context: &ParseContext<'_>, tokens: &[String], rejected: &Draft) -> Option<Draft> {
    if !context.settings.semantic_adapters || !matches!(rejected.decision, crate::types::ParseDecision::Reject { .. }) {
        return None;
    }
    let request = SemanticRequest { text: context.text, tokens, home: context.home };
    let mut proposals = fixture_proposals();
    proposals.extend(LocalEntityHintAdapter.propose(&request));
    for proposal in proposals {
        if let Some(draft) = revalidate(context, proposal) {
            return Some(draft);
        }
    }
    None
}

fn fixture_proposals() -> Vec<SemanticProposal> {
    FIXTURES.with(|slot| slot.borrow().clone())
}

fn revalidate(context: &ParseContext<'_>, mut proposal: SemanticProposal) -> Option<Draft> {
    cap_proposal_confidence(&mut proposal.plan);
    if proposal.plan.evidence.is_empty() {
        let evidence = Evidence {
            kind: "semantic_adapter".into(),
            source: proposal.provider.clone(),
            value: proposal.reason.clone(),
            score: proposal.plan.confidence,
            exact: false,
        };
        proposal.plan.evidence.push(evidence.clone());
        for step in &mut proposal.plan.steps {
            step.evidence.push(evidence.clone());
        }
    }
    if validate_plan(&proposal.plan, context.home).is_err() {
        return None;
    }
    Some(draft::from_adapter_plan(context, proposal.plan, &proposal.provider))
}

fn cap_proposal_confidence(plan: &mut IntentPlan) {
    plan.confidence = plan.confidence.min(ADAPTER_CONFIDENCE_CAP);
    for step in &mut plan.steps {
        step.confidence = step.confidence.min(ADAPTER_CONFIDENCE_CAP);
        for item in &mut step.evidence {
            item.score = item.score.min(ADAPTER_CONFIDENCE_CAP);
        }
    }
    for item in &mut plan.evidence {
        item.score = item.score.min(ADAPTER_CONFIDENCE_CAP);
    }
}

struct LocalEntityHintAdapter;

impl SemanticAdapter for LocalEntityHintAdapter {
    fn id(&self) -> &'static str {
        "local.entity_hint"
    }

    fn propose(&self, request: &SemanticRequest<'_>) -> Vec<SemanticProposal> {
        let Some(intent_name) = hint_intent(request.tokens) else {
            return Vec::new();
        };
        let leftovers: Vec<&str> = request
            .tokens
            .iter()
            .map(String::as_str)
            .filter(|token| request.home.entities.iter().any(|entity| name_hit(entity, token)))
            .collect();
        if leftovers.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<&str> = request
            .home
            .entities
            .iter()
            .filter(|entity| assist_visible(entity, request.home) && leftovers.iter().any(|token| name_hit(entity, token)))
            .map(|entity| entity.entity_id.as_str())
            .collect();
        hits.sort_unstable();
        hits.dedup();
        let [entity_id] = hits.as_slice() else {
            return Vec::new();
        };
        let intent = Intent::new(intent_name).with("entity_id", *entity_id);
        vec![SemanticProposal {
            provider: self.id().into(),
            plan: IntentPlan::from_intents(vec![intent], 0.82, &[]),
            reason: format!("unique visible entity {entity_id}"),
        }]
    }
}

fn hint_intent(tokens: &[String]) -> Option<&'static str> {
    let catalog = crate::lang::catalog();
    tokens.iter().find_map(|token| match catalog.verb(token) {
        Some(VerbKind::On | VerbKind::OnParticle) => Some("HassTurnOn"),
        Some(VerbKind::Off) => Some("HassTurnOff"),
        _ => None,
    })
}

fn name_hit(entity: &crate::types::EntityRec, token: &str) -> bool {
    let compact = |value: &str| value.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>();
    let needle = compact(token);
    needle.len() >= 4
        && (compact(&entity.name) == needle
            || entity.aliases.iter().any(|alias| compact(alias) == needle)
            || compact(&entity.entity_id) == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Intent;

    #[test]
    fn invalid_intent_name_is_not_a_proposal() {
        let plan = IntentPlan::from_intents(vec![Intent::new("NotAnIntent").with("entity_id", "light.wohnzimmer")], 0.9, &[]);
        assert!(validate_plan(&plan, &crate::home::default_home()).is_err());
    }

    #[test]
    fn local_hint_needs_unique_visible_name_and_verb() {
        let mut home = crate::home::default_home();
        home.entities.push(crate::types::EntityRec {
            entity_id: "light.zyxlamp".into(),
            name: "Zyxlamp".into(),
            domain: "light".into(),
            platform: None,
            area: Some("wohnzimmer".into()),
            aliases: vec!["zyxlamp".into()],
            tags: Vec::new(),
        });
        let tokens = |parts: &[&str]| parts.iter().map(|part| (*part).to_string()).collect::<Vec<_>>();
        let hit = LocalEntityHintAdapter.propose(&SemanticRequest { text: "", tokens: &tokens(&["on", "zyxlamp"]), home: &home });
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0].plan.intents()[0].slot("entity_id"), Some("light.zyxlamp"));
        assert!(LocalEntityHintAdapter.propose(&SemanticRequest { text: "", tokens: &tokens(&["zyxlamp"]), home: &home }).is_empty());
        home.entities.push(crate::types::EntityRec {
            entity_id: "light.zyxlamp_2".into(),
            name: "Zyxlamp".into(),
            domain: "light".into(),
            platform: None,
            area: Some("wohnzimmer".into()),
            aliases: vec!["zyxlamp".into()],
            tags: Vec::new(),
        });
        assert!(LocalEntityHintAdapter.propose(&SemanticRequest { text: "", tokens: &tokens(&["on", "zyxlamp"]), home: &home }).is_empty());
    }
}
