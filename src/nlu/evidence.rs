use crate::lang::Catalog;
use crate::parse::action::{detect_actions_bounded, Action};
use crate::parse::infer::prefer_action;
use crate::parse::resolve::ResolveEvidence;
use crate::types::Evidence;

#[derive(Debug, Clone)]
pub(super) struct ActionHypothesis {
    pub action: Action,
    pub evidence: Evidence,
}

pub(super) fn action_hypotheses(tokens: &[String], catalog: &'static Catalog) -> Vec<ActionHypothesis> {
    let mut detected = super::legacy::with_catalog(catalog, || detect_actions_bounded(tokens, super::binding::MAX_ACTION_HYPOTHESES));
    let selected = prefer_action(&detected);
    detected.sort_by_key(|(index, action)| (u8::from(Some(*action) != selected), *index));
    let mut rows = detected
        .into_iter()
        .map(|(index, action)| {
            let exact = tokens.get(index).is_some_and(|token| catalog.verb(token).is_some());
            let preferred = Some(action) == selected;
            ActionHypothesis {
                action,
                evidence: Evidence {
                    kind: "action".into(),
                    source: if exact { "lexicon_exact" } else { "lexicon_fuzzy" }.into(),
                    value: action_name(action).into(),
                    score: match (preferred, exact) {
                        (true, true) => 1.0,
                        (true, false) => 0.88,
                        (false, true) => 0.72,
                        (false, false) => 0.62,
                    },
                    exact,
                },
            }
        })
        .collect::<Vec<_>>();
    rows.dedup_by_key(|row| row.action);
    rows
}

pub(super) fn inferred_action_evidence(action: Action) -> Evidence {
    Evidence { kind: "action".into(), source: "context_inference".into(), value: action_name(action).into(), score: 0.65, exact: false }
}

pub(super) fn target_evidence(value: &ResolveEvidence) -> Evidence {
    Evidence {
        kind: format!("target_{}", value.kind),
        source: if value.exact { "resolver_exact" } else { "resolver_fuzzy" }.into(),
        value: value.target.clone(),
        score: value.score,
        exact: value.exact,
    }
}

pub(super) fn action_name(action: Action) -> &'static str {
    match action {
        Action::On => "on",
        Action::Off => "off",
        Action::Toggle => "toggle",
        Action::SetLight => "set_light",
        Action::SetTemp => "set_temperature",
        Action::GetState => "get_state",
        Action::MediaPause => "media_pause",
        Action::MediaPlay => "media_play",
        Action::MediaNext => "media_next",
        Action::MediaMute => "media_mute",
        Action::FanSpeed => "fan_speed",
        Action::VacuumStart => "vacuum_start",
        Action::VacuumDock => "vacuum_dock",
        Action::Scene => "scene",
        Action::CoverOpen => "cover_open",
        Action::CoverClose => "cover_close",
        Action::CoverSet => "cover_set",
        Action::Lock => "lock",
        Action::Unlock => "unlock",
        Action::TimerStart => "timer_start",
        Action::TimerAdd => "timer_add",
        Action::TimerCancel => "timer_cancel",
        Action::TimerPause => "timer_pause",
        Action::ListAdd => "list_add",
        Action::ListComplete => "list_complete",
        Action::ClarifyWrong => "clarify_wrong",
    }
}
