//! When several rooms or devices match, prefer the one the user most specifically named.
use super::token_eq;
use crate::lang::catalog;
use crate::parse::normalize::fold_label;
use crate::types::{AreaRec, EntityRec};

fn words(name: &str) -> Vec<String> {
    fold_label(name).split([' ', '_', '-']).filter(|word| !word.is_empty()).map(str::to_string).collect()
}

fn all_said(tokens: &[String], words: &[String]) -> bool {
    !words.is_empty() && words.iter().all(|word| tokens.iter().any(|token| token_eq(token, word)))
}

fn contains_all(longer: &[String], shorter: &[String]) -> bool {
    longer.len() > shorter.len() && shorter.iter().all(|word| longer.contains(word))
}

fn fully_named(tokens: &[String], entity: &EntityRec) -> bool {
    all_said(tokens, &words(&entity.name))
}

/// With a room named, a device in another room only counts when its whole name was said;
/// devices without a room (lists, whole-home heating) always stay.
pub(crate) fn narrow_to_scope(tokens: &[String], scope: &[String], candidates: &mut Vec<(f64, EntityRec)>) {
    if scope.is_empty() {
        return;
    }
    let in_scope = |e: &EntityRec| e.area.as_ref().is_some_and(|area| scope.contains(area));
    if candidates.iter().any(|(_, e)| in_scope(e)) {
        candidates.retain(|(score, e)| in_scope(e) || (*score >= 0.96 && fully_named(tokens, e)));
    } else {
        candidates.retain(|(_, e)| e.area.is_none() || fully_named(tokens, e));
    }
}

/// Keeps the side the user said ("left floor light") and drops a name contained in a longer
/// one that was said in full ("Office Light" inside "Anna's Office Light").
pub(crate) fn keep_most_specific(tokens: &[String], mut found: Vec<EntityRec>) -> Vec<EntityRec> {
    if found.len() < 2 {
        return found;
    }
    let cat = catalog();
    for side in [cat.left(), cat.right()] {
        if cat.any(tokens, side) {
            let sided: Vec<EntityRec> =
                found.iter().filter(|entity| words(&entity.name).iter().any(|word| side.contains(word.as_str()))).cloned().collect();
            if !sided.is_empty() {
                found = sided;
            }
        }
    }
    let shadowed = |entity: &EntityRec| {
        let mine = words(&entity.name);
        found.iter().any(|other| {
            let theirs = words(&other.name);
            other.entity_id != entity.entity_id && contains_all(&theirs, &mine) && all_said(tokens, &theirs)
        })
    };
    found.iter().filter(|entity| !shadowed(entity)).cloned().collect()
}

/// A weaker room match whose words all sit inside a stronger room's name ("Office" in
/// "Anna's Office") only matched on the shared words.
pub(crate) fn area_within(id: &str, strong: &[String], areas: &[AreaRec]) -> bool {
    let words_of = |id: &str| areas.iter().find(|area| area.area_id == id).map(|area| words(&area.name)).unwrap_or_default();
    let mine = words_of(id);
    !mine.is_empty() && strong.iter().any(|other| contains_all(&words_of(other), &mine))
}

/// "Kaffeemaschine" next to "Kaffeemaschine LED": the only device whose whole name was said wins
/// instead of asking which.
pub(crate) fn prefer_fully_named(tokens: &[String], entities: &mut Vec<EntityRec>, ambiguous: &mut Vec<EntityRec>) {
    if !entities.is_empty() || ambiguous.len() < 2 {
        return;
    }
    let fully: Vec<EntityRec> = ambiguous.iter().filter(|entity| fully_named(tokens, entity)).cloned().collect();
    if fully.len() == 1 {
        *entities = fully;
        ambiguous.clear();
    }
}
