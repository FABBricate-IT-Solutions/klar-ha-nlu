#![allow(dead_code)]

use super::schema::Sentences;
use super::{datasets_root, print_stats, run_groups, run_groups_lang, suite_home, RunStats};
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

pub(crate) fn run_parity(lang: &str) -> RunStats {
    let rooms = load_rooms(lang);
    let mut stats = RunStats { ok: 0, fail: 0, fails: Vec::new() };
    if lang == "en" {
        stats.absorb(run_groups("wohnung_en", WOHNUNG, suite_home("wohnung_en")));
        stats.absorb(run_groups_lang("wohnung_en", &["conversation"], suite_home("wohnung_en"), Some("en-only"), None));
        stats.absorb(run_groups("family_home_en", FAMILY, suite_home("family_home_en")));
        stats.absorb(run_groups("family_home_en", &["m0_exact"], suite_home("family_home_en")));
        stats.absorb(run_groups("family_home_en", &["m2_floors"], suite_home("family_home_en")));
    } else if lang == "de" {
        stats.absorb(run_groups("wohnung_mittel", WOHNUNG, suite_home("wohnung_mittel")));
        stats.absorb(run_groups("wohnung_mittel", &["conversation"], suite_home("wohnung_mittel")));
        stats.absorb(run_groups("familienhaus_de", FAMILY, suite_home("familienhaus_de")));
        stats.absorb(run_groups("familienhaus_de", &["m0_exact"], suite_home("familienhaus_de")));
        stats.absorb(run_groups("familienhaus_de", &["m2_floors"], suite_home("familienhaus_de")));
    } else {
        let wohnung = overlay_for(lang, "wohnung_mittel");
        stats.absorb(run_groups_lang(
            "wohnung_mittel",
            WOHNUNG,
            with_aliases(suite_home("wohnung_mittel"), &rooms),
            Some(lang),
            Some(&wohnung),
        ));
        let family_home = with_aliases(suite_home("familienhaus_de"), &rooms);
        let family = overlay_for(lang, "familienhaus_de");
        stats.absorb(run_groups_lang("familienhaus_de", FAMILY, family_home.clone(), Some(lang), Some(&family)));
        let exact = overlay_for(lang, "m0_exact");
        stats.absorb(run_groups_lang("familienhaus_de", &["m0_exact"], family_home.clone(), Some(lang), Some(&exact)));
        let floors = overlay_for(lang, "m2_floors");
        stats.absorb(run_groups_lang("familienhaus_de", &["m2_floors"], family_home, Some(lang), Some(&floors)));
        if std::env::var_os("KLAR_PARITY_REPORT").is_some() {
            stats.absorb(run_groups_lang(
                "wohnung_mittel",
                &["conversation"],
                with_aliases(suite_home("wohnung_mittel"), &rooms),
                Some(lang),
                Some(&wohnung),
            ));
        }
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
