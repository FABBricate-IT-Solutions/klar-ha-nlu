//! Area-conjunction labels ("Wohn und Esszimmer") and short room stems ("wohn" → Wohnzimmer).

use super::token_eq;
use crate::home::policy::is_whole_home;
use crate::lang::catalog;
use crate::parse::normalize::{compact, fold_label};
use crate::types::{EntityRec, HomeGraph};

/// Label shaped like "Wohn und Esszimmer" / "living and dining": two area stems with a
/// conjunction, not a single device name.
pub(crate) fn area_conjunction_entity(entity: &EntityRec, home: &HomeGraph) -> bool {
    std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str)).any(|label| {
        let parts: Vec<String> = fold_label(label).split([' ', '_', '-']).filter(|part| !part.is_empty()).map(str::to_string).collect();
        area_conjunction_label(&parts, home)
    })
}

pub(super) fn area_conjunction_label(parts: &[String], home: &HomeGraph) -> bool {
    if parts.len() < 3 {
        return false;
    }
    let cat = catalog();
    let Some(conj_at) = parts.iter().position(|part| cat.is_conj(part)) else {
        return false;
    };
    let left = &parts[..conj_at];
    let right = &parts[conj_at + 1..];
    !left.is_empty() && !right.is_empty() && area_id_for_stem(left, home).is_some() && area_id_for_stem(right, home).is_some()
}

/// Map a "Wohn und Esszimmer" group label onto the two room ids it names.
pub(super) fn rooms_from_conjunction_entities(entities: &[EntityRec], home: &HomeGraph) -> Vec<String> {
    let mut rooms = Vec::new();
    for entity in entities {
        for label in std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str)) {
            let parts: Vec<String> = fold_label(label).split([' ', '_', '-']).filter(|part| !part.is_empty()).map(str::to_string).collect();
            if !area_conjunction_label(&parts, home) {
                continue;
            }
            let cat = catalog();
            let Some(conj_at) = parts.iter().position(|part| cat.is_conj(part)) else {
                continue;
            };
            for side in [&parts[..conj_at], &parts[conj_at + 1..]] {
                if let Some(area_id) = area_id_for_stem(side, home) {
                    if !rooms.contains(&area_id) {
                        rooms.push(area_id);
                    }
                }
            }
        }
    }
    rooms
}

fn area_id_for_stem(parts: &[String], home: &HomeGraph) -> Option<String> {
    let joined = parts.join(" ");
    let squeezed = compact(&joined);
    let mut best: Option<(usize, String)> = None;
    for area in &home.areas {
        if is_whole_home(area) {
            continue;
        }
        let hit = std::iter::once(area.area_id.as_str())
            .chain(std::iter::once(area.name.as_str()))
            .chain(area.aliases.iter().map(String::as_str))
            .any(|name| {
                let folded = fold_label(name);
                let name_compact = compact(&folded);
                token_eq(&joined, &folded)
                    || name_compact == squeezed
                    || folded
                        .split([' ', '_', '-'])
                        .filter(|part| !part.is_empty())
                        .any(|part| parts.iter().any(|spoken| token_eq(spoken, part)))
                    || parts.iter().any(|spoken| stem_names_area(&compact(spoken), &name_compact))
            });
        if !hit {
            continue;
        }
        let score = compact(&fold_label(&area.name)).len().max(area.area_id.len());
        if best.as_ref().is_none_or(|(have, _)| score > *have) {
            best = Some((score, area.area_id.clone()));
        }
    }
    best.map(|(_, id)| id)
}

/// "wohn" → Wohnzimmer, not Wohnung; require a room suffix after a short stem.
fn stem_names_area(stem: &str, name_compact: &str) -> bool {
    if stem.len() < 3 {
        return false;
    }
    if name_compact == stem {
        return true;
    }
    if !name_compact.starts_with(stem) {
        return stem.starts_with(name_compact) && name_compact.len() >= 3;
    }
    let rest = &name_compact[stem.len()..];
    rest.is_empty()
        || rest.starts_with("zimmer")
        || rest.starts_with("raum")
        || rest.starts_with("room")
        || rest.starts_with("kitchen")
        || rest.starts_with("bath")
}
