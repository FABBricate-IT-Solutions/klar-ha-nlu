//! Acceptance for package 4.1: every pack token round-trips through fold/tokenize,
//! and each locale's alphabet is covered by the fold.

use klar_nlu::lang::{self, LangId};
use klar_nlu::parse::normalize::{fold_latin, tokenize};
use std::collections::BTreeSet;

fn pack_tokens(code: &str) -> BTreeSet<String> {
    let _bind = lang::bind(&[code.into()]);
    let cat = lang::catalog();
    let mut out = BTreeSet::new();
    for (word, _) in cat.verb_entries() {
        out.insert(word.to_string());
    }
    for word in cat.domain_map.keys() {
        out.insert((*word).to_string());
    }
    for (alias, canonical) in &cat.synonym_pairs {
        out.insert((*alias).to_string());
        out.insert((*canonical).to_string());
    }
    for words in [
        cat.fillers(),
        cat.action_keep(),
        cat.conjunctions(),
        cat.particles(),
        cat.affirm(),
        cat.or_words(),
        cat.all_words(),
        cat.light_nouns(),
        cat.cover_nouns(),
        cat.climate_nouns(),
        cat.media_nouns(),
        cat.lock_nouns(),
        cat.door_nouns(),
        cat.timer_nouns(),
        cat.list_nouns(),
        cat.calendar_nouns(),
        cat.fan_nouns(),
        cat.vacuum_nouns(),
        cat.scene_nouns(),
        cat.on_words(),
        cat.off_words(),
        cat.open_words(),
        cat.close_words(),
        cat.hours(),
        cat.minutes(),
        cat.seconds(),
        cat.question_words(),
        cat.kitchen(),
    ] {
        for word in words {
            out.insert((*word).to_string());
        }
    }
    // Hyphenated pack forms split on `-` (news topics rely on that); they are not
    // single-token round-trips. Spaces and underscores are multi-word or scene ids.
    out.retain(|word| !word.is_empty() && !word.contains(' ') && !word.contains('_') && !word.contains('-'));
    out
}

#[test]
fn every_pack_token_round_trips_through_tokenize() {
    let mut failures = Vec::new();
    for id in LangId::all() {
        let code = id.code();
        let tokens = pack_tokens(code);
        let _bind = lang::bind(&[code.into()]);
        for token in &tokens {
            let folded = fold_latin(token);
            if folded != *token {
                failures.push(format!("{code}: fold not idempotent on pack token {token:?} -> {folded:?}"));
                continue;
            }
            let got = tokenize(token);
            if got != [folded.clone()] {
                failures.push(format!("{code}: tokenize({token:?}) -> {got:?}, want [{folded:?}]"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "round-trip failures ({}):\n{}",
        failures.len(),
        failures.into_iter().take(40).collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn alphabet_coverage_per_locale_samples() {
    // Letters that used to fall through the fold (L-07, L-08, L-09).
    let samples = [
        ("sv", "kök", "koek"),
        ("fi", "keittiö", "keittioe"),
        ("hu", "függöny", "fueggoeny"),
        ("tr", "ışık", "isik"),
        ("pl", "światło", "swiatlo"),
        ("cs", "řádek", "radek"),
        ("vi", "đèn", "den"),
        ("el", "φῶς", "φως"),
        ("de", "Küche", "kueche"),
        ("fr", "entrée", "entree"),
    ];
    for (code, raw, want) in samples {
        let _bind = lang::bind(&[code.into()]);
        assert_eq!(fold_latin(raw), want, "{code} fold({raw})");
        assert_eq!(tokenize(raw), vec![want], "{code} tokenize({raw})");
    }
}
