use super::*;
use crate::lang::{bind_catalog, catalog_for};

fn read(lang: &str, text: &str) -> Option<Answer> {
    let catalog = catalog_for(&[lang.to_string()]);
    let _bound = bind_catalog(catalog);
    classify(&words(text), catalog)
}

#[test]
fn yes_and_no_in_english_and_german() {
    for text in ["yes", "Yes please", "okay", "yeah", "sure", "go ahead", "do it", "yes yes"] {
        assert_eq!(read("en", text), Some(Answer::Yes), "{text}");
    }
    for text in ["no", "nope", "no thanks", "cancel", "never mind", "forget it", "don't", "no cancel"] {
        assert_eq!(read("en", text), Some(Answer::No), "{text}");
    }
    for text in ["ja", "Ja bitte", "okay", "jawohl", "mach das", "na klar"] {
        assert_eq!(read("de", text), Some(Answer::Yes), "{text}");
    }
    for text in ["nein", "nö", "abbrechen", "lass es", "vergiss es", "bitte nicht", "mach das nicht", "doch nicht"] {
        assert_eq!(read("de", text), Some(Answer::No), "{text}");
    }
}

#[test]
fn anything_more_than_an_answer_is_no_answer() {
    for text in ["yes but only the kitchen", "yes no", "okay turn on the light", "no the hall", "yesterday", "", "not yes"] {
        assert_eq!(read("en", text), None, "{text}");
    }
    for text in ["ja aber nur die küche", "ja nein", "jaja", "eineinhalb", "nicht ja"] {
        assert_eq!(read("de", text), None, "{text}");
    }
}

#[test]
fn a_generated_language_answers_with_the_words_it_has() {
    assert_eq!(read("fr", "oui"), Some(Answer::Yes));
    assert_eq!(read("fr", "non"), Some(Answer::No));
    assert_eq!(read("fr", "annuler"), None, "no checked cancel list for French");
    assert_eq!(read("ja", "はい"), Some(Answer::Yes));
    assert_eq!(read("ja", "いいえ"), Some(Answer::No));
}
