use crate::home::expose::assist_visible;
use crate::types::{known_intent, HomeGraph, Intent, IntentPlan, PlanStep, MAX_PLAN_STEPS};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanInvalid {
    MissingTarget,
    UnsafeTarget,
    Schema,
}

pub(super) fn validate_plan(plan: &IntentPlan, home: &HomeGraph) -> Result<(), PlanInvalid> {
    if plan.steps.is_empty() || plan.steps.len() > MAX_PLAN_STEPS {
        return Err(PlanInvalid::Schema);
    }
    for step in &plan.steps {
        validate_intent(&step.intent, home)?;
    }
    Ok(())
}

pub(super) fn filter_valid_steps(plan: &IntentPlan, home: &HomeGraph) -> IntentPlan {
    let steps: Vec<PlanStep> = plan
        .steps
        .iter()
        .filter(|step| validate_intent(&step.intent, home).is_ok())
        .cloned()
        .enumerate()
        .map(|(index, mut step)| {
            step.index = index;
            step
        })
        .collect();
    IntentPlan::from_steps(steps, plan.margin)
}

pub(super) fn requires_confirmation(plan: &IntentPlan) -> bool {
    plan.steps.len() >= 8 || plan.steps.iter().any(|step| risky_intent(&step.intent))
}

fn validate_intent(intent: &Intent, home: &HomeGraph) -> Result<(), PlanInvalid> {
    if !known_intent(&intent.name) || conflicting_slots(intent) {
        return Err(PlanInvalid::Schema);
    }
    validate_slots(intent)?;
    validate_target(intent, home)?;
    validate_numeric(intent)?;
    Ok(())
}

fn validate_target(intent: &Intent, home: &HomeGraph) -> Result<(), PlanInvalid> {
    let entity_id = intent.slot("entity_id");
    let area = intent.slot("area");
    let floor = intent.slot("floor");
    let domain = intent.slot("domain");
    if requires_target(&intent.name) && entity_id.is_none() && area.is_none() && floor.is_none() {
        return Err(PlanInvalid::MissingTarget);
    }
    if let Some(entity_id) = entity_id {
        let entity = home.entities.iter().find(|entity| entity.entity_id == entity_id).ok_or(PlanInvalid::UnsafeTarget)?;
        if !assist_visible(entity, home) {
            return Err(PlanInvalid::UnsafeTarget);
        }
        if domain.is_some_and(|value| value != entity.domain) {
            return Err(PlanInvalid::Schema);
        }
        if let Some(expected) = required_domain(&intent.name) {
            if entity.domain != expected {
                return Err(PlanInvalid::Schema);
            }
        }
    }
    if let Some(area) = area {
        if !home.areas.iter().any(|record| record.area_id == area) {
            return Err(PlanInvalid::UnsafeTarget);
        }
        if domain.is_some_and(|value| !allowed_domain(value)) {
            return Err(PlanInvalid::Schema);
        }
        if let Some(expected) = required_domain(&intent.name) {
            if domain != Some(expected) {
                return Err(PlanInvalid::Schema);
            }
        }
        let target_domain = required_domain(&intent.name).or(domain);
        let has_visible_target = home.entities.iter().any(|entity| {
            entity.area.as_deref() == Some(area)
                && assist_visible(entity, home)
                && target_domain.is_none_or(|expected| entity.domain == expected)
        });
        if !has_visible_target {
            return Err(PlanInvalid::UnsafeTarget);
        }
    }
    if let Some(floor) = floor {
        if home.floor(floor).is_none() {
            return Err(PlanInvalid::UnsafeTarget);
        }
        if domain.is_some_and(|value| !allowed_domain(value)) {
            return Err(PlanInvalid::Schema);
        }
        if let Some(expected) = required_domain(&intent.name) {
            if domain != Some(expected) {
                return Err(PlanInvalid::Schema);
            }
        }
        let target_domain = required_domain(&intent.name).or(domain);
        if !home.areas_on_floor(floor).any(|area| {
            home.entities.iter().any(|entity| {
                entity.area.as_deref() == Some(area.area_id.as_str())
                    && assist_visible(entity, home)
                    && target_domain.is_none_or(|expected| entity.domain == expected)
            })
        }) {
            return Err(PlanInvalid::UnsafeTarget);
        }
    }
    if let Some(domain) = domain {
        if !allowed_domain(domain) {
            return Err(PlanInvalid::Schema);
        }
    }
    Ok(())
}

fn validate_slots(intent: &Intent) -> Result<(), PlanInvalid> {
    let allowed = allowed_slots(&intent.name);
    if intent.slots.iter().any(|slot| {
        slot.name.len() > 64 || slot.value.is_empty() || slot.value.chars().count() > 512 || !allowed.contains(&slot.name.as_str())
    }) {
        return Err(PlanInvalid::Schema);
    }
    match intent.name.as_str() {
        "HassLightSet" if intent.slot("brightness").is_none() && intent.slot("color").is_none() => Err(PlanInvalid::Schema),
        "HassClimateSetTemperature" if intent.slot("temperature").is_none() => Err(PlanInvalid::Schema),
        "HassFanSetSpeed" if intent.slot("percentage").is_none() => Err(PlanInvalid::Schema),
        "HassSetPosition" if intent.slot("position").is_none() => Err(PlanInvalid::Schema),
        "HassSetVolume" if intent.slot("volume_level").is_none() => Err(PlanInvalid::Schema),
        "HassSetVolumeRelative" if !matches!(intent.slot("volume_step"), Some("up" | "down")) => Err(PlanInvalid::Schema),
        "HassMediaSearchAndPlay" | "MassPlayMedia" if intent.slot("search_query").is_none() && intent.slot("media_id").is_none() => {
            Err(PlanInvalid::Schema)
        }
        "HassListAddItem" | "HassListCompleteItem" | "HassShoppingListAddItem" | "HassShoppingListCompleteItem"
            if intent.slot("item").is_none() =>
        {
            Err(PlanInvalid::Schema)
        }
        _ => Ok(()),
    }
}

fn validate_numeric(intent: &Intent) -> Result<(), PlanInvalid> {
    for slot in &intent.slots {
        let range = match slot.name.as_str() {
            "brightness" | "percentage" | "position" | "volume_level" => Some((0.0, 100.0)),
            "temperature" => Some((-50.0, 100.0)),
            "hours" => Some((0.0, 24.0)),
            "minutes" | "seconds" => Some((0.0, 86_400.0)),
            _ => None,
        };
        if let Some((minimum, maximum)) = range {
            let value = slot.value.parse::<f64>().map_err(|_| PlanInvalid::Schema)?;
            if !value.is_finite() || value < minimum || value > maximum {
                return Err(PlanInvalid::Schema);
            }
        }
    }
    Ok(())
}

pub(super) fn requires_target(name: &str) -> bool {
    !matches!(
        name,
        "HassStartTimer"
            | "HassIncreaseTimer"
            | "HassDecreaseTimer"
            | "HassCancelTimer"
            | "HassPauseTimer"
            | "HassListAddItem"
            | "HassListCompleteItem"
            | "HassShoppingListAddItem"
            | "HassShoppingListCompleteItem"
    )
}

fn required_domain(name: &str) -> Option<&'static str> {
    match name {
        "HassLightSet" => Some("light"),
        "HassClimateSetTemperature" | "HassClimateGetTemperature" => Some("climate"),
        "HassMediaPause"
        | "HassMediaUnpause"
        | "HassMediaNext"
        | "HassMediaPrevious"
        | "HassMediaPlayerMute"
        | "HassMediaPlayerUnmute"
        | "HassSetVolume"
        | "HassSetVolumeRelative"
        | "HassMediaSearchAndPlay"
        | "MassPlayMedia"
        | "MassTransferQueue"
        | "MassFavorite"
        | "MassGetQueue" => Some("media_player"),
        "HassFanSetSpeed" => Some("fan"),
        "HassVacuumStart" | "HassVacuumReturnToBase" => Some("vacuum"),
        "HassSetPosition" => Some("cover"),
        _ => None,
    }
}

fn allowed_domain(domain: &str) -> bool {
    matches!(
        domain,
        "light" | "switch" | "lock" | "cover" | "climate" | "fan" | "vacuum" | "media_player" | "scene" | "script" | "input_boolean"
    )
}

fn allowed_slots(name: &str) -> &'static [&'static str] {
    match name {
        "HassTurnOn" | "HassTurnOff" | "HassToggle" => &["entity_id", "area", "floor", "domain"],
        "HassGetState" => &["entity_id", "area", "floor", "domain", "media_status"],
        "HassLightSet" => &["entity_id", "area", "floor", "domain", "brightness", "color"],
        "HassClimateSetTemperature" => &["entity_id", "area", "floor", "domain", "temperature"],
        "HassClimateGetTemperature" => &["entity_id", "area", "floor", "domain"],
        "HassFanSetSpeed" => &["entity_id", "area", "floor", "domain", "percentage"],
        "HassSetPosition" => &["entity_id", "area", "floor", "domain", "position"],
        "HassSetVolume" => &["entity_id", "domain", "volume_level"],
        "HassSetVolumeRelative" => &["entity_id", "domain", "volume_step"],
        "HassMediaSearchAndPlay" => &["entity_id", "domain", "search_query", "media_id", "media_type", "media_class"],
        "MassPlayMedia" => {
            &["entity_id", "domain", "search_query", "media_id", "media_type", "media_class", "artist", "enqueue", "radio_mode"]
        }
        "MassTransferQueue" => &["entity_id", "domain", "target_entity_id", "source_player", "item"],
        "MassFavorite" => &["entity_id", "domain", "target_entity_id", "item"],
        "MassGetQueue" => &["entity_id", "domain", "target_entity_id", "item", "media_status"],
        "HassMediaPause" | "HassMediaUnpause" | "HassMediaNext" | "HassMediaPrevious" | "HassMediaPlayerMute" | "HassMediaPlayerUnmute" => {
            &["entity_id", "domain"]
        }
        "HassVacuumStart" | "HassVacuumReturnToBase" => &["entity_id", "area", "floor", "domain"],
        "HassStartTimer" | "HassIncreaseTimer" | "HassDecreaseTimer" => &["entity_id", "hours", "minutes", "seconds", "timer_name"],
        "HassCancelTimer" | "HassPauseTimer" => &["entity_id", "timer_name"],
        "HassListAddItem" | "HassListCompleteItem" | "HassShoppingListAddItem" | "HassShoppingListCompleteItem" => {
            &["entity_id", "name", "item"]
        }
        _ => &[],
    }
}

fn conflicting_slots(intent: &Intent) -> bool {
    let mut values = BTreeMap::new();
    intent
        .slots
        .iter()
        .any(|slot| values.insert(slot.name.as_str(), slot.value.as_str()).is_some_and(|existing| existing != slot.value.as_str()))
}

fn risky_intent(intent: &Intent) -> bool {
    if matches!(intent.name.as_str(), "HassGetState" | "HassClimateGetTemperature" | "HassTimerStatus" | "MassGetQueue") {
        return false;
    }
    let entity = intent.slot("entity_id").unwrap_or("");
    let domain = intent.slot("domain");
    if entity.starts_with("lock.") || domain == Some("lock") {
        return true;
    }
    let cover = entity.starts_with("cover.") || domain == Some("cover");
    if cover && matches!(intent.name.as_str(), "HassTurnOff" | "HassSetPosition") {
        return true;
    }
    (intent.slot("area").is_some() || intent.slot("floor").is_some())
        && domain.is_some_and(|value| matches!(value, "lock" | "cover" | "climate" | "fan" | "switch"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::types::PlanStep;

    fn plan(intent: Intent) -> IntentPlan {
        IntentPlan::from_intents(vec![intent], 1.0, &[])
    }

    #[test]
    fn rejects_targetless_controls() {
        assert_eq!(
            validate_plan(&plan(Intent::new("HassTurnOn").with("domain", "light")), &default_home()),
            Err(PlanInvalid::MissingTarget)
        );
    }

    #[test]
    fn rejects_nonexistent_and_unexposed_targets() {
        let home = default_home();
        assert_eq!(
            validate_plan(&plan(Intent::new("HassTurnOn").with("entity_id", "light.missing")), &home),
            Err(PlanInvalid::UnsafeTarget)
        );
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
        assert!(!requires_confirmation(&plan(
            Intent::new("HassClimateGetTemperature").with("area", "schlafzimmer").with("domain", "climate")
        )));
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
            PlanStep {
                index: 2,
                intent: Intent::new("HassTurnOn").with("entity_id", "lock.missing"),
                confidence: 0.8,
                evidence: Vec::new(),
            },
        ];
        let filtered = filter_valid_steps(&IntentPlan::from_steps(steps, 0.2), &home);
        assert_eq!(filtered.steps.len(), 1);
        assert_eq!(filtered.steps[0].intent.slot("entity_id"), Some("light.wohnzimmer"));
        assert_eq!(filtered.steps[0].index, 0);
    }
}
