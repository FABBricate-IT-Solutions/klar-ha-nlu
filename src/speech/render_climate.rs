//! Climate and floor-temperature lines after execute.

use crate::lang::Speech;
use crate::types::{SpeechEntity, SpeechSnapshot, UnitSystem};
use crate::units::{entity_temperature, speak_converted, spoken_unit_word};

use super::render_place::{slot, speak_state};

pub(super) fn floor_temps(snap: &SpeechSnapshot, speech: Speech, entities: &[&SpeechEntity], de: bool) -> String {
    let mut groups: Vec<(String, Vec<&SpeechEntity>)> = Vec::new();
    for entity in entities {
        if entity.domain == "weather" {
            continue;
        }
        let key = entity
            .area_name
            .as_deref()
            .filter(|name| !name.is_empty())
            .or(entity.area.as_deref().filter(|name| !name.is_empty()))
            .unwrap_or("")
            .to_string();
        if let Some((_, rows)) = groups.iter_mut().find(|(name, _)| *name == key) {
            rows.push(*entity);
        } else {
            groups.push((key, vec![*entity]));
        }
    }
    let mut parts = Vec::new();
    for (name, rows) in groups {
        let label = if name.is_empty() {
            slot(snap, "area_name").or_else(|| slot(snap, "area")).or_else(|| slot(snap, "floor")).unwrap_or("")
        } else {
            name.as_str()
        };
        let Some(temp) = area_temp_fact(&rows, snap.unit_system, de) else {
            continue;
        };
        let pretty = speech.room_name(&fold(label)).map_or_else(|| title(label), str::to_string);
        parts.push(format!("{pretty} {temp}."));
    }
    parts.join(" ")
}

pub(super) fn climate_query(snap: &SpeechSnapshot, entities: &[&SpeechEntity], de: bool) -> String {
    let area = slot(snap, "area_name").or_else(|| slot(snap, "area")).unwrap_or("");
    let unit = spoken_unit_word(snap.unit_system, de);
    for entity in entities {
        if let Some((raw, ha)) = entity_temperature(entity) {
            let temp = speak_converted(raw, ha, snap.unit_system);
            if de {
                return format!("{area} {temp} {unit}.").trim().to_string();
            }
            return format!("{area} is {temp} {unit}.").trim().to_string();
        }
        if entity.domain != "climate" && entity.domain != "weather" {
            continue;
        }
        let spoken = speak_state(&entity.state, if de { "de" } else { "en" });
        if entity.name.trim().is_empty() && spoken.trim().is_empty() {
            continue;
        }
        if de {
            return format!("{} ist {spoken}.", entity.name).trim().to_string();
        }
        return format!("{} is {spoken}.", entity.name).trim().to_string();
    }
    String::new()
}

pub(super) fn area_temp_fact(entities: &[&SpeechEntity], unit_system: UnitSystem, de: bool) -> Option<String> {
    for entity in entities {
        let Some((raw, ha)) = entity_temperature(entity) else {
            continue;
        };
        let temp = speak_converted(raw, ha, unit_system);
        let unit = spoken_unit_word(unit_system, de);
        return Some(format!("{temp} {unit}"));
    }
    None
}

fn title(raw: &str) -> String {
    let text = raw.replace('_', " ");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn fold(text: &str) -> String {
    text.to_lowercase().replace('ü', "u").replace('ä', "a").replace('ö', "o").replace('ß', "ss").replace(' ', "")
}
