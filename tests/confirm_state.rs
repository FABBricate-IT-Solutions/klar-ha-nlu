//! An open question is a state: it is answered by the language's own yes, no and cancel
//! words, by one of the options it offered, or replaced by a complete new command. It
//! expires, and it is answered once.
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::nlu::parse;
use klar_nlu::session::{Sessions, PENDING_TTL};
use klar_nlu::types::{HomeGraph, ParseDecision, ParseOutcome, Settings};

fn test_home(lang: &str) -> HomeGraph {
    let de = lang == "de";
    let name = |en: &'static str, german: &'static str| if de { german } else { en };
    home(
        vec![area("office", name("Office", "Büro")), area("hall", name("Hall", "Flur")), area("kitchen", name("Kitchen", "Küche"))],
        vec![
            ent("light.hall_light", name("Hall Light", "Flur Licht"), "hall"),
            ent("light.office_light", name("Office Light", "Büro Licht"), "office"),
            ent("cover.office_blinds", name("Office Blinds", "Büro Rollo"), "office"),
            ent("lock.front_door", name("Front Door", "Haustür"), "hall"),
            ent("switch.oven", name("Oven", "Backofen"), "kitchen"),
            ent("climate.heating", name("Heating", "Heizung"), "hall"),
        ],
        &[],
    )
}

fn set(wanted: &[&str]) -> BTreeSet<String> {
    wanted.iter().map(|item| item.to_string()).collect()
}

fn unlock(lang: &str) -> &'static str {
    if lang == "de" {
        "Schließ die Haustür auf"
    } else {
        "unlock the front door"
    }
}

fn which_light(lang: &str) -> &'static str {
    if lang == "de" {
        "Mach das Licht an"
    } else {
        "turn on the light"
    }
}

fn asks(talk: &mut Talk, text: &str) -> String {
    let out = talk.say(text);
    assert!(matches!(out.decision, ParseDecision::Confirm { .. }), "{text}: {}", describe(&out));
    assert!(effects(&out, talk.home).is_empty(), "{text}: acted before the answer: {}", describe(&out));
    out.speech
}

fn offers(talk: &mut Talk, text: &str) {
    let out = talk.say(text);
    assert!(matches!(&out.decision, ParseDecision::Clarify { options, .. } if options.len() == 2), "{text}: {}", describe(&out));
}

fn does_nothing(talk: &mut Talk, text: &str) -> ParseOutcome {
    let out = talk.say(text);
    assert!(effects(&out, talk.home).is_empty(), "{text}: {}", describe(&out));
    assert_ne!(out.decision, ParseDecision::Execute, "{text}: {}", describe(&out));
    out
}

fn acts_on(talk: &mut Talk, text: &str, wanted: &[&str]) {
    let out = talk.say(text);
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {}", describe(&out));
    assert_eq!(effects(&out, talk.home), set(wanted), "{text}: {}", describe(&out));
}

#[test]
fn the_languages_own_yes_words_confirm() {
    for (lang, words) in [
        ("en", vec!["yes", "Yeah", "sure", "okay", "ok", "go ahead", "do it", "yes please", "Yes, please."]),
        ("de", vec!["ja", "Jawohl", "ja bitte", "okay", "klar", "mach das", "Ja, gerne.", "auf jeden Fall"]),
    ] {
        let home = test_home(lang);
        for word in words {
            let mut talk = Talk::new(lang, &home);
            asks(&mut talk, unlock(lang));
            acts_on(&mut talk, word, &["HassTurnOff lock.front_door"]);
        }
    }
}

#[test]
fn the_languages_own_no_and_cancel_words_drop_the_question() {
    for (lang, words, yes) in [
        ("en", vec!["no", "nope", "No thanks", "cancel", "never mind", "forget it", "stop", "don't do it"], "yes"),
        ("de", vec!["nein", "nee", "nein danke", "abbrechen", "lass es", "vergiss es", "doch nicht", "bitte nicht"], "ja"),
    ] {
        let home = test_home(lang);
        for word in words {
            let mut talk = Talk::new(lang, &home);
            asks(&mut talk, unlock(lang));
            let out = does_nothing(&mut talk, word);
            assert!(matches!(out.decision, ParseDecision::Reject { .. }), "{lang} {word}: {}", describe(&out));
            assert!(talk.session.pending().is_none(), "{lang} {word}: the question is still open");
            does_nothing(&mut talk, yes);
        }
    }
}

#[test]
fn a_refusal_wins_over_a_yes_in_the_same_answer() {
    for (lang, answer) in [("en", "yes no"), ("en", "ok cancel"), ("de", "ja nein"), ("de", "ja doch nicht")] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        asks(&mut talk, unlock(lang));
        does_nothing(&mut talk, answer);
    }
}

#[test]
fn an_unclear_answer_asks_again_and_a_later_yes_still_counts() {
    for (lang, unclear, yes) in [("en", "maybe later perhaps", "yes"), ("de", "vielleicht irgendwann", "ja")] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        let prompt = asks(&mut talk, unlock(lang));
        let again = does_nothing(&mut talk, unclear);
        assert_eq!(again.speech, prompt, "{lang}: the same question is asked again");
        assert!(matches!(again.decision, ParseDecision::Confirm { .. }), "{lang}: {}", describe(&again));
        acts_on(&mut talk, yes, &["HassTurnOff lock.front_door"]);
    }
}

#[test]
fn an_either_or_question_does_not_accept_yes() {
    for (lang, yes, pick) in [("en", "yes", "office"), ("en", "okay", "the office"), ("de", "ja", "Büro"), ("de", "ja bitte", "im Büro")]
    {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        offers(&mut talk, which_light(lang));
        let out = does_nothing(&mut talk, yes);
        assert!(matches!(&out.decision, ParseDecision::Clarify { options, .. } if options.len() == 2), "{lang} {yes}: {}", describe(&out));
        acts_on(&mut talk, pick, &["HassTurnOn light.office_light"]);
    }
}

#[test]
fn an_either_or_question_is_dropped_by_no() {
    for (lang, no, pick) in [("en", "no", "office"), ("en", "never mind", "office"), ("de", "nein", "Büro"), ("de", "abbrechen", "Büro")]
    {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        offers(&mut talk, which_light(lang));
        does_nothing(&mut talk, no);
        assert!(talk.session.pending().is_none(), "{lang} {no}: the question is still open");
        let out = talk.say(pick);
        assert!(!effects(&out, &home).contains("HassTurnOn light.office_light"), "{lang}: {}", describe(&out));
    }
}

#[test]
fn options_match_word_for_word() {
    // "off" is the start of "office" and "hal" the start of "hall": neither names a room.
    for (lang, half) in [("en", "off"), ("en", "hal"), ("en", "of"), ("de", "bür"), ("de", "flu")] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        offers(&mut talk, which_light(lang));
        let out = does_nothing(&mut talk, half);
        assert!(matches!(out.decision, ParseDecision::Clarify { .. }), "{lang} {half}: {}", describe(&out));
    }
}

#[test]
fn a_new_command_replaces_an_either_or_question() {
    for (lang, command, wanted, pick) in [
        ("en", "turn off the oven", "HassTurnOff switch.oven", "office"),
        ("de", "Mach den Backofen aus", "HassTurnOff switch.oven", "Büro"),
    ] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        offers(&mut talk, which_light(lang));
        acts_on(&mut talk, command, &[wanted]);
        assert!(talk.session.pending().is_none(), "{lang}: the old question is still open");
        let out = talk.say(pick);
        assert!(
            !effects(&out, &home).contains("HassTurnOn light.office_light"),
            "{lang}: the old question was answered: {}",
            describe(&out)
        );
    }
}

#[test]
fn a_new_risky_command_replaces_the_confirmation_it_interrupts() {
    for (lang, other, yes, wanted) in [
        ("en", "close the office blinds", "yes", "HassTurnOff cover.office_blinds"),
        ("de", "Mach das Rollo im Büro zu", "ja", "HassTurnOff cover.office_blinds"),
    ] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        let first = asks(&mut talk, unlock(lang));
        let second = asks(&mut talk, other);
        assert_ne!(first, second, "{lang}: the new question names the new command");
        acts_on(&mut talk, yes, &[wanted]);
    }
}

#[test]
fn an_open_question_expires() {
    for (lang, yes, pick) in [("en", "yes", "office"), ("de", "ja", "Büro")] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        asks(&mut talk, unlock(lang));
        talk.session.backdate_pending(PENDING_TTL + Duration::from_secs(1));
        does_nothing(&mut talk, yes);

        let mut talk = Talk::new(lang, &home);
        offers(&mut talk, which_light(lang));
        talk.session.backdate_pending(PENDING_TTL + Duration::from_secs(1));
        let out = talk.say(pick);
        assert!(!effects(&out, &home).contains("HassTurnOn light.office_light"), "{lang}: {}", describe(&out));

        let mut talk = Talk::new(lang, &home);
        asks(&mut talk, unlock(lang));
        talk.session.backdate_pending(PENDING_TTL - Duration::from_secs(5));
        acts_on(&mut talk, yes, &["HassTurnOff lock.front_door"]);
    }
}

#[test]
fn confirming_works_once() {
    for (lang, yes) in [("en", "yes"), ("de", "ja")] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        asks(&mut talk, unlock(lang));
        acts_on(&mut talk, yes, &["HassTurnOff lock.front_door"]);
        does_nothing(&mut talk, yes);
        does_nothing(&mut talk, yes);
    }
}

fn yes_from_store(sessions: &Mutex<Sessions>, home: &HomeGraph, settings: &Settings) -> bool {
    let mut session = sessions.lock().unwrap().take(Some("hallway"));
    let out = parse("yes", home, &mut session, &[], settings);
    sessions.lock().unwrap().put(session);
    out.decision == ParseDecision::Execute && !effects(&out, home).is_empty()
}

#[test]
fn two_yes_at_the_same_time_execute_once() {
    let home = test_home("en");
    let settings = Settings::pinned("en");
    for _ in 0..20 {
        let sessions = Arc::new(Mutex::new(Sessions::default()));
        let mut asked = sessions.lock().unwrap().take(Some("hallway"));
        let out = parse("unlock the front door", &home, &mut asked, &[], &settings);
        assert!(matches!(out.decision, ParseDecision::Confirm { .. }), "{}", describe(&out));
        sessions.lock().unwrap().put(asked);

        // Both requests read the session before either has written its answer back.
        let mut first = sessions.lock().unwrap().take(Some("hallway"));
        let mut second = sessions.lock().unwrap().take(Some("hallway"));
        let outs = [parse("yes", &home, &mut first, &[], &settings), parse("yes", &home, &mut second, &[], &settings)];
        sessions.lock().unwrap().put(second);
        sessions.lock().unwrap().put(first);
        let ran = outs.iter().filter(|out| out.decision == ParseDecision::Execute).count();
        assert_eq!(ran, 1, "{} / {}", describe(&outs[0]), describe(&outs[1]));
        assert!(!yes_from_store(&sessions, &home, &settings), "a third yes ran it again");
    }

    for _ in 0..20 {
        let sessions = Arc::new(Mutex::new(Sessions::default()));
        let mut asked = sessions.lock().unwrap().take(Some("hallway"));
        parse("unlock the front door", &home, &mut asked, &[], &settings);
        sessions.lock().unwrap().put(asked);
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let (sessions, home, settings) = (Arc::clone(&sessions), home.clone(), settings.clone());
                thread::spawn(move || yes_from_store(&sessions, &home, &settings))
            })
            .collect();
        let ran = workers.into_iter().filter_map(|worker| worker.join().ok()).filter(|ran| *ran).count();
        assert_eq!(ran, 1, "eight parallel yes must unlock the door exactly once");
    }
}

#[test]
fn the_question_names_action_and_target() {
    for (lang, text, parts) in [
        ("en", "unlock the front door", vec!["unlock", "Front Door"]),
        ("en", "lock the front door", vec!["lock Front Door"]),
        ("en", "close the office blinds", vec!["close", "Office Blinds"]),
        ("en", "close the office blinds and unlock the front door", vec!["close Office Blinds", "unlock Front Door"]),
        ("de", "Schließ die Haustür auf", vec!["Haustür", "aufschließen"]),
        ("de", "Mach das Rollo im Büro zu", vec!["Büro Rollo", "schließen"]),
        // A generated pack has no action wording yet: its question names at least the device.
        ("fr", "déverrouille front door", vec!["Front Door"]),
    ] {
        let home = test_home(if lang == "de" { "de" } else { "en" });
        let mut talk = Talk::new(lang, &home);
        let prompt = asks(&mut talk, text);
        for part in parts {
            assert!(prompt.contains(part), "{lang} {text}: {prompt:?} does not say {part:?}");
        }
        assert!(!prompt.contains("lock.front_door") && !prompt.contains("cover."), "{prompt:?} shows an entity id");
    }
}

#[test]
fn a_generated_language_answers_with_its_own_words() {
    let home = test_home("en");
    for (answer, runs) in [("oui", true), ("Oui.", true), ("non", false), ("yes", false), ("ja", false)] {
        let mut talk = Talk::new("fr", &home);
        asks(&mut talk, "déverrouille front door");
        let out = talk.say(answer);
        assert_eq!(!effects(&out, &home).is_empty(), runs, "fr {answer}: {}", describe(&out));
    }
}

#[test]
fn a_word_that_only_contains_nein_is_not_a_correction() {
    let home = test_home("de");
    for text in
        ["Stell die Heizung auf eineinhalb Grad", "Mach das Licht im Büro an, einen Moment", "Ist die Haustür verschlossen, meine Liebe"]
    {
        let mut talk = Talk::new("de", &home);
        talk.say("Mach das Licht im Flur an");
        talk.say(text);
        assert!(talk.session.wrong_log.is_empty(), "{text}: taken as a correction of the last command");
    }
    let mut talk = Talk::new("de", &home);
    talk.say("Mach das Licht im Flur an");
    talk.say("nein, das war falsch");
    assert_eq!(talk.session.wrong_log.len(), 1, "a real correction is still heard");
}

#[test]
fn all_lights_off_in_a_large_home_does_not_ask() {
    let rooms: Vec<String> = (1..=9).map(|i| format!("room{i}")).collect();
    let home = home(
        rooms.iter().map(|room| area(room, &format!("Room {}", &room[4..]))).collect(),
        rooms.iter().map(|room| ent(&format!("light.{room}"), &format!("Lamp {}", &room[4..]), room)).collect(),
        &[],
    );
    let mut talk = Talk::new("en", &home);
    let out = talk.say("turn off all the lights");
    assert_eq!(out.decision, ParseDecision::Execute, "{}", describe(&out));
    assert_eq!(effects(&out, &home).len(), 9, "{}", describe(&out));
}
