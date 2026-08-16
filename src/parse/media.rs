use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::home::roles::{is_music_assistant_player, is_music_player};
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::normalize::{compact, fold_umlaut};
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
    let intent = status_intent(tokens)
        .or_else(|| volume_intent(tokens, session, number))
        .or_else(|| transport_intent(tokens, action))
        .or_else(|| favorite_intent(tokens))
        .or_else(|| transfer_intent(tokens, home, session))
        .or_else(|| play_intent(tokens, raw, home, action, resolved))?;
    let mass_only = matches!(intent.name.as_str(), "MassPlayMedia" | "MassTransferQueue" | "MassFavorite" | "MassGetQueue");
    let transfer = intent.name == "MassTransferQueue";
    if transfer && (intent.slot("source_player").is_none() || !explicit_destination(raw, home, resolved)) {
        return Some(ClauseOut::Intents(Vec::new()));
    }
    let allow_session_media = !matches!(intent.name.as_str(), "HassMediaSearchAndPlay" | "MassPlayMedia" | "MassTransferQueue");
    let Some(target) = target_player(tokens, home, session, resolved, allow_session_media, mass_only) else {
        return Some(ClauseOut::Intents(Vec::new()));
    };
    if transfer && intent.slot("source_player") == Some(target.entity_id.as_str()) {
        return Some(ClauseOut::Intents(Vec::new()));
    }
    Some(ClauseOut::Intents(vec![intent.with("entity_id", &target.entity_id)]))
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
    } else if any(tokens, &["leiser", "quieter", "runter"]) {
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

fn transport_intent(tokens: &[String], action: Action) -> Option<Intent> {
    let name = if matches!(action, Action::MediaPause) {
        "HassMediaPause"
    } else if any(tokens, &["vorheriges", "vorheriger", "previous", "zurueck"]) && media_context(tokens) {
        "HassMediaPrevious"
    } else if matches!(action, Action::MediaNext) {
        "HassMediaNext"
    } else if (matches!(action, Action::MediaPlay) && !has_search_tail(tokens)) || (matches!(action, Action::On) && music_resume(tokens)) {
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
        }) && catalog().any(tokens, &catalog().media_nouns))
}

fn play_intent(tokens: &[String], raw: &[String], home: &HomeGraph, action: Action, resolved: &Resolved) -> Option<Intent> {
    if !matches!(action, Action::MediaPlay) && !any(tokens, &["spiel", "spiele", "hoere", "hoer", "play", "listen", "queue"]) {
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
    if !any(tokens, &["verschiebe", "move", "transfer"]) || !catalog().any(tokens, &catalog().media_nouns) {
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
        || resolved.entities.iter().any(|entity| entity_word(word, entity))
        || resolved.ambiguous.iter().any(|entity| entity_word(word, entity))
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

fn target_player<'a>(
    tokens: &[String],
    home: &'a HomeGraph,
    session: &Session,
    resolved: &Resolved,
    allow_session_media: bool,
    mass_only: bool,
) -> Option<&'a EntityRec> {
    let players = if mass_only { mass_players(home) } else { music_players(home) };
    let resolved_ids: Vec<&str> = resolved
        .entities
        .iter()
        .chain(&resolved.ambiguous)
        .filter(|entity| eligible_media_player(entity, home) && explicitly_named(tokens, entity, home))
        .map(|entity| entity.entity_id.as_str())
        .collect();
    if !resolved_ids.is_empty() {
        let candidates: Vec<&EntityRec> = home
            .entities
            .iter()
            .filter(|entity| resolved_ids.contains(&entity.entity_id.as_str()))
            .filter(|entity| !mass_only || eligible_mass_player(entity, home))
            .collect();
        return select_player(&candidates, session);
    }
    match resolved.areas.as_slice() {
        [area] => return select_area_player(&players, area, session),
        [] => {}
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
    select_player(&players, session)
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
    session.last_entities().find_map(|id| players.iter().copied().find(|player| player.entity_id == id))
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
        label_has_word(&area.name, word)
            || label_has_word(&area.area_id, word)
            || area.aliases.iter().any(|alias| label_has_word(alias, word))
    })
}

fn entity_word(word: &str, entity: &EntityRec) -> bool {
    let suffix = entity.entity_id.rsplit('.').next().unwrap_or(&entity.entity_id);
    label_has_word(&entity.name, word) || label_has_word(suffix, word) || entity.aliases.iter().any(|alias| label_has_word(alias, word))
}

fn label_has_word(label: &str, word: &str) -> bool {
    let folded = fold_umlaut(label);
    folded.split(|c: char| !c.is_alphanumeric()).any(|part| part == word)
}

pub(crate) fn media_target_ids(home: &HomeGraph, area: &str) -> Vec<String> {
    music_players(home).into_iter().filter(|e| e.area.as_deref() == Some(area)).map(|e| e.entity_id.clone()).collect()
}
