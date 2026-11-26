//! Spoken floor/area status: locative heading, then comma-joined facts.

use crate::lang::Speech;
use crate::types::{SpeechEntity, SpeechSnapshot, UnitSystem};

use super::render_climate::area_temp_fact;
use super::render_place::{empty_place, slot};

const PRESENCE: &[&str] = &[
    "occupancy",
    "motion",
    "presence",
    "belegung",
    "präsenz",
    "prasenz",
    "anwesen",
    "bewegung",
    "occupat",
    "présence",
    "presenza",
    "presencia",
    "aanwezig",
];
const TEMP_NEEDLES: &[&str] = &["temperatur", "temperature", "température", "temperatura"];
const LIGHT_WORDS: &[&str] = &["licht", "light", "lampe", "lamp", "leuchte"];
const IDEO: &[&str] = &["zh-CN", "zh-TW", "zh-HK", "ja"];
const OFF: &[&str] = &["off", "not_home", "clear", "closed", "locked", "idle", "docked", "paused"];
const ON: &[&str] = &["on", "home", "detected", "open", "unlocked", "playing", "cleaning"];

pub(super) fn place_status(snap: &SpeechSnapshot, entities: &[&SpeechEntity], speech: Speech, pack: &str) -> String {
    if entities.is_empty() {
        return empty_place(pack);
    }
    let mut groups: Vec<(String, Vec<&SpeechEntity>)> = Vec::new();
    for entity in entities {
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
    if groups.len() == 1 && groups[0].0.is_empty() {
        let fallback = slot(snap, "area_name").or_else(|| slot(snap, "area")).or_else(|| slot(snap, "floor")).unwrap_or("");
        let line = area_status(fallback, entities, speech, pack, snap.unit_system);
        return if line.is_empty() { empty_place(pack) } else { line };
    }
    let mut parts = Vec::new();
    for (name, rows) in groups {
        let label = if name.is_empty() {
            slot(snap, "area_name").or_else(|| slot(snap, "area")).or_else(|| slot(snap, "floor")).unwrap_or("")
        } else {
            name.as_str()
        };
        let line = area_status(label, &rows, speech, pack, snap.unit_system);
        if !line.is_empty() {
            parts.push(line);
        }
    }
    if parts.is_empty() {
        return empty_place(pack);
    }
    let stop = if ideo(pack) { "。" } else { " " };
    parts.join(stop)
}

fn area_status(area: &str, entities: &[&SpeechEntity], speech: Speech, pack: &str, unit_system: UnitSystem) -> String {
    let pretty = speech.room_name(&fold(area)).map_or_else(|| preserve(area), str::to_string);
    let facts = area_facts(&pretty, entities, speech, unit_system);
    if facts.is_empty() {
        return String::new();
    }
    let heading = heading(&pretty, speech);
    if ideo(pack) {
        return format!("{heading}。{}", facts.join("，"));
    }
    if heading.eq_ignore_ascii_case(&pretty) {
        format!("{heading}: {}.", facts.join(", "))
    } else {
        format!("{heading} {}.", facts.join(", "))
    }
}

fn area_facts(area: &str, entities: &[&SpeechEntity], speech: Speech, unit_system: UnitSystem) -> Vec<String> {
    let lights: Vec<_> = entities.iter().copied().filter(|entity| entity.domain == "light").collect();
    let sockets: Vec<_> = entities.iter().copied().filter(|entity| class_of(entity) == "outlet").collect();
    let presence: Vec<_> = entities.iter().copied().filter(|entity| is_presence(entity)).collect();
    let mut facts = Vec::new();
    let mut generic_on = false;
    let mut generic_off = false;
    for entity in &lights {
        let label = device_label(entity, area, speech);
        if generic_light(&label, speech) {
            if is_on(entity) {
                generic_on = true;
            } else {
                generic_off = true;
            }
            continue;
        }
        facts.push(clause(&label, entity, speech));
    }
    if generic_on {
        facts.push(format!("{} {}", speech.light_noun, speech.state_word("on")));
    } else if generic_off {
        facts.push(format!("{} {}", speech.light_noun, speech.state_word("off")));
    }
    for entity in &sockets {
        let label = device_label(entity, area, speech);
        if !label.is_empty() {
            facts.push(clause(&label, entity, speech));
        }
    }
    if !presence.is_empty() {
        let occupied = presence.iter().any(|entity| is_on(entity));
        facts.push(if occupied { speech.presence_yes.into() } else { speech.presence_no.into() });
    }
    if let Some(temp) = area_temp_fact(entities, unit_system, speech) {
        facts.push(temp);
    }
    let have_temp = facts
        .iter()
        .any(|fact| fact.contains(speech.unit_c) || fact.contains(speech.unit_f) || fact.contains("Grad") || fact.contains("degree"));
    for entity in entities {
        if entity.domain == "light" || class_of(entity) == "outlet" || is_presence(entity) {
            continue;
        }
        if class_of(entity) == "illuminance" {
            continue;
        }
        if have_temp && entity.domain == "climate" && (is_off(entity) || entity.state.parse::<f64>().is_ok()) {
            continue;
        }
        if have_temp && looks_like_temp(entity) && entity.domain != "climate" {
            continue;
        }
        let label = device_label(entity, area, speech);
        if label.is_empty() {
            continue;
        }
        facts.push(clause(&label, entity, speech));
    }
    facts
}

fn heading(room: &str, speech: Speech) -> String {
    let folded = fold(room);
    if speech.loc_der_rooms.iter().any(|key| folded.contains(key)) || folded.ends_with('e') {
        return cap_first(&speech.loc_in_der.replace("{room}", room));
    }
    let loc = speech.loc_in.replace("{room}", room);
    if loc.trim() == room {
        return preserve(room);
    }
    cap_first(&loc)
}

fn device_label(entity: &SpeechEntity, area: &str, speech: Speech) -> String {
    let stripped = strip_area(&entity.name, area);
    if entity.domain == "light" && generic_light(&stripped, speech) {
        return speech.light_noun.to_string();
    }
    stripped
}

fn strip_area(name: &str, area: &str) -> String {
    let pretty = preserve(name);
    if area.is_empty() {
        return pretty;
    }
    let area_l = area.to_lowercase();
    let pretty_l = pretty.to_lowercase();
    if pretty_l == area_l {
        return String::new();
    }
    if let Some(rest) = pretty_l.strip_prefix(&(area_l + " ")) {
        return preserve(rest);
    }
    pretty
}

fn generic_light(label: &str, speech: Speech) -> bool {
    label.is_empty()
        || LIGHT_WORDS.iter().any(|word| label.to_lowercase().replace(' ', "") == *word)
        || fold(label) == fold(speech.light_noun)
}

fn clause(name: &str, entity: &SpeechEntity, speech: Speech) -> String {
    format!("{name} {}", speech.state_word(&entity.state)).trim().to_string()
}

fn is_presence(entity: &SpeechEntity) -> bool {
    let class = class_of(entity);
    if PRESENCE.iter().take(3).any(|item| class == *item) {
        return true;
    }
    let blob = format!("{} {}", entity.entity_id, entity.name).to_lowercase();
    PRESENCE.iter().any(|needle| blob.contains(needle))
}

fn looks_like_temp(entity: &SpeechEntity) -> bool {
    if entity.domain != "sensor" && entity.domain != "climate" {
        return false;
    }
    let blob = format!("{} {}", entity.entity_id, entity.name).to_lowercase();
    TEMP_NEEDLES.iter().any(|needle| blob.contains(needle))
}

fn class_of(entity: &SpeechEntity) -> String {
    entity.device_class.as_deref().unwrap_or("").to_lowercase()
}

fn is_on(entity: &SpeechEntity) -> bool {
    ON.contains(&entity.state.to_lowercase().as_str())
}

fn is_off(entity: &SpeechEntity) -> bool {
    OFF.contains(&entity.state.to_lowercase().as_str())
}

fn ideo(pack: &str) -> bool {
    IDEO.contains(&pack)
}

fn fold(text: &str) -> String {
    text.to_lowercase().replace('ü', "u").replace('ä', "a").replace('ö', "o").replace('ß', "ss").replace(' ', "")
}

fn preserve(raw: &str) -> String {
    raw.replace('_', " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cap_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
