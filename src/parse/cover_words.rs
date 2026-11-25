//! Cover halfway / all-blinds cues and vacuum dock hints (package 2.7).
use crate::lang::{catalog, VerbKind};
use crate::parse::action::has_cover_noun;

/// "Dock", "turn off" or "send it home": Home Assistant's off for a vacuum is return to base.
/// Pause and stop are not dock (P-30) — those are refused as unsupported.
pub(crate) fn dock_hint(tokens: &[String]) -> bool {
    let sent_home = tokens.first().is_some_and(|first| !matches!(catalog().verb(first), Some(VerbKind::Vacuum)));
    catalog().any(tokens, catalog().off_words())
        || tokens.iter().any(|token| {
            matches!(catalog().verb(token), Some(VerbKind::Dock))
                || matches!(token.as_str(), "dock" | "docking" | "station" | "base" | "zurueck" | "ladestation")
                || (token == "home" && sent_home)
        })
}

/// "halfway" / "halb" → cover position 50.
pub(crate) fn cover_halfway(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "halfway" | "halb" | "halbe" | "halbes" | "halben" | "haelfte" | "haelftig"))
        || tokens.windows(2).any(|pair| {
            matches!((pair[0].as_str(), pair[1].as_str()), ("half", "way") | ("zur", "haelfte") | ("auf", "halb") | ("die", "haelfte"))
        })
}

pub(crate) fn cover_halfway_position(tokens: &[String]) -> Option<i32> {
    cover_halfway(tokens).then_some(50)
}

pub(crate) fn wants_all_covers(tokens: &[String]) -> bool {
    has_cover_noun(tokens) && tokens.iter().any(|t| catalog().is_all(t))
}
