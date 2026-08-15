use crate::lang::catalog;
use crate::lexicon::Action;
use crate::normalize::{article_one, is_time_unit};

/// Parse number words and digits from a token list using the active language packs.
pub fn extract_numbers(tokens: &[String]) -> Vec<i32> {
    let cat = catalog();
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        if article_one(t) && tokens.get(i + 1).is_some_and(|n| is_time_unit(n)) {
            out.push(1);
            i += 1;
            continue;
        }
        if let Ok(n) = t.parse::<i32>() {
            if is_room_index(tokens, i, n) {
                i += 1;
                continue;
            }
            out.push(n);
            i += 1;
            continue;
        }
        if cat.has_german_und() && t.contains("und") && t.len() > 5 {
            if let Some(n) = parse_german_compound(t) {
                out.push(n);
                i += 1;
                continue;
            }
        }
        if let Some(val) = cat.number(t) {
            if is_room_index(tokens, i, val) {
                i += 1;
                continue;
            }
            if cat.has_english_tens() && (20..100).contains(&val) && i + 1 < tokens.len() {
                if let Some(ones) = cat.number(&tokens[i + 1]) {
                    if ones > 0 && ones < 10 {
                        out.push(val + ones);
                        i += 2;
                        continue;
                    }
                }
            }
            if cat.has_german_und() && i + 2 < tokens.len() && tokens[i + 1] == "und" {
                if let Some(tens) = cat.number(&tokens[i + 2]) {
                    if tens >= 20 {
                        out.push(val + tens);
                        i += 3;
                        continue;
                    }
                }
            }
            out.push(val);
            i += 1;
            continue;
        }
        i += 1;
    }
    out
}

fn parse_german_compound(token: &str) -> Option<i32> {
    let cat = catalog();
    let parts: Vec<&str> = token.split("und").collect();
    if parts.len() != 2 {
        return None;
    }
    let ones = if parts[0] == "ein" { 1 } else { cat.number(parts[0])? };
    let tens = cat.number(parts[1])?;
    if tens < 20 {
        return None;
    }
    Some(ones + tens)
}

fn is_room_index(tokens: &[String], i: usize, n: i32) -> bool {
    if !(1..=8).contains(&n) {
        return false;
    }
    tokens.get(i.saturating_sub(1)).is_some_and(|prev| catalog().room_index_nouns.contains(prev.as_str()))
}

pub fn first_number(tokens: &[String]) -> Option<i32> {
    extract_numbers(tokens).into_iter().next()
}

pub fn guess_numbered_action(tokens: &[String], last_climate: bool, last_cover: bool, last_fan: bool) -> Action {
    let cat = catalog();
    if last_climate || cat.any(tokens, &cat.climate_nouns) {
        return Action::SetTemp;
    }
    if last_cover || cat.any(tokens, &cat.cover_nouns) || crate::lexicon::is_garage_cover(tokens) {
        return Action::CoverSet;
    }
    if last_fan || cat.any(tokens, &cat.fan_nouns) {
        return Action::FanSpeed;
    }
    Action::SetLight
}
