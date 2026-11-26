use super::*;
use crate::lang::{bind_catalog, catalog_for};
use crate::parse::normalize::tokenize;

/// Every number read from the text, and whether one of them could not be added up.
fn read(lang: &str, text: &str) -> (Vec<i32>, bool) {
    let _bound = bind_catalog(catalog_for(&[lang.to_string()]));
    let tokens = tokenize(text);
    (extract_numbers(&tokens), unreadable_number(&tokens))
}

fn reads(lang: &str, text: &str, wanted: i32) {
    assert_eq!(read(lang, text), (vec![wanted], false), "{lang}: {text}");
}

fn unreadable(lang: &str, text: &str) {
    assert!(read(lang, text).1, "{lang}: {text} must not be read as {:?}", read(lang, text).0);
}

#[test]
fn english_hundreds_and_tens() {
    for (text, wanted) in [
        ("one hundred percent", 100),
        ("a hundred percent", 100),
        ("hundred", 100),
        ("two hundred seconds", 200),
        ("one hundred twenty seconds", 120),
        ("one hundred twenty five", 125),
        ("hundred five", 105),
        ("twenty-five percent", 25),
        ("twenty one degrees", 21),
        ("nineteen", 19),
        ("50", 50),
    ] {
        reads("en", text, wanted);
    }
}

#[test]
fn german_hundreds_and_compounds() {
    for (text, wanted) in [
        ("einhundert Prozent", 100),
        ("hundert Prozent", 100),
        ("ein hundert Prozent", 100),
        ("zweihundert Sekunden", 200),
        ("zwei hundert Sekunden", 200),
        ("hundertzwanzig Sekunden", 120),
        ("einhundertfünfundzwanzig", 125),
        ("hundert fünf und zwanzig", 125),
        ("fünfundzwanzig Prozent", 25),
        ("fünf und zwanzig Prozent", 25),
        ("ein und zwanzig Grad", 21),
        ("einundzwanzig Grad", 21),
        ("siebzehn", 17),
    ] {
        reads("de", text, wanted);
    }
}

#[test]
fn words_that_only_look_like_numbers_are_none() {
    for text in ["jahrhundert", "hundertwasser", "rundum", "hund und katze", "grundzwanzig"] {
        assert_eq!(read("de", text), (vec![], false), "{text}");
    }
}

#[test]
fn a_listed_compound_is_read_when_its_hyphen_is_gone() {
    reads("fr", "dix-sept degrés", 17);
    reads("fr", "dix sept degrés", 17);
    reads("fr", "quatre-vingts", 80);
    // "cent" in "pour cent" is a number word of its own, apart from the value.
    assert_eq!(read("fr", "quatre-vingts pour cent"), (vec![80, 100], false));
    assert_eq!(read("fr", "cinq pour cent"), (vec![5, 100], false));
    reads("fr", "vingt degrés", 20);
}

#[test]
fn number_words_the_language_cannot_add_up_are_unreadable() {
    for (lang, text) in [
        // Wrong order / impossible sums still rejected (1.14 cases that 4.2 keeps).
        ("es", "cinco treinta"),
        ("it", "cinque venti per cento"),
        ("en", "five ten percent"),
        ("en", "one hundred and five percent"),
        ("en", "twenty twenty"),
        ("en", "ten one"),
        ("de", "zwei drei Grad"),
        ("de", "zwanzig und dreißig"),
        ("de", "zehn und zwanzig"),
    ] {
        unreadable(lang, text);
    }
}

#[test]
fn package_4_2_grammars_read_tens_with_ones() {
    reads("es", "treinta y cinco grados", 35);
    reads("pt", "vinte e cinco", 25);
    reads("fr", "vingt et un degrés", 21);
    assert_eq!(read("fr", "vingt cinq pour cent"), (vec![25, 100], false));
    reads("nl", "vijf en twintig procent", 25);
}

#[test]
fn numbers_apart_from_each_other_stay_readable() {
    assert_eq!(read("en", "set the office to twenty and the hall to thirty"), (vec![20, 30], false));
    assert_eq!(read("en", "five minutes thirty seconds"), (vec![5, 30], false));
    assert_eq!(read("en", "bedroom two to fifty percent"), (vec![50], false));
    assert_eq!(read("de", "Kinderzimmer zwei auf fünfzig Prozent"), (vec![50], false));
    assert_eq!(read("es", "pon la luz a treinta y la persiana a cinco"), (vec![30, 5], false));
}

#[test]
fn an_article_before_a_number_is_not_a_one() {
    assert_eq!(read("nl", "zet een tien minuten timer"), (vec![10], false));
    assert!(!read("fr", "mets un minuteur de dix minutes").1);
    assert_eq!(read("fr", "une minute"), (vec![1], false));
}

#[test]
fn a_number_held_together_by_and_becomes_one_token() {
    let fold = |lang: &str, text: &str| {
        let _bound = bind_catalog(catalog_for(&[lang.to_string()]));
        fold_spoken_numbers(&tokenize(text)).join(" ")
    };
    assert_eq!(fold("de", "stell die Heizung auf ein und zwanzig Grad"), "stell die heizung auf 21 grad");
    assert_eq!(fold("de", "fünf und zwanzig Prozent"), "25 prozent");
    assert_eq!(fold("de", "hundert fünf und zwanzig"), "125");
    // Romance / Germanic grammars from package 4.2 fold before "and" splits a clause.
    assert_eq!(fold("es", "treinta y cinco grados"), "35 grados");
    assert_eq!(fold("nl", "vijf en twintig procent"), "25 procent");
    // Plain words stay words: names and search text keep their spelling.
    assert_eq!(fold("en", "play twenty one pilots"), "play twenty one pilots");
    assert_eq!(fold("de", "Licht und Rollo an"), "licht und rollo an");
    // What cannot be added up is left alone and rejected later.
    assert_eq!(fold("en", "five ten percent"), "five ten percent");
}
