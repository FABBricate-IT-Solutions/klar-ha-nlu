use crate::types::{ParseDecision, PolicyRule, Retrieval, RetrievalHit};

use super::context::ParseContext;

pub(super) fn build(context: &ParseContext<'_>, decision: &ParseDecision, evidence_values: &[String]) -> Option<Retrieval> {
    if !context.settings.nlu_rag || !matches!(decision, ParseDecision::Chat | ParseDecision::Reject { .. }) {
        return None;
    }
    let mut entities = Vec::new();
    for entity in &context.home.entities {
        let hit = evidence_values.iter().any(|value| value == &entity.entity_id || value.eq_ignore_ascii_case(&entity.name))
            || context.session.last.iter().any(|turn| turn.entity.as_deref() == Some(entity.entity_id.as_str()));
        if hit && entities.len() < 8 {
            entities.push(RetrievalHit {
                entity_id: entity.entity_id.clone(),
                name: entity.name.clone(),
                domain: entity.domain.clone(),
                area: entity.area.clone(),
            });
        }
    }
    let mut areas: Vec<String> = context
        .home
        .areas
        .iter()
        .filter(|area| evidence_values.iter().any(|value| value == &area.area_id || value.eq_ignore_ascii_case(&area.name)))
        .map(|area| area.name.clone())
        .take(8)
        .collect();
    if areas.is_empty() {
        areas.extend(context.session.last.iter().filter_map(|turn| turn.area.clone()).take(4));
    }
    let last = context.session.last.iter().map(|turn| turn.name.clone()).filter(|name| !name.is_empty()).take(8).collect();
    let custom = context.custom.iter().map(|row| row.phrase.clone()).take(8).collect();
    Some(Retrieval { entities, areas, last, custom, tokens: Vec::new() })
}

pub(super) fn prefer_bonus(rules: &[PolicyRule], entity_id: Option<&str>, area: Option<&str>) -> f64 {
    for rule in rules.iter().filter(|rule| rule.enabled) {
        match rule.effect {
            crate::types::PolicyEffect::PreferEntity if rule.prefer.as_deref() == entity_id => return 0.04,
            crate::types::PolicyEffect::PreferArea if rule.prefer.as_deref() == area => return 0.04,
            _ => {}
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::lang::catalog_for;
    use crate::nlu::context::ParseContext;
    use crate::session::Session;
    use crate::types::{ParseDecision, RejectReason, Settings};

    #[test]
    fn retrieval_stays_off_by_default() {
        let home = default_home();
        let session = Session::new();
        let settings = Settings::default();
        let catalog = catalog_for(&["de".into()]);
        let context = ParseContext::new("hi", &home, &session, &[], &settings, catalog);
        assert!(build(&context, &ParseDecision::Chat, &[]).is_none());
    }

    #[test]
    fn retrieval_only_on_chat_or_reject() {
        let home = default_home();
        let session = Session::new();
        let settings = Settings { nlu_rag: true, ..Settings::default() };
        let catalog = catalog_for(&["de".into()]);
        let context = ParseContext::new("hi", &home, &session, &[], &settings, catalog);
        assert!(build(&context, &ParseDecision::Execute, &[]).is_none());
        assert!(build(&context, &ParseDecision::Reject { reason: RejectReason::NoAction }, &[]).is_some());
    }
}
