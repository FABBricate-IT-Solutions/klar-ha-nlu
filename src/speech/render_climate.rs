//! Climate and floor-temperature lines after execute.

use crate::lang::Speech;
use crate::types::{SpeechEntity, SpeechSnapshot, UnitSystem};
use crate::units::{entity_temperature, speak_converted};

use super::render_place::slot;

pub(super) fn floor_temps(snap: &SpeechSnapshot, speech: Speech, entities: &[&SpeechEntity]) -> String {
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
        let Some(temp) = area_temp_fact(&rows, snap.unit_system, speech) else {
            continue;
        };
        let pretty = speech.room_name(&fold(label)).map_or_else(|| preserve_name(label), str::to_string);
        parts.push(format!("{pretty} {temp}."));
    }
    parts.join(" ")
}

pub(super) fn climate_query(snap: &SpeechSnapshot, entities: &[&SpeechEntity], speech: Speech) -> String {
    let area = slot(snap, "area_name").or_else(|| slot(snap, "area")).unwrap_or("");
    let unit = speech.unit_word(snap.unit_system);
    for entity in entities {
        if let Some((raw, ha)) = entity_temperature(entity) {
            let temp = speak_converted(raw, ha, snap.unit_system);
            let target = if area.is_empty() { entity.name.as_str() } else { area };
            return speech.temp_reading.replace("{target}", target).replace("{n}", &temp).replace("{unit}", unit).trim().to_string();
        }
        if entity.domain != "climate" && entity.domain != "weather" {
            continue;
        }
        let line = speech.entity_line(&entity.name, &entity.state);
        if entity.name.trim().is_empty() && line.trim().is_empty() {
            continue;
        }
        return line.trim().to_string();
    }
    String::new()
}

pub(super) fn area_temp_fact(entities: &[&SpeechEntity], unit_system: UnitSystem, speech: Speech) -> Option<String> {
    for entity in entities {
        let Some((raw, ha)) = entity_temperature(entity) else {
            continue;
        };
        let temp = speak_converted(raw, ha, unit_system);
        let unit = speech.unit_word(unit_system);
        return Some(format!("{temp} {unit}"));
    }
    None
}

fn preserve_name(raw: &str) -> String {
    raw.replace('_', " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fold(text: &str) -> String {
    text.to_lowercase().replace('ü', "u").replace('ä', "a").replace('ö', "o").replace('ß', "ss").replace(' ', "")
}
