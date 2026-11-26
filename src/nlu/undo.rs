//! Undo of the last execute: invert binary actions, or restore the prior value
//! for brightness, position, temperature, volume and fan speed (plan 6.5).

use crate::types::Intent;

/// Intent names that set a continuous value and need the prior stamped by HA.
pub(crate) const VALUE_INTENTS: &[&str] =
    &["HassLightSet", "HassSetPosition", "HassClimateSetTemperature", "HassSetVolume", "HassSetVolumeRelative", "HassFanSetSpeed"];

/// True when this intent belongs in `last_execute` so a later undo (after priors)
/// or an immediate relative invert can reverse it.
pub(crate) fn remembers_for_undo(intent: &Intent) -> bool {
    invert_intent(intent).is_some() || VALUE_INTENTS.contains(&intent.name.as_str())
}

/// Build the intent that puts the device back. Continuous absolute sets need a
/// `prior_*` slot (stamped from live HA state after the forward action).
pub(crate) fn invert_intent(intent: &Intent) -> Option<Intent> {
    if let Some(restored) = restore_value(intent) {
        return Some(restored);
    }
    let name = match intent.name.as_str() {
        "HassTurnOn" => "HassTurnOff",
        "HassTurnOff" => "HassTurnOn",
        "HassToggle" => "HassToggle",
        "HassMediaPause" => "HassMediaUnpause",
        "HassMediaUnpause" => "HassMediaPause",
        "HassMediaPlayerMute" => "HassMediaPlayerUnmute",
        "HassMediaPlayerUnmute" => "HassMediaPlayerMute",
        "HassVacuumStart" => "HassVacuumReturnToBase",
        "HassStartTimer" => "HassCancelTimer",
        "HassOpenCover" => "HassCloseCover",
        "HassCloseCover" => "HassOpenCover",
        "HassLock" => "HassUnlock",
        "HassUnlock" => "HassLock",
        _ => return None,
    };
    Some(Intent { name: name.into(), slots: intent.slots.clone() })
}

fn restore_value(intent: &Intent) -> Option<Intent> {
    let entity = intent.slot("entity_id")?;
    match intent.name.as_str() {
        "HassLightSet" => restore_light(intent, entity),
        "HassSetPosition" => {
            let position = intent.slot("prior_position")?;
            Some(Intent::new("HassSetPosition").with("entity_id", entity).with("position", position))
        }
        "HassClimateSetTemperature" => restore_climate(intent, entity),
        "HassSetVolume" => {
            let level = intent.slot("prior_volume_level")?;
            Some(Intent::new("HassSetVolume").with("entity_id", entity).with("volume_level", level))
        }
        "HassSetVolumeRelative" => restore_volume_relative(intent, entity),
        "HassFanSetSpeed" => {
            let pct = intent.slot("prior_percentage")?;
            Some(Intent::new("HassFanSetSpeed").with("entity_id", entity).with("percentage", pct))
        }
        _ => None,
    }
}

fn restore_light(intent: &Intent, entity: &str) -> Option<Intent> {
    if let Some(prior) = intent.slot("prior_brightness") {
        let mut out = Intent::new("HassLightSet").with("entity_id", entity).with("brightness", prior);
        if let Some(color) = intent.slot("prior_color") {
            out = out.with("color", color);
        }
        return Some(out);
    }
    if let Some(step) = intent.slot("brightness_step") {
        let flipped = flip_step(step)?;
        return Some(Intent::new("HassLightSet").with("entity_id", entity).with("brightness_step", flipped));
    }
    if let Some(color) = intent.slot("prior_color") {
        return Some(Intent::new("HassLightSet").with("entity_id", entity).with("color", color));
    }
    None
}

fn restore_climate(intent: &Intent, entity: &str) -> Option<Intent> {
    if let Some(prior) = intent.slot("prior_temperature") {
        return Some(Intent::new("HassClimateSetTemperature").with("entity_id", entity).with("temperature", prior));
    }
    let step = intent.slot("temperature_step")?;
    let flipped = negate_number(step)?;
    Some(Intent::new("HassClimateSetTemperature").with("entity_id", entity).with("temperature_step", flipped))
}

fn restore_volume_relative(intent: &Intent, entity: &str) -> Option<Intent> {
    if let Some(prior) = intent.slot("prior_volume_level") {
        return Some(Intent::new("HassSetVolume").with("entity_id", entity).with("volume_level", prior));
    }
    let step = intent.slot("volume_step")?;
    let flipped = flip_step(step)?;
    Some(Intent::new("HassSetVolumeRelative").with("entity_id", entity).with("volume_step", flipped))
}

fn flip_step(step: &str) -> Option<&'static str> {
    match step {
        "up" => Some("down"),
        "down" => Some("up"),
        _ => None,
    }
}

fn negate_number(raw: &str) -> Option<String> {
    let value: f64 = raw.parse().ok()?;
    if value == 0.0 {
        return None;
    }
    Some((-value).to_string())
}

/// Attach captured HA priors onto matching intents in `last_execute`.
pub(crate) fn stamp_priors(intents: &mut [Intent], priors: &[UndoPrior]) {
    for intent in intents {
        let Some(entity) = intent.slot("entity_id").map(str::to_string) else {
            continue;
        };
        let Some(prior) = priors.iter().find(|row| row.entity_id == entity) else {
            continue;
        };
        stamp_one(intent, prior);
    }
}

fn stamp_one(intent: &mut Intent, prior: &UndoPrior) {
    match intent.name.as_str() {
        "HassLightSet" => {
            if let Some(bri) = prior.brightness {
                *intent = intent.clone().with_set("prior_brightness", bri.to_string());
            }
            if let Some(color) = prior.color.as_deref().filter(|c| !c.is_empty()) {
                *intent = intent.clone().with_set("prior_color", color);
            }
        }
        "HassSetPosition" => {
            if let Some(position) = prior.position {
                *intent = intent.clone().with_set("prior_position", position.to_string());
            }
        }
        "HassClimateSetTemperature" => {
            if let Some(temp) = prior.temperature.as_deref().filter(|t| !t.is_empty()) {
                *intent = intent.clone().with_set("prior_temperature", temp);
            }
        }
        "HassSetVolume" | "HassSetVolumeRelative" => {
            if let Some(level) = prior.volume_level {
                *intent = intent.clone().with_set("prior_volume_level", level.to_string());
            }
        }
        "HassFanSetSpeed" => {
            if let Some(pct) = prior.percentage {
                *intent = intent.clone().with_set("prior_percentage", pct.to_string());
            }
        }
        _ => {}
    }
}

/// Prior continuous values captured from live HA state before the forward action ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoPrior {
    pub entity_id: String,
    pub brightness: Option<u8>,
    pub position: Option<u8>,
    pub temperature: Option<String>,
    pub volume_level: Option<u8>,
    pub percentage: Option<u8>,
    pub color: Option<String>,
}

impl UndoPrior {
    pub fn for_entity(entity_id: impl Into<String>) -> Self {
        Self {
            entity_id: entity_id.into(),
            brightness: None,
            position: None,
            temperature: None,
            volume_level: None,
            percentage: None,
            color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_invert_still_works() {
        assert_eq!(invert_intent(&Intent::new("HassTurnOn").with("entity_id", "light.a")).unwrap().name, "HassTurnOff");
        assert_eq!(invert_intent(&Intent::new("HassLock").with("entity_id", "lock.a")).unwrap().name, "HassUnlock");
        assert!(invert_intent(&Intent::new("HassGetState").with("entity_id", "lock.a")).is_none());
    }

    #[test]
    fn absolute_value_needs_prior() {
        assert!(invert_intent(&Intent::new("HassLightSet").with("entity_id", "light.a").with("brightness", "30")).is_none());
        assert!(invert_intent(&Intent::new("HassSetPosition").with("entity_id", "cover.a").with("position", "40")).is_none());
    }

    #[test]
    fn restores_brightness_from_prior() {
        let forward = Intent::new("HassLightSet").with("entity_id", "light.a").with("brightness", "30").with("prior_brightness", "80");
        let back = invert_intent(&forward).unwrap();
        assert_eq!(back.name, "HassLightSet");
        assert_eq!(back.slot("brightness"), Some("80"));
        assert_eq!(back.slot("entity_id"), Some("light.a"));
    }

    #[test]
    fn flips_brightness_step_without_prior() {
        let forward = Intent::new("HassLightSet").with("entity_id", "light.a").with("brightness_step", "up");
        let back = invert_intent(&forward).unwrap();
        assert_eq!(back.slot("brightness_step"), Some("down"));
    }

    #[test]
    fn restores_temperature_position_volume_fan() {
        let climate = Intent::new("HassClimateSetTemperature")
            .with("entity_id", "climate.a")
            .with("temperature", "22")
            .with("prior_temperature", "19");
        assert_eq!(invert_intent(&climate).unwrap().slot("temperature"), Some("19"));

        let cover = Intent::new("HassSetPosition").with("entity_id", "cover.a").with("position", "10").with("prior_position", "70");
        assert_eq!(invert_intent(&cover).unwrap().slot("position"), Some("70"));

        let volume =
            Intent::new("HassSetVolume").with("entity_id", "media_player.a").with("volume_level", "20").with("prior_volume_level", "55");
        assert_eq!(invert_intent(&volume).unwrap().slot("volume_level"), Some("55"));

        let fan = Intent::new("HassFanSetSpeed").with("entity_id", "fan.a").with("percentage", "100").with("prior_percentage", "40");
        assert_eq!(invert_intent(&fan).unwrap().slot("percentage"), Some("40"));
    }

    #[test]
    fn stamp_priors_attaches_slots() {
        let mut intents = vec![Intent::new("HassLightSet").with("entity_id", "light.a").with("brightness", "30")];
        let prior = UndoPrior { brightness: Some(75), ..UndoPrior::for_entity("light.a") };
        stamp_priors(&mut intents, &[prior]);
        assert_eq!(intents[0].slot("prior_brightness"), Some("75"));
        assert_eq!(invert_intent(&intents[0]).unwrap().slot("brightness"), Some("75"));
    }
}
