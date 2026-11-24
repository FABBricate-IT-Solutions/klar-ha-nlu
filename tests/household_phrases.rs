//! Teach and undo change the house, so a pack may only carry phrases a person wrote for them.
//! The generator used to build them from the pack's own verbs ("turn-off door", "set door"),
//! which are ordinary commands. Hungarian particle verbs: the particle names the direction.
mod common;

use common::home::{area, ent, home, Talk};
use klar_nlu::lang::{LangId, VerbKind};
use klar_nlu::types::{HomeGraph, ParseDecision, ParseOutcome};

fn french_home() -> HomeGraph {
    home(
        vec![area("salon", "Salon"), area("garage", "Garage"), area("entree", "Entrée")],
        vec![
            ent("light.salon", "Lumière du salon", "salon"),
            ent("cover.garage", "Porte du garage", "garage"),
            ent("switch.porte", "Porte", "entree"),
        ],
        &[],
    )
}

fn hungarian_home() -> HomeGraph {
    home(
        vec![area("nappali", "Nappali"), area("konyha", "Konyha")],
        vec![
            ent("light.nappali", "Nappali lámpa", "nappali"),
            ent("light.konyha", "Konyha lámpa", "konyha"),
            ent("switch.ajto", "Ajtó", "konyha"),
        ],
        &[],
    )
}

/// Every entity the last turn acts on now, or would act on after a "yes", with the intent name.
fn touched(out: &ParseOutcome, talk: &Talk) -> Vec<String> {
    let plan = match &out.decision {
        ParseDecision::Confirm { .. } => talk.session.pending_confirm().map(|pending| &pending.plan),
        _ => out.plan.as_ref(),
    };
    plan.map(|plan| plan.intents())
        .unwrap_or_default()
        .iter()
        .map(|intent| format!("{} {}", intent.name, intent.slot("entity_id").unwrap_or("-")))
        .collect()
}

/// L-02: "turn-off door" was the generated undo phrase, so it reversed the last action.
#[test]
fn switching_a_door_off_does_not_undo_the_last_action() {
    for (lang, home, first, second, lamp) in [
        ("fr", french_home(), "Allume la lumière du salon", "Éteins la porte", "light.salon"),
        ("hu", hungarian_home(), "Kapcsold be a nappali lámpát", "Oltsd az ajtó", "light.nappali"),
    ] {
        let mut talk = Talk::new(lang, &home);
        let out = talk.say(first);
        assert_eq!(touched(&out, &talk), [format!("HassTurnOn {lamp}")], "{lang} {first:?}");
        let out = talk.say(second);
        let steps = touched(&out, &talk);
        assert!(!steps.iter().any(|step| step.ends_with(lamp)), "{lang} {second:?} reversed the lamp: {:?} {steps:?}", out.decision);
    }
}

/// L-02: "set door" was the generated teach prefix, so setting a garage door renamed the last device.
#[test]
fn setting_a_door_is_not_teaching_a_name() {
    let home = french_home();
    let mut talk = Talk::new("fr", &home);
    talk.say("Allume la lumière du salon");
    let out = talk.say("Mets la porte du garage à 50");
    assert!(!matches!(out.decision, ParseDecision::Chat), "answered as a taught name: {:?}", out.speech);
    let steps = touched(&out, &talk);
    assert!(steps.len() == 1 && steps[0].ends_with("cover.garage"), "{:?} {steps:?}", out.decision);
}

/// A phrase that starts with one of the pack's own action verbs is a command, not a household phrase.
#[test]
fn no_teach_or_undo_phrase_starts_with_an_action_verb() {
    let acts = |kind: VerbKind| {
        matches!(
            kind,
            VerbKind::On
                | VerbKind::Off
                | VerbKind::Open
                | VerbKind::OpenDoor
                | VerbKind::Close
                | VerbKind::Set
                | VerbKind::Switch
                | VerbKind::Toggle
        )
    };
    for lang in LangId::all() {
        let pack = lang.pack();
        for phrase in pack.household.teach.iter().chain(pack.household.undo) {
            let first = phrase.split_whitespace().next().unwrap_or("");
            let verb = pack.verbs.iter().find(|(word, kind)| *word == first && acts(*kind));
            assert!(verb.is_none(), "{} household phrase {phrase:?} starts with the action verb {verb:?}", lang.code());
        }
    }
}

/// L-03: "kapcsold ki" (off) switched on, because "kapcsold" alone counted as "on".
#[test]
fn hungarian_particle_names_the_direction() {
    let home = hungarian_home();
    for (text, want) in [
        ("Kapcsold be a nappali lámpát", "HassTurnOn light.nappali"),
        ("Kapcsold ki a nappali lámpát", "HassTurnOff light.nappali"),
        ("Kapcsold fel a konyha lámpát", "HassTurnOn light.konyha"),
        ("Kapcsold le a konyha lámpát", "HassTurnOff light.konyha"),
        ("Oltsd le a konyha lámpát", "HassTurnOff light.konyha"),
        ("Nappali lámpa be", "HassTurnOn light.nappali"),
        ("Nappali lámpa ki", "HassTurnOff light.nappali"),
        ("Kapcsold a nappali lámpát", "HassTurnOn light.nappali"),
    ] {
        let mut talk = Talk::new("hu", &home);
        let out = talk.say(text);
        assert!(matches!(out.decision, ParseDecision::Execute), "{text:?}: {:?}", out.decision);
        assert_eq!(touched(&out, &talk), [want], "{text:?}");
    }
    // "van" (is) makes it a question about the state: the particle must not switch anything.
    for text in ["Ki van kapcsolva a nappali lámpa?", "Be van kapcsolva a nappali lámpa?"] {
        let mut talk = Talk::new("hu", &home);
        let out = talk.say(text);
        assert_eq!(touched(&out, &talk), ["HassGetState light.nappali"], "{text:?}: {:?}", out.decision);
    }
}
