//! Voice suite. German (`wohnung_mittel`) is required.
//! English (`wohnung_en`) is a smoke check — the upstream suite has no German.

use klar_nlu::lexicon::default_home;
use klar_nlu::parse::parse;
use klar_nlu::registry::load_home_config;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Intent, Settings};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Condition {
    #[serde(rename = "type", default = "default_action")]
    kind: String,
    #[serde(default)]
    entity_id: Option<String>,
    #[serde(default)]
    area: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    attributes: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    minutes: Option<i64>,
    #[serde(default)]
    hours: Option<i64>,
    #[serde(default)]
    seconds: Option<i64>,
    #[serde(default)]
    item: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml::Value>,
}

fn default_action() -> String {
    "action".into()
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Sentences {
    Turns(Vec<Vec<String>>),
    Flat(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct Case {
    #[serde(default)]
    name: String,
    #[serde(default)]
    conditions: Vec<Condition>,
    sentences: Sentences,
}

fn datasets_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/datasets")
}

fn load_cases(dir: &Path) -> Vec<(String, Case)> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return out;
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("yaml"))
        .collect();
    files.sort();
    for path in files {
        let raw = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
        if parsed.as_sequence().is_some() {
            let Ok(cases) = serde_yaml::from_value::<Vec<Case>>(parsed) else {
                continue;
            };
            for c in cases {
                out.push((format!("{}::{}", path.file_stem().unwrap().to_string_lossy(), c.name), c));
            }
        } else if let Ok(c) = serde_yaml::from_value::<Case>(parsed) {
            out.push((format!("{}::{}", path.file_stem().unwrap().to_string_lossy(), c.name), c));
        }
    }
    out
}

fn turns_of(s: &Sentences) -> Vec<Vec<String>> {
    match s {
        Sentences::Turns(t) => t.clone(),
        Sentences::Flat(v) => v.iter().map(|s| vec![s.clone()]).collect(),
    }
}

fn cond_attr<'a>(cond: &'a Condition, key: &str) -> Option<&'a serde_yaml::Value> {
    cond.attributes.get(key).or_else(|| cond.extra.get(key))
}

fn expected_intent_names(cond: &Condition) -> Vec<&'static str> {
    if cond_attr(cond, "temperature").is_some() {
        return vec!["HassClimateSetTemperature"];
    }
    if cond_attr(cond, "brightness").is_some() || cond_attr(cond, "color").is_some() {
        return vec!["HassLightSet"];
    }
    if cond_attr(cond, "percentage").is_some() {
        return vec!["HassFanSetSpeed"];
    }
    if cond_attr(cond, "position").is_some() {
        return vec!["HassSetPosition"];
    }
    if cond_attr(cond, "is_volume_muted").is_some() {
        return vec!["HassMediaPlayerMute", "HassMediaUnpause", "HassTurnOn"];
    }
    if cond.kind == "query" {
        return vec!["HassGetState"];
    }
    if cond.kind == "shopping_list" || cond.kind == "todo_list" || cond.item.is_some() {
        return vec![
            "HassListAddItem",
            "HassListCompleteItem",
            "HassShoppingListAddItem",
            "HassShoppingListCompleteItem",
        ];
    }
    if cond.minutes.is_some()
        || cond.hours.is_some()
        || cond.seconds.is_some()
        || cond.entity_id.as_deref().is_some_and(|e| e.starts_with("timer."))
    {
        return vec![
            "HassStartTimer",
            "HassIncreaseTimer",
            "HassDecreaseTimer",
            "HassTimerStatus",
            "HassPauseTimer",
            "HassCancelTimer",
        ];
    }
    let eid = cond.entity_id.as_deref().unwrap_or("");
    if eid.starts_with("vacuum.") {
        return vec!["HassVacuumStart", "HassVacuumReturnToBase", "HassTurnOn"];
    }
    if eid.starts_with("scene.") || eid.starts_with("script.") {
        return vec!["HassTurnOn"];
    }
    match cond.state.as_deref() {
        Some("paused") => vec!["HassMediaPause"],
        Some("off") | Some("closed") | Some("unlocked") => vec!["HassTurnOff"],
        Some("open") | Some("locked") => vec!["HassTurnOn"],
        _ => vec!["HassTurnOn"],
    }
}

fn scene_covers(cond: &Condition, intents: &[Intent], home: &HomeGraph) -> bool {
    let Some(want) = cond.entity_id.as_deref() else {
        return false;
    };
    intents.iter().any(|i| {
        let Some(sid) = i.slot("entity_id") else {
            return false;
        };
        if sid == want && (sid.starts_with("scene.") || sid.starts_with("script.")) {
            return true;
        }
        home.scene_members
            .get(sid)
            .is_some_and(|members| members.iter().any(|m| m == want))
    })
}

fn entity_in<'a>(home: &'a HomeGraph, id: &str) -> Option<&'a klar_nlu::types::EntityRec> {
    home.entities.iter().find(|e| e.entity_id == id)
}

fn target_ok(intent: &Intent, cond: &Condition, home: &HomeGraph) -> bool {
    if let Some(want) = cond.entity_id.as_deref() {
        if intent.slot("entity_id") == Some(want) {
            return slot_attrs_ok(intent, cond);
        }
        if let Some(ent) = entity_in(home, want) {
            let area_hit = intent.slot("area") == ent.area.as_deref();
            let domain_hit = intent.slot("domain").is_none_or(|d| d == ent.domain);
            if area_hit && domain_hit {
                return slot_attrs_ok(intent, cond);
            }
        }
        return false;
    }
    if let Some(area) = cond.area.as_deref() {
        if intent.slot("area") == Some(area) {
            if let Some(d) = cond.domain.as_deref() {
                if intent.slot("domain").is_some_and(|got| got != d) {
                    if let Some(eid) = intent.slot("entity_id") {
                        return entity_in(home, eid).is_some_and(|e| {
                            e.area.as_deref() == Some(area) && e.domain == d
                        }) && slot_attrs_ok(intent, cond);
                    }
                    return false;
                }
            }
            return slot_attrs_ok(intent, cond);
        }
        if let Some(eid) = intent.slot("entity_id") {
            return entity_in(home, eid).is_some_and(|e| {
                e.area.as_deref() == Some(area)
                    && cond.domain.as_deref().is_none_or(|d| e.domain == d)
            }) && slot_attrs_ok(intent, cond);
        }
        return false;
    }
    slot_attrs_ok(intent, cond)
}

fn slot_attrs_ok(intent: &Intent, cond: &Condition) -> bool {
    if let Some(t) = cond_attr(cond, "temperature") {
        let want = yaml_num(t);
        return intent.slot("temperature") == Some(want.as_str());
    }
    if let Some(t) = cond_attr(cond, "brightness") {
        let want = yaml_num(t);
        return intent.slot("brightness") == Some(want.as_str());
    }
    if let Some(t) = cond_attr(cond, "percentage") {
        let want = yaml_num(t);
        return intent.slot("percentage") == Some(want.as_str());
    }
    if let Some(t) = cond_attr(cond, "color") {
        let want = t.as_str().unwrap_or("").to_string();
        return intent.slot("color") == Some(want.as_str());
    }
    if let Some(t) = cond_attr(cond, "position") {
        let want = yaml_num(t);
        return intent.slot("position") == Some(want.as_str());
    }
    true
}

fn yaml_num(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        other => format!("{other:?}"),
    }
}

fn cond_ok(cond: &Condition, intents: &[Intent], home: &HomeGraph) -> Result<(), String> {
    if scene_covers(cond, intents, home) {
        return Ok(());
    }
    let want = expected_intent_names(cond);
    let hit = intents
        .iter()
        .any(|i| want.contains(&i.name.as_str()) && target_ok(i, cond, home));
    if hit {
        Ok(())
    } else {
        Err(format!(
            "wanted {want:?} {:?} / {:?} in {intents:?}",
            cond.entity_id, cond.area
        ))
    }
}

struct RunStats {
    ok: usize,
    fail: usize,
    fails: Vec<String>,
}

fn suite_home(name: &str) -> HomeGraph {
    let path = datasets_root().join(name).join("home_config.yaml");
    if path.exists() {
        load_home_config(&path).unwrap_or_else(|err| {
            panic!("home_config {}: {err}", path.display());
        })
    } else {
        default_home()
    }
}

fn run_suite(name: &str, clarify_dir: bool) -> RunStats {
    let home = suite_home(name);
    let settings = Settings::default();
    let root = datasets_root().join(name);
    let mut stats = RunStats {
        ok: 0,
        fail: 0,
        fails: Vec::new(),
    };

    let mut groups = vec![
        "area",
        "devices",
        "query_area",
        "query_devices",
        "multiple_intents",
    ];
    if clarify_dir {
        groups.push("clarifications");
        groups.push("state_persistance");
        groups.push("timers");
        groups.push("lists");
    }

    for group in groups {
        let cases = load_cases(&root.join(group));
        for (label, case) in cases {
            let is_clarify = group == "clarifications";
            for turn_list in turns_of(&case.sentences) {
                let mut session = Session::new();
                let mut last = None;
                for (i, sentence) in turn_list.iter().enumerate() {
                    let result = parse(sentence, &home, &mut session, &[], &settings);
                    last = Some(result.clone());
                    if is_clarify && i + 1 < turn_list.len() && !result.clarify {
                        stats.fail += 1;
                        stats.fails.push(format!(
                            "{group}/{label}: first turn should clarify: {sentence:?} → {:?}",
                            result.intents
                        ));
                        last = None;
                        break;
                    }
                }
                let Some(result) = last else { continue };
                if result.clarify && !is_clarify {
                    stats.fail += 1;
                    stats.fails.push(format!(
                        "{group}/{label}: unexpected clarify for {turn_list:?}: {}",
                        result.speech
                    ));
                    continue;
                }
                let mut err = None;
                if is_clarify {
                    if !case.conditions.iter().any(|c| cond_ok(c, &result.intents, &home).is_ok())
                    {
                        err = Some(format!(
                            "wanted {:?} in {:?}",
                            case.conditions
                                .iter()
                                .filter_map(|c| c.entity_id.as_deref())
                                .collect::<Vec<_>>(),
                            result.intents
                        ));
                    }
                } else {
                    for cond in &case.conditions {
                        if let Err(e) = cond_ok(cond, &result.intents, &home) {
                            err = Some(e);
                            break;
                        }
                    }
                }
                if let Some(e) = err {
                    stats.fail += 1;
                    stats.fails.push(format!("{group}/{label}: {turn_list:?} → {e}"));
                } else {
                    stats.ok += 1;
                }
            }
        }
    }
    stats
}

fn print_stats(title: &str, stats: &RunStats) {
    let total = stats.ok + stats.fail;
    let pct = if total == 0 {
        0.0
    } else {
        100.0 * stats.ok as f64 / total as f64
    };
    println!("\n=== {title} ===");
    println!(
        "  {total} Sätze  {} ok  {} fehl  {pct:.1}%",
        stats.ok, stats.fail
    );
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for line in &stats.fails {
        let kind = if line.contains("unexpected clarify") {
            "clarify"
        } else if line.contains("HassSetPosition") {
            "position"
        } else if line.contains("HassFanSetSpeed") {
            "fan"
        } else if line.contains("HassLightSet") {
            "lightset"
        } else if line.contains("HassClimate") {
            "climate"
        } else if line.contains("HassGetState") {
            "query"
        } else if line.contains("HassTurnOff") {
            "off"
        } else if line.contains("HassTurnOn") {
            "on"
        } else if line.contains("Timer") {
            "timer"
        } else if line.contains("Shopping") || line.contains("todo") {
            "list"
        } else {
            "other"
        };
        *kinds.entry(kind.into()).or_default() += 1;
    }
    println!("  fail-kinds {kinds:?}");
    for line in stats.fails.iter().take(25) {
        println!("  FAIL {line}");
    }
    if stats.fails.len() > 25 {
        println!("  … {} weitere", stats.fails.len() - 25);
    }
    let dump = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!(
            "suite_fails_{}.txt",
            title
                .split('(')
                .nth(1)
                .unwrap_or("x")
                .trim_end_matches(')')
                .replace([' ', '·'], "_")
        ));
    let _ = std::fs::create_dir_all(dump.parent().unwrap());
    let _ = std::fs::write(&dump, stats.fails.join("\n"));
}

#[test]
fn suite_deutsch() {
    let stats = run_suite("wohnung_mittel", true);
    print_stats("Klar NLU · Deutsch (wohnung_mittel)", &stats);
    assert!(
        stats.ok + stats.fail > 0,
        "keine Testdateien — scripts/gen_voice_suite.py ausführen"
    );
    let pct = 100.0 * stats.ok as f64 / (stats.ok + stats.fail) as f64;
    assert!(
        pct >= 95.0,
        "deutsche Wohnungssuite unter 95% ({pct:.1}%). Erste Fehler:\n{}",
        stats.fails.iter().take(12).cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn suite_english_smoke() {
    let stats = run_suite("wohnung_en", true);
    print_stats("Klar NLU · English smoke (wohnung_en)", &stats);
    assert!(stats.ok + stats.fail > 0, "keine englischen Testdateien");
    let pct = 100.0 * stats.ok as f64 / (stats.ok + stats.fail) as f64;
    assert!(
        pct >= 60.0,
        "englischer Smoke unter 60% ({pct:.1}%). Fehler:\n{}",
        stats.fails.join("\n")
    );
}

#[test]
fn suite_english_family_home() {
    let stats = run_suite("family_home_en", true);
    print_stats("Klar NLU · English (family_home_en)", &stats);
    assert!(stats.ok + stats.fail > 200, "englische Familiensuite nicht gefunden");
    let pct = 100.0 * stats.ok as f64 / (stats.ok + stats.fail) as f64;
    assert!(
        pct >= 99.5,
        "englische Familiensuite unter 99.5% ({pct:.1}%). Erste Fehler:\n{}",
        stats.fails.iter().take(15).cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn suite_deutsch_familienhaus() {
    let stats = run_suite("familienhaus_de", true);
    print_stats("Klar NLU · Deutsch vergleichbar (familienhaus_de)", &stats);
    assert!(stats.ok + stats.fail > 200, "deutsche Vergleichssuite fehlt — scripts/gen_familienhaus_de.py");
    let pct = 100.0 * stats.ok as f64 / (stats.ok + stats.fail) as f64;
    assert!(
        pct >= 99.5,
        "deutsche Vergleichssuite unter 99.5% ({pct:.1}%). Erste Fehler:\n{}",
        stats.fails.iter().take(15).cloned().collect::<Vec<_>>().join("\n")
    );
}
