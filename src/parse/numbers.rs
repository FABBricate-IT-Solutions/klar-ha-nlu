use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::normalize::{article_one, is_time_unit};
use crate::types::IntentPlan;

/// Slots that name a target. Every other slot with a number in it is a value that was spoken.
const TARGET_SLOTS: &[&str] = &["entity_id", "area", "floor", "name", "domain", "item"];

/// One number as it was spoken. `readable` is false when several number words stand
/// together and the active language has no rule that adds them up ("dix sept" where only
/// "dixsept" is listed, "treinta y cinco"). `value` is then only the first word.
struct Spoken {
    value: i32,
    readable: bool,
    /// Where it stands in the tokens.
    start: usize,
    end: usize,
    /// Held together by a conjunction or an article ("ein und zwanzig").
    glued: bool,
}

enum Piece<'a> {
    Word(i32),
    /// A conjunction between two number words.
    And(&'a str),
}

/// Parse number words and digits from a token list using the active language packs.
pub fn extract_numbers(tokens: &[String]) -> Vec<i32> {
    scan(tokens).into_iter().map(|number| number.value).collect()
}

/// The words hold a number this language cannot add up. Acting on the part that was
/// understood would set 10 degrees for "dix-sept", so the caller must not act on it.
fn unreadable_number(tokens: &[String]) -> bool {
    scan(tokens).iter().any(|number| !number.readable)
}

/// The plan would set a value, and the words hold a number that was only partly read.
pub fn sets_misread_number(plan: &IntentPlan, tokens: &[String]) -> bool {
    let sets_value = plan
        .steps
        .iter()
        .flat_map(|step| &step.intent.slots)
        .any(|slot| !TARGET_SLOTS.contains(&slot.name.as_str()) && slot.value.parse::<f64>().is_ok());
    sets_value && unreadable_number(tokens)
}

/// Writes a number that is held together by "and" as one digit token: "ein und zwanzig"
/// becomes "21". Later stages drop articles and cut clauses at "and", and would otherwise
/// take such a number apart.
pub fn fold_spoken_numbers(tokens: &[String]) -> Vec<String> {
    let glued: Vec<Spoken> = scan(tokens).into_iter().filter(|number| number.readable && number.glued).collect();
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if let Some(number) = glued.iter().find(|number| number.start == i) {
            out.push(number.value.to_string());
            i = number.end;
        } else {
            out.push(tokens[i].clone());
            i += 1;
        }
    }
    out
}

fn scan(tokens: &[String]) -> Vec<Spoken> {
    let cat = catalog();
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        let single = |value: i32| Spoken { value, readable: true, start: i, end: i + 1, glued: false };
        if article_one(t) && tokens.get(i + 1).is_some_and(|n| is_time_unit(n)) {
            out.push(single(1));
            i += 1;
            continue;
        }
        if let Ok(n) = t.parse::<i32>() {
            if !is_room_index(tokens, i, n) {
                out.push(single(n));
            }
            i += 1;
            continue;
        }
        // "ein und zwanzig": the article stands for one.
        if cat.has_german_und() && article_one(t) && tokens.get(i + 1).is_some_and(|next| next == "und") {
            if let Some(tens) = tokens.get(i + 2).and_then(|word| cat.number(word)).filter(|tens| is_tens(*tens)) {
                out.push(Spoken { value: 1 + tens, readable: true, start: i, end: i + 3, glued: true });
                i += 3;
                continue;
            }
        }
        // "een tien minuten timer": a word that is both article and "one" is the article
        // when a number follows it.
        if article_one(t) && word_at(tokens, i + 1).is_some() {
            i += 1;
            continue;
        }
        let Some((first, width)) = word_at(tokens, i) else {
            i += 1;
            continue;
        };
        if is_room_index(tokens, i, first) {
            i += 1;
            continue;
        }
        let mut run = vec![Piece::Word(first)];
        let mut end = i + width;
        loop {
            if let Some((value, width)) = word_at(tokens, end) {
                run.push(Piece::Word(value));
                end += width;
            } else if let Some((value, width)) = tokens.get(end).filter(|word| cat.is_conj(word)).and_then(|_| word_at(tokens, end + 1)) {
                run.push(Piece::And(&tokens[end]));
                run.push(Piece::Word(value));
                end += 1 + width;
            } else {
                break;
            }
        }
        let value = compose(&run);
        let glued = run.iter().any(|piece| matches!(piece, Piece::And(_)));
        out.push(Spoken { value: value.unwrap_or(first), readable: value.is_some(), start: i, end, glued });
        i = end;
    }
    out
}

/// Longest listed number that can be written in several words.
const MAX_WORDS_PER_NUMBER: usize = 4;

/// The number word that starts at `at` and how many tokens it takes. "dix sept" is listed
/// as "dixsept" and "quatre vingts" as "quatrevingts": the hyphen is gone by now, so the
/// longest run of tokens that spells a listed number is that number.
fn word_at(tokens: &[String], at: usize) -> Option<(i32, usize)> {
    let cat = catalog();
    let rest = tokens.get(at..)?;
    let spelled = (2..=MAX_WORDS_PER_NUMBER.min(rest.len())).rev().find_map(|width| {
        let words = &rest[..width];
        let no_digits = words.iter().all(|word| word.parse::<i32>().is_err());
        no_digits.then(|| cat.number(&words.concat())).flatten().map(|value| (value, width))
    });
    spelled.or_else(|| word_value(rest.first()?).map(|value| (value, 1)))
}

/// A number word of the active packs, or a German compound written as one word.
fn word_value(token: &str) -> Option<i32> {
    let cat = catalog();
    if token.parse::<i32>().is_ok() {
        return None;
    }
    cat.number(token).or_else(|| cat.has_german_und().then(|| german_compound(token)).flatten())
}

/// Adds up number words that stand together, by the rules the active packs bring.
fn compose(run: &[Piece<'_>]) -> Option<i32> {
    let cat = catalog();
    if let [Piece::Word(value)] = run {
        return Some(*value);
    }
    let (hundreds, rest) = match run {
        [Piece::Word(count), Piece::Word(100), rest @ ..] if (1..10).contains(count) => (count * 100, rest),
        [Piece::Word(round), rest @ ..] if *round >= 100 && round % 100 == 0 => (*round, rest),
        _ => (0, run),
    };
    let below = match rest {
        [] if hundreds > 0 => Some(0),
        [Piece::Word(value)] if *value < 100 && hundreds > 0 => Some(*value),
        [Piece::Word(tens), Piece::Word(ones)] if cat.has_english_tens() && is_tens(*tens) && (1..10).contains(ones) => Some(tens + ones),
        [Piece::Word(ones), Piece::And("und"), Piece::Word(tens)] if cat.has_german_und() && (1..10).contains(ones) && is_tens(*tens) => {
            Some(ones + tens)
        }
        _ => None,
    };
    let composes_hundreds = cat.has_english_tens() || cat.has_german_und();
    below.filter(|_| hundreds == 0 || composes_hundreds).map(|below| hundreds + below)
}

fn is_tens(value: i32) -> bool {
    (20..100).contains(&value) && value % 10 == 0
}

/// "einundzwanzig", "einhundert", "hundertzwanzig", "zweihundertfuenfundvierzig".
fn german_compound(token: &str) -> Option<i32> {
    let cat = catalog();
    let below_hundred = |word: &str| -> Option<i32> {
        if let Some(listed) = cat.number(word).filter(|value| *value < 100) {
            return Some(listed);
        }
        let (ones, tens) = word.split_once("und")?;
        let ones = if ones == "ein" { 1 } else { cat.number(ones).filter(|ones| (1..10).contains(ones))? };
        cat.number(tens).filter(|tens| is_tens(*tens)).map(|tens| ones + tens)
    };
    let Some((count, rest)) = token.split_once("hundert") else {
        return below_hundred(token);
    };
    let count = if count.is_empty() || count == "ein" { 1 } else { cat.number(count).filter(|count| (1..10).contains(count))? };
    let rest = rest.strip_prefix("und").unwrap_or(rest);
    let below = if rest.is_empty() { 0 } else { below_hundred(rest)? };
    Some(count * 100 + below)
}

fn is_room_index(tokens: &[String], i: usize, n: i32) -> bool {
    if !(1..=8).contains(&n) {
        return false;
    }
    tokens.get(i.saturating_sub(1)).is_some_and(|prev| catalog().room_index_nouns().contains(prev.as_str()))
}

pub fn first_number(tokens: &[String]) -> Option<i32> {
    extract_numbers(tokens).into_iter().next()
}

pub fn guess_numbered_action(tokens: &[String], last_climate: bool, last_cover: bool, last_fan: bool) -> Action {
    let cat = catalog();
    if last_climate || cat.any(tokens, cat.climate_nouns()) {
        return Action::SetTemp;
    }
    if last_cover || cat.any(tokens, cat.cover_nouns()) || crate::parse::action::is_garage_cover(tokens) {
        return Action::CoverSet;
    }
    if last_fan || cat.any(tokens, cat.fan_nouns()) {
        return Action::FanSpeed;
    }
    Action::SetLight
}

#[cfg(test)]
#[path = "numbers_tests.rs"]
mod tests;
