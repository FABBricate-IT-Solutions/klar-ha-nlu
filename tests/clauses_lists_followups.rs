//! Package 2.3: clauses, shopping-list entries, follow-ups, satellite area, ASR repairs.
use std::collections::BTreeSet;

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::session::Sessions;
use klar_nlu::types::{EntityRec, HomeGraph, ParseDecision, Settings};

fn music(entity: EntityRec) -> EntityRec {
    EntityRec { tags: vec!["music".into()], ..entity }
}

fn fixture() -> HomeGraph {
    home(
        vec![
            area("hall", "Hall"),
            area("office", "Office"),
            area("kitchen", "Kitchen"),
            area("living_room", "Living Room"),
            area("bedroom", "Bedroom"),
        ],
        vec![
            ent("light.hall_light", "Hall Light", "hall"),
            ent("light.office_light", "Office Light", "office"),
            ent("light.kitchen_ceiling", "Kitchen Ceiling", "kitchen"),
            ent("switch.oven", "Oven", "kitchen"),
            ent("switch.coffee_machine", "Coffee Machine", "kitchen"),
            ent("switch.tv_plug", "TV Plug", "living_room"),
            ent("fan.office_fan", "Office Fan", "office"),
            ent("fan.bedroom_fan", "Bedroom Fan", "bedroom"),
            ent("cover.office_blinds", "Office Blinds", "office"),
            ent("cover.bedroom_blinds", "Bedroom Blinds", "bedroom"),
            music(ent("media_player.kitchen_speaker", "Kitchen Speaker", "kitchen")),
            ent("todo.shopping_list", "Shopping List", ""),
            ent("todo.chores", "Chores", ""),
            ent("weather.home", "Home", ""),
        ],
        &[],
    )
}

fn acts(out: &klar_nlu::types::ParseOutcome, home: &HomeGraph, wanted: &[&str]) {
    let wanted: BTreeSet<String> = wanted.iter().map(|s| (*s).to_string()).collect();
    assert_eq!(out.decision, ParseDecision::Execute, "{}", describe(out));
    assert_eq!(effects(out, home), wanted, "{}", describe(out));
}

#[test]
fn p08_three_named_devices_across_domains() {
    let home = fixture();
    for (text, wanted) in [
        (
            "turn off the hall light and oven and coffee machine",
            ["HassTurnOff light.hall_light", "HassTurnOff switch.oven", "HassTurnOff switch.coffee_machine"].as_slice(),
        ),
        (
            "turn off the oven and the coffee machine and the tv plug",
            ["HassTurnOff switch.oven", "HassTurnOff switch.coffee_machine", "HassTurnOff switch.tv_plug"].as_slice(),
        ),
    ] {
        let out = Talk::new("en", &home).without_confirm().say(text);
        acts(&out, &home, wanted);
    }
}

#[test]
fn p13_which_lights_are_on_is_a_list_status_query() {
    let home = fixture();
    for text in ["which lights are on", "are any lights on", "how many lights are on"] {
        let out = Talk::new("en", &home).without_confirm().say(text);
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {}", describe(&out));
        let intent = out.plan.as_ref().expect("plan").intents().into_iter().next().expect("intent");
        assert_eq!(intent.name, "HassGetState", "{text}: {intent:?}");
        assert_eq!(intent.slot("domain"), Some("light"), "{text}: {intent:?}");
        assert_eq!(intent.slot("response"), Some("list"), "{text}: {intent:?}");
        assert_eq!(intent.slot("state"), Some("on"), "{text}: {intent:?}");
        assert!(!out.speech.contains("What should I turn on"), "{text}: {}", out.speech);
    }
}

#[test]
fn p14_and_the_hall_carries_turn_on() {
    let home = fixture();
    let mut talk = Talk::new("en", &home).without_confirm();
    acts(&talk.say("turn on the kitchen lights"), &home, &["HassTurnOn light.kitchen_ceiling"]);
    acts(&talk.say("and the hall"), &home, &["HassTurnOn light.hall_light"]);
}

#[test]
fn p14_turn_it_off_after_weather_does_not_target_weather() {
    let home = fixture();
    let mut talk = Talk::new("en", &home).without_confirm();
    let weather = talk.say("what's the weather");
    assert_eq!(weather.decision, ParseDecision::Execute, "{}", describe(&weather));
    assert!(
        weather.plan.as_ref().is_some_and(|plan| plan.intents().iter().any(|intent| intent.slot("entity_id") == Some("weather.home"))),
        "{}",
        describe(&weather)
    );
    let follow = talk.say("turn it off");
    assert!(
        !follow.plan.as_ref().is_some_and(|plan| plan.intents().iter().any(|intent| intent.slot("entity_id") == Some("weather.home"))),
        "weather must not be turned off: {}",
        describe(&follow)
    );
}

#[test]
fn p18_multi_entry_add_and_take_out_item() {
    let home = fixture();
    let multi = Talk::new("en", &home).without_confirm().say("add milk and eggs to the shopping list");
    assert_eq!(multi.decision, ParseDecision::Execute, "{}", describe(&multi));
    let items: BTreeSet<_> = multi
        .plan
        .as_ref()
        .map(|plan| plan.intents())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|intent| intent.slot("item").map(str::to_string))
        .collect();
    assert_eq!(items, BTreeSet::from(["milk".into(), "eggs".into()]), "{}", describe(&multi));

    let take_out = Talk::new("en", &home).without_confirm().say("add take out the trash to my chores");
    assert_eq!(take_out.decision, ParseDecision::Execute, "{}", describe(&take_out));
    let intent = take_out.plan.as_ref().unwrap().intents().into_iter().next().unwrap();
    assert_eq!(intent.name, "HassListAddItem", "{intent:?}");
    assert_eq!(intent.slot("item"), Some("take out trash"), "{intent:?}");

    let bare = Talk::new("en", &home).without_confirm().say("add milk");
    assert_eq!(bare.decision, ParseDecision::Execute, "{}", describe(&bare));
    let intent = bare.plan.as_ref().unwrap().intents().into_iter().next().unwrap();
    assert_eq!(intent.name, "HassListAddItem", "{intent:?}");
    assert_eq!(intent.slot("item"), Some("milk"), "{intent:?}");

    let remove = Talk::new("en", &home).without_confirm().say("take milk out of the shopping list");
    assert_eq!(remove.decision, ParseDecision::Execute, "{}", describe(&remove));
    let intent = remove.plan.as_ref().unwrap().intents().into_iter().next().unwrap();
    assert_eq!(intent.name, "HassListRemoveItem", "{intent:?}");
    assert_eq!(intent.slot("item"), Some("milk"), "{intent:?}");
}

#[test]
fn p22_preferred_satellite_area_for_fan_and_cover() {
    let home = fixture();
    let mut talk = Talk::new("en", &home).without_confirm();
    talk.session.preferred_area = Some("office".into());
    acts(&talk.say("turn on the light"), &home, &["HassTurnOn light.office_light"]);
    acts(&talk.say("turn on the fan"), &home, &["HassTurnOn fan.office_fan"]);
    acts(&talk.say("close the blinds"), &home, &["HassTurnOff cover.office_blinds"]);
}

#[test]
fn p23_new_conversation_same_satellite_inherits_context() {
    let home = fixture();
    let settings = Settings::pinned("en");
    let mut sessions = Sessions::default();
    let mut first = sessions.take_with_area(Some("conv-1"), "kitchen");
    let out = klar_nlu::nlu::parse("turn on the light", &home, &mut first, &[], &settings);
    assert_eq!(out.decision, ParseDecision::Execute, "{}", describe(&out));
    sessions.put(first);
    let mut second = sessions.take_with_area(Some("conv-2"), "kitchen");
    assert!(
        second.last_areas().any(|area| area == "kitchen") || second.last_entities().next().is_some(),
        "expected inherited kitchen context, last={:?}",
        second.last
    );
    let follow = klar_nlu::nlu::parse("turn them off", &home, &mut second, &[], &settings);
    acts(&follow, &home, &["HassTurnOff light.kitchen_ceiling"]);
}

#[test]
fn p29_asr_turn_of_and_fullwidth_latin() {
    let home = fixture();
    for text in ["turn of the hall light", "switch of the hall light", "ｔｕｒｎ ｏｆｆ ｔｈｅ ｈａｌｌ ｌｉｇｈｔ"] {
        let out = Talk::new("en", &home).without_confirm().say(text);
        acts(&out, &home, &["HassTurnOff light.hall_light"]);
    }
}

#[test]
fn p32_repeated_clauses_do_not_duplicate_steps() {
    let home = fixture();
    let text = "turn on the hall light and turn on the hall light and turn on the hall light and turn on the hall light and turn on the hall light and turn on the hall light and turn on the hall light and turn on the hall light and turn on the hall light";
    let out = Talk::new("en", &home).without_confirm().say(text);
    assert_eq!(out.decision, ParseDecision::Execute, "{}", describe(&out));
    let intents = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    assert_eq!(intents.len(), 1, "duplicate steps: {}", describe(&out));
    assert_eq!(intents[0].slot("entity_id"), Some("light.hall_light"), "{}", describe(&out));
}

#[test]
fn p35_pause_music_with_sole_unlabelled_player() {
    let home = home(vec![area("kitchen", "Kitchen")], vec![ent("media_player.kitchen_speaker", "Kitchen Speaker", "kitchen")], &[]);
    let out = Talk::new("en", &home).without_confirm().say("Pause the music");
    acts(&out, &home, &["HassMediaPause media_player.kitchen_speaker"]);
}
