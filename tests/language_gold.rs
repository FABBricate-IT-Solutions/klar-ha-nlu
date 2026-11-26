//! Gold natural sentences for every shipped locale (Q-02, next step after #478).
//!
//! Fixtures under `tests/datasets/gold/` are written by hand in each language's real
//! orthography. They come from the plan-evidence natural probe plus hand-authored
//! sentences for the other locales, never from `scripts/parity/` pack rewriting and
//! never from nlu.to. Every file carries its own tiny home (four rooms, eight devices
//! named in that language) so the sentences really resolve.
//!
//! Gates, from hard to soft:
//! 1. Every compiled locale ships `{code}.yaml` with at least ten cases and the core
//!    shapes (power on, power off, ListAdd, CHAT), unless it is a regional overlay that
//!    explicitly inherits a parent file (`INHERITS`).
//! 2. No case takes the wrong action family: a list add never switches a device, a
//!    power command never edits a list, a chat or confirm sentence never executes.
//! 3. Every case without a `soft` reason must also match its `expect` exactly.
//!
//! Cases with `soft:` stay in the file as probes. They are printed (`GOLD_SOFT`) but only
//! the family gate (2) applies to them.
use klar_nlu::lang::{pin_language, LangId, Tier};
use klar_nlu::nlu::parse;
use klar_nlu::parse::normalize::fold_latin;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, Intent, ParseDecision, RejectReason, Settings};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Regional overlays that run their parent's gold file under their own code. They must
/// not ship a file of their own, so the inheritance is never silent.
const INHERITS: [(&str, &str); 3] = [("de-AT", "de"), ("de-CH", "de"), ("en-GB", "en")];

/// Soft cases are probes; at least this many cases per locale stay exact gates.
const MIN_HARD_CASES: usize = 5;

const ROOM_IDS: [&str; 4] = ["living", "kitchen", "bedroom", "hall"];
/// Fixed device ids; the YAML only names them.
const DEVICES: [(&str, &str, &str); 7] = [
    ("light_living", "light.living", "living"),
    ("light_kitchen", "light.kitchen", "kitchen"),
    ("light_bedroom", "light.bedroom", "bedroom"),
    ("cover_bedroom", "cover.bedroom", "bedroom"),
    ("climate_living", "climate.living", "living"),
    ("lock_front", "lock.front", "hall"),
    ("shopping_list", "todo.shopping", ""),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldFile {
    rooms: BTreeMap<String, String>,
    devices: BTreeMap<String, String>,
    cases: Vec<GoldCase>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldCase {
    text: String,
    expect: String,
    #[serde(default)]
    kind: Option<String>,
    /// A known gap: the case is a probe, not an exact-match gate.
    #[serde(default)]
    soft: Option<String>,
}

fn gold_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/datasets/gold")
}

fn inherited_parent(code: &str) -> Option<&'static str> {
    INHERITS.iter().find(|(child, _)| *child == code).map(|(_, parent)| *parent)
}

fn load(code: &str) -> GoldFile {
    let path = gold_dir().join(format!("{code}.yaml"));
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("missing gold file for {code}: {path:?}: {error}"));
    serde_yaml::from_str(&raw).unwrap_or_else(|error| panic!("{path:?}: {error}"))
}

/// Every compiled locale with the gold file it runs: its own, or its parent's.
fn gold_targets() -> Vec<(&'static str, GoldFile)> {
    LangId::all().iter().map(|id| (id.code(), load(inherited_parent(id.code()).unwrap_or(id.code())))).collect()
}

fn home_for(file: &GoldFile) -> HomeGraph {
    let areas = ROOM_IDS
        .iter()
        .map(|id| AreaRec { area_id: (*id).into(), name: file.rooms[*id].clone(), aliases: vec![], floor_id: None })
        .collect();
    let entities = DEVICES
        .iter()
        .map(|(key, id, area)| EntityRec {
            entity_id: (*id).into(),
            name: file.devices[*key].clone(),
            domain: id.split_once('.').map_or("", |(domain, _)| domain).into(),
            platform: None,
            area: (!area.is_empty()).then(|| (*area).into()),
            aliases: vec![],
            tags: vec![],
        })
        .chain([music_speaker()])
        .collect();
    HomeGraph { areas, entities, ..HomeGraph::default() }
}

/// "Pause the music" needs a music player (a TV never counts), so every gold home owns one.
/// Its name is a brand, not a word of the language under test, and no sentence names it.
fn music_speaker() -> EntityRec {
    EntityRec {
        entity_id: "media_player.living".into(),
        name: "Sonos".into(),
        domain: "media_player".into(),
        platform: None,
        area: Some("living".into()),
        aliases: vec![],
        tags: vec!["music".into()],
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    Power,
    Set,
    Query,
    Timer,
    List,
    Media,
    Other,
}

fn intent_family(name: &str) -> Family {
    match name {
        "HassTurnOn" | "HassTurnOff" | "HassToggle" => Family::Power,
        "HassLightSet" | "HassClimateSetTemperature" => Family::Set,
        "HassGetState" | "HassClimateGetTemperature" => Family::Query,
        "HassStartTimer" | "HassTimerStart" | "HassTimerSet" => Family::Timer,
        "HassListAddItem" | "HassShoppingListAddItem" => Family::List,
        "HassMediaPause" | "HassMediaUnpause" | "HassMediaNext" | "KlarNoMusicPlayer" => Family::Media,
        _ => Family::Other,
    }
}

/// Names a gold head may resolve to, and the family it belongs to.
fn head_names(head: &str) -> (&'static [&'static str], Family) {
    match head {
        "TurnOn" => (&["HassTurnOn"], Family::Power),
        "TurnOff" => (&["HassTurnOff"], Family::Power),
        "LightSet" => (&["HassLightSet"], Family::Set),
        "ClimateSetTemperature" => (&["HassClimateSetTemperature"], Family::Set),
        "GetState" => (&["HassGetState"], Family::Query),
        "GetTemp" => (&["HassClimateGetTemperature", "HassGetState"], Family::Query),
        "StartTimer" => (&["HassStartTimer"], Family::Timer),
        "ListAdd" => (&["HassListAddItem", "HassShoppingListAddItem"], Family::List),
        "MediaPause" => (&["HassMediaPause", "KlarNoMusicPlayer"], Family::Media),
        other => panic!("unknown gold head {other:?}"),
    }
}

/// Reads are filtered out before this runs, so a state change must be exactly the wanted family.
fn families_compatible(wanted: Family, got: Family) -> bool {
    wanted == got
}

/// One `Head[@area][:slot=value]` part of an expect string.
struct Want {
    head: String,
    area: Option<String>,
    slot: Option<(String, String)>,
}

fn parse_want(part: &str) -> Want {
    let (rest, slot) = match part.split_once(':') {
        Some((rest, slot)) => (rest, slot.split_once('=').map(|(key, value)| (key.to_string(), value.to_lowercase()))),
        None => (part, None),
    };
    let (head, area) = match rest.split_once('@') {
        Some((head, area)) => (head, Some(area.to_string())),
        None => (rest, None),
    };
    Want { head: head.to_string(), area, slot }
}

fn want_hit(want: &Want, intents: &[Intent]) -> bool {
    let (names, _) = head_names(&want.head);
    intents.iter().any(|intent| {
        if !names.contains(&intent.name.as_str()) {
            return false;
        }
        if let Some(area) = &want.area {
            let by_entity = intent.slot("entity_id").is_some_and(|id| id.ends_with(&format!(".{area}")));
            if intent.slot("area") != Some(area.as_str()) && !by_entity {
                return false;
            }
        }
        match &want.slot {
            None => true,
            Some((key, value)) if key == "item" => intent.slot(key).is_some_and(|got| squash(got).contains(&squash(value))),
            Some((key, value)) => intent.slot(key).is_some_and(|got| got.to_lowercase() == *value),
        }
    })
}

/// The engine folds speech (`mjölk` → `mjoelk`) and may keep neighbours of the item, so an
/// item matches when the folded slot still contains the folded word.
fn squash(text: &str) -> String {
    fold_latin(text).chars().filter(|c| !c.is_whitespace()).collect()
}

struct Run {
    decision: ParseDecision,
    intents: Vec<Intent>,
    speech: String,
}

fn run_case(code: &str, home: &HomeGraph, text: &str) -> Run {
    let mut session = Session::new();
    let out = parse(text, home, &mut session, &[], &Settings::pinned(code));
    Run { intents: out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default(), decision: out.decision, speech: out.speech }
}

fn describe(run: &Run) -> String {
    let names: Vec<String> = run
        .intents
        .iter()
        .map(|intent| format!("{}{:?}", intent.name, intent.slots.iter().map(|s| format!("{}={}", s.name, s.value)).collect::<Vec<_>>()))
        .collect();
    format!("decision={:?} intents={names:?} speech={}", run.decision, run.speech)
}

/// A sentence that degrades to a read is a miss, not a wrong action.
fn is_read(name: &str) -> bool {
    intent_family(name) == Family::Query || name.starts_with("KlarGet") || name.starts_with("MassGet") || name == "KlarNoMusicPlayer"
}

/// `None` unless the home is changed the wrong way; rejects, clarifies and reads are fine.
fn family_violation(expect: &str, run: &Run) -> Option<String> {
    if !matches!(run.decision, ParseDecision::Execute) {
        return None;
    }
    let writes = run.intents.iter().filter(|intent| !is_read(&intent.name));
    match expect {
        "CHAT" | "CONFIRM" => writes.map(|intent| format!("{expect} sentence executed {} instead", intent.name)).next(),
        _ => {
            let heads: Vec<_> = expect.split('+').map(|part| head_names(&parse_want(part).head)).collect();
            let families: Vec<Family> = heads.iter().map(|(_, family)| *family).collect();
            let powers: Vec<&str> =
                heads.iter().filter(|(_, family)| *family == Family::Power).flat_map(|(names, _)| *names).copied().collect();
            writes
                .filter_map(|intent| {
                    let got = intent_family(&intent.name);
                    if !families.iter().any(|wanted| families_compatible(*wanted, got)) {
                        return Some(format!("{} is family {got:?}, wanted {families:?}", intent.name));
                    }
                    (got == Family::Power && !powers.contains(&intent.name.as_str()))
                        .then(|| format!("{} is the opposite of {expect}", intent.name))
                })
                .next()
        }
    }
}

fn exact_match(expect: &str, run: &Run) -> bool {
    match expect {
        "CONFIRM" => matches!(run.decision, ParseDecision::Confirm { .. }),
        "CHAT" => matches!(run.decision, ParseDecision::Chat),
        _ => {
            let wants: Vec<Want> = expect.split('+').map(parse_want).collect();
            matches!(run.decision, ParseDecision::Execute)
                && run.intents.len() >= wants.len()
                && wants.iter().all(|want| want_hit(want, &run.intents))
        }
    }
}

#[test]
fn every_locale_has_a_gold_file_with_natural_cases() {
    let dir = gold_dir();
    for id in LangId::all() {
        let code = id.code();
        let own = dir.join(format!("{code}.yaml"));
        if let Some(parent) = inherited_parent(code) {
            assert!(!own.exists(), "{code} inherits {parent}; delete {own:?} or drop it from INHERITS");
            assert_eq!(id.parent().map(|p| p.code()), Some(parent), "{code} must be a regional pack of {parent}");
            continue;
        }
        let file = load(code);
        assert!(file.cases.len() >= 10, "{code} gold needs ≥10 natural sentences, got {}", file.cases.len());
        for key in ROOM_IDS {
            assert!(file.rooms.get(key).is_some_and(|name| !name.trim().is_empty()), "{code} gold needs room {key}");
        }
        for (key, _, _) in DEVICES {
            assert!(file.devices.get(key).is_some_and(|name| !name.trim().is_empty()), "{code} gold needs device {key}");
        }
        let mut seen = BTreeSet::new();
        let mut heads = BTreeSet::new();
        for case in &file.cases {
            assert!(!case.text.trim().is_empty() && !case.expect.trim().is_empty(), "{code} empty gold case");
            assert!(seen.insert(case.text.clone()), "{code} duplicate gold text {:?}", case.text);
            if let Some(reason) = &case.soft {
                assert!(!reason.trim().is_empty(), "{code} soft case {:?} needs a reason", case.text);
            }
            if let Some(kind) = &case.kind {
                assert!(matches!(kind.as_str(), "execute" | "confirm" | "chat"), "{code} unknown kind {kind}");
            }
            for part in case.expect.split('+') {
                heads.insert(parse_want(part).head);
            }
        }
        for needed in ["TurnOn", "TurnOff", "ListAdd"] {
            assert!(heads.contains(needed), "{code} gold must cover {needed}");
        }
        assert!(file.cases.iter().any(|case| case.expect == "CHAT"), "{code} gold must cover CHAT");
    }
}

#[test]
fn gold_covers_every_compiled_locale() {
    let own: BTreeSet<String> = std::fs::read_dir(gold_dir())
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()? == "yaml").then(|| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))?
        })
        .collect();
    let compiled: BTreeSet<String> = LangId::all().iter().map(|id| id.code().to_string()).collect();
    let expected: BTreeSet<String> = compiled.iter().filter(|code| inherited_parent(code).is_none()).cloned().collect();
    assert_eq!(own, expected, "gold files must match the compiled locales (minus INHERITS)");
}

/// Q-02 / L-06: gold must never take a wrong action family (list must not turn on, etc.).
#[test]
fn gold_never_takes_the_wrong_action_family() {
    let mut bad = Vec::new();
    for (code, file) in gold_targets() {
        let home = home_for(&file);
        for case in &file.cases {
            let run = run_case(code, &home, &case.text);
            if let Some(why) = family_violation(&case.expect, &run) {
                bad.push(format!("{code}: \"{}\" expect {}: {why}; {}", case.text, case.expect, describe(&run)));
            }
        }
    }
    assert!(bad.is_empty(), "wrong action family on gold:\n{}", bad.join("\n"));
}

/// Every gold case without a `soft` reason must match its expectation exactly, and a soft
/// case that now matches must lose its marker so the gate only ever gets stricter.
#[test]
fn gold_cases_without_soft_reason_match_exactly() {
    let mut bad = Vec::new();
    let mut stale = Vec::new();
    let (mut hard, mut soft) = (0usize, 0usize);
    for (code, file) in gold_targets() {
        let home = home_for(&file);
        let (mut hard_ok, mut hard_total) = (0usize, 0usize);
        for case in &file.cases {
            let run = run_case(code, &home, &case.text);
            let ok = exact_match(&case.expect, &run);
            match &case.soft {
                None => {
                    hard_total += 1;
                    hard_ok += usize::from(ok);
                    if !ok {
                        bad.push(format!("{code}: \"{}\" expect {}; {}", case.text, case.expect, describe(&run)));
                    }
                }
                Some(reason) => {
                    soft += 1;
                    if ok {
                        stale.push(format!("{code}: \"{}\" now matches {}; drop `soft: {reason}`", case.text, case.expect));
                    } else {
                        println!("GOLD_SOFT {code}: \"{}\" expect {} ({reason}); {}", case.text, case.expect, describe(&run));
                    }
                }
            }
        }
        hard += hard_total;
        println!("GOLD_SCORE {code}: {hard_ok}/{hard_total} hard cases, {} soft", file.cases.len() - hard_total);
    }
    println!("GOLD_TOTAL hard={hard} soft={soft}");
    assert!(bad.is_empty(), "gold cases without a soft reason must match exactly:\n{}", bad.join("\n"));
    assert!(stale.is_empty(), "soft gold cases that pass now:\n{}", stale.join("\n"));
}

/// A locale may not hide behind soft cases: some natural sentences must be exact gates.
#[test]
fn every_locale_keeps_a_core_of_hard_gold_cases() {
    for id in LangId::all() {
        let code = id.code();
        if inherited_parent(code).is_some() {
            continue;
        }
        let file = load(code);
        let hard = file.cases.iter().filter(|case| case.soft.is_none()).count();
        assert!(hard >= MIN_HARD_CASES, "{code}: only {hard} hard gold cases; fix the pack instead of softening more");
    }
}

/// The family gate must actually bite: wrong writes fail, harmless reads and rejects do not.
#[test]
fn family_gate_flags_wrong_writes_and_ignores_reads() {
    let run = |decision: ParseDecision, names: &[&str]| Run {
        decision,
        intents: names.iter().map(|name| Intent::new(*name)).collect(),
        speech: String::new(),
    };
    let wrong = [
        ("ListAdd:item=milk", run(ParseDecision::Execute, &["HassTurnOn"])),
        ("TurnOn@living", run(ParseDecision::Execute, &["HassListAddItem"])),
        ("MediaPause", run(ParseDecision::Execute, &["HassMediaSearchAndPlay"])),
        ("GetState@kitchen", run(ParseDecision::Execute, &["HassTurnOn"])),
        ("CONFIRM", run(ParseDecision::Execute, &["HassTurnOff"])),
        ("CHAT", run(ParseDecision::Execute, &["HassTurnOn"])),
        ("TurnOff@kitchen", run(ParseDecision::Execute, &["HassLightSet"])),
        ("TurnOff@kitchen", run(ParseDecision::Execute, &["HassTurnOn"])),
        ("TurnOn@living", run(ParseDecision::Execute, &["HassTurnOff"])),
    ];
    for (expect, got) in &wrong {
        assert!(family_violation(expect, got).is_some(), "{expect} must flag {}", describe(got));
    }
    let fine = [
        ("ListAdd:item=milk", run(ParseDecision::Execute, &["HassGetState"])),
        ("CONFIRM", run(ParseDecision::Execute, &["HassGetState"])),
        ("TurnOn@living", run(ParseDecision::Reject { reason: RejectReason::NoAction }, &[])),
        ("TurnOn@living+TurnOff@kitchen", run(ParseDecision::Execute, &["HassTurnOn", "HassTurnOff"])),
        ("GetTemp@living", run(ParseDecision::Execute, &["HassClimateGetTemperature"])),
    ];
    for (expect, got) in &fine {
        assert!(family_violation(expect, got).is_none(), "{expect} must not flag {}", describe(got));
    }
}

/// L-06: Dutch shopping compound must add to the list, never turn the list on.
#[test]
fn nl_zet_melk_op_de_boodschappenlijst_adds_to_the_list() {
    let file = load("nl");
    let run = run_case("nl", &home_for(&file), "Zet melk op de boodschappenlijst");
    assert!(
        matches!(run.decision, ParseDecision::Execute) && run.intents.iter().any(|intent| intent.name == "HassListAddItem"),
        "expected ListAdd, got {}",
        describe(&run)
    );
    assert!(!run.intents.iter().any(|intent| intent.name == "HassTurnOn"), "must not turn the list on: {}", describe(&run));
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
