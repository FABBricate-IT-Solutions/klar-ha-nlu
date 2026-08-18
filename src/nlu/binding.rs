use crate::parse::action::{domain_for, Action};
use crate::parse::clause::{parse_clause_candidates_for_action, ClauseCandidate};
use crate::parse::compound::CompoundSplit;
use crate::parse::infer::looks_like_question;
use crate::parse::normalize::tokenize;
use crate::parse::policy::{candidate, PolicyId};
use crate::parse::resolve::{resolve_scored, ResolveReport};
use crate::parse::slots::{intent_from_action, ClauseOut};
use crate::types::{Evidence, HomeGraph, Intent, ParseTrace, StageTrace, MAX_CLARIFY_OPTIONS, MAX_PLAN_STEPS};
use std::time::Instant;

use super::context::ParseContext;
use super::evidence::{action_hypotheses, inferred_action_evidence, ActionHypothesis};
use super::ranking::{provisional_selection, MAX_BINDINGS_PER_CLAUSE};

pub(super) const MAX_CLAUSES: usize = 16;
pub(super) const MAX_ACTION_HYPOTHESES: usize = 8;
const MAX_POLICIES_PER_ACTION: usize = 16;
const MAX_TARGETS_PER_REPORT: usize = 16;

#[derive(Debug, Clone, Copy)]
pub(super) struct ComplexityLimit;

pub(super) struct Analysis {
    pub index: usize,
    pub bindings: Vec<BindingAnalysis>,
}

pub(super) struct BindingAnalysis {
    pub action_evidence: Evidence,
    pub targets: ResolveReport,
    pub policy: ClauseCandidate,
    pub allowed_targets: Option<Vec<String>>,
}

pub(super) fn build_analyses(
    context: &ParseContext<'_>,
    clauses: Vec<Vec<String>>,
    raw_tokens: &[String],
    split: &CompoundSplit,
    trace: &mut ParseTrace,
) -> Result<Vec<Analysis>, ComplexityLimit> {
    if clauses.len() > MAX_CLAUSES {
        return Err(ComplexityLimit);
    }
    let started = Instant::now();
    let action_rows = clauses
        .iter()
        .map(|clause| {
            let mut hypotheses = action_hypotheses(clause, context.catalog);
            hypotheses.truncate(MAX_ACTION_HYPOTHESES);
            hypotheses
        })
        .collect::<Vec<_>>();
    record_stage(trace, "action_candidates", started, format!("{} action hypotheses", action_rows.iter().map(Vec::len).sum::<usize>()));
    let targets_started = Instant::now();
    let mut working = context.session.clone();
    working.begin_remember_batch();
    let pending_action = context.session.pending_clarify().and_then(|pending| action_for_intent(&pending.template.name));
    let allowed_targets = context.session.pending_clarify().map(|pending| pending.options.clone());
    if let Some(pending) = working.pending_clarify().cloned() {
        working.remember(&pending.template);
    }
    let mut analyses = Vec::new();
    for (index, (clause, hypotheses)) in clauses.into_iter().zip(action_rows).enumerate() {
        let question = looks_like_question(&clause);
        let cat = context.catalog;
        let question_context = question
            || clause
                .first()
                .is_some_and(|token| matches!(token.as_str(), "ist" | "sind" | "wie" | "was" | "are" | "is" | "how" | "what" | "whats"));
        let list_marker = clause.iter().any(|token| token == "list" || token == "liste");
        let complete_marker = cat.any(&clause, cat.list_complete())
            || cat.any(&clause, cat.off_words())
            || clause.iter().any(|token| matches!(cat.verb(token), Some(crate::lang::VerbKind::ListComplete)));
        let explicit_list_completion = list_marker && complete_marker;
        let mut bindings = Vec::new();
        let mut baseline =
            parse_clause_candidates_for_action(&clause, raw_tokens, context.home, &working, context.settings, &split.light_areas, None);
        baseline.retain(|policy| !invalid_named_list_fallback(&clause, context.home, policy));
        if baseline.len() > MAX_POLICIES_PER_ACTION {
            return Err(ComplexityLimit);
        }
        let baseline_has_list = baseline.iter().any(|policy| policy.policy == PolicyId::List);
        for policy in baseline {
            if policy_too_complex(&policy) {
                return Err(ComplexityLimit);
            }
            let evidence_action = policy_outcome_action(&policy).unwrap_or(policy.action);
            let mut action_evidence = hypotheses
                .iter()
                .find(|hypothesis| hypothesis.action == evidence_action)
                .map(|hypothesis| hypothesis.evidence.clone())
                .unwrap_or_else(|| inferred_action_evidence(evidence_action));
            adjust_contextual_action_evidence(&clause, evidence_action, &mut action_evidence);
            if explicit_list_completion && evidence_action == Action::ListComplete {
                action_evidence.source = "lexicon_exact_list_complete".into();
                action_evidence.score = 1.0;
                action_evidence.exact = true;
            } else if question_context && evidence_action == Action::GetState {
                action_evidence.source = if action_evidence.exact { "lexicon_exact_question" } else { "question_context" }.into();
                action_evidence.score = 0.96;
            } else if question_context {
                action_evidence.source = "question_conflict".into();
                action_evidence.score = action_evidence.score.min(0.45);
                action_evidence.exact = false;
            }
            if pending_action == Some(policy.action) {
                action_evidence.source = "pending_interaction".into();
                action_evidence.score = 0.86;
                action_evidence.exact = false;
            }
            let targets = capped_report(resolve_scored(&clause, context.home, domain_for(policy.action, &clause)));
            let narrowed = lexical_disambiguation(&clause, context.home, &policy);
            push_binding(
                &mut bindings,
                BindingAnalysis {
                    action_evidence: action_evidence.clone(),
                    targets: targets.clone(),
                    policy: policy.clone(),
                    allowed_targets: allowed_targets.clone(),
                },
            )?;
            if let Some(intent) = narrowed {
                push_binding(
                    &mut bindings,
                    BindingAnalysis {
                        action_evidence,
                        targets,
                        policy: candidate(PolicyId::GroundedEntities, policy.action, ClauseOut::Intents(vec![intent])),
                        allowed_targets: allowed_targets.clone(),
                    },
                )?;
            }
        }
        let mut forced = hypotheses.clone();
        if let Some(action) = pending_action.filter(|action| !forced.iter().any(|hypothesis| hypothesis.action == *action)) {
            let mut evidence = inferred_action_evidence(action);
            evidence.source = "pending_interaction".into();
            evidence.score = 0.86;
            evidence.exact = false;
            forced.push(ActionHypothesis { action, evidence });
        }
        forced.truncate(MAX_ACTION_HYPOTHESES);
        for hypothesis in forced {
            let targets = capped_report(resolve_scored(&clause, context.home, domain_for(hypothesis.action, &clause)));
            let policies = parse_clause_candidates_for_action(
                &clause,
                raw_tokens,
                context.home,
                &working,
                context.settings,
                &split.light_areas,
                Some(hypothesis.action),
            );
            if policies.len() > MAX_POLICIES_PER_ACTION {
                return Err(ComplexityLimit);
            }
            for policy in policies {
                if policy_too_complex(&policy) {
                    return Err(ComplexityLimit);
                }
                if invalid_named_list_fallback(&clause, context.home, &policy) {
                    continue;
                }
                if policy.policy == PolicyId::List && !baseline_has_list {
                    continue;
                }
                if !bindings.iter().any(|binding| binding.policy.policy == policy.policy && binding.policy.action == policy.action) {
                    let evidence_action = policy_outcome_action(&policy).unwrap_or(policy.action);
                    let mut policy_evidence = hypotheses
                        .iter()
                        .find(|row| row.action == evidence_action)
                        .map(|row| row.evidence.clone())
                        .unwrap_or_else(|| inferred_action_evidence(evidence_action));
                    adjust_contextual_action_evidence(&clause, evidence_action, &mut policy_evidence);
                    if explicit_list_completion && evidence_action == Action::ListComplete {
                        policy_evidence.source = "lexicon_exact_list_complete".into();
                        policy_evidence.score = 1.0;
                        policy_evidence.exact = true;
                    } else if question_context && evidence_action == Action::GetState {
                        policy_evidence.source = if policy_evidence.exact { "lexicon_exact_question" } else { "question_context" }.into();
                        policy_evidence.score = 0.96;
                    } else if question_context {
                        policy_evidence.source = "question_conflict".into();
                        policy_evidence.score = policy_evidence.score.min(0.45);
                        policy_evidence.exact = false;
                    }
                    push_binding(
                        &mut bindings,
                        BindingAnalysis {
                            action_evidence: policy_evidence,
                            targets: targets.clone(),
                            policy,
                            allowed_targets: allowed_targets.clone(),
                        },
                    )?;
                }
            }
        }
        let analysis = Analysis { index, bindings };
        if let Some(binding_index) = provisional_selection(&analysis) {
            if let ClauseOut::Intents(intents) = &analysis.bindings[binding_index].policy.outcome {
                for intent in intents {
                    working.remember(intent);
                }
            }
        }
        analyses.push(analysis);
    }
    let target_count = analyses.iter().flat_map(|analysis| &analysis.bindings).map(|binding| binding.targets.ranked.len()).sum::<usize>();
    record_stage(trace, "target_resolution", targets_started, format!("{target_count} scored targets"));
    let binding_count = analyses.iter().map(|analysis| analysis.bindings.len()).sum::<usize>();
    record_stage(trace, "binding", Instant::now(), format!("{binding_count} complete candidate plans"));
    Ok(analyses)
}

fn push_binding(bindings: &mut Vec<BindingAnalysis>, binding: BindingAnalysis) -> Result<(), ComplexityLimit> {
    if bindings.len() >= MAX_BINDINGS_PER_CLAUSE {
        return Err(ComplexityLimit);
    }
    bindings.push(binding);
    Ok(())
}

fn capped_report(mut report: ResolveReport) -> ResolveReport {
    report.ranked.truncate(MAX_TARGETS_PER_REPORT);
    report.resolved.entities.truncate(MAX_TARGETS_PER_REPORT);
    report.resolved.areas.truncate(MAX_TARGETS_PER_REPORT);
    report.resolved.floors.truncate(MAX_TARGETS_PER_REPORT);
    report.resolved.ambiguous.truncate(MAX_TARGETS_PER_REPORT);
    report
}

fn adjust_contextual_action_evidence(tokens: &[String], action: Action, evidence: &mut Evidence) {
    let lockish = tokens.iter().any(|token| token.contains("lock") || matches!(token.as_str(), "unlock" | "unlocked" | "secure"));
    let close = tokens.iter().any(|token| matches!(token.as_str(), "close" | "closed" | "shut" | "lower" | "secure"));
    let open = tokens.iter().any(|token| {
        matches!(token.as_str(), "open" | "opened" | "raise" | "unlock" | "unlocked" | "release" | "disengage")
            || (!lockish && token == "up" && !close)
    });
    let on_particle = tokens.windows(2).any(|pair| pair[0] == "on" && !matches!(pair[1].as_str(), "the" | "a" | "an"))
        || tokens.last().is_some_and(|token| token == "on");
    let expected = if lockish {
        if open {
            Some(Action::Off)
        } else if close || on_particle {
            Some(Action::On)
        } else {
            None
        }
    } else if open && !close {
        Some(Action::On)
    } else if close {
        Some(Action::Off)
    } else {
        None
    };
    let aligned = match action {
        Action::On | Action::Lock | Action::CoverOpen => Some(Action::On),
        Action::Off | Action::Unlock | Action::CoverClose => Some(Action::Off),
        _ => None,
    };
    if let (Some(expected), Some(aligned)) = (expected, aligned) {
        if aligned == expected {
            evidence.source = "lexicon_context_exact".into();
            evidence.score = 1.0;
            evidence.exact = true;
        } else {
            evidence.source = "lexicon_context_conflict".into();
            evidence.score = evidence.score.min(0.35);
            evidence.exact = false;
        }
    }
}

fn policy_too_complex(policy: &ClauseCandidate) -> bool {
    match &policy.outcome {
        ClauseOut::Intents(intents) => intents.len() > MAX_PLAN_STEPS,
        ClauseOut::Clarify(options, _) => options.len() > MAX_CLARIFY_OPTIONS,
    }
}

fn record_stage(trace: &mut ParseTrace, stage: &str, started: Instant, detail: String) {
    trace.stages.push(StageTrace { stage: stage.into(), duration_us: started.elapsed().as_micros() as u64, detail });
}

fn lexical_disambiguation(tokens: &[String], home: &HomeGraph, policy: &ClauseCandidate) -> Option<Intent> {
    let ClauseOut::Clarify(choices, _) = &policy.outcome else {
        return None;
    };
    let domain = if tokens.iter().any(|token| token == "sensor") {
        "binary_sensor"
    } else if tokens.iter().any(|token| matches!(token.as_str(), "lock" | "schloss")) {
        "lock"
    } else {
        return None;
    };
    let matches = choices
        .iter()
        .filter(|entity_id| home.entities.iter().any(|entity| entity.entity_id == entity_id.as_str() && entity.domain == domain))
        .collect::<Vec<_>>();
    let [selected] = matches.as_slice() else {
        return None;
    };
    Some(intent_from_action(policy.action, tokens).with("entity_id", (*selected).clone()))
}

fn invalid_named_list_fallback(tokens: &[String], home: &HomeGraph, policy: &ClauseCandidate) -> bool {
    let ClauseOut::Intents(intents) = &policy.outcome else {
        return false;
    };
    let shopping_fallback = intents.iter().any(|intent| intent.slot("name") == Some("shopping_list") && intent.slot("entity_id").is_none());
    let named = home.entities.iter().filter(|entity| entity.domain == "todo").any(|entity| {
        phrase_tokens_present(tokens, &entity.name)
            || entity.aliases.iter().any(|alias| phrase_tokens_present(tokens, alias))
            || entity.tags.iter().any(|tag| phrase_tokens_present(tokens, tag))
    });
    shopping_fallback && named
}

fn phrase_tokens_present(tokens: &[String], phrase: &str) -> bool {
    let words = tokenize(phrase);
    !words.is_empty() && words.iter().all(|word| tokens.contains(word))
}

fn action_for_intent(name: &str) -> Option<Action> {
    match name {
        "HassTurnOn" => Some(Action::On),
        "HassTurnOff" => Some(Action::Off),
        "HassToggle" => Some(Action::Toggle),
        "HassLightSet" => Some(Action::SetLight),
        "HassClimateSetTemperature" => Some(Action::SetTemp),
        "HassGetState" | "HassClimateGetTemperature" => Some(Action::GetState),
        "HassSetPosition" => Some(Action::CoverSet),
        "HassFanSetSpeed" => Some(Action::FanSpeed),
        "HassVacuumStart" => Some(Action::VacuumStart),
        "HassVacuumReturnToBase" => Some(Action::VacuumDock),
        "HassStartTimer" => Some(Action::TimerStart),
        "HassIncreaseTimer" | "HassDecreaseTimer" => Some(Action::TimerAdd),
        "HassCancelTimer" => Some(Action::TimerCancel),
        "HassPauseTimer" => Some(Action::TimerPause),
        "HassListAddItem" | "HassShoppingListAddItem" => Some(Action::ListAdd),
        "HassListCompleteItem" | "HassShoppingListCompleteItem" => Some(Action::ListComplete),
        _ => None,
    }
}

fn policy_outcome_action(policy: &ClauseCandidate) -> Option<Action> {
    match &policy.outcome {
        ClauseOut::Intents(intents) => intents.first().and_then(|intent| action_for_intent(&intent.name)),
        ClauseOut::Clarify(_, template) => action_for_intent(&template.name),
    }
}
