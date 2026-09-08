//! Media playback and queue lines for post-execute speech.

use crate::types::{SpeechEntity, SpeechSnapshot};

use super::render_place::{slot, speak_state};

pub(super) fn media_action(name: &str, where_: &str, snap: &SpeechSnapshot, de: bool) -> Option<String> {
    Some(match name {
        "HassMediaPause" => {
            if de {
                format!("{where_} ist pausiert.")
            } else {
                format!("{where_} is paused.")
            }
        }
        "HassMediaUnpause" => {
            if de {
                format!("{where_} spielt weiter.")
            } else {
                format!("{where_} resumed playback.")
            }
        }
        "HassMediaNext" => {
            if de {
                format!("Auf {where_} läuft der nächste Titel.")
            } else {
                format!("The next track is playing on {where_}.")
            }
        }
        "HassMediaPrevious" => {
            if de {
                format!("Auf {where_} läuft der vorherige Titel.")
            } else {
                format!("The previous track is playing on {where_}.")
            }
        }
        "HassMediaPlayerMute" => {
            if de {
                format!("{where_} ist stumm.")
            } else {
                format!("{where_} is muted.")
            }
        }
        "HassMediaPlayerUnmute" => {
            if de {
                format!("Der Ton von {where_} ist an.")
            } else {
                format!("{where_} is unmuted.")
            }
        }
        "MassFavorite" => {
            if de {
                "Als Favorit markiert.".into()
            } else {
                "Marked as a favorite.".into()
            }
        }
        "HassMediaSearchAndPlay" | "MassPlayMedia" => {
            if de {
                "Die Wiedergabe wurde gestartet.".into()
            } else {
                "Playback started.".into()
            }
        }
        "MassTransferQueue" => {
            if de {
                "Die Warteschlange wurde übertragen.".into()
            } else {
                "The queue was transferred.".into()
            }
        }
        "HassSetVolume" => {
            let level = slot(snap, "volume_level").unwrap_or("?");
            if de {
                format!("Die Lautstärke von {where_} ist auf {level} Prozent.")
            } else {
                format!("{where_} volume is set to {level} percent.")
            }
        }
        "HassSetVolumeRelative" => {
            let down = slot(snap, "volume_step").is_some_and(|step| step == "down");
            if de {
                format!("Die Lautstärke von {where_} wurde {}.", if down { "verringert" } else { "erhöht" })
            } else {
                format!("{where_} volume was {}.", if down { "lowered" } else { "raised" })
            }
        }
        "MassGetQueue" => queue_speech(snap, de),
        _ => return None,
    })
}

pub(super) fn media_status(snap: &SpeechSnapshot, status: &str, de: bool) -> String {
    let Some(player) = snap.entities.iter().find(|entity| entity.domain == "media_player") else {
        return String::new();
    };
    match status {
        "volume" => {
            let pct = volume_percent(attr_num(player, "volume_level"));
            let muted = attr_bool(player, "is_volume_muted");
            if de {
                let body =
                    if pct.is_empty() { "Ich kann die Lautstärke nicht lesen.".into() } else { format!("Lautstärke ist {pct} Prozent.") };
                if muted {
                    format!("{body} Der Ton ist stumm.")
                } else {
                    body
                }
            } else {
                let body = if pct.is_empty() { "I cannot read the volume.".into() } else { format!("Volume is {pct} percent.") };
                if muted {
                    format!("{body} It is muted.")
                } else {
                    body
                }
            }
        }
        "mute" => {
            let muted = attr_bool(player, "is_volume_muted");
            if de {
                if muted {
                    "Der Ton ist stumm.".into()
                } else {
                    "Der Ton ist an.".into()
                }
            } else if muted {
                "It is muted.".into()
            } else {
                "It is not muted.".into()
            }
        }
        "now_playing" | "player" => {
            let title = media_title(player);
            if title.is_empty() {
                let spoken = speak_state(&player.state, if de { "de" } else { "en" });
                if de {
                    format!("Der Player ist {spoken}.")
                } else {
                    format!("The player is {spoken}.")
                }
            } else if de {
                let prefix = if player.state == "playing" { "Gerade läuft" } else { "Ausgewählt ist" };
                format!("{prefix} {title}.")
            } else {
                let prefix = if player.state == "playing" { "Now playing" } else { "Selected" };
                format!("{prefix} {title}.")
            }
        }
        _ => String::new(),
    }
}

fn queue_speech(snap: &SpeechSnapshot, de: bool) -> String {
    let current = snap.entities.iter().find(|entity| entity.domain == "media_player").map(media_title).unwrap_or_default();
    let upcoming: Vec<&str> =
        snap.media_queue.iter().map(|item| item.title.as_str()).filter(|title| !title.is_empty() && *title != current).take(3).collect();
    if de {
        let mut bits = Vec::new();
        if !current.is_empty() {
            bits.push(format!("Gerade läuft {current}."));
        }
        if upcoming.is_empty() {
            bits.push(if current.is_empty() { "Die Warteschlange ist leer.".into() } else { "Danach ist die Warteschlange leer.".into() });
            return bits.join(" ");
        }
        bits.push(format!("Als Nächstes kommt {}.", upcoming[0]));
        if upcoming.len() > 1 {
            bits.push(format!("Danach {}.", upcoming[1..].join(", ")));
        }
        bits.join(" ")
    } else {
        let mut bits = Vec::new();
        if !current.is_empty() {
            bits.push(format!("Now playing {current}."));
        }
        if upcoming.is_empty() {
            bits.push(if current.is_empty() { "The queue is empty.".into() } else { "There is nothing else in the queue.".into() });
            return bits.join(" ");
        }
        bits.push(format!("Next is {}.", upcoming[0]));
        if upcoming.len() > 1 {
            bits.push(format!("Then {}.", upcoming[1..].join(", ")));
        }
        bits.join(" ")
    }
}

fn attr_str(entity: &SpeechEntity, key: &str) -> Option<String> {
    match entity.attributes.get(key)? {
        serde_json::Value::String(text) if !text.is_empty() => Some(text.clone()),
        serde_json::Value::Number(num) => Some(num.to_string()),
        _ => None,
    }
}

fn attr_num(entity: &SpeechEntity, key: &str) -> Option<f64> {
    match entity.attributes.get(key)? {
        serde_json::Value::Number(num) => num.as_f64(),
        serde_json::Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn attr_bool(entity: &SpeechEntity, key: &str) -> bool {
    match entity.attributes.get(key) {
        Some(serde_json::Value::Bool(flag)) => *flag,
        Some(serde_json::Value::String(text)) => text == "true" || text == "on",
        _ => false,
    }
}

fn volume_percent(raw: Option<f64>) -> String {
    let Some(mut value) = raw else {
        return String::new();
    };
    if value <= 1.0 {
        value *= 100.0;
    }
    format!("{}", value.round() as i64)
}

fn media_title(entity: &SpeechEntity) -> String {
    let title = attr_str(entity, "media_title").unwrap_or_default();
    let artist = attr_str(entity, "media_artist").unwrap_or_default();
    if !title.is_empty() && !artist.is_empty() {
        format!("{title} by {artist}")
    } else {
        title
    }
}
