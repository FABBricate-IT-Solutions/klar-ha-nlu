use klar_nlu::home::default_home;
use klar_nlu::home::overlay::load_overlay;
use klar_nlu::migrate::{apply, inspect};
use std::path::PathBuf;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("klar-migrate-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_v1(dir: &std::path::Path, body: &str) -> PathBuf {
    let path = dir.join("klar_nlu.json");
    std::fs::write(&path, body).unwrap();
    path
}

#[test]
fn dry_run_reports_conflicts_orphans_and_security_without_writing_v2() {
    let dir = temp_dir("dry");
    let path = write_v1(
        &dir,
        r#"{
          "legacy_flag": true,
          "aliases": {"light.missing": ["orb"]},
          "settings": {"personality": "default", "mode": "full", "confirm_risky_actions": false},
          "custom": [
            {"phrase": "filmabend", "intent": "HassTurnOn", "slots": {"entity_id": "scene.film"}},
            {"phrase": "filmabend", "intent": "HassTurnOff", "slots": {}},
            {"phrase": "tuer zu", "intent": "HassTurnOn", "slots": {"entity_id": "lock.front_door"}},
            {"phrase": "no", "intent": "NotARealIntent", "slots": {}}
          ]
        }"#,
    );
    let home = default_home();
    let report = inspect(&path, Some(&home)).expect("inspect");
    assert!(report.dry_run);
    assert!(!report.applied);
    assert!(report.conflicts.iter().any(|issue| issue.kind == "unknown_key" && issue.path == "legacy_flag"));
    assert!(report.conflicts.iter().any(|issue| issue.kind == "duplicate_phrase"));
    assert!(report.orphans.iter().any(|issue| issue.path == "light.missing"));
    assert!(report.security.iter().any(|issue| issue.kind == "unsafe_setting"));
    assert!(report.security.iter().any(|issue| issue.kind == "risky_custom"));
    assert!(report.security.iter().any(|issue| issue.kind == "unknown_intent"));
    assert_eq!(report.dropped_custom, 1);
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains("legacy_flag"), "dry-run must not rewrite the source");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn apply_writes_v2_only_and_forces_confirm() {
    let dir = temp_dir("apply");
    let path = write_v1(
        &dir,
        r#"{
          "legacy_flag": true,
          "settings": {"personality": "default", "mode": "full", "confirm_risky_actions": false},
          "custom": [
            {"phrase": "filmabend", "intent": "HassTurnOn", "slots": {"entity_id": "scene.film"}},
            {"phrase": "x", "intent": "NotARealIntent", "slots": {}}
          ]
        }"#,
    );
    let report = apply(&path, &dir, Some(&default_home())).expect("apply");
    assert!(!report.dry_run);
    assert!(report.applied);
    assert_eq!(report.accepted_custom, 1);
    let overlay = load_overlay(&dir);
    assert_eq!(overlay.custom.len(), 1);
    assert_eq!(overlay.custom[0].phrase, "filmabend");
    assert_eq!(overlay.custom[0].intent, "HassTurnOn");
    assert_eq!(overlay.custom[0].slots.get("entity_id").map(String::as_str), Some("scene.film"));
    assert!(overlay.settings.as_ref().is_some_and(|settings| settings.confirm_risky_actions));
    let raw = std::fs::read_to_string(dir.join("klar_nlu.json")).unwrap();
    assert!(!raw.contains("legacy_flag"));
    let _ = std::fs::remove_dir_all(&dir);
}
