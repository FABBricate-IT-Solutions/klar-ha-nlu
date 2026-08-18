mod expect;
mod legacy;
pub(crate) mod parity;
mod runner;
mod schema;
mod waivers;
mod world;

use self::schema::Case;
use self::world::TestWorld;
use klar_nlu::home::{default_home, load_home_config};
use klar_nlu::types::HomeGraph;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
pub(crate) use runner::{print_stats, run_groups, run_groups_lang, run_suite};

pub(crate) struct RunStats {
    pub(crate) ok: usize,
    pub(crate) fail: usize,
    pub(crate) waived: usize,
    pub(crate) fails: Vec<String>,
    pub(crate) waivers: Vec<String>,
    pub(crate) used_waivers: BTreeSet<&'static str>,
}

impl RunStats {
    #[allow(dead_code)]
    pub(crate) fn absorb(&mut self, other: Self) {
        self.ok += other.ok;
        self.fail += other.fail;
        self.waived += other.waived;
        self.fails.extend(other.fails);
        self.waivers.extend(other.waivers);
        self.used_waivers.extend(other.used_waivers);
    }
}

pub(crate) fn suite_home(name: &str) -> HomeGraph {
    let path = datasets_root().join(name).join("home_config.yaml");
    if path.exists() {
        load_home_config(&path).unwrap_or_else(|error| panic!("home_config {}: {error}", path.display()))
    } else {
        default_home()
    }
}

pub(crate) fn datasets_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/datasets")
}

pub(crate) fn load_cases(dir: &Path) -> Vec<(String, Case)> {
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

#[cfg(test)]
mod tests {
    use super::expect::{exact_intents_ok, exact_result_ok};
    use super::schema::{ExpectedIntent, NluExpectation};
    use super::world::TestWorld;
    use super::*;
    use klar_nlu::session::Session;
    use klar_nlu::types::{Intent, ParseResult};

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
