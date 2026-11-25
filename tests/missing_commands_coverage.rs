//! Package 2.7 — coverage matrix for cover halfway/all/stop, vacuum pause/stop/home,
//! button press, and alarm arm/disarm. Every row is an intent or a clear Unsupported
//! reject — never GetState standing in for an unmet command (no „unbekannt“).

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::types::{HomeGraph, ParseDecision, RejectReason};
use std::collections::BTreeSet;

fn matrix_home() -> HomeGraph {
    home(
        vec![area("office", "Office"), area("living_room", "Living Room"), area("bedroom", "Bedroom"), area("kitchen", "Kitchen")],
        vec![
            ent("cover.office_blinds", "Office Blinds", "office"),
            ent("cover.living_room_blinds", "Living Room Blinds", "living_room"),
            ent("cover.bedroom_blinds", "Bedroom Blinds", "bedroom"),
            ent("vacuum.robby", "Robby", "living_room"),
            ent("button.doorbell", "Doorbell", "kitchen"),
            ent("alarm_control_panel.home", "Home Alarm", "living_room"),
            ent("light.office_light", "Office Light", "office"),
            ent("weather.home", "Home", ""),
        ],
        &[],
    )
}

fn de_home() -> HomeGraph {
    home(
        vec![area("buero", "Büro"), area("wohnzimmer", "Wohnzimmer")],
        vec![
            ent("cover.buero_rollo", "Büro Rollo", "buero"),
            ent("cover.wohnzimmer_rollo", "Wohnzimmer Rollo", "wohnzimmer"),
            ent("vacuum.sauger", "Staubsauger", "wohnzimmer"),
            ent("button.klingel", "Klingel", "wohnzimmer"),
            ent("alarm_control_panel.alarmanlage", "Alarmanlage", "wohnzimmer"),
        ],
        &[],
    )
}

fn executed(home: &HomeGraph, lang: &str, text: &str) -> BTreeSet<String> {
    let out = Talk::new(lang, home).without_confirm().say(text);
    assert!(
        matches!(out.decision, ParseDecision::Execute | ParseDecision::Confirm { .. }),
        "[{lang}] {text}: expected execute/confirm, got {}",
        describe(&out)
    );
    effects(&out, home)
}

fn unsupported(home: &HomeGraph, lang: &str, text: &str) {
    let out = Talk::new(lang, home).say(text);
    assert!(
        matches!(out.decision, ParseDecision::Reject { reason: RejectReason::Unsupported }),
        "[{lang}] {text}: expected Unsupported, got {}",
        describe(&out)
    );
}

fn asks_state(home: &HomeGraph, lang: &str, text: &str, entity: &str) {
    let out = Talk::new(lang, home).say(text);
    let ok = matches!(out.decision, ParseDecision::Execute)
        && out
            .plan
            .as_ref()
            .is_some_and(|plan| plan.intents().iter().any(|i| i.name == "HassGetState" && i.slot("entity_id") == Some(entity)));
    assert!(ok, "[{lang}] {text}: expected GetState {entity}, got {}", describe(&out));
}

#[test]
fn p21_cover_halfway_all_room_and_stop() {
    let home = matrix_home();
    assert_eq!(
        executed(&home, "en", "open the office blinds halfway"),
        BTreeSet::from(["HassSetPosition cover.office_blinds position=50".into()])
    );
    assert_eq!(
        executed(&home, "en", "open the office blinds half way"),
        BTreeSet::from(["HassSetPosition cover.office_blinds position=50".into()])
    );
    assert_eq!(
        executed(&home, "en", "set the office blinds to halfway"),
        BTreeSet::from(["HassSetPosition cover.office_blinds position=50".into()])
    );
    assert_eq!(executed(&home, "en", "close the blinds in the office"), BTreeSet::from(["HassTurnOff cover.office_blinds".into()]));
    let all_closed = BTreeSet::from([
        "HassTurnOff cover.bedroom_blinds".into(),
        "HassTurnOff cover.living_room_blinds".into(),
        "HassTurnOff cover.office_blinds".into(),
    ]);
    assert_eq!(executed(&home, "en", "close all the blinds"), all_closed);
    assert_eq!(executed(&home, "en", "close all blinds"), all_closed.clone());
    let all_open = BTreeSet::from([
        "HassTurnOn cover.bedroom_blinds".into(),
        "HassTurnOn cover.living_room_blinds".into(),
        "HassTurnOn cover.office_blinds".into(),
    ]);
    assert_eq!(executed(&home, "en", "open all the blinds"), all_open);
    assert_eq!(executed(&home, "en", "stop the office blinds"), BTreeSet::from(["HassStopMoving cover.office_blinds".into()]));
    let all_stop = BTreeSet::from([
        "HassStopMoving cover.bedroom_blinds".into(),
        "HassStopMoving cover.living_room_blinds".into(),
        "HassStopMoving cover.office_blinds".into(),
    ]);
    assert_eq!(executed(&home, "en", "stop the blinds"), all_stop);
}

#[test]
fn p21_cover_halfway_german() {
    let home = de_home();
    assert_eq!(
        executed(&home, "de", "mach das Büro Rollo halb auf"),
        BTreeSet::from(["HassSetPosition cover.buero_rollo position=50".into()])
    );
}

#[test]
fn p30_vacuum_pause_stop_reject_home_docks() {
    let home = matrix_home();
    assert_eq!(executed(&home, "en", "start the vacuum"), BTreeSet::from(["HassVacuumStart vacuum.robby".into()]));
    assert_eq!(executed(&home, "en", "send robby home"), BTreeSet::from(["HassVacuumReturnToBase vacuum.robby".into()]));
    assert_eq!(executed(&home, "en", "send the vacuum home"), BTreeSet::from(["HassVacuumReturnToBase vacuum.robby".into()]));
    assert_eq!(executed(&home, "en", "dock robby"), BTreeSet::from(["HassVacuumReturnToBase vacuum.robby".into()]));
    assert_eq!(executed(&home, "en", "turn off the vacuum"), BTreeSet::from(["HassVacuumReturnToBase vacuum.robby".into()]));
    for text in ["pause the vacuum", "stop the vacuum", "pause robby", "stop robby"] {
        unsupported(&home, "en", text);
    }
}

#[test]
fn p36_button_press_and_alarm_arm() {
    let home = matrix_home();
    assert_eq!(executed(&home, "en", "press the doorbell"), BTreeSet::from(["HassTurnOn button.doorbell".into()]));
    assert_eq!(executed(&home, "en", "press the doorbell button"), BTreeSet::from(["HassTurnOn button.doorbell".into()]));
    for text in ["arm the alarm", "arm the home alarm", "disarm the alarm", "arm alarm away"] {
        unsupported(&home, "en", text);
    }
    asks_state(&home, "en", "is the alarm armed", "alarm_control_panel.home");
}

#[test]
fn p36_german_button_and_alarm() {
    let home = de_home();
    assert_eq!(executed(&home, "de", "drücke die Klingel"), BTreeSet::from(["HassTurnOn button.klingel".into()]));
    unsupported(&home, "de", "Alarmanlage scharf schalten");
    unsupported(&home, "de", "scharf die Alarmanlage");
}
