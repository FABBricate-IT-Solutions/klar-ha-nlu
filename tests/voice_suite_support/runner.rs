use super::expect::{exact_result_ok, failure_kind, record_failure};
use super::legacy;
use super::schema::{Case, Sentences};
use super::world::TestWorld;
use super::{load_cases, RunStats};
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Settings};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[allow(dead_code)]
pub(crate) fn run_suite(name: &str, extended: bool) -> RunStats {
    let mut groups = vec!["area", "devices", "query_area", "query_devices", "multiple_intents", "assist"];
    if extended {
        groups.extend(["clarifications", "state_persistance", "timers", "lists"]);
    }
    run_groups(name, &groups, super::suite_home(name))
}

fn suite_language(name: &str) -> Option<&'static str> {
    if name.ends_with("_en") || name.contains("_en_") {
        return Some("en");
    }
    if name.ends_with("_de") || name.contains("_de_") || name.starts_with("wohnung") {
        return Some("de");
    }
    None
}

pub(crate) fn run_groups(name: &str, groups: &[&str], home: HomeGraph) -> RunStats {
    run_groups_lang(name, groups, home, suite_language(name), None)
}

pub(crate) fn run_groups_lang(
    name: &str,
    groups: &[&str],
    home: HomeGraph,
    language: Option<&str>,
    overlay: Option<&BTreeMap<String, super::schema::Sentences>>,
) -> RunStats {
    let settings = match language {
        Some("de") | Some("en") => Settings { languages: vec!["de".into(), "en".into()], ..Settings::default() },
        Some("en-only") => Settings::pinned("en"),
        Some(code) => Settings::pinned(code),
        None => Settings { languages: vec!["de".into(), "en".into()], ..Settings::default() },
    };
    let root = super::datasets_root().join(name);
    let mut stats = RunStats { ok: 0, fail: 0, fails: Vec::new() };
    for group in groups {
        for (label, mut case) in load_cases(&root.join(group)) {
            if let Some(map) = overlay {
                match map.get(&format!("{group}/{label}")) {
                    Some(sentences) => {
                        case.sentences = sentences.clone();
                        case.speech_has.clear();
                        case.speech_forbids.clear();
                    }
                    None if language.is_some_and(|code| code != "de" && code != "en") => {
                        record_failure(&mut stats, group, &label, &[], format!("missing parity sentence for {language:?}"));
                        continue;
                    }
                    None => {}
                }
            }
            let legacy_clarify = *group == "clarifications";
            for turns in turns_of(&case.sentences) {
                run_case(group, &label, &case, &turns, legacy_clarify, &home, &settings, &mut stats);
            }
        }
    }
    stats
}

#[allow(clippy::too_many_arguments)]
fn run_case(
    group: &str,
    label: &str,
    case: &Case,
    turns: &[String],
    legacy_clarify: bool,
    home: &HomeGraph,
    settings: &Settings,
    stats: &mut RunStats,
) {
    let mut world = TestWorld::from_setup(&case.setup).expect("setup validated at load");
    let mut session = Session::new();
    let mut last = None;
    for (index, sentence) in turns.iter().enumerate() {
        let result = parse(sentence, home, &mut session, &[], settings);
        if let Err(error) = world.apply_intents(&result.intents, home, !case.world_expect.is_empty()) {
            record_failure(stats, group, label, turns, error);
            return;
        }
        last = Some(result.clone());
        if legacy_clarify && index + 1 < turns.len() && !result.clarify {
            record_failure(stats, group, label, turns, format!("first turn should clarify: {sentence:?} → {:?}", result.intents));
            return;
        }
    }
    let Some(result) = last else {
        record_failure(stats, group, label, turns, "case has no turns".into());
        return;
    };
    if result.clarify && !legacy_clarify && case.nlu_expect.as_ref().and_then(|expected| expected.clarify) != Some(true) {
        record_failure(stats, group, label, turns, format!("unexpected clarify: {}", result.speech));
        return;
    }
    let checked = match &case.nlu_expect {
        Some(expected) => exact_result_ok(expected, &result),
        None => legacy::result_ok(case, &result.intents, legacy_clarify, home),
    }
    .and_then(|()| world.assert_records(&case.world_expect))
    .and_then(|()| legacy::forbid_ok(&result.intents, &case.forbid))
    .and_then(|()| legacy::speech_ok(&result.speech, &case.speech_has, &case.speech_forbids));
    match checked {
        Ok(()) => stats.ok += 1,
        Err(error) => record_failure(stats, group, label, turns, error),
    }
}

pub(crate) fn print_stats(title: &str, stats: &RunStats) {
    let total = stats.ok + stats.fail;
    let pct = if total == 0 { 0.0 } else { 100.0 * stats.ok as f64 / total as f64 };
    println!("\n=== {title} ===");
    println!("  {total} Sätze  {} ok  {} fehl  {pct:.1}%", stats.ok, stats.fail);
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for line in &stats.fails {
        *kinds.entry(failure_kind(line).into()).or_default() += 1;
    }
    println!("  fail-kinds {kinds:?}");
    for line in stats.fails.iter().take(25) {
        println!("  FAIL {line}");
    }
    if stats.fails.len() > 25 {
        println!("  … {} weitere", stats.fails.len() - 25);
    }
    write_failures(title, stats);
}

fn write_failures(title: &str, stats: &RunStats) {
    let file_name = title.split('(').nth(1).unwrap_or("x").trim_end_matches(')').replace([' ', '·'], "_");
    let dump = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join(format!("suite_fails_{file_name}.txt"));
    let _ = std::fs::create_dir_all(dump.parent().expect("target parent"));
    let _ = std::fs::write(dump, stats.fails.join("\n"));
}

fn turns_of(sentences: &Sentences) -> Vec<Vec<String>> {
    match sentences {
        Sentences::Turns(turns) => turns.clone(),
        Sentences::Flat(sentences) => sentences.iter().map(|sentence| vec![sentence.clone()]).collect(),
    }
}
