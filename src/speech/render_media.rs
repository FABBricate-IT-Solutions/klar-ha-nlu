//! Media playback and volume lines for post-execute speech.

use crate::lang::Speech;
use crate::types::{SpeechEntity, SpeechSnapshot};

use super::render_place::slot;

pub(super) fn media_action(name: &str, where_: &str, snap: &SpeechSnapshot, speech: Speech) -> Option<String> {
    Some(match name {
        "HassMediaPause" => speech.media_pause.replace("{target}", where_),
        "HassMediaUnpause" => speech.media_play.replace("{target}", where_),
        "HassMediaNext" => speech.media_next.replace("{target}", where_),
        "HassMediaPrevious" => speech.media_previous.replace("{target}", where_),
        "HassMediaPlayerMute" => speech.media_mute.replace("{target}", where_),
        "HassMediaPlayerUnmute" => speech.media_unmute.replace("{target}", where_),
        "MassFavorite" => speech.media_favorite.to_string(),
        "HassMediaSearchAndPlay" | "MassPlayMedia" => media_started(snap, speech),
        // media_search doubles as the no-title "playback started" frame.
        "MassTransferQueue" => speech.media_transfer.to_string(),
        "HassSetVolume" => {
            let level = slot(snap, "volume_level").unwrap_or("?");
            speech.media_volume_set.replace("{target}", where_).replace("{n}", level)
        }
        "HassSetVolumeRelative" => {
            let down = slot(snap, "volume_step").is_some_and(|step| step == "down");
            if down {
                speech.media_volume_down.replace("{target}", where_)
            } else {
                speech.media_volume_up.replace("{target}", where_)
            }
        }
        "MassGetQueue" => queue_speech(snap, speech),
        _ => return None,
    })
}

fn media_started(snap: &SpeechSnapshot, speech: Speech) -> String {
    let player = snap.entities.iter().find(|entity| entity.domain == "media_player");
    let title = player.and_then(|entity| attr_str(entity, "media_title"));
    let artist = player.and_then(|entity| attr_str(entity, "media_artist"));
    match (title.as_deref(), artist.as_deref()) {
        (Some(title), Some(artist)) => speech.media_playing_by.replace("{title}", title).replace("{artist}", artist),
        (Some(title), None) => speech.media_playing.replace("{title}", title),
        _ => speech.media_search.to_string(),
    }
}

pub(super) fn media_status(snap: &SpeechSnapshot, status: &str, speech: Speech) -> String {
    let Some(player) = snap.entities.iter().find(|entity| entity.domain == "media_player") else {
        return String::new();
    };
    match status {
        "volume" => {
            let pct = volume_percent(attr_num(player, "volume_level"));
            let muted = attr_bool(player, "is_volume_muted");
            let body = if pct.is_empty() { speech.media_volume.to_string() } else { speech.media_volume_query.replace("{n}", &pct) };
            if muted {
                format!("{body} {}", speech.media_mute.to_ascii_lowercase().trim_end_matches('.'))
            } else {
                body
            }
        }
        "mute" => {
            if attr_bool(player, "is_volume_muted") {
                speech.media_mute.to_string()
            } else {
                speech.media_unmute.to_string()
            }
        }
        "now_playing" | "player" => {
            let title = media_title(player);
            if title.is_empty() {
                speech.entity_line(&player.name, &player.state)
            } else {
                speech.media_playing.replace("{title}", &title)
            }
        }
        _ => String::new(),
    }
}

fn queue_speech(snap: &SpeechSnapshot, speech: Speech) -> String {
    let current = snap.entities.iter().find(|entity| entity.domain == "media_player").map(media_title).unwrap_or_default();
    let upcoming: Vec<&str> =
        snap.media_queue.iter().map(|item| item.title.as_str()).filter(|title| !title.is_empty() && *title != current).take(3).collect();
    let mut bits = Vec::new();
    if !current.is_empty() {
        bits.push(speech.media_playing.replace("{title}", &current));
    }
    if upcoming.is_empty() {
        bits.push(speech.queue_empty.to_string());
        return bits.join(" ");
    }
    bits.push(speech.media_playing.replace("{title}", upcoming[0]));
    if upcoming.len() > 1 {
        bits.push(upcoming[1..].join(", "));
    }
    bits.join(" ")
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
        format!("{title} · {artist}")
    } else {
        title
    }
}
