//! Every plan passes the safety decision: undo, routines and rule scripts get no way around
//! the confirmation, an allow rule speaks only for its own device, a bare "yes" needs an
//! open question, and household phrases are whole utterances.
use std::collections::{BTreeSet, HashMap};

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::nlu::parse_with_policies;
use klar_nlu::types::{
    CustomSentence, HomeGraph, IntentPlan, ParseDecision, ParseOutcome, PolicyEffect, PolicyMatch, PolicyRule, RejectReason, SpeechBank,
};

fn test_home(lang: &str) -> HomeGraph {
    let de = lang == "de";
    let name = |en: &'static str, german: &'static str| if de { german } else { en };
    home(
        vec![area("office", name("Office", "Büro")), area("hall", name("Hall", "Flur")), area("garage", "Garage")],
        vec![
            ent("light.hall_light", name("Hall Light", "Flur Licht"), "hall"),
            ent("light.office_light", name("Office Light", "Büro Licht"), "office"),
            ent("cover.office_blinds", name("Office Blinds", "Büro Rollo"), "office"),
            ent("cover.garage_door", name("Garage Door", "Garagentor"), "garage"),
            ent("lock.front_door", name("Front Door", "Haustür"), "hall"),
            ent("script.evening", name("Evening Routine", "Abendroutine"), ""),
            ent("script.hidden", name("Hidden Routine", "Versteckte Routine"), ""),
        ],
        &["script.hidden"],
    )
}

fn rule(entity: &str, effect: PolicyEffect) -> PolicyRule {
    PolicyRule {
        id: format!("rule-{entity}"),
        enabled: true,
        label: entity.into(),
        when: PolicyMatch { entity_id: Some(entity.into()), ..PolicyMatch::default() },
        effect,
        prefer: None,
        payload: None,
    }
}

fn routine(phrase: &str, script: &str) -> CustomSentence {
    CustomSentence { phrase: phrase.into(), intent: "HassTurnOn".into(), slots: HashMap::from([("entity_id".into(), script.into())]) }
}

fn say_with(talk: &mut Talk, routines: &[CustomSentence], text: &str) -> ParseOutcome {
    parse_with_policies(text, talk.home, &mut talk.session, routines, &talk.settings, &talk.rules, &SpeechBank::default())
}

fn set(wanted: &[&str]) -> BTreeSet<String> {
    wanted.iter().map(|item| item.to_string()).collect()
}

fn acts_on(talk: &mut Talk, text: &str, wanted: &[&str]) {
    let out = talk.say(text);
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {}", describe(&out));
    assert_eq!(effects(&out, talk.home), set(wanted), "{text}: {}", describe(&out));
}

fn asks_first(talk: &mut Talk, text: &str) {
    let out = talk.say(text);
    assert!(matches!(out.decision, ParseDecision::Confirm { .. }), "{text}: {}", describe(&out));
    assert!(out.plan.is_none(), "{text}: nothing may run before yes: {}", describe(&out));
}

fn does_nothing(talk: &mut Talk, text: &str) {
    let out = talk.say(text);
    assert!(effects(&out, talk.home).is_empty(), "{text}: {}", describe(&out));
    assert!(!matches!(out.decision, ParseDecision::Execute), "{text}: {}", describe(&out));
}

/// S-01: undo used to carry `safety_confirmed` and reversed a lock without a question.
#[test]
fn undo_after_a_lock_asks_again() {
    for (lang, lock, unlock, yes, undo) in [
        ("en", "lock the front door", "unlock the front door", "yes", "undo"),
        ("de", "Schließ die Haustür ab", "Schließe die Haustür auf", "ja", "rückgängig"),
    ] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        asks_first(&mut talk, lock);
        acts_on(&mut talk, yes, &["HassTurnOn lock.front_door"]);
        asks_first(&mut talk, undo);
        acts_on(&mut talk, yes, &["HassTurnOff lock.front_door"]);

        let mut talk = Talk::new(lang, &home);
        asks_first(&mut talk, unlock);
        acts_on(&mut talk, yes, &["HassTurnOff lock.front_door"]);
        asks_first(&mut talk, undo);
    }
}

#[test]
fn undo_after_a_cover_asks_again() {
    for (lang, close, open, yes, undo) in [
        ("en", "close the garage door", "open the office blinds", "yes", "undo that"),
        ("de", "Mach das Garagentor zu", "Mach das Rollo im Büro auf", "ja", "Mach das bitte rückgängig"),
    ] {
        let home = test_home(lang);
        // Reversing a confirmed close opens the garage: that needs its own yes.
        let mut talk = Talk::new(lang, &home);
        asks_first(&mut talk, close);
        acts_on(&mut talk, yes, &["HassTurnOff cover.garage_door"]);
        asks_first(&mut talk, undo);
        acts_on(&mut talk, yes, &["HassTurnOn cover.garage_door"]);
        // Reversing an open is a close, which asks on its own account.
        let mut talk = Talk::new(lang, &home);
        acts_on(&mut talk, open, &["HassTurnOn cover.office_blinds"]);
        asks_first(&mut talk, undo);
    }
}

#[test]
fn undo_of_a_light_still_runs_directly() {
    for (lang, on, undo) in [("en", "turn on the hall light", "undo"), ("de", "Mach das Licht im Flur an", "rückgängig")] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        acts_on(&mut talk, on, &["HassTurnOn light.hall_light"]);
        acts_on(&mut talk, undo, &["HassTurnOff light.hall_light"]);
    }
}

#[test]
fn undo_obeys_a_block_rule_and_the_assist_exposure() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home);
    acts_on(&mut talk, "turn on the hall light", &["HassTurnOn light.hall_light"]);
    talk.rules = vec![rule("light.hall_light", PolicyEffect::Block)];
    let out = talk.say("undo");
    assert!(matches!(out.decision, ParseDecision::Reject { reason: RejectReason::Unsafe }), "{}", describe(&out));
    assert!(out.plan.is_none());

    let mut talk = Talk::new("en", &home);
    acts_on(&mut talk, "turn on the hall light", &["HassTurnOn light.hall_light"]);
    let unexposed = test_home("en");
    let unexposed = HomeGraph { assist: Some(Default::default()), ..unexposed };
    talk.home = &unexposed;
    does_nothing(&mut talk, "undo");
}

/// S-02: an allow rule for the blinds used to lift the confirmation for the lock as well.
#[test]
fn allow_rule_lifts_only_its_own_step_in_a_compound_plan() {
    for (lang, alone, both, yes) in [
        ("en", "close the office blinds", "close the office blinds and unlock the front door", "yes"),
        ("de", "Mach das Rollo im Büro zu", "Mach das Rollo im Büro zu und schließe die Haustür auf", "ja"),
    ] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        talk.rules = vec![rule("cover.office_blinds", PolicyEffect::Allow)];
        acts_on(&mut talk, alone, &["HassTurnOff cover.office_blinds"]);
        asks_first(&mut talk, both);
        acts_on(&mut talk, yes, &["HassTurnOff cover.office_blinds", "HassTurnOff lock.front_door"]);
    }
}

#[test]
fn allow_rules_for_every_step_skip_the_prompt() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home);
    talk.rules = vec![rule("cover.office_blinds", PolicyEffect::Allow), rule("lock.front_door", PolicyEffect::Allow)];
    acts_on(
        &mut talk,
        "close the office blinds and unlock the front door",
        &["HassTurnOff cover.office_blinds", "HassTurnOff lock.front_door"],
    );
}

#[test]
fn block_rule_on_a_later_step_stops_the_plan_despite_an_earlier_allow() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home);
    talk.rules = vec![rule("cover.office_blinds", PolicyEffect::Allow), rule("lock.front_door", PolicyEffect::Block)];
    let out = talk.say("close the office blinds and unlock the front door");
    assert!(matches!(out.decision, ParseDecision::Reject { reason: RejectReason::Unsafe }), "{}", describe(&out));
    assert!(out.plan.is_none());
}

/// S-09: "yes" with nothing open used to switch the device named last.
#[test]
fn a_bare_yes_without_an_open_question_does_nothing() {
    for (lang, first, yes_words) in
        [("en", "turn on the hall light", ["yes", "ok", "okay"]), ("de", "Mach das Licht im Flur an", ["ja", "ok", "okay"])]
    {
        let home = test_home(lang);
        for yes in yes_words {
            let mut talk = Talk::new(lang, &home);
            acts_on(&mut talk, first, &["HassTurnOn light.hall_light"]);
            does_nothing(&mut talk, yes);
        }
    }
}

#[test]
fn a_yes_after_the_question_was_answered_does_nothing() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home);
    asks_first(&mut talk, "unlock the front door");
    acts_on(&mut talk, "yes", &["HassTurnOff lock.front_door"]);
    does_nothing(&mut talk, "yes");
}

/// S-18: "movie" matched inside any sentence that contained it.
#[test]
fn a_routine_phrase_is_the_whole_utterance() {
    let home = test_home("en");
    let routines = [routine("wind down", "script.evening")];
    for text in ["wind down", "Wind down, please!", "can you wind down"] {
        let out = say_with(&mut Talk::new("en", &home), &routines, text);
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {}", describe(&out));
        assert_eq!(effects(&out, &home), set(&["HassTurnOn script.evening"]), "{text}: {}", describe(&out));
    }
    for text in [
        "when do the kids wind down",
        "turn on the hall light before we wind down",
        "do not wind down yet",
        "is it time to wind down",
        "wind downstairs",
    ] {
        let out = say_with(&mut Talk::new("en", &home), &routines, text);
        assert!(!effects(&out, &home).contains("HassTurnOn script.evening"), "{text}: {}", describe(&out));
    }
}

#[test]
fn a_german_routine_phrase_is_the_whole_utterance() {
    let home = test_home("de");
    let routines = [routine("Feierabend", "script.evening")];
    for text in ["Feierabend", "Feierabend bitte"] {
        let out = say_with(&mut Talk::new("de", &home), &routines, text);
        assert_eq!(effects(&out, &home), set(&["HassTurnOn script.evening"]), "{text}: {}", describe(&out));
    }
    for text in ["Wann hat Papa Feierabend", "Mach das Licht im Flur an wenn Feierabend ist", "Feierabendbier"] {
        let out = say_with(&mut Talk::new("de", &home), &routines, text);
        assert!(!effects(&out, &home).contains("HassTurnOn script.evening"), "{text}: {}", describe(&out));
    }
}

fn starts(out: &ParseOutcome, script: &str) -> bool {
    let named =
        |plan: &IntentPlan| plan.intents().iter().any(|intent| intent.name == "HassTurnOn" && intent.slot("entity_id") == Some(script));
    matches!(out.decision, ParseDecision::Execute) && out.plan.as_ref().is_some_and(named)
}

/// The operator chose the script in Klar, so a routine starts it whether or not Home
/// Assistant exposes it to Assist, as it did before routines met the safety decision.
/// Speaking the script's own name still needs the exposure.
#[test]
fn a_routine_starts_the_script_the_operator_named_even_when_it_is_not_exposed() {
    let home = test_home("en");
    let routines = [routine("secret mode", "script.hidden"), routine("ghost mode", "script.missing")];
    for (text, script) in [("secret mode", "script.hidden"), ("ghost mode", "script.missing")] {
        let out = say_with(&mut Talk::new("en", &home), &routines, text);
        assert!(starts(&out, script), "{text}: {}", describe(&out));
    }
    let out = say_with(&mut Talk::new("en", &home), &routines, "run the hidden routine");
    assert!(!starts(&out, "script.hidden"), "{}", describe(&out));
}

/// The exemption covers the exposure check only: the user's rules still decide.
#[test]
fn rules_still_govern_a_routine_script_that_is_not_exposed() {
    let home = test_home("en");
    let routines = [routine("secret mode", "script.hidden")];
    let mut talk = Talk::new("en", &home);
    talk.rules = vec![rule("script.hidden", PolicyEffect::Block)];
    let out = say_with(&mut talk, &routines, "secret mode");
    assert!(matches!(out.decision, ParseDecision::Reject { reason: RejectReason::Unsafe }), "{}", describe(&out));

    let mut talk = Talk::new("en", &home);
    talk.rules = vec![rule("script.hidden", PolicyEffect::Confirm)];
    let out = say_with(&mut talk, &routines, "secret mode");
    assert!(matches!(out.decision, ParseDecision::Confirm { .. }), "{}", describe(&out));
    assert!(out.plan.is_none());
    let out = say_with(&mut talk, &routines, "yes");
    assert!(starts(&out, "script.hidden"), "{}", describe(&out));
}

#[test]
fn a_block_rule_stops_a_routine_and_a_confirm_rule_asks() {
    let home = test_home("en");
    let routines = [routine("wind down", "script.evening")];
    let mut talk = Talk::new("en", &home);
    talk.rules = vec![rule("script.evening", PolicyEffect::Block)];
    let out = say_with(&mut talk, &routines, "wind down");
    assert!(matches!(out.decision, ParseDecision::Reject { reason: RejectReason::Unsafe }), "{}", describe(&out));

    let mut talk = Talk::new("en", &home);
    talk.rules = vec![rule("script.evening", PolicyEffect::Confirm)];
    let out = say_with(&mut talk, &routines, "wind down");
    assert!(matches!(out.decision, ParseDecision::Confirm { .. }), "{}", describe(&out));
    assert!(out.plan.is_none());
    let out = say_with(&mut talk, &routines, "yes");
    assert_eq!(effects(&out, &home), set(&["HassTurnOn script.evening"]), "{}", describe(&out));
}

fn phrase_script(phrase: &str, script: &str) -> PolicyRule {
    PolicyRule {
        id: "phrase-script".into(),
        enabled: true,
        label: phrase.into(),
        when: PolicyMatch { phrase: Some(phrase.into()), ..PolicyMatch::default() },
        effect: PolicyEffect::Script,
        prefer: None,
        payload: Some(script.into()),
    }
}

#[test]
fn a_rule_script_runs_and_keeps_its_trace() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home);
    talk.rules = vec![phrase_script("time for bed", "script.evening")];
    let out = talk.say("time for bed");
    assert_eq!(effects(&out, &home), set(&["HassTurnOn script.evening"]), "{}", describe(&out));
    let trace = out.policy_trace.as_ref().expect("policy trace");
    assert_eq!(trace.hit.as_deref(), Some("script"));
    assert_eq!(trace.matched_rule.as_deref(), Some("phrase-script"));

    // The rule names the script, so it starts without being exposed to Assist.
    for script in ["script.hidden", "script.missing"] {
        let mut talk = Talk::new("en", &home);
        talk.rules = vec![phrase_script("time for bed", script)];
        let out = talk.say("time for bed");
        assert!(starts(&out, script), "{script}: {}", describe(&out));
    }
}

#[test]
fn the_clock_question_is_not_any_sentence_with_what_time() {
    for (lang, clock, other) in
        [("en", "what time is it", "what time is my appointment"), ("de", "Wie spät ist es", "Wie spät ist mein Termin")]
    {
        let home = test_home(lang);
        let out = Talk::new(lang, &home).say(clock);
        assert_eq!(out.decision, ParseDecision::Chat, "{clock}: {}", describe(&out));
        assert!(out.speech.contains(':'), "{clock}: {}", out.speech);
        let out = Talk::new(lang, &home).say(other);
        assert!(!out.speech.contains(':'), "{other} is not the clock: {}", out.speech);
    }
}

#[test]
fn call_it_a_day_teaches_no_alias() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home);
    acts_on(&mut talk, "turn on the hall light", &["HassTurnOn light.hall_light"]);
    does_nothing(&mut talk, "call it a day");
    assert!(talk.session.pending_teach.is_none(), "{:?}", talk.session.pending_teach);

    let out = talk.say("call it reading lamp");
    assert_eq!(out.decision, ParseDecision::Chat, "{}", describe(&out));
    assert_eq!(talk.session.pending_teach, Some(("light.hall_light".into(), "reading lamp".into())));
}

#[test]
fn undo_is_a_whole_utterance_not_a_word_in_one() {
    let home = test_home("en");
    for text in ["undo the office light", "do not undo that", "what does undo do"] {
        let mut talk = Talk::new("en", &home);
        acts_on(&mut talk, "turn on the hall light", &["HassTurnOn light.hall_light"]);
        let out = talk.say(text);
        assert!(!effects(&out, &home).contains("HassTurnOff light.hall_light"), "{text}: {}", describe(&out));
    }
}
