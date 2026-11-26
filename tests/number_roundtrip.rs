//! Acceptance for package 4.2: every locale round-trips 0..=100 through spell → parse
//! against the pack reference table (and the small per-locale grammar for en/de).

use klar_nlu::lang::{self, LangId};
use klar_nlu::parse::normalize::tokenize;
use klar_nlu::parse::numbers::{extract_numbers, spell_number};

#[test]
fn zero_to_one_hundred_round_trips_per_locale() {
    let mut failures = Vec::new();
    for id in LangId::all() {
        let code = id.code();
        let codes = match id.meta().and_then(|meta| meta.parent) {
            Some(parent) => vec![code.to_string(), parent.to_string()],
            None => vec![code.to_string()],
        };
        let _bind = lang::bind(&codes);
        for n in 0..=100 {
            let Some(spelled) = spell_number(n) else {
                failures.push(format!("{code}: no spelling for {n}"));
                continue;
            };
            let tokens = tokenize(&spelled);
            let got = extract_numbers(&tokens);
            if got != [n] {
                failures.push(format!("{code}: {n} spelled {spelled:?} -> tokens {tokens:?} -> {got:?}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "0..=100 round-trip failures ({}):\n{}",
        failures.len(),
        failures.into_iter().take(60).collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn known_typos_from_l16_are_fixed_in_reference_tables() {
    let cases =
        [("sw", 14, "kumimanne"), ("sw", 18, "kumimanane"), ("cy", 30, "degarhugain"), ("sk", 14, "stirnast"), ("sk", 50, "padesiat")];
    for (lang, n, word) in cases {
        let _bind = lang::bind(&[lang.into()]);
        let cat = lang::catalog();
        assert_eq!(cat.number(word), Some(n), "{lang}: {word} should be {n}");
        assert_ne!(cat.number("kumimanane2"), Some(18), "unreachable sw form must be gone");
        assert_ne!(cat.number("degarddeg"), Some(30), "cy typo must be gone");
        assert_ne!(cat.number("padest"), Some(50), "sk typo must be gone");
        let spelled = spell_number(n).expect("spell");
        assert_eq!(extract_numbers(&tokenize(&spelled)), vec![n], "{lang} spell {n} = {spelled}");
    }
}
