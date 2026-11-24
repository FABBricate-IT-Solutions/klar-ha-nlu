//! Only a positive, present-tense command may change a device (plan package 1.2).
//!
//! A negated command, a question, a report of what happened, a condition and a schedule
//! must end in a state query or a rejection. Polite requests, exceptions, names that
//! contain such words and a command next to a question still act. English and German run
//! in the collision home; the other languages run in the built-in demo home.
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

mod common;
use common::acts::{intended_changes, reach, show};
use common::collision_home::collision_home;
use common::home::{area, describe, ent, home, Talk};
use klar_nlu::home::default_home;
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{CustomSentence, HomeGraph, ParseDecision, ParseOutcome, Settings};

/// The outcome and every state change the engine committed to, as text.
fn said(lang: &str, home: &HomeGraph, text: &str) -> (ParseOutcome, Vec<String>) {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(text);
    let changes = intended_changes(&out, &talk.session, home).iter().map(show).collect();
    (out, changes)
}

/// Every device the sentence would change, now or after one more word from the user.
fn touched(lang: &str, home: &HomeGraph, text: &str) -> Vec<String> {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(text);
    intended_changes(&out, &talk.session, home).iter().flat_map(|step| reach(step, home)).collect()
}

fn assert_changes_nothing(lang: &str, home: &HomeGraph, text: &str) {
    let (out, changes) = said(lang, home, text);
    assert!(changes.is_empty(), "[{lang}] {text:?} must not change anything, got {} ({changes:?})", describe(&out));
}

fn assert_rejected(lang: &str, home: &HomeGraph, text: &str) {
    let (out, _) = said(lang, home, text);
    assert!(matches!(out.decision, ParseDecision::Reject { .. }), "[{lang}] {text:?} must be rejected, got {}", describe(&out));
}

/// The sentence reads a state and changes none.
fn assert_reads(lang: &str, home: &HomeGraph, text: &str, slot: (&str, &str)) {
    let (out, changes) = said(lang, home, text);
    let steps = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    let reads = steps.iter().any(|step| step.name == "HassGetState" && step.slot(slot.0) == Some(slot.1));
    assert!(
        matches!(out.decision, ParseDecision::Execute) && reads && changes.is_empty(),
        "[{lang}] {text:?} must read {}={}, got {}",
        slot.0,
        slot.1,
        describe(&out)
    );
}

fn assert_touches(lang: &str, home: &HomeGraph, text: &str, device: &str) {
    let devices = touched(lang, home, text);
    assert!(devices.iter().any(|id| id == device), "[{lang}] {text:?} must still reach {device}, got {devices:?}");
}

#[test]
fn a_negated_command_changes_nothing() {
    let en = collision_home("en");
    for text in [
        "do not unlock the back door",
        "please don't switch on the coffee machine",
        "never open the bedroom blinds",
        "you must not turn off the office light",
        "the garage door should not be opened",
        "no, do not start the fan",
        "turn on the guest lamp, actually don't",
    ] {
        assert_changes_nothing("en", &en, text);
    }
    let de = collision_home("de");
    for text in [
        "schließ die Hintertür nicht auf",
        "bitte nicht die Kaffeemaschine einschalten",
        "auf keinen Fall das Garagentor aufmachen",
        "mach das Büro Licht niemals aus",
        "das Garagentor darf nicht geöffnet werden",
        "kein Licht im Büro anmachen",
    ] {
        assert_changes_nothing("de", &de, text);
    }
}

#[test]
fn a_question_or_a_report_reads_the_state() {
    let en = collision_home("en");
    assert_reads("en", &en, "is the garage door open", ("entity_id", "cover.garage_door"));
    assert_reads("en", &en, "was the garage door opened", ("entity_id", "cover.garage_door"));
    assert_reads("en", &en, "has the front door been locked", ("entity_id", "lock.front_door"));
    assert_reads("en", &en, "the front door was locked", ("entity_id", "lock.front_door"));
    for text in [
        "did someone unlock the back door",
        "who switched on the coffee machine",
        "why did the bedroom blinds open",
        "should I lock the back door",
        "would the fan start",
        "must the office light turn off",
        "I switched off the coffee machine",
        "we already closed the bedroom blinds",
        "somebody turned the oven on",
        "tell me if the oven is switched on",
        "I wonder whether the back door is locked",
        "lock the back door?",
    ] {
        assert_changes_nothing("en", &en, text);
    }
    let de = collision_home("de");
    assert_reads("de", &de, "ist das Garagentor offen", ("entity_id", "cover.garagentor"));
    assert_reads("de", &de, "wurde das Garagentor geöffnet", ("entity_id", "cover.garagentor"));
    for text in [
        "hat jemand die Hintertür aufgeschlossen",
        "wer hat die Kaffeemaschine eingeschaltet",
        "warum ist das Garagentor aufgegangen",
        "soll ich die Hintertür abschließen",
        "ich habe die Kaffeemaschine ausgeschaltet",
        "wir haben das Garagentor schon zugemacht",
        "sag mir ob der Ofen eingeschaltet ist",
        "schließ die Hintertür ab?",
    ] {
        assert_changes_nothing("de", &de, text);
    }
}

#[test]
fn a_window_question_reads_the_window_contacts() {
    let en = collision_home("en");
    for text in ["did I leave any windows open", "have we still got a window open", "are there any windows left open"] {
        assert_reads("en", &en, text, ("device_class", "window"));
    }
    assert_reads("en", &en, "did I leave the kitchen window open", ("area", "kitchen"));
    let de = collision_home("de");
    for text in ["habe ich irgendwo ein Fenster offen gelassen", "sind noch Fenster offen", "steht irgendwo noch ein Fenster offen"] {
        assert_reads("de", &de, text, ("device_class", "window"));
    }
}

#[test]
fn a_condition_or_a_schedule_is_rejected() {
    let en = collision_home("en");
    for text in [
        "switch on the coffee machine in 20 minutes",
        "open the bedroom blinds tomorrow",
        "start the fan at 7",
        "turn off the office light at 11 pm",
        "lock the back door every evening",
        "turn on the guest lamp later",
        "if it gets dark turn on the guest lamp",
        "start the fan unless the bedroom window is open",
        "turn off the oven as soon as the timer rings",
        "turn on the guest lamp and switch off the oven in an hour",
    ] {
        assert_rejected("en", &en, text);
    }
    // "when" also opens a question, so this one may be answered with the state of the door.
    assert_changes_nothing("en", &en, "unlock the back door when I get home");
    let de = collision_home("de");
    for text in [
        "schalte die Kaffeemaschine in 20 Minuten ein",
        "mach morgen das Garagentor auf",
        "schalte den Ventilator um 7 ein",
        "mach das Büro Licht heute Abend aus",
        "schließ jeden Abend die Hintertür ab",
        "mach das Gästelicht später an",
        "wenn es dunkel wird mach das Gästelicht an",
        "falls es regnet mach das Garagentor zu",
        "schalte den Ofen aus sobald der Timer klingelt",
    ] {
        assert_rejected("de", &de, text);
    }
}

#[test]
fn a_polite_request_still_acts() {
    let en = collision_home("en");
    for (text, device) in [
        ("can you switch on the coffee machine?", "switch.coffee_machine"),
        ("could you please start the fan?", "fan.bedroom_fan"),
        ("would you turn off the office light", "light.office_light"),
        ("I'd like the coffee machine on", "switch.coffee_machine"),
        ("let's switch on the coffee machine", "switch.coffee_machine"),
        ("switch on the coffee machine if you don't mind", "switch.coffee_machine"),
        ("if it's not too much trouble start the fan", "fan.bedroom_fan"),
        ("pull the bedroom blinds closed", "cover.bedroom_blinds"),
        ("flip the back door locked", "lock.back_door"),
    ] {
        assert_touches("en", &en, text, device);
    }
    let de = collision_home("de");
    for (text, device) in [
        ("kannst du bitte die Kaffeemaschine einschalten", "switch.kaffeemaschine"),
        ("ich möchte die Kaffeemaschine an", "switch.kaffeemaschine"),
        ("schalte die Kaffeemaschine ein wenn es geht", "switch.kaffeemaschine"),
        ("Kaffeemaschine an", "switch.kaffeemaschine"),
    ] {
        assert_touches("de", &de, text, device);
    }
}

#[test]
fn an_exception_is_not_a_refusal() {
    let en = collision_home("en");
    assert_touches("en", &en, "turn off all lights but not the desk lamp", "light.office_light");
    assert_touches("en", &en, "turn off all lights not in the bedroom", "light.office_light");
    let de = collision_home("de");
    assert_touches("de", &de, "alle Lichter aus aber nicht die Stehlampe", "light.buro_licht");
    assert_touches("de", &de, "alle Lichter aus nicht im Schlafzimmer", "light.buro_licht");
}

#[test]
fn a_command_next_to_a_question_still_acts() {
    let en = collision_home("en");
    let (out, changes) = said("en", &en, "switch on the coffee machine and is the garage door open");
    assert_eq!(changes, ["HassTurnOn{entity_id=switch.coffee_machine}"], "{}", describe(&out));
    assert_reads_also(&out, "cover.garage_door");

    let de = collision_home("de");
    let (out, changes) = said("de", &de, "ist das Garagentor offen und schalte die Kaffeemaschine ein");
    assert_eq!(changes, ["HassTurnOn{entity_id=switch.kaffeemaschine}"], "{}", describe(&out));
    assert_reads_also(&out, "cover.garagentor");

    // The forbidden half is dropped, the commanded half stays.
    let devices = touched("de", &de, "schalte die Kaffeemaschine ein und mach das Garagentor nicht auf");
    assert_eq!(devices, ["switch.kaffeemaschine"]);
}

fn assert_reads_also(out: &ParseOutcome, entity: &str) {
    let steps = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    assert!(
        steps.iter().any(|step| step.name == "HassGetState" && step.slot("entity_id") == Some(entity)),
        "the question about {entity} must stay a state query, got {}",
        describe(out)
    );
}

#[test]
fn a_timer_keeps_its_own_time_words() {
    let en = collision_home("en");
    let (out, changes) = said("en", &en, "set a timer for 12 minutes");
    assert_eq!(changes, ["HassStartTimer{minutes=12}"], "{}", describe(&out));
    let (out, changes) = said("en", &en, "switch on the coffee machine and set a timer for 4 minutes");
    assert_eq!(changes, ["HassTurnOn{entity_id=switch.coffee_machine}", "HassStartTimer{minutes=4}"], "{}", describe(&out));
    let de = collision_home("de");
    let (out, changes) = said("de", &de, "stell einen Timer auf 12 Minuten");
    assert_eq!(changes, ["HassStartTimer{minutes=12}"], "{}", describe(&out));
}

#[test]
fn a_number_after_a_clock_word_is_a_value_only_if_the_command_sets_it() {
    let en = collision_home("en");
    let (out, changes) = said("en", &en, "set the bedroom fan at 30");
    assert_eq!(changes, ["HassFanSetSpeed{entity_id=fan.bedroom_fan,percentage=30}"], "{}", describe(&out));
    assert_rejected("en", &en, "start the bedroom fan at 7");
    assert_rejected("en", &en, "turn on the desk lamp at 8");
    assert_rejected("en", &en, "set the bedroom fan to 30 at 7");
    let de = collision_home("de");
    let (out, changes) = said("de", &de, "Heizung im Schlafzimmer um 2 wärmer");
    assert_eq!(changes, ["HassClimateSetTemperature{entity_id=climate.schlafzimmer,temperature=2}"], "{}", describe(&out));
    assert_rejected("de", &de, "mach die Schreibtischlampe um 8 an");
}

/// Names the household chose are not grammar, whatever words they contain.
#[test]
fn mood_words_inside_a_name_do_not_close_the_command() {
    let rooms = || vec![area("den", "Den")];
    let en = home(
        rooms(),
        vec![
            ent("light.tomorrow_lamp", "Tomorrow Lamp", "den"),
            ent("switch.not_yet", "Not Yet Plug", "den"),
            ent("switch.if_then", "If Then Box", "den"),
        ],
        &[],
    );
    assert_touches("en", &en, "turn on the tomorrow lamp", "light.tomorrow_lamp");
    assert_touches("en", &en, "switch on the not yet plug", "switch.not_yet");
    assert_touches("en", &en, "switch on the if then box", "switch.if_then");
    assert_changes_nothing("en", &en, "do not turn on the tomorrow lamp");
    assert_rejected("en", &en, "turn on the tomorrow lamp tomorrow");

    let de = home(rooms(), vec![ent("light.spaeter", "Lampe Später", "den"), ent("switch.falls", "Dose Falls Nicht", "den")], &[]);
    assert_touches("de", &de, "schalte die Lampe Später ein", "light.spaeter");
    assert_touches("de", &de, "schalte die Dose Falls Nicht ein", "switch.falls");
    assert_changes_nothing("de", &de, "schalte die Lampe Später nicht ein");
}

/// A sentence the household taught is taken as taught, even if it reads like a report.
#[test]
fn a_taught_sentence_runs_as_taught() {
    let en = collision_home("en");
    let custom = [CustomSentence {
        phrase: "I am leaving later".into(),
        intent: "HassTurnOn".into(),
        slots: HashMap::from([("entity_id".to_string(), "scene.all_off".to_string())]),
    }];
    let mut session = Session::new();
    let out = parse("I am leaving later", &en, &mut session, &custom, &Settings::pinned("en"));
    let devices: Vec<String> = intended_changes(&out, &session, &en).iter().flat_map(|step| reach(step, &en)).collect();
    assert_eq!(devices, ["scene.all_off"], "{}", describe(&out));
}

fn demo(lang: &str, text: &str) -> (ParseOutcome, Vec<String>) {
    let home = default_home();
    let settings = Settings { languages: vec![lang.into()], ..Settings::default() };
    let mut session = Session::default();
    let out = parse(text, &home, &mut session, &[], &settings);
    let changes = intended_changes(&out, &session, &home).iter().map(show).collect();
    (out, changes)
}

/// The "light on" sentence of every language in Klar's own `assist` dataset.
fn light_on_sentences() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/datasets/assist");
    let mut found = Vec::new();
    for entry in fs::read_dir(&root).unwrap_or_else(|err| panic!("{}: {err}", root.display())) {
        let dir = entry.expect("dataset entry").path();
        let lang = dir.file_name().and_then(|name| name.to_str()).expect("language folder").to_string();
        let text = fs::read_to_string(dir.join("representative.yaml")).unwrap_or_else(|err| panic!("{lang}: {err}"));
        let cases: Vec<serde_yaml::Value> = serde_yaml::from_str(&text).unwrap_or_else(|err| panic!("{lang}: {err}"));
        let sentence = cases
            .iter()
            .find(|case| case["name"].as_str() == Some("light_on"))
            .and_then(|case| case["sentences"][0].as_str())
            .unwrap_or_else(|| panic!("{lang}: no light_on sentence"));
        found.push((lang, sentence.to_string()));
    }
    found.sort();
    found
}

/// Every language closes on a question mark, in the script people type it in.
#[test]
fn every_language_closes_on_a_question_mark() {
    let sentences = light_on_sentences();
    assert!(sentences.len() >= 60, "expected every generated language, found {}", sentences.len());
    for (lang, sentence) in &sentences {
        let (out, changes) = demo(lang, sentence);
        assert!(!changes.is_empty(), "[{lang}] {sentence:?} is a command and must act, got {}", describe(&out));
        for mark in ["?", " ?", "？", "؟"] {
            let asked = format!("{sentence}{mark}");
            let (out, changes) = demo(lang, &asked);
            assert!(changes.is_empty(), "[{lang}] {asked:?} is a question, got {} ({changes:?})", describe(&out));
        }
    }
    let (out, changes) = demo("es", "¿enciende luz salon?");
    assert!(changes.is_empty(), "{}", describe(&out));
    let (out, changes) = demo("el", "ανάψε φως σαλόνι;");
    assert!(changes.is_empty(), "{}", describe(&out));
}

/// The question particle of a language closes the sentence without any mark.
#[test]
fn a_question_particle_closes_the_sentence() {
    for (lang, text) in [
        ("fr", "est-ce que la lumiere salon est allumee"),
        ("fa", "آیا چراغ پذیرایی روشن است"),
        ("zh-CN", "客厅灯打开了吗"),
        ("zh-TW", "客廳燈打開了嗎"),
        ("ko", "거실 불 켜져 있나요"),
        ("th", "เปิดไฟห้องนั่งเล่นไหม"),
        ("vi", "bat bongden phongkhach không"),
        ("pl", "czy swiatlo salon jest wlaczone"),
        ("tr", "isik salon yak mı"),
        ("ja", "リビングの電気ついてますか"),
        ("hi", "क्या बत्ती बैठक चालू"),
    ] {
        let (out, changes) = demo(lang, text);
        assert!(changes.is_empty(), "[{lang}] {text:?} is a question, got {} ({changes:?})", describe(&out));
    }
}

/// A polite request ends in a question mark and is still a command.
#[test]
fn a_polite_request_in_another_language_still_acts() {
    for (lang, text) in [("fr", "peux-tu allumer la lumiere salon ?"), ("nl", "kun je het licht woonkamer aanzetten?")] {
        let (out, changes) = demo(lang, text);
        assert!(!changes.is_empty(), "[{lang}] {text:?} is a request and must act, got {}", describe(&out));
    }
}
