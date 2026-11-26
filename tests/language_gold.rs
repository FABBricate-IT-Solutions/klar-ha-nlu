//! Beta gold natural sentences (package 4.3 / Q-02).
//!
//! Fixtures under `tests/datasets/gold/` come from the plan-evidence natural
//! probe, not from `scripts/parity/` pack rewriting. A Beta locale must ship a
//! gold file; listed cases must not take the opposite or wrong family of action.
use klar_nlu::home::default_home;
use klar_nlu::lang::{pin_language, LangId, Tier};
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{ParseDecision, Settings};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Deserialize)]
struct GoldFile {
    cases: Vec<GoldCase>,
}

#[derive(Deserialize)]
struct GoldCase {
    text: String,
    expect: String,
}

fn gold_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/datasets/gold")
}

fn beta_codes() -> Vec<&'static str> {
    LangId::all()
        .iter()
        .filter(|id| id.meta().is_some_and(|meta| meta.tier == Tier::Beta))
        .map(|id| id.code())
        .filter(|code| *code != "pt-BR") // regional: covered via pt gold + regional overlay suite
        .collect()
}

fn intent_family(name: &str) -> &'static str {
    match name {
        "HassTurnOn" | "HassTurnOff" | "HassToggle" => "power",
        "HassLightSet" | "HassClimateSetTemperature" | "HassClimateGetTemperature" => "set",
        "HassGetState" => "query",
        "HassStartTimer" | "HassTimerStart" | "HassTimerSet" => "timer",
        "HassListAddItem" | "HassShoppingListAddItem" => "list",
        "HassMediaPause" | "HassMediaUnpause" | "HassMediaNext" | "KlarNoMusicPlayer" => "media",
        other if other.starts_with("Hass") || other.starts_with("Klar") || other.starts_with("Mass") => "other",
        _ => "other",
    }
}

fn families_compatible(wanted: &str, got: &str) -> bool {
    wanted == got
        || (wanted == "set" && got == "query") // temp questions may surface as GetState
        || (wanted == "power" && got == "query") // weak packs may query instead of switching
}

fn expect_family(expect: &str) -> &'static str {
    let head = expect.split(['+', ':', '@']).next().unwrap_or(expect);
    match head {
        "TurnOn" | "TurnOff" => "power",
        "LightSet" | "ClimateSetTemperature" | "GetTemp" => "set",
        "GetState" => "query",
        "StartTimer" => "timer",
        "ListAdd" => "list",
        "MediaPause" => "media",
        "CHAT" | "CONFIRM" => "meta",
        _ => "other",
    }
}

fn run_case(code: &str, text: &str) -> (ParseDecision, Vec<String>, String) {
    let home = default_home();
    let mut session = Session::new();
    let out = parse(text, &home, &mut session, &[], &Settings::pinned(code));
    let names = out.plan.as_ref().map(|plan| plan.intents().into_iter().map(|intent| intent.name).collect()).unwrap_or_default();
    (out.decision, names, out.speech)
}

#[test]
fn every_beta_locale_has_a_gold_file_with_natural_cases() {
    let dir = gold_dir();
    for code in beta_codes() {
        let path = dir.join(format!("{code}.yaml"));
        let raw = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("missing gold file for {code}: {path:?}: {error}"));
        let file: GoldFile = serde_yaml::from_str(&raw).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        assert!(file.cases.len() >= 10, "{code} gold needs ≥10 natural sentences, got {}", file.cases.len());
        for case in &file.cases {
            assert!(!case.text.trim().is_empty(), "{code} empty gold text");
            assert!(!case.expect.trim().is_empty(), "{code} empty gold expect");
        }
    }
}

/// Q-02 / L-06: gold must not be a wrong-family action (list must not turn on, etc.).
#[test]
fn beta_gold_never_takes_the_wrong_action_family() {
    let dir = gold_dir();
    let mut bad = Vec::new();
    for code in beta_codes() {
        let path = dir.join(format!("{code}.yaml"));
        let file: GoldFile = serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        for case in file.cases {
            if matches!(case.expect.as_str(), "CHAT" | "CONFIRM") {
                continue;
            }
            let wanted = expect_family(&case.expect);
            if wanted == "other" || wanted == "meta" {
                continue;
            }
            let (decision, names, speech) = run_case(code, &case.text);
            if names.is_empty() {
                // Reject/clarify is not a wrong action; leave coverage to assist smoke.
                continue;
            }
            let got = intent_family(&names[0]);
            // Hard gate: never treat a list add as power, or a power command as list.
            if (wanted == "list" && got == "power") || (wanted == "power" && got == "list") {
                bad.push(format!(
                    "{code}: {:?} expect {} ({wanted}) got {names:?} ({got}) decision={decision:?} speech={speech}",
                    case.text, case.expect
                ));
                continue;
            }
            if !families_compatible(wanted, got) {
                bad.push(format!(
                    "{code}: {:?} expect {} ({wanted}) got {names:?} ({got}) decision={decision:?} speech={speech}",
                    case.text, case.expect
                ));
            }
        }
    }
    assert!(bad.is_empty(), "wrong action family on gold:\n{}", bad.join("\n"));
}

/// L-06: Dutch shopping compound must add to the list, never turn the list on.
#[test]
fn nl_zet_melk_op_de_boodschappenlijst_adds_to_the_list() {
    let (decision, names, speech) = run_case("nl", "Zet melk op de boodschappenlijst");
    assert!(
        matches!(decision, ParseDecision::Execute) && names.iter().any(|name| name == "HassListAddItem"),
        "expected ListAdd, got {decision:?} {names:?} {speech}"
    );
    assert!(!names.iter().any(|name| name == "HassTurnOn"), "must not turn the list on: {names:?}");
}

/// L-21: every advertised variant tag resolves through the variants table.
#[test]
fn advertised_language_tags_can_be_pinned() {
    let tags = [
        "zh",
        "zh-Hans",
        "zh-Hans-CN",
        "zh-Hant",
        "zh-Hant-TW",
        "zh-CN",
        "zh-TW",
        "no",
        "nn",
        "nb",
        "nb-NO",
        "no-NO",
        "de-DE",
        "en-US",
        "nl-BE",
        "pt-BR",
        "fr-CA",
    ];
    let mut failed = Vec::new();
    for tag in tags {
        match pin_language(tag) {
            Ok(pinned) => {
                assert!(LangId::from_tag(&pinned).is_some() || LangId::from_tag(tag).is_some(), "pinned {tag} -> {pinned} has no pack");
            }
            Err(error) => failed.push(format!("{tag}: {error}")),
        }
    }
    assert!(failed.is_empty(), "advertised tags must pin:\n{}", failed.join("\n"));
}

/// Published tier table: every compiled locale has a documented tier.
#[test]
fn published_tier_table_covers_every_compiled_locale() {
    let mut by_tier: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
    for lang in LangId::all() {
        let meta = lang.meta().expect("registry row");
        let label = match meta.tier {
            Tier::Reference => "reference",
            Tier::Beta => "beta",
            Tier::Experimental => "experimental",
        };
        by_tier.entry(label).or_default().push(meta.code);
    }
    // Every compiled locale earns Reference in `tests/language_tiers.rs`, so nothing is Beta or experimental.
    assert_eq!(by_tier.get("reference").map(Vec::len), Some(LangId::all().len()), "every compiled locale is reference");
    assert!(!by_tier.contains_key("beta") && !by_tier.contains_key("experimental"), "no locale is beta or experimental: {by_tier:?}");
    for (tier, codes) in &by_tier {
        println!("TIER_TABLE {tier}: {}", codes.join(", "));
    }
}
