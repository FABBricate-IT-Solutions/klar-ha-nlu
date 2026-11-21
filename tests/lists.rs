//! To-do and shopping list commands: add, tick off and remove, in English and German.
//!
//! Each row pins what Home Assistant receives (intent + item) and what the user hears.
//! A remove or tick-off phrase must never turn into an add with the verb inside the item.

use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, ParseResult, Settings};

fn run(lang: &str, sentence: &str) -> ParseResult {
    parse(sentence, &HomeGraph::default(), &mut Session::new(), &[], &Settings::pinned(lang))
}

fn check(lang: &str, rows: &[(&str, &str, &str, &str)]) {
    for (sentence, intent, item, speech) in rows {
        let result = run(lang, sentence);
        assert!(!result.clarify, "{sentence}: unexpected clarify {result:?}");
        assert_eq!(result.intents.len(), 1, "{sentence}: {result:?}");
        let got = &result.intents[0];
        assert_eq!(got.name, *intent, "{sentence}: {got:?}");
        assert_eq!(got.slot("item"), Some(*item), "{sentence}: {got:?}");
        assert_eq!(got.slot("name"), Some("shopping_list"), "{sentence}: {got:?}");
        assert_eq!(result.speech, *speech, "{sentence}");
    }
}

#[test]
fn english_add() {
    check(
        "en",
        &[
            ("Add painkillers to the shopping list", "HassListAddItem", "painkillers", "Added painkillers to the list."),
            ("Put milk on the list", "HassListAddItem", "milk", "Added milk to the list."),
            ("Stick eggs on my shopping list", "HassListAddItem", "eggs", "Added eggs to the list."),
        ],
    );
}

#[test]
fn english_tick_off() {
    check(
        "en",
        &[
            ("Mark painkillers as done on the shopping list", "HassListCompleteItem", "painkillers", "Ticked painkillers off the list."),
            ("Check off painkillers on the shopping list", "HassListCompleteItem", "painkillers", "Ticked painkillers off the list."),
            ("Tick milk off the shopping list", "HassListCompleteItem", "milk", "Ticked milk off the list."),
            ("Cross milk off the list", "HassListCompleteItem", "milk", "Ticked milk off the list."),
        ],
    );
}

#[test]
fn english_remove() {
    check(
        "en",
        &[
            ("Remove painkillers from the shopping list", "HassListRemoveItem", "painkillers", "Removed painkillers from the list."),
            ("Delete painkillers from the shopping list", "HassListRemoveItem", "painkillers", "Removed painkillers from the list."),
            ("Take painkillers off the shopping list", "HassListRemoveItem", "painkillers", "Removed painkillers from the list."),
            ("Remove milk from my list", "HassListRemoveItem", "milk", "Removed milk from the list."),
            ("Erase milk from the shopping list", "HassListRemoveItem", "milk", "Removed milk from the list."),
        ],
    );
}

#[test]
fn german_add() {
    check(
        "de",
        &[
            ("Setze Milch auf die Einkaufsliste", "HassListAddItem", "milch", "Milch auf die Liste gesetzt."),
            ("Füge Brot zur Liste hinzu", "HassListAddItem", "brot", "Brot auf die Liste gesetzt."),
        ],
    );
}

#[test]
fn german_tick_off() {
    check(
        "de",
        &[
            ("Hake Milch auf der Einkaufsliste ab", "HassListCompleteItem", "milch", "Milch abgehakt."),
            ("Milch auf der Liste abhaken", "HassListCompleteItem", "milch", "Milch abgehakt."),
        ],
    );
}

#[test]
fn german_remove() {
    check(
        "de",
        &[
            ("Entferne Milch von der Einkaufsliste", "HassListRemoveItem", "milch", "Milch von der Liste entfernt."),
            ("Entferne Milch von der Liste", "HassListRemoveItem", "milch", "Milch von der Liste entfernt."),
            ("Lösche Milch von der Einkaufsliste", "HassListRemoveItem", "milch", "Milch von der Liste entfernt."),
            ("Streiche Milch von der Einkaufsliste", "HassListRemoveItem", "milch", "Milch von der Liste entfernt."),
        ],
    );
}

#[test]
fn stop_words_still_mean_stop_without_a_list() {
    for (lang, sentence, intent) in [
        ("en", "Cancel the timer", "HassCancelTimer"),
        ("en", "Delete the timer", "HassCancelTimer"),
        ("de", "Lösche den Timer", "HassCancelTimer"),
    ] {
        let result = run(lang, sentence);
        assert_eq!(result.intents.first().map(|intent| intent.name.as_str()), Some(intent), "{sentence}: {result:?}");
    }
}

#[test]
fn remove_words_do_nothing_without_a_list() {
    for (lang, sentence) in [("en", "Take a picture"), ("en", "Remove the noise"), ("de", "Entferne das")] {
        let result = run(lang, sentence);
        assert!(result.intents.iter().all(|intent| !intent.name.starts_with("HassList")), "{sentence}: {result:?}");
    }
}
