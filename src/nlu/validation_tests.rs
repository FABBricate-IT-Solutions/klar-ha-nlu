use super::*;
use crate::home::default_home;
use crate::types::PlanStep;

fn plan(intent: Intent) -> IntentPlan {
    IntentPlan::from_intents(vec![intent], 1.0, &[])
}

fn requires_confirmation(plan: &IntentPlan) -> bool {
    crate::nlu::risk::requires_confirmation(plan, &default_home())
}

#[test]
fn rejects_targetless_controls() {
    assert_eq!(validate_plan(&plan(Intent::new("HassTurnOn").with("domain", "light")), &default_home()), Err(PlanInvalid::MissingTarget));
}

#[test]
fn rejects_nonexistent_and_unexposed_targets() {
    let home = default_home();
    assert_eq!(validate_plan(&plan(Intent::new("HassTurnOn").with("entity_id", "light.missing")), &home), Err(PlanInvalid::UnsafeTarget));
    let mut restricted = home.clone();
    restricted.assist = Some(std::collections::HashSet::new());
    assert_eq!(
        validate_plan(&plan(Intent::new("HassTurnOn").with("entity_id", "light.wohnzimmer")), &restricted),
        Err(PlanInvalid::UnsafeTarget)
    );
}

#[test]
fn rejects_out_of_range_and_wrong_domain() {
    let home = default_home();
    assert_eq!(
        validate_plan(&plan(Intent::new("HassLightSet").with("entity_id", "light.wohnzimmer").with("brightness", "101")), &home),
        Err(PlanInvalid::Schema)
    );
    assert_eq!(
        validate_plan(&plan(Intent::new("HassFanSetSpeed").with("entity_id", "light.wohnzimmer").with("percentage", "50")), &home),
        Err(PlanInvalid::Schema)
    );
}

#[test]
fn relative_volume_accepts_only_direction_enum() {
    let mut home = default_home();
    home.entities.push(crate::types::EntityRec {
        entity_id: "media_player.test".into(),
        name: "Test player".into(),
        domain: "media_player".into(),
        platform: None,
        area: Some("wohnzimmer".into()),
        aliases: Vec::new(),
        tags: Vec::new(),
    });
    let target = "media_player.test";
    for direction in ["up", "down"] {
        assert_eq!(
            validate_plan(&plan(Intent::new("HassSetVolumeRelative").with("entity_id", target).with("volume_step", direction)), &home),
            Ok(())
        );
    }
    for malformed in ["1", "-10", "louder", "UP", ""] {
        assert_eq!(
            validate_plan(&plan(Intent::new("HassSetVolumeRelative").with("entity_id", target).with("volume_step", malformed)), &home),
            Err(PlanInvalid::Schema)
        );
    }
}

#[test]
fn rejects_area_without_visible_domain_target() {
    let mut home = default_home();
    home.assist = Some(std::collections::HashSet::new());
    assert_eq!(
        validate_plan(&plan(Intent::new("HassTurnOn").with("area", "wohnzimmer").with("domain", "light")), &home),
        Err(PlanInvalid::UnsafeTarget)
    );
}

#[test]
fn rejects_floor_without_visible_domain_target() {
    let mut home = default_home();
    home.floors.push(crate::types::FloorRec {
        floor_id: "upper".into(),
        name: "Upper Floor".into(),
        aliases: vec!["upstairs".into()],
        level: Some(1),
    });
    home.areas[0].floor_id = Some("upper".into());
    home.assist = Some(std::collections::HashSet::new());
    assert_eq!(
        validate_plan(&plan(Intent::new("HassTurnOn").with("floor", "upper").with("domain", "light")), &home),
        Err(PlanInvalid::UnsafeTarget)
    );
    home.assist = None;
    assert_eq!(validate_plan(&plan(Intent::new("HassTurnOn").with("floor", "upper").with("domain", "light")), &home), Ok(()));
}

#[test]
fn rejects_step_overflow() {
    let intent = Intent::new("HassTurnOn").with("entity_id", "light.wohnzimmer");
    let steps =
        (0..=MAX_PLAN_STEPS).map(|index| PlanStep { index, intent: intent.clone(), confidence: 1.0, evidence: Vec::new() }).collect();
    assert_eq!(validate_plan(&IntentPlan::from_steps(steps, 1.0), &default_home()), Err(PlanInvalid::Schema));
}

#[test]
fn marks_lock_and_broad_controls_risky() {
    assert!(requires_confirmation(&plan(Intent::new("HassTurnOn").with("entity_id", "lock.wohnungstuer"))));
    assert!(requires_confirmation(&plan(Intent::new("HassTurnOff").with("entity_id", "lock.wohnungstuer"))));
    assert!(requires_confirmation(&plan(Intent::new("HassTurnOff").with("entity_id", "cover.wohnzimmer_rollo"))));
    assert!(requires_confirmation(&plan(Intent::new("HassTurnOff").with("area", "wohnzimmer").with("domain", "switch"))));
    assert!(!requires_confirmation(&plan(Intent::new("HassTurnOn").with("entity_id", "cover.wohnzimmer_rollo"))));
    assert!(!requires_confirmation(&plan(Intent::new("HassClimateGetTemperature").with("area", "schlafzimmer").with("domain", "climate"))));
    assert!(!requires_confirmation(&plan(Intent::new("HassGetState").with("area", "wohnzimmer").with("domain", "climate"))));
}

#[test]
fn drops_invalid_steps_and_keeps_valid_ones() {
    let home = default_home();
    let steps = vec![
        PlanStep {
            index: 0,
            intent: Intent::new("HassTurnOn").with("entity_id", "light.wohnzimmer"),
            confidence: 0.9,
            evidence: Vec::new(),
        },
        PlanStep {
            index: 1,
            intent: Intent::new("HassClimateSetTemperature").with("domain", "climate").with("temperature", "21"),
            confidence: 0.7,
            evidence: Vec::new(),
        },
        PlanStep { index: 2, intent: Intent::new("HassTurnOn").with("entity_id", "lock.missing"), confidence: 0.8, evidence: Vec::new() },
    ];
    let filtered = filter_valid_steps(&IntentPlan::from_steps(steps, 0.2), &home);
    assert_eq!(filtered.steps.len(), 1);
    assert_eq!(filtered.steps[0].intent.slot("entity_id"), Some("light.wohnzimmer"));
    assert_eq!(filtered.steps[0].index, 0);
}

#[test]
fn lock_cover_script_are_fail_closed() {
    assert!(fail_closed_intent(&Intent::new("HassTurnOn").with("entity_id", "lock.front")));
    assert!(fail_closed_intent(&Intent::new("HassTurnOff").with("domain", "cover")));
    assert!(fail_closed_intent(&Intent::new("HassTurnOn").with("entity_id", "script.bedtime")));
    assert!(!fail_closed_intent(&Intent::new("HassTurnOn").with("entity_id", "light.wohnzimmer")));
}
