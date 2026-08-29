use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::home::roles::{is_music_assistant_player, is_music_player, looks_like_tv, tv_asked};
use crate::lang::catalog;
use crate::lang::VerbKind;
use crate::parse::action::Action;
use crate::parse::normalize::{compact, umlaut_eq};
use crate::parse::resolve::Resolved;
use crate::parse::slots::ClauseOut;
use crate::session::Session;
use crate::types::{EntityRec, HomeGraph, Intent};

#[path = "media_words.rs"]
mod media_words;
use media_words::*;
pub(crate) use media_words::{is_media_move_or_play, now_playing_status};

pub(crate) fn media_clause(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    action: Action,
    number: Option<i32>,
    resolved: &Resolved,
) -> Option<ClauseOut> {
    let intent = status_intent(tokens)
        .or_else(|| volume_intent(tokens, session, number))
        .or_else(|| transport_intent(tokens, action, home, resolved))
        .or_else(|| favorite_intent(tokens))
        .or_else(|| transfer_intent(tokens, home, session))
        .or_else(|| play_intent(tokens, raw, home, action, resolved))?;
    let mass_only = matches!(intent.name.as_str(), "MassPlayMedia" | "MassTransferQueue" | "MassFavorite" | "MassGetQueue");
    let transfer = intent.name == "MassTransferQueue";
    if transfer && (intent.slot("source_player").is_none() || !explicit_destination(raw, home, resolved)) {
        return Some(ClauseOut::Intents(Vec::new()));
    }
    let allow_session_media = !matches!(intent.name.as_str(), "HassMediaSearchAndPlay" | "MassPlayMedia" | "MassTransferQueue");
    let volume_default =
        matches!(intent.name.as_str(), "HassSetVolume" | "HassSetVolumeRelative" | "HassMediaPlayerMute" | "HassMediaPlayerUnmute");
    let Some(target) = target_player(tokens, home, session, resolved, allow_session_media, mass_only, volume_default) else {
        let any_player =
            home.entities.iter().any(|entity| entity.domain == "media_player" && assist_visible(entity, home) && !is_infra(entity));
        if any_player {
            return Some(ClauseOut::Intents(Vec::new()));
        }
        return Some(ClauseOut::Intents(vec![Intent::new("KlarNoMusicPlayer")]));
    };
    if transfer && intent.slot("source_player") == Some(target.entity_id.as_str()) {
        return Some(ClauseOut::Intents(Vec::new()));
    }
    Some(ClauseOut::Intents(vec![bind_play_backend(intent, target).with("entity_id", &target.entity_id)]))
}

fn bind_play_backend(intent: Intent, target: &EntityRec) -> Intent {
    if intent.name != "HassMediaSearchAndPlay" || !is_music_assistant_player(target) {
        return intent;
    }
    let query = intent.slot("media_id").or_else(|| intent.slot("search_query")).unwrap_or_default();
    if query.is_empty() {
        return intent;
    }
    let mut rewritten = Intent::new("MassPlayMedia").with("media_id", query).with("search_query", query);
    for slot in ["media_class", "media_type", "artist", "enqueue", "radio_mode"] {
        if let Some(value) = intent.slot(slot) {
            rewritten = rewritten.with(slot, value);
        }
    }
    rewritten
}

fn status_intent(tokens: &[String]) -> Option<Intent> {
    let status = if queue_status(tokens) {
        "queue"
    } else if volume_status(tokens) {
        "volume"
    } else if mute_status(tokens) {
        "mute"
    } else if now_playing_status(tokens) {
        "now_playing"
    } else if player_status(tokens) {
        "player"
    } else {
        return None;
    };
    Some(if status == "queue" { Intent::new("MassGetQueue") } else { Intent::new("HassGetState") }.with("media_status", status))
}

fn volume_intent(tokens: &[String], session: &Session, number: Option<i32>) -> Option<Intent> {
    let session_media =
        session.last_domains().any(|d| d == "media_player") || session.last_entities().any(|id| id.starts_with("media_player."));
    let intent = if let Some(n) = number.filter(|_| has_volume_word(tokens) || session_media) {
        Intent::new("HassSetVolume").with("volume_level", n.clamp(0, 100).to_string())
    } else if any(tokens, &["lauter", "louder", "hoch"]) {
        Intent::new("HassSetVolumeRelative").with("volume_step", "up")
    } else if any(tokens, &["leiser", "quieter", "runter"]) || (any(tokens, &["down"]) && has_volume_word(tokens)) {
        Intent::new("HassSetVolumeRelative").with("volume_step", "down")
    } else if any(tokens, &["stumm", "lautlos", "mute", "silence", "quiet"]) && media_context(tokens) {
        Intent::new("HassMediaPlayerMute")
    } else if any(tokens, &["ton", "unmute"]) && any(tokens, &["an", "on"]) {
        Intent::new("HassMediaPlayerUnmute")
    } else {
        return None;
    };
    Some(intent)
}

fn transport_intent(tokens: &[String], action: Action, home: &HomeGraph, resolved: &Resolved) -> Option<Intent> {
    let name = if matches!(action, Action::MediaPause) {
        "HassMediaPause"
    } else if any(tokens, &["vorheriges", "vorheriger", "previous", "zurueck"]) && media_context(tokens) {
        "HassMediaPrevious"
    } else if matches!(action, Action::MediaNext) {
        "HassMediaNext"
    } else if (matches!(action, Action::MediaPlay) && !has_search_tail(tokens, home, resolved))
        || (matches!(action, Action::On) && music_resume(tokens))
        || (catalog().any(tokens, catalog().playback_resume()) && music_context(tokens) && !has_search_tail(tokens, home, resolved))
    {
        "HassMediaUnpause"
    } else {
        return None;
    };
    Some(Intent::new(name))
}

pub(crate) fn media_transport_form(tokens: &[String], action: Action) -> bool {
    matches!(action, Action::MediaPause | Action::MediaPlay | Action::MediaNext | Action::MediaMute)
        || (tokens.iter().any(|token| {
            matches!(
                token.as_str(),
                "pause"
                    | "pausiere"
                    | "pausieren"
                    | "naechster"
                    | "naechste"
                    | "naechstes"
                    | "vorheriger"
                    | "vorheriges"
                    | "resume"
                    | "unpause"
            )
        }) && catalog().any(tokens, catalog().media_nouns()))
}

fn play_intent(tokens: &[String], raw: &[String], home: &HomeGraph, action: Action, resolved: &Resolved) -> Option<Intent> {
    if now_playing_status(tokens)
        || (!matches!(action, Action::MediaPlay)
            && !any(tokens, &["spiel", "spiele", "hoere", "hoer", "play", "listen", "queue"])
            && !tokens.iter().any(|token| matches!(catalog().verb(token), Some(VerbKind::Play))))
    {
        return None;
    }
    let media = media_request(raw, home, resolved)?;
    if media.media_id.is_empty() {
        return Some(Intent::new("HassMediaUnpause"));
    }
    let mut intent = if media.needs_mass {
        Intent::new("MassPlayMedia").with("media_id", media.media_id)
    } else {
        Intent::new("HassMediaSearchAndPlay").with("search_query", media.media_id)
    };
    if let Some(media_type) = media.media_type {
        intent = intent.with("media_class", media_type).with("media_type", media_type);
    }
    if let Some(artist) = media.artist {
        intent = intent.with("artist", artist);
    }
    if let Some(enqueue) = media.enqueue {
        intent = intent.with("enqueue", enqueue);
    }
    if media.radio_mode {
        intent = intent.with("radio_mode", "true");
    }
    Some(intent)
}

fn favorite_intent(tokens: &[String]) -> Option<Intent> {
    if !(any(tokens, &["favorisiere", "favorisieren", "favorite", "gefällt", "gefaellt"])
        || any(tokens, &["like"]) && media_context(tokens))
    {
        return None;
    }
    Some(Intent::new("MassFavorite"))
}

fn transfer_intent(tokens: &[String], home: &HomeGraph, session: &Session) -> Option<Intent> {
    if !any(tokens, &["verschiebe", "move", "transfer"]) || !catalog().any(tokens, catalog().media_nouns()) {
        return None;
    }
    let mut intent = Intent::new("MassTransferQueue");
    if let Some(src) =
        session.last_entities().find(|id| home.entities.iter().any(|entity| entity.entity_id == **id && eligible_mass_player(entity, home)))
    {
        intent = intent.with("source_player", src);
    }
    Some(intent)
}

struct MediaRequest {
    media_id: String,
    media_type: Option<&'static str>,
    artist: Option<String>,
    enqueue: Option<&'static str>,
    radio_mode: bool,
    needs_mass: bool,
}

fn media_request(raw: &[String], home: &HomeGraph, resolved: &Resolved) -> Option<MediaRequest> {
    let radio_mode = has_phrase(raw, &["radio", "mode"]) || has_phrase(raw, &["radio", "modus"]) || any(raw, &["radiomodus"]);
    let enqueue = if any(raw, &["queue", "warteschlange"]) {
        Some("add")
    } else if has_phrase(raw, &["als", "naechstes"]) || any(raw, &["next"]) {
        Some("next")
    } else {
        None
    };
    let media_type = match media_type(raw) {
        Some("radio") if radio_mode => None,
        other => other,
    };
    let by_at = raw.iter().position(|t| matches!(t.as_str(), "von" | "by"));
    let (main, artist) = if let Some(at) = by_at {
        (raw[..at].to_vec(), clean_media_words(&raw[at + 1..], home, resolved).join(" "))
    } else {
        (raw.to_vec(), String::new())
    };
    let media_id = strip_area_query(clean_media_words(&main, home, resolved).join(" "), home);
    if media_id.is_empty() && artist.is_empty() && !music_resume(raw) && !music_context(raw) {
        return None;
    }
    Some(MediaRequest {
        media_id,
        media_type,
        artist: (!artist.is_empty()).then_some(artist),
        enqueue,
        radio_mode,
        needs_mass: radio_mode || enqueue.is_some() || by_at.is_some(),
    })
}

fn target_player<'a>(
    tokens: &[String],
    home: &'a HomeGraph,
    session: &Session,
    resolved: &Resolved,
    allow_session_media: bool,
    mass_only: bool,
    volume_default: bool,
) -> Option<&'a EntityRec> {
    let players = player_pool(home, tokens, mass_only);
    let resolved_ids: Vec<&str> = resolved
        .entities
        .iter()
        .chain(&resolved.ambiguous)
        .filter(|entity| eligible_media_player(entity, home) && explicitly_named(tokens, entity, home))
        .map(|entity| entity.entity_id.as_str())
        .collect();
    if !resolved_ids.is_empty() && !now_playing_status(tokens) {
        let candidates: Vec<&EntityRec> = home
            .entities
            .iter()
            .filter(|entity| resolved_ids.contains(&entity.entity_id.as_str()))
            .filter(|entity| !mass_only || eligible_mass_player(entity, home))
            .collect();
        return select_player(&candidates, session);
    }
    match resolved.areas.as_slice() {
        [area] => return music_in_named_area(home, &players, area, session),
        [] => {
            if let Some(area) = area_from_tokens(tokens, home) {
                return music_in_named_area(home, &players, &area, session);
            }
        }
        _ => return None,
    }
    if let Some(area) = session.preferred_area.as_deref() {
        return select_area_player(&players, area, session);
    }
    if allow_session_media {
        if let Some(entity) = session
            .last_entities()
            .find_map(|id| home.entities.iter().find(|entity| entity.entity_id == id && eligible_media_player(entity, home)))
        {
            return Some(entity);
        }
    }
    select_player(&players, session).or_else(|| {
        volume_default
            .then(|| {
                players.iter().copied().find(|player| player.area.as_deref() == Some("wohnzimmer")).or_else(|| players.first().copied())
            })
            .flatten()
    })
}

fn player_pool<'a>(home: &'a HomeGraph, tokens: &[String], mass_only: bool) -> Vec<&'a EntityRec> {
    if mass_only {
        return mass_players(home);
    }
    let music = music_players(home);
    if tv_asked(tokens) && !any(tokens, &["soundbar"]) {
        return home.entities.iter().filter(|entity| eligible_media_player(entity, home) && looks_like_tv(entity)).collect();
    }
    if any(tokens, &["soundbar"])
        || (tokens.iter().any(|token| token.contains("volume") || matches!(token.as_str(), "laut" | "lautstaerke"))
            && !music_context(tokens)
            && music.is_empty())
    {
        return home.entities.iter().filter(|entity| eligible_media_player(entity, home)).collect();
    }
    if let Some(area) = area_from_tokens(tokens, home) {
        let in_area: Vec<&EntityRec> = music.iter().copied().filter(|entity| entity.area.as_deref() == Some(area.as_str())).collect();
        return in_area;
    }
    let mass: Vec<&EntityRec> = music.iter().copied().filter(|entity| is_music_assistant_player(entity)).collect();
    if !mass.is_empty() {
        return mass;
    }
    music
}

fn music_in_named_area<'a>(home: &'a HomeGraph, players: &[&'a EntityRec], area: &str, session: &Session) -> Option<&'a EntityRec> {
    select_area_player(players, area, session).or_else(|| select_area_player(&music_players(home), area, session))
}

fn music_players(home: &HomeGraph) -> Vec<&EntityRec> {
    home.entities.iter().filter(|entity| eligible_music_player(entity, home)).collect()
}

fn mass_players(home: &HomeGraph) -> Vec<&EntityRec> {
    home.entities.iter().filter(|entity| eligible_mass_player(entity, home)).collect()
}

fn eligible_music_player(entity: &EntityRec, home: &HomeGraph) -> bool {
    assist_visible(entity, home) && !is_infra(entity) && (is_music_assistant_player(entity) || is_music_player(entity))
}

fn eligible_mass_player(entity: &EntityRec, home: &HomeGraph) -> bool {
    assist_visible(entity, home) && !is_infra(entity) && is_music_assistant_player(entity)
}

fn eligible_media_player(entity: &EntityRec, home: &HomeGraph) -> bool {
    entity.domain == "media_player" && assist_visible(entity, home) && !is_infra(entity)
}

fn explicitly_named(tokens: &[String], entity: &EntityRec, home: &HomeGraph) -> bool {
    let media_player_phrase = has_phrase(tokens, &["media", "player"]);
    media_player_phrase
        || tokens.iter().any(|token| {
            entity_word(token, entity)
                && !matches!(
                    token.as_str(),
                    "music"
                        | "musik"
                        | "media"
                        | "playback"
                        | "song"
                        | "track"
                        | "titel"
                        | "lied"
                        | "radio"
                        | "playlist"
                        | "wiedergabeliste"
                )
                && !home.areas.iter().any(|area| area_word(token, &area.area_id, home))
        })
}

fn explicit_destination(tokens: &[String], home: &HomeGraph, resolved: &Resolved) -> bool {
    tokens.iter().enumerate().filter(|(_, token)| matches!(token.as_str(), "to" | "nach" | "in" | "ins" | "zum" | "zur")).any(
        |(index, _)| {
            let tail = &tokens[index + 1..];
            let segment = tail.split(|token| catalog().is_conj(token)).next().unwrap_or(tail);
            let area = resolved
                .areas
                .iter()
                .filter(|_| resolved.areas.len() == 1)
                .any(|area| segment.iter().any(|word| area_word(word, area, home)));
            let entity = resolved
                .entities
                .iter()
                .chain(&resolved.ambiguous)
                .filter(|candidate| eligible_mass_player(candidate, home))
                .any(|candidate| segment.iter().any(|word| entity_word(word, candidate)));
            area || entity
        },
    )
}

fn strip_area_query(query: String, home: &HomeGraph) -> String {
    let folded = compact(&query);
    if folded.is_empty() {
        return String::new();
    }
    let area_only = home.areas.iter().any(|area| {
        compact(&area.area_id) == folded
            || compact(&area.name) == folded
            || umlaut_eq(&folded, &compact(&area.area_id))
            || umlaut_eq(&folded, &compact(&area.name))
            || area.aliases.iter().any(|alias| compact(alias) == folded || umlaut_eq(&folded, &compact(alias)))
    });
    if area_only {
        String::new()
    } else {
        query
    }
}

fn area_from_tokens(tokens: &[String], home: &HomeGraph) -> Option<String> {
    home.areas.iter().find_map(|area| {
        tokens
            .iter()
            .any(|token| {
                let folded = compact(token);
                folded == compact(&area.area_id)
                    || folded == compact(&area.name)
                    || umlaut_eq(&folded, &compact(&area.area_id))
                    || umlaut_eq(&folded, &compact(&area.name))
                    || area.aliases.iter().any(|alias| folded == compact(alias) || umlaut_eq(&folded, &compact(alias)))
            })
            .then(|| area.area_id.clone())
    })
}

fn select_area_player<'a>(players: &[&'a EntityRec], area: &str, session: &Session) -> Option<&'a EntityRec> {
    let candidates: Vec<&EntityRec> = players.iter().copied().filter(|player| player.area.as_deref() == Some(area)).collect();
    select_player(&candidates, session)
}

fn select_player<'a>(players: &[&'a EntityRec], session: &Session) -> Option<&'a EntityRec> {
    if players.len() == 1 {
        return players.first().copied();
    }
    let preferred: Vec<&EntityRec> =
        players.iter().copied().filter(|player| player.tags.iter().any(|tag| compact(tag) == "preferred")).collect();
    if preferred.len() == 1 {
        return preferred.first().copied();
    }
    if let Some(entity) = session.last_entities().find_map(|id| players.iter().copied().find(|player| player.entity_id == id)) {
        return Some(entity);
    }
    let mass: Vec<&EntityRec> = players.iter().copied().filter(|player| is_music_assistant_player(player)).collect();
    (mass.len() == 1).then(|| mass[0])
}
