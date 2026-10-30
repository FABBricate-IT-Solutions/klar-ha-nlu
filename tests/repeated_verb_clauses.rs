//! The same verb restated after "and" starts a new clause:
//! "Turn off the TV in the living room and turn off the kitchen lights and turn on the hall light".
//! Merged into one clause, the kitchen step took the TV's media-player domain, so the kitchen
//! speakers went off instead of the kitchen lights, and the TV step widened to every media player.

use std::collections::BTreeSet;

use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, Intent, ParseDecision, ParseOutcome, Settings};

fn area(id: &str, name: &str) -> AreaRec {
    AreaRec { area_id: id.into(), name: name.into(), aliases: vec![], floor_id: None }
}

fn entity(id: &str, name: &str, area: &str) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: id.split_once('.').map(|(domain, _)| domain).unwrap_or("").into(),
        platform: None,
        area: Some(area.into()),
        aliases: vec![],
        tags: vec![],
    }
}

/// Every room has a light and a media player, so a domain leaking between steps
/// always hits a real, wrong device.
fn en_home() -> HomeGraph {
    HomeGraph {
        areas: vec![area("living_room", "Living Room"), area("kitchen", "Kitchen"), area("hall", "Hall"), area("bedroom", "Bedroom")],
        entities: vec![
            entity("media_player.living_room_tv", "Living Room TV", "living_room"),
            entity("media_player.living_room_homepod", "Living Room HomePod", "living_room"),
            entity("light.living_room", "Living Room Light", "living_room"),
            entity("light.kitchen_lights", "Kitchen Lights", "kitchen"),
            entity("media_player.kitchen_speaker", "Kitchen Speaker", "kitchen"),
            entity("light.hall_light", "Hall Light", "hall"),
            entity("media_player.hall_speaker", "Hall Speaker", "hall"),
            entity("light.bedroom_light", "Bedroom Light", "bedroom"),
        ],
        ..HomeGraph::default()
    }
}

fn de_home() -> HomeGraph {
    HomeGraph {
        areas: vec![area("wohnzimmer", "Wohnzimmer"), area("kuche", "Küche"), area("flur", "Flur")],
        entities: vec![
            entity("media_player.wohnzimmer_tv", "Wohnzimmer Fernseher", "wohnzimmer"),
            entity("media_player.wohnzimmer_homepod", "Wohnzimmer HomePod", "wohnzimmer"),
            entity("light.kuche", "Küche Licht", "kuche"),
            entity("media_player.kuche_lautsprecher", "Küche Lautsprecher", "kuche"),
            entity("light.flur", "Flur Licht", "flur"),
        ],
        ..HomeGraph::default()
    }
}

fn run(text: &str, home: &HomeGraph, lang: &str) -> ParseOutcome {
    let settings = Settings { confirm_risky_actions: false, ..Settings::pinned(lang) };
    parse(text, home, &mut Session::new(), &[], &settings)
}

fn domain_of(intent: &Intent) -> &str {
    intent.slot("domain").or_else(|| intent.slot("entity_id").and_then(|id| id.split_once('.').map(|(head, _)| head))).unwrap_or("")
}

/// Every (action, device) pair Home Assistant would act on, however the plan is shaped.
fn effects(out: &ParseOutcome, home: &HomeGraph) -> BTreeSet<(String, String)> {
    let intents = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    intents
        .iter()
        .flat_map(|intent| {
            home.entities
                .iter()
                .filter(|entity| match intent.slot("entity_id") {
                    Some(id) => entity.entity_id == id,
                    None => entity.area.as_deref() == intent.slot("area") && entity.domain == domain_of(intent),
                })
                .map(|entity| (intent.name.clone(), entity.entity_id.clone()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn expect(pairs: &[(&str, &str)]) -> BTreeSet<(String, String)> {
    pairs.iter().map(|(action, id)| ((*action).to_string(), (*id).to_string())).collect()
}

const ON: &str = "HassTurnOn";
const OFF: &str = "HassTurnOff";

fn assert_effects(text: &str, home: &HomeGraph, lang: &str, wanted: &[(&str, &str)]) {
    let out = run(text, home, lang);
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    let plan = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    assert_eq!(effects(&out, home), expect(wanted), "{text}\nplan: {plan:?}");
}

#[test]
fn en_restated_verb_keeps_each_step_on_its_own_devices() {
    let home = en_home();
    for (text, wanted) in [
        (
            "Turn off the TV in the living room and turn off the kitchen lights and turn on the hall light",
            vec![(OFF, "media_player.living_room_tv"), (OFF, "light.kitchen_lights"), (ON, "light.hall_light")],
        ),
        (
            "turn off the kitchen lights and turn off the TV in the living room and turn on the hall light",
            vec![(OFF, "light.kitchen_lights"), (OFF, "media_player.living_room_tv"), (ON, "light.hall_light")],
        ),
        (
            "turn off the TV in the living room and turn off the kitchen lights and turn off the hall light",
            vec![(OFF, "media_player.living_room_tv"), (OFF, "light.kitchen_lights"), (OFF, "light.hall_light")],
        ),
        (
            "turn off the TV in the living room and turn off the kitchen lights and turn on the hall light and turn on the bedroom light",
            vec![
                (OFF, "media_player.living_room_tv"),
                (OFF, "light.kitchen_lights"),
                (ON, "light.hall_light"),
                (ON, "light.bedroom_light"),
            ],
        ),
        (
            "turn off the TV in the living room and turn off the kitchen lights",
            vec![(OFF, "media_player.living_room_tv"), (OFF, "light.kitchen_lights")],
        ),
        ("turn on the hall light and turn on the bedroom light", vec![(ON, "light.hall_light"), (ON, "light.bedroom_light")]),
    ] {
        assert_effects(text, &home, "en", &wanted);
    }
}

#[test]
fn en_one_verb_over_several_rooms_still_works() {
    // Not restated: one command across two rooms, unchanged by the fix.
    let home = en_home();
    assert_effects(
        "turn off the lights in the kitchen and the hall",
        &home,
        "en",
        &[(OFF, "light.kitchen_lights"), (OFF, "light.hall_light")],
    );
    assert_effects("turn off the kitchen and turn off the hall", &home, "en", &[(OFF, "light.kitchen_lights"), (OFF, "light.hall_light")]);
}

#[test]
fn de_restated_verb_keeps_each_step_on_its_own_devices() {
    let home = de_home();
    let wanted = [(OFF, "media_player.wohnzimmer_tv"), (OFF, "light.kuche"), (ON, "light.flur")];
    for text in [
        "Schalte den Fernseher im Wohnzimmer aus und schalte das Licht in der Küche aus und schalte das Licht im Flur an",
        "Mach den Fernseher im Wohnzimmer aus und mach das Licht in der Küche aus und mach das Licht im Flur an",
        "Schalte aus den Fernseher im Wohnzimmer und schalte aus das Licht in der Küche und schalte an das Licht im Flur",
    ] {
        assert_effects(text, &home, "de", &wanted);
    }
}

#[test]
fn de_one_verb_over_several_rooms_still_works() {
    assert_effects("Mach das Licht in der Küche an und im Flur an", &de_home(), "de", &[(ON, "light.kuche"), (ON, "light.flur")]);
}

#[test]
fn a_light_step_never_switches_a_media_player() {
    // The reported symptom: "the kitchen lights" turned off the kitchen speakers.
    let home = en_home();
    let out = run("Turn off the TV in the living room and turn off the kitchen lights and turn on the hall light", &home, "en");
    for (_, id) in effects(&out, &home) {
        assert!(id != "media_player.kitchen_speaker" && id != "media_player.living_room_homepod", "wrong device {id}");
    }
}

#[test]
fn en_commas_between_steps_split_like_and() {
    // Commas are gone after tokenising; "…TV, turn off the kitchen lights…" once merged the first two steps.
    let home = en_home();
    let wanted = [(OFF, "media_player.living_room_tv"), (OFF, "light.kitchen_lights"), (ON, "light.hall_light")];
    for text in [
        "Turn off the living room TV, turn off the kitchen lights and turn on the hall light.",
        "Turn off the living room TV, turn off the kitchen lights, turn on the hall light.",
        "Turn the living room TV off, turn the kitchen lights off and turn the hall light on.",
    ] {
        assert_effects(text, &home, "en", &wanted);
    }
    assert_effects(
        "turn on the kitchen lights, turn on the hall light",
        &home,
        "en",
        &[(ON, "light.kitchen_lights"), (ON, "light.hall_light")],
    );
}

#[test]
fn de_commas_between_steps_split_like_und() {
    let home = de_home();
    let wanted = [(OFF, "media_player.wohnzimmer_tv"), (OFF, "light.kuche"), (ON, "light.flur")];
    for text in [
        "Mach den Fernseher im Wohnzimmer aus, mach das Licht in der Küche aus und mach das Licht im Flur an",
        "Schalte den Fernseher im Wohnzimmer aus, schalte das Licht in der Küche aus und schalte das Licht im Flur an",
        "Schalte aus den Fernseher im Wohnzimmer, schalte aus das Licht in der Küche und schalte an das Licht im Flur",
    ] {
        assert_effects(text, &home, "de", &wanted);
    }
}
