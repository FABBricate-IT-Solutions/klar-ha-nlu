use super::schema::{Case, Condition};
use klar_nlu::types::{HomeGraph, Intent};

pub(super) fn result_ok(case: &Case, intents: &[Intent], clarify: bool, home: &HomeGraph) -> Result<(), String> {
    if clarify {
        if case.conditions.iter().any(|condition| cond_ok(condition, intents, home).is_ok()) {
            Ok(())
        } else {
            Err(format!(
                "wanted one of {:?} in {intents:?}",
                case.conditions.iter().filter_map(|condition| condition.entity_id.as_deref()).collect::<Vec<_>>()
            ))
        }
    } else {
        for condition in &case.conditions {
            cond_ok(condition, intents, home)?;
        }
        Ok(())
    }
}

pub(super) fn forbid_ok(intents: &[Intent], forbidden: &[String]) -> Result<(), String> {
    for bad in forbidden {
        if intents.iter().any(|intent| intent.slot("entity_id") == Some(bad) || intent.slot("area") == Some(bad)) {
            return Err(format!("forbid {bad} in {intents:?}"));
        }
    }
    Ok(())
}

pub(super) fn speech_ok(speech: &str, required: &[String], forbidden: &[String]) -> Result<(), String> {
    let folded = speech.to_lowercase();
    for needle in required {
        if !folded.contains(&needle.to_lowercase()) {
            return Err(format!("speech missing {needle:?} in {speech:?}"));
        }
    }
    for needle in forbidden {
        if folded.contains(&needle.to_lowercase()) {
            return Err(format!("speech has {needle:?} in {speech:?}"));
        }
    }
    Ok(())
}

fn cond_ok(condition: &Condition, intents: &[Intent], home: &HomeGraph) -> Result<(), String> {
    if scene_covers(condition, intents, home) {
        return Ok(());
    }
    let wanted = expected_intent_names(condition);
    if intents.iter().any(|intent| wanted.contains(&intent.name.as_str()) && target_ok(intent, condition, home)) {
        Ok(())
    } else {
        Err(format!("wanted {wanted:?} {:?} / {:?} in {intents:?}", condition.entity_id, condition.area))
    }
}

fn cond_attr<'a>(condition: &'a Condition, key: &str) -> Option<&'a serde_yaml::Value> {
    condition.attributes.get(key).or_else(|| condition.extra.get(key))
}

fn expected_intent_names(condition: &Condition) -> Vec<&'static str> {
    if cond_attr(condition, "temperature").is_some() {
        return vec!["HassClimateSetTemperature"];
    }
    if cond_attr(condition, "brightness").is_some() || cond_attr(condition, "color").is_some() {
        return vec!["HassLightSet"];
    }
    if cond_attr(condition, "percentage").is_some() {
        return vec!["HassFanSetSpeed"];
    }
    if cond_attr(condition, "position").is_some() {
        return vec!["HassSetPosition"];
    }
    if cond_attr(condition, "search_query").is_some() || cond_attr(condition, "media_id").is_some() {
        return vec!["HassMediaSearchAndPlay", "MassPlayMedia"];
    }
    if cond_attr(condition, "volume_level").is_some() || cond_attr(condition, "volume_step").is_some() {
        return vec!["HassSetVolume", "HassSetVolumeRelative"];
    }
    if cond_attr(condition, "is_volume_muted").is_some() {
        return vec!["HassMediaPlayerMute", "HassMediaPlayerUnmute", "HassMediaUnpause", "HassTurnOn"];
    }
    if condition.kind == "query" {
        return vec!["HassGetState", "HassClimateGetTemperature"];
    }
    if condition.kind == "shopping_list" || condition.kind == "todo_list" || condition.item.is_some() {
        return vec!["HassListAddItem", "HassListCompleteItem", "HassShoppingListAddItem", "HassShoppingListCompleteItem"];
    }
    if condition.minutes.is_some()
        || condition.hours.is_some()
        || condition.seconds.is_some()
        || condition.entity_id.as_deref().is_some_and(|entity| entity.starts_with("timer."))
    {
        return vec!["HassStartTimer", "HassIncreaseTimer", "HassDecreaseTimer", "HassTimerStatus", "HassPauseTimer", "HassCancelTimer"];
    }
    let entity = condition.entity_id.as_deref().unwrap_or("");
    if entity.starts_with("vacuum.") {
        return vec!["HassVacuumStart", "HassVacuumReturnToBase", "HassTurnOn"];
    }
    if entity.starts_with("scene.") || entity.starts_with("script.") {
        return vec!["HassTurnOn"];
    }
    match condition.state.as_deref() {
        Some("paused") => vec!["HassMediaPause"],
        Some("playing") => vec!["HassMediaUnpause", "HassTurnOn"],
        Some("off" | "closed" | "unlocked") => vec!["HassTurnOff"],
        Some("open" | "locked") => vec!["HassTurnOn"],
        _ => vec!["HassTurnOn"],
    }
}

fn scene_covers(condition: &Condition, intents: &[Intent], home: &HomeGraph) -> bool {
    let Some(wanted) = condition.entity_id.as_deref() else {
        return false;
    };
    intents.iter().any(|intent| {
        let Some(scene_id) = intent.slot("entity_id") else {
            return false;
        };
        (scene_id == wanted && (scene_id.starts_with("scene.") || scene_id.starts_with("script.")))
            || home.scene_members.get(scene_id).is_some_and(|members| members.iter().any(|member| member == wanted))
    })
}

fn target_ok(intent: &Intent, condition: &Condition, home: &HomeGraph) -> bool {
    if let Some(wanted) = condition.entity_id.as_deref() {
        if intent.slot("entity_id") == Some(wanted) {
            return slot_attrs_ok(intent, condition);
        }
        if let Some(entity) = entity_in(home, wanted) {
            let area_hit = intent.slot("area") == entity.area.as_deref();
            let domain_hit = intent.slot("domain").is_none_or(|domain| domain == entity.domain);
            return area_hit && domain_hit && slot_attrs_ok(intent, condition);
        }
        return false;
    }
    if let Some(area) = condition.area.as_deref() {
        if intent.slot("area") == Some(area) {
            if let Some(domain) = condition.domain.as_deref() {
                if intent.slot("domain").is_some_and(|got| got != domain) {
                    return intent.slot("entity_id").is_some_and(|entity_id| {
                        entity_in(home, entity_id).is_some_and(|entity| entity.area.as_deref() == Some(area) && entity.domain == domain)
                    }) && slot_attrs_ok(intent, condition);
                }
            }
            return slot_attrs_ok(intent, condition);
        }
        return intent.slot("entity_id").is_some_and(|entity_id| {
            entity_in(home, entity_id).is_some_and(|entity| {
                entity.area.as_deref() == Some(area) && condition.domain.as_deref().is_none_or(|domain| entity.domain == domain)
            })
        }) && slot_attrs_ok(intent, condition);
    }
    slot_attrs_ok(intent, condition)
}

fn entity_in<'a>(home: &'a HomeGraph, entity_id: &str) -> Option<&'a klar_nlu::types::EntityRec> {
    home.entities.iter().find(|entity| entity.entity_id == entity_id)
}

fn slot_attrs_ok(intent: &Intent, condition: &Condition) -> bool {
    for key in [
        "temperature",
        "brightness",
        "percentage",
        "color",
        "position",
        "search_query",
        "media_id",
        "media_type",
        "media_class",
        "artist",
        "enqueue",
        "radio_mode",
        "volume_step",
    ] {
        if let Some(value) = cond_attr(condition, key) {
            let Some(wanted) = scalar(value) else {
                return false;
            };
            return intent.slot(key) == Some(wanted.as_str());
        }
    }
    true
}

fn scalar(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::Null => Some("null".into()),
        serde_yaml::Value::Bool(value) => Some(value.to_string()),
        serde_yaml::Value::Number(value) => Some(value.to_string()),
        serde_yaml::Value::String(value) => Some(value.clone()),
        _ => None,
    }
}
