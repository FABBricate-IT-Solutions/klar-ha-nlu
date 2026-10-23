use super::render_place::slot;
use crate::types::SpeechSnapshot;

pub(super) fn window_list(snap: &SpeechSnapshot) -> Option<String> {
    if snap.language != "nl" || slot(snap, "device_class") != Some("window") || slot(snap, "response") != Some("list") {
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
    Some(if names.is_empty() {
        match state {
            Some("on") => "Er staan geen ramen open.",
            Some("off") => "Er zijn geen gesloten ramen.",
            _ => "Er zijn geen raamsensoren gevonden.",
        }
        .into()
    } else {
        match state {
            Some("on") => format!("Deze ramen staan open: {}.", names.join(", ")),
            Some("off") => format!("Deze ramen zijn gesloten: {}.", names.join(", ")),
            _ => format!("Raamsensoren: {}.", names.join(", ")),
        }
    })
}
