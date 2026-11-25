//! Spoken replies for device_class list queries (window contacts and kin).

use super::render_place::slot;
use crate::types::SpeechSnapshot;

pub(super) fn window_list(snap: &SpeechSnapshot) -> Option<String> {
    let class = slot(snap, "device_class")?;
    if slot(snap, "response") != Some("list") {
        return None;
    }
    // Binary contact classes share the open/closed list shape; value sensors fall through.
    if !matches!(class, "window" | "door" | "motion" | "smoke") {
        return None;
    }
    let state = slot(snap, "state");
    let of_class: Vec<_> = snap
        .entities
        .iter()
        .filter(|entity| entity.device_class.as_deref() == Some(class))
        .filter(|entity| entity.domain == "binary_sensor" || entity.domain == "sensor")
        .collect();
    // No visible sensors of this class at all — not the same as "none are open".
    if of_class.is_empty() {
        return Some(none_visible(&snap.language, class));
    }
    let names: Vec<_> = of_class
        .iter()
        .filter(|entity| state.is_none_or(|wanted| entity.state == wanted))
        .map(|entity| entity.name.as_str())
        .filter(|name| !name.is_empty())
        .collect();
    speak(&snap.language, class, state, &names)
}

fn none_visible(language: &str, class: &str) -> String {
    let base = language.split(['-', '_']).next().unwrap_or(language);
    match (base, class) {
        ("nl", "window") => "Er zijn geen raamsensoren gevonden.".into(),
        ("de", "window") => "Es wurden keine Fenstersensoren gefunden.".into(),
        (_, "window") => "No window sensors were found.".into(),
        ("nl", "door") => "Er zijn geen deursensoren gevonden.".into(),
        ("de", "door") => "Es wurden keine Türsensoren gefunden.".into(),
        (_, "door") => "No door sensors were found.".into(),
        ("nl", "motion") => "Er zijn geen bewegingssensoren gevonden.".into(),
        ("de", "motion") => "Es wurden keine Bewegungsmelder gefunden.".into(),
        (_, "motion") => "No motion sensors were found.".into(),
        ("nl", "smoke") => "Er zijn geen rookmelders gevonden.".into(),
        ("de", "smoke") => "Es wurden keine Rauchmelder gefunden.".into(),
        (_, "smoke") => "No smoke sensors were found.".into(),
        _ => "No sensors were found.".into(),
    }
}

fn speak(language: &str, class: &str, state: Option<&str>, names: &[&str]) -> Option<String> {
    let base = language.split(['-', '_']).next().unwrap_or(language);
    if class != "window" {
        return Some(generic(base, class, state, names));
    }
    Some(match base {
        "nl" => nl(state, names),
        "en" => en(state, names),
        "de" => de(state, names),
        _ => return None,
    })
}

fn generic(base: &str, class: &str, state: Option<&str>, names: &[&str]) -> String {
    if names.is_empty() {
        return match (base, class, state) {
            ("de", "door", Some("on")) => "Es sind keine Türen offen.".into(),
            ("de", "motion", Some("on")) => "Es wurde keine Bewegung erkannt.".into(),
            ("de", "smoke", Some("on")) => "Es wurde kein Rauch erkannt.".into(),
            (_, "door", Some("on")) => "No doors are open.".into(),
            (_, "door", Some("off")) => "No doors are closed.".into(),
            (_, "motion", Some("on")) => "No motion is detected.".into(),
            (_, "smoke", Some("on")) => "No smoke is detected.".into(),
            _ => format!("No {class} sensors match."),
        };
    }
    let joined = names.join(", ");
    match (base, class, state) {
        ("de", "door", Some("on")) => format!("Diese Türen sind offen: {joined}."),
        (_, "door", Some("on")) => format!("These doors are open: {joined}."),
        (_, "door", Some("off")) => format!("These doors are closed: {joined}."),
        (_, "motion", Some("on")) => format!("Motion detected: {joined}."),
        (_, "smoke", Some("on")) => format!("Smoke detected: {joined}."),
        _ => format!("{class}: {joined}."),
    }
}

fn nl(state: Option<&str>, names: &[&str]) -> String {
    if names.is_empty() {
        return match state {
            Some("on") => "Er staan geen ramen open.",
            Some("off") => "Er zijn geen gesloten ramen.",
            _ => "Er zijn geen raamsensoren gevonden.",
        }
        .into();
    }
    match state {
        Some("on") => format!("Deze ramen staan open: {}.", names.join(", ")),
        Some("off") => format!("Deze ramen zijn gesloten: {}.", names.join(", ")),
        _ => format!("Raamsensoren: {}.", names.join(", ")),
    }
}

fn en(state: Option<&str>, names: &[&str]) -> String {
    if names.is_empty() {
        return match state {
            Some("on") => "No windows are open.",
            Some("off") => "No windows are closed.",
            _ => "No window sensors were found.",
        }
        .into();
    }
    match state {
        Some("on") => format!("These windows are open: {}.", names.join(", ")),
        Some("off") => format!("These windows are closed: {}.", names.join(", ")),
        _ => format!("Window sensors: {}.", names.join(", ")),
    }
}

fn de(state: Option<&str>, names: &[&str]) -> String {
    if names.is_empty() {
        return match state {
            Some("on") => "Es sind keine Fenster offen.",
            Some("off") => "Es sind keine Fenster geschlossen.",
            _ => "Es wurden keine Fenstersensoren gefunden.",
        }
        .into();
    }
    match state {
        Some("on") => format!("Diese Fenster sind offen: {}.", names.join(", ")),
        Some("off") => format!("Diese Fenster sind geschlossen: {}.", names.join(", ")),
        _ => format!("Fenstersensoren: {}.", names.join(", ")),
    }
}
