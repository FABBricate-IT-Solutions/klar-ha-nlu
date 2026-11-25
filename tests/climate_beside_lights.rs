//! A thermostat is not a light, and a sentence that names a heating device or a reading
//! next to lights keeps each side with its own device.
use std::collections::BTreeSet;

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::types::{HomeGraph, ParseDecision};

/// Two rooms with a light and a thermostat each, a hall whose devices carry names no
/// word list knows, and a front door.
fn test_home(lang: &str) -> HomeGraph {
    let (light, climate) = if lang == "de" { ("Licht", "Heizung") } else { ("Light", "Thermostat") };
    home(
        vec![area("stube", "Stube"), area("kammer", "Kammer"), area("diele", "Diele")],
        vec![
            ent("light.stube", &format!("Stube {light}"), "stube"),
            ent("light.kammer", &format!("Kammer {light}"), "kammer"),
            ent("light.diele", &format!("Diele {light}"), "diele"),
            ent("climate.stube", &format!("Stube {climate}"), "stube"),
            ent("climate.kammer", &format!("Kammer {climate}"), "kammer"),
            ent("climate.diele", "Diele Radiator", "diele"),
            ent("cover.diele", "Diele Shutter", "diele"),
            ent("lock.front", "Front Door", "diele"),
        ],
        &[],
    )
}

fn does(lang: &str, turns: &[&str], wanted: &[&str]) {
    let home = test_home(lang);
    let mut talk = Talk::new(lang, &home).without_confirm();
    let mut out = None;
    for turn in turns {
        out = Some(talk.say(turn));
    }
    let out = out.expect("at least one turn");
    let wanted: BTreeSet<String> = wanted.iter().map(|step| step.to_string()).collect();
    assert_eq!(out.decision, ParseDecision::Execute, "{lang} {turns:?}: {}", describe(&out));
    assert_eq!(effects(&out, &home), wanted, "{lang} {turns:?}: {}", describe(&out));
}

#[test]
fn a_thermostat_is_not_a_light() {
    let both = ["HassClimateSetTemperature climate.stube temperature=21", "HassClimateSetTemperature climate.kammer temperature=21"];
    does("en", &["set the stube thermostat to 21"], &both[..1]);
    does("en", &["set the stube and kammer thermostats to 21"], &both);
    does("en", &["turn off the stube thermostat"], &["HassTurnOff climate.stube"]);
    does("de", &["Stell die Thermostate in Stube und Kammer auf 21"], &both);
    does("de", &["Stell die Heizungen in Stube und Kammer auf 21"], &both);
    let off = ["HassTurnOff climate.stube", "HassTurnOff climate.kammer"];
    does("en", &["turn off the stube and kammer thermostats"], &off);
    does("en", &["turn off the stube and kammer heating"], &off);
    does("de", &["Schalte die Heizungen in Stube und Kammer aus"], &off);
}

#[test]
fn a_device_the_home_names_decides_over_the_light_default() {
    // "radiator" and "shutter" are in no word list: only the home knows what they are.
    does("en", &["set the diele radiator to 21"], &["HassClimateSetTemperature climate.diele temperature=21"]);
    does("en", &["set the diele shutter to 40"], &["HassSetPosition cover.diele position=40"]);
    // British English has no word for a thermostat in its pack.
    does("en-GB", &["set the stube thermostat to 21"], &["HassClimateSetTemperature climate.stube temperature=21"]);
}

#[test]
fn a_room_with_a_number_still_means_its_light() {
    does("en", &["set the diele to 40"], &["HassLightSet light.diele brightness=40"]);
    does("en", &["set the diele light to 40"], &["HassLightSet light.diele brightness=40"]);
}

#[test]
fn a_reading_beside_a_switch_is_asked_and_not_switched() {
    let wanted = ["HassClimateGetTemperature climate.stube", "HassTurnOff light.kammer"];
    does("de", &["Temperatur Stube und alle Lichter in der Kammer aus"], &wanted);
    does("de", &["Temperatur Stube und Licht Kammer aus"], &wanted);
    does("en", &["stube temperature and kammer lights off"], &wanted);
}

#[test]
fn heating_and_lights_in_one_sentence_keep_their_own_device() {
    let off = ["HassTurnOff climate.stube", "HassTurnOff light.kammer"];
    does("de", &["Heizung Stube und alle Lichter in der Kammer aus"], &off);
    does("de", &["Licht Kammer und Heizung Stube aus"], &off);
    does("de", &["Mach die Heizung in der Stube und das Licht in der Kammer aus"], &off);
    does("en", &["stube heater and kammer lights off"], &off);
    does("en", &["turn off the stube heater and the kammer lights"], &off);
    does("en", &["turn on the kammer lights and the stube heater"], &["HassTurnOn climate.stube", "HassTurnOn light.kammer"]);
    does(
        "en",
        &["set the kammer lights to 40 and the stube thermostat to 21"],
        &["HassLightSet light.kammer brightness=40", "HassClimateSetTemperature climate.stube temperature=21"],
    );
}

#[test]
fn a_light_named_after_a_heating_command_stays_a_light() {
    let light = ["HassLightSet light.kammer brightness=30"];
    does("en", &["set the stube heater to 21", "kammer lights to 30"], &light);
    does("de", &["Stell die Heizung in der Stube auf 21", "Licht Kammer auf 30"], &light);
    does(
        "en",
        &["stube heater to 21 and kammer lights to 30"],
        &["HassClimateSetTemperature climate.stube temperature=21", "HassLightSet light.kammer brightness=30"],
    );
}

#[test]
fn a_second_command_keeps_its_own_verb() {
    does("en", &["turn off the stube lights and lock the front door"], &["HassTurnOff light.stube", "HassTurnOn lock.front"]);
    does(
        "de",
        &["Stell die Heizung in der Kammer auf 20 und mach das Licht in der Kammer an"],
        &["HassClimateSetTemperature climate.kammer temperature=20", "HassTurnOn light.kammer"],
    );
}
