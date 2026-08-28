use klar_nlu::io::{hash_conversation_id, redact_entries, replay_tokens, BundleEntry, BundleRequest, BundleResponse, NameMap};
use klar_nlu::types::Intent;

fn entry(text: &str, entity_id: &str) -> BundleEntry {
    BundleEntry {
        id: "1".into(),
        ts_ms: 1,
        source: "http".into(),
        language: Some("de".into()),
        tokens: replay_tokens(text),
        request: BundleRequest { text: text.into(), conversation_id: Some("secret-session".into()) },
        response: BundleResponse {
            intents: vec![Intent::new("HassTurnOn").with("entity_id", entity_id).with("area", "kitchen")],
            speech: "Küche an".into(),
            clarify: false,
            chat: false,
            briefing: false,
        },
    }
}

#[test]
fn conversation_ids_are_hashed_and_stable() {
    assert_eq!(hash_conversation_id("secret-session"), hash_conversation_id("secret-session"));
    assert_ne!(hash_conversation_id("secret-session"), hash_conversation_id("other"));
    assert!(hash_conversation_id("secret-session").starts_with("cid_"));
}

#[test]
fn export_redacts_raw_text_entities_and_speech() {
    let redacted = redact_entries(&[entry("Mach das Küchenlicht an", "light.kitchen_island")], false);
    assert_eq!(redacted.len(), 1);
    assert!(!redacted[0].request.text.contains("Küchenlicht"), "{:?}", redacted[0].request.text);
    assert_eq!(redacted[0].request.text, redacted[0].tokens.join(" "));
    assert_eq!(redacted[0].request.conversation_id.as_deref(), Some(hash_conversation_id("secret-session").as_str()));
    assert_eq!(redacted[0].response.speech, "");
    assert_eq!(redacted[0].response.intents[0].slot("entity_id"), Some("light.e01"));
    assert_eq!(redacted[0].response.intents[0].slot("area"), Some("a01"));
}

#[test]
fn consent_keeps_raw_text_but_still_hashes_and_pseudonymizes() {
    let redacted = redact_entries(&[entry("Mach das Küchenlicht an", "light.kitchen_island")], true);
    assert_eq!(redacted[0].request.text, "Mach das Küchenlicht an");
    assert_eq!(redacted[0].response.speech, "Küche an");
    assert_eq!(redacted[0].request.conversation_id.as_deref(), Some(hash_conversation_id("secret-session").as_str()));
    assert_eq!(redacted[0].response.intents[0].slot("entity_id"), Some("light.e01"));
}

#[test]
fn entity_pseudonyms_are_stable_across_rows() {
    let mut names = NameMap::default();
    assert_eq!(names.entity("light.kitchen_island"), "light.e01");
    assert_eq!(names.entity("light.kitchen_island"), "light.e01");
    assert_eq!(names.entity("switch.dryer"), "switch.e02");
}

#[test]
fn replay_tokens_are_stable_and_normalized() {
    assert_eq!(replay_tokens("Küche AN!"), replay_tokens("kueche an"));
}

#[test]
fn journal_tokens_are_not_raw_sentence() {
    let spoken = "Mach das Küchenlicht an";
    let tokens = replay_tokens(spoken);
    assert_eq!(tokens, replay_tokens(spoken));
    assert_ne!(tokens.join(" "), spoken);
    assert!(!tokens.iter().any(|token| token.contains("Küche") || token.contains("Licht")));
}
