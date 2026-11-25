//! Where a sentence takes something out of its command: "all lights except the hall
//! light", "the kitchen lights but not the counter", "alles im Wohnzimmer außer dem
//! Fernseher". The words come from the language packs; the sentence is read on the tokens
//! as spoken, before fillers are dropped.

use crate::home::expose::assist_visible;
use crate::lang::Catalog;
use crate::parse::normalize::tokenize;
use crate::types::HomeGraph;

/// Names of the home with more than one word, as the tokenizer writes them. A name can
/// hold a command word ("On Air", "Auf Sendung"), a conjunction or an exception word
/// ("Außer Haus"). Inside a name these words are part of the name and nothing else.
pub(crate) struct Names {
    sequences: Vec<Vec<String>>,
}

impl Names {
    pub(crate) fn of(home: &HomeGraph) -> Self {
        let entities = home
            .entities
            .iter()
            .filter(|entity| assist_visible(entity, home))
            .flat_map(|entity| std::iter::once(&entity.name).chain(&entity.aliases));
        let areas = home.areas.iter().flat_map(|area| std::iter::once(&area.name).chain(&area.aliases));
        let floors = home.floors.iter().flat_map(|floor| std::iter::once(&floor.name).chain(&floor.aliases));
        let mut sequences: Vec<Vec<String>> =
            entities.chain(areas).chain(floors).map(|label| tokenize(label)).filter(|words| words.len() > 1).collect();
        sequences.sort();
        sequences.dedup();
        Self { sequences }
    }

    /// Length of the longest name that starts at `at`, or 0.
    fn len_at(&self, tokens: &[String], at: usize) -> usize {
        self.sequences.iter().filter(|name| tokens[at..].starts_with(name)).map(Vec::len).max().unwrap_or(0)
    }

    /// For every token whether it is part of a name.
    fn cover(&self, tokens: &[String]) -> Vec<bool> {
        let mut covered = vec![false; tokens.len()];
        for at in 0..tokens.len() {
            let len = self.len_at(tokens, at);
            covered[at..at + len].fill(true);
        }
        covered
    }
}

/// A command and what the speaker took out of it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Exception {
    /// The sentence without the exception: what to do.
    pub command: Vec<String>,
    /// What to leave out, one entry per thing named: "the oven", "the coffee machine".
    pub items: Vec<Vec<String>>,
}

/// Why a sentence with an exception word cannot be read.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Unreadable {
    /// The exception names nothing, or nothing is left to do.
    Incomplete,
    /// It is not clear which side is the command and which is left out.
    WhichSide,
}

/// Reads the exception out of `tokens`. `None` means the sentence has none.
pub(crate) fn read(tokens: &[String], cat: &Catalog, names: &Names) -> Option<Result<Exception, Unreadable>> {
    let named = names.cover(tokens);
    let (at, len) = marker(tokens, &named, cat)?;
    Some(split(tokens, &named, at, len, cat))
}

/// The first exception word or phrase outside a name, and how many tokens it takes.
fn marker(tokens: &[String], named: &[bool], cat: &Catalog) -> Option<(usize, usize)> {
    let all = tokens.iter().any(|token| cat.is_all(token));
    (0..tokens.len()).filter(|at| !named[*at]).find_map(|at| {
        let len = phrase_len(&tokens[at..], cat.excepts()).or_else(|| led_negation(tokens, at, all, cat))?;
        Some((at, len))
    })
}

/// Length of the longest entry of `phrases` that `tokens` start with.
fn phrase_len<'a>(tokens: &[String], phrases: impl Iterator<Item = &'a str>) -> Option<usize> {
    phrases
        .filter_map(|phrase| {
            let words: Vec<&str> = phrase.split_whitespace().collect();
            let hit = !words.is_empty() && tokens.len() >= words.len() && tokens.iter().zip(&words).all(|(token, word)| token == word);
            hit.then_some(words.len())
        })
        .max()
}

/// "but not", "nur nicht": a negation right after an exception lead. And in a sentence
/// about all of something, a negation right before a place: "alle Lichter aus, nicht im Büro".
fn led_negation(tokens: &[String], at: usize, all: bool, cat: &Catalog) -> Option<usize> {
    cat.moods().find_map(|words| {
        if words.exception_leads.contains(&tokens[at].as_str()) {
            return phrase_len(&tokens[at + 1..], words.negation.iter().copied()).map(|len| len + 1);
        }
        let len = phrase_len(&tokens[at..], words.negation.iter().copied())?;
        let placed = all && tokens.get(at + len).is_some_and(|next| words.exception_places.contains(&next.as_str()));
        placed.then_some(len)
    })
}

fn split(tokens: &[String], named: &[bool], at: usize, len: usize, cat: &Catalog) -> Result<Exception, Unreadable> {
    let verb = |index: usize| !named[index] && cat.verb(&tokens[index]).is_some();
    let all = |index: usize| !named[index] && cat.is_all(&tokens[index]);
    let (before, after) = (0..at, at + len..tokens.len());
    // A group called "Alle Lichter" is a name and still says what the command is about.
    let scope_before = before.clone().any(|index| cat.verb(&tokens[index]).is_some() || cat.is_all(&tokens[index]));
    if !scope_before && !before.is_empty() {
        // "TV hariç tüm ışıkları kapat": the exception word follows what it leaves out.
        if !after.clone().any(all) {
            return Err(Unreadable::WhichSide);
        }
        return finish(tokens[after].to_vec(), vec![tokens[before].to_vec()]);
    }
    // The exception word leads what it leaves out. The list ends where a new command
    // starts: at "and" plus a verb, or, if no command was given yet, at the first verb.
    // "Turn" and "switch" are fillers in some packs; the verb is the word after them.
    let commands = |from: usize| (from..tokens.len()).find(|index| named[*index] || !cat.is_filler(&tokens[*index])).is_some_and(verb);
    let mut items: Vec<Vec<usize>> = vec![Vec::new()];
    let mut rest = tokens.len();
    for index in after.clone() {
        let conj = !named[index] && cat.is_conj(&tokens[index]);
        if (conj && commands(index + 1)) || (!scope_before && verb(index)) {
            rest = index;
            break;
        }
        if conj {
            items.push(Vec::new());
        } else if let Some(item) = items.last_mut() {
            item.push(index);
        }
    }
    // German puts the verb or its particle last: "… außer dem Flurlicht aus".
    let mut trailing = Vec::new();
    if rest == tokens.len() {
        while let Some(index) = items.last().and_then(|item| item.last().copied()) {
            if named[index] || !(cat.verb(&tokens[index]).is_some() || cat.is_particle(&tokens[index])) {
                break;
            }
            trailing.insert(0, index);
            if let Some(item) = items.last_mut() {
                item.pop();
            }
        }
    }
    let command = before.chain(trailing).chain(rest..tokens.len()).map(|index| tokens[index].clone()).collect();
    let items = items.into_iter().map(|item| item.into_iter().map(|index| tokens[index].clone()).collect()).collect();
    finish(command, items)
}

fn finish(command: Vec<String>, items: Vec<Vec<String>>) -> Result<Exception, Unreadable> {
    let items: Vec<Vec<String>> = items.into_iter().filter(|item| !item.is_empty()).collect();
    if command.is_empty() || items.is_empty() {
        return Err(Unreadable::Incomplete);
    }
    Ok(Exception { command, items })
}

#[cfg(test)]
#[path = "exception_tests.rs"]
mod tests;
