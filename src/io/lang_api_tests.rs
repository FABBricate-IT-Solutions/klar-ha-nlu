use super::*;
use crate::home::{default_home, LoadedHome};
use crate::lang::{installed_user_overlay, reset_runtime_packs, SetDelta};
use crate::types::CustomSentence;
use crate::types::ParseDecision;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::{Mutex, MutexGuard};

fn overlay_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

async fn lock_overlay() -> MutexGuard<'static, ()> {
    overlay_lock().lock().await
}

fn state(tag: &str) -> AppState {
    reset_runtime_packs();
    let dir = std::env::temp_dir().join(format!("klar-m5-{tag}-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::remove_file(dir.join("klar_nlu.json"));
    AppState::new(
        LoadedHome {
            graph: default_home(),
            settings: Settings::default(),
            custom: Vec::new(),
            language: Default::default(),
            policies: Vec::new(),
            speech_bank: Default::default(),
            match_controls: Vec::new(),
        },
        dir,
        None,
    )
}

fn peer() -> ConnectInfo<SocketAddr> {
    ConnectInfo("127.0.0.1:9".parse().unwrap())
}

fn rule(phrase: &str, intent: &str) -> CustomSentence {
    CustomSentence { phrase: phrase.into(), intent: intent.into(), slots: HashMap::new() }
}

#[tokio::test]
async fn omitted_language_keeps_set_deltas() {
    let _guard = lock_overlay().await;
    let state = state("keep");
    let language =
        LanguageOverlay { sets: [("nouns.light_nouns".into(), SetDelta { add: vec!["kugelchen".into()], remove: vec![] })].into() };
    let _ = set_overlay(
        State(state.clone()),
        peer(),
        HeaderMap::new(),
        Json(OverlayIn { custom: vec![rule("filmabend", "HassTurnOn")], language: Some(language.clone()), label: Some("sets".into()) }),
    )
    .await
    .expect("save with sets");
    let kept = set_overlay(
        State(state.clone()),
        peer(),
        HeaderMap::new(),
        Json(
            serde_json::from_value(serde_json::json!({
                "custom": [{"phrase": "filmabend zwei", "intent": "HassTurnOn", "slots": {}}],
                "label": "phrase-only"
            }))
            .unwrap(),
        ),
    )
    .await
    .expect("save without language")
    .0;
    assert_eq!(kept.custom[0].phrase, "filmabend zwei");
    assert_eq!(kept.language, language);
    assert!(installed_user_overlay().is_some_and(|overlay| overlay.sets.contains_key("nouns.light_nouns")));
    reset_runtime_packs();
}

#[tokio::test]
async fn default_rollback_restores_latest_history() {
    let _guard = lock_overlay().await;
    let state = state("roll");
    let _ = set_overlay(
        State(state.clone()),
        peer(),
        HeaderMap::new(),
        Json(OverlayIn { custom: vec![rule("erste regel", "HassTurnOn")], language: None, label: Some("a".into()) }),
    )
    .await
    .expect("save a");
    let after_b = set_overlay(
        State(state.clone()),
        peer(),
        HeaderMap::new(),
        Json(OverlayIn { custom: vec![rule("zweite regel", "HassTurnOff")], language: None, label: Some("b".into()) }),
    )
    .await
    .expect("save b")
    .0;
    assert_eq!(after_b.custom[0].phrase, "zweite regel");
    let named = rollback(State(state.clone()), peer(), HeaderMap::new(), Json(RollbackIn { hash: Some(after_b.history[0].hash.clone()) }))
        .await
        .expect("named rollback")
        .0;
    assert_eq!(named.custom[0].phrase, "erste regel");
    let _ = set_overlay(
        State(state.clone()),
        peer(),
        HeaderMap::new(),
        Json(OverlayIn { custom: vec![rule("zweite regel", "HassTurnOff")], language: None, label: Some("b2".into()) }),
    )
    .await
    .expect("save b again");
    let rolled = rollback(State(state), peer(), HeaderMap::new(), Json(RollbackIn { hash: None })).await.expect("default rollback").0;
    assert_eq!(rolled.custom[0].phrase, "erste regel");
    reset_runtime_packs();
}

#[tokio::test]
async fn preview_does_not_install_live_overlay() {
    let _guard = lock_overlay().await;
    let state = state("prev");
    let proposed =
        LanguageOverlay { sets: [("nouns.light_nouns".into(), SetDelta { add: vec!["vorschauwort".into()], remove: vec![] })].into() };
    let outcome = preview(
        State(state),
        peer(),
        HeaderMap::new(),
        Json(PreviewIn { text: "Licht an".into(), language: Some("de".into()), custom: None, language_overlay: Some(proposed) }),
    )
    .await
    .expect("preview")
    .0;
    assert!(matches!(
        outcome.decision,
        ParseDecision::Execute | ParseDecision::Clarify { .. } | ParseDecision::Reject { .. } | ParseDecision::Confirm { .. }
    ));
    assert!(
        installed_user_overlay().is_none_or(|overlay| overlay
            .sets
            .get("nouns.light_nouns")
            .is_none_or(|delta| !delta.add.iter().any(|word| word == "vorschauwort"))),
        "preview must not install the proposed overlay"
    );
    reset_runtime_packs();
}

#[test]
fn explain_speech_speaks_path_ids() {
    let outcome = ParseOutcome {
        schema_version: "2.0".into(),
        text: "Licht an".into(),
        conversation_id: "t".into(),
        decision: ParseDecision::Execute,
        speech: "Wohnzimmerlicht ist an.".into(),
        confidence: 0.9,
        margin: 0.1,
        selected_candidate_id: None,
        candidates: Vec::new(),
        plan: None,
        evidence: Vec::new(),
        trace: crate::types::ParseTrace { stages: Vec::new(), discarded: Vec::new(), tokens: Vec::new(), normalized: String::new() },
        briefing: false,
        retrieval: None,
        policy_trace: Some(crate::types::PolicyTrace {
            match_node: Some(crate::types::PolicyTraceMatch { id: "area_command".into(), score: 0.93, origin: "engine".into() }),
            house: Some(crate::types::PolicyTraceLayer {
                id: "prefer-ceiling".into(),
                hit: Some("prefer_entity".into()),
                origin: "operator".into(),
            }),
            band: Some("execute".into()),
            ..crate::types::PolicyTrace::default()
        }),
        quiet_ack_eligible: false,
        refine_band: None,
    };
    let out = explain_outcome("en", &outcome);
    assert_eq!(out.decision, "execute");
    assert_eq!(out.reply, "Wohnzimmerlicht ist an.");
    assert!(out.speech.contains("Match `area_command`"));
    assert!(out.speech.contains("house `prefer-ceiling`"));
    assert!(out.speech.contains("execute"));
    assert_eq!(out.policy_trace.as_ref().and_then(|trace| trace.match_node.as_ref()).map(|node| node.id.as_str()), Some("area_command"));
    let de = explain_outcome("de", &outcome);
    assert!(de.speech.contains("Haus `prefer-ceiling`"));
    assert!(de.speech.contains("ausgeführt"));
}
