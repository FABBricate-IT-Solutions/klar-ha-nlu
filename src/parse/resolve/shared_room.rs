//! One spoken room name can match the same room on several floors.
//! Prefer the floor that heard the sentence. Otherwise ask.
//! `Küche` and the Home Assistant slug `kuche` are the same stem.

use super::token_hit;
use crate::parse::normalize::{compact, fold_label, umlaut_eq};
use crate::parse::resolve::Resolved;
use crate::types::{AreaRec, HomeGraph};
use std::collections::BTreeSet;

pub(crate) fn narrow_shared_rooms(resolved: &mut Resolved, tokens: &[String], home: &HomeGraph, preferred_area: Option<&str>) {
    if resolved.areas.len() < 2 {
        return;
    }
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for id in &resolved.areas {
        let Some(area) = home.areas.iter().find(|area| area.area_id == *id) else {
            continue;
        };
        let Some(stem) = matched_stem(tokens, area) else {
            continue;
        };
        if let Some((_, ids)) = groups.iter_mut().find(|(key, _)| key == &stem || umlaut_eq(key, &stem)) {
            ids.push(id.clone());
        } else {
            groups.push((stem, vec![id.clone()]));
        }
    }
    let crowded: Vec<Vec<String>> = groups.into_iter().map(|(_, ids)| ids).filter(|ids| ids.len() >= 2).collect();
    if crowded.is_empty() {
        return;
    }
    let heard = preferred_floor(home, preferred_area);
    let mut drop = BTreeSet::new();
    let mut ask = Vec::new();
    for ids in crowded {
        match choose(&ids, home, &resolved.floors, heard) {
            Choice::Keep(id) => {
                for other in ids {
                    if other != id {
                        drop.insert(other);
                    }
                }
            }
            Choice::Ask(ids) => {
                drop.extend(ids.iter().cloned());
                ask.extend(ids);
            }
        }
    }
    if drop.is_empty() && ask.is_empty() {
        return;
    }
    resolved.areas.retain(|id| !drop.contains(id));
    resolved.ask_areas = ask;
}

enum Choice {
    Keep(String),
    Ask(Vec<String>),
}

fn choose(ids: &[String], home: &HomeGraph, named_floors: &[String], heard: Option<&str>) -> Choice {
    if !named_floors.is_empty() {
        let on_floor = on_floors(ids, home, named_floors);
        if let Some(choice) = one_or_ask(&on_floor) {
            return choice;
        }
    }
    if let Some(floor) = heard {
        let on_heard = on_floors(ids, home, &[floor.to_string()]);
        if let Some(choice) = one_or_ask(&on_heard) {
            return choice;
        }
    }
    Choice::Ask(ids.to_vec())
}

fn one_or_ask(ids: &[String]) -> Option<Choice> {
    match ids {
        [] => None,
        [one] => Some(Choice::Keep(one.clone())),
        _ => Some(Choice::Ask(ids.to_vec())),
    }
}

fn on_floors(ids: &[String], home: &HomeGraph, floors: &[String]) -> Vec<String> {
    ids.iter()
        .filter(|id| {
            home.areas.iter().any(|area| area.area_id == **id && area.floor_id.as_ref().is_some_and(|floor| floors.contains(floor)))
        })
        .cloned()
        .collect()
}

fn preferred_floor<'a>(home: &'a HomeGraph, preferred_area: Option<&str>) -> Option<&'a str> {
    let area_id = preferred_area?;
    home.areas.iter().find(|area| area.area_id == area_id)?.floor_id.as_deref()
}

fn matched_stem(tokens: &[String], area: &AreaRec) -> Option<String> {
    let labels = std::iter::once(fold_label(&area.name))
        .chain(std::iter::once(fold_label(&area.area_id)))
        .chain(area.aliases.iter().map(|alias| fold_label(alias)));
    let mut best: Option<(usize, String)> = None;
    for label in labels {
        if label.is_empty() || !token_hit(tokens, &label) {
            continue;
        }
        let parts = label.split([' ', '_', '-']).filter(|part| !part.is_empty()).count().max(1);
        if best.as_ref().is_none_or(|(score, _)| parts > *score) {
            best = Some((parts, compact(&label)));
        }
    }
    best.map(|(_, stem)| stem)
}
