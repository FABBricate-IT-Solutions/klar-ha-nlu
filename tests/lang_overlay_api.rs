//! `POST /api/lang/overlay` changes only the parts its body names: a save without `custom`
//! keeps the user's phrases, a save without `language` keeps the lexicon.
mod state_support;

use axum::http::StatusCode;
use serde_json::{json, Value};
use state_support::{read_json, Engine, TempDir, OVERLAY_FILE};

fn phrases() -> Value {
    json!([
        {"phrase": "feierabend im wintergarten", "intent": "HassTurnOff", "slots": {}},
        {"phrase": "leselicht in der bibliothek", "intent": "HassTurnOn", "slots": {}}
    ])
}

fn lexicon() -> Value {
    json!({"sets": {"nouns.light_nouns": {"add": ["funzel"], "remove": []}}})
}

fn phrase_texts(overlay: &Value) -> Vec<String> {
    overlay["custom"]
        .as_array()
        .map(|rows| rows.iter().filter_map(|row| row["phrase"].as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

#[tokio::test]
async fn a_lexicon_save_without_custom_keeps_the_phrases() {
    let dir = TempDir::new("lexicon-keeps-phrases");
    let engine = Engine::start(dir.path());
    assert_eq!(engine.post("/api/lang/overlay", json!({"custom": phrases(), "label": "phrases"})).await, StatusCode::OK);

    let status = engine.post("/api/lang/overlay", json!({"language": lexicon(), "label": "lexicon-add"})).await;

    assert_eq!(status, StatusCode::OK);
    let expected = vec!["feierabend im wintergarten".to_string(), "leselicht in der bibliothek".to_string()];
    let (_, served) = engine.get("/api/lang/overlay").await;
    assert_eq!(phrase_texts(&served), expected, "served phrases");
    assert_eq!(served["language"]["sets"]["nouns.light_nouns"]["add"], json!(["funzel"]));
    assert_eq!(phrase_texts(&read_json(&dir.file(OVERLAY_FILE))), expected, "phrases on disk");
    assert_eq!(engine.state.custom.lock().await.len(), 2, "phrases the parser uses");
}

#[tokio::test]
async fn an_explicit_empty_custom_list_clears_the_phrases() {
    let dir = TempDir::new("empty-custom-clears");
    let engine = Engine::start(dir.path());
    assert_eq!(engine.post("/api/lang/overlay", json!({"custom": phrases(), "label": "phrases"})).await, StatusCode::OK);

    let status = engine.post("/api/lang/overlay", json!({"custom": [], "label": "cleared"})).await;

    assert_eq!(status, StatusCode::OK);
    let (_, served) = engine.get("/api/lang/overlay").await;
    assert_eq!(phrase_texts(&served), Vec::<String>::new());
    assert!(engine.state.custom.lock().await.is_empty());
}

#[tokio::test]
async fn a_phrase_save_without_language_keeps_the_lexicon() {
    let dir = TempDir::new("phrases-keep-lexicon");
    let engine = Engine::start(dir.path());
    assert_eq!(engine.post("/api/lang/overlay", json!({"language": lexicon(), "label": "lexicon-add"})).await, StatusCode::OK);

    let status = engine.post("/api/lang/overlay", json!({"custom": phrases(), "label": "phrases"})).await;

    assert_eq!(status, StatusCode::OK);
    let (_, served) = engine.get("/api/lang/overlay").await;
    assert_eq!(served["language"]["sets"]["nouns.light_nouns"]["add"], json!(["funzel"]));
    assert_eq!(phrase_texts(&served).len(), 2);
}
