mod legacy;
mod schema;
mod waivers;
mod world;

use self::schema::{Case, ExpectedIntent, NluExpectation, Sentences};
use self::world::TestWorld;
use klar_nlu::home::{default_home, load_home_config};
use klar_nlu::parse::parse_checked;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Intent, ParseResult, Settings};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) struct RunStats {
    pub(crate) ok: usize,
    pub(crate) fail: usize,
    pub(crate) waived: usize,
    pub(crate) fails: Vec<String>,
    pub(crate) waivers: Vec<String>,
    used_waivers: BTreeSet<&'static str>,
}

pub(crate) fn suite_home(name: &str) -> HomeGraph {
    let path = datasets_root().join(name).join("home_config.yaml");
    if path.exists() {
        load_home_config(&path).unwrap_or_else(|error| panic!("home_config {}: {error}", path.display()))
    } else {
        default_home()
    }
}

pub(crate) fn run_suite(name: &str, extended: bool) -> RunStats {
    let mut groups = vec!["area", "devices", "query_area", "query_devices", "multiple_intents", "assist"];
    if extended {
        groups.extend(["clarifications", "state_persistance", "timers", "lists"]);
    }
    run_groups(name, &groups, suite_home(name))
}

pub(crate) fn run_groups(name: &str, groups: &[&str], home: HomeGraph) -> RunStats {
    let settings = Settings::default();
    let root = datasets_root().join(name);
    let mut stats = RunStats { ok: 0, fail: 0, waived: 0, fails: Vec::new(), waivers: Vec::new(), used_waivers: BTreeSet::new() };
    for group in groups {
        for (label, case) in load_cases(&root.join(group)) {
            let legacy_clarify = *group == "clarifications";
            for turns in turns_of(&case.sentences) {
                run_case(name, group, &label, &case, &turns, legacy_clarify, &home, &settings, &mut stats);
            }
        }
    }
    for waiver in waivers::expected_for(name, groups) {
        if !stats.used_waivers.contains(waiver.id) {
            stats.fail += 1;
            stats.fails.push(format!("stale waiver {} for {}/{}/{}", waiver.id, waiver.suite, waiver.group, waiver.label));
        }
    }
    stats
}

#[allow(clippy::too_many_arguments)]
fn run_case(
    suite: &str,
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
        let (result, parity_error) = parse_checked(sentence, home, &mut session, &[], settings);
        if let Some(error) = parity_error {
            record_waiver_or_failure(stats, suite, group, label, turns, error);
            return;
        }
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
        Err(error) => record_waiver_or_failure(stats, suite, group, label, turns, error),
    }
}

pub(crate) fn print_stats(title: &str, stats: &RunStats) {
    let total = stats.ok + stats.fail;
    let pct = if total == 0 { 0.0 } else { 100.0 * stats.ok as f64 / total as f64 };
    println!("\n=== {title} ===");
    println!("  {total} Sätze  {} ok  {} fehl  {pct:.1}%", stats.ok, stats.fail);
    println!("  {} explizit freigegebene Legacy-Waiver", stats.waived);
    println!("  V1/V2 output and session parity checked inline");
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for line in &stats.fails {
        *kinds.entry(failure_kind(line).into()).or_default() += 1;
    }
    println!("  fail-kinds {kinds:?}");
    for line in stats.fails.iter().take(25) {
        println!("  FAIL {line}");
    }
    for line in &stats.waivers {
        println!("  WAIVER {line}");
    }
    if stats.fails.len() > 25 {
        println!("  … {} weitere", stats.fails.len() - 25);
    }
    write_failures(title, stats);
}

fn datasets_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/datasets")
}

fn load_cases(dir: &Path) -> Vec<(String, Case)> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read suite directory {}: {error}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("yaml"))
        .collect();
    files.sort();
    files.into_iter().flat_map(load_case_file).collect()
}

fn load_case_file(path: PathBuf) -> Vec<(String, Case)> {
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read suite yaml {}: {error}", path.display()));
    let parsed: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap_or_else(|error| panic!("suite yaml {}: {error}", path.display()));
    let cases = if parsed.as_sequence().is_some() {
        serde_yaml::from_value::<Vec<Case>>(parsed)
    } else {
        serde_yaml::from_value::<Case>(parsed).map(|case| vec![case])
    }
    .unwrap_or_else(|error| panic!("suite yaml {}: {error}", path.display()));
    let stem = path.file_stem().expect("yaml stem").to_string_lossy();
    cases
        .into_iter()
        .map(|case| {
            let label = format!("{stem}::{}", case.name);
            validate_case_or_panic(&case, &format!("{}::{label}", path.display()));
            (label, case)
        })
        .collect()
}

fn validate_case_or_panic(case: &Case, label: &str) {
    case.validate_schema().unwrap_or_else(|error| panic!("{label}: invalid case schema: {error}"));
    TestWorld::from_setup(&case.setup).unwrap_or_else(|error| panic!("{label}: invalid setup: {error}"));
    TestWorld::validate_expectations(&case.world_expect).unwrap_or_else(|error| panic!("{label}: invalid world expectation: {error}"));
}

fn turns_of(sentences: &Sentences) -> Vec<Vec<String>> {
    match sentences {
        Sentences::Turns(turns) => turns.clone(),
        Sentences::Flat(sentences) => sentences.iter().map(|sentence| vec![sentence.clone()]).collect(),
    }
}

fn exact_result_ok(expected: &NluExpectation, result: &ParseResult) -> Result<(), String> {
    if let Some(intents) = &expected.intents {
        exact_intents_ok(intents, &result.intents)?;
    }
    if let Some(reject) = expected.reject {
        let actual_reject = result.intents.is_empty() && !result.clarify && !result.chat;
        if actual_reject != reject {
            return Err(format!(
                "expected reject={reject}, got intents={:?} clarify={} chat={}",
                result.intents, result.clarify, result.chat
            ));
        }
    }
    if let Some(clarify) = expected.clarify {
        if result.clarify != clarify {
            return Err(format!("expected clarify={clarify}, got clarify={} intents={:?}", result.clarify, result.intents));
        }
    }
    Ok(())
}

fn exact_intents_ok(expected: &[ExpectedIntent], actual: &[Intent]) -> Result<(), String> {
    if expected.len() != actual.len() {
        return Err(format!("exact intents: expected {} in declared order, got {}: {actual:?}", expected.len(), actual.len()));
    }
    for (index, (wanted, got)) in expected.iter().zip(actual).enumerate() {
        if wanted.intent != got.name {
            return Err(format!("exact intent[{index}]: expected {}, got {}", wanted.intent, got.name));
        }
        exact_slots_ok(index, wanted, got)?;
    }
    Ok(())
}

fn exact_slots_ok(index: usize, expected: &ExpectedIntent, actual: &Intent) -> Result<(), String> {
    let expected_slots = expected
        .slots
        .iter()
        .map(|(name, value)| scalar(value).map(|value| (name.clone(), value)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut actual_slots = BTreeMap::new();
    for slot in &actual.slots {
        if actual_slots.insert(slot.name.clone(), slot.value.clone()).is_some() {
            return Err(format!("exact intent[{index}] {} has duplicate slot {}", actual.name, slot.name));
        }
    }
    if expected_slots != actual_slots {
        return Err(format!("exact intent[{index}] {} slots: expected {expected_slots:?}, got {actual_slots:?}", actual.name));
    }
    Ok(())
}

fn scalar(value: &serde_yaml::Value) -> Result<String, String> {
    match value {
        serde_yaml::Value::Null => Ok("null".into()),
        serde_yaml::Value::Bool(value) => Ok(value.to_string()),
        serde_yaml::Value::Number(value) => Ok(value.to_string()),
        serde_yaml::Value::String(value) => Ok(value.clone()),
        _ => Err(format!("expected a scalar value, got {value:?}")),
    }
}

fn write_failures(title: &str, stats: &RunStats) {
    let file_name = title.split('(').nth(1).unwrap_or("x").trim_end_matches(')').replace([' ', '·'], "_");
    let dump = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join(format!("suite_fails_{file_name}.txt"));
    let _ = std::fs::create_dir_all(dump.parent().expect("target parent"));
    let _ = std::fs::write(dump, stats.fails.join("\n"));
}

fn record_waiver_or_failure(stats: &mut RunStats, suite: &str, group: &str, label: &str, turns: &[String], error: String) {
    let kind = mismatch_kind(&error);
    let fingerprint = mismatch_fingerprint(&error);
    match waivers::matching(suite, group, label, kind, fingerprint) {
        Some(waiver) => {
            stats.waived += 1;
            stats.used_waivers.insert(waiver.id);
            stats
                .waivers
                .push(format!("{} {group}/{label}: {turns:?} — kind={kind} fingerprint={fingerprint:016x} — {}", waiver.id, waiver.reason));
        }
        None => {
            let expected = waivers::for_case(suite, group, label)
                .map(|waiver| format!("{}={}:{:016x}", waiver.id, waiver.kind, waiver.fingerprint))
                .collect::<Vec<_>>();
            let detail = if expected.is_empty() { "no waiver".into() } else { format!("expected {}", expected.join(",")) };
            record_failure(
                stats,
                group,
                label,
                turns,
                format!("unwaived mismatch kind={kind} fingerprint={fingerprint:016x} ({detail}): {error}"),
            );
        }
    }
}

fn mismatch_kind(error: &str) -> &'static str {
    if error.starts_with("V1/V2 parity mismatch") {
        "parity"
    } else {
        "oracle"
    }
}

fn mismatch_fingerprint(error: &str) -> u64 {
    let normalized = normalize_mismatch(error);
    normalized.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3))
}

fn normalize_mismatch(error: &str) -> String {
    if error.starts_with("V1/V2 parity mismatch") {
        let legacy = parse_result_intents(error, "legacy=ParseResult").unwrap_or("missing");
        let current = parse_result_intents(error, "current=ParseResult").unwrap_or("missing");
        return format!("legacy={legacy}|current={current}");
    }
    let intent_names = error.split("Intent { name: \"").skip(1).filter_map(|tail| tail.split('"').next()).collect::<Vec<_>>().join(",");
    if !intent_names.is_empty() {
        return format!("oracle:{}:{intent_names}", failure_kind(error));
    }
    let mut normalized = String::with_capacity(error.len());
    let mut index = 0;
    let bytes = error.as_bytes();
    while index < bytes.len() {
        if error.get(index..index + 36).is_some_and(uuid_like) {
            normalized.push_str("<conversation-id>");
            index += 36;
        } else {
            let character = error[index..].chars().next().expect("valid string boundary");
            if !character.is_whitespace() || !normalized.ends_with(' ') {
                normalized.push(if character.is_whitespace() { ' ' } else { character });
            }
            index += character.len_utf8();
        }
    }
    normalized
}

fn parse_result_intents<'a>(error: &'a str, marker: &str) -> Option<&'a str> {
    let result = error.split_once(marker)?.1;
    let intents = result.split_once("intents: [")?.1;
    intents.split_once("], speech:").map(|(value, _)| value)
}

fn uuid_like(value: &str) -> bool {
    value.char_indices().all(|(index, character)| {
        matches!(index, 8 | 13 | 18 | 23) && character == '-' || !matches!(index, 8 | 13 | 18 | 23) && character.is_ascii_hexdigit()
    })
}

fn record_failure(stats: &mut RunStats, group: &str, label: &str, turns: &[String], error: String) {
    stats.fail += 1;
    stats.fails.push(format!("{group}/{label}: {turns:?} → {error}"));
}

fn failure_kind(line: &str) -> &'static str {
    for (needle, kind) in [
        ("exact intent", "exact"),
        ("world_expect", "world"),
        ("reject=", "reject"),
        ("clarify", "clarify"),
        ("HassSetPosition", "position"),
        ("HassFanSetSpeed", "fan"),
        ("HassLightSet", "lightset"),
        ("HassClimate", "climate"),
        ("HassGetState", "query"),
        ("HassTurnOff", "off"),
        ("HassTurnOn", "on"),
        ("Timer", "timer"),
        ("Shopping", "list"),
        ("todo", "list"),
    ] {
        if line.contains(needle) {
            return kind;
        }
    }
    "other"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_builds_world_without_inventing_conversation_history() {
        let case: Case = serde_yaml::from_str(
            "setup: [{entity_id: light.first, state: off, brightness: 40}]\n\
             sentences: [off]\nconditions: [{entity_id: light.first, state: off}]",
        )
        .expect("case");
        let world = TestWorld::from_setup(&case.setup).expect("world");
        world.assert_records(&case.setup).expect("preserved setup");
        let session = Session::new();
        assert_eq!(session.last_entities().count(), 0);
    }

    #[test]
    fn world_applies_state_transition_and_attributes() {
        let setup: Vec<schema::StateRecord> =
            serde_yaml::from_str("- {entity_id: light.entryway_light, state: off, brightness: 60}").expect("setup");
        let expected: Vec<schema::StateRecord> =
            serde_yaml::from_str("- {entity_id: light.entryway_light, brightness: 12}").expect("expected");
        let mut world = TestWorld::from_setup(&setup).expect("world");
        world
            .apply_intents(
                &[Intent::new("HassLightSet").with("entity_id", "light.entryway_light").with("brightness", "12")],
                &HomeGraph::default(),
                true,
            )
            .expect("supported transition");
        world.assert_records(&expected).expect("transition");
    }

    #[test]
    fn world_simulates_required_m0_transitions() {
        let setup: Vec<schema::StateRecord> = serde_yaml::from_str(
            "- {entity_id: switch.pump, state: on}\n\
             - {entity_id: media_player.kitchen, volume_level: 50, is_volume_muted: false}\n\
             - {entity_id: timer.oven, state: paused, minutes: 30}\n\
             - {list_name: todo.chores, todo_item: laundry}\n\
             - {shopping_list_item: milk}\n\
             - {shopping_list_item: eggs}",
        )
        .expect("setup");
        let mut world = TestWorld::from_setup(&setup).expect("world");
        world
            .apply_intents(
                &[
                    Intent::new("HassToggle").with("entity_id", "switch.pump"),
                    Intent::new("HassMediaPlayerMute").with("entity_id", "media_player.kitchen"),
                    Intent::new("HassSetVolumeRelative").with("entity_id", "media_player.kitchen").with("volume_step", "up"),
                    Intent::new("HassStartTimer").with("entity_id", "timer.oven"),
                    Intent::new("HassListCompleteItem").with("entity_id", "todo.chores").with("item", "laundry"),
                    Intent::new("HassShoppingListCompleteItem").with("item", "milk"),
                    Intent::new("HassListCompleteItem").with("name", "shopping_list").with("item", "eggs"),
                ],
                &HomeGraph::default(),
                true,
            )
            .expect("supported transitions");
        let expected: Vec<schema::StateRecord> = serde_yaml::from_str(
            "- {entity_id: switch.pump, state: off}\n\
             - {entity_id: media_player.kitchen, is_volume_muted: true, volume_level: 50, volume_step: up}\n\
             - {entity_id: timer.oven, state: active, minutes: 30}\n\
             - {list_name: todo.chores, todo_completed_item: laundry}\n\
             - {shopping_list_completed_item: milk}\n\
             - {shopping_list_completed_item: eggs}",
        )
        .expect("expected");
        world.assert_records(&expected).expect("transitions");
        world
            .apply_intents(&[Intent::new("HassMediaPlayerUnmute").with("entity_id", "media_player.kitchen")], &HomeGraph::default(), true)
            .expect("unmute");
        let unmuted: Vec<schema::StateRecord> =
            serde_yaml::from_str("- {entity_id: media_player.kitchen, is_volume_muted: false}").expect("unmuted");
        world.assert_records(&unmuted).expect("unmuted transition");
    }

    #[test]
    fn unsupported_world_transition_is_an_error() {
        let mut world = TestWorld::default();
        assert!(world
            .apply_intents(&[Intent::new("HassVacuumStart").with("entity_id", "vacuum.downstairs")], &HomeGraph::default(), true,)
            .is_err());
    }

    #[test]
    fn strict_completion_requires_setup_item() {
        for intent in [
            Intent::new("HassShoppingListCompleteItem").with("item", "missing"),
            Intent::new("HassListCompleteItem").with("name", "shopping_list").with("item", "missing"),
            Intent::new("HassListCompleteItem").with("entity_id", "todo.chores").with("item", "missing"),
        ] {
            let mut world = TestWorld::default();
            let error = world.apply_intents(&[intent], &HomeGraph::default(), true).expect_err("completion without setup must fail");
            assert!(error.contains("missing"), "{error}");
        }
    }

    #[test]
    fn mixed_legacy_and_exact_schema_panics_during_validation() {
        let case: Case = serde_yaml::from_str(
            "sentences: [on]\nconditions: [{state: on}]\n\
             nlu_expect: {intents: [{intent: HassTurnOn}]}",
        )
        .expect("case");
        assert!(std::panic::catch_unwind(|| validate_case_or_panic(&case, "mixed")).is_err());
    }

    #[test]
    fn missing_or_empty_oracle_panics_during_validation() {
        for yaml in [
            "sentences: [on]",
            "sentences: [on]\nconditions: []",
            "sentences: [on]\nconditions: []\nworld_expect: [{entity_id: light.a, state: on}]",
        ] {
            let case: Case = serde_yaml::from_str(yaml).expect("case");
            assert!(std::panic::catch_unwind(|| validate_case_or_panic(&case, "oracle")).is_err());
        }
    }

    #[test]
    fn invalid_setup_panics_during_validation() {
        for yaml in [
            "sentences: [on]\nsetup: [{state: on}]\nconditions: [{state: on}]",
            "sentences: [on]\nsetup: [{entity_id: light.a, mystery: x}]\nconditions: [{state: on}]",
        ] {
            let case: Case = serde_yaml::from_str(yaml).expect("case");
            assert!(std::panic::catch_unwind(|| validate_case_or_panic(&case, "invalid")).is_err());
        }
    }

    #[test]
    fn removed_expected_alias_is_rejected() {
        let parsed = serde_yaml::from_str::<Case>("sentences: [on]\nexpected: {intents: [{intent: HassTurnOn}]}");
        assert!(parsed.is_err());
    }

    #[test]
    fn checked_in_exact_gates_cover_required_categories_in_both_languages() {
        for suite in ["family_home_en", "familienhaus_de"] {
            let cases = load_cases(&datasets_root().join(suite).join("m0_exact"));
            let labels = cases.iter().map(|(label, _)| label.as_str()).collect::<Vec<_>>();
            for category in ["multi_intent", "timer", "list", "clarify", "reject", "state_persistence"] {
                assert!(labels.iter().any(|label| label.contains(category)), "{suite} lacks exact {category}: {labels:?}");
            }
            assert!(cases.iter().all(|(_, case)| case.nlu_expect.is_some()));
        }
    }

    #[test]
    fn exact_intents_pair_full_slots_and_preserve_order() {
        let expected: Vec<ExpectedIntent> = serde_yaml::from_str(
            "- intent: HassTurnOn\n  slots: {area: kuche, domain: light}\n\
             - intent: HassTurnOff\n  slots: {entity_id: light.globe}",
        )
        .expect("expectation");
        let actual = vec![
            Intent::new("HassTurnOn").with("area", "kuche").with("domain", "light"),
            Intent::new("HassTurnOff").with("entity_id", "light.globe"),
        ];
        exact_intents_ok(&expected, &actual).expect("exact intents");
        assert!(exact_intents_ok(&expected, &[actual[1].clone(), actual[0].clone()]).is_err());
        assert!(exact_intents_ok(&expected, &[actual[0].clone()]).is_err());
    }

    #[test]
    fn reject_and_clarify_are_explicit_exact_outcomes() {
        let reject: NluExpectation = serde_yaml::from_str("reject: true\nclarify: false").expect("reject");
        let clarify: NluExpectation = serde_yaml::from_str("clarify: true\nintents: []").expect("clarify");
        let rejected = result(Vec::new(), false, false);
        exact_result_ok(&reject, &rejected).expect("rejected result");
        exact_result_ok(&clarify, &result(Vec::new(), true, false)).expect("clarified result");
        assert!(exact_result_ok(&reject, &result(Vec::new(), true, false)).is_err());
        assert!(exact_result_ok(&reject, &result(Vec::new(), false, true)).is_err(), "Chat must not count as reject");
    }

    fn result(intents: Vec<Intent>, clarify: bool, chat: bool) -> ParseResult {
        ParseResult { text: "test".into(), intents, speech: String::new(), clarify, conversation_id: "test".into(), chat, briefing: false }
    }
}
