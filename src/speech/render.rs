//! Factual post-execute lines from a sanitized snapshot.

use crate::lang::{LangId, Speech};
use crate::types::{SpeechEntity, SpeechRenderOut, SpeechSnapshot};
use crate::units::{entity_temp_scale, speak_temp, spoken_unit_word};

use super::render_climate::{climate_query, floor_temps};
use super::render_media::{media_action, media_status};
use super::render_place::{color_word, empty_place, slot, speak_state};

pub fn render_snapshot(snap: &SpeechSnapshot) -> SpeechRenderOut {
    let speech = pack_for(&snap.language);
    let de = is_de(&snap.language);
    let spoken = if snap.outcome == "error" { speech.unknown.to_string() } else { interpolate(snap, speech, de) };
    SpeechRenderOut { speech: spoken, quiet_ack: false, source: "post_execute" }
}

fn interpolate(snap: &SpeechSnapshot, speech: Speech, de: bool) -> String {
    let name = snap.intent.name.as_str();
    let where_ = pretty_where(snap, speech);
    if is_query(name) {
        return query_speech(snap, speech, de);
    }
    if let Some(line) = media_action(name, &where_, snap, de) {
        return line;
    }
    if let Some(line) = timer_action(name, speech) {
        return line;
    }
    match name {
        "HassTurnOn" if domain_of(snap) == "scene" => fill(speech.turn_on_scene, &where_, "", ""),
        "HassTurnOn" => fill(speech.turn_on, &where_, "", ""),
        "HassTurnOff" => fill(speech.turn_off, &where_, "", ""),
        "HassToggle" => fill(speech.toggle, &where_, "", ""),
        "HassLightSet" => light_set(snap, speech, &where_, de),
        "HassClimateSetTemperature" => climate_set(snap, speech, &where_, de),
        "HassVacuumStart" => fill(speech.vacuum_start, &where_, "", ""),
        "HassVacuumReturnToBase" => fill(speech.vacuum_dock, &where_, "", ""),
        "HassFanSetSpeed" | "HassFanSetPercentage" => fill(speech.fan_set, &where_, slot(snap, "percentage").unwrap_or(""), ""),
        "KlarGetCalendarEvents" | "KlarCreateCalendarEvent" | "KlarDeleteCalendarEvent" | "KlarMoveCalendarEvent" | "KlarNoMusicPlayer" => {
            calendar_speech(snap, speech)
        }
        _ => {
            if !where_.is_empty() {
                fill(speech.done, &where_, "", "")
            } else {
                speech.unknown.to_string()
            }
        }
    }
}

fn light_set(snap: &SpeechSnapshot, speech: Speech, where_: &str, de: bool) -> String {
    if let Some(color) = color_word(slot(snap, "color"), de) {
        return fill(speech.light_color, where_, "", &color);
    }
    let level = slot(snap, "brightness").or_else(|| slot(snap, "percentage")).unwrap_or("");
    fill(speech.light_set, where_, level, "")
}

fn climate_set(snap: &SpeechSnapshot, speech: Speech, where_: &str, de: bool) -> String {
    let Some(raw) = slot(snap, "temperature") else {
        return speech.unknown.to_string();
    };
    let ha = entity_temp_scale(snap.entities.iter().find(|entity| entity.domain == "climate" || entity.domain == "weather"));
    let temp = speak_temp(raw, ha, snap.unit_system);
    let unit = spoken_unit_word(snap.unit_system, de);
    if de {
        format!("{where_} auf {temp} {unit}.")
    } else {
        format!("{where_} is at {temp} {unit}.")
    }
}

fn timer_action(name: &str, speech: Speech) -> Option<String> {
    Some(match name {
        "HassStartTimer" | "HassIncreaseTimer" | "HassDecreaseTimer" => speech.timer_start.to_string(),
        "HassCancelTimer" => speech.timer_cancel.to_string(),
        "HassPauseTimer" => speech.timer_pause.to_string(),
        _ => return None,
    })
}

fn query_speech(snap: &SpeechSnapshot, speech: Speech, de: bool) -> String {
    let status = slot(snap, "media_status").unwrap_or("");
    if !status.is_empty() {
        return media_status(snap, status, de);
    }
    let entities: Vec<&SpeechEntity> = snap.entities.iter().filter(|entity| !is_infra(entity)).collect();
    if snap.intent.name == "HassClimateGetTemperature" {
        if slot(snap, "floor").is_some() {
            let line = floor_temps(snap, speech, &entities, de);
            return if line.is_empty() { empty_place(&snap.language) } else { line };
        }
        if entities.is_empty() {
            return String::new();
        }
        let line = climate_query(snap, &entities, de);
        if !line.is_empty() {
            return line;
        }
    }
    if is_place_query(snap, &entities) {
        return crate::speech::render_status::place_status(snap, &entities, speech, &snap.language);
    }
    if entities.iter().any(|entity| entity.domain == "climate" || entity.domain == "weather") {
        let line = climate_query(snap, &entities, de);
        if !line.is_empty() {
            return line;
        }
    }
    if entities.is_empty() {
        return String::new();
    }
    let lights: Vec<_> = entities.iter().filter(|entity| entity.domain == "light").copied().collect();
    if lights.len() >= 2 {
        return light_counts(&lights, de);
    }
    entities
        .iter()
        .take(4)
        .filter_map(|entity| {
            let spoken = speak_state(&entity.state, &snap.language);
            if entity.name.trim().is_empty() && spoken.trim().is_empty() {
                return None;
            }
            Some(if de {
                format!("{} ist {spoken}.", entity.name).trim().to_string()
            } else {
                format!("{} is {spoken}.", entity.name).trim().to_string()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn cap_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn light_counts(lights: &[&SpeechEntity], de: bool) -> String {
    let on = lights.iter().filter(|entity| entity.state == "on").count();
    let off = lights.iter().filter(|entity| entity.state == "off").count();
    if de {
        let mut bits = Vec::new();
        if on > 0 {
            bits.push(if on == 1 { "1 Licht an".into() } else { format!("{on} Lichter an") });
        }
        if off > 0 {
            bits.push(if off == 1 { "1 Licht aus".into() } else { format!("{off} Lichter aus") });
        }
        bits.join(", ") + "."
    } else {
        let mut bits = Vec::new();
        if on > 0 {
            bits.push(format!("{on} lights on"));
        }
        if off > 0 {
            bits.push(format!("{off} lights off"));
        }
        bits.join(", ") + "."
    }
}

fn calendar_speech(snap: &SpeechSnapshot, speech: Speech) -> String {
    let need = slot(snap, "need").unwrap_or("");
    let cue = slot(snap, "cue").unwrap_or("");
    if matches!(need, "title") || cue == "need_title" {
        return speech.calendar_need_title.to_string();
    }
    if matches!(need, "when") || cue == "need_when" {
        return speech.calendar_need_when.to_string();
    }
    if matches!(need, "which") || cue == "which" {
        return speech.calendar_which.to_string();
    }
    if matches!(cue, "none") {
        return speech.calendar_none.to_string();
    }
    if matches!(cue, "readonly") {
        return speech.calendar_readonly.to_string();
    }
    if matches!(cue, "no_uid") {
        return speech.calendar_no_uid.to_string();
    }
    match snap.intent.name.as_str() {
        "KlarNoMusicPlayer" => speech.no_music_player.to_string(),
        "KlarGetCalendarEvents" => calendar_line(snap, speech),
        "KlarCreateCalendarEvent" => fill_named(speech.calendar_created, snap),
        "KlarDeleteCalendarEvent" => fill_named(speech.calendar_deleted, snap),
        "KlarMoveCalendarEvent" => fill_named(speech.calendar_moved, snap),
        _ => calendar_line(snap, speech),
    }
}

fn calendar_line(snap: &SpeechSnapshot, speech: Speech) -> String {
    if snap.calendar_events.is_empty() {
        return speech.calendar_empty.to_string();
    }
    let items = snap
        .calendar_events
        .iter()
        .map(|event| if event.start.is_empty() { event.summary.clone() } else { format!("{} {}", event.summary, event.start) })
        .collect::<Vec<_>>()
        .join(". ");
    speech.calendar_list.replace("{items}", &items)
}

fn fill_named(template: &str, snap: &SpeechSnapshot) -> String {
    template.replace("{summary}", slot(snap, "summary").unwrap_or("")).replace("{when}", slot(snap, "when").unwrap_or(""))
}

fn pretty_where(snap: &SpeechSnapshot, speech: Speech) -> String {
    let media = is_media(snap);
    let name_slot = slot(snap, "name").filter(|name| !name.is_empty() && !name.contains('.'));
    if let Some(name) = name_slot.filter(|name| !generic_light(name) || media) {
        return spoken_device(name, slot(snap, "entity_id").unwrap_or(""), speech);
    }
    let room = room_from_id(slot(snap, "entity_id").or_else(|| snap.entities.first().map(|entity| entity.entity_id.as_str())), speech)
        .or_else(|| slot(snap, "area_name").or_else(|| slot(snap, "area")).map(|area| roomish(area, speech)));
    let entity_name = snap.entities.iter().map(|entity| entity.name.as_str()).find(|name| !name.is_empty() && !name.contains('.'));
    if let (Some(room), true) = (room.as_deref(), !media) {
        if entity_name.is_some_and(generic_light)
            || name_slot.is_some_and(generic_light)
            || (entity_name.is_none() && domain_of(snap) == "light")
        {
            return area_light_phrase(room, speech);
        }
    }
    if let Some(name) = name_slot {
        return spoken_device(name, slot(snap, "entity_id").unwrap_or(""), speech);
    }
    if let Some(name) = entity_name {
        return spoken_device(name, slot(snap, "entity_id").unwrap_or(""), speech);
    }
    if let Some(room) = room.filter(|_| !media) {
        return area_light_phrase(&room, speech);
    }
    if let Some(entity_id) = slot(snap, "entity_id") {
        return spoken_device("", entity_id, speech);
    }
    speech.or_home.to_string()
}

fn is_media(snap: &SpeechSnapshot) -> bool {
    snap.intent.name.starts_with("HassMedia")
        || snap.intent.name.starts_with("Mass")
        || domain_of(snap) == "media_player"
        || slot(snap, "entity_id").is_some_and(|id| id.starts_with("media_player."))
}

fn generic_light(name: &str) -> bool {
    matches!(name.trim().to_lowercase().as_str(), "licht" | "light" | "lampe" | "lamp" | "leuchte")
}

fn light_word(name: &str) -> bool {
    let folded = fold(name);
    ["licht", "light", "lampe", "lamp", "leuchte"].iter().any(|token| folded.contains(token))
}

fn area_light_phrase(room: &str, speech: Speech) -> String {
    let folded = fold(room);
    let loc = if speech.loc_der_rooms.iter().any(|key| folded.contains(key)) || folded.ends_with('e') {
        speech.loc_in_der.replace("{room}", room)
    } else {
        speech.loc_in.replace("{room}", room)
    };
    speech.area_light.replace("{loc}", &loc)
}

fn spoken_device(name: &str, entity_id: &str, speech: Speech) -> String {
    let domain = entity_id.split_once('.').map(|(head, _)| head).unwrap_or("");
    let mut pretty = name.trim().to_string();
    if pretty.is_empty() || pretty.contains('.') {
        pretty = entity_id.split_once('.').map(|(_, tail)| tail.replace('_', " ")).unwrap_or_else(|| entity_id.replace('_', " "));
    }
    if pretty.is_empty() {
        return String::new();
    }
    pretty = cap_first(&pretty);
    if domain == "light" && !light_word(&pretty) {
        pretty.push_str(speech.light_suffix);
    }
    pretty
}

fn room_from_id(entity_id: Option<&str>, speech: Speech) -> Option<String> {
    let id = entity_id?;
    let folded = fold(id);
    speech.room_names.iter().find(|(key, _)| folded.contains(key)).map(|(_, name)| (*name).to_string())
}

fn roomish(raw: &str, speech: Speech) -> String {
    let folded = fold(raw);
    if let Some(name) = speech.room_name(&folded) {
        return name.to_string();
    }
    for (key, name) in speech.room_names {
        if folded.contains(key) {
            return (*name).to_string();
        }
    }
    raw.to_string()
}

fn fill(template: &str, target: &str, n: &str, color: &str) -> String {
    template.replace("{target}", target).replace("{n}", n).replace("{color}", color).replace("{name}", target).replace("{loc}", target)
}

fn domain_of(snap: &SpeechSnapshot) -> &str {
    snap.entities
        .first()
        .map(|entity| entity.domain.as_str())
        .unwrap_or_else(|| slot(snap, "entity_id").and_then(|id| id.split_once('.')).map(|(domain, _)| domain).unwrap_or(""))
}

fn is_query(name: &str) -> bool {
    matches!(name, "HassGetState" | "HassClimateGetTemperature")
}

fn is_place_query(snap: &SpeechSnapshot, entities: &[&SpeechEntity]) -> bool {
    if slot(snap, "entity_id").is_some() {
        return false;
    }
    slot(snap, "floor").is_some()
        || slot(snap, "area").is_some()
        || slot(snap, "area_name").is_some()
        || entities.iter().any(|entity| entity.area_name.as_ref().is_some_and(|name| !name.is_empty()))
}

fn is_de(pack: &str) -> bool {
    pack == "de" || pack.starts_with("de-")
}

fn pack_for(language: &str) -> Speech {
    LangId::from_tag(language).or_else(|| LangId::from_code(language)).unwrap_or(LangId::En).pack().speech
}

fn fold(text: &str) -> String {
    text.to_lowercase().replace('ü', "u").replace('ä', "a").replace('ö', "o").replace('ß', "ss").replace(' ', "")
}

fn is_infra(entity: &SpeechEntity) -> bool {
    let blob = format!("{} {}", entity.entity_id, entity.name).to_lowercase();
    [
        "satellite",
        "led_ring",
        "ledring",
        "cpu_temperature",
        "processor_temperature",
        "assist_satellite",
        "adaptive_lighting",
        "voice_led",
        "wake_sound",
        "child_lock",
    ]
    .iter()
    .any(|needle| blob.contains(needle))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
