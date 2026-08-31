use crate::types::{fill_speech, pick_speech, ParseDecision, PolicyRule};

use super::context::ParseContext;
use super::draft::Draft;

pub(super) fn apply_rule_speech(draft: &mut Draft, context: &ParseContext<'_>, rule: Option<&PolicyRule>) {
    let Some(rule) = rule else {
        return;
    };
    let language = context.catalog.langs.first().map(|lang| lang.code()).unwrap_or("en");
    let Some(template) = pick_speech(
        context.speech_bank,
        &rule.id,
        language,
        context.settings.personality,
        &context.session.id,
        context.session.last.len() as u64,
    ) else {
        return;
    };
    let filled = draft.plan.as_ref().map(|plan| fill_speech(&template, plan)).unwrap_or(template);
    draft.speech = filled.clone();
    if let ParseDecision::Confirm { prompt, .. } = &mut draft.decision {
        *prompt = filled;
    }
}

pub(super) fn confirmation_prompt(context: &ParseContext<'_>) -> String {
    context.catalog.speech().confirm.to_string()
}
