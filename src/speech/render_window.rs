use super::render_place::slot;
use crate::types::SpeechSnapshot;

pub(super) fn window_list(snap: &SpeechSnapshot) -> Option<String> {
    if slot(snap, "device_class") != Some("window") || slot(snap, "response") != Some("list") {
        return None;
    }
    let state = slot(snap, "state");
    let names: Vec<_> = snap
        .entities
        .iter()
        .filter(|entity| entity.domain == "binary_sensor" && entity.device_class.as_deref() == Some("window"))
        .filter(|entity| state.is_none_or(|wanted| entity.state == wanted))
        .map(|entity| entity.name.as_str())
        .filter(|name| !name.is_empty())
        .collect();
    speak(&snap.language, state, &names)
}

fn speak(language: &str, state: Option<&str>, names: &[&str]) -> Option<String> {
    let base = language.split(['-', '_']).next().unwrap_or(language);
    let line = match base {
        "nl" => nl(state, names),
        "en" => en(state, names),
        "de" => de(state, names),
        _ => return None,
    };
    Some(line)
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
