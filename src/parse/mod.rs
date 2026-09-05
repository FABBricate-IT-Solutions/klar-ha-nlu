#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod action;
pub(crate) mod also;
pub(crate) mod calendar;
pub(crate) mod chat;
pub(crate) mod clause;
pub(crate) mod clause_area;
pub(crate) mod clause_early;
pub(crate) mod clause_session;
pub(crate) mod clause_support;
pub mod compound;
pub(crate) mod fuzzy;
pub(crate) mod infer;
pub(crate) mod media;
pub mod normalize;
pub mod numbers;
pub(crate) mod policy;
pub mod resolve;
pub(crate) mod resolve_named;
pub mod respond;
pub(crate) mod slots;
pub mod split;

use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, ParseResult, Settings};

pub fn match_catalog() -> Vec<crate::types::MatchCatalogRow> {
    policy::PolicyId::catalog_rows()
}

pub fn sanitize_match_controls(rows: Vec<crate::types::MatchControl>) -> Result<Vec<crate::types::MatchControl>, String> {
    let mut seen = std::collections::BTreeSet::new();
    for row in &rows {
        if policy::PolicyId::parse_id(&row.id).is_none() {
            return Err(format!("unknown match id {}", row.id));
        }
        if !seen.insert(row.id.as_str()) {
            return Err(format!("duplicate match id {}", row.id));
        }
    }
    Ok(rows)
}

pub fn match_control_warnings(rows: &[crate::types::MatchControl]) -> Vec<String> {
    rows.iter()
        .filter(|row| !row.enabled)
        .filter(|row| matches!(row.id.as_str(), "area_command" | "all_lights"))
        .map(|row| format!("match.disable.{}", row.id))
        .collect()
}

pub fn parse(text: &str, home: &HomeGraph, session: &mut Session, custom: &[CustomSentence], settings: &Settings) -> ParseResult {
    let mut compatibility = settings.clone();
    compatibility.confirm_risky_actions = false;
    crate::nlu::parse_compatible(text, home, session, custom, &compatibility)
}

#[cfg(test)]
mod tests {
    use super::{match_control_warnings, sanitize_match_controls};
    use crate::types::MatchControl;

    fn row(id: &str, enabled: bool) -> MatchControl {
        MatchControl { id: id.into(), enabled, precedence: None }
    }

    #[test]
    fn unknown_or_duplicate_match_id_is_rejected() {
        assert!(sanitize_match_controls(vec![row("media_new_matcher", true)]).is_err());
        assert!(sanitize_match_controls(vec![row("media", true), row("media", false)]).is_err());
        assert!(sanitize_match_controls(vec![row("media", false), row("timer", true)]).is_ok());
    }

    #[test]
    fn disable_warnings_only_for_area_and_all_lights() {
        assert!(match_control_warnings(&[row("media", false)]).is_empty());
        assert_eq!(
            match_control_warnings(&[row("area_command", false), row("all_lights", false)]),
            ["match.disable.area_command", "match.disable.all_lights"]
        );
    }
}
