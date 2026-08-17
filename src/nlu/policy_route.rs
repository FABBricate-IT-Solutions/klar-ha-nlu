use crate::parse::fuzzy::{evidence, Profile};
use crate::parse::normalize::{fold_umlaut, join_tokens, tokenize};
use crate::types::{first_matching_rule, script_entity_id, Intent, IntentPlan, PolicyHit, PolicyMatch, PolicyRule, PolicyTrace};

use super::context::ParseContext;
use super::draft::{chat, execute, Draft};

pub(super) fn route_phrase(context: &ParseContext<'_>) -> Option<Draft> {
    let (rule, hit) = first_phrase_action(context.policies, context.text)?;
    apply_action(context, rule, hit)
}

pub(super) fn apply_matched_action(context: &ParseContext<'_>, plan: &IntentPlan) -> Option<Draft> {
    let (rule, hit) = first_matching_rule(context.policies, plan)?;
    hit.is_action().then(|| apply_action(context, rule, hit)).flatten()
}

pub(super) fn action_trace(rule: &PolicyRule, hit: PolicyHit) -> PolicyTrace {
    PolicyTrace {
        matched_rule: Some(rule.id.clone()),
        hit: Some(hit.as_str().into()),
        compiled_risky: false,
        payload: rule.payload.clone(),
    }
}

fn apply_action(context: &ParseContext<'_>, rule: &PolicyRule, hit: PolicyHit) -> Option<Draft> {
    let payload = rule.payload.as_deref().map(str::trim).filter(|text| !text.is_empty());
    let mut draft = match hit {
        PolicyHit::Reply => chat(payload.unwrap_or("").to_string(), false, false),
        PolicyHit::Template | PolicyHit::Llm => chat(String::new(), false, false),
        PolicyHit::Script => {
            let entity_id = script_entity_id(payload.unwrap_or(""))?;
            execute(
                context,
                vec![Intent::new("HassTurnOn").with("entity_id", &entity_id).with("domain", "script")],
                "policy_script",
                1.0,
                1.0,
                false,
                false,
            )
        }
        PolicyHit::Confirm | PolicyHit::Block | PolicyHit::Allow | PolicyHit::PreferEntity | PolicyHit::PreferArea => return None,
    };
    draft.policy_trace = Some(action_trace(rule, hit));
    Some(draft)
}

fn first_phrase_action<'a>(rules: &'a [PolicyRule], text: &str) -> Option<(&'a PolicyRule, PolicyHit)> {
    rules.iter().filter(|rule| rule.enabled).find_map(|rule| {
        let hit = PolicyHit::from_effect(rule.effect);
        (hit.is_action() && matches_phrase(&rule.when, text)).then_some((rule, hit))
    })
}

fn matches_phrase(when: &PolicyMatch, text: &str) -> bool {
    let Some(phrase) = when.phrase.as_deref() else {
        return false;
    };
    let observed = fold_umlaut(text.trim());
    let candidate = fold_umlaut(phrase.trim());
    if observed.is_empty() || candidate.is_empty() {
        return false;
    }
    if observed == candidate {
        return true;
    }
    let tokens = tokenize(&observed);
    let words: Vec<&str> = candidate.split_whitespace().filter(|word| !word.is_empty()).collect();
    if words.len() >= 2 && words.iter().all(|word| tokens.iter().any(|token| token == word)) {
        return true;
    }
    if candidate.chars().count() < 8 {
        return false;
    }
    evidence(&join_tokens(&tokens), &candidate, Profile::Phrase).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrase_matches_folded_and_token_cover() {
        let when = PolicyMatch { phrase: Some("Gute Nacht".into()), ..PolicyMatch::default() };
        assert!(matches_phrase(&when, "gute nacht"));
        assert!(matches_phrase(&when, "Gute Nacht dann"));
        assert!(!matches_phrase(&when, "Licht an"));
    }
}
