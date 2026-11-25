//! Which rooms a sentence names.
//!
//! A room is named by its whole name, word next to word. Where one name lies inside a
//! longer one that was said ("guest" in "guest bathroom"), only the longer room is meant.

use super::specific::area_within;
use super::{token_eq, token_hit};
use crate::home::policy::{is_whole_home, mentions_generic_bedroom, primary_bedroom};
use crate::parse::normalize::{compact, fold_label, umlaut_eq};
use crate::parse::resolve_named::fuzzy_areas;
use crate::types::{AreaRec, HomeGraph};

/// Where a name was said: its first word and how many words it took.
type Span = (usize, usize);

/// One room name found in the sentence.
struct Said {
    area: usize,
    /// Words the name has; a longer name is the stronger match.
    parts: usize,
    span: Span,
}

pub(super) fn match_areas(tokens: &[String], areas: &[AreaRec]) -> Vec<String> {
    let mut said: Vec<Said> = Vec::new();
    for (index, area) in areas.iter().enumerate() {
        let names = [fold_label(&area.name), area.area_id.clone()].into_iter().chain(area.aliases.iter().map(|alias| fold_label(alias)));
        for name in names {
            let parts = name.split([' ', '_']).filter(|part| !part.is_empty()).count().max(1);
            said.extend(spans(tokens, &name).into_iter().map(|span| Said { area: index, parts, span }));
        }
    }
    let mut scored: Vec<(usize, String)> = Vec::new();
    for (index, area) in areas.iter().enumerate() {
        let best = if said.is_empty() {
            // Nothing was said word by word: a name whose words are spread over the sentence counts.
            loose(tokens, area)
        } else {
            said.iter().filter(|hit| hit.area == index && !inside_longer(hit, &said)).map(|hit| hit.parts).max().unwrap_or(0)
        };
        if best > 0 {
            scored.push((best, area.area_id.clone()));
        }
    }
    let max = scored.iter().map(|(s, _)| *s).max().unwrap_or(0);
    let strong: Vec<String> = scored.iter().filter(|(s, _)| *s == max).map(|(_, id)| id.clone()).collect();
    let mut ids: Vec<String> = scored
        .into_iter()
        .filter(|(s, id)| *s == max || !strong.iter().any(|other| other.split('_').next() == id.split('_').next()))
        .filter(|(s, id)| *s == max || !area_within(id, &strong, areas))
        .map(|(_, id)| id)
        .collect();
    if ids.is_empty() {
        if let Some(fuzzy) = fuzzy_areas(tokens, areas) {
            return fuzzy;
        }
    }
    if ids.is_empty() && mentions_generic_bedroom(tokens) {
        if let Some(bedroom) = primary_bedroom(&HomeGraph { areas: areas.to_vec(), ..Default::default() }) {
            return vec![bedroom];
        }
    }
    if ids.len() > 1 {
        ids.retain(|id| !areas.iter().any(|area| area.area_id == *id && is_whole_home(area)));
    }
    ids
}

/// Every place where the label stands in the sentence as a whole.
fn spans(tokens: &[String], label: &str) -> Vec<Span> {
    let parts: Vec<&str> = label.split([' ', '_', '-']).filter(|part| !part.is_empty()).collect();
    let squeezed = compact(label);
    if parts.is_empty() || squeezed.is_empty() {
        return Vec::new();
    }
    let whole = |token: &String| {
        let folded = compact(token);
        token_eq(token, label) || (!folded.is_empty() && (folded == squeezed || umlaut_eq(&folded, &squeezed)))
    };
    let mut found: Vec<Span> = tokens.iter().enumerate().filter(|(_, token)| whole(token)).map(|(at, _)| (at, 1)).collect();
    if parts.len() > 1 {
        let run = |window: &[String]| window.iter().zip(&parts).all(|(token, part)| token_eq(token, part));
        found.extend(tokens.windows(parts.len()).enumerate().filter(|(_, window)| run(window)).map(|(at, _)| (at, parts.len())));
    }
    found
}

/// The name was said as a piece of a longer name of another room.
fn inside_longer(hit: &Said, said: &[Said]) -> bool {
    let (start, len) = hit.span;
    said.iter().any(|other| {
        let (from, width) = other.span;
        other.area != hit.area && width > len && from <= start && start + len <= from + width
    })
}

fn loose(tokens: &[String], area: &AreaRec) -> usize {
    [fold_label(&area.name), area.area_id.clone()]
        .into_iter()
        .chain(area.aliases.iter().map(|alias| fold_label(alias)))
        .filter(|name| token_hit(tokens, name))
        .map(|name| name.split([' ', '_']).filter(|part| !part.is_empty()).count().max(1))
        .max()
        .unwrap_or(0)
}
