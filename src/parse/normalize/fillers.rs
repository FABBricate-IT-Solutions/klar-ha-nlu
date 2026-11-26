//! Filler stripping and area-name collapse after tokenization.

use crate::lang::catalog;
use crate::parse::normalize::token::tokenize;
use crate::types::AreaRec;

/// Like [`strip_fillers`], but a multi-word area name that contains a filler ("living room",
/// "tv room") is first collapsed to one compact `area_id` token ("livingroom"), the form area
/// matching already compares against. Otherwise "room" is stripped and the name is lost, and
/// its other words ("tv", "play", "front") read as commands or devices.
pub fn strip_fillers_for_areas(tokens: &[String], areas: &[AreaRec]) -> Vec<String> {
    strip_fillers(&collapse_area_names(tokens, areas))
}

fn collapse_area_names(tokens: &[String], areas: &[AreaRec]) -> Vec<String> {
    let cat = &catalog();
    let mut names: Vec<(Vec<String>, String)> = areas
        .iter()
        .flat_map(|area| {
            let labels = std::iter::once(area.name.as_str()).chain(area.aliases.iter().map(String::as_str));
            let token = compact(&area.area_id);
            labels.map(move |label| (tokenize(label), token.clone()))
        })
        .filter(|(words, _)| words.len() > 1 && words.iter().any(|w| cat.is_filler(w)) && !words.iter().all(|w| cat.is_filler(w)))
        .collect();
    // Longest names first, so "guest bed room" wins over "bed room".
    names.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        match names.iter().find(|(words, _)| tokens[i..].starts_with(words)) {
            Some((words, token)) => {
                out.push(token.clone());
                i += words.len();
            }
            None => {
                out.push(tokens[i].clone());
                i += 1;
            }
        }
    }
    out
}

pub fn strip_fillers(tokens: &[String]) -> Vec<String> {
    let cat = &catalog();
    tokens
        .iter()
        .enumerate()
        .filter(|(i, t)| {
            if cat.strip_pairs.iter().any(|(w, nxt)| t.as_str() == *w && tokens.get(i + 1).is_some_and(|n| n == nxt)) {
                return false;
            }
            if article_one(t) && tokens.get(i + 1).is_some_and(|n| is_time_unit(n)) {
                return true;
            }
            if cat.is_action_keep(t) {
                return true;
            }
            if cat
                .keep_after
                .iter()
                .any(|(prev, keep)| t.as_str() == *keep && tokens.get(i.saturating_sub(1)).is_some_and(|p| prev.contains(&p.as_str())))
            {
                return true;
            }
            !cat.is_filler(t)
        })
        .map(|(_, t)| t.clone())
        .collect()
}

pub fn join_tokens(tokens: &[String]) -> String {
    tokens.join(" ")
}

pub fn compact(s: &str) -> String {
    use crate::parse::normalize::fold::fold_latin;
    use crate::parse::normalize::token::is_word_char;
    fold_latin(s).chars().filter(|c| is_word_char(*c) && *c != '\'').collect()
}

/// "Schlafzimmern" / "Wohnzimmers" / "bedrooms" match the room stem.
pub fn inflected_eq(token: &str, label: &str) -> bool {
    label.len() >= 6
        && catalog().morphology.effective_room_suffixes().iter().any(|suffix| token.strip_suffix(suffix).is_some_and(|stem| stem == label))
}

pub(crate) fn is_time_unit(token: &str) -> bool {
    let cat = &catalog();
    cat.hours().contains(token)
        || cat.minutes().contains(token)
        || cat.seconds().contains(token)
        || matches!(token, "hour" | "hours" | "hrs" | "minute" | "minutes" | "min" | "mins" | "second" | "seconds" | "sec" | "secs")
}

pub(crate) fn article_one(token: &str) -> bool {
    catalog().article_one().contains(token)
}

#[cfg(test)]
mod tests {
    use super::compact;

    #[test]
    fn compact_keeps_letters_outside_ascii() {
        assert_eq!(compact("Wohnzimmer!"), "wohnzimmer");
        assert_eq!(compact("филм"), "филм");
        assert_ne!(compact("филм"), compact("антре"));
        assert_eq!(compact("فيلم"), "فيلم");
        assert_eq!(compact("סרט"), "סרט");
    }

    #[test]
    fn indic_compact_keeps_virama() {
        assert_eq!(compact("জ্বালাও"), "জ্বালাও");
        assert_eq!(compact("बत्ती"), "बत्ती");
    }
}
