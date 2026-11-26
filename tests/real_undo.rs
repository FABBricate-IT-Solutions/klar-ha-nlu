//! Package 6.5: undo restores prior brightness, position, temperature and volume
//! (and still inverts binary actions), through the safety decision.
use std::collections::BTreeSet;

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::session::UndoPrior;
use klar_nlu::types::{HomeGraph, Intent, ParseDecision};

fn test_home(lang: &str) -> HomeGraph {
    let de = lang == "de";
    let name = |en: &'static str, german: &'static str| if de { german } else { en };
    home(
        vec![
            area("office", name("Office", "Büro")),
            area("hall", name("Hall", "Flur")),
            area("living", name("Living Room", "Wohnzimmer")),
            area("garage", "Garage"),
        ],
        vec![
            ent("light.hall_light", name("Hall Light", "Flur Licht"), "hall"),
            ent("light.living_room_lights", name("Living Room Lights", "Wohnzimmerlicht"), "living"),
            ent("cover.office_blinds", name("Office Blinds", "Büro Rollo"), "office"),
            ent("cover.garage_door", name("Garage Door", "Garagentor"), "garage"),
            ent("lock.front_door", name("Front Door", "Haustür"), "hall"),
            ent("climate.living_thermostat", name("Living Thermostat", "Wohnzimmerthermostat"), "living"),
            ent("media_player.living_speaker", name("Living Speaker", "Wohnzimmerlautsprecher"), "living"),
            ent("fan.office_fan", name("Office Fan", "Büroventilator"), "office"),
        ],
        &[],
    )
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

/// Seed the last execute as HA would after a successful forward action, with priors stamped.
fn seed_with_prior(talk: &mut Talk, intent: Intent, prior: UndoPrior) {
    talk.session.begin_remember_batch();
    talk.session.remember(&intent);
    talk.session.last_execute = vec![intent];
    talk.session.stamp_undo_priors(&[prior]);
}

/// Acceptance: undo works for eight intent types (binary invert + continuous restore).
#[test]
fn undo_covers_eight_intent_types() {
    let home = test_home("en");

    // 1. HassTurnOn → HassTurnOff
    let mut talk = Talk::new("en", &home).without_confirm();
    acts_on(&mut talk, "turn on the hall light", &["HassTurnOn light.hall_light"]);
    acts_on(&mut talk, "undo that", &["HassTurnOff light.hall_light"]);

    // 2. HassLightSet restores prior brightness
    let mut talk = Talk::new("en", &home).without_confirm();
    seed_with_prior(
        &mut talk,
        Intent::new("HassLightSet").with("entity_id", "light.living_room_lights").with("brightness", "30"),
        UndoPrior { brightness: Some(80), ..UndoPrior::for_entity("light.living_room_lights") },
    );
    acts_on(&mut talk, "undo that", &["HassLightSet light.living_room_lights brightness=80"]);

    // 3. HassSetPosition restores prior position
    let mut talk = Talk::new("en", &home).without_confirm();
    seed_with_prior(
        &mut talk,
        Intent::new("HassSetPosition").with("entity_id", "cover.office_blinds").with("position", "20"),
        UndoPrior { position: Some(90), ..UndoPrior::for_entity("cover.office_blinds") },
    );
    acts_on(&mut talk, "undo that", &["HassSetPosition cover.office_blinds position=90"]);

    // 4. HassClimateSetTemperature restores prior temperature
    let mut talk = Talk::new("en", &home).without_confirm();
    seed_with_prior(
        &mut talk,
        Intent::new("HassClimateSetTemperature").with("entity_id", "climate.living_thermostat").with("temperature", "22"),
        UndoPrior { temperature: Some("19".into()), ..UndoPrior::for_entity("climate.living_thermostat") },
    );
    acts_on(&mut talk, "undo that", &["HassClimateSetTemperature climate.living_thermostat temperature=19"]);

    // 5. HassSetVolume restores prior volume
    let mut talk = Talk::new("en", &home).without_confirm();
    seed_with_prior(
        &mut talk,
        Intent::new("HassSetVolume").with("entity_id", "media_player.living_speaker").with("volume_level", "20"),
        UndoPrior { volume_level: Some(55), ..UndoPrior::for_entity("media_player.living_speaker") },
    );
    acts_on(&mut talk, "undo that", &["HassSetVolume media_player.living_speaker volume_level=55"]);

    // 6. HassFanSetSpeed restores prior percentage
    let mut talk = Talk::new("en", &home).without_confirm();
    seed_with_prior(
        &mut talk,
        Intent::new("HassFanSetSpeed").with("entity_id", "fan.office_fan").with("percentage", "100"),
        UndoPrior { percentage: Some(40), ..UndoPrior::for_entity("fan.office_fan") },
    );
    acts_on(&mut talk, "undo that", &["HassFanSetSpeed fan.office_fan percentage=40"]);

    // 7. Lock undo asks again through the safety level
    let mut talk = Talk::new("en", &home);
    asks_first(&mut talk, "lock the front door");
    acts_on(&mut talk, "yes", &["HassTurnOn lock.front_door"]);
    asks_first(&mut talk, "undo that");
    acts_on(&mut talk, "yes", &["HassTurnOff lock.front_door"]);

    // 8. Cover close undo asks again before opening the garage
    let mut talk = Talk::new("en", &home);
    asks_first(&mut talk, "close the garage door");
    acts_on(&mut talk, "yes", &["HassTurnOff cover.garage_door"]);
    asks_first(&mut talk, "undo that");
    acts_on(&mut talk, "yes", &["HassTurnOn cover.garage_door"]);
}

#[test]
fn brightness_step_flips_without_a_prior() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home).without_confirm();
    talk.session.begin_remember_batch();
    let forward = Intent::new("HassLightSet").with("entity_id", "light.living_room_lights").with("brightness_step", "up");
    talk.session.remember(&forward);
    talk.session.last_execute = vec![forward];
    let out = talk.say("undo that");
    assert_eq!(out.decision, ParseDecision::Execute, "{}", describe(&out));
    let intent = &out.plan.as_ref().unwrap().intents()[0];
    assert_eq!(intent.name, "HassLightSet");
    assert_eq!(intent.slot("entity_id"), Some("light.living_room_lights"));
    assert_eq!(intent.slot("brightness_step"), Some("down"));
}

#[test]
fn absolute_brightness_without_prior_cannot_undo() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home).without_confirm();
    talk.session.begin_remember_batch();
    let forward = Intent::new("HassLightSet").with("entity_id", "light.living_room_lights").with("brightness", "30");
    talk.session.remember(&forward);
    talk.session.last_execute = vec![forward];
    let out = talk.say("undo that");
    assert!(!matches!(out.decision, ParseDecision::Execute), "absolute set without prior must not invent a restore: {}", describe(&out));
}

#[test]
fn spoken_brightness_set_then_stamped_prior_undoes() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home).without_confirm();
    acts_on(&mut talk, "set the living room lights to 30 percent", &["HassLightSet light.living_room_lights brightness=30"]);
    talk.session.stamp_undo_priors(&[UndoPrior { brightness: Some(80), ..UndoPrior::for_entity("light.living_room_lights") }]);
    acts_on(&mut talk, "undo that", &["HassLightSet light.living_room_lights brightness=80"]);
}
