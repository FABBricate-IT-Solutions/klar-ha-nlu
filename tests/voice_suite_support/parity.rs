#![allow(dead_code)]

use super::schema::Sentences;
use super::{datasets_root, print_stats, run_groups, run_groups_inherited, run_groups_lang, suite_home, RunStats};
use klar_nlu::lang::LangId;
use klar_nlu::types::HomeGraph;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

const WOHNUNG: &[&str] = &[
    "area",
    "devices",
    "query_area",
    "query_devices",
    "multiple_intents",
    "assist",
    "clarifications",
    "state_persistance",
    "timers",
    "lists",
];

const FAMILY: &[&str] = &[
    "area",
    "devices",
    "query_area",
    "query_devices",
    "multiple_intents",
    "assist",
    "clarifications",
    "state_persistance",
    "timers",
    "lists",
];

#[derive(Debug, Clone, Deserialize, Default)]
struct LocaleRooms {
    #[serde(default)]
    areas: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    floors: BTreeMap<String, Vec<String>>,
}

/// Klar's own apartment suite for one locale: `wohnung_en` for English, `wohnung_mittel`
/// for German, and the `wohnung_mittel` overlay for every generated locale.
pub(crate) fn run_parity_own(lang: &str) -> RunStats {
    let stats = match lang {
        "en" => run_groups("wohnung_en", WOHNUNG, suite_home("wohnung_en")),
        "de" => run_groups("wohnung_mittel", WOHNUNG, suite_home("wohnung_mittel")),
        _ => run_wohnung_overlay(lang, WOHNUNG),
    };
    print_stats(&format!("Klar NLU · parity own {lang}"), &stats);
    stats
}

/// Conversation cases of the own apartment suite. Generated locales are not at zero yet.
pub(crate) fn run_parity_conversation(lang: &str) -> RunStats {
    let stats = conversation(lang);
    print_stats(&format!("Klar NLU · parity conversation {lang}"), &stats);
    stats
}

fn conversation(lang: &str) -> RunStats {
    match lang {
        "en" => run_groups_lang("wohnung_en", &["conversation"], suite_home("wohnung_en"), Some("en-only"), None),
        "de" => run_groups("wohnung_mittel", &["conversation"], suite_home("wohnung_mittel")),
        _ => run_wohnung_overlay(lang, &["conversation"]),
    }
}

/// A regional pack is an overlay on its parent, so its apartment suite is the parent's: the
/// parent's sentences under the regional code. It has no sentence overlay of its own there.
fn run_wohnung_overlay(lang: &str, groups: &[&str]) -> RunStats {
    let parent = LangId::from_code(lang).and_then(LangId::parent).map(LangId::code);
    match parent {
        Some("de") => return run_groups_inherited("wohnung_mittel", groups, suite_home("wohnung_mittel"), lang),
        Some("en") => return run_groups_inherited("wohnung_en", groups, suite_home("wohnung_en"), lang),
        _ => {}
    }
    let sentences = parent.unwrap_or(lang);
    let rooms = load_rooms(sentences);
    let overlay = overlay_for(sentences, "wohnung_mittel");
    run_groups_lang("wohnung_mittel", groups, with_aliases(suite_home("wohnung_mittel"), &rooms), Some(lang), Some(&overlay))
}

pub(crate) fn run_parity(lang: &str) -> RunStats {
    let mut stats = RunStats { ok: 0, fail: 0, fails: Vec::new() };
    let parent = LangId::from_code(lang).and_then(LangId::parent).map(LangId::code);
    if lang == "en" {
        stats.absorb(run_groups("wohnung_en", WOHNUNG, suite_home("wohnung_en")));
        stats.absorb(conversation(lang));
        stats.absorb(run_groups("family_home_en", FAMILY, suite_home("family_home_en")));
        stats.absorb(run_groups("family_home_en", &["m0_exact"], suite_home("family_home_en")));
        stats.absorb(run_groups("family_home_en", &["m2_floors"], suite_home("family_home_en")));
    } else if lang == "de" {
        stats.absorb(run_groups("wohnung_mittel", WOHNUNG, suite_home("wohnung_mittel")));
        stats.absorb(conversation(lang));
        stats.absorb(run_groups("familienhaus_de", FAMILY, suite_home("familienhaus_de")));
        stats.absorb(run_groups("familienhaus_de", &["m0_exact"], suite_home("familienhaus_de")));
        stats.absorb(run_groups("familienhaus_de", &["m2_floors"], suite_home("familienhaus_de")));
    } else if parent == Some("en") {
        // en-GB runs the English suites under its own code; familienhaus overlays on disk still
        // mirror the parent oracle so the derived files stay as gates.
        stats.absorb(run_groups_inherited("wohnung_en", WOHNUNG, suite_home("wohnung_en"), lang));
        stats.absorb(run_groups_inherited("family_home_en", FAMILY, suite_home("family_home_en"), lang));
        stats.absorb(run_groups_inherited("family_home_en", &["m0_exact"], suite_home("family_home_en"), lang));
        stats.absorb(run_groups_inherited("family_home_en", &["m2_floors"], suite_home("family_home_en"), lang));
    } else {
        stats.absorb(run_wohnung_overlay(lang, WOHNUNG));
        // de-AT/de-CH overlays are the German oracle (plus like-words); skip made-up regional
        // room stems that collide (vorzimmer → entryway and hallway). Other regionals reuse the
        // parent locale's room aliases and can reuse the parent's overlay sentences.
        let overlay_lang = match parent {
            Some("de") => lang,
            Some(code) => code,
            None => lang,
        };
        let rooms = if parent == Some("de") { LocaleRooms::default() } else { load_rooms(overlay_lang) };
        let family_home = with_aliases(suite_home("familienhaus_de"), &rooms);
        let family = overlay_for(overlay_lang, "familienhaus_de");
        stats.absorb(run_groups_lang("familienhaus_de", FAMILY, family_home.clone(), Some(lang), Some(&family)));
        let exact = overlay_for(overlay_lang, "m0_exact");
        stats.absorb(run_groups_lang("familienhaus_de", &["m0_exact"], family_home.clone(), Some(lang), Some(&exact)));
        let floors = overlay_for(overlay_lang, "m2_floors");
        stats.absorb(run_groups_lang("familienhaus_de", &["m2_floors"], family_home, Some(lang), Some(&floors)));
    }
    print_stats(&format!("Klar NLU · parity {lang}"), &stats);
    stats
}

fn load_rooms(lang: &str) -> LocaleRooms {
    let path = datasets_root().join("parity/rooms.yaml");
    if !path.exists() {
        return LocaleRooms::default();
    }
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let all: BTreeMap<String, LocaleRooms> = serde_yaml::from_str(&raw).unwrap_or_else(|error| panic!("parity rooms.yaml: {error}"));
    all.get(lang).cloned().unwrap_or_default()
}

fn with_aliases(mut home: HomeGraph, rooms: &LocaleRooms) -> HomeGraph {
    for area in &mut home.areas {
        if let Some(extra) = rooms.areas.get(&area.area_id) {
            for alias in extra {
                if !area.aliases.iter().any(|have| have == alias) {
                    area.aliases.push(alias.clone());
                }
            }
        }
    }
    for floor in &mut home.floors {
        if let Some(extra) = rooms.floors.get(&floor.floor_id) {
            for alias in extra {
                if !floor.aliases.iter().any(|have| have == alias) {
                    floor.aliases.push(alias.clone());
                }
            }
        }
    }
    // Keep the inverted index in sync so protected-number typos (PT quarto≈quatro) stay known.
    klar_nlu::home::rebuild_label_index(&mut home);
    home
}

fn overlay_for(lang: &str, suite: &str) -> BTreeMap<String, Sentences> {
    let mut map = BTreeMap::new();
    let root = datasets_root().join("parity").join(lang).join(suite);
    load_overlay_dir(&root, &mut map);
    map
}

fn load_overlay_dir(dir: &Path, map: &mut BTreeMap<String, Sentences>) {
    if !dir.is_dir() {
        return;
    }
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    files.sort();
    for path in files {
        if path.is_dir() {
            load_overlay_dir(&path, map);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        let parsed: BTreeMap<String, Sentences> =
            serde_yaml::from_str(&raw).unwrap_or_else(|error| panic!("parity overlay {}: {error}", path.display()));
        map.extend(parsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_parity_oracles_cover_required_categories() {
        for suite in ["familienhaus_de"] {
            let cases = super::super::load_cases(&datasets_root().join(suite).join("m0_exact"));
            let labels = cases.iter().map(|(label, _)| label.as_str()).collect::<Vec<_>>();
            for category in ["multi_intent", "timer", "list", "clarify", "reject", "state_persistence"] {
                assert!(labels.iter().any(|label| label.contains(category)), "{suite} lacks exact {category}: {labels:?}");
            }
        }
    }
}
