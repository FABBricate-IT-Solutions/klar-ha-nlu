//! Staging eval: ASR boost prefers fuzzy-prone household names (Vorhang vs Vorrang).
//! NLU robustness stays in `tests/asr_robustness.rs` — this only checks prompt selection.

use klar_nlu::home::default_home;
use klar_nlu::lang::catalog_for;
use klar_nlu::speech::build_asr_boost;
use klar_nlu::types::{CustomSentence, EntityRec, HomeGraph};
use std::collections::HashMap;

fn de() -> &'static klar_nlu::lang::Catalog {
    catalog_for(&["de".into()])
}

#[test]
fn boost_includes_vorhang_alias_and_short_names_before_budget_exhaustion() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "cover.studio_vorhang".into(),
        name: "Studio Vorhang".into(),
        domain: "cover".into(),
        platform: None,
        area: Some("wohnzimmer".into()),
        aliases: vec!["Vorhang".into()],
        tags: Vec::new(),
    });
    home.assist = Some(home.entities.iter().map(|e| e.entity_id.clone()).collect());

    let custom = vec![CustomSentence { phrase: "Papiertonne raus".into(), intent: "HassTurnOn".into(), slots: HashMap::new() }];

    let out = build_asr_boost(&home, de(), &custom, "de", 200);
    let texts: Vec<&str> = out.terms.iter().map(|t| t.text.as_str()).collect();

    assert!(texts.iter().any(|t| t.contains("Vorhang")), "missing Vorhang in {texts:?}");
    assert!(texts.iter().any(|t| *t == "Kugel" || t.contains("Kugel")), "missing Kugel alias in {texts:?}");
    assert!(texts.iter().any(|t| t.contains("Papiertonne") || t.eq_ignore_ascii_case("papiertonne")), "missing custom anchor in {texts:?}");

    let vorhang = out.terms.iter().find(|t| t.text == "Vorhang" || t.text.contains("Vorhang")).expect("vorhang term");
    assert!(vorhang.tier == 2 || vorhang.tier == 3, "tier {}", vorhang.tier);

    // Prompt must not contain the common ASR confusable as a preferred term.
    assert!(!out.prompt.split_whitespace().any(|w| w == "Vorrang"), "prompt leaked Vorrang: {}", out.prompt);
}

#[test]
fn boost_stays_under_token_budget() {
    let mut home = default_home();
    home.assist = Some(home.entities.iter().map(|e| e.entity_id.clone()).collect());
    let out = build_asr_boost(&home, de(), &[], "de", 200);
    let tokens: usize = out.prompt.split_whitespace().map(|w| ((w.chars().count() + 3) / 4).max(1)).sum();
    assert!(tokens <= 200, "tokens={tokens} prompt={}", out.prompt);
    assert!(!out.updated_at.is_empty());
}

#[test]
fn empty_home_still_emits_pack_device_words() {
    let out = build_asr_boost(&HomeGraph::default(), de(), &[], "de", 40);
    assert!(
        out.terms.iter().any(|t| t.tier == 5 && (t.text.contains("rollo") || t.text.contains("Rollo") || t.text == "rollo")),
        "expected pack cover/curtain word in {:?}",
        out.terms
    );
}
