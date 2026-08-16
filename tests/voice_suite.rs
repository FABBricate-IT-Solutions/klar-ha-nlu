//! Voice suite. German (`wohnung_mittel`) is required.
//! English (`wohnung_en`) is a smoke check — the upstream suite has no German.

mod voice_suite_support;

use klar_nlu::types::HomeGraph;
use voice_suite_support::{print_stats, run_groups, run_suite, suite_home};

#[test]
fn suite_wohnung_live_assist() {
    let home: HomeGraph = serde_json::from_str(include_str!("fixtures/wohnung_live.json")).expect("wohnung_live.json");
    let stats = run_groups("wohnung_live", &["assist"], home);
    print_stats("Klar NLU · Assist live (Home Assistant)", &stats);
    assert!(stats.ok + stats.fail > 0, "keine Assist-Testdateien — scripts/gen_voice_suite.py");
    assert_eq!(stats.fail, 0, "Assist-Sätze gegen den Live-Graphen fehl:\n{}", stats.fails.join("\n"));
}

#[test]
fn suite_deutsch() {
    let stats = run_suite("wohnung_mittel", true);
    print_stats("Klar NLU · Deutsch (wohnung_mittel)", &stats);
    assert!(stats.ok + stats.fail > 0, "keine Testdateien — scripts/gen_voice_suite.py ausführen");
    assert_eq!(stats.fail, 0, "unerwartete deutsche Fehler:\n{}", stats.fails.join("\n"));
}

#[test]
fn suite_english_smoke() {
    let stats = run_suite("wohnung_en", true);
    print_stats("Klar NLU · English smoke (wohnung_en)", &stats);
    assert!(stats.ok + stats.fail > 0, "keine englischen Testdateien");
    assert_eq!(stats.fail, 0, "unerwartete englische Smoke-Fehler:\n{}", stats.fails.join("\n"));
}

#[test]
fn suite_english_family_home() {
    let stats = run_suite("family_home_en", true);
    print_stats("Klar NLU · English (family_home_en)", &stats);
    assert!(stats.ok + stats.fail > 200, "englische Familiensuite nicht gefunden");
    assert_eq!(stats.fail, 0, "unerwartete englische Familienfehler:\n{}", stats.fails.join("\n"));
}

#[test]
fn suite_deutsch_familienhaus() {
    let stats = run_suite("familienhaus_de", true);
    print_stats("Klar NLU · Deutsch vergleichbar (familienhaus_de)", &stats);
    assert!(stats.ok + stats.fail > 200, "deutsche Vergleichssuite fehlt — scripts/gen_familienhaus_de.py");
    assert_eq!(stats.fail, 0, "unerwartete deutsche Familienfehler:\n{}", stats.fails.join("\n"));
}

#[test]
fn suite_m0_exact_english() {
    let stats = run_groups("family_home_en", &["m0_exact"], suite_home("family_home_en"));
    print_stats("Klar NLU · M0 exact English", &stats);
    assert!(stats.ok > 0, "English m0_exact cases are missing");
    assert_eq!(stats.fail, 0, "English m0_exact failures:\n{}", stats.fails.join("\n"));
}

#[test]
fn suite_m0_exact_german() {
    let stats = run_groups("familienhaus_de", &["m0_exact"], suite_home("familienhaus_de"));
    print_stats("Klar NLU · M0 exact German", &stats);
    assert!(stats.ok > 0, "German m0_exact cases are missing");
    assert_eq!(stats.fail, 0, "German m0_exact failures:\n{}", stats.fails.join("\n"));
}

#[test]
fn suite_m2_floors_english() {
    let stats = run_groups("family_home_en", &["m2_floors"], suite_home("family_home_en"));
    print_stats("Klar NLU · M2 floors English", &stats);
    assert!(stats.ok > 0, "English m2_floors cases are missing");
    assert_eq!(stats.fail, 0, "English m2_floors failures:\n{}", stats.fails.join("\n"));
}

#[test]
fn suite_m2_floors_german() {
    let stats = run_groups("familienhaus_de", &["m2_floors"], suite_home("familienhaus_de"));
    print_stats("Klar NLU · M2 floors German", &stats);
    assert!(stats.ok > 0, "German m2_floors cases are missing");
    assert_eq!(stats.fail, 0, "German m2_floors failures:\n{}", stats.fails.join("\n"));
}
