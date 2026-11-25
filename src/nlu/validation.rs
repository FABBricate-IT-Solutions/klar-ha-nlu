use crate::home::expose::assist_visible;
use crate::types::{HomeGraph, Intent, IntentPlan, PlanStep, MAX_PLAN_STEPS};
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

pub(super) fn fail_closed_intent(intent: &Intent) -> bool {
    let entity = intent.slot("entity_id").unwrap_or("");
    let domain = intent.slot("domain");
    entity.starts_with("lock.")
        || entity.starts_with("cover.")
        || entity.starts_with("script.")
        || matches!(domain, Some("lock" | "cover" | "script"))
        || super::reach::unbounded(intent)
}

pub(super) fn missing_timer_duration(plan: &IntentPlan, text: &str) -> bool {
    let tokens = crate::parse::normalize::tokenize(text);
    plan.steps.iter().any(|step| crate::parse::slots::needs_timer_duration_prompt(&step.intent, &tokens))
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

fn validate_intent(intent: &Intent, home: &HomeGraph) -> Result<(), PlanInvalid> {
    if !home.allows_intent(&intent.name) || conflicting_slots(intent) {
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
    if matches!(domain, Some("binary_sensor" | "weather")) && intent.name != "HassGetState" {
        return Err(PlanInvalid::Schema);
    }
    let window_query = intent.name == "HassGetState" && domain == Some("binary_sensor") && intent.slot("device_class") == Some("window");
    if requires_target(&intent.name) && !window_query && entity_id.is_none() && area.is_none() && floor.is_none() {
        return Err(PlanInvalid::MissingTarget);
    }
    if super::reach::unbounded(intent) {
        return Err(PlanInvalid::UnsafeTarget);
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
    // A forecast question carries what was asked ("rain", "tomorrow", "evening") for the reply.
    let forecast = intent.name == "HassGetState" && intent.slot("domain") == Some("weather");
    let known = |name: &str| allowed.contains(&name) || (forecast && matches!(name, "weather_ask" | "weather_day" | "weather_part"));
    if intent
        .slots
        .iter()
        .any(|slot| slot.name.len() > 64 || slot.value.is_empty() || slot.value.chars().count() > 512 || !known(&slot.name))
    {
        return Err(PlanInvalid::Schema);
    }
    match intent.name.as_str() {
        "HassLightSet"
            if intent.slot("brightness").is_none()
                && intent.slot("color").is_none()
                && !matches!(intent.slot("brightness_step"), Some("up" | "down")) =>
        {
            Err(PlanInvalid::Schema)
        }
        "HassClimateSetTemperature" if intent.slot("temperature").is_none() && intent.slot("temperature_step").is_none() => {
            Err(PlanInvalid::Schema)
        }
        "HassFanSetSpeed" if intent.slot("percentage").is_none() => Err(PlanInvalid::Schema),
        "HassSetPosition" if intent.slot("position").is_none() => Err(PlanInvalid::Schema),
        "HassSetVolume" if intent.slot("volume_level").is_none() => Err(PlanInvalid::Schema),
        "HassSetVolumeRelative" if !matches!(intent.slot("volume_step"), Some("up" | "down")) => Err(PlanInvalid::Schema),
        "HassMediaSearchAndPlay" | "MassPlayMedia" if intent.slot("search_query").is_none() && intent.slot("media_id").is_none() => {
            Err(PlanInvalid::Schema)
        }
        "HassListAddItem" | "HassListCompleteItem" | "HassListRemoveItem" | "HassShoppingListAddItem" | "HassShoppingListCompleteItem"
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
            // Reject absurd set-points like 99 °C without clipping (P-20). Hot water ~55 stays valid.
            "temperature" => Some((-40.0, 90.0)),
            "temperature_step" => Some((-15.0, 15.0)),
            "hours" | "hour" => Some((0.0, 24.0)),
            "in_days" => Some((1.0, 14.0)),
            // Zero-length timers are invalid (P-06).
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
    if matches!(intent.name.as_str(), "HassStartTimer" | "HassIncreaseTimer" | "HassDecreaseTimer") {
        let total = ["hours", "minutes", "seconds"].iter().filter_map(|k| intent.slot(k)).count();
        let secs = intent.slot("hours").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) * 3600.0
            + intent.slot("minutes").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) * 60.0
            + intent.slot("seconds").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        if total > 0 && secs <= 0.0 {
            return Err(PlanInvalid::Schema);
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
            | "HassListRemoveItem"
            | "HassShoppingListAddItem"
            | "HassShoppingListCompleteItem"
            | "KlarGetCalendarEvents"
            | "KlarCreateCalendarEvent"
            | "KlarDeleteCalendarEvent"
            | "KlarMoveCalendarEvent"
            | "KlarNoMusicPlayer"
            | "SetReminderRelative"
            | "SetReminderAbsolute"
            | "SetReminderDaypart"
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
        "light"
            | "binary_sensor"
            | "switch"
            | "lock"
            | "cover"
            | "climate"
            | "fan"
            | "vacuum"
            | "media_player"
            | "scene"
            | "script"
            | "input_boolean"
            | "calendar"
            | "weather"
    )
}

fn allowed_slots(name: &str) -> &'static [&'static str] {
    match name {
        "HassTurnOn" | "HassTurnOff" | "HassToggle" => &["entity_id", "area", "floor", "domain"],
        "HassGetState" => &["entity_id", "area", "floor", "domain", "media_status", "device_class", "state", "response"],
        "HassLightSet" => &["entity_id", "area", "floor", "domain", "brightness", "color", "brightness_step"],
        "HassClimateSetTemperature" => &["entity_id", "area", "floor", "domain", "temperature", "temperature_step"],
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
        "HassListAddItem" | "HassListCompleteItem" | "HassListRemoveItem" | "HassShoppingListAddItem" | "HassShoppingListCompleteItem" => {
            &["entity_id", "name", "item"]
        }
        "KlarGetCalendarEvents" => &["entity_id", "domain", "day", "hour", "in_days"],
        "KlarCreateCalendarEvent" | "KlarDeleteCalendarEvent" | "KlarMoveCalendarEvent" => {
            &["entity_id", "domain", "summary", "day", "hour", "in_days", "need"]
        }
        "KlarNoMusicPlayer" => &[],
        "SetReminderRelative" => &["count", "unit", "task"],
        "SetReminderAbsolute" | "SetReminderDaypart" => &["hour", "minute", "meridiem", "hourshift", "dayref", "task"],
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

pub(super) fn risky_intent(intent: &Intent) -> bool {
    if matches!(
        intent.name.as_str(),
        "HassGetState" | "HassClimateGetTemperature" | "HassTimerStatus" | "MassGetQueue" | "KlarGetCalendarEvents" | "KlarNoMusicPlayer"
    ) {
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
#[path = "validation_tests.rs"]
mod tests;
