use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::home::roles::{is_music_assistant_player, is_music_player};
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::normalize::compact;
use crate::parse::resolve::Resolved;
use crate::parse::slots::ClauseOut;
use crate::session::Session;
use crate::types::{EntityRec, HomeGraph, Intent};

pub(crate) fn media_clause(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    action: Action,
    number: Option<i32>,
    resolved: &Resolved,
) -> Option<ClauseOut> {
    if let Some(intent) = status_intent(tokens, raw, home, session, resolved) {
        return Some(ClauseOut::Intents(vec![intent]));
    }
    if let Some(intent) = volume_intent(tokens, home, session, number, resolved) {
        return Some(ClauseOut::Intents(vec![intent]));
    }
    if let Some(intent) = transport_intent(tokens, home, session, action, resolved) {
        return Some(ClauseOut::Intents(vec![intent]));
    }
    if let Some(intent) = favorite_intent(tokens, home, session, resolved) {
        return Some(ClauseOut::Intents(vec![intent]));
    }
    if let Some(intent) = transfer_intent(tokens, home, session, resolved) {
        return Some(ClauseOut::Intents(vec![intent]));
    }
    play_intent(tokens, raw, home, session, action, resolved).map(|intent| ClauseOut::Intents(vec![intent]))
}

fn status_intent(tokens: &[String], raw: &[String], home: &HomeGraph, session: &Session, resolved: &Resolved) -> Option<Intent> {
    let status = if queue_status(tokens) || queue_status(raw) {
        "queue"
    } else if volume_status(tokens) || volume_status(raw) {
        "volume"
    } else if mute_status(tokens) || mute_status(raw) {
        "mute"
    } else if now_playing_status(tokens) || now_playing_status(raw) {
        "now_playing"
    } else if player_status(tokens) || player_status(raw) {
        "player"
    } else {
        return None;
    };
    let mut intent = if status == "queue" { Intent::new("MassGetQueue") } else { Intent::new("HassGetState") }.with("media_status", status);
    add_target_strict(&mut intent, target_player(home, session, resolved, true), resolved, session);
    Some(intent)
}

fn volume_intent(tokens: &[String], home: &HomeGraph, session: &Session, number: Option<i32>, resolved: &Resolved) -> Option<Intent> {
    let target = target_player(home, session, resolved, false);
    let session_media =
        session.last_domains().any(|d| d == "media_player") || session.last_entities().any(|id| id.starts_with("media_player."));
    let mut intent = if let Some(n) = number.filter(|_| has_volume_word(tokens) || session_media) {
        Intent::new("HassSetVolume").with("volume_level", n.clamp(0, 100).to_string())
    } else if any(tokens, &["lauter", "louder", "hoch"]) {
        Intent::new("HassSetVolumeRelative").with("volume_step", "up")
    } else if any(tokens, &["leiser", "quieter", "runter"]) {
        Intent::new("HassSetVolumeRelative").with("volume_step", "down")
    } else if any(tokens, &["stumm", "lautlos", "mute", "silence", "quiet"]) && media_context(tokens) {
        Intent::new("HassMediaPlayerMute")
    } else if any(tokens, &["ton", "unmute"]) && any(tokens, &["an", "on"]) {
        Intent::new("HassMediaPlayerUnmute")
    } else {
        return None;
    };
    add_target(&mut intent, target, resolved);
    Some(intent)
}

fn transport_intent(tokens: &[String], home: &HomeGraph, session: &Session, action: Action, resolved: &Resolved) -> Option<Intent> {
    let name = if matches!(action, Action::MediaPause) {
        "HassMediaPause"
    } else if any(tokens, &["vorheriges", "vorheriger", "previous", "zurueck"]) && media_context(tokens) {
        "HassMediaPrevious"
    } else if matches!(action, Action::MediaNext) {
        "HassMediaNext"
    } else if matches!(action, Action::MediaPlay) && !has_search_tail(tokens) {
        "HassMediaUnpause"
    } else if matches!(action, Action::On) && music_resume(tokens) {
        "HassMediaUnpause"
    } else {
        return None;
    };
    let mut intent = Intent::new(name);
    add_target(&mut intent, target_player(home, session, resolved, false), resolved);
    Some(intent)
}

fn play_intent(
    tokens: &[String],
    raw: &[String],
    home: &HomeGraph,
    session: &Session,
    action: Action,
    resolved: &Resolved,
) -> Option<Intent> {
    if !matches!(action, Action::MediaPlay) && !any(tokens, &["spiel", "spiele", "hoere", "hoer", "play", "listen", "queue"]) {
        return None;
    }
    let media = media_request(raw, home, resolved)?;
    if media.media_id.is_empty() {
        let mut intent = Intent::new("HassMediaUnpause");
        add_target_strict(&mut intent, target_player(home, session, resolved, true), resolved, session);
        return Some(intent);
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
    add_target_strict(&mut intent, target_player(home, session, resolved, true), resolved, session);
    Some(intent)
}

fn favorite_intent(tokens: &[String], home: &HomeGraph, session: &Session, resolved: &Resolved) -> Option<Intent> {
    if !any(tokens, &["favorisiere", "favorisieren", "favorite", "gefällt", "gefaellt"])
        && !(any(tokens, &["like"]) && media_context(tokens))
    {
        return None;
    }
    let mut intent = Intent::new("MassFavorite");
    add_target(&mut intent, target_player(home, session, resolved, false), resolved);
    Some(intent)
}

fn transfer_intent(tokens: &[String], home: &HomeGraph, session: &Session, resolved: &Resolved) -> Option<Intent> {
    if !any(tokens, &["verschiebe", "move", "transfer"]) || !catalog().any(tokens, &catalog().media_nouns) {
        return None;
    }
    let mut intent = Intent::new("MassTransferQueue");
    if let Some(src) = session.last_entities().find(|id| id.starts_with("media_player.")) {
        intent = intent.with("source_player", src);
    }
    add_target(&mut intent, target_player(home, session, resolved, false), resolved);
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
    let media_id = clean_media_words(&main, home, resolved).join(" ");
    if media_id.is_empty() && artist.is_empty() && !music_resume(raw) {
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

fn clean_media_words(words: &[String], home: &HomeGraph, resolved: &Resolved) -> Vec<String> {
    words.iter().filter(|word| !skip_media_word(word, home, resolved)).cloned().collect()
}

fn skip_media_word(word: &str, home: &HomeGraph, resolved: &Resolved) -> bool {
    is_play_word(word)
        || matches!(
            word,
            "das"
                | "die"
                | "der"
                | "den"
                | "dem"
                | "the"
                | "a"
                | "an"
                | "in"
                | "im"
                | "on"
                | "using"
                | "with"
                | "mit"
                | "auf"
                | "ueber"
                | "von"
                | "by"
                | "to"
                | "als"
                | "naechstes"
                | "next"
                | "queue"
                | "warteschlange"
                | "radiomodus"
                | "mode"
                | "modus"
                | "room"
                | "zimmer"
                | "tv"
                | "television"
                | "playback"
                | "media"
                | "music"
                | "musik"
        )
        || media_type_word(word).is_some()
        || resolved.areas.iter().any(|area| area_word(word, area, home))
}

fn media_type(words: &[String]) -> Option<&'static str> {
    words.iter().find_map(|word| media_type_word(word))
}

fn media_type_word(word: &str) -> Option<&'static str> {
    match word {
        "lied" | "titel" | "song" | "track" => Some("track"),
        "album" | "platte" | "record" | "ep" | "single" => Some("album"),
        "playlist" | "wiedergabeliste" => Some("playlist"),
        "kuenstler" | "artist" | "band" | "gruppe" | "group" => Some("artist"),
        "radio" | "sender" | "radiosender" | "station" => Some("radio"),
        "podcast" => Some("podcast"),
        "hoerbuch" | "audiobook" => Some("audiobook"),
        _ => None,
    }
}

fn target_player<'a>(home: &'a HomeGraph, session: &'a Session, resolved: &'a Resolved, strict_area: bool) -> Option<&'a EntityRec> {
    if let Some(entity) = resolved.entities.iter().find(|e| is_music_assistant_player(e)) {
        return Some(entity);
    }
    if let Some(entity) =
        resolved.areas.first().and_then(|area| music_players(home).into_iter().find(|e| e.area.as_deref() == Some(area.as_str())))
    {
        return Some(entity);
    }
    if let Some(entity) =
        session.preferred_area.as_deref().and_then(|area| music_players(home).into_iter().find(|e| e.area.as_deref() == Some(area)))
    {
        return Some(entity);
    }
    if strict_area && (session.preferred_area.is_some() || !resolved.areas.is_empty()) {
        return None;
    }
    session
        .last_entities()
        .find_map(|id| home.entities.iter().find(|e| e.entity_id == id && is_music_player(e)))
        .or_else(|| session.last_entities().find_map(|id| home.entities.iter().find(|e| e.entity_id == id && e.domain == "media_player")))
        .or_else(|| resolved.entities.iter().find(|e| e.domain == "media_player"))
        .or_else(|| home.entities.iter().find(|e| is_music_assistant_player(e) && e.tags.iter().any(|tag| compact(tag) == "preferred")))
        .or_else(|| single_music_player(home))
}

fn add_target(intent: &mut Intent, target: Option<&EntityRec>, resolved: &Resolved) {
    if let Some(entity) = target {
        *intent = intent.clone().with("entity_id", &entity.entity_id);
    } else if let Some(area) = resolved.areas.first() {
        *intent = intent.clone().with("area", area);
    }
}

fn add_target_strict(intent: &mut Intent, target: Option<&EntityRec>, resolved: &Resolved, session: &Session) {
    if let Some(entity) = target {
        *intent = intent.clone().with("entity_id", &entity.entity_id);
    } else if let Some(area) = resolved.areas.first().or(session.preferred_area.as_ref()) {
        *intent = intent.clone().with("area", area);
    }
}

fn music_players(home: &HomeGraph) -> Vec<&EntityRec> {
    let players: Vec<&EntityRec> =
        home.entities.iter().filter(|e| assist_visible(e, home) && !is_infra(e) && is_music_assistant_player(e)).collect();
    if players.is_empty() {
        home.entities.iter().filter(|e| assist_visible(e, home) && !is_infra(e) && is_music_player(e)).collect()
    } else {
        players
    }
}

fn single_music_player(home: &HomeGraph) -> Option<&EntityRec> {
    let players = music_players(home);
    (players.len() == 1).then(|| players[0])
}

fn has_volume_word(tokens: &[String]) -> bool {
    any(tokens, &["lautstaerke", "volume"]) || media_context(tokens)
}

fn queue_status(tokens: &[String]) -> bool {
    is_question(tokens) && (any(tokens, &["queue", "warteschlange"]) || (any(tokens, &["next", "naechstes"]) && media_context(tokens)))
        || has_phrase(tokens, &["kommt", "als", "naechstes"])
}

fn volume_status(tokens: &[String]) -> bool {
    is_question(tokens) && any(tokens, &["laut", "lautstaerke", "volume"])
}

fn mute_status(tokens: &[String]) -> bool {
    is_question(tokens) && any(tokens, &["stumm", "muted", "mute"]) && media_context(tokens)
}

fn now_playing_status(tokens: &[String]) -> bool {
    has_phrase(tokens, &["was", "laeuft"])
        || has_phrase(tokens, &["was", "spielt"])
        || has_phrase(tokens, &["whats", "playing"])
        || has_phrase(tokens, &["what", "playing"])
        || has_phrase(tokens, &["what", "s", "playing"])
        || has_phrase(tokens, &["what", "is", "playing"])
        || (is_question(tokens) && any(tokens, &["titel", "track"]) && any(tokens, &["aktuell", "current", "now"]))
}

fn player_status(tokens: &[String]) -> bool {
    is_question(tokens) && music_context(tokens) && any(tokens, &["status", "zustand", "an", "on", "playing", "laeuft"])
}

fn media_context(tokens: &[String]) -> bool {
    catalog().any(tokens, &catalog().media_nouns) || any(tokens, &["song", "track", "titel", "lied", "musik", "music"])
}

fn music_context(tokens: &[String]) -> bool {
    any(tokens, &["musik", "music", "radio", "song", "track", "titel", "lied", "playlist", "wiedergabeliste", "playback"])
}

fn music_resume(tokens: &[String]) -> bool {
    any(tokens, &["musik", "music", "radio", "playback"]) && any(tokens, &["an", "on", "weiter", "resume", "unpause"])
}

fn has_search_tail(tokens: &[String]) -> bool {
    !clean_media_words(tokens, &HomeGraph::default(), &Resolved { areas: Vec::new(), entities: Vec::new(), ambiguous: Vec::new() })
        .is_empty()
}

fn is_play_word(word: &str) -> bool {
    matches!(
        word,
        "spiel" | "spiele" | "hoere" | "hoer" | "abspielen" | "weiter" | "fortsetzen" | "play" | "listen" | "put" | "resume" | "unpause"
    )
}

fn any(tokens: &[String], words: &[&str]) -> bool {
    tokens.iter().any(|token| words.contains(&token.as_str()))
}

fn has_phrase(tokens: &[String], phrase: &[&str]) -> bool {
    tokens.windows(phrase.len()).any(|window| window.iter().map(String::as_str).eq(phrase.iter().copied()))
}

fn is_question(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "was" | "wie" | "ist" | "sind" | "what" | "whats" | "is" | "are" | "how"))
}

fn area_word(word: &str, area_id: &str, home: &HomeGraph) -> bool {
    home.areas.iter().find(|area| area.area_id == area_id).is_some_and(|area| {
        compact(&area.name) == compact(word) || area.area_id == word || area.aliases.iter().any(|alias| compact(alias) == compact(word))
    })
}

pub(crate) fn media_target_ids(home: &HomeGraph, area: &str) -> Vec<String> {
    music_players(home).into_iter().filter(|e| e.area.as_deref() == Some(area)).map(|e| e.entity_id.clone()).collect()
}
