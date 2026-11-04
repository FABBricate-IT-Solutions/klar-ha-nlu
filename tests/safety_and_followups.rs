//! Safety gates (bad input, user policy rules, confirmations) and follow-ups that reuse the
//! last device ("21" after the heating, "50" after a fan), in English and German.
use std::collections::BTreeSet;

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::types::{HomeGraph, ParseDecision, PolicyEffect, PolicyMatch, PolicyRule, RejectReason};

fn test_home(lang: &str) -> HomeGraph {
    let de = lang == "de";
    let name = |en: &'static str, german: &'static str| if de { german } else { en };
    home(
        vec![area("office", name("Office", "Büro")), area("hall", name("Hall", "Flur")), area("bedroom", name("Bedroom", "Schlafzimmer"))],
        vec![
            ent("light.office_light", name("Office Light", "Büro Licht"), "office"),
            ent("cover.office_blinds", name("Office Blinds", "Büro Rollo"), "office"),
            ent("light.hall_light", name("Hall Light", "Flur Licht"), "hall"),
            ent("climate.heating", name("Heating", "Heizung"), "hall"),
            ent("fan.bedroom_fan", name("Bedroom Fan", "Schlafzimmer Ventilator"), "bedroom"),
        ],
        &[],
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

fn acts_on(talk: &mut Talk, text: &str, wanted: &[&str]) {
    let out = talk.say(text);
    let wanted: BTreeSet<String> = wanted.iter().map(|s| s.to_string()).collect();
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {}", describe(&out));
    assert_eq!(effects(&out, talk.home), wanted, "{text}: {}", describe(&out));
}

fn asks_first(talk: &mut Talk, text: &str) {
    let out = talk.say(text);
    assert!(matches!(out.decision, ParseDecision::Confirm { .. }), "{text}: {}", describe(&out));
    assert!(out.plan.is_none(), "{text}: nothing may run before yes: {}", describe(&out));
}

#[test]
fn control_characters_and_overlong_input_are_rejected() {
    let home = test_home("en");
    for text in ["turn on the\u{7} office light".to_string(), "turn on the office light ".repeat(60)] {
        let out = Talk::new("en", &home).say(&text);
        assert!(matches!(out.decision, ParseDecision::Reject { reason: RejectReason::InvalidInput }), "{}", describe(&out));
        assert!(out.plan.is_none());
    }
}

#[test]
fn confirm_rule_asks_even_for_a_light() {
    for (lang, text, yes) in [("en", "turn on the office light", "yes"), ("de", "Mach das Licht im Büro an", "ja")] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        talk.rules = vec![rule("light.office_light", PolicyEffect::Confirm)];
        asks_first(&mut talk, text);
        acts_on(&mut talk, yes, &["HassTurnOn light.office_light"]);
    }
}

#[test]
fn allow_rule_skips_the_prompt_for_a_cover() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home);
    asks_first(&mut talk, "close the office blinds");
    let mut allowed = Talk::new("en", &home);
    allowed.rules = vec![rule("cover.office_blinds", PolicyEffect::Allow)];
    acts_on(&mut allowed, "close the office blinds", &["HassTurnOff cover.office_blinds"]);
}

#[test]
fn block_rule_never_runs_the_device() {
    let home = test_home("en");
    let mut talk = Talk::new("en", &home).without_confirm();
    talk.rules = vec![rule("light.hall_light", PolicyEffect::Block)];
    let out = talk.say("turn on the hall light");
    assert!(matches!(out.decision, ParseDecision::Reject { reason: RejectReason::Unsafe }), "{}", describe(&out));
    assert!(out.plan.is_none());
}

#[test]
#[ignore = "bug: a new command while a confirmation is pending re-asks the old question, so the new command never runs and a later yes runs the old one"]
fn new_command_while_confirming_replaces_the_pending_one() {
    for (lang, risky, other, yes) in [
        ("en", "close the office blinds", "turn on the hall light", "yes"),
        ("de", "Mach das Rollo im Büro zu", "Mach das Licht im Flur an", "ja"),
    ] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home);
        asks_first(&mut talk, risky);
        acts_on(&mut talk, other, &["HassTurnOn light.hall_light"]);
        let out = talk.say(yes);
        assert!(effects(&out, &home).is_empty(), "{lang}: the old question was answered by the new command: {}", describe(&out));
    }
}

#[test]
fn a_bare_number_sets_the_last_device() {
    for (lang, first, number, wanted) in [
        ("en", "turn on the heating", "21", "HassClimateSetTemperature climate.heating temperature=21"),
        ("en", "turn on the heating", "set it to 21", "HassClimateSetTemperature climate.heating temperature=21"),
        ("en", "turn on the bedroom fan", "50", "HassFanSetSpeed fan.bedroom_fan percentage=50"),
        ("en", "turn on the office light", "40", "HassLightSet light.office_light brightness=40"),
        ("de", "Mach die Heizung an", "21", "HassClimateSetTemperature climate.heating temperature=21"),
        ("de", "Mach den Ventilator im Schlafzimmer an", "50", "HassFanSetSpeed fan.bedroom_fan percentage=50"),
    ] {
        let home = test_home(lang);
        let mut talk = Talk::new(lang, &home).without_confirm();
        talk.say(first);
        acts_on(&mut talk, number, &[wanted]);
    }
}
