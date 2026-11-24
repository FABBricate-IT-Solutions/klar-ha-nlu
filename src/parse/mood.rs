//! Reads whether a clause is a positive, present-tense command. Anything else must not
//! change a device: a negation, a question, a report, a condition or a schedule.
//!
//! The word classes live on the language packs (`MoodWords`). A clause is read once per
//! bound pack and the results are joined, so a second language can only close a clause.

use crate::lang::{Catalog, MoodWords};
use crate::parse::normalize::{fold_latin, is_script_unit, is_word_char};

/// Question marks of every script. An inverted or Armenian mark sits inside the sentence.
const QUESTION_MARKS: &[char] = &['?', '？', '؟', '\u{037E}', '՞', '¿', '⸮'];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Reading {
    /// The clause forbids the action.
    pub negated: bool,
    /// A negation with nothing of its own to forbid: it takes back what was said before.
    pub bare_negation: bool,
    /// The action hangs on a condition.
    pub conditional: bool,
    /// The action is meant for another time.
    pub timed: bool,
    /// An hour after a clock word ("at 6", "um 6"). A time, unless it is the value the
    /// command sets ("set the heating at 21", "um 2 waermer").
    pub clock: Option<i32>,
    /// A question or a report: the clause asks about a state or tells one.
    pub asked: bool,
    /// The clause opens like a polite request ("can you"), so a question mark does not close it.
    pub requested: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Spot {
    /// Grammar: the word counts.
    Word,
    /// Part of a name or a payload. It holds its place in the clause but is not grammar.
    Name,
    /// Politeness or an opening frame. The clause starts after it.
    Skipped,
}

/// The tokens of one clause and which of them still count as grammar.
pub(crate) struct Clause<'a> {
    tokens: &'a [String],
    spots: Vec<Spot>,
}

impl<'a> Clause<'a> {
    pub(crate) fn new(tokens: &'a [String]) -> Self {
        Self { tokens, spots: vec![Spot::Word; tokens.len()] }
    }

    /// Every place where `words` stand in a row is a name: a device, a room, a taught phrase.
    pub(crate) fn mask_name(&mut self, words: &[String]) {
        if words.is_empty() || words.len() > self.tokens.len() {
            return;
        }
        for start in 0..=self.tokens.len() - words.len() {
            if self.tokens[start..start + words.len()] == *words {
                self.spots[start..start + words.len()].fill(Spot::Name);
            }
        }
    }

    /// Everything from the first word of a payload to its last one is content.
    pub(crate) fn mask_span(&mut self, words: &[String]) {
        let (Some(first), Some(last)) = (words.first(), words.last()) else {
            return;
        };
        let start = self.tokens.iter().position(|token| token == first);
        let end = self.tokens.iter().rposition(|token| token == last);
        if let (Some(start), Some(end)) = (start, end) {
            if start <= end {
                self.spots[start..=end].fill(Spot::Name);
            }
        }
    }

    pub(crate) fn read(&self, cat: &Catalog) -> Reading {
        let mut reading = Reading::default();
        for words in cat.moods() {
            let pack = self.read_pack(words, cat);
            reading.negated |= pack.negated;
            reading.bare_negation |= pack.bare_negation;
            reading.conditional |= pack.conditional;
            reading.timed |= pack.timed;
            reading.clock = reading.clock.or(pack.clock);
            reading.asked |= pack.asked;
            reading.requested |= pack.requested;
        }
        reading
    }

    fn read_pack(&self, words: &MoodWords, cat: &Catalog) -> Reading {
        let tokens = self.tokens;
        let mut spots = self.spots.clone();
        let mut framed = false;
        for at in 0..tokens.len() {
            if let Some(len) = phrase_at(tokens, &spots, at, words.courtesy) {
                spots[at..at + len].fill(Spot::Skipped);
            }
        }
        for at in 0..tokens.len() {
            if let Some(len) = phrase_at(tokens, &spots, at, words.ask_frames) {
                spots[at..at + len].fill(Spot::Skipped);
                framed = true;
            }
        }
        let mut requested = false;
        while let Some((at, len)) = start(&spots).and_then(|at| phrase_at(tokens, &spots, at, words.request_frames).map(|len| (at, len))) {
            spots[at..at + len].fill(Spot::Skipped);
            requested = true;
        }
        let first = start(&spots);
        let opens = |list: &[&str]| first.is_some_and(|at| phrase_at(tokens, &spots, at, list).is_some());
        let anywhere = |list: &[&str]| (0..tokens.len()).any(|at| phrase_at(tokens, &spots, at, list).is_some());
        let negations = negations(tokens, &spots, words, cat);
        let rest = spots.iter().filter(|spot| **spot != Spot::Skipped).count();
        Reading {
            negated: negations > 0,
            bare_negation: negations > 0 && rest < negations + 2,
            conditional: anywhere(words.conditions),
            timed: anywhere(words.time_words) || clock_mark(tokens, &spots, words, cat),
            clock: clock_lead(tokens, &spots, words, cat),
            asked: framed
                || opens(words.question_starts)
                || opens(words.question_openers)
                || opens(words.subjects)
                || anywhere(words.state_verbs)
                || anywhere(words.participles)
                || anywhere(words.question_particles),
            requested,
        }
    }
}

/// The sentence carries a question mark of any script.
pub(crate) fn question_mark(text: &str) -> bool {
    text.chars().any(|ch| QUESTION_MARKS.contains(&ch))
}

/// The sentence ends in a question particle of a bound pack. A particle in letters must be
/// a whole word; in a script written without spaces it is the tail of the sentence.
pub(crate) fn question_ending(text: &str, cat: &Catalog) -> bool {
    let folded = fold_latin(text);
    let raw = folded.trim();
    let bare = raw.trim_end_matches(|ch: char| !is_word_char(ch));
    cat.moods().flat_map(|words| words.question_endings.iter()).any(|ending| {
        if ending.is_empty() {
            return false;
        }
        if !ending.chars().any(is_word_char) {
            return raw.ends_with(ending);
        }
        bare.strip_suffix(ending)
            .is_some_and(|head| ending.chars().next().is_some_and(is_script_unit) || !head.chars().next_back().is_some_and(is_word_char))
    })
}

/// Where the clause starts once politeness and opening frames are set aside.
fn start(spots: &[Spot]) -> Option<usize> {
    spots.iter().position(|spot| *spot != Spot::Skipped)
}

/// Length of the longest entry of `list` that stands at `at`, every token still grammar.
fn phrase_at(tokens: &[String], spots: &[Spot], at: usize, list: &[&str]) -> Option<usize> {
    list.iter()
        .filter_map(|entry| {
            let mut len = 0;
            for word in entry.split(' ') {
                if tokens.get(at + len).map(String::as_str) != Some(word) || spots[at + len] != Spot::Word {
                    return None;
                }
                len += 1;
            }
            (len > 0).then_some(len)
        })
        .max()
}

/// Negation words that forbid. "but not the island" and "all lights, not in the bedroom"
/// take something out of the command instead; that is an exception, not a refusal.
fn negations(tokens: &[String], spots: &[Spot], words: &MoodWords, cat: &Catalog) -> usize {
    let all = tokens.iter().any(|token| cat.is_all(token));
    (0..tokens.len())
        .filter(|&at| {
            let Some(len) = phrase_at(tokens, spots, at, words.negation) else {
                return false;
            };
            let led = at > 0 && spots[at - 1] == Spot::Word && words.exception_leads.contains(&tokens[at - 1].as_str());
            let placed = all && tokens.get(at + len).is_some_and(|next| words.exception_places.contains(&next.as_str()));
            !led && !placed
        })
        .count()
}

fn number(token: &str, cat: &Catalog) -> Option<i32> {
    let digits = token.len() <= 4 && token.chars().all(|ch| ch.is_ascii_digit());
    if digits {
        token.parse().ok()
    } else {
        cat.number(token)
    }
}

/// "at 6", "um 6": an hour after a clock word. A unit makes it a value ("um 2 grad"),
/// and so does a number no clock shows ("at 72").
fn clock_lead(tokens: &[String], spots: &[Spot], words: &MoodWords, cat: &Catalog) -> Option<i32> {
    (0..tokens.len().saturating_sub(1)).find_map(|at| {
        if spots[at] != Spot::Word || spots[at + 1] != Spot::Word || !words.clock_leads.contains(&tokens[at].as_str()) {
            return None;
        }
        let (value, len) = spoken_number(&tokens[at + 1..], cat)?;
        let unit = tokens.get(at + 1 + len).is_some_and(|unit| words.measure_units.contains(&unit.as_str()));
        (!unit && (0..=24).contains(&value)).then_some(value)
    })
}

/// The number at the start of `tokens` and how many tokens it takes: "55", "fifty five".
fn spoken_number(tokens: &[String], cat: &Catalog) -> Option<(i32, usize)> {
    let tens = number(tokens.first()?, cat)?;
    let ones = tokens.get(1).and_then(|token| number(token, cat));
    match ones {
        Some(ones) if tens >= 20 && tens % 10 == 0 && (1..10).contains(&ones) => Some((tens + ones, 2)),
        _ => Some((tens, 1)),
    }
}

/// "10 pm": a number before a clock mark.
fn clock_mark(tokens: &[String], spots: &[Spot], words: &MoodWords, cat: &Catalog) -> bool {
    (1..tokens.len())
        .any(|at| spots[at] == Spot::Word && words.clock_marks.contains(&tokens[at].as_str()) && number(&tokens[at - 1], cat).is_some())
}
