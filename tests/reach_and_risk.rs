//! A plan is judged by the devices it reaches (plan package 1.3, findings S-07, S-10, S-17).
//!
//! Moving a garage door, a gate, a door or a lock asks first, whichever way it moves and
//! however the sentence names it. A room or floor step always carries a domain, also in a
//! follow-up. A device marked "ignore this target" is never part of a plan.
use std::collections::HashMap;

mod common;
use common::acts::{changes_state, intended_changes, reach, show};
use common::collision_home::collision_home;
use common::home::{area, describe, ent, home, Talk};
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{CustomSentence, HomeGraph, ParseDecision, ParseOutcome, Settings};

/// The collision home without its scenes and scripts. "Open Garage" is a script there and
/// competes with the garage door by name; that clash belongs to another test.
fn plain_home(lang: &str) -> HomeGraph {
    let mut graph = collision_home(lang);
    graph.entities.retain(|entity| !matches!(entity.domain.as_str(), "scene" | "script"));
    graph
}

fn ignoring(mut graph: HomeGraph, ids: &[&str]) -> HomeGraph {
    for id in ids {
        let entity = graph.entities.iter_mut().find(|entity| entity.entity_id == *id).unwrap_or_else(|| panic!("no {id} in the home"));
        entity.tags.push("nlu_ignore".into());
    }
    graph
}

/// Every device the last turn would change, now or after one more word from the user.
fn reached(talk: &Talk<'_>, out: &ParseOutcome) -> Vec<String> {
    intended_changes(out, &talk.session, talk.home).iter().flat_map(|step| reach(step, talk.home)).collect()
}

fn assert_asks_before(lang: &str, home: &HomeGraph, text: &str, device: &str) {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(text);
    let devices = reached(&talk, &out);
    assert!(
        matches!(out.decision, ParseDecision::Confirm { .. }) && devices == [device],
        "[{lang}] {text:?} must ask before it moves {device}, got {} reaching {devices:?}",
        describe(&out)
    );
}

fn assert_runs(lang: &str, home: &HomeGraph, text: &str, devices: &[&str]) {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(text);
    let got = reached(&talk, &out);
    assert!(
        matches!(out.decision, ParseDecision::Execute) && got == devices,
        "[{lang}] {text:?} must run on {devices:?}, got {} reaching {got:?}",
        describe(&out)
    );
}

fn assert_never_reaches(lang: &str, home: &HomeGraph, turns: &[&str], devices: &[&str]) {
    let mut talk = Talk::new(lang, home);
    for text in turns {
        let out = talk.say(text);
        let got = reached(&talk, &out);
        for device in devices {
            assert!(!got.iter().any(|id| id == device), "[{lang}] {turns:?}: {text:?} reaches {device}: {}", describe(&out));
        }
    }
}

// S-10: opening and toggling asks, not only closing.

#[test]
fn moving_the_garage_door_asks_in_either_direction() {
    let en = plain_home("en");
    for text in ["open the garage door", "close the garage door", "toggle the garage door"] {
        assert_asks_before("en", &en, text, "cover.garage_door");
    }
    let de = plain_home("de");
    for text in ["mach das Garagentor auf", "öffne das Garagentor", "schließe das Garagentor", "mach das Garagentor zu"] {
        assert_asks_before("de", &de, text, "cover.garagentor");
    }
}

/// With the script "Open Garage" next to the door the engine asks which one is meant
/// instead. Either way the sentence does not open the door on its own.
#[test]
fn the_garage_door_is_not_opened_unasked_next_to_a_script() {
    for (lang, text, door) in [("en", "open the garage door", "cover.garage_door"), ("de", "öffne das Garagentor", "cover.garagentor")] {
        let graph = collision_home(lang);
        let mut talk = Talk::new(lang, &graph);
        let out = talk.say(text);
        let asks = matches!(out.decision, ParseDecision::Confirm { .. } | ParseDecision::Clarify { .. });
        assert!(asks && reached(&talk, &out).iter().any(|id| id == door), "[{lang}] {text:?}: {}", describe(&out));
    }
}

#[test]
fn a_blind_opens_without_a_question() {
    let en = plain_home("en");
    assert_runs("en", &en, "open the blinds in the bedroom", &["cover.bedroom_blinds"]);
    let de = plain_home("de");
    assert_runs("de", &de, "mach das Rollo im Schlafzimmer auf", &["cover.schlafzimmer_rollo"]);
}

/// The registry's device class decides where it is known; the name is only the fallback.
#[test]
fn the_device_class_says_what_is_an_entrance() {
    let rooms = vec![area("yard", "Yard"), area("garage", "Garage")];
    let devices = vec![
        ent("cover.big_roller", "Big Roller", "yard"),
        ent("cover.driveway", "Driveway", "yard"),
        ent("cover.side_entry", "Side Entry", "yard"),
        ent("cover.garage_skylight", "Garage Skylight", "garage"),
        ent("cover.awning", "Awning", "yard"),
    ];
    let mut graph = home(rooms, devices, &[]);
    let classes = [
        ("cover.big_roller", "garage"),
        ("cover.driveway", "gate"),
        ("cover.side_entry", "door"),
        ("cover.garage_skylight", "window"),
        ("cover.awning", "awning"),
    ];
    graph.device_classes = classes.iter().map(|(id, class)| (id.to_string(), class.to_string())).collect::<HashMap<_, _>>();

    assert_asks_before("en", &graph, "open the big roller", "cover.big_roller");
    assert_asks_before("en", &graph, "open the driveway", "cover.driveway");
    assert_asks_before("en", &graph, "open the side entry", "cover.side_entry");
    // "Garage" in the name, but the registry says it is a window.
    assert_runs("en", &graph, "open the garage skylight", &["cover.garage_skylight"]);
    assert_runs("en", &graph, "open the awning", &["cover.awning"]);
}

/// The class is not a word of any language: the same home asks in every one of them.
#[test]
fn a_gate_asks_in_other_languages() {
    let cases = [
        ("fr", "Portail", "ouvre le portail"),
        ("es", "Portón", "abre el portón"),
        ("it", "Cancello", "apri il cancello"),
        ("nl", "Poort", "open de poort"),
        ("pt", "Portão", "abre o portão"),
        ("sv", "Grind", "öppna grinden"),
        ("pl", "Brama", "otwórz bramę"),
    ];
    let mut asked = Vec::new();
    for (lang, name, text) in cases {
        let mut graph = home(vec![area("yard", "Patio")], vec![ent("cover.gate", name, "yard")], &[]);
        graph.device_classes.insert("cover.gate".into(), "gate".into());
        let mut talk = Talk::new(lang, &graph);
        let out = talk.say(text);
        assert!(
            !matches!(out.decision, ParseDecision::Execute) || reached(&talk, &out).is_empty(),
            "[{lang}] {text:?}: {}",
            describe(&out)
        );
        if matches!(out.decision, ParseDecision::Confirm { .. }) {
            asked.push(lang);
        }
    }
    // Where the sentence is understood at all it must end in the question.
    for lang in ["fr", "es", "it", "nl", "pt"] {
        assert!(asked.contains(&lang), "[{lang}] opening the gate no longer asks; asked: {asked:?}");
    }
}

// S-07: a room named instead of its door, and steps without a domain.

#[test]
fn closing_the_garage_means_its_door_and_asks() {
    assert_asks_before("en", &plain_home("en"), "close the garage", "cover.garage_door");
    assert_asks_before("de", &plain_home("de"), "mach die Garage zu", "cover.garagentor");
}

#[test]
fn switching_the_garage_off_means_its_lights() {
    let lights = ["light.garage_light", "light.workbench"];
    assert_runs("en", &plain_home("en"), "turn off the garage", &lights);
    assert_runs("de", &plain_home("de"), "schalte die Garage aus", &["light.garagenlicht", "light.werkbank"]);
}

#[test]
fn a_follow_up_keeps_the_domain() {
    let en = plain_home("en");
    let mut talk = Talk::new("en", &en);
    talk.say("turn on the lights in the garage");
    let out = talk.say("turn them off");
    assert_eq!(reached(&talk, &out), ["light.garage_light", "light.workbench"], "{}", describe(&out));
    assert!(matches!(out.decision, ParseDecision::Execute), "{}", describe(&out));

    let de = plain_home("de");
    let mut talk = Talk::new("de", &de);
    talk.say("mach das Licht in der Garage an");
    let out = talk.say("mach sie aus");
    assert_eq!(reached(&talk, &out), ["light.garagenlicht", "light.werkbank"], "{}", describe(&out));
}

/// No sentence about a room or a floor ends in a step that Home Assistant would run on
/// every device there.
#[test]
fn no_room_or_floor_step_goes_without_a_domain() {
    let sentences: &[(&str, &[&str])] = &[
        (
            "en",
            &[
                "turn off everything downstairs",
                "switch off the upstairs",
                "turn off the kitchen",
                "turn on the garage",
                "turn off everything in the living room",
                "turn off everything upstairs",
                "open everything in the garage",
                "close the garage",
                "turn it all off in the hall",
            ],
        ),
        (
            "de",
            &[
                "schalte unten alles aus",
                "mach oben alles aus",
                "schalte die Küche aus",
                "mach die Garage an",
                "schalte alles im Flur aus",
                "mach die Garage zu",
            ],
        ),
    ];
    for (lang, texts) in sentences {
        let graph = plain_home(lang);
        for text in *texts {
            let mut talk = Talk::new(lang, &graph);
            let out = talk.say(text);
            for step in intended_changes(&out, &talk.session, &graph) {
                let bounded = step.slot("entity_id").is_some() || step.slot("domain").is_some();
                assert!(bounded, "[{lang}] {text:?} plans {} without a domain: {}", show(&step), describe(&out));
            }
            let devices = reached(&talk, &out);
            let entrance = devices.iter().any(|id| id.starts_with("lock.") || id.starts_with("cover.garag"));
            assert!(
                !entrance || !matches!(out.decision, ParseDecision::Execute),
                "[{lang}] {text:?} moves an entrance unasked: {}",
                describe(&out)
            );
        }
    }
}

fn taught(phrase: &str, slots: &[(&str, &str)]) -> [CustomSentence; 1] {
    [CustomSentence {
        phrase: phrase.into(),
        intent: "HassTurnOff".into(),
        slots: slots.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect(),
    }]
}

/// The rule holds for every way a plan is made, also for a sentence the user taught.
#[test]
fn a_taught_room_step_needs_a_domain() {
    let en = plain_home("en");
    let mut session = Session::new();
    let out = parse("garage shutdown", &en, &mut session, &taught("garage shutdown", &[("area", "garage")]), &Settings::pinned("en"));
    assert!(matches!(out.decision, ParseDecision::Reject { .. }), "{}", describe(&out));
    assert!(intended_changes(&out, &session, &en).is_empty(), "{}", describe(&out));

    let mut session = Session::new();
    let custom = taught("garage shutdown", &[("area", "garage"), ("domain", "light")]);
    let out = parse("garage shutdown", &en, &mut session, &custom, &Settings::pinned("en"));
    let devices: Vec<String> = intended_changes(&out, &session, &en).iter().flat_map(|step| reach(step, &en)).collect();
    assert_eq!(devices, ["light.garage_light", "light.workbench"], "{}", describe(&out));
}

// S-17: "ignore this target" holds on every path into a plan.

#[test]
fn an_ignored_device_is_not_the_fallback_for_its_kind() {
    let en = ignoring(plain_home("en"), &["fan.bedroom_fan"]);
    for text in ["turn on the fan", "turn on the bedroom fan", "set the fan to 50 percent", "turn off the fan in the bedroom"] {
        assert_never_reaches("en", &en, &[text], &["fan.bedroom_fan"]);
    }
    let en = ignoring(plain_home("en"), &["media_player.living_room_tv"]);
    assert_never_reaches("en", &en, &["turn on the tv"], &["media_player.living_room_tv"]);
    assert_never_reaches("en", &en, &["turn off the tv in the living room"], &["media_player.living_room_tv"]);
    let en = ignoring(plain_home("en"), &["climate.bedroom"]);
    assert_never_reaches("en", &en, &["set the temperature in the bedroom to 21"], &["climate.bedroom"]);
    let en = ignoring(plain_home("en"), &["cover.garage_door"]);
    assert_never_reaches("en", &en, &["close the garage"], &["cover.garage_door"]);
    assert_never_reaches("en", &en, &["open the garage door"], &["cover.garage_door"]);

    let de = ignoring(plain_home("de"), &["fan.schlafzimmer_ventilator", "cover.garagentor"]);
    for text in ["mach den Ventilator an", "schalte den Ventilator im Schlafzimmer ein", "mach das Garagentor auf", "mach die Garage zu"] {
        assert_never_reaches("de", &de, &[text], &["fan.schlafzimmer_ventilator", "cover.garagentor"]);
    }
}

#[test]
fn an_ignored_device_is_taken_out_of_its_room() {
    let en = ignoring(plain_home("en"), &["light.floor_lamp"]);
    assert_runs("en", &en, "turn off the lights in the hall", &["light.hall_light"]);
    assert_never_reaches("en", &en, &["turn off all the lights"], &["light.floor_lamp"]);
    assert_never_reaches("en", &en, &["turn off the lights downstairs"], &["light.floor_lamp"]);
    assert_never_reaches("en", &en, &["turn on the lights in the hall", "turn them off"], &["light.floor_lamp"]);
    assert_never_reaches("en", &en, &["turn on the floor lamp"], &["light.floor_lamp"]);

    let de = ignoring(plain_home("de"), &["light.stehlampe"]);
    assert_runs("de", &de, "mach das Licht im Flur aus", &["light.flur_licht"]);
    assert_never_reaches("de", &de, &["schalte alle Lichter aus"], &["light.stehlampe"]);
    assert_never_reaches("de", &de, &["mach das Licht im Flur an", "mach es aus"], &["light.stehlampe"]);
}

/// The mark is about what Klar may pick on its own. A sentence the user taught for that
/// very device says so explicitly and keeps working.
#[test]
fn a_taught_sentence_may_name_an_ignored_device() {
    let en = ignoring(plain_home("en"), &["light.floor_lamp"]);
    let mut session = Session::new();
    let custom = taught("reading time is over", &[("entity_id", "light.floor_lamp")]);
    let out = parse("reading time is over", &en, &mut session, &custom, &Settings::pinned("en"));
    let steps = intended_changes(&out, &session, &en);
    assert!(steps.iter().all(changes_state) && steps.len() == 1, "{}", describe(&out));
    assert_eq!(reach(&steps[0], &en), ["light.floor_lamp"], "{}", describe(&out));
}
