use klar_nlu::nlu;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, ParseDecision, ParseOutcome, Settings};

fn lamp(id: &str, name: &str, area: &str) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: "light".into(),
        platform: None,
        area: Some(area.into()),
        aliases: Vec::new(),
        tags: Vec::new(),
    }
}

/// Home Assistant's default slug for "Living Room" is `living_room`, with no aliases.
fn living_room_home() -> HomeGraph {
    HomeGraph {
        areas: vec![
            AreaRec { area_id: "living_room".into(), name: "Living Room".into(), aliases: Vec::new(), floor_id: None },
            AreaRec { area_id: "kitchen".into(), name: "Kitchen".into(), aliases: Vec::new(), floor_id: None },
            AreaRec { area_id: "bedroom".into(), name: "Bedroom".into(), aliases: Vec::new(), floor_id: None },
        ],
        entities: vec![
            lamp("light.living_room_main_light", "Living Room Main Light", "living_room"),
            lamp("light.living_room_lamp", "Living Room Lamp", "living_room"),
            lamp("light.kitchen_lights", "Kitchen Lights", "kitchen"),
            lamp("light.bedroom_light", "Bedroom Light", "bedroom"),
        ],
        ..HomeGraph::default()
    }
}

fn parse(text: &str, home: &HomeGraph) -> ParseOutcome {
    let mut session = Session::new();
    let settings = Settings { languages: vec!["en".into()], ..Settings::default() };
    nlu::parse(text, home, &mut session, &[], &settings)
}

fn hits_living_room(outcome: &ParseOutcome) -> bool {
    outcome.plan.as_ref().is_some_and(|plan| {
        !plan.steps.is_empty()
            && plan.steps.iter().all(|step| {
                step.intent.slots.iter().any(|slot| match slot.name.as_str() {
                    "area" => slot.value == "living_room",
                    "entity_id" => slot.value.starts_with("light.living_room_"),
                    _ => false,
                })
            })
    })
}

#[test]
fn room_suffix_area_resolves_without_alias() {
    let home = living_room_home();
    for text in [
        "Turn off the living room lights",
        "Turn off the lights in the living room",
        "Turn off all lights in the living room",
        "Turn off all the lights in the living room",
    ] {
        let outcome = parse(text, &home);
        assert!(matches!(outcome.decision, ParseDecision::Execute), "{text}: {outcome:#?}");
        assert!(hits_living_room(&outcome), "{text}: {outcome:#?}");
    }
}

/// Areas whose first word is also a command or device word ("play", "tv", "music", "front").
fn room_word_home() -> HomeGraph {
    let mut home = living_room_home();
    for (id, name) in [("play_room", "Play Room"), ("tv_room", "TV Room"), ("music_room", "Music Room"), ("front_room", "Front Room")] {
        home.areas.push(AreaRec { area_id: id.into(), name: name.into(), aliases: Vec::new(), floor_id: None });
        home.entities.push(lamp(&format!("light.{id}_light"), &format!("{name} Light"), id));
    }
    home.areas.push(AreaRec { area_id: "hall".into(), name: "Hall".into(), aliases: Vec::new(), floor_id: None });
    home.entities.push(lamp("light.hall_front_door", "Front Door Light", "hall"));
    home.entities.push(EntityRec { domain: "media_player".into(), ..lamp("media_player.living_room_tv", "Living Room TV", "living_room") });
    home
}

/// Every `area` and `entity_id` slot value across the plan's steps.
fn targets(outcome: &ParseOutcome) -> Vec<String> {
    outcome
        .plan
        .iter()
        .flat_map(|plan| &plan.steps)
        .flat_map(|step| &step.intent.slots)
        .filter(|slot| slot.name == "area" || slot.name == "entity_id")
        .map(|slot| slot.value.clone())
        .collect()
}

fn in_area(target: &str, area: &str) -> bool {
    target == area || target.contains(&format!(".{area}_"))
}

#[test]
fn room_suffix_areas_with_command_words_resolve() {
    let home = room_word_home();
    for (text, area) in [
        ("Turn on the play room lights", "play_room"),
        ("Turn on the tv room light", "tv_room"),
        ("Turn on the music room light", "music_room"),
        ("Turn on the front room light", "front_room"),
    ] {
        let outcome = parse(text, &home);
        let got = targets(&outcome);
        assert!(matches!(outcome.decision, ParseDecision::Execute), "{text}: {outcome:#?}");
        assert!(!got.is_empty() && got.iter().all(|t| in_area(t, area)), "{text}: {got:?}");
    }
}

/// Without "room", the first word alone must not pick that room.
#[test]
fn first_word_of_room_name_alone_does_not_pick_the_room() {
    let home = room_word_home();
    for (text, area) in [
        ("Turn on the front light", "front_room"),
        ("Turn off the lights in the front", "front_room"),
        ("Turn off the play lights", "play_room"),
        ("Turn on the front door light", "front_room"),
    ] {
        let got = targets(&parse(text, &home));
        assert!(!got.iter().any(|t| in_area(t, area)), "{text} picked {area}: {got:?}");
    }
}

#[test]
fn single_word_areas_are_unchanged() {
    let home = room_word_home();
    let outcome = parse("Turn off the kitchen lights", &home);
    let got = targets(&outcome);
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    assert!(!got.is_empty() && got.iter().all(|t| in_area(t, "kitchen")), "{got:?}");
}

#[test]
fn tv_word_outside_room_name_still_targets_the_tv() {
    let home = room_word_home();
    let outcome = parse("Turn off the living room tv", &home);
    let got = targets(&outcome);
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
    assert_eq!(got, vec!["media_player.living_room_tv".to_string()], "{got:?}");
}
