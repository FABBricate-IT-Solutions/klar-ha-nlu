//! Restore installs nothing it did not check, keeps the files as they were, replaces both
//! files or neither, and leaves the running home as a restart would load it.
mod state_support;

use axum::http::{Method, StatusCode};
use serde_json::{json, Value};
use state_support::{lamp_id, read_json, settings_archive, write_registry, Engine, TempDir, LLM_FILE, OVERLAY_FILE};
use std::os::unix::fs::PermissionsExt;

const BEFORE_RESTORE: &str = "klar_nlu.json.before-restore";
const LLM_BEFORE_RESTORE: &str = "llm_endpoint.json.before-restore";
const MADE_UP_KEY: &str = "sk-made-up-for-tests";
const VALID_ENDPOINT: &str = r#"{"base_url": "http://127.0.0.1:9/v1", "api_key": "", "model": "kleines-modell"}"#;

/// What an operator sees of the running state, route by route.
async fn memory(engine: &Engine) -> Vec<Value> {
    let mut seen = Vec::new();
    for path in
        ["/api/settings", "/api/custom", "/api/v2/policies", "/api/entities", "/api/ui", "/api/v2/llm/endpoint", "/api/lang/overlay"]
    {
        let (status, body) = engine.get(path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        seen.push(body);
    }
    seen
}

/// An engine with something in every section, so that a wrongly installed backup shows.
async fn calibrated(dir: &TempDir) -> Engine {
    let engine = Engine::start(dir.path());
    engine.push_home().await;
    let writes = [
        ("/api/entities", json!({"entity_id": lamp_id(3), "aliases": ["hobelbanklicht"], "nlu_ignore": true})),
        ("/api/settings", json!({"personality": "butler", "mode": "full", "quiet_ack": true})),
        ("/api/custom", json!([{"phrase": "werkstatt dicht machen", "intent": "HassTurnOff", "slots": {}}])),
        ("/api/v2/policies", json!({"policies": [{"id": "schloss", "effect": "block", "when": {"domain": "lock"}}]})),
        ("/api/ui", json!({"tab": "graph", "theme": "light"})),
        ("/api/v2/llm/endpoint", json!({"base_url": "http://127.0.0.1:9/v1", "api_key": MADE_UP_KEY, "model": "kleines-modell"})),
    ];
    for (path, body) in writes {
        assert_eq!(engine.post(path, body).await, StatusCode::OK, "{path}");
    }
    engine
}

#[tokio::test]
async fn every_invalid_section_answers_400_and_leaves_disk_and_memory_alone() {
    let dir = TempDir::new("restore-invalid");
    let engine = calibrated(&dir).await;
    let disk = (std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), std::fs::read(dir.file(LLM_FILE)).unwrap());
    let before = memory(&engine).await;
    let names = dir.names();

    let twice = json!({"id": "doppelt", "effect": "block", "when": {"domain": "lock"}});
    let invalid: Vec<(&str, Value, Option<&str>)> = vec![
        ("settings", json!({"settings": {"personality": "kapitaen", "mode": "full"}}), None),
        ("custom, phrase too short", json!({"custom": [{"phrase": "ab", "intent": "HassTurnOn", "slots": {}}]}), None),
        ("custom, unknown intent", json!({"custom": [{"phrase": "kaffee kochen bitte", "intent": "HassBrewCoffee", "slots": {}}]}), None),
        ("language", json!({"language": {"sets": {"nouns.gibt_es_nicht": {"add": ["funzel"]}}}}), None),
        (
            "language_history",
            json!({"language_history": [{"hash": "0", "label": "alt", "saved_at": "0",
                "custom": [{"phrase": "ab", "intent": "HassTurnOn", "slots": {}}]}]}),
            None,
        ),
        ("policies", json!({"policies": [twice, twice]}), None),
        ("speech_bank", json!({"speech_bank": {"entries": [{"rule_id": "", "variants": []}]}}), None),
        ("match_controls", json!({"match_controls": [{"id": "gibt_es_nicht", "enabled": false}]}), None),
        ("ui", json!({"ui": {"graph": {"light.lampe_02": {"x": 1e39, "y": 4}}}}), None),
        ("aliases", json!({"aliases": {"keine entity id": ["funzel"]}}), None),
        ("areas", json!({"areas": {"lampe": "werkstatt"}}), None),
        ("preferred", json!({"preferred": ["lampe"]}), None),
        ("nlu_ignore", json!({"nlu_ignore": [""]}), None),
        ("endpoint, not http", json!({}), Some(r#"{"base_url": "ftp://127.0.0.1/v1", "api_key": "", "model": "m"}"#)),
        ("endpoint, no model", json!({}), Some(r#"{"base_url": "http://127.0.0.1:9/v1", "api_key": "", "model": ""}"#)),
        ("endpoint, link-local", json!({}), Some(r#"{"base_url": "http://169.254.169.254/v1", "api_key": "", "model": "m"}"#)),
        ("valid endpoint beside an invalid overlay", json!({"preferred": ["lampe"]}), Some(VALID_ENDPOINT)),
    ];
    for (section, overlay, endpoint) in invalid {
        let status = engine.restore(settings_archive(&overlay.to_string(), endpoint)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{section}");
        assert_eq!(std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), disk.0, "{section}: klar_nlu.json changed");
        assert_eq!(std::fs::read(dir.file(LLM_FILE)).unwrap(), disk.1, "{section}: llm_endpoint.json changed");
        assert_eq!(dir.names(), names, "{section}: files appeared or went");
        assert_eq!(memory(&engine).await, before, "{section}: the running state changed");
    }
}

#[tokio::test]
async fn a_restore_keeps_both_files_as_they_were() {
    let dir = TempDir::new("restore-before");
    let engine = calibrated(&dir).await;
    let disk = (std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), std::fs::read(dir.file(LLM_FILE)).unwrap());

    let overlay = json!({"aliases": {lamp_id(8): ["drehbanklicht"]}, "settings": {"personality": "pirat", "mode": "full"}});
    assert_eq!(engine.restore(settings_archive(&overlay.to_string(), Some(VALID_ENDPOINT))).await, StatusCode::NO_CONTENT);

    assert_eq!(std::fs::read(dir.file(BEFORE_RESTORE)).unwrap(), disk.0);
    assert_eq!(std::fs::read(dir.file(LLM_BEFORE_RESTORE)).unwrap(), disk.1);
    let mode = std::fs::metadata(dir.file(LLM_BEFORE_RESTORE)).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "the copy holds the key");
    assert_eq!(read_json(&dir.file(OVERLAY_FILE))["aliases"][lamp_id(8)], json!(["drehbanklicht"]));
    assert_eq!(engine.get("/api/settings").await.1["personality"], "pirat");
    assert!(!dir.names().iter().any(|name| name.ends_with(".tmp")), "{:?}", dir.names());
}

#[tokio::test]
async fn the_local_key_stays_only_when_the_url_is_unchanged() {
    let dir = TempDir::new("restore-key");
    let engine = calibrated(&dir).await;

    let same = r#"{"base_url": "http://127.0.0.1:9/v1/", "api_key": "", "model": "anderes-modell"}"#;
    assert_eq!(engine.restore(settings_archive("{}", Some(same))).await, StatusCode::NO_CONTENT);
    let stored = read_json(&dir.file(LLM_FILE));
    assert_eq!(stored["model"], "anderes-modell");
    assert_eq!(stored["api_key"], MADE_UP_KEY, "same address, the key stays");

    let moved = r#"{"base_url": "http://192.168.77.5:8000/v1", "api_key": "", "model": "anderes-modell"}"#;
    assert_eq!(engine.restore(settings_archive("{}", Some(moved))).await, StatusCode::NO_CONTENT);
    let stored = read_json(&dir.file(LLM_FILE));
    assert_eq!(stored["base_url"], "http://192.168.77.5:8000/v1");
    assert_eq!(stored["api_key"], "", "the key must not follow to another address");
}

/// The lamp as `/api/entities` shows it.
async fn lamp(engine: &Engine, index: usize) -> Value {
    let (_, entities) = engine.get("/api/entities").await;
    entities.as_array().unwrap().iter().find(|entity| entity["entity_id"] == lamp_id(index).as_str()).cloned().expect("lamp")
}

/// The entity a German sentence would act on, or `null` when the engine does not act.
async fn target_of(engine: &Engine, text: &str) -> Value {
    let body = serde_json::to_vec(&json!({"text": text, "language": "de"})).unwrap();
    let (status, bytes) = engine.send(Method::POST, "/api/v2/parse", "application/json", body).await;
    assert_eq!(status, StatusCode::OK);
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    parsed["plan"]["steps"][0]["intent"]["slots"][0]["value"].clone()
}

async fn backup(engine: &Engine) -> Vec<u8> {
    let (status, archive) = engine.send(Method::GET, "/api/v2/settings/backup", "application/json", Vec::new()).await;
    assert_eq!(status, StatusCode::OK);
    archive
}

const BY_ALIAS: &str = "schalte das hobelbanklicht ein";

async fn the_running_home_follows_each_restore(engine: &Engine) {
    let clean = backup(engine).await;
    let named = json!({"entity_id": lamp_id(3), "aliases": ["hobelbanklicht"], "preferred": true});
    assert_eq!(engine.post("/api/entities", named).await, StatusCode::OK);
    assert_eq!(engine.post("/api/entities", json!({"entity_id": lamp_id(5), "nlu_ignore": true})).await, StatusCode::OK);
    assert_eq!(lamp(engine, 3).await["aliases"], json!(["hobelbanklicht"]));
    assert_eq!(lamp(engine, 5).await["tags"], json!(["nlu_ignore"]));
    let calibrated = backup(engine).await;

    assert_eq!(engine.restore(clean.clone()).await, StatusCode::NO_CONTENT);
    let restored = lamp(engine, 3).await;
    assert_eq!(restored["aliases"], json!([]), "an alias the backup does not hold must not stay active");
    assert_eq!(restored["tags"], json!([]), "a tag the backup does not hold must not stay active");
    assert_eq!(restored["name"], "Lampe 3", "the home itself is still there");
    assert_eq!(lamp(engine, 5).await["tags"], json!([]), "an ignore tag the backup does not hold must not stay active");
    assert_eq!(target_of(engine, BY_ALIAS).await, Value::Null);

    assert_eq!(engine.restore(calibrated).await, StatusCode::NO_CONTENT);
    assert_eq!(lamp(engine, 3).await["aliases"], json!(["hobelbanklicht"]));
    assert_eq!(lamp(engine, 5).await["tags"], json!(["nlu_ignore"]));
    assert_eq!(target_of(engine, BY_ALIAS).await, json!(lamp_id(3)), "a restored alias must resolve without a restart");

    assert_eq!(engine.restore(clean).await, StatusCode::NO_CONTENT);
    assert_eq!(target_of(engine, BY_ALIAS).await, Value::Null, "the removed alias still resolves");
}

#[tokio::test]
async fn the_running_home_follows_a_restore_when_home_assistant_pushed_it() {
    let dir = TempDir::new("restore-live");
    let engine = Engine::start(dir.path());
    engine.push_home().await;
    the_running_home_follows_each_restore(&engine).await;
}

#[tokio::test]
async fn the_running_home_follows_a_restore_when_it_came_from_the_registry() {
    let root = TempDir::new("restore-registry");
    let (config, data) = (root.file("config"), root.file("data"));
    std::fs::create_dir_all(&data).unwrap();
    write_registry(&config);
    let engine = Engine::from_dirs(&config, &data);
    the_running_home_follows_each_restore(&engine).await;
}

#[tokio::test]
async fn a_pushed_home_knows_the_aliases_saved_before_the_push() {
    let dir = TempDir::new("push-alias");
    let engine = Engine::start(dir.path());
    assert_eq!(engine.post("/api/entities", json!({"entity_id": lamp_id(3), "aliases": ["hobelbanklicht"]})).await, StatusCode::OK);
    engine.push_home().await;
    assert_eq!(target_of(&engine, BY_ALIAS).await, json!(lamp_id(3)));
}
