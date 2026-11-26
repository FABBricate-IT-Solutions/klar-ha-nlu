//! Spoken replies for device_class list queries (window contacts and kin).

use super::render_place::{slot, speech_for};
use crate::lang::Speech;
use crate::types::SpeechSnapshot;

pub(super) fn window_list(snap: &SpeechSnapshot) -> Option<String> {
    let class = slot(snap, "device_class")?;
    if slot(snap, "response") != Some("list") {
        return None;
    }
    if !matches!(class, "window" | "door" | "motion" | "smoke") {
        return None;
    }
    let state = slot(snap, "state");
    let speech = speech_for(&snap.language);
    let of_class: Vec<_> = snap
        .entities
        .iter()
        .filter(|entity| entity.device_class.as_deref() == Some(class))
        .filter(|entity| entity.domain == "binary_sensor" || entity.domain == "sensor")
        .collect();
    let kind = kind(speech, class);
    if of_class.is_empty() {
        return Some(speech.list_none_found.replace("{kind}", &kind));
    }
    let names: Vec<_> = of_class
        .iter()
        .filter(|entity| state.is_none_or(|wanted| entity.state == wanted))
        .map(|entity| entity.name.as_str())
        .filter(|name| !name.is_empty())
        .collect();
    let joined = names.join(", ");
    if state.is_none() {
        return Some(speech.list_all.replace("{kind}", &kind).replace("{names}", &joined));
    }
    let spoken_state = match state {
        Some("on") | Some("open") => speech.state_open.to_string(),
        Some("off") | Some("closed") => speech.state_closed.to_string(),
        _ => speech.state_word(state.unwrap_or("")),
    };
    if names.is_empty() {
        return Some(speech.list_none_state.replace("{kind}", &kind).replace("{state}", &spoken_state));
    }
    // Window lists keep locale-specific wording in list_state; other classes use {kind}.
    let line = if class == "window" {
        speech.list_state.replace("{state}", &spoken_state).replace("{names}", &joined).replace("{kind}", &kind)
    } else {
        speech
            .list_state
            .replace("{kind}", &kind)
            .replace("{state}", &spoken_state)
            .replace("{names}", &joined)
            .replace("windows", kind.as_str())
            .replace("Fenster", kind.as_str())
    };
    Some(line)
}

fn kind(speech: Speech, class: &str) -> String {
    match class {
        "window" => speech.kind_window.to_string(),
        "door" => speech.kind_door.to_string(),
        "motion" => speech.kind_motion.to_string(),
        "smoke" => speech.kind_smoke.to_string(),
        _ => class.to_string(),
    }
}
