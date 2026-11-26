//! `klar_nlu.json` has one owner: concurrent saves keep every change, an unreadable file is
//! never overwritten, one unknown value does not void the rest, and a failed save is an error.
mod state_support;

use axum::http::{Method, StatusCode};
use klar_nlu::home::overlay::load_overlay;
use serde_json::{json, Value};
use state_support::{lamp_id, process, read_json, settings_archive, Engine, TempDir, OVERLAY_FILE};
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

const BROKEN: &str = "{\"aliases\": {\"light.lampe_03\": [\"schraubstocklicht\"]}, \"settings\": ";

/// A file an older or hand-edited install can hold: `settings` without `personality`, a
/// personality this build does not know, and a graph point without a number.
const ODD_BUT_VALUABLE: &str = r#"{
  "aliases": {"light.lampe_03": ["schraubstocklicht", "bankleuchte"]},
  "custom": [{"phrase": "feierabend in der werkstatt", "intent": "HassTurnOff", "slots": {}}],
  "settings": {"mode": "full", "languages": ["de"], "quiet_ack": true},
  "ui": {"theme": "light", "graph": {"light.lampe_03": {"x": null, "y": 1}, "light.lampe_04": {"x": 2, "y": 3}}}
}"#;

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn twenty_four_parallel_saves_keep_twenty_four_aliases() {
    let dir = TempDir::new("parallel");
    let engine = Arc::new(Engine::start(dir.path()));
    let mut tasks = Vec::new();
    for index in 0..24 {
        let engine = engine.clone();
        tasks.push(tokio::spawn(async move {
            engine.post("/api/entities", json!({"entity_id": lamp_id(index), "aliases": [format!("funzel {index}")]})).await
        }));
    }
    for task in tasks {
        assert_eq!(task.await.unwrap(), StatusCode::OK);
    }
    let saved = read_json(&dir.file(OVERLAY_FILE));
    let aliases = saved["aliases"].as_object().expect("aliases object");
    assert_eq!(aliases.len(), 24, "every concurrent save must survive: {aliases:?}");
    for index in 0..24 {
        assert_eq!(aliases[&lamp_id(index)], json!([format!("funzel {index}")]));
    }
    assert!(!dir.names().iter().any(|name| name.ends_with(".tmp")), "no temp file may stay behind: {:?}", dir.names());
}

#[tokio::test]
async fn an_unreadable_file_stays_byte_identical_and_every_write_route_fails() {
    let dir = TempDir::new("corrupt");
    std::fs::write(dir.file(OVERLAY_FILE), BROKEN).unwrap();
    let engine = Engine::start(dir.path());
    let writes = [
        ("/api/entities", json!({"entity_id": lamp_id(5), "aliases": ["hobellicht"]})),
        ("/api/settings", json!({"personality": "butler", "mode": "full"})),
        ("/api/custom", json!([{"phrase": "werkbank aufräumen", "intent": "HassTurnOff", "slots": {}}])),
        ("/api/ui", json!({"tab": "graph"})),
        ("/api/v2/policies", json!({"policies": []})),
        ("/api/assignment/undo", json!({})),
    ];
    for (path, body) in writes {
        assert_eq!(engine.post(path, body).await, StatusCode::INTERNAL_SERVER_ERROR, "{path}");
        assert_eq!(std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), BROKEN.as_bytes(), "{path} changed the unreadable file");
    }
    // D-19: apply with nothing to do must not touch the undo stack or the corrupt file.
    assert_eq!(engine.post("/api/assignment/apply", json!({})).await, StatusCode::OK);
    assert_eq!(std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), BROKEN.as_bytes());
    assert_eq!(std::fs::read(dir.file("klar_nlu.json.bad")).unwrap(), BROKEN.as_bytes(), "a .bad copy keeps the bytes");
    let (_, settings) = engine.get("/api/settings").await;
    assert_eq!(settings["personality"], "default", "a refused save must not change the running settings");
}

#[tokio::test]
async fn a_backup_of_an_unreadable_file_is_an_error_not_an_empty_archive() {
    let dir = TempDir::new("corrupt-backup");
    std::fs::write(dir.file(OVERLAY_FILE), BROKEN).unwrap();
    let engine = Engine::start(dir.path());
    let (status, _) = engine.send(Method::GET, "/api/v2/settings/backup", "application/json", Vec::new()).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn one_unknown_value_does_not_void_the_file() {
    let dir = TempDir::new("tolerant");
    std::fs::write(dir.file(OVERLAY_FILE), ODD_BUT_VALUABLE).unwrap();
    let loaded = load_overlay(dir.path());
    assert_eq!(loaded.aliases["light.lampe_03"], ["schraubstocklicht", "bankleuchte"]);
    assert_eq!(loaded.custom[0].phrase, "feierabend in der werkstatt");
    let settings = loaded.settings.expect("settings survive a missing personality");
    assert!(settings.quiet_ack);
    assert_eq!(settings.languages, ["de"]);
    assert_eq!(loaded.ui.theme, "light");
    assert!(loaded.ui.graph.contains_key("light.lampe_04"));
    assert!(!loaded.ui.graph.contains_key("light.lampe_03"), "only the point without a number is dropped");

    let engine = Engine::start(dir.path());
    assert_eq!(engine.post("/api/entities", json!({"entity_id": lamp_id(7), "aliases": ["drehbanklicht"]})).await, StatusCode::OK);
    let saved = read_json(&dir.file(OVERLAY_FILE));
    assert_eq!(saved["aliases"]["light.lampe_03"], json!(["schraubstocklicht", "bankleuchte"]));
    assert_eq!(saved["aliases"]["light.lampe_07"], json!(["drehbanklicht"]));
    assert_eq!(saved["custom"][0]["phrase"], "feierabend in der werkstatt");
    assert_eq!(saved["settings"]["quiet_ack"], true);
    assert_eq!(
        std::fs::read(dir.file("klar_nlu.json.bad")).unwrap(),
        ODD_BUT_VALUABLE.as_bytes(),
        "what the engine could not read is kept beside the file"
    );
}

#[tokio::test]
async fn an_unknown_enum_value_falls_back_to_its_default() {
    let dir = TempDir::new("enum");
    let raw = r#"{"aliases": {"light.lampe_01": ["leselicht"]},
        "settings": {"personality": "kapitaen", "mode": "telepathie", "unit_system": "nautical", "refine_bands": ["status", "gesang"], "nlu_rag": true}}"#;
    std::fs::write(dir.file(OVERLAY_FILE), raw).unwrap();
    let loaded = load_overlay(dir.path());
    assert_eq!(loaded.aliases["light.lampe_01"], ["leselicht"]);
    let settings = serde_json::to_value(loaded.settings.expect("settings")).unwrap();
    assert_eq!(settings["personality"], "default");
    assert_eq!(settings["mode"], "full");
    assert_eq!(settings["unit_system"], "metric");
    assert_eq!(settings["refine_bands"], json!(["status"]));
    assert_eq!(settings["nlu_rag"], true);
}

#[test]
fn the_support_bundle_switch_does_not_empty_the_file_at_start() {
    let root = TempDir::new("bundle-start");
    std::fs::create_dir_all(root.file("data")).unwrap();
    let file = root.file("data").join(OVERLAY_FILE);
    let raw = ODD_BUT_VALUABLE.replace(r#""settings": {"mode": "full", "languages": ["de"], "quiet_ack": true},"#, "");
    std::fs::write(&file, &raw).unwrap();
    let running = process::start(&root, &[("KLAR_SUPPORT_BUNDLE", "1")]);
    let (status, body) = running.get("/api/settings");
    assert_eq!(status, 200, "{}", running.log());
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["support_bundle"], true);
    let saved = read_json(&file);
    assert_eq!(saved["aliases"]["light.lampe_03"], json!(["schraubstocklicht", "bankleuchte"]), "{}", running.log());
    assert_eq!(saved["custom"][0]["phrase"], "feierabend in der werkstatt");
    assert_eq!(saved["settings"]["support_bundle"], true);
}

#[tokio::test]
async fn a_restore_with_a_number_too_large_is_refused_and_the_file_stays_readable() {
    let dir = TempDir::new("restore-1e39");
    let engine = Engine::start(dir.path());
    assert_eq!(engine.post("/api/entities", json!({"entity_id": lamp_id(2), "aliases": ["saegelicht"]})).await, StatusCode::OK);
    let before = std::fs::read(dir.file(OVERLAY_FILE)).unwrap();
    let archive = settings_archive(r#"{"ui": {"graph": {"light.lampe_02": {"x": 1e39, "y": 4}}}}"#, None);
    assert_eq!(engine.restore(archive).await, StatusCode::BAD_REQUEST);
    assert_eq!(std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), before, "a refused restore leaves the file alone");
    assert_eq!(read_json(&dir.file(OVERLAY_FILE))["aliases"]["light.lampe_02"], json!(["saegelicht"]));
}

#[tokio::test]
async fn a_save_that_cannot_reach_the_disk_answers_500_and_keeps_memory() {
    let dir = TempDir::new("readonly");
    let engine = Engine::start(dir.path());
    assert_eq!(engine.post("/api/entities", json!({"entity_id": lamp_id(1), "aliases": ["feilenlicht"]})).await, StatusCode::OK);
    let before = std::fs::read(dir.file(OVERLAY_FILE)).unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    if std::fs::write(dir.file("probe"), b"x").is_ok() {
        // Running as root: a read-only directory does not stop writes, so there is nothing to prove.
        let _ = std::fs::remove_file(dir.file("probe"));
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let writes = [
        ("/api/settings", json!({"personality": "butler", "mode": "full"})),
        ("/api/entities", json!({"entity_id": lamp_id(9), "aliases": ["raspellicht"]})),
        ("/api/ui", json!({"tab": "graph"})),
        ("/api/v2/policies", json!({"policies": [{"id": "sperre", "effect": "block", "when": {"domain": "lock"}}]})),
    ];
    let mut statuses = Vec::new();
    for (path, body) in writes {
        statuses.push((path, engine.post(path, body).await));
    }
    let (_, settings) = engine.get("/api/settings").await;
    let (_, policies) = engine.get("/api/v2/policies").await;
    let (_, entities) = engine.get("/api/entities").await;
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    for (path, status) in statuses {
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{path}");
    }
    assert_eq!(std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), before);
    assert_eq!(settings["personality"], "default", "settings in memory must match the disk");
    assert_eq!(policies["policies"], json!([]), "policies in memory must match the disk");
    let lamp = entities.as_array().unwrap().iter().find(|entity| entity["entity_id"] == lamp_id(9).as_str()).unwrap();
    assert_eq!(lamp["aliases"], json!([]), "aliases in memory must match the disk");
}
