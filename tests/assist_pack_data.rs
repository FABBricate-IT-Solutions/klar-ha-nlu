//! Package 6.9: pack corrections live in one assist YAML data file.

use klar_nlu::home::default_home;
use klar_nlu::lang::{install_runtime_packs, new_case, reset_runtime_packs, ExternalPack};
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{ParseDecision, Settings};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

fn lock_packs() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|err| err.into_inner())
}

#[test]
fn new_case_writes_exactly_one_assist_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let assist = dir.path().join("packs").join("assist");
    fs::create_dir_all(&assist).expect("mkdir");
    let path = assist.join("fr.yaml");
    let before: Vec<PathBuf> = walk_files(dir.path());

    let msg = new_case("fr", "mode cinema klar69", "HassTurnOn", &["entity_id=scene.filmabend".into()], Some(&path)).expect("new-case");
    assert!(msg.contains(path.display().to_string().as_str()), "{msg}");

    let after: Vec<PathBuf> = walk_files(dir.path());
    let added: Vec<_> = after.iter().filter(|p| !before.contains(p)).collect();
    assert_eq!(added, vec![&path], "pack correction must touch one data file, got {added:?}");

    let pack = ExternalPack::load_file(&path).expect("load");
    assert_eq!(pack.id, "fr");
    assert_eq!(pack.intents.len(), 1);
    assert_eq!(pack.intents[0].phrase, "mode cinema klar69");
    assert_eq!(pack.intents[0].intent, "HassTurnOn");
}

#[test]
fn assist_yaml_correction_drives_parse() {
    let _guard = lock_packs();
    reset_runtime_packs();
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("fr.yaml");
    new_case("fr", "mode cinema klar69", "HassTurnOn", &["entity_id=scene.filmabend".into()], Some(&path)).expect("new-case");
    let pack = ExternalPack::load_file(&path).expect("load");
    install_runtime_packs(vec![pack]);

    let settings = Settings { languages: vec!["fr".into()], ..Settings::default() };
    let home = default_home();
    let mut session = Session::default();
    let outcome = parse("mode cinema klar69", &home, &mut session, &[], &settings);
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{:?}", outcome.decision);
    let intents = outcome.plan.expect("plan").intents();
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[0].name, "HassTurnOn");
    let entity = intents[0].slots.iter().find(|slot| slot.name == "entity_id").map(|slot| slot.value.as_str());
    assert_eq!(entity, Some("scene.filmabend"));
    reset_runtime_packs();
}

fn walk_files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in walkdir(root) {
        if entry.is_file() {
            out.push(entry);
        }
    }
    out.sort();
    out
}

fn walkdir(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walkdir(&path));
        } else {
            out.push(path);
        }
    }
    out
}
