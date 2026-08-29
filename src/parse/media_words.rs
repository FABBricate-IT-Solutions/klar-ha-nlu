use crate::lang::catalog;
use crate::lang::VerbKind;
use crate::parse::normalize::fold_umlaut;
use crate::parse::resolve::Resolved;
use crate::types::{EntityRec, HomeGraph};

const SKIP_MEDIA: &[&str] = &[
    "on",
    "using",
    "with",
    "mit",
    "auf",
    "ueber",
    "von",
    "by",
    "to",
    "als",
    "naechstes",
    "next",
    "queue",
    "warteschlange",
    "radiomodus",
    "room",
    "zimmer",
    "tv",
    "television",
    "playback",
    "wiedergabe",
    "media",
    "music",
    "musik",
];

pub(super) fn clean_media_words(words: &[String], home: &HomeGraph, resolved: &Resolved) -> Vec<String> {
    words.iter().filter(|word| !skip_media_word(word, words, home, resolved)).cloned().collect()
}

fn skip_media_word(word: &str, words: &[String], home: &HomeGraph, resolved: &Resolved) -> bool {
    if matches!(word, "mode" | "modus") && words.iter().any(|item| item == "radio" || item == "radiomodus") {
        return true;
    }
    is_play_word(word)
        || catalog().is_filler(word)
        || catalog().is_conj(word)
        || catalog().command_hedges().contains(word)
        || SKIP_MEDIA.contains(&word)
        || media_type_word(word).is_some()
        || resolved.areas.iter().any(|area| area_word(word, area, home))
        || resolved.entities.iter().any(|entity| entity_word(word, entity))
        || resolved.ambiguous.iter().any(|entity| entity_word(word, entity))
}

pub(super) fn media_type(words: &[String]) -> Option<&'static str> {
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

pub(super) fn has_volume_word(tokens: &[String]) -> bool {
    tokens.iter().any(|token| token.contains("volume") || matches!(token.as_str(), "laut" | "lautstaerke" | "leiser" | "quieter"))
}

pub(super) fn queue_status(tokens: &[String]) -> bool {
    is_question(tokens) && (any(tokens, &["queue", "warteschlange"]) || (any(tokens, &["next", "naechstes"]) && media_context(tokens)))
        || has_phrase(tokens, &["kommt", "als", "naechstes"])
}

pub(super) fn volume_status(tokens: &[String]) -> bool {
    is_question(tokens) && tokens.iter().any(|token| token.contains("volume") || matches!(token.as_str(), "laut" | "lautstaerke"))
}

pub(super) fn mute_status(tokens: &[String]) -> bool {
    is_question(tokens) && any(tokens, &["stumm", "muted", "mute"]) && media_context(tokens)
}

pub(crate) fn now_playing_status(tokens: &[String]) -> bool {
    has_phrase(tokens, &["was", "laeuft"])
        || has_phrase(tokens, &["was", "spielt"])
        || has_phrase(tokens, &["whats", "playing"])
        || has_phrase(tokens, &["what", "playing"])
        || has_phrase(tokens, &["what", "s", "playing"])
        || has_phrase(tokens, &["what", "is", "playing"])
        || (is_question(tokens) && any(tokens, &["titel", "track"]) && any(tokens, &["aktuell", "current", "now"]))
        || (is_question(tokens) && !catalog().any(tokens, catalog().tv_words()) && music_context(tokens))
}

pub(super) fn player_status(tokens: &[String]) -> bool {
    is_question(tokens) && music_context(tokens) && any(tokens, &["status", "zustand", "an", "on", "playing", "laeuft"])
}

pub(super) fn media_context(tokens: &[String]) -> bool {
    catalog().any(tokens, catalog().media_nouns()) || any(tokens, &["song", "track", "titel", "lied", "musik", "music"])
}

pub(super) fn music_context(tokens: &[String]) -> bool {
    any(tokens, &["musik", "music", "radio", "song", "track", "titel", "lied", "playlist", "wiedergabeliste", "playback", "wiedergabe"])
}

pub(super) fn music_resume(tokens: &[String]) -> bool {
    any(tokens, &["musik", "music", "radio", "playback", "wiedergabe"])
        && (catalog().any(tokens, catalog().playback_resume()) || any(tokens, &["an", "on", "weiter", "resume", "unpause"]))
}

pub(super) fn has_search_tail(tokens: &[String], home: &HomeGraph, resolved: &Resolved) -> bool {
    !clean_media_words(tokens, home, resolved).is_empty()
}

pub(super) fn is_play_word(word: &str) -> bool {
    ["spiel", "spiele", "hoere", "hoer", "abspielen", "weiter", "fortsetzen", "play", "listen", "put", "resume", "unpause"].contains(&word)
        || matches!(catalog().verb(word), Some(VerbKind::Play))
        || catalog().playback_resume().contains(word)
}

pub(super) fn any(tokens: &[String], words: &[&str]) -> bool {
    tokens.iter().any(|token| words.contains(&token.as_str()))
}

pub(super) fn has_phrase(tokens: &[String], phrase: &[&str]) -> bool {
    tokens.windows(phrase.len()).any(|window| window.iter().map(String::as_str).eq(phrase.iter().copied()))
}

fn is_question(tokens: &[String]) -> bool {
    let cat = catalog();
    any(tokens, &["was", "wie", "ist", "sind", "what", "whats", "is", "are", "how"])
        || tokens.iter().any(|token| matches!(cat.verb(token), Some(VerbKind::Query)))
        || cat.any(tokens, cat.question_words())
        || cat.any(tokens, cat.query_hint())
}

pub(super) fn area_word(word: &str, area_id: &str, home: &HomeGraph) -> bool {
    home.areas.iter().find(|area| area.area_id == area_id).is_some_and(|area| {
        label_has_word(&area.name, word)
            || label_has_word(&area.area_id, word)
            || area.aliases.iter().any(|alias| label_has_word(alias, word))
    })
}

pub(super) fn entity_word(word: &str, entity: &EntityRec) -> bool {
    let suffix = entity.entity_id.rsplit('.').next().unwrap_or(&entity.entity_id);
    label_has_word(&entity.name, word) || label_has_word(suffix, word) || entity.aliases.iter().any(|alias| label_has_word(alias, word))
}

fn label_has_word(label: &str, word: &str) -> bool {
    let folded = fold_umlaut(label);
    folded.split(|c: char| !crate::parse::normalize::is_word_char(c)).any(|part| part == word)
}
